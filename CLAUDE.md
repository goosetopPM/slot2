# SLOT2 — 작업 규칙

## 1. 역할 분리

메인 스레드(Claude)가 **직접** 하는 것은 네 가지뿐이다.

1. **계약 작성** — `todo!()` 스켈레톤, `tests/` 계약 테스트, `tasks/NN-*.md` 태스크 명세
2. **판정** — `cargo test` / `cargo clippy` 결과 해석, diff 검토
3. **git commit**
4. **사용자 보고**

나머지는 전부 Task 서브에이전트에 위임한다. 위임 범위는
**bai-gjc 실행 → `.gjc-logs` 폴링 → 로그 확인 → `cargo test` → `cargo clippy` → 실패 시 수정 재지시(최대 2회)**.

메인은 서브에이전트로부터 **200토큰 이내 요약만** 받는다. 로그 원문·파일 전체·diff 전체를 메인
컨텍스트로 가져오지 않는다. 길면 서브에이전트가 파일로 남기고 경로만 보고한다.

## 2. 코드 생성 워커 — bai-gjc

맨 `gjc` 를 쓰지 않는다. 그건 다른 프로필(`~/.gjc`)이다.

- PowerShell/cmd: `C:\Users\gyuha\bai-gjc-starter\bin\bai-gjc.cmd`
- git-bash: `C:\Users\gyuha\bai-gjc-starter\bin\bai-gjc`
- PATH 에 `bai-gjc` 가 있으면 같은 파일이다.

기본형:

```
bai-gjc -p --no-session --model bai/glm-5.3-flash "<태스크 프롬프트>" *> .gjc-logs\NN-<이름>.log
```

- **모델**: 기본 `bai/glm-5.3-flash`(executor, 최저가). 어려운 태스크만 `bai/deepseek-v4.1-flash`(default).
  둘 다 실패해도 `--model` 을 바꿔가며 재시도하지 않는다 — `~/.gjc-bai/agent/config.yml` 의
  `fallbackChains` 가 B.AI 안에서 자동으로 넘어간다.
- **프리플라이트**: `BAI_API_KEY` 가 비면 런처가 즉시 exit 1 이다. 첫 호출 전에 존재를 확인하고,
  없으면 사용자에게 알리고 중단한다.
- **타임아웃**: 런처가 `PI_STREAM_FIRST_EVENT_TIMEOUT_MS=300000` 을 세팅한다. reasoning 모델이라
  첫 토큰이 5분까지 안 올 수 있다. 서브에이전트가 성급하게 죽이지 않도록 위임 프롬프트에 명시한다.
- 런처가 `ANTHROPIC_*` / `OPENAI_API_KEY` / `OPENROUTER_API_KEY` 를 지우고 실행한다. 이 동작을 우회하지 않는다.
- 로그: `.gjc-logs\NN-<이름>.log`. bai-gjc 자체 세션 기록은 `~/.gjc-bai/agent/sessions/`,
  로그는 `~/.gjc-bai/logs/`.

B.AI 는 90% 할인 모델이지 무료가 아니다. 싸지만 크레딧이 나간다.

## 3. 서브에이전트 위임 프롬프트에 반드시 넣을 것

1. 태스크 파일 **절대경로**
2. 완료 기준 **명령 원문**
3. 재시도 **최대 2회**
4. 첫 응답까지 **최대 5분 대기**
5. 보고 형식: 성공/실패 + 마지막 테스트 결과 줄 + 실패 시 원인 1줄 + 생성/수정한 파일 목록

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
