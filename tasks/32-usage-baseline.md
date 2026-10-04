# Task 32 사용량 기준선

기록: 2026-09-25, OpenCodex 실행 전.

- Codex 계정 스냅샷: 5시간 창 79% 사용, 주간 창 43% 사용.
- 이 수치는 Task32 시작 비용이 아니라 현재 계정 전체 상태다. 다른 창 사용, 반올림, 리셋의 영향을 받는다.
- 비교 대상 Task27 OpenCodex/Codex CLI 보고값: `tokens used 50,394`. 당시 input/output/cached 세부가 없고 작업 범위도 다르므로 직접 절감률의 정확한 분모로 쓰지 않는다.
- Task29 동일 읽기 참고: 기존 input 41,199, 규칙 축약 후 input 32,606(1회 표본, 약20.86% 감소).

Task32 회수 시 기록할 값:

1. OpenCodex 작업 결과의 input/output/cached 토큰과 실행 시간. 미확인이면 미확인.
2. OpenCodex 실행 횟수와 같은 태스크 누적 시도 수.
3. Codex 계정 5시간/주간 창의 회수 시 스냅샷. 차이는 상한 참고치이며 Task32 단독 사용량으로 단정하지 않음.
4. Codex 측 사용자 왕복: 목표 2회(명세 제공, 결과 검토). 오류·방향 변경은 별도 집계.
5. 검증 재실행 수와 Codex가 직접 수행한 구현 수정 수.

첫 수동 입력은 채팅 렌더링 과정에서 PowerShell 백틱·밑줄·URL이 변형되어 Codex CLI의 로컬 config 파싱 단계에서 종료됐다. OpenCodex/모델에 요청이 도달하지 않았고 결과 파일도 생성되지 않았으므로 구현 시도에는 포함하지 않는다. 원본 오류 로그는 `.gjc-logs/32-opencodex.jsonl`에 보존한다. 이후 Task32 전용 실행기 `build/run-task32-opencodex.ps1`로 인자 복사 오류를 제거한다.

전용 실행기 첫 호출도 PowerShell 네이티브 인자 전달에서 inline TOML의 큰따옴표가 제거되어 같은 로컬 config 파싱 단계에서 종료됐다. JSONL/stderr는 0바이트이고 결과 파일도 없다. 구현 시도에는 포함하지 않는다. inline table을 없애고 각 provider 필드를 dotted override와 TOML 작은따옴표 문자열로 전달하도록 수정했다.

실제 구현 시도1: exit1, 155.3초, usage 없음, 결과 파일 없음. 작업자는 실행됐지만 PowerShell 도구로 읽은 한국어가 mojibake로 전달됐고, 파일 조회 뒤 후속 요청이 `502 Provider connect timeout after 30000ms`로 실패했다. 허용 소스 변경 없음. 구현 시도1/2로 집계한다. 최종 시도는 ASCII 영문 명세와 영문 프롬프트만 전달하고 불필요한 문서 읽기를 명시적으로 금지한다.

최종 추산은 (a) 동일 읽기 실험의 컨텍스트 감소, (b) Task27 대비 메인 개입 감소, (c) Task32 자체 보고 토큰을 분리해서 제시한다. 서로 다른 태스크의 총 토큰만으로 품질 보정 없는 단일 절감률을 만들지 않는다.

## 회수 결과

- 시도2: exit1, 206.5초, usage 없음, 결과 파일 없음. `in_game_menu.rs` 신규와 `lib.rs` 수정 후 502 timeout.
- Codex 계정 회수 스냅샷: 5시간96%, 주간46%. 시작 대비 +17%p/+3%p이며 Task32 단독 비용으로 단정하지 않는다.
- 완료 산출물0, 부분 파일2. 기존 대비 사용량 절감 성공으로 판정할 수 없다. 상세 `tasks/32-ingame-menu-ui.result.md`.
