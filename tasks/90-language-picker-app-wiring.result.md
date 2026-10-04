# Task 90 — Codex 최종 판정

## 판정

**통과 (누적 호출 2/2).** 추가 작업자 호출은 하지 않는다.

## 확인한 내용

- Language 화면을 열 때 `I18n::available`을 한 번 호출하고, 같은 카드 언어 디렉터리에서 실제 load에
  성공한 code와 self-name만 picker에 넣는다. 잘못된 카드 `en.ftl`은 내장 영어로 복구한다.
- picker의 current는 저장 requested 값이 아니라 startup `UiCtx`의 effective language에서 온다.
- unchanged 선택은 저장·context 재생성 없이 Shelf로 돌아가며, changed 선택만 one-shot request를 남긴다.
- 공용 `service_language_request`가 candidate `UiCtx`를 만든 뒤 effective code를 확인하고,
  `Card::write_language` 성공 뒤에만 context를 교체한다. host/device가 이 순서를 복제하지 않는다.
- load mismatch와 save 실패에서 기존 context, current language, 카드 값을 유지하고 Language 화면에
  남아 각각의 toast를 표시한다.
- 성공 시 새 `UiCtx`의 profile/font directory는 유지되고 이전 face cache는 재사용하지 않는다.
- pending request 동안 Tap 입력은 선행 guard가 버린다. 나머지 Action 종류도 Language 화면에서 별도
  전역 동작과 매칭되지 않아 catch-all로 무시된다.
- Language 화면은 picker panel 하나만 그리고, toast/hold bar/HUD의 기존 z-order를 유지한다.
- 영문·한글 load/save 실패 문구가 Fluent를 통해 제공된다.

## 검증 근거

- 집중 테스트: `language_picker_app` **9 passed / 0 failed / 0 ignored**
- `slot2` 전체: **334 passed / 0 failed / 0 ignored**
- `slot2-i18n` 전체: **34 passed / 0 failed / 0 ignored**
- device check, fmt, clippy: 모두 종료 코드 0
- 최종 검증 뒤 코드 변경이 없다.

위 수치는 작업자 보고서의 완료 후 실행 결과다. Codex는 프로젝트 규칙에 따라 테스트를 다시 실행하지
않고 App 상태 전이, host/device 호출 위치, load→save→swap 순서와 실패 보존 경계를 대조했다.

## 남은 범위

- 언어팩 `lang-font` preferred font 적용
- M5의 한국어 문구 전수 검토, 글꼴 subset/지연 로딩, 혼합 언어 정렬과 특수문자 처리
- 언어 전환과 카드 언어팩의 실기 표시 확인은 사용자 검증 항목이다.
