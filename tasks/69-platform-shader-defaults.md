# Task 69 — 플랫폼별 셰이더 기본값

현재 checkout에서 직접 작업한다. 플랫폼 registry에 셰이더 기본값을 안정적인 의미 계약으로 추가하고,
Task68의 Session이 게임별 `shader` key가 없을 때만 그 기본값을 적용하게 한다. 이번 태스크는 **registry와
Session 해석 경계만** 다룬다. Display 메뉴 UI와 App의 즉시 변경·저장은 후속 태스크로 분리한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\69-platform-shader-defaults.md`
- `C:\SLOT2\tasks\66-game-shader-settings-store.result.md`
- `C:\SLOT2\tasks\68-session-shader-routing.result.md`
- `C:\SLOT2\crates\slot2-retro\src\registry.rs` 전체
- `C:\SLOT2\crates\slot2-retro\src\lib.rs`의 registry re-export 주변만
- `C:\SLOT2\crates\slot2-retro\tests\registry.rs` 전체
- `C:\SLOT2\crates\slot2\src\session.rs`의 platform 변환, `Session::open`, shader helper와
  accessor/setter, draw 주변만
- `C:\SLOT2\crates\slot2\tests\session.rs`의 platform/settings/shader fixture와 테스트 주변만
- `C:\SLOT2\docs\DESIGN.md`의 shader preset 기본값과 game settings 레이아웃 문장만

직접 필요한 test helper만 추가로 읽는다. App, UI, shader GLSL source, 다른 태스크·워커 로그와 저장소
이력은 읽지 않는다.

## 현재 계약과 확정 기본값

- store의 `GameSettings::shader`는 `None`이면 플랫폼 기본값 상속, `Some(Off)`이면 명시적 끄기,
  나머지 네 preset이면 해당 효과를 명시적으로 선택한다.
- `docs/DESIGN.md`가 정한 기본값은 휴대기 `Lcd3x`, 거치기 `ZfastCrt`다. 현재 지원 플랫폼에 적용하면
  정확히 다음과 같다.
  - GB, GBC, GBA → `Lcd3x`
  - NES, SNES, MD, SMS → `ZfastCrt`
- 기본값은 platform의 속성이다. 선택한 core, 화면 배율, overscan이나 실행 기기의 panel geometry에 따라
  달라지지 않는다.
- explicit Off는 어떤 platform에서도 plain draw다. key 부재와 Off를 합치거나 settings 파일에
  계산된 기본값을 써 넣지 않는다.
- 특정 shader program을 실제 GL driver가 지원하지 않을 때의 plain fallback은 Task67의 Canvas 계약이다.
  Session의 원하는 runtime effect 자체를 fallback 결과로 바꾸지 않는다.

## 구현 계약

### 1. retro registry가 소유하는 의미 타입

- `slot2-retro`에 public `PlatformShader` enum을 추가한다. variant는 정확히
  `SharpBilinear`, `Lcd3x`, `ZfastCrt`, `Scanline` 네 개이며 최소한
  `Clone, Copy, Debug, PartialEq, Eq, Hash`를 지원한다.
- `Off` variant를 넣지 않는다. registry의 모든 지원 platform에는 실제 기본 effect가 있고, 게임별
  plain 선택은 store의 explicit Off가 담당한다.
- `PlatformDef`에 public `shader_default: PlatformShader` 필드를 추가하고 `PLATFORMS`의 일곱 항목을 위
  표대로 채운다. platform lookup이나 기존 field 의미와 순서는 바꾸지 않는다.
- `PlatformShader`를 crate root에서 기존 registry 타입들과 같은 방식으로 re-export한다.
- `slot2-retro`에서 store나 gfx를 의존하지 않는다. 문자열, store enum 또는 gfx enum을 registry 계약으로
  사용하지 않는다.
- registry 상단의 셰이더가 미래 단계라는 낡은 설명은 현재 사실에 맞게 좁게 고친다.

### 2. Session의 단일 해석 경계

- Task68의 shader 변환 helper가 `slot2_retro::Platform`과 `Option<slot2_store::ShaderPreset>`을 받아
  최종 `Option<slot2_gfx::ShaderEffect>`를 반환하게 확장한다. 별도 중복 변환 helper를 만들지 않는다.
- 매핑은 다음 우선순위를 exhaustive match로 표현한다.
  1. `Some(Off)` → `None`
  2. `Some(SharpBilinear/Lcd3x/ZfastCrt/Scanline)` → 같은 이름의 gfx effect
  3. store 값 `None` → `slot2_retro::def(platform).shader_default`의 같은 이름 gfx effect
- enum 순서/index, 문자열 round-trip, platform 이름 비교로 변환하지 않는다. store, retro, gfx 중 어느
  enum에 variant가 추가돼도 compile 시 이 경계를 다시 검토하게 한다.
- `Session::open`은 이미 계산한 retro platform과 이미 한 번 읽은 settings를 위 helper에 전달한다.
  shader 때문에 platform 재탐지, settings 재읽기 또는 파일 쓰기를 추가하지 않는다.
- `start`와 `start_named`는 계속 같은 `open` 경로를 쓴다. runtime accessor/setter와 draw routing은
  Task68의 계약을 유지한다.

### 3. 문서의 구현 상태 정합성

- `docs/DESIGN.md`의 휴대기/거치기 기본값은 바꾸지 않는다.
- game settings 카드 레이아웃에서 shader를 아직 미래 기능이라고 한 문장만 현재 구현 상태에 맞게
  좁게 고친다. 다른 설계·결정 문구는 재작성하지 않는다.

## 테스트 계약

### registry 집중 테스트

`crates/slot2-retro/tests/registry.rs`에서 다음을 검증한다.

- GB/GBC/GBA의 `shader_default`가 정확히 `Lcd3x`
- NES/SNES/MD/SMS의 `shader_default`가 정확히 `ZfastCrt`
- 기존 일곱 platform coverage와 중복 없음 계약이 계속 통과함

### Session 집중 테스트

`crates/slot2/tests/session.rs`의 Task68 테스트를 새 의미에 맞게 고치고 보강한다. 단언을 약화하거나
기존 회귀 테스트를 지우지 않는다.

- pure helper에서 key 부재가 GB/GBC/GBA는 gfx `Lcd3x`, NES/SNES/MD/SMS는 gfx `ZfastCrt`가 됨
- explicit Off는 일곱 platform 모두 `None`이 됨
- 네 explicit preset이 platform 기본값보다 항상 우선함. 적어도 GBA에서 `ZfastCrt`, NES에서 `Lcd3x`처럼
  반대 계열 override를 포함하며 가능하면 모든 platform/preset 조합을 표 기반으로 검증함
- shader key가 없는 real GBA Session은 accessor가 `Some(Lcd3x)`이고 draw가 정확히 하나의
  `Op::ImageEffect`/`Lcd3x`를 사용함. launch/read만으로 settings 파일을 만들거나 카드 bytes를 바꾸지 않음
- explicit Off인 GBA Session은 계속 plain `Op::Image`를 사용하고 카드에는 Off가 그대로 남음
- explicit effect가 GBA 기본값을 덮어쓰며 기존 geometry, crop UV와 tint를 보존함
- raw invalid shader 값은 store에서 상속 의미인 `None`으로 읽혀 GBA의 `Lcd3x`가 되며, launch가 원본
  파일을 정정하거나 다시 쓰지 않음
- `start_named`도 같은 default/override 해석을 사용함
- runtime setter는 여전히 다음 draw만 바꾸며 카드 I/O나 core/frame/texture/audio 부작용을 만들지 않음
- 기존 scale, no-frame, texture reuse와 shader routing 테스트가 계속 통과함

real-core fixture가 명시된 기존 사유로 없으면 해당 테스트만 skip하고 정확한 수와 이유를 보고한다.
pure registry/mapping 테스트는 core 유무와 관계없이 반드시 실행한다. GPU/GL window와 실기를 열지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-retro\src\registry.rs`
- `C:\SLOT2\crates\slot2-retro\src\lib.rs`
- `C:\SLOT2\crates\slot2-retro\tests\registry.rs`
- `C:\SLOT2\crates\slot2\src\session.rs`
- `C:\SLOT2\crates\slot2\tests\session.rs`
- `C:\SLOT2\docs\DESIGN.md`의 위 두 shader 관련 문장만
- `C:\SLOT2\tasks\69-platform-shader-defaults.worker-result.md`

`Cargo.toml`과 lockfile을 포함한 다른 production/test 파일은 수정하지 않는다. 구현이 불가능하다고
판단되면 범위를 넓히지 말고 보고서에 이유를 적어 실패로 둔다. 관련 없는 미커밋 변경을 정리·복원·
재포맷하지 않는다.

## 범위 밖 및 금지

- store의 ini 문법·preset·safe-write 계약, gfx shader source/program/fallback 변경
- Display 메뉴 shader 선택 UI, 입력 처리, live apply, persistence, 실패 toast
- core별 또는 게임 DB별 추천값, 사용자 정의 shader, overlay/bezel 기능
- scale/overscan/rewind/core 선택과 Session start 실패 정책 변경
- 전체 workspace 테스트, GL 창, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-retro --test registry
cargo test -p slot2 --test session
cargo test -p slot2-retro -p slot2 --lib
cargo check -p slot2 --tests
cargo clippy -p slot2-retro -p slot2 --all-targets -- -D warnings
```

모두 종료 0이어야 한다. core-dependent test가 skip되면 정확한 개수와 이유를 보고한다. 검증 뒤 코드를
바꾸면 영향받는 명령부터 다시 실행한다. workspace test, GL test와 device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\69-platform-shader-defaults.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- `PlatformShader`, 일곱 platform의 정확한 기본값과 crate 의존 방향
- Session의 override/Off/상속 우선순위와 start/start_named 결과
- settings 무작성·원본 보존, draw routing과 runtime setter 무부작용 근거
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄과 core-dependent skip 수
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- App/UI 배선과 실기 화질이 범위 밖으로 남았다는 확인
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
