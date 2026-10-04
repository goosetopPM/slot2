# Task 37 결과 — 상태 스위처 앱 배선

판정: **성공.** 누적 시도 1/2. 약 16분(21:57~22:13).

## 변경 파일과 최종 동작

- `crates/slot2/src/app.rs` — `Screen::Switcher(InGameMenu)`(메뉴 행으로 돌아가기 위한 값만 보유)와
  App 필드 `state_switcher: StateSwitcher` 1개 추가. `Screen`은 Copy/Debug/Eq 그대로다.
  인게임 메뉴 A on SaveState → 세션 cart로 `Card::list_states` → `refresh` → 빈 목록이어도 진입.
  스위처에서 Left/Right는 이동(다른 방향키는 무동작), A는 선택 슬롯을 `load_numbered_state`로
  로드해 성공 시 `state-loaded` + Playing, 빈 목록이면 `states-empty`로 그대로, 실패면
  `state-load-failed`로 그대로, B/MENU는 같은 행의 인게임 메뉴로 복귀. `audio_paused`에 추가,
  벽지·HUD·인게임 메뉴를 그리지 않고 세션 프레임 뒤에 스위처만 덮는다. 모듈 주석 갱신.
- `crates/slot2-ui/src/state_switcher.rs` — `refresh(Vec<StateSlot>)` 추가. `new`와 같은
  정규화 함수(`numbered`)를 공유해 Resume 제외·번호 정렬·최대 번호 선택·빈 목록을 동일하게
  처리하고, 썸네일 캐시는 그대로 둔다(살아 있는 텍스처 id를 버리지 않는다).
- `crates/slot2-ui/tests/state_switcher.rs` — `refreshing_takes_the_new_slots_in_place_and_keeps_the_pictures_it_has`.
  갱신 후 새 최대 번호가 선택되고, 이전 방문의 그림은 다시 업로드되지 않으며(캐시 유지),
  빈 목록 갱신도 안전함을 확인.
- `crates/slot2/tests/state_switcher_app.rs` (신규) — 실기 코어 skip 패턴 4개 + 무코어 1개.
- `tasks/37-state-switcher-app-wiring.worker-result.md` — 이 보고서.

## 검증 (마지막 코드 변경 뒤, 명세 순서)

1. `cargo fmt --all -- --check` — 종료 **0**.
2. `cargo test -p slot2 -p slot2-ui` — 종료 **0**, 실패 0.
   - `app::tests` 16개(신규 3: 스위처 이동·메뉴 행 복귀, 빈 스위처 `states-empty` 유지,
     그리기에서 clear/벽지/메뉴 박스 없음).
   - `tests/state_switcher_app.rs` 5개: Save State 행 진입 + 최대 번호 선택(1번·Resume을
     손상 바이트 + 최신 mtime으로 두고 2번만 정상 → 로드 성공, 재저장 바이트 동일),
     코어 정지·세션 유지·싱크 요청 없음, 게임 프레임이 첫 마크이고 그 뒤에 스위처 dim,
     빈 카드 진입 + `states-empty` + 상태 파일 미생성 + 복귀 후 재개, 손상 바이트 →
     `state-load-failed` 유지 + 세션 계속 사용, 이후 저장이 다음 방문에서 최대 슬롯으로 노출.
   - HUD: 게이지를 물린 shelf에서는 캡슐이 있고 스위처에서는 없다(양성 대조 포함).
   - `tests/state_switcher.rs` 10개(신규 refresh 1개 포함), 기존 `slot2`/`slot2-ui` 전부 통과.
3. `cargo clippy -p slot2 -p slot2-ui --all-targets -- -D warnings` — 종료 **0**, 경고 0.

최종 검증 이후 코드 변경 없음. 커밋·푸시 없음. 워크스페이스 전체 테스트와 dist 빌드 미실행.

## 남은 항목과 계약 의견

- 로드+토스트 로직을 `load_numbered_state`로 묶어 스위처와 퀵 로드가 공유한다(중복 제거).
  그 과정에서 quick load의 로그 한 줄이 `slot2: quick load failed: …`에서
  `slot2: cannot load state {n}: …`로 바뀌었다. 토스트·화면·싱크 동작은 그대로다.
- 선택된 슬롯 번호를 App 밖에서 읽는 공개 접근자는 만들지 않았다. 통합 테스트는 "로드 성공
  여부 + 재저장 바이트 비교"로 최대 번호 선택을 증명하고, 세부 선택·이동은 app.rs 단위
  테스트가 스위처 필드를 직접 본다.
- 세션이 없는 상태에서 스위처 A는 조용히 아무것도 하지 않는다(스위처는 살아 있는 세션 위에서만
  열리므로 도달 불가 경로다).
- 삭제·undo·보존 정책·Resume 동작·새 메시지·새 제스처는 손대지 않았다.
