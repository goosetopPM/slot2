# Task 26 — OpenCodex B.AI 전용 워커 검증 결과

최종 갱신 2026-09-25. 판정: 연결 및 읽기 전용 Codex 워커 검증 성공.
아래 초기 실패 기록은 2026-09-24의 경과이며, 사용자 소유 상태 폴더로 해결됨.

## 최종 검증 (2026-09-25)

- 사용자가 일반 PowerShell에서 `%LOCALAPPDATA%\SLOT2\opencodex-worker`를 생성하여 프록시 시작 성공. `Codex integration OFF; startup left Codex native.` 확인.
- 프록시 `/v1/responses`에 `bai/glm-5.3-flash`로 최소 요청: `completed`, `BAI_OCX_OK`, 109 total tokens.
- 최초 Codex 워커 실행 요청은 자동 승인 검토의 사용량 제한 때문에 실행되지 않았음. 안전성 거절과 구분.
- 사용자의 작업 재개 지시 후 꺼져 있던 프록시를 동일 사용자 소유 설정으로 재시작. healthz status=ok, version=2.64.0, port=10126.
- Codex CLI v0.155.0-alpha.16.4, model=bai/glm-5.3-flash, provider=slot2_bai, sandbox=read-only로 실제 실행.
- 워커가 PowerShell `Get-Content -LiteralPath 'C:\SLOT2\dist-device\System\VERSION.txt' -TotalCount 1` 도구 호출. 결과 `SLOT2 0.1.0 (610a815)` 일치, `OCX_WORKER_READ_OK`, 종료 코드 0.
- CLI가 보고한 tokens used=39,548. 단순 연결 검사 대비 컨텍스트가 크므로 실제 작업 전 불필요한 로드 항목을 줄일 여지가 있음. 비용 자체는 조회하지 않음.
- 원래 Codex config.toml 및 bai-gjc 공용 YAML 2개 해시 모두 기존 기준과 동일.
- 로그: `.gjc-logs/26-ocx-worker.log`, 응답: `.gjc-logs/26-ocx-worker-answer.txt`.
- 프록시는 이 세션에서 시작한 프로세스로 현재 실행 중. 자동 시작 서비스는 설치하지 않음.
- 검증 범위는 실제 추론 및 파일 읽기 도구 왕복. 코드 생성·수정 품질, 장시간 실행, 모델 fallback은 아직 미검증.

검증된 워커 명령(PowerShell, 프록시 실행 중 필요):

```powershell
& 'C:\Users\gyuha\AppData\Local\OpenAI\Codex\bin\13995fba801849b0\codex.exe' exec --ephemeral --sandbox read-only -C C:\SLOT2 -m bai/glm-5.3-flash -c 'model_provider="slot2_bai"' -c 'model_providers.slot2_bai={name="SLOT2 BAI worker",base_url="http://127.0.0.1:10126/v1",wire_api="responses",requires_openai_auth=false,request_max_retries=0,stream_max_retries=0}' -c 'web_search="disabled"' -c 'model_reasoning_effort="low"' -c 'features.multi_agent=false' 'Read C:\SLOT2\dist-device\System\VERSION.txt using a file tool. Report its first line. Do not edit files or delegate.'
```

위 바이너리 경로는 현재 설치본 기준. 실제 코드 수정 태스크는 별도로 범위와 sandbox 권한을 정해야 함.

## 사용자 선택

현재 Codex 대화 모델은 유지하고 B.AI 코드 생성 워커만 연결한다. 전역 Codex 모델 전환은 요청하지 않았다.

## 완료한 것

- 제작자 저장소 `https://github.com/lidge-jun/opencodex`와 공식 프로젝트 문서 확인.
- `@bitkyc08/opencodex@2.64.0`을 `.gjc-logs/ocx-worker/install`에 Bun으로 설치. 설치 종료 코드 0.
- `.gjc-logs/ocx-worker/state/config.json`에 별도 루프백 10126, B.AI `openai-chat` 제공자, `glm-5.3-flash` 설정.
- 키는 `${BAI_API_KEY}` 환경변수 참조만 저장. 실제 키는 사용자 환경에서 시작 프로세스로 주입.
- `clientIntegrations`의 codex/grok/claude-desktop 비활성화, Codex shim 자동 복구·자동 시작·이력 동기화·Claude 환경 주입 비활성화.
- 원래 `~/.codex/config.toml`, `~/.gjc-bai/agent/config.yml`, `models.yml`의 전후 SHA256 모두 동일 확인.

## 실패 근거

1. `bun .../opencodex/src/cli/index.ts start --port 10126`: 종료 코드 1. `Spend-ledger ownership could not be established safely.`
2. 같은 설치본의 공식 `acquireSpendLedgerOwner()` 함수를 검증 폴더에서 호출하여 cause만 출력: 종료 코드 1.
   - `SPEND_LEDGER_OWNER_UNAVAILABLE`
   - 내부 원인 `EICACLS`: `ACL hardening failed (EICACLS) — icacls command error; filesystem may not support per-user NTFS ACLs`.

파일시스템 미지원이라는 문구는 오류 메시지의 가능성 설명이며 확정 진단이 아니다. 보호 검사를 끄거나 설치본을 패치하지 않았다. B.AI 추론 요청·도구 호출·Codex 워커 실행 단계에는 도달하지 못했다.

마무리 읽기 전용 확인: C 드라이브는 NTFS. 취소한 Task 25 PID 18552는 해당 명령의 프로세스로 남아 있지 않음.

## 후속 읽기 전용 진단 — 사용자 요청: 해결 방법 안내

`icacls C:\SLOT2\.gjc-logs\ocx-worker\state` 조회 종료 코드 0.
`Get-Acl`의 Owner는 `kros_n100\CodexSandboxOffline`, AreAccessRulesProtected=False.
ACL에는 CodexSandboxUsers 수정(M), 일반 Users 읽기/실행(RX), Authenticated Users 수정(M)이
상속되어 있고, 실제 사용자 gyuha에 대한 명시적 Full Control은 보이지 않음.
설치본은 `/grant:r` → `/inheritance:r` 순서로 ACL을 수정한다.
따라서 샌드박스 계정이 만든 폴더를 실제 사용자 실행에서 권한 변경하려다 실패했을
가능성이 높음. 실제 실패 명령의 stderr는 설치본에서 버려지므로 확정은 아님.
권장: 일반 사용자 PowerShell에서 LOCALAPPDATA 하위 별도 상태 폴더를 만들고 설정 내용만
복사하여 해당 OPENCODEX_HOME으로 실행. 기존 폴더 ACL 완화나 관리자 상시 실행은 불필요.
이번 후속 요청은 방법 안내이므로 세 번째 프록시 시작이나 ACL 수정은 수행하지 않음.

## 범위와 후속

- 설치만 성공했으며 사용 가능 검증은 실패. 바로 프로젝트 코드 생성에 쓰면 안 됨.
- 프로젝트 규칙에 따라 세 번째 시도는 하지 않음. Windows ACL 실패에 대한 별도 사용자 지시가 필요.
- 재사용 실행 명령은 아직 검증되지 않음. 다음 읽기 전용 확인 명령: `Get-Content .gjc-logs/26-opencodex-server.log -Tail 20`.
- 소스 코드 수정·커밋·푸시·실기 조작 없음. 설치물·프록시 설정·진단 코드는 gitignore된 `.gjc-logs/`에 있음.
- 마지막 테스트 결과 줄: 없음(Rust 테스트 미실행).

## 출처

- https://opencodex.me/guides/providers/ — 사용자 정의 OpenAI Chat Completions 제공자 지원.
- https://opencodex.me/guides/codex-integration/ — 기본 시작 시 전역 Codex 연동 주의.
- https://docs.b.ai/llmservice/api/ — B.AI 프로토콜 및 모델별 엔드포인트 차이.
- 설치본 `src/codex/desired-state.ts` — `clientIntegrations.codex=false` 시작 동기화 제외.
- 설치본 `src/lib/spend-ledger-owner.ts`, `windows-secret-acl.ts` — 소유권 확인 실패 및 내부 원인.
