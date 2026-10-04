# Task 78 — Codex 최종 판정

- 판정: **통과**
- 누적 호출: **1/2**
- 작업자 보고서: `tasks/78-overlay-menu-app-wiring.worker-result.md`

## 검토 결과

- Display는 crop 없는 플랫폼에서 기존 다섯 행 뒤 Overlay를 추가한 6행/높이 316, crop 플랫폼에서 기존
  여섯 행 뒤 Overlay를 추가한 7행/높이 352이며 Overlay가 두 목록의 마지막이다.
- `Screen::Overlay`가 parent menu를 보존하고 Up/Down, B/MENU, pause와 game-bearing draw 범주에 정확히
  연결됐다.
- commit은 전체 settings에서 overlay만 바꿔 먼저 저장한다. 성공 뒤에만 launch와 공유하는
  `set_overlay_for` 경계로 source를 retarget하고, 실패하면 기존 card bytes와 source/texture를 유지한다.
- On은 지원 geometry의 source를 즉시 활성화하고 Off/Platform default는 다음 draw에서 texture를 한 번
  해제한다. 동일 source 재선택은 no-op이며 missing/corrupt/wrong-size asset은 선택을 보존하고 그림만
  생략한다.
- draw 순서는 game → runtime overlay → OverlayMenu → hold/toast이고 Session/core/game texture/audio/sink와
  기존 core switch/recovery/stop 수명을 바꾸지 않는다.
- 허용 목록에 없던 `display_menu_app.rs`의 수정은 새 마지막 행 때문에 낡아진 위치 단언 세 곳만 정확한
  새 위치로 고친 것이다. 완료 기준이 해당 suite 통과를 요구하고 단언을 약화하지 않아 범위 예외를
  수용했다.
- Display row 설명 중 Overlay도 같은 index를 유지한다는 잘못된 문장은 Codex가 동작 변경 없이
  “scale/Shader는 유지되고 Overlay는 두 목록 모두 마지막”으로 정정했다. `git diff --check`로 확인했다.

## 작업자 검증 증거

- `cargo fmt --all -- --check`: 종료 0
- Display/Overlay UI 집중 테스트: 14 passed / 0 failed / 0 ignored
- Display App + 기존 Overlay App + Overlay menu App: 61 passed / 0 failed / 0 ignored
- `slot2` + `slot2-ui` + `slot2-i18n`: 552 passed / 0 failed / 0 ignored
- core-dependent skip: 0
- device feature check: 종료 0
- clippy all-targets: 종료 0

## 남은 범위

- production `BUILT_IN_OVERLAYS`는 비어 있어 카드 override PNG가 없으면 실제 그림은 보이지 않는다.
- 실제 내장 overlay asset 추가와 실기 화질 확인은 후속 태스크다.
