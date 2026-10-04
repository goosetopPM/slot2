# Task 77 — 게임별 오버레이 선택 메뉴 UI

현재 checkout에서 직접 작업한다. `GameSettings::overlay: Option<bool>`의 세 저장 의미를 표시하고
선택할 수 있는 독립적인 `OverlayMenu` UI를 `slot2-ui`에 추가한다. 이번 태스크는 **UI 컴포넌트와
번역만** 다룬다. Display 메뉴의 진입 행, App의 즉시 미리보기와 카드 저장은 후속 태스크로 분리한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\77-overlay-menu-ui.md`
- `C:\SLOT2\tasks\74-game-overlay-settings-store.result.md`
- `C:\SLOT2\crates\slot2-ui\src\overscan_menu.rs`
- `C:\SLOT2\crates\slot2-ui\tests\overscan_menu.rs`
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`의 module 선언과 re-export 주변만
- `C:\SLOT2\crates\slot2-store\src\settings.rs`의 `GameSettings::overlay` 정의와 주석만
- `C:\SLOT2\assets\lang\en.ftl`, `ko.ftl` 및 i18n 직접 정의 테스트의 Display/overscan key 주변만

직접 필요한 shared draw/span helper만 추가로 읽는다. App, runtime overlay loader/layer, Session,
DisplayMenu 구현, 실제 PNG asset, 다른 태스크·보고서·로그와 저장소 이력은 읽지 않는다.

## 현재 계약

- 카드 값 `None`은 플랫폼 기본값 상속, `Some(true)`는 이 게임에서 오버레이 사용,
  `Some(false)`는 이 게임에서 명시적으로 끄기다.
- 현재 플랫폼 기본값은 오버레이 없음이지만 `None`을 `Some(false)`로 합치지 않는다. 향후 플랫폼
  기본값이 생기면 `None`인 게임만 그 변경을 따라야 한다.
- 오버레이 그림은 플랫폼×geometry에 하나다. 이 버전에는 preset 이름이나 경로를 선택하는 UI가 없다.
- 이 UI는 저장 의미만 다룬다. 현재 플랫폼·geometry에 실제 PNG가 존재하는지 검사하거나,
  `None`을 계산된 bool로 바꾸지 않는다. 그림이 없더라도 사용자가 고른 `Some(true)`는 유효한 선택이다.

## 구현 계약

### 1. public 선택 모델

- `slot2-ui`에 public `overlay_menu` module과 main `OverlayMenu` type을 추가하고 crate root에서
  re-export한다.
- 새 enum을 만들지 않는다. 행 값과 public selected query는 기존 `Option<bool>`을 그대로 사용한다.
- 선택지는 다음 세 개를 정확히 이 순서로 둔다.
  1. `None` — Platform default
  2. `Some(true)` — On
  3. `Some(false)` — Off
- 세 값을 public `ROWS: [Option<bool>; 3]`에 두어 draw, navigation과 테스트가 같은 순서를 사용한다.
- `OverlayMenu::new(current)`는 정확히 해당 행을 선택하고 `selected()`는 highlighted row의 정확한
  `Option<bool>`을 반환한다.
- Up/Down은 양 끝에서 wrap한다. 입력 event, filesystem, asset resolution, platform lookup과 App state는
  컴포넌트에 넣지 않는다.
- key mapping은 세 값을 exhaustive match한다. bool을 문자열로 format하거나 row index에 의존하지 않는다.

### 2. overlay draw

- 기존 Overscan/Shader/Display/InGame 메뉴의 palette, text 크기, safe-area와 face cache 방식을 재사용한다.
- game frame과 runtime overlay 위에 그릴 메뉴 컴포넌트이므로 `Canvas::clear`를 호출하지 않는다.
- draw 순서는 physical panel 전체의 `BLACK alpha 0.6` dim, safe area 중앙 panel, title, 세 rows,
  select/back hints다. 정확히 한 행만 highlight한다.
- layout 상수는 `BOX_W = 360`, `BOX_H = 208`, `PAD = 16`, `ROW_H = 36`으로 둔다. row 시작은
  `box_y + PAD + 28`이며 title/rows/highlight/hints가 640×480 safe area 안에 있어야 한다.
- 같은 menu와 language를 warm 상태에서 반복 draw하고 highlight를 옮겨도 새 glyph texture upload가
  없어야 한다.

### 3. 표시 문구

영문/한글 pack에 다음 key를 정확히 직접 정의한다. 첫 행은 같은 저장 의미의 기존
`display-platform-default`를 재사용한다.

| key | English | 한국어 |
|---|---|---|
| `overlay-title` | `Overlay` | `오버레이` |
| `overlay-on` | `On` | `켜기` |
| `overlay-off` | `Off` | `끄기` |

Korean fallback이 누락을 숨기지 않도록 i18n 직접 정의 테스트를 추가한다. 기존 번역 key의 문구를
바꾸거나 새 generic On/Off key를 만들지 않는다.

## 테스트 계약

`crates/slot2-ui/tests/overlay_menu.rs`에 집중 테스트를 추가한다.

- `None`, `Some(true)`, `Some(false)` 각각의 constructor와 selected round-trip
- Platform default와 explicit Off가 별도 행이라는 단언
- 고정 순서대로 Up/Down이 이동하고 양 끝에서 wrap함
- clear 없이 dim → panel 순서이며 정확히 하나의 selected-row highlight만 존재함
- `rgsp`, `rg35xxsp`, `rgcubexx`의 영문/한글에서 box, title, 세 labels, highlight와 hints가
  640×480 safe area 안에 있음
- 세 행이 정확한 localized label을 자기 row에 그림
- 동일 menu warm redraw 8회 및 highlight 이동 뒤 `UploadAlpha8`/`UploadRgba8`가 0개임
- public rows가 중복 없이 정확한 세 값/순서이며 key mapping도 중복되지 않음
- 영문/한글 pack이 신규 key를 fallback 없이 직접 정의함

RecordingCanvas와 geometry 단언만 사용한다. GPU/GL window, screenshot golden, 실제 PNG와 texture id
단언은 쓰지 않는다. 기존 Display/Shader/Overscan menu 테스트를 수정하거나 약화하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-ui\src\overlay_menu.rs` (신규)
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`의 module/re-export 두 줄
- `C:\SLOT2\crates\slot2-ui\tests\overlay_menu.rs` (신규)
- `C:\SLOT2\assets\lang\en.ftl`
- `C:\SLOT2\assets\lang\ko.ftl`
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`의 신규 key 직접 정의 테스트만
- `C:\SLOT2\tasks\77-overlay-menu-ui.worker-result.md`

다른 production/test 파일은 수정하지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- 기존 `DisplayMenu`의 rows, layout, API 변경과 Overlay 진입 행 추가
- App screen variant, input wiring, live overlay source 교체와 settings 저장
- save-failure rollback/toast, launch/core switch/eject 수명 변경
- store, runtime overlay resolver/layer, Session, gfx, retro registry 변경
- production `BUILT_IN_OVERLAYS` entry와 실제 `assets/overlays/` PNG 추가
- generic settings/menu framework, animation과 새로운 input gesture
- 전체 workspace 테스트, GL 창, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-ui --test overlay_menu
cargo test -p slot2-ui -p slot2-i18n
cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test,
GL test와 device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\77-overlay-menu-ui.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- 세 값/행 순서, constructor/query/navigation과 draw 결과
- 영문/한글 문구, safe-area와 warm redraw 검증
- 각 완료 기준 명령, 종료 코드와 마지막 결과 줄
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- Display/App 진입·저장·live preview와 실제 PNG가 범위 밖으로 남았다는 확인
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
