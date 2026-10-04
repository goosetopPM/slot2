# Task 55 최종 검토 — mGBA disabled 치트 전달 수정

## 판정

**통과** — 누적 호출 1/2.

코어별 전달 정책은 `slot2-retro::quirks`에 있으며 mGBA만 enabled entry를 전달한다. Session은 named
core와 fallback을 모두 해결한 실제 dylib에서 `CoreId`를 얻어 보존하고, 초기 적용·양방향 토글·실패
rollback 모두 하나의 `apply_cheats` 경로와 같은 정책을 사용한다. mGBA에 보내는 index는 subset 위치로
압축하지 않고 원래 file index를 유지하며 Session 목록과 `.cht`는 바뀌지 않는다.

MIT GBA ROM과 실제 mGBA core의 serialize state 비교에서 disabled-only는 baseline과 같고 enabled는
달랐다. 첫 프레임 전 off→on은 enabled 시작과 같고 on→off는 baseline과 같아 UI 상태뿐 아니라 실제
core 적용까지 확인됐다.

## 검증 근거

- `cargo fmt --all -- --check` — 종료 0.
- `cargo test -p slot2-retro --test cheat_quirks` — 17 passed, 0 failed, 종료 0.
- `cargo test -p slot2 --test session` — 27 passed, 0 failed, 종료 0. mGBA 실코어 실행, skip 없음.
- `cargo clippy -p slot2-retro -p slot2 --all-targets -- -D warnings` — 종료 0.
- 작업자 최종 검증 뒤 코드 변경 없음.
- Codex가 delivery policy, 실제 dylib identity, 초기 적용·wanted·rollback 호출과 실코어 상태 비교
  테스트를 계약에 대조했다. Codex에서는 테스트를 중복 실행하지 않았다.

## 남은 비차단 사항

- `Session::cheats()`와 필드 주석의 “core가 보유한 목록” 표현은 mGBA에서 정확하지 않다. 반환값은
  disabled entry까지 포함한 Session의 desired/file-order 목록이고 실제 mGBA에는 enabled subset만
  전달된다. 다음 Session 수정에서 문구를 바로잡아야 한다.
- 알 수 없는 외부 core는 `core_id == None`이라 기존처럼 모든 entry를 전달한다. 호환성 우선이라는
  이번 계약과 일치하지만 그 core가 enabled flag를 무시하는지는 알 수 없다.
- gpSP·FCEUmm·Genesis Plus GX 문법은 여전히 `Unchecked`다.
