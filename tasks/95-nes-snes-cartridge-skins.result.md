# Task 95 — Codex 최종 판정

## 판정

**통과 (누적 호출 1/2).** 추가 작업자 호출은 필요하지 않다.

## 확인한 내용

- NES와 SNES에 각각 독립 shell/detail SVG가 추가됐고 `PlatformSkin`에서 `borrowed: false`로
  연결됐다. 두 플랫폼의 크기·label rect·shell 색상은 명세와 일치한다.
- MD와 SMS만 GBA의 cart/detail/size/label을 빌리는 fallback으로 남았다.
- 새 SVG는 투명 배경의 흰 coverage 도형이며 text·logo·bitmap·외부 참조가 없다. NES는 세로형,
  SNES는 가로형으로 서로 다른 비율과 몰딩을 가진다.
- 기존 public API와 `ShelfView` drawing/caching은 바뀌지 않았다. 플랫폼별 port API·MD/SMS 자산·
  삽입 곡선·효과음은 완료로 표시하지 않았다.
- provenance, DESIGN §7, MILESTONES M3/M6 진행 문구가 실제 구현 범위에 맞게 갱신됐다.
- 기존 borrow 테스트는 현재 사실에 맞게 이름을 바꾸고 detail·label 단언을 늘렸으므로 삭제나 약화가
  아니다.

## 검증 근거

- `skin`: **16 passed / 0 failed / 0 ignored**
- `shelf_draw`: **10 passed / 0 failed / 0 ignored**
- `insert`: **18 passed / 0 failed / 0 ignored**
- `label`: **20 passed / 0 failed / 0 ignored**
- `slot2-ui` 전체: **269 passed / 0 failed / 0 ignored**
- device check, fmt, clippy: 모두 종료 코드 0, clippy warning 0

위 수치는 작업자 보고서의 최종 변경 후 실행 결과다. Codex는 프로젝트 규칙에 따라 테스트를 다시
실행하지 않고 스킨 테이블, SVG 원문, 계약 테스트와 문서 변경 범위를 대조했다.

## 남은 범위

- MD·SMS 독립 카트리지 자산과 fallback 해제
- 플랫폼별 port SVG와 `PlatformSkin` port 계약
- 삽입·배출 곡선과 효과음
- 새 비율과 색상의 실기 표시 확인은 사용자 검증 항목
