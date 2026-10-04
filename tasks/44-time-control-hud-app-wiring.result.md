# Task 44 검토 결과 — 시간 제어 HUD App 연결

## 판정

**성공** — 누적 호출 1/2.

## 검토 내용

- `App::time_control()`이 `Playing`과 활성 세션을 전제로 시간 제어 상태를 한 곳에서 판정한다.
- 우선순위가 `L2 되감기 > R2 또는 빨리감기 latch > 일반 속도`로 구현됐다.
- `run_frame()`과 `draw()`가 같은 판정을 사용하므로 실행 속도와 HUD 표시가 어긋나지 않는다.
- 배지는 게임 프레임 뒤, 메뉴·스위처·토스트 같은 일시적 오버레이 앞에 합성된다.
- 메뉴·스위처와 비게임 화면에서는 배지만 숨기고 빨리감기 latch는 보존한다.
- R2 누름·해제, latch, L2의 일시적 우선 표시와 복귀, 세션 없음, warm redraw 무업로드,
  `draw()` 무부작용 회귀 테스트가 추가됐다.

## 작업자 검증 증거

- `cargo fmt --all -- --check`: 종료 코드 0
- `cargo test -p slot2 -p slot2-ui`: 종료 코드 0
- `cargo clippy -p slot2 -p slot2-ui --all-targets -- -D warnings`: 종료 코드 0
- 검증 뒤 코드 변경 없음

검토 과정에서는 충분한 작업자 증거가 있어 동일 명령을 다시 실행하지 않았다.

## 변경 파일

- `crates/slot2/src/app.rs`
- `crates/slot2/tests/time_controls_app.rs`
- `tasks/44-time-control-hud-app-wiring.worker-result.md`

## 후속

- M4의 시간 제어 HUD 배지 항목을 완료로 표시한다.
- 다음 태스크 지시서는 아직 작성하지 않았다.
