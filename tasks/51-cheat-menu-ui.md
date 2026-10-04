# Task 51 — 인게임 치트 메뉴 UI

## 목적

Task 50의 Session 치트 목록을 나중에 표시·토글할 수 있도록 `slot2-ui`에 순수 치트 메뉴 화면을
만든다. 이 태스크는 선택·스크롤·그리기와 영문/한국어 문구만 담당한다. App 입력과 실제 Session
토글은 다음 태스크에서 연결한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 모두 횟수에 포함한다.

## 먼저 읽을 범위

- `AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `tasks/50-session-cheat-application.result.md`
- `crates/slot2-ui/src/in_game_menu.rs`
- `crates/slot2-ui/src/display_menu.rs`와 해당 테스트에서 레이아웃·warm redraw 방식만
- `crates/slot2-ui/src/lib.rs`
- `crates/slot2-store/src/cheats.rs`의 공개 `Cheat` 구조체만
- `assets/lang/en.ftl`, `assets/lang/ko.ftl`의 인게임 메뉴 주변
- `crates/slot2-i18n/tests/i18n.rs`의 키 동등성 계약

## 구현 계약

### 1. 공개 모델

- `slot2-ui`에 별도 `cheat_menu` 모듈과 공개 `CheatMenu`를 추가한다.
- 메뉴는 Session 목록을 복제하거나 enabled 값을 소유하지 않는다. 목록 길이, 현재 선택, 첫 visible
  index처럼 화면 탐색에 필요한 값만 소유하고, 그리기는 호출자가 준 `&[slot2_store::Cheat]`를
  읽는다.
- 권장 API는 다음과 같다. 기존 UI 관례에 맞는 동등한 형태는 허용한다.

```rust
pub fn new(len: usize) -> Self;
pub fn selected_index(&self) -> Option<usize>;
pub fn up(&mut self);
pub fn down(&mut self);
pub fn draw(
    &self,
    canvas: &mut dyn Canvas,
    ctx: &mut UiCtx,
    cheats: &[slot2_store::Cheat],
);
```

- Session 한 개의 치트 개수는 메뉴가 열린 동안 바뀌지 않는다는 Task 50 계약을 사용한다. draw에
  다른 길이가 들어와도 panic하거나 범위 밖 접근하지 말고 안전하게 현재 slice에 맞춰 표시한다.
- 빈 목록은 선택이 `None`이며 up/down이 no-op다.
- 항목이 있으면 처음에는 index 0이 선택된다. up/down은 전체 목록 끝에서 반대쪽으로 순환한다.

### 2. 스크롤

- 한 화면에 표시할 고정 최대 행 수를 정하고 공개 상수로 둔다. 640×480 안전영역에서도 제목,
  목록, 힌트가 겹치지 않는 5~7행 범위로 한다.
- 선택은 항상 visible window 안에 있어야 한다. 아래·위 이동, 양 끝 wrap 후에도 맞아야 한다.
- 목록이 화면보다 길면 위나 아래에 더 있다는 시각적 표시를 제공한다. 단순 화살표 또는 동등한
  작은 표시면 충분하며, 실제로 숨은 방향에만 표시한다.

### 3. 그리기

- 현재 인게임/Display 메뉴와 같은 dim overlay, 중앙 panel, palette, safe-area 좌표계를 사용한다.
- 게임 프레임 위에 합성하므로 `canvas.clear`를 호출하지 않는다.
- 제목은 인게임 메뉴의 `ingame-cheats` 번역을 재사용해도 된다.
- 각 행은 왼쪽에 `.cht`의 description, 오른쪽에 localized enabled 상태를 표시한다.
- 선택 행은 기존 메뉴와 같은 highlight를 사용한다. enabled/disabled는 색만으로 구분하지 말고
  반드시 영문/한국어 텍스트도 표시한다.
- 빈 목록은 중앙에 localized empty 문구를 표시하고 highlight를 그리지 않는다.
- 하단 힌트는 목록이 있을 때 A toggle과 B back, 빈 목록일 때 B back만 표시한다.
- description은 임의 길이와 UTF-8일 수 있다. 상태 열과 panel 밖으로 침범하지 않도록 표시 폭에
  맞춰 Unicode 경계를 지키며 말줄임한다. byte 중간을 자르거나 panic하지 않는다. 짧은 문자열은
  바꾸지 않는다.
- 같은 상태를 반복 draw할 때 새 텍스처 업로드가 발생하지 않는 기존 face cache 성질을 유지한다.

### 4. 번역

영문과 한국어에 동일한 새 message key를 직접 추가한다. 최소 의미:

- 빈 목록: 이 게임에는 치트가 없음
- enabled / disabled 상태
- A로 토글하는 힌트

버튼은 직접 문자열로 붙이지 말고 기존 `BTN` placeholder를 사용한다. 기존 `hint-back`은 재사용한다.
다른 언어 fallback과 영·한 key parity를 깨지 않는다.

## 테스트 계약

`RecordingCanvas` 기반 집중 테스트로 최소 다음을 직접 확인한다.

- 0개에서 선택 없음, up/down no-op, localized empty 문구, highlight 없음, B 힌트만 표시
- 1개와 여러 항목에서 초기 선택, 위·아래 순환
- visible 최대치보다 긴 목록에서 양방향 이동과 wrap 때 선택이 항상 보이고 hidden 방향 표시가 맞음
- description과 enabled 상태가 같은 행의 좌·우에 나오며 on/off 텍스트가 영문·한국어에서 정확함
- 긴 ASCII와 긴 한글 description이 상태 열/panel 경계를 침범하지 않고 유효 UTF-8 말줄임됨
- 640×480, 720×480, 720×720 세 geometry와 en/ko에서 panel·행·힌트가 safe area 안에 있음
- draw가 기존 게임 frame을 지우지 않음
- 동일 상태 warm redraw가 텍스처를 추가 업로드하지 않음
- 영문/한국어 번역 키 동등성 테스트 통과

좌표나 draw 횟수를 맞추기 위한 의미 없는 추가 그리기는 금지한다. 테스트는 사용자에게 보이는
계약을 검증해야 한다.

## 수정 허용 범위

- 새 파일 `crates/slot2-ui/src/cheat_menu.rs`
- `crates/slot2-ui/src/lib.rs`
- 새 파일 `crates/slot2-ui/tests/cheat_menu.rs`
- `assets/lang/en.ftl`
- `assets/lang/ko.ftl`
- 새 키 검증에 꼭 필요할 때만 `crates/slot2-i18n/tests/i18n.rs`
- `tasks/51-cheat-menu-ui.worker-result.md`

그 밖의 소스는 수정하지 않는다. 특히 `slot2/src/app.rs`, `slot2/src/session.rs`, `slot2-retro`,
`slot2-store` 구현과 공용 GJC 설정은 건드리지 않는다.

## 금지·범위 밖

- 커밋·푸시
- 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- `~/.gjc-bai/agent/*.yml` 및 공용 설정 수정
- App screen state와 버튼 입력 연결
- 실제 `Session::set_cheat_enabled` 호출
- `.cht` 쓰기 또는 enable 영속화
- 코어별 치트 형식 검증
- 워크스페이스 전체 테스트와 배포 빌드

## 완료 기준

최종 코드 상태에서 아래를 원문 그대로 순서대로 실행한다.

```text
cargo fmt --all -- --check
cargo test -p slot2-ui --test cheat_menu
cargo test -p slot2-i18n --test i18n
cargo clippy -p slot2-ui --all-targets -- -D warnings
```

각 명령의 종료 코드와 마지막 결과 줄을 기록한다. 검증 뒤 소스를 고치면 네 명령을 다시 실행한다.
첫 응답까지 최대 5분, 전체 최대 45분 기다린다. 진행 신호는 로그 크기가 아니라
`git status --short`다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\51-cheat-menu-ui.worker-result.md`를 작성한다.

- 성공/실패와 누적 호출 횟수
- 공개 API와 선택·스크롤 불변식
- 빈 상태·행·상태·힌트·말줄임의 최종 표현
- 세 geometry와 en/ko 검증 결과
- warm redraw 업로드 결과
- 네 검증 명령의 종료 코드와 마지막 결과 줄
- 생성·수정 파일 목록
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄

실패해도 반드시 원인과 마지막 상태를 쓴다. 두 번째 호출도 실패하면 더 시도하지 않는다.
