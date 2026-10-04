# Task 43 결과 — 되감기/빠른감기 HUD 배지 UI

판정: **성공.** 누적 호출 1/2. 약 33분(12:06~12:39).

## 변경 파일

- `crates/slot2-ui/src/hud.rs` — `pub enum TimeControl { Rewind, FastForward { speed: u32 } }`
  (Copy/Debug/PartialEq/Eq, App·input 타입 없음)와 `Hud::draw_time_control(&mut self, canvas,
  ctx, safe, Option<TimeControl>)`. 모듈 주석을 "모서리 시계/게이지 + 상단 중앙 시간 제어 배지"로
  갱신(DESIGN 167의 패널 좌표 규칙 유지). 기존 `draw` API·출력은 그대로.
- `crates/slot2-ui/tests/hud.rs` — 배지 테스트 4개 추가(기존 15개 불변).
- `assets/lang/en.ftl`, `ko.ftl` — `time-rewind = Rewind`/`되감기`,
  `time-fast-forward = { $speed }×`/`{ $speed }배속`.
- `crates/slot2-i18n/tests/i18n.rs` — 두 키의 en/ko 직접 정의 검증 1개.
- `tasks/43-time-control-hud-ui.worker-result.md` — 이 보고서.

`lib.rs` re-export는 필요하지 않아 손대지 않았다(`slot2_ui::hud::TimeControl`로 접근 가능).

## 최종 동작

- `None`은 아무것도 그리지 않고, Rewind/FastForward는 배지 하나만 그린다(clear·origin·시계·게이지·
  토스트·게임 프레임 없음 — App이 순서와 화면 허용 여부를 정한다).
- 배치는 패널 좌표: 위 `HUD_MARGIN`, 높이 `HUD_H`, `safe.panel_w` 기준 가로 중앙. `safe.x/y`를
  쓰지 않아 720x720에서도 안전 영역(120px 아래)을 따라 내려가지 않는다.
- 어두운 plate(기존 PLATE·PLATE_ALPHA) 하나에 밝은 INK 텍스트, 좌우 PAD_X 대칭 패딩, 밴드 중앙
  정렬. 문구는 그리고 나서가 아니라 **측정한 뒤** 배치한다.
- 속도는 enum에서 온다: `time-fast-forward`에 `$speed`를 넘겨 en `4×`/`8×`, ko `4배속`으로
  렌더링하며 그리기 함수에 4가 박혀 있지 않다.
- 캐시: 기존 face 캐시를 그대로 쓴다. 같은 배지를 반복해도 업로드 0, Rewind ↔ FastForward 전환은
  새 face 1회까지만 업로드하고 다시 정착한다. 이미지·SVG·타이머·텍스처 소유자·파일 접근 없음.

## 검증 (마지막 코드 변경 뒤, 명세 순서)

1. `cargo fmt --all -- --check` — 종료 **0**.
2. `cargo test -p slot2-ui -p slot2-i18n` — 종료 **0**, 실패 0. `tests/hud.rs` 19개(신규 4:
   None 무도, plate 1개 + 로컬라이즈 문구·패딩 대칭·clear 없음·speed 4/8, 3기기×2언어 중앙 정렬·
   HUD_MARGIN·패널 내부·정사각 패널에서 safe.y 불추종, 반복/전환 업로드 0), i18n 22개(신규 1),
   기존 slot2-ui 스위트 전부 통과.
3. `cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings` — 종료 **0**.

최종 검증 이후 코드 변경 없음. 커밋·푸시·실기·Pi·dist 빌드 없음.

## 남은 항목과 계약 의견

- App 배선은 계약대로 없다(Task 44 몫). 배지는 `slot2_ui::hud::TimeControl`만 알면 되고 App/입력
  타입에 의존하지 않는다.
- 계약 의견 없음. 프레임당 할당은 로컬라이즈 문자열 하나로, 기존 시계가 `clock_text`로 하는 것과
  같은 패턴이다.
