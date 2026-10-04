# Task 48 최종 검토 — RetroArch 치트 파일 로더

## 판정

**통과** — 누적 호출 2/2.

`Card::read_cheats`는 `System/cheats/<PLAT>/<stem>.cht`를 읽어 공식 RetroArch 필드와 인용·
이스케이프·유니코드·복수 코드 파트를 보존한다. 없는 파일은 빈 목록으로 처리하고, 손상된 선언·
인덱스·필수 필드는 경로와 줄 또는 키 맥락이 있는 오류로 전체 실패한다.

2회차에서 선언된 `cheats` 값을 할당 크기나 반복 횟수로 사용하던 결함을 제거했다. 실제로 파싱한
owned entry만 정렬 맵에서 검증하고 그 개수만큼만 결과를 할당한다. 따라서 `4294967295` 같은 작은
손상 파일도 선언값에 비례한 메모리나 CPU를 요구하지 않는다. `handler`, `mem_search` 같은 미지원
indexed 메타데이터는 entry를 만들지 않으며 gap·범위·count 판정에도 영향을 주지 않는다.

## 검증 근거

- `cargo fmt --all -- --check` — 종료 0.
- `cargo test -p slot2-store` — 종료 0. `cheats` 15개를 포함해 0 failed.
- `cargo clippy -p slot2-store --all-targets -- -D warnings` — 종료 0.
- 기존 11개 테스트를 유지한 채 거대 count, u32 초과, 먼 unknown metadata, 범위 밖 owned field
  회귀 테스트를 추가했고, metadata가 다시 phantom entry를 만들면 테스트가 실패함을 역방향으로
  확인했다.

## 남은 비차단 사항

- count 선언이 없는 빈 파일은 계약대로 오류다. 생성기는 빈 목록도 `cheats = 0`으로 써야 한다.
- 검색 전용 메타데이터만 있고 `desc`/`code`가 없는 항목은 실행 가능한 치트가 아니므로 거부한다.
- 이번 태스크는 파일 로딩까지만 다뤘다. libretro Core에 reset/set을 전달하는 작업은 Task 49다.
