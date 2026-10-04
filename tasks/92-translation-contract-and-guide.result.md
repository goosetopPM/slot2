# Task 92 — Codex 최종 판정

## 판정

**통과 (누적 호출 1/2).** 추가 작업자 호출은 필요하지 않다.

## 확인한 내용

- 내장 en/ko pack은 각각 125개 key이며 duplicate, missing, extra가 없다.
- 같은 key의 변수 이름 집합과 BTN id 순서·개수가 일치하고 모든 BTN id가 runtime에 등록돼 있다.
- 영어에는 JOSA가 없고, 한국어 JOSA는 message 변수와 지원 조사 pair만 사용한다.
- literal button cap과 허용되지 않은 함수 호출을 거부하며, 내장 두 pack 모두 실제 parse/load된다.
- test-only scanner는 multiline select의 닫는 brace를 포함하고, 이해하지 못하는 top-level line이나 함수
  형태를 조용히 건너뛰지 않는다. production loader와 card partial fallback은 바꾸지 않았다.
- 한국어 125개 문구의 의미·용어·조사·버튼 힌트를 전수 감사했다. 사용자 문구 수정은 필요 없었고,
  영문 변수 목록과 한국어에 남아 있던 영문 주석만 현재 내용에 맞게 고쳤다.
- `docs/TRANSLATING.md`에 canonical/complete/partial 관계, card pack 추가, `lang-font`, BTN, JOSA,
  fallback, 검증 및 font license 규칙을 기록했다.
- Codex 검토에서 변수 집합과 BTN 목록의 규칙을 더 정확히 표현하고, 새 내장 언어를 추가할 때
  `pack_contract` 검사 대상도 확장하도록 가이드 문구만 정정했다.
- M5의 한국어 전수 번역, 언어팩 override/font 지정, 번역 기여 가이드 세 항목을 완료로 반영했다.

## 검증 근거

- `pack_contract`: **8 passed / 0 failed / 0 ignored**
- `slot2-i18n` 전체: **42 passed / 0 failed / 0 ignored**
- `slot2-ui` 전체: **263 passed / 0 failed / 0 ignored**
- `slot2` 전체: **335 passed / 0 failed / 0 ignored**
- device check, fmt, clippy: 모두 종료 코드 0
- 작업자 최종 검증 뒤 소스 변경이 없다. 이후 Codex 변경은 실행 동작과 무관한 가이드 문구 세 곳뿐이다.

위 수치는 작업자 보고서의 완료 후 실행 결과다. Codex는 프로젝트 규칙에 따라 테스트를 다시 실행하지
않고 scanner의 message grouping, 변수·함수 추출, 각 단언과 가이드 내용을 대조했다.

## 남은 범위

- Noto Sans KR subset 생성 스크립트와 V-10 실기 지연 로딩 시간·메모리 측정
- 한·영 혼합 정렬 정책과 파일명 특수문자 처리
- 한국어 전 화면 실기 순회와 M5 Acceptance
