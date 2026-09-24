# SLOT2 — 작업 규칙

## 1. 역할 분리 (예외 없음)

### 1-1. 메인이 직접 하는 것 — 아래 5개만

(a) 태스크 명세·계약 작성 (Write/Edit: `tasks/NN-*.md`, 스켈레톤)
(b) 판정 — 서브에이전트가 남긴 `tasks/NN-*.result.md` 를 읽고 채택/반려 결정
(c) `git add` / `git commit` / `git log` / `git diff --stat` 만
(d) `.claude/resume.md` 갱신
(e) 사용자 보고

### 1-2. 위임 트리거 (정량, 재량 없음) — 하나라도 걸리면 서브에이전트로 간다

- 파일을 1개라도 읽어야 한다 (Read/Grep/Glob)
- 명령을 1개라도 실행해야 한다 (빌드·테스트·검증·검색·스크립트)
- 파일을 1개라도 수정해야 한다 (`tasks/NN-*.md` 와 `.claude/resume.md` 는 제외)
- 웹을 봐야 한다 (WebFetch/WebSearch)

"사소하니까", "한 줄이니까", "빠르니까"는 예외 사유가 아니다.

### 1-3. 메인 금지 목록 — 절대 하지 마라

- Read / Grep / Glob / WebFetch / WebSearch 로 소스·문서를 직접 뒤지는 것
- Bash·PowerShell 로 빌드·테스트·검증·검색·스크립트를 돌리는 것
- bai-gjc 를 직접 실행하는 것 (반드시 서브에이전트를 통해서만)
- 소스 파일을 직접 수정하는 것 (계약 스켈레톤 최초 작성 시점만 예외)

위 작업이 필요하면 Task 로 서브에이전트를 띄우고, 결과를 파일로 받아라.

### 1-4. 보고 규약 (양방향 크기 제한)

- 서브에이전트는 결과를 `tasks/NN-<이름>.result.md` 에 쓴다. 메인은 그 파일만 읽는다.
- Task 반환값은 5줄 이내로 강제한다:
  "성공/실패 / 실행한 검증 명령 / 마지막 결과 줄 / 수정 파일 목록 / 실기 확인 필요 항목"
- 로그 원문·diff 전체·파일 전체를 반환값에 넣으면 메인이 거부하고 재위임한다.
- 서브에이전트에게 넘길 때도 "파일은 네가 읽고, 요약만 남겨라"를 명시한다.
  메인이 파일을 읽어서 넘기지 마라.

### 1-5. 실패 시 에스컬레이션

- 서브에이전트 2회 실패 → 메인이 직접 하지 말고 사용자에게 보고하고 멈춘다.
  "이 태스크는 자동 처리 실패. 원인: <1줄>. 직접 지시가 필요합니다."
- 메인이 직접 손대는 것은 이 규칙 위반이다. 예외 없다.

### 1-6. 사용자 우선

- 사용자가 "메인에서 직접 해"라고 명시하면 그 지시가 이 규칙보다 우선한다.
  단 그때 "컨텍스트 소모가 커집니다"를 한 줄로 알려라.

## 2. 코드 생성 워커 — bai-gjc

맨 `gjc` 를 쓰지 않는다. 그건 다른 프로필(`~/.gjc`)이다.

- PowerShell/cmd: `C:\Users\gyuha\bai-gjc-starter\bin\bai-gjc.cmd`
- git-bash: `C:\Users\gyuha\bai-gjc-starter\bin\bai-gjc`
- PATH 에 `bai-gjc` 가 있으면 같은 파일이다.

기본형:

```
bai-gjc -p --no-session --model bai/glm-5.3-flash "<태스크 프롬프트>"
```

- **모델**: 기본 `bai/glm-5.3-flash`(executor, 최저가). 어려운 태스크만 `bai/deepseek-v4.1-flash`(default).
  둘 다 실패해도 `--model` 을 바꿔가며 재시도하지 않는다 — `~/.gjc-bai/agent/config.yml` 의
  `modelRoles` 배열(`executor: [glm-5.3-flash, qwen3.8-flash, deepseek-v4.1-flash]`)을 따라
  B.AI 안에서 자동으로 넘어간다. 설정은 v2 스키마다. **`fallbackChains` 는 없다** — 파일 주석이
  legacy 라고 못박고 있다.
- **로그는 리다이렉트하되, 진행 신호로 쓰지 마라.** `*> .gjc-logs\NN-<이름>.log` 로 남긴다
  (cwd 상대경로라 프로젝트별로 분리된다). 다만 출력은 **버퍼링돼서 종료 시에 한 번에
  떨어진다** — 돌고 있는 동안 0바이트인 것은 정상이고, 죽이면 그대로 날아간다. 실제로 그
  빈 로그를 근거로 멀쩡히 돌던 워커를 세 번 죽였다. 진행 신호는 **`git status --short`** 다.
- **프리플라이트**: `BAI_API_KEY` 가 비면 런처가 즉시 exit 1 이다. 첫 호출 전에 존재를 확인하고,
  없으면 사용자에게 알리고 중단한다.
- **타임아웃**: 런처가 `PI_STREAM_FIRST_EVENT_TIMEOUT_MS=300000` 을 세팅한다. reasoning 모델이라
  첫 토큰이 5분까지 안 올 수 있다. 서브에이전트가 성급하게 죽이지 않도록 위임 프롬프트에 명시한다.
- 런처가 `ANTHROPIC_*` / `OPENAI_API_KEY` / `OPENROUTER_API_KEY` 를 지우고 실행한다. 이 동작을 우회하지 않는다.
- 로그: bai-gjc 자체 세션 기록이 `~/.gjc-bai/agent/sessions/`, 로그가 `~/.gjc-bai/logs/` 에
  남는다. 실행 후 조사할 게 있으면 거기를 본다. `GJC_CONFIG_DIR` 은 런처의 상대경로 표기와
  무관하게 `~/.gjc-bai` 로 해석된다.

B.AI 는 90% 할인 모델이지 무료가 아니다. 싸지만 크레딧이 나간다.

### 2-1. 워커 프로필은 프로젝트 간 공용이다

SLOT2 와 다른 프로젝트 창에서 같은 워커를 동시에 쓴다. 2026-09-24 실측 기준.

- 프로필은 `~/.gjc-bai` **하나**이고 cwd 와 무관하다. `GJC_CONFIG_DIR` 은 경로가 아니라
  `$HOME` 에 붙는 이름이다. 어느 프로젝트에서 돌려도 같은 설정·같은 DB·같은 키를 쓴다.
- **동시 실행은 안전하다.** 두 프로젝트에서 29초간 완전히 겹쳐 돌려 양쪽 exit 0,
  SQLITE_BUSY·락 에러 0 이었다. 공유 SQLite 는 WAL + busy_timeout, crash-index·yaml 패치는
  파일 락으로 보호된다. **상대가 끝나기를 기다리지 마라.**
- 실행 직전 1회만 점검한다 (30초 넘는 점검은 하지 마라):

  ```
  Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -match 'gjc\.js' -and $_.CommandLine -match '--no-session' } | ForEach-Object { '{0,6}  {1}  {2}' -f $_.ProcessId, $_.CreationDate.ToString('HH:mm:ss'), $(if ($_.CommandLine -match 'SLOT2') {'SLOT2'} elseif ($_.CommandLine -match 'spruceos') {'spruceos'} else {'?'}) }
  ```

  0~1개면 그냥 실행한다. **이미 2개면 실행하지 말고 사용자에게 한 줄로 알리고 기다린다** —
  키가 하나라 요금·레이트리밋을 공유하고, 3병렬 이상은 확인되지 않았다.
  논리적 워커 1개 = OS 프로세스 3개(`cmd.exe` + `gjc.exe` + `bun`)라, `gjc` 로 대충 grep 하면
  3배로 세고 Claude Code 자신의 셸까지 잡힌다. 위 식(`gjc.js` + `--no-session`)만 쓴다.

### 2-2. 공유 프로필은 절대 건드리지 마라 (가장 중요)

`~/.gjc-bai/agent/models.yml` 과 `~/.gjc-bai/agent/config.yml` 은 두 프로젝트의 공용 자산이고,
**이게 유일한 실제 파손 경로다** — 워커가 읽는 중에 덮어쓰면 반쯤 쓰인 YAML 을 읽고 죽는다.

수정이 필요하다고 판단되면 고치지 말고 **무엇을 왜 바꿔야 하는지 한 줄로 보고하고 승인을
받아라.** 사용자가 양쪽 창을 다 보므로 사람이 직렬화 장치 역할을 한다. 승인 후에는
(a) 2-1 점검으로 워커 0 확인 → (b) 임시파일에 쓴 뒤 `Move-Item -Force` 로 교체 →
(c) 바꾼 내용 보고, 이 순서를 지킨다.

프로젝트 안의 `tasks/`, `.gjc-logs/`, 소스 파일은 각자 것이니 자유롭게 고쳐도 된다.

### 2-3. 로그는 섞인다

`~/.gjc-bai/logs/gjc.<날짜>.log` 와 `~/.gjc-bai/logs/http-400-requests/` 는 두 프로젝트가 같이
쌓는다. 여기서 뭘 읽을 때는 **반드시 pid 로 거른다.** 다른 프로젝트 pid 의 에러를 우리 문제로
오진하지 마라.

### 2-4. HTTP 400 이 나면

`code=400001` 은 `role:"developer"` 문제이고 2026-09-24 에 고쳤다
(`models.yml` 의 `compat: { supportsDeveloperRole: false }`). 재발하면 추측하지 말고
`~/.gjc-bai/logs/http-400-requests/*.json` 의 `body.messages[*].role` 을 먼저 본다 — gjc 가
거부된 요청 본문을 그대로 떨어뜨린다. `max_completion_tokens` 는 무죄다.

### 2-5. 오래된 대화형 인스턴스

`models.yml` 은 프로세스 시작 시 1회 읽힌다. 설정을 고친 시각보다 먼저 뜬 대화형 gjc 는 옛
설정을 메모리에 들고 있어 계속 400 이 난다. 확인:

```
$cfg = (Get-Item "$env:USERPROFILE\.gjc-bai\agent\models.yml").LastWriteTime; Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -match 'gjc\.js' -and $_.CommandLine -notmatch '--no-session' } | ForEach-Object { '{0}  started {1}  {2}' -f $_.ProcessId, $_.CreationDate.ToString('HH:mm:ss'), $(if ($_.CreationDate -lt $cfg) {'STALE-재시작필요'} else {'OK'}) }
```

STALE 이 보이면 직접 죽이지 말고 "pid NNNN 은 <시각> 에 떠서 옛 설정을 쓰고 있으니 그 창을
재시작해달라"고 알린다. `sdk/broker/internal...` 로 뜬 bun 은 워커가 아니라 SDK 브로커
자식이다. 요청 페이로드를 만들지 않으므로 400 과 무관하고 부모가 죽으면 스스로 정리한다.
건드리지 마라.

### 2-6. 키 주입

매 호출마다 같은 PowerShell 안에서:

```
$env:BAI_API_KEY = [Environment]::GetEnvironmentVariable('BAI_API_KEY','User')
```

**값은 절대 출력하지 마라.** 비어 있으면 사용자에게 알리고 중단한다.

## 3. 서브에이전트 위임 프롬프트에 반드시 넣을 것

1. 태스크 파일 **절대경로**
2. 완료 기준 **명령 원문**
3. 재시도 **최대 2회**
4. 첫 응답까지 **최대 5분 대기**, 전체는 **최대 45분**. 진행 신호는 `git status --short`
5. 워커에게 **`CLAUDE.md` 를 무시하라고 명시**한다. 이건 오케스트레이터용 규칙서라, 워커가
   읽으면 자기가 서브에이전트에 위임하려 든다
6. 보고 형식: 성공/실패 + 마지막 테스트 결과 줄 + 실패 시 원인 1줄 + 생성/수정한 파일 목록
   + **계약이 틀려 보이면 그 줄**

## 3-1. 태스크 크기

**`todo!()` 20개 / 크레이트 3개 = 한 번에 안 된다.** 그렇게 준 태스크는 세 번 돌려 93분 동안
파일 하나 안 만들고 끝났다. 크레이트 경계로 자른다 — 계약 파일이 이미 크레이트마다 하나씩이니
그 선이 그대로 태스크 경계다. 한 태스크에 `todo!()` 15개 안쪽을 목표로 한다.

## 4. 컨텍스트 예산 (하드)

메인 컨텍스트가 **150,000 토큰**을 넘으면 새 작업을 시작하지 않는다. 즉시:

1. `C:\SLOT2\.claude\resume.md` 갱신 — (a) 현재 마일스톤 (b) 완료 태스크 (c) 미완 태스크
   (d) 다음 실행할 정확한 명령 3줄 이내
2. 사용자에게 딱 한 줄: `/clear 후 '이어서 계속' 이라고 입력해주세요.`

**핸드오프 파일은 `.claude/resume.md` 다.** `docs/HANDOFF.md` 가 아니다 — 유저 레벨 훅 두 개가
그 경로로 고정돼 있다. `~/.claude/hooks/inject-resume.sh` 가 SessionStart 에서 `$cwd/.claude/resume.md`
를 그대로 출력하고, `~/.claude/hooks/ctx-budget.sh` 가 PreToolUse 에서 150K 초과 시 `*resume.md*`
쓰기 외의 모든 도구를 exit 2 로 막는다. 다른 파일에 써 두면 /clear 뒤에 아무것도 안 나오고,
예산을 넘긴 뒤에는 그 파일을 쓰는 것조차 차단된다. 두 훅 모두 프로젝트 무관하게 동작한다.

## 5. 세션 단위

**마일스톤 또는 태스크 묶음 하나 = 세션 하나.** 태스크가 끝나면 커밋 → HANDOFF.md 갱신 → `/clear` 요청.
한 세션에서 다음 태스크로 넘어가지 않는다.

## 6. 그 밖에 계속 유효한 것

- **말투**: 간결하되 **존댓말**.
- **읽기 전용**: `D:\Refrom\SpruceOS\dist_final\pack` 은 배포판 자료다. 읽기만 하고 아무것도 쓰지 않는다.
- **ROM**: 상용 ROM 은 절대 저장소에 커밋하지 않는다. `assets/test/local/`(gitignore) 또는
  `$SLOT2_TEST_ROMS` 가 가리키는 곳에만 둔다.
- **하우스 스타일**: 주석은 *왜* 를 쓰고 *무엇* 은 쓰지 않는다. 호출자의 산술에 `unwrap()` 금지.
  린트를 `#[allow(...)]` 로 덮지 않고 원인을 고친다.
- **테스트를 통과시키기 위한 코드 금지**: id 번호를 벌리려는 패딩 업로드, 단언을 "만족시키려고" 넣은
  추가 드로우, 특정 테스트 입력에 맞춘 상수. 계약이 틀렸다고 판단되면 보고서에 그렇게 적고 실패인 채로 둔다.


### 6-4. 실기 검증은 사용자만 한다

RG SP 실기 확인은 **사용자만** 한다. 워커·서브에이전트가 adb / Samba(`S:`) / SD 카드를 만지면
안 된다. 서브에이전트 보고의 "실기 확인 필요 항목"은 사용자에게 넘기는 목록이지 실행 지시가
아니다.

### 6-5. 역할 분리 강제 훅 (아직 켜지 않음)

1-1~1-6 은 전부 말로만 하는 규칙이라, 메인이 어겨도 아무 일도 일어나지 않는다. 2~3 세션 돌려
보고 그래도 메인에서 Bash/Read 가 새면 이걸 켠다. **지금은 스펙만 적어 둔 상태다.**

`~/.claude/hooks/main-role-guard.sh` 를 만들고 PreToolUse(match `*`)에 등록한다. 기존
`ctx-budget.sh` 와 같은 matcher 이므로 한 스크립트로 합쳐도 된다.

- stdin 으로 PreToolUse JSON 을 받는다.
- `transcript_path` 에 `subagents` 가 있으면 즉시 exit 0 (서브에이전트는 자유롭게).
- 메인 스레드일 때만 판정한다:
  - `Task` / `TodoWrite` / `AskUserQuestion` → exit 0
  - `Write` / `Edit` → `file_path` 가 `tasks/` 안이거나 `resume.md` 일 때만 exit 0, 아니면 exit 2
  - `Read` → `tasks/*.result.md`, `.claude/resume.md`, `CLAUDE.md` 일 때만 exit 0, 아니면 exit 2
  - `Grep` / `Glob` / `WebFetch` / `WebSearch` → exit 2
  - `Bash` / `PowerShell` → command 가 `^\s*git (add|commit|log|diff|status|rev-parse)` 일 때만
    exit 0, 아니면 exit 2
  - 그 외 → exit 0
- exit 2 일 때 stderr: "메인 스레드에서 <tool> 금지. Task 서브에이전트로 위임하고 결과를
  `tasks/<NN>-<이름>.result.md` 로 받아라. 이 규칙은 1-3항이다."
- **비상 토글**: `~/.claude/allow-main-tools` 가 존재하면 전부 exit 0. 사용자가 직접 개입해
  메인에서 작업시킬 때만 만들고, 끝나면 반드시 지운다.
- 검증: 가짜 JSON 을 echo 로 먹여 6가지 exit code 를 실제로 확인한다 —
  Read `tasks/x.result.md` / Read `src/x.rs` / Bash `git commit` / Bash `cargo test` /
  `subagents` transcript_path / `allow-main-tools` 존재.

**주의**: 1-1(b) 판정 경로(`tasks/*.result.md` 읽기)가 허용 목록에 살아 있어야 한다. 막으면
메인이 판정 자체를 못 한다. 그리고 훅을 켜기 전에 비상 토글을 만들었다 지우는 테스트를 반드시
먼저 하라 — 없으면 급할 때 완전히 잠긴다.

