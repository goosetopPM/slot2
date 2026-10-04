# Task 93 — Codex 최종 판정

## 판정

**통과 (누적 호출 1/2).** 추가 작업자 호출은 필요하지 않다.

## 확인한 내용

- `Card::scan`의 production 로직은 바뀌지 않았다. 기존 UTF-8 stem 보존과 Rust `str` 정렬을 새 계약
  테스트 6개로 봉인했다.
- ASCII 대문자/소문자, 가나, 한자, 한글이 Unicode scalar value 순서로 정렬되고 파일 생성 순서에
  영향을 받지 않는다. `Alpha < alpha`와 `가 < 각 < 나`도 직접 단언한다.
- 마지막 허용 확장자 하나만 제거하며 공백, 다중 점, 괄호, 대괄호, 아포스트로피, `+`, `&`, `!`,
  `#`, `%`, `@`, `_`, `-`, 한글·일본어가 stem/title과 ROM path에 그대로 보존된다.
- 같은 exact stem이 label, save, states, game settings, cheat 다섯 부속 경로에 사용되고 save/state
  round-trip까지 확인한다. 플랫폼이 다르면 모든 경로가 격리된다.
- 숨김 파일, AppleDouble, directory, unsupported final extension은 제외되고 확장자의 ASCII 대소문자만
  무시한다.
- 같은 플랫폼의 동일 stem `.sfc`/`.smc`는 결정적인 확장자 순서로 카트 둘을 표시하며, 기존 카드
  호환 계약에 따라 다섯 부속 경로를 공유한다.
- module 문서와 DESIGN에 위 정책과 제한을 기록했고 M5의 혼합 정렬/특수문자 항목을 완료로 반영했다.
- Codex 검토에서 동작 변경 없이 DESIGN의 `코드포인트(= UTF-8 바이트)` 표현을 Unicode scalar value
  순서와 UTF-8 비교의 순서 보존 관계로 정확히 고쳤다.

## 검증 근거

- 신규 `card_filename_contract`: **6 passed / 0 failed / 0 ignored**
- 기존 `card`: **19 passed / 0 failed / 0 ignored**
- `slot2-store` 전체: **118 passed / 0 failed / 0 ignored**
- host/device check, fmt, clippy: 모두 종료 코드 0, clippy warning 0
- core 의존 skip 없음, 작업자 최종 검증 뒤 production/test 코드 변경 없음

위 수치는 작업자 보고서의 완료 후 실행 결과다. Codex는 프로젝트 규칙에 따라 테스트를 다시 실행하지
않고 신규 테스트의 각 단언, module 문서, DESIGN과 milestone 변경을 대조했다.

## 남은 범위

- Noto Sans KR subset 생성 스크립트와 V-10 실기 지연 로딩 시간·메모리 측정
- M5 시간대 항목은 Task80~84 구현 근거와 milestone 표기의 일치 여부 정리가 남아 있다.
- 한국어 전 화면 실기 순회와 M5 Acceptance는 사용자 검증 항목이다.
