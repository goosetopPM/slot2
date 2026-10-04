# Task 72 — 게임별 오버스캔 선택 메뉴 UI

현재 checkout에서 직접 작업한다. 기존 `GameSettings::overscan: Option<bool>`의 세 저장 의미를 표시하고
선택할 수 있는 독립적인 `OverscanMenu` UI를 `slot2-ui`에 추가한다. 이번 태스크는 **UI 컴포넌트와
번역만** 다룬다. NES에서만 Display 진입 행을 노출하고 App/Session에 연결하는 작업은 Task73으로 분리한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\72-overscan-menu-ui.md`
- `C:\SLOT2\crates\slot2-ui\src\shader_menu.rs`
- `C:\SLOT2\crates\slot2-ui\tests\shader_menu.rs`
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`의 module/re-export 주변만
- `C:\SLOT2\crates\slot2-store\src\settings.rs`의 `GameSettings::overscan`과 bool ini 의미만
- `C:\SLOT2\crates\slot2\src\session.rs`의 launch overscan 초기화와 setter 주석만
- `C:\SLOT2\crates\slot2-retro\src\registry.rs`의 `Overscan`과 platform def 주변만
- `C:\SLOT2\assets\lang\en.ftl`, `ko.ftl` 및 i18n 직접 정의 테스트의 Display/shader key 주변만

직접 필요한 shared draw/span helper만 추가로 읽는다. App, 다른 Session 코드, 실제 core/ROM 테스트,
다른 태스크·보고서·로그와 저장소 이력은 읽지 않는다.

## 현재 계약

- 카드 값 `None`은 platform default 상속, `Some(true)`는 명시적 crop, `Some(false)`는 명시적으로 전체
  이미지를 보여 달라는 뜻이다.
- 현재 platform default는 registry의 `PlatformDef::overscan`을 적용하는 것이다. NES만 상하 8줄을
  crop하고 나머지 여섯 platform은 `Overscan::NONE`이다.
- `None`과 `Some(true)`는 NES에서 같은 화면을 만들 수 있지만 카드 의미가 다르다. `None`은 향후
  platform 기본값 변경을 따르고 explicit true는 게임별 결정이므로 별도 행으로 보존한다.
- 이 UI는 stored meaning만 다룬다. platform을 보고 값을 계산하거나 NES 여부를 판단하지 않는다.
  후속 App이 실제 crop이 있는 platform에서만 진입점을 노출한다.

## 구현 계약

### 1. public 선택 모델

- `slot2-ui`에 public `overscan_menu` module과 main `OverscanMenu` type을 추가하고 crate root에서
  re-export한다.
- 새 enum을 만들지 않는다. 행 값과 public selected query는 기존 `Option<bool>`을 그대로 사용한다.
- 선택지는 다음 세 개를 정확히 이 순서로 둔다.
  1. `None` — Platform default
  2. `Some(true)` — Crop edges
  3. `Some(false)` — Show full image
- 세 값을 public `ROWS: [Option<bool>; 3]`에 두어 draw, navigation과 테스트가 같은 순서를 사용한다.
- `OverscanMenu::new(current)`는 정확히 해당 행을 선택하고 `selected()`는 highlighted row의 정확한
  `Option<bool>`을 반환한다.
- Up/Down은 양 끝에서 wrap한다. 입력 event, filesystem, platform lookup과 App state는 넣지 않는다.
- key mapping은 세 값을 exhaustive match한다. bool을 문자열로 format하거나 index에 의존하지 않는다.

### 2. overlay draw

- 기존 Shader/Display/InGame 메뉴의 palette, text size, safe-area와 face cache 방식을 재사용한다.
- game frame 위 overlay이므로 `Canvas::clear`를 호출하지 않는다.
- draw 순서는 physical panel 전체 `BLACK alpha 0.6` dim, safe area 중앙 panel, title, 세 rows,
  select/back hints다. 정확히 한 행만 highlight한다.
- layout 상수는 `BOX_W = 360`, `BOX_H = 208`, `PAD = 16`, `ROW_H = 36`으로 둔다. row 시작은
  `box_y + PAD + 28`이며 title/rows/highlight/hints가 640×480 safe area 안에 있어야 한다.
- 같은 menu/language를 warm 상태에서 반복 draw하고 highlight를 옮겨도 새 glyph texture upload가 없어야
  한다.

### 3. 표시 문구

영문/한글 pack에 다음 key를 정확히 직접 정의한다. 첫 행은 같은 의미의 기존
`display-platform-default`를 재사용한다.

| key | English | 한국어 |
|---|---|---|
| `overscan-title` | `Overscan` | `오버스캔` |
| `overscan-crop` | `Crop edges` | `가장자리 자르기` |
| `overscan-full` | `Show full image` | `전체 이미지 표시` |

Korean fallback이 누락을 숨기지 않도록 i18n 직접 정의 테스트를 추가한다.

## 테스트 계약

`crates/slot2-ui/tests/overscan_menu.rs`에 집중 테스트를 추가한다.

- `None`, `Some(true)`, `Some(false)` 각각의 constructor와 selected round-trip
- Platform default와 explicit crop이 별도 행이라는 단언
- 고정 순서대로 Up/Down이 이동하고 양 끝에서 wrap함
- clear 없이 dim → panel 순서이며 정확히 하나의 selected-row highlight만 존재함
- `rgsp`, `rg35xxsp`, `rgcubexx` × 영문/한글에서 box, title, 세 labels, highlight와 hints가 safe area 안
- 세 행이 정확한 localized label을 자기 row에 그림
- 동일 menu warm redraw 8회 및 highlight 이동 뒤 texture upload 0개
- public rows가 중복 없이 정확한 세 값/순서이며 key mapping도 중복되지 않음
- 영문/한글 pack이 신규 key를 fallback 없이 직접 정의함

RecordingCanvas와 geometry 단언만 사용한다. GPU/GL window, screenshot golden과 texture id 단언은 쓰지
않는다. 기존 Display/Shader menu 테스트를 수정하거나 약화하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-ui\src\overscan_menu.rs` (신규)
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`의 module/re-export 두 줄
- `C:\SLOT2\crates\slot2-ui\tests\overscan_menu.rs` (신규)
- `C:\SLOT2\assets\lang\en.ftl`
- `C:\SLOT2\assets\lang\ko.ftl`
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`의 신규 key 직접 정의 테스트만
- `C:\SLOT2\tasks\72-overscan-menu-ui.worker-result.md`

다른 production/test 파일은 수정하지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- 기존 Display/Shader menu의 rows, layout, API 변경
- App screen variant, NES 조건부 진입 행, input wiring, live crop와 settings 저장
- Session, store, gfx, retro registry와 core option 변경
- overlay selector, generic settings framework, animation과 새로운 gesture
- 전체 workspace 테스트, GL 창, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-ui --test overscan_menu
cargo test -p slot2-ui -p slot2-i18n
cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test, GL test와
device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\72-overscan-menu-ui.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- 세 값/행 순서, constructor/query/navigation과 draw 결과
- 영문/한글 문구, safe-area와 warm redraw 검증
- 각 완료 기준 명령, 종료 코드와 마지막 결과 줄
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- NES 조건부 App 진입·저장·live crop이 범위 밖으로 남았다는 확인
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
