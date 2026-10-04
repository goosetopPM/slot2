# Task 41 결과 — 선반에서 이어하기 / 새로 시작

판정: **성공.** 누적 호출 **2/2**(시도 1 부분 완료 + 시도 2 보완). 시도 1 약 48분, 시도 2 약 10분.

## 동작 (최종)

- 선반에서 **Tap(A)=resume-if-present**, **Hold(A)=fresh**. 카트와 의도(`Launch`)는 제스처가
  받아들여진 순간 `PendingLaunch`로 고정되고, 좌석(SEATED_AT)에서 `start_launch`가 그 기록으로
  코어를 연다. 화면·사운드·싱크·거부 경로는 기존 그대로, `Screen`은 Copy/Debug/Eq 유지.
- Resume 로드는 세션이 Playing으로 노출되기 전에 끝난다. 파일이 없는 경합은 조용히 새 게임,
  읽을 수 없거나 손상된 Resume은 로그 1줄 + `resume-load-failed` 후 파일을 남긴다.
- fresh는 세션 시작이 성공한 뒤에만 Resume 상태와 썸네일을 지운다(실패는 로그만). 시작 실패 시
  Resume은 그대로 남고 기존 실패 경로를 따른다.
- Hold(A)로 시작한 발사는 release 전까지 코어 입력에서 A가 빠지고, release 뒤의 새 A 입력은
  정상 전달된다.
- `rescan`이 카트별 Resume 불린만 캐시해 ShelfView에 넘긴다. draw는 파일시스템을 보지 않는다.
  힌트는 Resume 없으면 `hint-play`, 있으면 `hint-resume`+`hint-new-game`(슬롯 위, 안전 영역 안,
  `draw`와 `draw_insert(seat=0)` 동일).

## 변경 파일

`crates/slot2/src/app.rs`, `crates/slot2/tests/resume_app.rs`(신규), `crates/slot2-ui/src/shelf_view.rs`,
`crates/slot2-ui/tests/shelf_resume.rs`(신규), `assets/lang/{en,ko}.ftl`, `crates/slot2-i18n/tests/i18n.rs`.

## 시도 2/2 보완 (검토 지적)

1. `resume_into`가 존재 확인에 `Card::read_state`를 쓰던 것을 경로의 `metadata` 결과로 바꿨다:
   `NotFound`만 조용히 넘기고, 경로가 있거나 조회할 수 없으면 `Session::load_state`를 시도해
   실패를 로그+토스트로 알린다. 상태를 두 번 읽지 않으며 `Card`/`Session`은 손대지 않았다.
2. fresh 테스트가 `resume.state`와 `.png`를 모두 만들어, 세션 시작 전 두 파일 보존과 시작 뒤
   두 파일 삭제를 함께 증명한다.
3. 직접 증거 추가: `app.rs` 단위 3개(Hold(A) 억제·release 해제·이후 A 전달, 제스처 시점 카트·
   의도 고정(선택 이동·rescan 뒤에도), rescan에서만 갱신되는 캐시와 카드 root가 사라져도
   파일시스템을 묻지 않는 draw), `resume_app.rs` 1개(경로는 있으나 상태로 읽을 수 없으면 실패
   토스트, 사라진 파일 경합은 침묵), `i18n.rs` 1개(`resume-load-failed`·`hint-resume`·
   `hint-new-game` en/ko 직접 정의).
4. 기존 통합 6개·선반 3개는 그대로 통과하며 약화하지 않았다.

## 검증 (마지막 코드 변경 뒤, 명세 순서)

1. `cargo fmt --all -- --check` — 종료 **0**.
2. `cargo test -p slot2 -p slot2-ui -p slot2-i18n` — 종료 **0**, 실패 0. slot2 lib 25개(신규 3),
   `resume_app` 7개(신규 1), `shelf_resume` 3개, i18n 21개(신규 1), 기존 insert/session/
   state-switcher/quick-state 전부 통과.
3. `cargo clippy -p slot2 -p slot2-ui -p slot2-i18n --all-targets -- -D warnings` — 종료 **0**.

최종 검증 이후 코드 변경 없음. 커밋·푸시·실기·Pi·dist 빌드 없음.

## 남은 항목과 계약 의견

- `resume_into`는 이제 경로 조회 1회 + 상태 읽기 1회만 한다. 경로 조회가 실패(권한 등)하면
  로드를 시도해 실패를 보고하므로, "조회 불가"도 조용히 넘어가지 않는다.
- 남은 계약 의견 없음. 시도 1이 보고한 간접 증거 항목은 모두 직접 테스트로 채웠다.
