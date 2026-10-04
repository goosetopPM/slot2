# Task 29 — 최종 수정 시도(2/2)

원본 명세 C:\SLOT2\tasks\29-lean-worker-validation.md를 따르되 아래 실제 결함을 고친다. 이전 실행 exit1, 파일 두 개만 생성된 상태. 수정 허용은 build/run-bai-worker.ps1, build/verify-workspace.ps1, tasks/29-lean-worker.worker-result.md. 커밋·재귀 위임·실기·전역설정 변경 금지.

1. run-bai-worker ProcessStartInfo.ArgumentList는 Windows PowerShell5.1/.NET Framework에서 없다. `prompt | & $CodexExe @argList 1>stdout 2>stderr` 방식과 PS5.1 quoting 호환성을 사용하거나 검증된 Windows argv quoting을 구현한다. .NET 순차 ReadToEnd 두 스트림은 deadlock 가능하니 금지. 프롬프트 UTF8 stdin 보장. 리다이렉트 경고가 명령 실패와 혼동되지 않게 native exit 캡처.
2. skills.config는 indexed 이름 테이블이 아니다. USERPROFILE/.agents/skills 및 .codex/skills 아래 SKILL.md 경로를 나열하고 부모 디렉터리에 `{path="C:/...",enabled=false}` array override를 만든다. 파일 본문 읽기 금지. mcp_servers.node_repl.env를 별도 서버 node_repl.env로 오인하지 않게 정확한 최상위 테이블만 처리한다. TOML literal/quoted와 bare key 지원(복잡한 escaped quote는 무리하게 추측하지 말고 제외/실패).
3. verify Invoke-Step에서 Write-Output 텍스트와 return 객체가 섞여 $r이 배열이 되는 결함: 진행 문구는 Write-Host, return은 단일 객체.
4. Rust 결과 regex 실제 예: `test result: ok. 3 passed; 0 failed; ...`. 현재 regex는 ok. 때문에 항상0 집계. FAILED도 처리.
5. verify 반드시 Push-Location $root/finally Pop-Location. lock OpenOrCreate에는 ReadWrite 사용. lock 실패 포함 최상위 catch에서 failed summary 기록. 모든 선택 로그를 실행 전 검사하여 동일 RunId 일부 실행/덮어쓰기 방지. 중복 summary 생성 동시 경쟁은 lock 안에서 재확인.
6. Dist 경로 검사는 스크립트 파일만이 아니라 실제 재귀 삭제 대상인 root/dist-device의 절대경로 및 reparse point 검사. 예상 root내 정확한 dist-device인 경우만 실행. -Adb 금지 유지.
7. 실패 요약에는 원인 문자열을 남기고, 미확인 토큰은 null/unknown으로 표시하여0으로 오인하지 않게 한다. 예외시에도 실행 로그 덮어쓰기 금지. 45분은 현재 프롬프트 예산인지 강제 timeout인지 솔직히 보고.

검증: 두 파일 powershell5.1 Parser::ParseFile 오류0, 그리고 실제 `powershell -NoProfile -File C:\SLOT2\build\verify-workspace.ps1 -RunId 29-worker-fmt -Steps Fmt` exit0 및 summary.steps가 단일 객체 배열이고 ok=true인지 확인. 전체 Rust test/clippy/dist 및 중첩 B.AI 실행 금지. worker 런처는 잘못된 TaskFile 입력이 nonzero인지 확인. 그 외 미검증은 보고서에 명시한다.

첫 응답 최대5분, 총45분. 이번은 2번째/마지막 구현 시도. 완료/실패 및 검증 결과, 파일 목록, 계약 문제를 tasks/29-lean-worker.worker-result.md에 작성한다. 기존 코드를 필요한 부분만 수정하고 길게 재설계하지 않는다.
