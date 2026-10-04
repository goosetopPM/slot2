# Task 51 최종 검토 — 인게임 치트 메뉴 UI

## 판정

**통과** — 누적 호출 2/2.

`CheatMenu`는 Session 목록을 복제하지 않고 길이·선택·visible window만 소유한다. 빈 목록은 선택
없이 B 힌트만 표시하고, 목록이 있으면 양 끝 순환과 최대 6개 행 스크롤, 숨은 방향 표시, 설명과
localized on/off 상태, A 토글·B 뒤로 힌트를 제공한다. draw는 게임 프레임을 지우지 않는다.

2회차에서 긴 description의 prefix를 매번 다시 측정하던 제곱 시간 탐색을 제거했다. UTF-8 문자
경계를 한 번 수집하고 실제 font 폭으로 가장 긴 prefix를 이분 탐색한다. ellipsis조차 들어가지 않는
극소 폭은 빈 문자열로 처리한다. 위쪽 hidden-row 막대도 제목 line box와 첫 행 사이의 실제 빈
대역으로 옮겼다.

## 검증 근거

- `cargo fmt --all -- --check` — 종료 0.
- `cargo test -p slot2-ui --test cheat_menu` — 16 passed, 0 failed, 종료 0.
- `cargo test -p slot2-i18n --test i18n` — 24 passed, 0 failed, 종료 0.
- `cargo clippy -p slot2-ui --all-targets -- -D warnings` — 종료 0.
- 기존 13개 집중 테스트를 유지하고 5만 자 ASCII·4만 자 한글, 극소 폭, 위 막대 간격 테스트 3개를
  추가했다.
- 640×480, 720×480, 720×720과 en/ko에서 safe area·제목/행/힌트·막대 간격을 확인했다.
- 반복 draw와 선택 이동의 warm 상태에서 추가 텍스처 업로드가 없다.
- Codex가 2회차 구현과 테스트를 대조했고 지정 파일의 `git diff --check`에 새 공백 오류가 없다.

## 남은 비차단 사항

- 위 막대 공간은 현재 배포 폰트의 line height로 검증됐다. 훗날 PX_BODY line height가 제목 영역을
  거의 채우는 다른 폰트를 허용한다면 제목 영역 또는 행 시작 위치도 함께 늘려야 한다.
- App 화면 상태와 `Session::set_cheat_enabled` 연결은 후속 태스크다.
