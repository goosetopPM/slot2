# Task 56 — Codex 최종 판정

## 판정

**통과. 누적 호출 1/2.** 추가 작업자 호출은 필요하지 않다.

## 확인한 내용

- `validate_cheat(Fceumm, Nes, ...)`가 pinned adapter의 raw `AAAA:VV`, compare raw
  `AAAA?CC:VV`, 6/8자 NES Game Genie, 8-hex PAR 형식과 여섯 separator를 검사한다.
- ASCII 및 1023-byte 상한을 먼저 적용하고, separator run은 `strtok`처럼 건너뛰며 token이
  하나도 없거나 하나라도 잘못되면 전체를 거부한다.
- FCEUmm과 mGBA가 `EnabledEntriesOnly`, gpSP·SNES9x·Genesis Plus GX가
  `PassAllEntries`다. Session은 기존 공통 `cheat_delivery` 경로를 사용하므로 수정 없이 새 정책을
  초기 적용·토글·rollback에 사용한다.
- SNES9x/mGBA validator와 지원 matrix, 공통 empty/NUL 오류 순서가 유지됐다.
- 수정 파일은 허용 범위인 `quirks.rs`, `cheat_quirks.rs`, 작업자 보고서뿐이다.

## 작업자 검증 증거

- `cargo fmt --all -- --check` — 종료 0
- `cargo test -p slot2-retro --test cheat_quirks` — 21 passed / 0 failed
- `cargo test -p slot2 --test session` — 27 passed / 0 failed, mGBA 실코어, skip 없음
- `cargo clippy -p slot2-retro -p slot2 --all-targets -- -D warnings` — 종료 0
- 최종 검증 뒤 코드 변경 없음

Codex는 사용자 운영 규칙에 따라 위 테스트를 다시 실행하지 않고 보고서와 구현·테스트 코드를
대조했다.

## 비차단 정리 항목

- `fceumm_token_ok` 주석과 작업자 보고서는 8자리 Game Genie alphabet과 ASCII hex가 서로
  부분집합이 아니므로 “모호하지 않다”고 적었지만, 두 집합은 겹친다. 예를 들어 `AAAAAAAA`는 두
  형식 모두에 해당하며 pinned adapter는 Game Genie를 먼저 선택한다. validator는 양쪽 모두를
  허용하는 boolean 판정이라 실제 동작에는 영향이 없다. 다음 `quirks.rs` 수정에서 주석과 테스트
  설명을 “겹칠 수 있으며 core의 판정 순서는 Game Genie 우선”으로 고친다.
- Task55부터 남은 `Session::cheats()`의 “core가 보유한 목록” 설명도 실제 desired/file-order 전체
  목록을 뜻하도록 다음 Session 수정 때 바로잡는다.

## 다음 방향

D-21의 아직 `Unchecked`인 코어는 gpSP와 Genesis Plus GX다. 다음 태스크는 범위가 작은 gpSP GBA
validator를 pinned parser 기준으로 추가하고, 위 FCEUmm 주석 정리를 함께 수행하는 것이 적절하다.
