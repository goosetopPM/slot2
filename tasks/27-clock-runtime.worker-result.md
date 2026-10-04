# Task 27 결과 보고서 — 시계 런타임 오프셋 기반

**성공.**

## 마지막 테스트 결과

- `cargo test -p slot2-platform`: **EXIT 0** — 유닛 6 passed, battery 15 passed, clock_offset 2 passed, doc-tests 0 passed / **0 failed**.
- `cargo fmt --all -- --check`: **EXIT 0** (첫 실행에서 clock_offset.rs 개행 1건 지적 → 수정 후 통과).

## 변경 파일

- `crates/slot2-platform/src/clock.rs` — `OnceLock` 제거, `AtomicI32` 기반 `utc_offset_min()`, 신규 `set_utc_offset_min(i32) -> Result<(), i32>`, 신규 순수 helper `hud_local(utc, offset)`. 센티넬 `UNSET = i32::MIN` (유효 범위 밖이라 실제 오프셋과 충돌 불가). 초기화는 compare_exchange라 setter와 경합해도 환경값이 setter 결과를 덮지 않음.
- `crates/slot2-platform/tests/clock_offset.rs` (신규) — 독립 통합 바이너리. setter 즉시 반영, 범위 밖 거부(OFFSET_MIN-1, OFFSET_MAX+1, i32 최솟/최댓값), 경계값 허용, UTC 판정 경계(SET_AFTER ±1초를 +14/-12 오프셋에서) 검증. 기존 테스트 미삭제·미약화.
- `crates/slot2/src/app.rs` — HUD: `utc_now()` 한 번 샘플 후 `hud_local` 로 is_set 판정 및 오프셋 적용. 기존의 "시프트된 값으로 is_set" 판정 제거.
- `crates/slot2/src/main.rs` — 부팅 배너에 `utc_offset_min={} epoch={utc}` 추가. 의존성 추가 없음.

## 계약 판정

틀린 계약 없음. 요구사항 5항(범위·경합·경계·전역 상태 격리) 모두 충족.

## 비고

- 커밋·푸시 안 함. 기존 미커밋 문서(docs/*)와 tasks/24~26 파일 보존 확인.
- 실기 확인 필요 항목: 없음(순수 소프트웨어 변경).
