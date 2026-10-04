# Task 53 최종 검토 — SNES9x 치트 검증 기반

## 판정

**통과** — 누적 호출 1/2.

`CoreId::from_library_path`가 다섯 canonical core의 실제 라이브러리 파일명을 허용된 세 확장자와
ASCII 대소문자 차이까지 식별하고, 부분 이름·백업/버전 suffix·알 수 없는 확장자를 거부한다.
부모 경로는 판정에 관여하지 않으며 기존 registry 계약은 유지됐다.

새 순수 validator는 공통 empty/interior-NUL 오류를 먼저 처리한다. SNES9x에는 pinned adapter의
255-byte 입력 한계, 다섯 separator, PAR·address/value·Game Genie 세 token 형식이 적용된다.
나머지 네 core는 문법을 추측하지 않고 `Unchecked`를 반환하므로 "검증 성공"으로 오인되지 않는다.
Session과 App에는 아직 연결하지 않았다.

## 검증 근거

- `cargo fmt --all -- --check` — 종료 0.
- `cargo test -p slot2-retro --test registry` — 14 passed, 0 failed, 종료 0.
- `cargo test -p slot2-retro --test cheat_quirks` — 9 passed, 0 failed, 종료 0.
- `cargo clippy -p slot2-retro --all-targets -- -D warnings` — 종료 0.
- 작업자 최종 검증 뒤 코드 변경 없음.
- Codex가 `registry.rs`, `quirks.rs`, 두 집중 테스트와 crate root re-export를 계약에 대조했다.
  Codex에서는 테스트를 중복 실행하지 않았다.

## 남은 비차단 사항

- mGBA·gpSP·FCEUmm·Genesis Plus GX는 의도대로 아직 `Unchecked`다. 각 pinned parser의 실제 형식을
  확인해 validator를 추가해야 D-21 전체 검증을 Session에 연결할 수 있다.
- SNES9x validator도 코드가 특정 게임에서 실제 효과가 있는지는 판정하지 않는다. 문법과 안전한
  전달 경계만 검증하는 현재 역할이 맞다.
