# Task 97 — Codex 최종 판정

## 판정

**통과 (누적 호출 1/2).** 추가 작업자 호출은 필요하지 않다.

## 확인한 내용

- 7개 플랫폼별 port trim SVG가 추가됐고 `PlatformSkin`에 `port`와 `port_size`가 연결됐다. 모든
  source와 raster coverage가 플랫폼별로 구분된다.
- port 폭은 각 cart mouth에 좌우 20px trim을 더한 값이고 높이는 `MOUTH_H`와 같다. 각 mask의 중앙
  seated-cart 노출 구역은 투명하며 양옆 trim에는 coverage가 있다.
- `ShelfView`는 port를 natural size로 `ArtCache`에 저장하고 panel 중앙의 `band_y`에 `LIP` tint로
  그린다. draw와 draw_insert가 같은 경로를 사용하며 빈 선반에서도 해당 플랫폼 port가 표시된다.
- draw order는 back/slit → cart → procedural front band/lip → port다. 기존 bay·band·lip과
  occlusion/seat/travel 수학은 유지됐고, port는 cart 뒤에 그려지면서 투명 opening으로 중앙 카트를
  계속 보이게 한다.
- 빈 선반 플랫폼 전환, 반복 redraw/scroll cache, insert front order, 3개 geometry×7개 platform
  panel bounds 계약이 추가됐다. 기존 테스트는 삭제되거나 약화되지 않았다.
- provenance, DESIGN §7, MILESTONES M3/M6가 실제 범위에 맞게 갱신됐고 curve·sfx는 완료로 표시하지
  않았다.

## 검증 근거

- `skin`: **21 passed / 0 failed / 0 ignored**
- `shelf_draw`: **13 passed / 0 failed / 0 ignored**
- `insert`: **21 passed / 0 failed / 0 ignored**
- `slot2-ui` 전체: **280 passed / 0 failed / 0 ignored**
- device check, fmt, clippy: 모두 종료 코드 0, clippy warning 0

위 수치는 작업자 보고서의 최종 변경 후 실행 결과다. Codex는 프로젝트 규칙에 따라 테스트를 다시
실행하지 않고 7개 SVG 원문, table 값, draw/cache 경로와 계약 테스트를 대조했다.

## 남은 범위

- 플랫폼별 삽입·배출 곡선
- 플랫폼별 삽입·배출 효과음과 재생 연결
- 실제 기기에서 port 색·정렬·카트 노출 확인

DESIGN §7의 구조 예시는 아직 구현되지 않은 `mouth_rect`, `seat_depth`, curve와 sfx까지 포함한 목표
모델이다. 이번 태스크의 실제 API는 `port`와 `port_size`이며 나머지는 후속 범위다.
