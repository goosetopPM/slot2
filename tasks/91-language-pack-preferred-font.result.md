# Task 91 — Codex 최종 판정

## 판정

**통과 (누적 호출 2/2).** 추가 작업자 호출은 하지 않는다.

## 확인한 내용

- `UiCtx::new`가 요청 문자열이 아니라 effective `I18n::font()`를 사용해 preferred font를 정한다.
- `lang-font`는 안전한 단일 파일명만 허용하고, `font_dirs`의 caller 순서대로 첫 regular file을 고른다.
  경로 구분자, absolute/prefix, `.`/`..`와 missing 이름은 기본 체인으로 fallback한다.
- 폰트 체인은 preferred(lazy) → embedded OpenSans(eager) → CJK(lazy) 순서이며, preferred와 CJK의
  선택 경로가 같으면 한 번만 등록한다.
- 한국어 pack의 잘못된 `.ttf` 이름을 실제 배포 파일 `NotoSansKR-Regular.otf`로 바로잡았다.
- corrupt preferred는 lazy slot으로 유지되지만, glyph와 line metrics 모두 뒤의 첫 정상 font로
  fallback한다. 실제 raster bitmap의 높이와 ink도 보존된다.
- runtime 언어 전환 후 새 context가 card preferred를 사용하고 이전 face cache를 재사용하지 않는다.
  missing preferred도 언어 load/save/context swap 성공을 막지 않는다.
- 한국어 Noto의 더 큰 line box를 수용하도록 Cheat 메뉴 제목 영역을 30px로 조정했고, 영어·한국어와
  세 geometry에서 title/bar/row/hint clearance를 유지한다.
- DESIGN과 D-14의 오래된 `font=` 표기를 실제 Fluent key인 `lang-font`로 정정했다.

## 검증 근거

- `slot2-text`: **12 passed / 0 failed / 0 ignored**
- `language_font` + `cheat_menu`: **23 passed / 0 failed / 0 ignored**
- `slot2-ui` 전체: **263 passed / 0 failed / 0 ignored**
- `slot2-i18n` 전체: **34 passed / 0 failed / 0 ignored**
- `language_startup` + `language_picker_app`: **20 passed / 0 failed / 0 ignored**
- device check, fmt, clippy: 모두 종료 코드 0
- 최종 검증 뒤 코드 변경이 없다.

위 수치는 작업자 보고서의 완료 후 실행 결과다. Codex는 프로젝트 규칙에 따라 테스트를 다시 실행하지
않고 preferred 탐색 경계, font chain 순서, failed-slot metrics fallback, 실제 bitmap 단언과 Cheat 메뉴
layout을 대조했다.

## 남은 범위

- Noto Sans KR subset 생성 스크립트와 V-10 실기 지연 로딩 시간·메모리 측정
- 한국어 문구 전수 검토와 한·영 혼합 정렬·파일명 특수문자 처리
- `TITLE_AREA_H=30`은 배포 OpenSans/Noto metrics를 수용한다. 더 큰 line box를 가진 서드파티 폰트까지
  지원하려면 측정값 기반 adaptive menu layout이 별도 작업으로 필요하다.
