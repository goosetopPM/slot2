# Task 54 최종 검토 — mGBA 플랫폼별 치트 validator

## 판정

**통과** — 누적 호출 1/2.

`validate_cheat`가 `(CoreId, Platform)`을 함께 받아 지원하지 않는 조합을 먼저 거부한다. 지원 관계는
`CoreId::supports_platform` 한 곳에 정의되어 gpSP의 GBA 대체 코어 관계도 빠지지 않는다.

mGBA는 GBA의 CodeBreaker·GameShark/PAR·32-bit VBA 단위와 GB/GBC의 GameShark·두 Game Genie·
CodeBreaker·VBA 단위를 각각 검증한다. adapter의 일곱 separator, 정확한 unit 소비, ASCII 제한을
반영하며 다른 세 core는 여전히 `Unchecked`다. 기존 SNES9x 형식과 255-byte 경계도 유지됐다.

## 검증 근거

- `cargo fmt --all -- --check` — 종료 0.
- `cargo test -p slot2-retro --test registry` — 15 passed, 0 failed, 종료 0.
- `cargo test -p slot2-retro --test cheat_quirks` — 15 passed, 0 failed, 종료 0.
- `cargo clippy -p slot2-retro --all-targets -- -D warnings` — 종료 0.
- 작업자 최종 검증 뒤 코드 변경 없음.
- Codex가 플랫폼 matrix, `quirks.rs`의 mGBA parser, 기존 SNES9x 분기와 집중 테스트를 계약에
  대조했다. Codex에서는 테스트를 중복 실행하지 않았다.

## 후속 필수 사항

- pinned mGBA adapter는 `retro_cheat_set`의 `index`와 `enabled`를 무시한다. 현재 Session은 disabled
  entry도 core에 전달하므로, mGBA 메뉴의 off 표시가 실제 비활성화를 보장하지 않는다.
- 다음 작업에서 실제 선택된 dylib를 `CoreId::from_library_path`로 식별하고, mGBA에는 enabled entry만
  reset 후 재전달해야 한다. 적용 실패 rollback도 같은 정책을 써야 한다.
- gpSP·FCEUmm·Genesis Plus GX 문법은 아직 `Unchecked`이며 D-21 완료 전에 pinned parser 검증이
  추가로 필요하다.
