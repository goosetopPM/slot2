# Task 89 — Codex 최종 판정

## 판정

**통과 (누적 호출 1/2).** 추가 수정 호출은 필요하지 않다.

## 확인한 내용

- `LanguageOption`이 code/name을 그대로 소유하고, `LanguagePicker`가 exact code 중복만 처음 항목으로
  축약하면서 caller 순서를 보존한다.
- 현재 언어 행과 선택 highlight가 분리돼 있다. 이동·wrap 뒤에도 current badge는 고정되고 선택이
  돌아오면 `changed()`가 false로 복구된다.
- 빈 목록, 한 행, current가 목록에 없는 경우가 명시적으로 처리되고 accessor/navigation이 안전하다.
- 6행 visible window가 긴 목록에서 선택을 따라가며 위·아래 wrap 뒤에도 highlight가 화면 안에 남는다.
- 행 이름은 caller가 넘긴 self-name을 사용하고 빈 이름은 code로 대체한다. 이름과 code/current 문장은
  고정 column 폭에 맞춰 Unicode char boundary를 지키며 말줄임한다.
- title, code/current, empty, position, select/back hint가 Fluent 계약을 사용한다. current 문구를 코드에서
  조립하지 않는다.
- 세 geometry에서 panel과 내용이 640×480 safe area 안에 있고 두 번째 draw가 texture upload를 만들지
  않는다.
- picker는 파일 시스템, pack discovery/load, Card/store, 환경변수, `UiCtx` 교체와 App에 의존하지 않는다.

## 검증 근거

- 집중 테스트: `language_picker` **11 passed / 0 failed / 0 ignored**
- `slot2-ui` 전체: **256 passed / 0 failed / 0 ignored**
- `slot2-i18n` 전체: **34 passed / 0 failed / 0 ignored**
- device check, fmt, UI/i18n clippy: 모두 종료 코드 0
- 최종 검증 뒤 코드 변경이 없다.

위 수치는 작업자 보고서의 완료 후 실행 결과다. Codex는 프로젝트 규칙에 따라 테스트를 다시 실행하지
않고 모델, window 계산, draw 경계와 보고서를 대조했다.

## 남은 범위

- `I18n::available/load`를 이용한 실제 후보 discovery와 load 실패 filtering
- Shelf Language 행 활성화와 A/B 입력 배선
- 선택 즉시 runtime `UiCtx`/i18n/font/face cache 교체
- `write_language` 저장과 실패 rollback/toast
- 언어팩 `lang-font` 적용
