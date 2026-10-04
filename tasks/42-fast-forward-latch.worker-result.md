# Task 42 결과 — R2 빠른 감기 래치 배선

판정: **성공.** 누적 호출 **2/2** — 호출 1(01:53~02:17, 약 24분)에서 구현·검증을 끝냈고,
호출 2(08:12~08:2x, 약 10분)는 그 구현이 그대로 남아 있는지 확인하고 지정 검증만 다시 돌렸다.
호출 2에서 **코드 변경 없음**.

## 변경 파일 (호출 1에서 완료, 호출 2에서 불변)

- `crates/slot2/src/app.rs` — `ff_latch: bool` 필드, `(Screen::Playing, DoubleTap(R2)) if
  session.is_some()`에서 토글, `run_frame` 속도 = `R2 물리 눌림 || ff_latch`,
  `stop_session`에서 `ff_latch = false`, 모듈 주석에 Hold(R2)/DoubleTap(R2)/Hold(L2) 추가,
  단위 테스트 3개.
- `crates/slot2/tests/time_controls_app.rs` (신규, 02:12) — 실기 코어 시간 제어 테스트 3개.
- `tasks/42-fast-forward-latch.worker-result.md` — 이 보고서.

호출 2 확인: 위 항목이 모두 현재 트리에 그대로 있고(`git log` HEAD 미변경), 허용 파일 밖 변경도
없다. 45분 상한 전에 멈췄고 세 번째 호출은 하지 않는다.

## 최종 동작

- **순간**: R2를 누르면 즉시 speed 4, 떼면 1.
- **래치**: 게임 중 R2 더블 탭이면 ON — 떼도 speed 4. 다시 더블 탭이면 OFF(두 번째 누름 프레임까지
  빠를 수 있고 떼면 1). 게임 밖(선반·인서트/이젝트·파워·인게임 메뉴·스위처)의 더블 탭은 무시되고
  다음 세션을 무장하지 않는다.
- **되감기 우선**: L2를 누르는 동안은 코어를 전진시키지 않고 `rewind_step`만 한다. L2를 떼면
  래치가 살아 있으면 speed 4.
- **초기화**: `stop_session`이 래치를 지운다(메뉴홀 이젝트·인게임 메뉴 Eject·전원·전원 메뉴
  Restart/PowerOff). 메뉴·스위처 왕복은 래치를 유지하고 코어를 돌리지 않는다.
  `FAST_FORWARD = 4`, 오디오/캡핑, 되감기, 원시 버튼, 싱크, Resume, 세이브 스테이트, UI는 그대로.

## 검증 (호출 2, 마지막 코드 변경 뒤 명세 순서)

1. `cargo fmt --all -- --check` — 종료 **0**.
2. `cargo test -p slot2 -p slot2-input` — 종료 **0**, 실패 0. slot2 lib 28개, `time_controls_app`
   3개, `slot2-input` 12개를 포함해 모든 스위트 통과.
3. `cargo clippy -p slot2 -p slot2-input --all-targets -- -D warnings` — 종료 **0**.

검증 이후 코드 변경 없음. 커밋·푸시·실기·Pi·dist 빌드 없음.

## 남은 항목과 계약 의견

- 실기 코어가 없는 환경에서는 `time_controls_app`이 기존 방식대로 skip하고, 순수 래치·화면·초기화
  테스트는 코어 없이 돈다(여기서는 실제로 실행됐다).
- 계약 의견 없음. Task 43의 배지가 읽을 유효 빠른감기/되감기 상태는 `ff_latch`와
  `Session::speed()`로 이미 관측 가능하다.
