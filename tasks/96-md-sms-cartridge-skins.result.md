# Task 96 — Codex 최종 판정

## 판정

**통과 (누적 호출 1/2).** 추가 작업자 호출은 필요하지 않다.

## 확인한 내용

- MD와 SMS에 각각 독립 shell/detail SVG가 추가됐고 `PlatformSkin`에서 `borrowed: false`로
  연결됐다. 크기·label rect·shell 색상은 명세와 일치한다.
- 7개 플랫폼의 cart와 detail source가 각각 모든 쌍에서 다르며, 마지막 fallback과 호출자가 없어진
  `borrowing` helper가 제거됐다. `PlatformSkin` public shape와 기존 다섯 플랫폼 값은 유지됐다.
- 새 SVG는 투명 배경의 흰 coverage 도형이며 text·logo·bitmap·외부 참조가 없다. MD는 넓고 낮은
  stepped-top/taper 형상, SMS는 세로형 cut-corner/stepped-foot 형상으로 구분된다.
- Task95 테스트가 7개 플랫폼 독립성, viewBox, detail coverage와 네 자체 제작 카트의 label·raster·
  shell transparency·aspect 계약으로 일반화됐다. 기존 단언은 삭제되거나 약화되지 않았다.
- provenance, DESIGN §7, MILESTONES M3/M6가 실제 구현 범위에 맞게 갱신됐다. port·curve·sfx는
  완료로 표시하지 않았다.

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

- 플랫폼별 port SVG와 `PlatformSkin` port 계약
- 삽입·배출 곡선과 효과음
- 7개 카트 비율·색·label 크기의 실기 표시 확인

`borrowed` 필드는 현재 모든 row에서 false지만 public shape 보존 계약 때문에 유지됐다. 제거 여부는
별도 API 정리 태스크에서 호출자와 문서 영향을 검토해야 한다.
