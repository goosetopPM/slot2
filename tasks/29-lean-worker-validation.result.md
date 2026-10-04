# Task 29 — 작업 방식 개선 검증

2026-09-25. 최종 판정: 컨텍스트 축약 비교는 성공. 자동 실행·검증 스크립트 구현은 2회 실패하여 미채택, 자동 처리 중단.

## 동일 읽기 비교

동일 CLI 바이너리(0.155.0-alpha.16.4), 동일 B.AI 모델(glm-5.3-flash), 동일 read-only sandbox, 동일 프롬프트로 VERSION.txt를 실제 파일 도구로 읽었다. 각 구성 1회이므로 통계적 성능 검증은 아니다.

| 구성 | input tokens | output tokens | cached input | 초 | 정확성 |
|---|---:|---:|---:|---:|---|
| 기존 설정 | 41,199 | 155 | 20,480 | 30.2 | 통과 |
| plugin/MCP/skill 비활성 override | 41,111 | 155 | 11,200 | 22.2 | 통과 |
| 위 설정 + AGENTS 축약 | 32,606 | 155 | 14,336 | 28.1 | 통과 |

모두 SLOT2 0.1.0 (610a815) 및 OCX_WORKER_READ_OK 반환. input 감소는 기존 대비 20.86%. 캐시 패턴과 실행시간이 다르므로 실제 비용·계정 한도 절감률로 환산하지 않는다. plugin 제외 단독 효과는 0.21%로 이 실험에서는 미미했다. 특정 설정이 실제 어느 토큰을 제거했는지 요청 전체를 조사한 결과는 아니다.

셸이 읽은 파일 한 줄만 별도 API에 전달한 비교는 Provider connect timeout after 30000ms로 실패하여 제외. 실제 소모 토큰·청구액은 알 수 없다. 성공했다고 기록하지 않는다.

## 적용한 문서 변경

- AGENTS.md: 실행에 필요한 핵심 규칙으로 압축. 원문은 docs/AGENT-OPERATIONS-LEGACY.md에 보존.
- 실기 금지, 공용 YAML 승인 요구, 키 비공개, 2회 실패 중단, 테스트 계약 유지, 커밋 요청 시만 등 제약 유지.
- HANDOFF 최상단의 여러 시점 기록을 현재 상태로 정리하고 과거 근거는 tasks 결과 파일로 연결.
- C:\Users\gyuha\.codex\config.toml, ~/.gjc-bai/agent/config.yml, models.yml 해시 모두 시작 시와 동일.

## 스크립트 작성 검증

build/run-bai-worker.ps1, build/verify-workspace.ps1은 B.AI 워커에게 구현 위임했다. 첫 구현은 파일 생성 후 502 Provider connect timeout으로 종료1. 독립 검토에서 PS5.1 ArgumentList 비호환, 스트림 deadlock 가능성, skill override 구조, 테스트 regex, 진행 출력/결과 객체 혼합, 경로/실패 처리 문제를 발견했다.

tasks/29-lean-worker-fix.md로 결함을 좁혀 2번째이자 마지막 구현 시도를 수행했으나 다시 502 Provider connect timeout after 30000ms로 종료1. 최종 워커 보고서도 생성되지 않았다. 두 실행 모두 코드는 일부 생성했으나 완료 기준을 만족하지 못했다.

두 번째 수정본의 run-bai-worker.ps1:93~94는 skills.config를 여전히 이름별 테이블로 만들며 공식 스키마의 array<object>와 맞지 않는다. :85에서는 사용자 .codex/skills 대신 프로젝트 경로를 사용한다. 따라서 연결 실패만 해결되면 완성이라는 결론도 내릴 수 없다. PowerShell 5.1 실동작, 실패 집계/중복 실행 방지 등은 최종 검증되지 않았다.

프로젝트 규칙에 따라 세 번째 수정·다른 모델 우회 재시도를 하지 않았다. build/run-bai-worker.ps1과 build/verify-workspace.ps1은 미검증 초안이며 사용하면 안 된다. 현재 검증된 수동 워커 명령은 Task 26에 유지된다. 필요한 후속은 사용자의 새 지시를 받은 후 연결 시간 초과 원인 확인 및 런처/검증기를 별도 작은 태스크로 분리하는 것이다. 메인 Codex가 직접 구현을 마무리하는 방법도 선택할 수 있으나 이번에는 규칙을 우회하지 않았다.

## 증거

- .gjc-logs/29-{baseline,lean,compact}-summary.json, 29-comparison.json
- .gjc-logs/29-implementation.jsonl, 29-fix.jsonl
- .gjc-logs/29-global-hashes.json

공식 설정 근거: https://learn.chatgpt.com/docs/config-file/config-reference 및 https://learn.chatgpt.com/docs/config-file/config-advanced. 글로벌 파일 수정 대신 CLI -c 일회성 override 사용.

## 범위

M4 기능 개발 없음, 커밋·푸시·실기 조작 없음. Task 27의 기존 372 passed는 유지하며 같은 테스트를 절감 측정 명목으로 반복하지 않았다. clippy/dist는 계속 보류 상태다. 이 검증은 전체 개발 파이프라인 비용 절감이나 안정성을 입증하지 못했다.

최종 확인 명령: `Get-Content tasks/29-lean-worker-validation.result.md`. 자동 구현은 중단 상태이며 다음 실행에는 사용자 지시가 필요하다.
