# Task 50 워커 결과 — Session 치트 로드·즉시 재적용

**성공. 누적 호출 1/2.** 지정 검증 3종 모두 종료 0. mGBA 실코어가 있는 환경에서 신규 테스트가
skip 없이 실제로 실행됐다.

## 시작 시 읽기·reset·ordered 전체 적용

- `Session::start`가 `card.read_cheats(cart)?`를 정확히 한 번 호출한다(코어 로드 전). 파일이
  없으면 빈 목록, 읽기·파싱 오류는 `Error::Store`로 시작 실패한다.
- 코어가 게임을 로드하고 save RAM을 복원한 뒤, 첫 `run_frame` 전에 `apply_cheats`로
  ① `reset_cheats()` 한 번 ② 파일 순서 `0..N`의 모든 항목에 `set_cheat(index, enabled, code)`
  한 번씩 적용한다. 파일 index를 그대로 libretro index로 쓰고 disabled 항목도 생략하지 않는다.
- 적용 중 오류(예: 코드 내부 NUL)면 곧바로 `reset_cheats()`하고 시작을 실패시킨다 — 부분 적용이
  남은 Session은 반환되지 않는다.
- 코드 문자열은 trim·분해·대소문자 변환 없이 그대로 전달한다.

## 공개 API (최종 형태)

- `pub fn cheats(&self) -> &[slot2_store::Cheat]` — 적용에 성공한 파일 순서 목록, 복사 없음.
- `pub fn set_cheat_enabled(&mut self, index: usize, enabled: bool) -> Result<(), Error>` —
  범위 밖이면 코어를 건드리기 전에 `Error::NoCheat(index)`, 같은 값이면 성공 no-op, 유효 변경은
  `reset → 전체 ordered set 재적용` 후에만 `cheats`를 갱신한다.
- `Error`에 최소 variant 2개 추가: `NoCheat(usize)`, `Cheat(String)`(재적용 실패 + 복구 실패 메시지
  동시 보존). 기존 `NoCore`/`Retro`/`Store` 의미는 그대로다.

## 오류 시 부분 적용 제거와 rollback

재적용 실패 시 `reset_cheats()` → 이전 목록 재적용 → 메모리 상태 유지 순서로 되돌리고 원래 오류를
반환한다. 복구까지 실패하면 `Error::Cheat("{원래 오류}; putting the previous cheats back also
failed: {복구 오류}")`로 두 원인을 모두 남긴다. 다만 이 rollback 경로는 실제로 실패를 유도할
수단이 없어(시작이 성공한 이상 같은 목록의 코드는 다시 받아들여진다) 테스트로는 덮지 못했고, 테스트
전용 hook를 넣지 않기로 한 계약에 따라 코드 구조로만 보장한다.

## 파일 불변·새 Session 초기화

토글은 세션 메모리와 실행 코어에만 적용한다. `.cht` bytes를 쓰지 않고 `cheatN_enable`도 고치지
않으며, 같은 카드로 새 Session을 시작하면 파일의 원래 enabled 값으로 돌아온다(테스트에서 start
전후 파일 bytes 동일과 두 번째 Session의 값 복귀를 확인).

## 테스트 (신규 6개, 실코어 실행)

`cheats()` 없음/빈 목록 정상 시작, 다중 항목의 설명·코드·enabled·순서와 `frames_run()==0`(첫 프레임
전) 확인, disabled 포함 목록으로 프레임 계속, 유효 index on/off 즉시 반영과 이후 프레임 실행,
토글 후 파일 bytes 불변 + 새 Session 파일 값 복귀, 범위 밖 index 오류와 목록·실행 유지, 코드 내부
NUL로 시작 실패(부분 Session 없음, 실패 후 정상 파일로 재시작 가능), 손상 파일 `Error::Store`
(없는 파일과 구분). 제품 코드에 테스트 전용 분기·hook는 추가하지 않았다.

## 검증 (최종 코드 상태, 명세 순서)

- `cargo fmt --all -- --check` → 종료 **0**.
- `cargo test -p slot2 --test session` → 종료 **0**, 마지막 결과 줄
  `test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 19.79s`.
- `cargo clippy -p slot2 --all-targets -- -D warnings` → 종료 **0**(경고·오류 없음).
- 최종 검증 후 코드 변경 없음.

## 생성·수정 파일

- `crates/slot2/src/session.rs` (+114/−6): 모듈 doc의 Cheats 절, `apply_cheats`, `Error` variant 2개,
  `Session.cheats` 필드, `start`의 읽기·적용, `cheats()`/`set_cheat_enabled`.
- `crates/slot2/tests/session.rs` (+238): 실코어 치트 테스트 6개.
- 신규 파일 없음. `lib.rs`, `slot2-store`, `slot2-retro`, `slot2-ui`, `app.rs`, 언어 리소스는 손대지 않았다.

## 계약 우려 / 남은 위험

`set_cheat_enabled`의 rollback·복구 실패 분기는 실패를 유도할 관찰 수단이 없어 실코어 테스트로
검증되지 않았다(구조와 코드 검토로만 보장). 소요 약 7분(상한 45분 내).
