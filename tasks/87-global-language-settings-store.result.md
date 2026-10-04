# Task 87 — Codex 최종 판정

## 판정

**통과 (누적 호출 1/2).** 추가 수정 호출은 필요하지 않다.

## 확인한 내용

- `LANGUAGE_KEY = "language"`, `DEFAULT_LANGUAGE = "en"`과 `Card::read_language` /
  `Card::write_language`가 crate root에서 계약대로 제공된다.
- read와 write가 같은 private validator를 사용한다. UTF-8 byte 1..=64의 단일 파일 stem만 허용하고,
  빈 값·`.`·`..`·경로 구분자·공백·제어문자를 거부한다. 유효한 값은 대소문자와 철자를 보존한다.
- write는 잘못된 코드를 설정 파일을 읽기 전에 `Error::Invalid`로 거부한다. read는 누락·손상·읽기
  실패·잘못된 코드에서 파일을 바꾸지 않고 `en`으로 돌아간다.
- `en`은 키 부재로 저장되고, 다른 키가 없을 때만 빈 `slot2.ini`를 제거한다.
- 언어를 `GlobalSettings` 필드에 넣지 않아 volume debounce와 분리했다. language, volume,
  `utc_offset_minutes`, unknown key의 양방향 보존이 회귀 테스트로 고정됐다.
- store는 `System/Lang`, 환경변수, `slot2-i18n`을 참조하지 않는다. 설치 여부와 FTL parse 판정은
  후속 App/i18n 경계에 남아 있다.

## 검증 근거

- 집중 테스트: `language_settings` **11 passed / 0 failed**
- `slot2-store` 전체: **112 passed / 0 failed / 0 ignored**
- host/device `cargo check`, fmt, store clippy: 모두 종료 코드 0
- 기준 Task 80의 store 101개에 신규 11개가 더해져 테스트 수 감소가 없다.

위 수치는 작업자 보고서의 완료 후 실행 결과다. Codex는 프로젝트 규칙에 따라 테스트를 다시 실행하지
않고 구현의 저장·검증 경계와 보고서를 대조했다.

## 남은 범위

- App 시작 시 저장 언어를 적용하고 `SLOT2_LANG`과의 우선순위를 확정한다.
- 카드·내장 pack의 실제 가용성과 parse 실패 시 `en` fallback을 App/i18n 경계에서 처리한다.
- Shelf Language 행, 언어 선택 UI, runtime `UiCtx`/font chain 교체, 저장 실패 rollback/toast를 연결한다.
- `...` 같은 이름도 현재 명세상 유효하다. 후속 선택기는 `I18n::available`이 반환한 실제 pack만
  제시하므로 임의 코드를 생성하지 않아야 한다.
