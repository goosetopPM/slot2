# Task 88 — Codex 최종 판정

## 판정

**통과 (누적 호출 2/2).** 첫 검토에서 발견한 로그 escaping 결함까지 수정됐다.

## 확인한 내용

- `requested_language(card, env_override)`는 `Some`을 철자 그대로 사용하고 `None`에서만 카드 설정을
  읽는다. 환경변수를 건드리지 않고 precedence를 테스트할 수 있다.
- `Boot`가 `SLOT2_LANG` 부재와 명시값을 구분한다. 비 Unicode 값도 카드로 조용히 내려가지 않고
  명시적인 실패 요청으로 취급돼 내장 영어 fallback으로 간다.
- host/device 모두 Card 생성·layout 확인 → requested 결정 → `UiCtx` 생성 → 같은 Card를 App에 전달하는
  순서를 사용한다. device input probe도 동일한 시작 언어 context를 받는다.
- env override → 카드 저장값 → store 기본 `en`의 요청 우선순위와, 실제 pack load 실패 → 내장 `en`의
  effective fallback이 분리돼 있다.
- built-in, 카드 전용, built-in override, unknown, malformed pack, invalid UTF-8 설정에서 원본을
  고치지 않고 계약대로 부팅한다.
- 1차 결함이었던 effective code raw 로그를 수정해 최종 형식이
  `requested={requested:?} effective={effective:?}`가 됐다. 두 필드의 제어문자가 모두 한 로그 줄 안에
  escape된다.

## 검증 근거

- 집중 테스트: `language_startup` **10 passed / 0 failed / 0 ignored**
- `slot2` 전체: **325 passed / 0 failed / 0 ignored**
- core-dependent skip: **0**
- device check, fmt, clippy: 모두 종료 코드 0
- 2차 수정 뒤 위 검증을 다시 실행했고 이후 코드 변경이 없다.

위 수치는 작업자 누적 2/2 보고서의 최종 실행 결과다. Codex는 프로젝트 규칙에 따라 테스트를 다시
실행하지 않고 구현·수정 범위와 보고서를 대조했다.

## 남은 범위

- Shelf Language 행과 언어 picker UI
- 실행 중 `UiCtx`/font chain 교체와 texture cache 정리
- 선택 언어 저장, 저장 실패 rollback/toast
- `I18n::available` 기반 목록과 언어팩 `lang-font` 적용
