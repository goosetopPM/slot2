# Task 80 — Codex 최종 판정

- **판정:** 통과
- **누적 작업자 호출:** 1/2
- **작업자 보고서:** `tasks/80-global-timezone-settings-store.worker-result.md`

## 검토 결과

- `slot2-store`가 `utc_offset_minutes`의 기본값 0과 범위 `-720..=840`을 공개 상수와 독립 Card API로
  제공한다.
- `read_utc_offset_minutes()`는 signed decimal과 앞뒤 공백을 허용하고, 누락·범위 밖·parse 실패·읽기
  실패를 UTC 0으로 처리하며 원본을 변경하지 않는다.
- `write_utc_offset_minutes()`는 범위 밖 입력을 파일 접근 전에 거부하고, 정상 입력은 기존 atomic ini
  경로로 저장한다. 기본 0은 소유 key만 제거하며 빈 파일만 삭제한다.
- 시간대 write는 정상·비정상 volume과 unknown key를 보존하고, 기존 volume write도 정상·비정상
  시간대 값을 보존한다.
- invalid UTF-8과 directory fixture에서 default/non-default write가 모두 실패하고 원본 bytes/path가
  유지된다.
- 기존 `GlobalSettings { volume }` shape와 volume default/range/signature는 바뀌지 않았다.
- store 구현은 환경변수, system clock과 mtime에 접근하지 않는다. D-25의 표시 오프셋 경계를 지킨다.
- 수정 범위는 명세의 허용 파일 안이며 관련 변경의 `git diff --check`를 통과했다.

## 작업자 검증 근거

- 집중 테스트: **23 passed / 0 failed / 0 ignored**
- `cargo test -p slot2-store`: **101 passed / 0 failed / 0 ignored**
- `cargo fmt --all -- --check`: 종료 코드 **0**
- `cargo clippy -p slot2-store --all-targets -- -D warnings`: 종료 코드 **0**
- 최종 검증 뒤 코드 변경 없음

## 남은 작업

- App 시작 시 카드 값을 `slot2_platform::clock::set_utc_offset_min`에 적용해야 한다.
- store의 범위 상수와 platform clock의 범위 상수가 같은지 App 경계에서 검증해야 한다.
- 시간대 선택 UI, 즉시 적용, 안전 저장과 실패 rollback은 후속 태스크다.
