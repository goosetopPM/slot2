# Task 29 — 경량 B.AI 워커와 순차 검증 실행기

## 목적
현재 Codex 유지, OpenCodex localhost:10126의 B.AI 워커만 경량화한다. 전역 설정과 공용 B.AI YAML은 수정하지 않는다. M4 개발은 하지 않는다.

## 워커 구현 범위
허용 수정: build/run-bai-worker.ps1, build/verify-workspace.ps1, tasks/29-lean-worker.worker-result.md.
PowerShell 5.1 호환. 외부 모듈 설치 금지. 과도한 프레임워크 없이 간결하게 작성.

### run-bai-worker.ps1
- 필수 TaskFile(절대경로의 기존 파일), RunId(안전한 파일명 패턴), 선택 CodexExe(기본 Get-Command codex로 탐색), Sandbox(read-only 기본/workspace-write), Model(기본 bai/glm-5.3-flash, 선택 bai/deepseek-v4.1-flash).
- 프로젝트 root는 스크립트 부모. 로그는 .gjc-logs/<RunId>.*. 기존 출력 덮어쓰기 금지. healthz 확인 실패 시 즉시 종료. 프록시 시작/설정/키 조작 금지.
- 기존 사용자 config를 수정하지 않고 실행 인자로만 경량화: 등록된 plugins 및 mcp_servers에 enabled=false, 개인/시스템 skill 디렉터리는 skills.config의 path/enabled=false 지원 키로 제외. 스킬 디렉터리만 나열하고 내용은 읽지 않는다. built-in core shell/apply_patch 도구 유지. 기존 AGENTS 및 샌드박스 제약은 유지.
- user config의 plugins/mcp 테이블 이름 추출은 quoted/unquoted 이름을 처리. 프로젝트 config 추가 MCP 가능성은 한계로 기록. 자동 재귀 위임 금지.
- codex exec --ephemeral --json --sandbox <mode> -C <root> -m <model>, model_provider=slot2_bai, model_providers.slot2_bai={name="SLOT2 BAI worker",base_url="http://127.0.0.1:10126/v1",wire_api="responses",requires_openai_auth=false,request_max_retries=0,stream_max_retries=0}, web_search="disabled", model_reasoning_effort="low", features.multi_agent=false.
- 프롬프트는 stdin(-)로 전달하여 PowerShell 인자 quoting 오류를 피한다. 프로세스 종료까지 단일 호출에서 대기한다. 모니터링/모델 재시도 루프 금지.
- 프롬프트에 태스크 절대경로, 완료 조건은 명세에 따름, 최대 2회 시도, 첫 응답5분/전체45분 예산, CLAUDE/AGENTS의 오케스트레이터 역할 분리만 무시(나머지 제약 유지), 위임/커밋/실기/공유설정수정 금지, 보고 형식을 포함.
- 종료 코드, elapsed, JSON turn.completed usage를 요약 JSON에 기록. 실제 청구량이라는 표현 금지. 실패도 요약 작성하고 원래 종료코드 반환. 토큰 합계는 각각 input/output/cached를 분리하고 cache 중복 합산하지 말 것.
- CLI 인자 배열로 호출(Invoke-Expression 금지), $HOME/$home 재사용 금지, 전역 파일 쓰기 금지.

### verify-workspace.ps1
- 기본 fmt --check, cargo test --workspace, cargo clippy --workspace --all-targets -- -D warnings, powershell -File build/dist-device.ps1 순차 수행.
- RunId 필수, 선택 단계 배열(ValidateSet Fmt/Test/Clippy/Dist). 명시 순서는 항상 canonical 위 순서. 외부 명령의 exit code 즉시 캡처, 첫 실패에서 중단. 예외도 실패 결과를 남김.
- 루트 .gjc-logs/verify.lock FileShare.None으로 동시 검증 차단, finally에서 해제. 다른 워커와의 상호배제까지 보장한다고 주장하지 말 것.
- dist 경로가 프로젝트 안의 정확한 dist-device임 확인, reparse point면 중단. -Adb 등 인자 금지.
- 로그는 단계별 파일, 요약 JSON은 명령/exit/elapsed/test passed failed 합계/test result 줄 개수, completed/failed 상태. 동일 RunId 덮어쓰기 금지.
- 테스트 baseline 372 passed 미만 또는 failed>0이면 실패 처리. Dist exit0여도 '==> done' 없으면 실패. 로그 전문 출력 금지, 단계별 1줄 결과와 summary 경로만 출력.
- 테스트를 매번 다시 돌리지 않도록 Steps로 남은 검사만 선택 가능.

## 완료 기준
명령 원문: powershell -NoProfile -Command "$errors=$null; [System.Management.Automation.Language.Parser]::ParseFile('C:\SLOT2\build\run-bai-worker.ps1',[ref]$null,[ref]$errors) > $null; if($errors){exit 1}; [System.Management.Automation.Language.Parser]::ParseFile('C:\SLOT2\build\verify-workspace.ps1',[ref]$null,[ref]$errors) > $null; if($errors){exit 1}"
위는 셸에서 quoting 주의: 워커는 동등한 PowerShell parser API 직접 실행 가능. 실제 워커 중첩 호출/전체 Rust 검사는 하지 않는다. 오케스트레이터가 검토 뒤 실행한다.

## 보고
성공/실패, parser 결과, 생성 파일, 미검증/한계, 계약이 잘못된 부분. 구현 최대2회,45분. 커밋 금지.
