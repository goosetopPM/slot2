# Task 74 — 게임별 오버레이 설정 저장 계약

현재 checkout에서 직접 작업한다. `slot2-store`의 게임별 ini에 D-11/D-23의 오버레이 선택을
안전하게 읽고 쓰는 계약을 추가한다. 이번 태스크는 **store 계약만** 다룬다. PNG 탐색·decode,
geometry별 asset 선택, 렌더링, Display 메뉴와 App/Session 배선은 후속 태스크로 분리한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\74-game-overlay-settings-store.md`
- `C:\SLOT2\tasks\66-game-shader-settings-store.result.md`
- `C:\SLOT2\tasks\73-overscan-menu-app-wiring.result.md`
- `C:\SLOT2\crates\slot2-store\src\settings.rs`
- `C:\SLOT2\crates\slot2-store\src\lib.rs`의 export 부분만
- `C:\SLOT2\crates\slot2-store\tests\card.rs`의 settings 테스트 부분만
- `C:\SLOT2\crates\slot2-store\tests\shader_settings.rs`의 safe-write 패턴만
- `C:\SLOT2\docs\DECISIONS.md`의 D-11과 D-23만
- `C:\SLOT2\docs\DESIGN.md`의 오버레이 경로·표시 파이프라인·게임별 ini 부분만

직접 관련된 `Ini`, `Card::read_settings`와 `write_settings` 구현만 추가로 읽는다. App, Session,
UI, gfx renderer, 실제 PNG asset, 워커 로그와 저장소 이력은 읽지 않는다.

## 현재 계약과 결정

- D-11: 오버레이는 플랫폼×geometry별 PNG이며 기본값은 없음이다. 내장 경로는
  `assets/overlays/<platform>/<geometry>.png`, 카드 덮어쓰기 경로는 `System/Overlays/`다.
- D-23: 화면 설정은 전역 → 플랫폼 기본 → 게임별 계층을 따르고 게임별 ini에 저장된다.
- 이번 태스크는 **그림의 위치나 존재 여부를 판단하지 않는다.** 저장 계약은 사용자의 의도만
  보존한다. `overlay = on`인데 해당 geometry의 PNG가 없는 경우 후속 loader가 안전하게 아무것도
  그리지 않아야 하며, store가 그 선택을 지우거나 오류로 바꾸면 안 된다.
- 따라서 디스크에는 세 의미가 필요하다.
  - key 부재 (`None`): 플랫폼 기본값 상속. 현재 기본값이 없음이어도 명시적 Off와 합치지 않는다.
  - `overlay = on` (`Some(true)`): 이 게임에서 플랫폼×geometry 오버레이 사용 요청.
  - `overlay = off` (`Some(false)`): 이 게임에서 오버레이를 명시적으로 끔.
- 하나의 플랫폼×geometry에 하나의 PNG라는 D-11 계약이므로 이번 버전에는 이름·경로 문자열이나
  preset enum을 저장하지 않는다.
- `slot2-store`가 안정적인 카드 표기만 소유한다. gfx/UI/platform 타입에 의존하지 않는다.
- 기존 safe-write 계약을 유지한다. unreadable 원본을 빈 ini로 간주해 덮어쓰면 안 되며, 이 버전이
  모르는 key는 보존해야 한다.

## 구현 계약

### 1. `GameSettings` 확장

- `GameSettings`에 `pub overlay: Option<bool>`을 추가한다.
- 소유 key는 정확히 `overlay`다. `GameSettings::KEY_OVERLAY`를 추가한다.
- `from_ini`는 기존 bool parser와 같은 표기를 사용한다.
  - true: `on`, `true`, `yes`, `1`
  - false: `off`, `false`, `no`, `0`
  - 앞뒤 공백과 ASCII 대소문자 차이는 허용한다.
  - key 부재, 빈 값, 알 수 없는 값은 `None`이다.
- `apply_to`는 `Some(true)`를 canonical `overlay = on`, `Some(false)`를 canonical
  `overlay = off`로 쓰고 `None`이면 overlay key만 제거한다.
- `GameSettings::default()`와 `is_default()`는 overlay가 `None`일 때만 이 필드가 기본인 것으로 본다.
- 기존 core, scale, overscan, rewind, shader의 공개 의미와 디스크 표기를 바꾸지 않는다.
- overlay 전용 Card API, 별도 ini, generic setting framework를 만들지 않는다. 기존
  `Card::read_settings`/`write_settings` 경로를 그대로 사용한다.

### 2. 안전한 쓰기와 호환성

- overlay 변경은 기존 다섯 setting과 unknown/future key를 정확히 보존해야 한다.
- 다른 setting 변경도 이미 저장된 overlay를 보존해야 한다.
- overlay를 `None`으로 돌리면 overlay key만 제거한다. 다른 key가 남으면 파일을 유지하고, 완전히
  비면 기존 계약대로 파일을 제거한다.
- existing file이 invalid UTF-8이거나 settings path가 directory이면 overlay 설정·해제 모두 error를
  반환하고 원본 bytes/path를 보존한다.
- read의 invalid overlay 값은 `None`으로 해석하되 파일 원문을 고치지 않는다.
- 기존 full `GameSettings` literal은 새 필드를 명시하거나 `..Default::default()`를 사용하도록
  컴파일에 필요한 최소 수정만 한다. 특히 다음 세 위치의 좁은 literal 보정은 허용한다.
  - `crates/slot2-store/tests/card.rs`
  - `crates/slot2-store/tests/shader_settings.rs`
  - `crates/slot2/tests/display_menu_app.rs`
- 이 literal 보정에서 기존 테스트 의미·값·단언은 바꾸지 않는다.

## 테스트 계약

`crates/slot2-store/tests/overlay_settings.rs`를 새로 만들고 최소한 다음을 직접 검증한다.

- `GameSettings::default()`의 overlay가 `None`이고 `is_default()`가 true임
- key 부재 `None`, `Some(true)`, `Some(false)`의 Card write/read round trip과 세 상태 구분
- true/false 허용 표기, 공백·대소문자 입력을 모두 읽고 write는 canonical `on`/`off`만 사용함
- 빈 값과 unknown/typo 값은 `None`으로 읽히며 read가 파일 원문을 바꾸지 않음
- `None` write가 overlay key만 제거하고, 마지막 key였다면 파일도 제거함
- overlay 변경이 core/scale/overscan/rewind/shader와 unknown key를 보존함
- core·scale 또는 shader 변경이 저장된 overlay와 unknown key를 보존함
- invalid UTF-8 기존 파일에 overlay 설정과 해제를 시도하면 모두 error이고 exact bytes가 보존됨
- settings path가 directory일 때 overlay 설정과 해제가 모두 error이며 directory를 제거하지 않음
- 기존 store 테스트가 새 필드 추가 뒤에도 같은 계약으로 통과함

Windows에서 의미가 다른 permission bit로 failure를 만들지 않는다. invalid UTF-8과 directory path를
portable failure fixture로 사용한다. 테스트 전용 production 분기, sleep/mtime 의존 검증, PNG fixture,
가짜 asset resolver는 추가하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-store\src\settings.rs`
- `C:\SLOT2\crates\slot2-store\tests\overlay_settings.rs` (신규)
- `C:\SLOT2\crates\slot2-store\tests\card.rs` (새 필드 literal의 좁은 수정만)
- `C:\SLOT2\crates\slot2-store\tests\shader_settings.rs` (새 필드 literal의 좁은 수정만)
- `C:\SLOT2\crates\slot2\tests\display_menu_app.rs` (새 필드 literal의 좁은 수정만)
- `C:\SLOT2\tasks\74-game-overlay-settings-store.worker-result.md`

`lib.rs`는 `GameSettings`가 이미 export되어 있어 변경하지 않는다. 다른 production 파일 변경은
허용하지 않는다. 구현이 불가능하다고 판단되면 범위를 넓히지 말고 보고서에 이유를 적어 실패로 둔다.
관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- 플랫폼별 overlay 기본값 타입이나 registry 변경
- `assets/overlays/` PNG 제작·추가, 카드 `System/Overlays/` 탐색 또는 asset precedence 결정
- PNG decode, texture upload/free, renderer draw order와 Session/App 배선
- Display/Overlay 메뉴, 번역, 즉시 미리보기와 저장 실패 toast
- `slot2-gfx`, `slot2-ui`, `slot2-retro`, `slot2-platform` production 변경
- `build/dist-device.ps1`과 배포 산출물 변경
- 전체 workspace 테스트, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-store
cargo check -p slot2 --tests
cargo clippy -p slot2-store --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test나
device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\74-game-overlay-settings-store.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- overlay 세 상태와 canonical 표기 결과
- safe write, known/unknown key 보존, 기본 복귀와 unreadable 원본 보존 결과
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- PNG loader/render/UI는 구현하지 않았다는 명시
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
