# Task 52 — 치트 메뉴 App 연결

## 목적

Task 50의 Session 치트 상태와 Task 51의 `CheatMenu`를 App 화면 상태에 연결한다. 인게임 메뉴의
Cheats 행에서 목록을 열고, A로 현재 항목을 세션 안에서 즉시 on/off하며, 게임 프레임 위에 결과를
바로 표시한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 모두 횟수에 포함한다.

## 먼저 읽을 범위

- `AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `tasks/50-session-cheat-application.result.md`
- `tasks/51-cheat-menu-ui.result.md`
- `crates/slot2/src/app.rs`의 `Screen`, 인게임/Display 입력, `start_launch`, draw 주변만
- `crates/slot2/tests/display_menu_app.rs`의 실코어 fixture와 하위 메뉴 전환 패턴
- `crates/slot2-ui/src/cheat_menu.rs`의 공개 API만
- `assets/lang/en.ftl`, `assets/lang/ko.ftl`의 인게임 메뉴·오류 문구 주변
- `crates/slot2-i18n/tests/i18n.rs`의 관련 메시지 테스트만

## 구현 계약

### 1. 화면 상태와 열기

- `Screen`에 부모 `InGameMenu`와 `CheatMenu`를 함께 보존하는 치트 하위 화면을 추가한다. 권장 형태:

```rust
Cheats(InGameMenu, CheatMenu)
```

- `InGameChoice::Cheats`에서 A를 누르면 실행 중 Session의 `cheats().len()`으로 새 `CheatMenu`를
  만들어 연다.
- Session이 없으면 화면을 바꾸지 않는다. 임의의 빈 Session이나 가짜 목록을 만들지 않는다.
- 메뉴를 새로 열 때 선택은 Task 51 계약대로 index 0에서 시작한다. B 또는 MENU로 닫았다가 다시
  열어도 현재 enabled 값은 Session에서 읽되 탐색 선택은 0에서 새로 시작한다.

### 2. 입력과 즉시 토글

- 치트 화면에서 Up/Down은 `CheatMenu::up/down`만 호출한다.
- A는 `selected_index()`가 있을 때 현재 Session 항목의 enabled 값을 읽고 반대 값으로
  `Session::set_cheat_enabled`를 정확히 한 번 호출한다.
- 성공하면 같은 치트 화면과 같은 선택에 머문다. draw는 Session slice를 읽으므로 새 on/off 상태가
  즉시 보인다.
- 빈 목록에서 A는 no-op이며 오류 토스트를 만들지 않는다.
- B 또는 MENU는 보존한 부모 `InGameMenu`로 돌아가며 Cheats 행 선택을 유지한다.
- 다른 버튼은 무시한다. 입력이 게임 core로 전달되거나 프레임을 전진시키면 안 된다.
- 토글 실패 시 Session의 이전 상태를 유지하고 메뉴도 열린 채로 두며, stderr에 cart/index와 오류를
  남기고 localized `cheat-toggle-failed` 토스트를 표시한다.

### 3. launch 오류 분류

- Task 50에서 손상되거나 읽을 수 없는 `.cht`는 `Session::start`의 `Error::Store`로 launch를
  실패시킨다. 이를 일반 `cart-broken`으로 뭉개지 말고 localized `cheat-load-failed` 토스트로
  분류한다. `$title`을 전달해 어느 게임인지 알 수 있게 한다.
- core 없음과 다른 Retro 오류의 기존 토스트 분류는 변경하지 않는다.

### 4. 그리기와 실행 정지

- 치트 화면은 Playing/InGame/Display/Switcher와 같은 게임 overlay 계열이다.
  - wallpaper와 shelf를 그리지 않는다.
  - Session의 마지막 게임 frame을 먼저 그린다.
  - HUD 시간 제어 배지는 숨긴다.
  - 그 위에 `CheatMenu::draw(canvas, ctx, session.cheats())`를 그린다.
  - hold progress와 toast는 치트 메뉴보다 위에 유지한다.
- 치트 화면에 머무는 동안 기존 메뉴와 마찬가지로 core frame과 게임 audio를 진행하지 않는다.
- Session이 비정상적으로 사라진 상태에서도 draw와 입력이 panic하지 않아야 한다.

### 5. 저장 정책

- `.cht` 파일은 읽기조차 다시 하지 않고 수정하지도 않는다. Task 50의 Session 상태만 사용한다.
- B/MENU 복귀, 메뉴 재진입, draw는 카드 I/O를 일으키지 않는다.
- 세션 토글은 다음 launch에 영속화하지 않는다.

## 테스트 계약

새 `crates/slot2/tests/cheat_menu_app.rs`에서 기존 MIT test ROM과 mGBA 실코어 fixture를 사용한다.
코어가 없는 환경은 기존 방식으로 명시적으로 skip하되, 현재 개발 환경에서는 실제 실행한다.

최소 다음을 직접 확인한다.

- Cheats 행 A가 Session 목록 길이의 치트 화면을 열고 index 0을 선택함
- 빈 목록도 화면은 열리며 선택 없음, A no-op, Session과 toast 불변
- Up/Down 순환과 긴 목록 스크롤 선택이 App screen 안에서 유지됨
- A 토글 성공 후 같은 index/화면 유지, Session enabled 즉시 반전, 반대 방향 재토글 가능
- 여러 항목 중 하나를 토글해도 나머지는 유지되고 이후 게임으로 돌아가 프레임 실행 가능
- 토글과 탐색·닫기·재열기가 `.cht` bytes를 변경하지 않으며 재열기 선택은 0, 상태는 Session 값
- B와 MENU가 각각 Cheats 행이 선택된 부모 인게임 메뉴로 복귀
- 치트 화면에서 `run_frame`을 반복 호출해도 Session `frames_run`과 audio 진행이 없음
- 마지막 게임 frame → dim/menu 순서로 그려지고 wallpaper/shelf/clear가 없으며 toast는 메뉴 위
- Session이 없는 비정상 치트 화면에서 A/draw가 panic하지 않음
- screen/menu 길이 불일치를 제품 hook 없이 직접 구성해 범위 밖 선택 오류를 유도하고,
  `cheat-toggle-failed` 토스트·이전 Session 상태·열린 화면을 확인
- 손상된 `.cht` launch가 `cheat-load-failed` 토스트를 내며 `cart-broken`으로 표시되지 않음
- `cheat-toggle-failed`, `cheat-load-failed`의 영문·한국어 문구와 `$title` 치환 확인

테스트 좌표나 draw 횟수를 맞추기 위한 제품 분기와 mock hook는 금지한다.

## 수정 허용 범위

- `crates/slot2/src/app.rs`
- 새 파일 `crates/slot2/tests/cheat_menu_app.rs`
- `assets/lang/en.ftl`
- `assets/lang/ko.ftl`
- 새 문구 검증에 꼭 필요한 범위의 `crates/slot2-i18n/tests/i18n.rs`
- `tasks/52-cheat-menu-app-wiring.worker-result.md`

그 밖의 소스는 수정하지 않는다. 특히 `session.rs`, `slot2-ui`, `slot2-store`, `slot2-retro`, host/device
loop와 공용 GJC 설정은 건드리지 않는다.

## 금지·범위 밖

- 커밋·푸시
- 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- `~/.gjc-bai/agent/*.yml` 및 공용 설정 수정
- `.cht` 쓰기와 토글 영속화
- 코어별 치트 형식 validator/quirks
- 치트 검색·편집·다운로드
- 워크스페이스 전체 테스트와 배포 빌드

## 완료 기준

최종 코드 상태에서 아래를 원문 그대로 순서대로 실행한다.

```text
cargo fmt --all -- --check
cargo test -p slot2 --test cheat_menu_app
cargo test -p slot2-i18n --test i18n
cargo clippy -p slot2 --all-targets -- -D warnings
```

각 명령의 종료 코드와 마지막 결과 줄을 기록한다. 검증 뒤 소스를 고치면 네 명령을 다시 실행한다.
첫 응답까지 최대 5분, 전체 최대 45분 기다린다. 진행 신호는 로그 크기가 아니라
`git status --short`다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\52-cheat-menu-app-wiring.worker-result.md`를 작성한다.

- 성공/실패와 누적 호출 횟수
- Screen 상태와 열기·닫기·재진입 동작
- 실제 Session 토글과 실패 rollback/toast 결과
- 손상 파일 launch 오류 분류
- pause·draw 순서·파일 불변 검증
- 실코어 실행 또는 skip 여부
- 네 검증 명령의 종료 코드와 마지막 결과 줄
- 생성·수정 파일 목록
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄

실패해도 반드시 원인과 마지막 상태를 쓴다. 두 번째 호출도 실패하면 더 시도하지 않는다.
