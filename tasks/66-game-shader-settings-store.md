# Task 66 — 게임별 셰이더 설정 저장 계약

현재 checkout에서 직접 작업한다. `slot2-store`의 게임별 ini에 셰이더 선택을 안전하게 읽고 쓰는
계약을 추가한다. 이번 태스크는 **store 한 크레이트만** 다룬다. 플랫폼 기본값, 실제 GPU 셰이더,
Display 메뉴와 App/Session 배선은 후속 태스크로 분리한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\66-game-shader-settings-store.md`
- `C:\SLOT2\tasks\47-safe-game-settings-write.result.md`
- `C:\SLOT2\crates\slot2-store\src\settings.rs`
- `C:\SLOT2\crates\slot2-store\src\lib.rs`의 export 부분만
- `C:\SLOT2\crates\slot2-store\tests\card.rs`의 settings 테스트 부분만
- `C:\SLOT2\docs\DECISIONS.md`의 D-10과 D-23만
- `C:\SLOT2\docs\DESIGN.md`의 설정 계층과 shader preset 관련 부분만

직접 관련된 `Ini`, `Card::read_settings`와 `write_settings` 구현만 추가로 읽는다. App, Session,
UI, gfx renderer, platform registry, shader asset, 워커 로그와 저장소 이력은 읽지 않는다.

## 현재 계약과 결정

- 게임별 설정은 `System/games/<PLAT>/<stem>.ini`에 저장되며, 빠진 key는 플랫폼 기본값을 상속한다.
- D-10의 내장 preset은 LCD3x, sharp-bilinear, zfast-crt, scanline 네 종류다. 사용자는 게임별로
  셰이더를 명시적으로 끌 수도 있어야 한다.
- 따라서 `shader` key 부재와 `shader = none`은 서로 다르다.
  - key 부재: 플랫폼 기본값 상속
  - `shader = none`: 이 게임에서는 셰이더를 명시적으로 끔
- 아직 플랫폼별 기본 preset은 구현하지 않는다. 이 태스크는 그 기본값을 저장하거나 추측하지 않는다.
- `slot2-store`가 카드의 안정적인 파일 형식을 소유한다. gfx 크레이트 타입에 의존하지 않는다.
- 기존 safe-write 계약은 유지한다. unreadable 원본을 빈 ini로 취급해 덮어쓰면 안 되고, 이 버전이
  모르는 key는 보존해야 한다.

## 구현 계약

### 1. 공개 preset 타입과 디스크 표기

- `slot2_store::ShaderPreset`을 공개하고 `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`를 제공한다.
- variant는 정확히 `Off`, `SharpBilinear`, `Lcd3x`, `ZfastCrt`, `Scanline`이다.
- `as_str()`의 canonical 표기는 각각 다음과 같다.
  - `Off` → `none`
  - `SharpBilinear` → `sharp-bilinear`
  - `Lcd3x` → `lcd3x`
  - `ZfastCrt` → `zfast-crt`
  - `Scanline` → `scanline`
- `parse()`는 앞뒤 공백과 ASCII 대소문자 차이를 허용하고 위 canonical 표기를 해석한다.
- 수동 편집 호환을 위해 `off`, `sharp_bilinear`, `sharpbilinear`, `lcd-3x`, `zfast_crt`,
  `zfastcrt`, `scanlines`도 각각 명백한 variant로 받아도 된다. write는 언제나 canonical 표기만 쓴다.
- 빈 값과 알 수 없는 값은 `None`이다. typo가 게임 실행을 막거나 명시적 Off로 바뀌면 안 된다.

### 2. `GameSettings` 확장

- `GameSettings`에 `pub shader: Option<ShaderPreset>`을 추가한다.
- `shader == None`은 **플랫폼 기본값 상속**이다. `Some(ShaderPreset::Off)`와 합치지 않는다.
- 소유 key는 정확히 `shader`다. `GameSettings::KEY_SHADER`를 추가한다.
- `from_ini`는 shader를 위 계약으로 읽는다. 빠진 값, 빈 값, invalid 값은 모두 `None`이다.
- `apply_to`는 `Some(preset)`이면 canonical 값을 쓰고 `None`이면 shader key만 제거한다.
- `GameSettings::default()`와 `is_default()`는 shader가 `None`일 때만 이 필드가 기본인 것으로 본다.
- 기존 core, scale, overscan, rewind의 공개 의미와 디스크 표기를 바꾸지 않는다.

### 3. 안전한 쓰기와 호환성

- 기존 `Card::read_settings`와 `Card::write_settings` 경로를 그대로 사용한다. shader 전용 Card API,
  generic settings framework, 새 파일 형식을 만들지 않는다.
- shader 변경은 core/scale/overscan/rewind와 unknown/future key를 보존해야 한다.
- 다른 setting 변경도 이미 저장된 shader를 보존해야 한다.
- shader를 `None`으로 돌리면 shader key만 제거한다. 다른 key가 남으면 파일을 유지하고, 완전히 비면
  기존 계약대로 파일을 제거한다.
- existing file이 invalid UTF-8이거나 settings path가 directory이면 shader 설정과 해제 모두 error를
  반환하고 원본 bytes/path를 보존한다.
- 기존 `a_key_this_version_does_not_know_survives_a_save` 테스트는 이제 `shader`를 unknown key 예제로
  쓸 수 없다. `future_filter`처럼 실제로 소유하지 않는 key로 좁게 바꿔 같은 회귀 계약을 유지한다.

## 테스트 계약

`crates/slot2-store/tests/shader_settings.rs`를 새로 만들고 최소한 다음을 직접 검증한다.

- `GameSettings::default()`의 shader가 `None`이고 `is_default()`가 true임
- 다섯 variant의 `as_str` canonical 표기와 대소문자·공백 parse
- canonical 다섯 값의 Card write/read round trip
- key 부재의 `None`과 `shader = none`의 `Some(Off)`가 구분됨
- `Some(Off)`는 정확히 canonical `none`으로 저장됨
- 빈 값과 unknown/typo 값은 `None`으로 읽히며 read가 파일 원문을 바꾸지 않음
- `None` write가 shader key만 제거하고, 마지막 key였다면 파일도 제거함
- shader 변경이 기존 네 setting과 unknown key를 보존함
- core 또는 scale 변경이 기존 shader와 unknown key를 보존함
- invalid UTF-8 기존 파일에 shader 설정과 해제를 시도하면 모두 error이고 exact bytes가 보존됨
- settings path가 directory일 때 shader 설정과 해제가 모두 error이며 directory를 제거하지 않음
- `card.rs`의 기존 full `GameSettings` literal과 unknown-key 테스트가 새 필드 계약에 맞고 계속 통과함

Windows에서 의미가 다른 permission bit로 failure를 만들지 않는다. invalid UTF-8과 directory path를
portable failure fixture로 사용한다. 테스트를 통과시키기 위한 fixture 전용 production 분기나
sleep/mtime 의존 검증은 추가하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-store\src\settings.rs`
- `C:\SLOT2\crates\slot2-store\src\lib.rs`
- `C:\SLOT2\crates\slot2-store\tests\shader_settings.rs` (신규)
- `C:\SLOT2\crates\slot2-store\tests\card.rs` (새 필드와 unknown-key fixture의 좁은 수정만)
- `C:\SLOT2\tasks\66-game-shader-settings-store.worker-result.md`

다른 production 파일 변경은 허용하지 않는다. 계약 구현이 불가능하다고 판단되면 범위를 넓히지 말고
보고서에 이유를 적어 실패로 둔다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- 플랫폼별 shader 기본값 결정 또는 `PlatformDef` 변경
- shader source/asset 추가, compile, GPU renderer와 compositor 변경
- Display 메뉴·App·Session 배선, 즉시 미리보기, overlay 기능
- `slot2-gfx`, `slot2-ui`, `slot2`, `slot2-retro`, `slot2-platform` production 변경
- 사용자 GLSL, `.glslp`, `.slang` 지원
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

끝나기 전에 `C:\SLOT2\tasks\66-game-shader-settings-store.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- `ShaderPreset`, canonical 표기, 상속과 명시적 Off 구분 결과
- safe write, known/unknown key 보존, 기본 복귀와 unreadable 원본 보존 결과
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
