# Task 39 결과 — 스위처 상태 삭제와 30초 undo

판정: **성공.** 누적 시도 1/2. 약 62분(23:04~00:06).

## 변경 파일과 최종 동작

- `crates/slot2-ui/src/state_switcher.rs` — `refresh`가 새 목록에 없는 썸네일 경로의 캐시 항목을
  버리고(성공/실패 기억 모두) 그 텍스처를 `pending`에 담아 다음 `draw`나 `clear`에서 해제한다.
  `select_after_removing(n)`(위 번호 → 없으면 아래 최대 → 없으면 선택 없음)과
  `select_number(n) -> bool` 추가. `draw`가 `undo_available`을 받아 힌트 줄을 조합한다:
  카드 있으면 A load·X delete·B back, 없으면 B back, undo가 살아 있으면 Y undo를 덧붙임.
- `crates/slot2/src/app.rs` — `pub const STATE_UNDO_S: u64 = 30`, 전용 `PendingUndo{backup,expires}`
  하나(`App::undo`), `act(action, now)`로 동작 시각을 인자화(`feed`는 이벤트의 `at`, `tick`은
  그 tick의 `Instant`), `tick`이 액션 처리 **전에** 만료를 버린다(토스트 없음).
  - X: 선택 슬롯을 `Card::take_state`로 떼어내고 목록을 카드 기준으로 refresh한 뒤 삭제된 자리
    규칙으로 선택을 옮기고 `state-deleted`(번호)를 띄운다. 실패/사라진 상태는 로그 1줄 +
    refresh + `state-delete-failed`이며 이전 undo는 그대로 둔다.
  - Y: 살아 있는 백업만 `Card::restore_state`로 되돌린다. 성공하면 백업을 버리고 refresh +
    복원 번호 선택 + `state-restored`. 실패하면 백업과 **원래 만료 시각**을 그대로 두고
    `state-restore-failed`. 없거나 만료면 `undo-empty`.
  - `stop_session`이 undo를 지운다. A 로드·인게임 메뉴 왕복은 지우지 않는다. 모듈 주석 갱신.
- `assets/lang/en.ftl`, `ko.ftl` — `state-deleted`, `state-delete-failed`, `state-restored`,
  `state-restore-failed`, `undo-empty`, `hint-load`, `hint-delete`, `hint-undo` 8키.
- 테스트: `slot2-ui/tests/state_switcher.rs` 3개(선택 규칙, 캐시 방출·재사용·정확히 1회 해제,
  3기기×2언어×4힌트 조합의 안전 영역과 실제 힌트 내용), `slot2/src/app.rs` 6개(30초 경계
  29.999초/정확히 30초/이후, 실패한 undo의 백업·마감 유지와 재시도, 마감 연장 없음, 빈 선택 X,
  undo 유무에 따른 힌트), `slot2/tests/state_switcher_app.rs` 5개(실기 코어: 삭제·연속 삭제로
  이전 undo 대체·빈 화면·Y 복원과 원본 바이트·두 번째 Y, 위 번호 선택, 로드 후에도 살아 있는
  undo와 복원 슬롯 선택, 세션 정지 시 소멸, 실패한 삭제가 이전 undo 보존, 사라진 상태 후 목록
  동기화), `slot2-i18n/tests/i18n.rs` 1개(새 8키 en/ko 직접 정의와 BTN id).
- `tasks/39-state-switcher-delete-undo.worker-result.md` — 이 보고서.

## 검증 (마지막 코드 변경 뒤, 명세 순서)

1. `cargo fmt --all -- --check` — 종료 **0**.
2. `cargo test -p slot2 -p slot2-ui -p slot2-i18n` — 종료 **0**, 실패 0 (slot2 lib 22,
   `state_switcher_app` 10, `state_switcher` 13, i18n 20, `quick_state_app` 6, `ingame_menu_app` 2,
   `session` 17 등 전부 통과).
3. `cargo clippy -p slot2 -p slot2-ui -p slot2-i18n --all-targets -- -D warnings` — 종료 **0**.

최종 검증 이후 코드 변경 없음. 커밋·푸시 없음. 워크스페이스 전체 테스트·dist 빌드 미실행.

## 남은 항목과 계약 의견

- 마감 기준 시각은 "동작이 처리된 순간"이다: 탭이 `feed`에서 성립하면 이벤트의 `at`, `tick`에서
  처리되면 그 tick의 `Instant`. 런타임 루프는 한 프레임의 이벤트와 tick에 같은 `Instant`를
  넘기므로 계약의 "처리한 tick의 Instant"와 같고, 테스트는 명시적 `Instant`로 경계를 고정한다.
- `select_after_removing`의 "위가 없으면 아래 최대" 갈래는 지운 슬롯이 최대일 때 refresh 기본값과
  같은 결과라, 끝단 검증은 "위 번호" 갈래(실기)와 UI 단위 테스트(모든 갈래)로 나눠 증명했다.
- 계약 5의 두 실패 경로를 모두 실제로 밟았다: 사라진 상태(`Ok(None)`)와 삭제 실패(`Err`, 상태
  파일 자리에 디렉터리를 두어 유도). 실패 시 목록 동기화는 "A가 실제로 있는 슬롯을 로드한다"로
  관측한다.
- 힌트는 스위처가 그리므로 `draw`에 `undo_available` 인자를 추가했고, App이 `undo.is_some()`을
  넘긴다. 30초 카운트다운 표시·확인 대화상자·다단계 undo·휴지통은 계약대로 없다.
