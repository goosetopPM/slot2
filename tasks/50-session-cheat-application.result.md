# Task 50 최종 검토 — Session 치트 로드·즉시 재적용

## 판정

**통과** — 누적 호출 1/2.

`Session::start`가 카드의 치트 파일을 한 번 읽고, Core가 게임과 save RAM을 받은 뒤 첫 프레임 전에
reset과 ordered 전체 적용을 수행한다. disabled 항목도 파일 index 그대로 전달한다. 읽기·파싱
오류는 `Error::Store`, C 문자열 준비 오류는 `Error::Retro`로 시작을 실패시키며 적용 도중 오류가
나면 Core를 다시 reset해 부분 Session을 반환하지 않는다.

Session은 적용된 `Vec<Cheat>`를 소유하고 읽기 전용 `cheats()`와 세션 범위
`set_cheat_enabled()`를 제공한다. 유효한 토글은 후보 목록 전체를 재적용한 뒤에만 메모리 상태를
갱신한다. 범위 밖 index와 같은 값 요청의 동작도 명확하며 `.cht` 원본은 수정하지 않는다.

## 검증 근거

- `cargo fmt --all -- --check` — 종료 0.
- `cargo test -p slot2 --test session` — 23 passed, 0 failed, 종료 0.
- `cargo clippy -p slot2 --all-targets -- -D warnings` — 종료 0.
- mGBA 실코어 환경에서 신규 테스트가 skip 없이 실행됐다.
- 빈 목록, ordered 필드 보존, 첫 프레임 전 상태, disabled 포함 실행, on/off 후 실행, 파일 byte 불변,
  새 Session의 파일 기본값 복귀, 범위 밖 index, 내부 NUL 시작 실패, 손상 파일의 Store 오류를
  확인했다.
- Codex가 관련 Session 구현과 테스트 diff를 대조했다. 지정 범위의 `git diff --check`에는 새 공백
  오류가 없다.

## 남은 비차단 사항

- 성공한 Session의 기존 목록은 이미 C 문자열 변환을 통과했으므로 현재 API로 토글 재적용 실패와
  rollback 실패를 실제 유도할 수 없다. 해당 방어 분기는 코드 구조로만 검토됐다.
- 코어가 특정 치트 형식을 실제로 받아들였는지는 void libretro API로 알 수 없다. D-21의 코어별
  사전 검증은 후속 태스크로 남아 있다.
