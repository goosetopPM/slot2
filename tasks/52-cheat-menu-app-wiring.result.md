# Task 52 최종 검토 — 치트 메뉴 App 연결

## 판정

**통과** — 누적 호출 1/2.

`Screen::Cheats(InGameMenu, CheatMenu)`가 인게임 Cheats 행과 연결됐다. 실행 중 Session의 목록
길이로 메뉴를 열며, Up/Down은 탐색만 하고 A는 현재 enabled의 반대 값을
`Session::set_cheat_enabled`에 한 번 전달한다. 성공 후 같은 화면·선택에 머물고, B/MENU는 Cheats
행이 선택된 부모 메뉴로 돌아간다. 빈 목록과 Session이 없는 비정상 상태도 panic 없이 처리한다.

치트 화면은 마지막 게임 frame 위에 합성되고 wallpaper·shelf·시간 제어 HUD를 그리지 않는다.
core frame과 audio는 정지하며 hold progress와 toast는 메뉴 위에 남는다. 토글은 Session에만 적용돼
`.cht` bytes를 바꾸지 않는다. 손상된 치트 파일의 `Error::Store`는 일반 cart 오류와 구분해
`cheat-load-failed`로 표시한다.

## 검증 근거

- `cargo fmt --all -- --check` — 종료 0.
- `cargo test -p slot2 --test cheat_menu_app` — 11 passed, 0 failed, 종료 0.
- `cargo test -p slot2-i18n --test i18n` — 25 passed, 0 failed, 종료 0.
- `cargo clippy -p slot2 --all-targets -- -D warnings` — 종료 0.
- 현재 환경의 mGBA 실코어로 신규 App 테스트가 skip 없이 실행됐다.
- 열기·빈 목록·순환·긴 목록·양방향 토글·재진입·B/MENU 복귀·pause·draw 순서·파일 불변을
  확인했다.
- 화면과 Session 길이를 의도적으로 다르게 구성해 범위 밖 선택이 이전 상태를 보존하고
  `cheat-toggle-failed`를 메뉴 위에 표시하는 오류 경로를 확인했다.
- 손상 `.cht` launch가 Session 없이 `cheat-load-failed`를 표시하고 `cart-broken`으로 뭉개지지
  않음을 확인했다.
- Codex가 관련 App 구현과 집중 테스트를 대조했고 지정 파일의 `git diff --check`에 새 공백 오류가
  없다.

## 남은 비차단 사항

- `Session::start`에서 `Error::Store`를 만드는 현재 호출은 `read_cheats`뿐이다. 앞으로 다른 필수
  store 읽기가 추가되면 launch toast 분류를 더 세분해야 한다.
- libretro API가 코드 수락 여부를 반환하지 않으므로 D-21의 코어별 형식 사전 검증은 후속 태스크다.
