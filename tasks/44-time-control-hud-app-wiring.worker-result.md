# Task 44 결과 — 시간 제어 배지를 App에 배선

판정: **성공.** 누적 호출 1/2. 약 32분(14:18~14:50).

## 변경 파일

- `crates/slot2/src/app.rs` — private `fn time_control(&self) -> Option<TimeControl>` 하나 추가:
  `Screen::Playing` + 살아 있는 세션이 아니면 `None`, L2 눌림이면 `Rewind`, 아니면 R2 눌림이나
  `ff_latch`면 `FastForward { speed: FAST_FORWARD }`, 그 외 `None`. `run_frame`이 이 질의로
  속도/되감기를 정하고, `draw`가 게임 프레임 직후·App 소유의 일시적 오버레이(홀드 바·메뉴·스위처·
  토스트) 앞에서 `Hud::draw_time_control`을 호출한다. 모듈 그리기 주석에 배지 노출 규칙 추가.
  기존 `let safe = ctx.safe;` 중복 하나를 정리(값 동일).
- `crates/slot2/tests/time_controls_app.rs` — 배지 테스트 4개 추가(기존 3개 불변).
- `tasks/44-time-control-hud-app-wiring.worker-result.md` — 이 보고서.

`slot2-ui`, 언어, i18n, Session, 입력, gfx, 런타임 루프는 손대지 않았다. 공개 App API도 추가하지
않았다.

## 최종 동작

- 우선순위는 한 곳(`time_control`)에만 있다: L2 되감기 > R2 눌림/래치 빠른감기 > 없음. 그래서
  화면에 그려지는 배지와 코어가 실제로 받는 제어가 어긋날 수 없다.
- 배지는 게임 프레임 뒤에 그려져 게임 위에 보이고, App이 그리는 일시적 요소보다 아래에 있다.
  clear·게임 프레임 재업로드·origin 변경은 없다.
- Playing 이외의 화면(선반·스플래시·인서트/이젝트·파워·인게임 메뉴·스위처)과 세션 없는 Playing은
  배지가 없다. 래치가 켜져 있어도 메뉴/스위처에서는 숨고, Playing으로 돌아오면 다시 보인다.
- 물리 R2는 즉시 배지, 떼면 사라짐(래치 없을 때). 더블 탭은 래치되어 유지되고, 해제 더블 탭의
  마지막 release 뒤 사라진다. L2는 래치 배지를 되감기로 잠시 대체하고, 떼면 빠른감기 배지가 돌아온다.
- `draw`는 프레임 진행·되감기 소비·래치 토글/해제·세션 속도 변경·싱크 요청·화면 변경을 하지 않는다.

## 검증 (마지막 코드 변경 뒤, 명세 순서)

1. `cargo fmt --all -- --check` — 종료 **0**.
2. `cargo test -p slot2 -p slot2-ui` — 종료 **0**, 실패 0. `time_controls_app` 7개(신규 4: 버튼/래치
   추종과 게임 프레임 뒤 순서·warm 업로드 0, 메뉴가 배지를 숨기고 래치 유지, draw 무부작용
   (프레임 수·속도·되감기·화면·싱크 불변), 세션 없는 모든 화면 무배지), `tests/hud.rs` 19개,
   slot2 lib 28개와 기존 스위트 전부 통과.
3. `cargo clippy -p slot2 -p slot2-ui --all-targets -- -D warnings` — 종료 **0**.

최종 검증 이후 코드 변경 없음. 커밋·푸시·실기·Pi·dist 빌드 없음.

## 남은 항목과 계약 의견

- 실기 코어가 없는 환경에서는 배지 순서/부작용 테스트가 skip하고, 화면 무배지 테스트는 코어 없이
  돈다(여기서는 실제로 실행됐다). 배지는 Task 43 지오메트리(plate 높이·상단 여백·패널 중앙)와
  로컬라이즈 문구 폭으로 식별하므로 텍스처 id에 결합하지 않는다.
- 계약 의견 없음.
