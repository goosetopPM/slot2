# Task 38 결과 — 되돌릴 수 있는 상태 삭제(store)

판정: **성공.** 누적 시도 1/2. 약 10분(22:32~22:42).

## 변경 파일과 최종 동작

- `crates/slot2-store/src/card.rs` — `StateBackup`(불투명, `kind()`만 공개)과 `Card::take_state`,
  `Card::restore_state` 추가. 모듈 주석에 두 API 한 줄 보강.
  - `take_state(&cart, Numbered(n))`: Resume이면 파일을 건드리기 전에 `Error::Invalid`.
    `.state`가 없으면 `Ok(None)`(고아 썸네일은 그대로 둔다). 상태 바이트와 선택적 썸네일
    **원본 바이트**를 먼저 읽고(디코드하지 않는다), 썸네일 → 상태 순서로 삭제한다. 상태 삭제가
    실패하면 이미 지운 썸네일을 `atomic_write`로 되돌리고 오류를 반환한다.
  - `restore_state(&StateBackup)`(소비하지 않음): 다른 카드 root면 쓰기 전에 `Error::Invalid`.
    대상 `.state` 또는 `.png`가 이미 있으면 거부(덮어쓰지 않음, 부분 수정 없음). 썸네일을 먼저
    `atomic_write`하고 상태를 마지막에 쓴다 — `.state`가 가시성 경계라 `list_states`는 썸네일이
    준비된 뒤에야 슬롯을 보여준다. 상태 쓰기가 실패하면 이번 시도가 만든 썸네일을 제거한다.
- `crates/slot2-store/src/lib.rs` — `StateBackup` re-export, 모듈 주석에 백업/복원 한 단락 추가.
- `crates/slot2-store/tests/state_undo.rs` (신규) — 계약 테스트 11개.
- `tasks/38-reversible-state-delete-store.worker-result.md` — 이 보고서.

기존 `delete_state`(영구 삭제), 상태 번호 규칙, write 포맷, Resume 동작, 공개 API는 그대로다.
`atomic.rs`, App/UI/입력/언어/매니페스트/문서는 손대지 않았다.

## 검증 (마지막 코드 변경 뒤, 명세 순서)

1. `cargo fmt --all -- --check` — 종료 **0**.
2. `cargo test -p slot2-store` — 종료 **0**, 실패 0. lib 1개, `tests/card.rs` 15개(기존 전부 통과),
   `tests/state_undo.rs` 11개:
   - 썸네일 있는 상태 take → 디스크·목록에서 사라짐 → restore가 상태 바이트와 PNG 원본 바이트를
     그대로 복원; 썸네일 없는 상태는 PNG를 만들지 않음; 비-PNG/손상 썸네일도 바이트 단위 왕복;
     없는 상태는 `Ok(None)`이며 고아 썸네일 유지; Resume 거부 + 두 파일 불변.
   - 다른 Card root로의 복원 거부(그쪽에 아무것도 쓰지 않음) 후 같은 백업으로 원래 카드에 복원 성공;
     대상 상태/썸네일이 있으면 거부하고 대상 불변, 방해물을 치우면 같은 백업으로 복원 성공;
     성공 후 재복원은 거부되고 바이트 불변.
   - 상태 쓰기 실패(임시 파일 자리에 디렉터리) 시 썸네일이 제거되고 백업은 재사용 가능.
   - 영구 `delete_state`는 여전히 영구적이고 번호는 재사용되지 않음.
3. `cargo clippy -p slot2-store --all-targets -- -D warnings` — 종료 **0**, 경고 0.

최종 검증 이후 코드 변경 없음. 커밋·푸시 없음. 워크스페이스 전체 테스트·dist 빌드 미실행.

## 남은 항목과 계약 의견

- `take_state`의 상태 삭제 실패 + 썸네일 롤백 경로는 코드에 있지만 테스트하지 않았다. 그 상황을
  결정적으로 만들려면 읽기 전용 속성 같은 권한 조작이 필요한데 계약이 그것에 의존하지 말라고 했다.
  복원 실패 시 썸네일 정리 경로는 임시 파일 자리를 디렉터리로 막아 실제로 검증했다.
- 계약 6을 문자 그대로 적용해, 백업에 썸네일이 없어도 대상 `.png`가 존재하면 복원을 거부한다
  (반쯤 남은 삭제의 고아 PNG 옆에 상태를 붙이지 않기 위해서다).
- 30초 타이머·버튼 배선·토스트·UI는 계약대로 이 태스크 범위 밖이며, 백업은 호출자가 들고 있는
  동안만 메모리에 존재한다(휴지통 디렉터리·정리 스캔 없음).
