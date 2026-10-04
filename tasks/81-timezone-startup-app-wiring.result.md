# Task 81 — Codex 최종 판정

- **판정:** 통과
- **누적 작업자 호출:** 1/2
- **작업자 보고서:** `tasks/81-timezone-startup-app-wiring.worker-result.md`

## 검토 결과

- production `slot2` 빌드가 store/platform의 UTC offset 최솟값·최댓값과 기본값 계약을 compile-time
  assertion으로 봉인한다.
- `App::with_card`가 struct 생성 전에 카드 offset을 정확히 한 번 읽어 process-global runtime clock에
  적용한다. `App::new`도 같은 경로를 사용한다.
- 저장된 양수·음수·0·양 경계값이 그대로 적용되며, key/file 부재·invalid·범위 밖·invalid UTF-8은
  store 계약에 따라 이전 runtime 값을 UTC 0으로 덮는다.
- 예상 밖 setter 거부는 panic/unwrap 없이 로그를 남기고 UTC 0 적용을 시도한다.
- 설정 파일은 시작 전후 bytes가 같고 missing file도 생성되지 않는다. volume 시작값도 기존 계약대로
  유지된다.
- 단일 integration test 함수 안에서 process-global clock 시나리오를 순차 실행하고 종료 전에 이전 값을
  복원한다. 병렬 테스트 간 offset 경쟁을 만들지 않는다.
- HUD 변환은 같은 UTC sample에 offset만 분 단위로 더하며, `SET_AFTER` 이전 sample의 visibility는
  offset과 무관하게 `None`이다.
- 수정 범위는 명세 안이며 관련 변경의 `git diff --check`를 통과했다.

## 작업자 검증 근거

- `timezone_startup_app`: **1 passed / 0 failed / 0 ignored**
- `cargo test -p slot2`: **313 passed / 0 failed / 0 ignored**
- core-dependent skip: **0**
- device feature check: 종료 코드 **0**
- fmt와 clippy: 종료 코드 **0**
- 최종 검증 뒤 코드 변경 없음

## 남은 작업

- 시간대 선택 UI, runtime 즉시 적용, 안전 저장과 실패 rollback/toast가 필요하다.
- `main.rs::boot()`의 진단 로그는 App 생성 전 값을 출력하므로 카드 설정과 다를 수 있다. 실제 HUD
  runtime 값에는 영향이 없지만 후속 마무리에서 로그 순서 또는 문구를 정리해야 한다.
