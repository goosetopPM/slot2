# Task 94 — Codex 최종 판정

## 판정

**통과 (누적 호출 1/2).** 추가 작업자 호출은 필요하지 않다.

## 확인한 내용

- production 코드는 바뀌지 않았고 `language_picker_app`에 M5 Acceptance용 통합 테스트 하나만
  추가됐다.
- 카드 `System/Lang/ja.ftl`은 `lang-name`, `power-off`, `resume` 정확히 세 message만 가지며 picker가
  `en`, `ja`, `ko` 순서와 self-name `日本語`를 표시한다.
- 실제 App 입력으로 `ja`를 선택하고 공용 `service_language_request`를 처리한 뒤 effective `UiCtx`,
  App current, 카드 저장값이 모두 `ja`다.
- 카드가 정의한 두 문구는 일본어로 나오고 누락된 canonical key는 내장 영어로 fallback한다. 성공 뒤
  Language 행의 parent Shelf로 돌아가며 failure toast가 없다.
- Task80~84의 store/startup/UI/preview·저장·취소·실패 rollback 근거에 맞춰 M5 시간대 항목을
  완료 처리했다. M3 HUD와 DESIGN의 오래된 미래형 문구도 현재 동작으로 고쳤다.
- TRANSLATING은 언어 picker의 simple code 순서와 ROM 파일명의 Unicode scalar 정렬·exact stem 정책을
  분리해 설명한다.
- Noto subset, V-10 실기 측정, 한국어 전 화면 실기 순회는 완료로 표시하지 않았다.

## 검증 근거

- `language_picker_app`: **11 passed / 0 failed / 0 ignored**
- `timezone_startup_app`: **1 passed / 0 failed / 0 ignored**
- `timezone_menu_app`: **1 passed / 0 failed / 0 ignored**
- `slot2-i18n`: **42 passed / 0 failed / 0 ignored**
- `timezone_settings`: **12 passed / 0 failed / 0 ignored**
- device check, fmt, clippy: 모두 종료 코드 0, clippy warning 0
- core 의존 skip 없음, 작업자 최종 검증 뒤 code/test 변경 없음

위 수치는 작업자 보고서의 완료 후 실행 결과다. Codex는 프로젝트 규칙에 따라 테스트를 다시 실행하지
않고 신규 App test의 상태 전이와 문서 변경 범위를 대조했다.

## 남은 범위

- Noto Sans KR subset 생성 스크립트는 host에 `pyftsubset`/fontTools가 없어 계속 보류한다.
- V-10의 RG SP 로딩 시간·메모리와 한국어 전 화면 표시 품질은 사용자 실기 검증 항목이다.
- 위 두 항목 전에는 M5 전체를 완료로 간주하지 않는다.
