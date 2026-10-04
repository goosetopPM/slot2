# Task 69 시도 2/2 — App 통합 테스트의 플랫폼 기본 셰이더 회귀 정리

현재 checkout에서 직접 작업한다. Task69 production 구현은 유지하고, GBA의 기본 game draw가
`Op::ImageEffect { effect: Lcd3x, .. }`가 된 사실 때문에 실패하는 기존 App 통합 테스트 8개만 고친다.
이것이 **마지막 호출 2/2**다. 실패해도 세 번째 시도를 하지 않는다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\69-platform-shader-defaults-attempt2.md`
- `C:\SLOT2\tasks\69-platform-shader-defaults.worker-result.md`
- 아래 수정 허용 테스트 파일에서 보고서가 지목한 실패 테스트와 그 바로 인접한 helper만
- `C:\SLOT2\crates\slot2-gfx\src\canvas.rs`의 `Op::Image`/`Op::ImageEffect` field 정의만

production 파일, 다른 테스트, 다른 태스크·로그와 저장소 이력은 읽지 않는다.

## 수정 계약

- settings가 없는 GBA game frame은 정확히
  `Op::ImageEffect { effect: ShaderEffect::Lcd3x, .. }`로 식별한다.
- game quad의 기존 geometry와 draw 순서 단언을 유지한다. `Image | ImageEffect`처럼 무엇이 와도
  허용하는 느슨한 단언으로 바꾸지 말고, 이번 fixture의 platform default가 Lcd3x임을 함께 단언한다.
- 메뉴, HUD, toast, thumbnail, text atlas 등 일반 `Op::Image`는 effect draw로 바꾸지 않는다. 특히
  quick-state의 작은 thumbnail/toast image와 time-control HUD image 식별은 기존 의미를 유지한다.
- 여러 테스트가 한 파일 안에서 같은 game-frame helper를 쓰면 그 helper만 좁게 고쳐도 된다. helper가
  UI image까지 수집한다면 전체를 무차별적으로 effect-aware하게 만들지 말고 game quad 조건과 effect를
  분리해 판별한다.
- production 코드, Task69 registry/Session 집중 테스트와 shader 기본값은 수정하지 않는다.
- 실패를 없애려고 settings에 explicit Off를 넣지 않는다. 그러면 플랫폼 기본값 통합을 검증하지 못한다.
- 테스트를 ignore하거나 조기 return, 조건부 skip, 단언 삭제·완화로 통과시키지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2\tests\cheat_menu_app.rs`
- `C:\SLOT2\crates\slot2\tests\core_picker_app.rs`
- `C:\SLOT2\crates\slot2\tests\device_menu_app.rs`
- `C:\SLOT2\crates\slot2\tests\display_menu_app.rs`
- `C:\SLOT2\crates\slot2\tests\ingame_menu_app.rs`
- `C:\SLOT2\crates\slot2\tests\quick_state_app.rs`
- `C:\SLOT2\crates\slot2\tests\state_switcher_app.rs`
- `C:\SLOT2\crates\slot2\tests\time_controls_app.rs`
- `C:\SLOT2\tasks\69-platform-shader-defaults.worker-result.md`

다른 파일은 수정하지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- production, registry, Session, store, gfx, UI 구현과 shader source 변경
- App의 Display shader 선택·저장 배선 구현
- 다른 기존 테스트의 기대값 정리
- workspace test, GL 창, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2 --test cheat_menu_app --test core_picker_app --test device_menu_app --test display_menu_app --test ingame_menu_app --test quick_state_app --test state_switcher_app --test time_controls_app
cargo test -p slot2 --tests --no-fail-fast
cargo check -p slot2 --tests
cargo clippy -p slot2 --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 두 test 명령의 passed/failed/ignored 합계와 core-dependent skip 수를 보고한다.
검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test, GL test와 device 배포는 실행하지
않는다.

## 결과 보고서

기존 `C:\SLOT2\tasks\69-platform-shader-defaults.worker-result.md`를 **누적 2/2 보고서**로 갱신한다.

- 최종 성공/실패와 누적 호출 2/2
- 여덟 실패 각각에서 game frame을 어떤 정확한 Lcd3x 단언으로 바꿨는지
- 일반 UI image/thumbnail/HUD 단언을 보존한 근거
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored와 skip 수
- 수정 파일 및 최종 검증 뒤 코드 변경 여부
- production 파일 무변경 확인
- 남은 위험 또는 계약이 틀려 보이는 부분 1줄
- 이번 호출 소요 시간과 누적 소요 시간

코드·로그 전문이나 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다. 실패해도
보고서를 갱신하고 더 시도하지 않는다.
