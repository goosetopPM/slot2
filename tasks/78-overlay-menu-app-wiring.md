# Task 78 — Display 오버레이 메뉴 App 배선

현재 checkout에서 직접 작업한다. 기존 Display 화면에 Overlay 진입 행을 추가하고 Task77의
`OverlayMenu`를 App 상태기계에 연결한다. 선택한 게임별 overlay 설정은 **카드 저장 성공 뒤에만** 실행
중 `OverlayLayer` source에 즉시 반영한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\78-overlay-menu-app-wiring.md`
- `C:\SLOT2\tasks\76-overlay-app-runtime-wiring.result.md`
- `C:\SLOT2\tasks\77-overlay-menu-ui.result.md`
- `C:\SLOT2\crates\slot2-ui\src\display_menu.rs`
- `C:\SLOT2\crates\slot2-ui\src\overlay_menu.rs`
- `C:\SLOT2\crates\slot2-ui\tests\display_menu.rs`
- `C:\SLOT2\crates\slot2\src\app.rs`의 `Screen`, pause/audio guard, Display/Shader/Overscan 입력·helper와
  overlay resolve/draw 주변만
- `C:\SLOT2\crates\slot2\src\overlay.rs`의 setting/geometry helper, resolver와 `OverlayLayer` public API만
- `C:\SLOT2\crates\slot2\tests\display_menu_app.rs`의 Display screen fixture 패턴만
- `C:\SLOT2\crates\slot2\tests\overlay_app.rs`의 합성 GBA, 카드 PNG, draw/lifecycle fixture 패턴만
- `C:\SLOT2\crates\slot2-store\src\settings.rs`의 `GameSettings::overlay` 의미만
- `C:\SLOT2\assets\lang\en.ftl`, `ko.ftl`과 i18n 직접 정의 테스트의 overlay key 주변만

직접 필요한 test helper만 추가로 읽는다. PNG decoder 내부, Session/core 구현, gfx/GL 구현, 다른 App
테스트, 다른 태스크·워커 로그와 저장소 이력은 읽지 않는다.

## 현재 계약

- `GameSettings::overlay`는 `None`(platform default), `Some(true)`(explicit on), `Some(false)`(explicit
  off)의 세 저장 의미를 가진다. 현재 platform default는 off지만 `None`과 explicit off는 합치지 않는다.
- Task76은 성공한 launch에서 setting과 geometry를 한 번 resolve하고 game → overlay → HUD/UI 순서로
  그린다. core switch/recovery는 texture를 유지하고 stop/eject는 lazy free한다.
- Task77의 `OverlayMenu`는 세 저장 의미를 정확히 표시하지만 아직 어디에서도 열리지 않는다.
- overlay asset은 플랫폼×geometry에 하나다. On을 골라도 asset이 없거나 깨졌으면 게임은 계속되고 그림만
  생략한다. 이것은 save failure가 아니며 사용자의 On 선택을 지우거나 Off로 바꾸지 않는다.
- 카드 저장은 전체 `GameSettings`의 read-modify-write다. write 실패 전에 runtime source를 바꾸면 다음
  launch와 현재 화면이 어긋나므로 반드시 저장이 먼저다.

## 구현 계약

### 1. Display 화면의 Overlay 진입 행

- `DisplayChoice`에 `Overlay` variant를 추가한다.
- Display rows는 기존 행의 상대 순서를 유지하고 Overlay를 마지막에 둔다.
  - crop 없음: Platform default, Integer, Aspect fit, Fill, Shader, Overlay — 6행
  - crop 있음: Platform default, Integer, Aspect fit, Fill, Shader, Overscan, Overlay — 7행
- non-crop 높이는 기존 280에서 **316**, crop 높이는 기존 316에서 **352**로 한 행씩 늘린다.
  `BOX_W`, `PAD`, `ROW_H`, row 시작과 동적 `choices()`/`box_h()` 경계를 유지한다.
- Overlay label은 Task77의 `overlay-title`을 재사용한다. 중복 번역 key를 만들지 않는다.
- Overlay 행은 실제 PNG 존재 여부와 관계없이 두 mode 모두 제공한다. 이 행은 asset browser가 아니라
  게임별 저장 의도를 고르는 곳이며, 카드에 PNG를 나중에 추가해도 저장한 On이 살아 있어야 한다.
- 기존 scale constructor, Shader/Overscan 행과 navigation 의미를 바꾸지 않는다. `DisplayChoice`와 menu의
  Copy/Debug/Eq 계약을 유지한다.

### 2. App screen과 navigation

- `Screen`에 `Overlay(InGameMenu, DisplayMenu, OverlayMenu)` variant를 추가한다. parent menu 둘을
  보존해 B/MENU가 같은 Display Overlay 행으로 돌아오게 하고 `Screen` Copy/Debug/Eq를 유지한다.
- Display A는 `Scale`, `Shader`, `Overscan`의 기존 동작을 그대로 유지하고 `Overlay`이면 active cart의
  현재 `settings.overlay`로 Overlay screen을 연다.
- active Session이 없으면 Overlay screen을 열거나 카드 I/O를 하지 않는다.
- Overlay screen에서 Up/Down은 세 행을 wrap한다. B/MENU는 settings/runtime을 바꾸지 않고 parent
  Display의 Overlay 행으로 돌아간다. 다른 ordinary button은 아무 동작도 하지 않는다.
- Overlay screen은 다른 인게임 submenu처럼 core frame/audio를 pause하고 sink open/close 요청을 만들지
  않는다. global power/volume은 기존 전역 처리 그대로 유지한다.
- App의 화면 전이·draw 문서에서 Overlay menu가 빠진 부분과 Display의 낡은 row 설명만 현재 사실에
  맞게 고친다.

### 3. 저장 후 runtime source 즉시 반영

- Overlay 화면의 A는 active cart의 전체 settings를 읽고 **overlay 필드만** `menu.selected()` 값으로
  바꿔 `Card::write_settings`를 호출한다. core, scale, overscan, rewind, shader와 unknown key를 보존한다.
- 저장 성공 뒤에만 selected setting을 기존 `overlay_enabled`, `geometry_for_panel`, `overlay::resolve`와
  App의 `OverlayLayer`에 적용한다. App에 bool 의미, geometry mapping, path precedence를 중복 구현하지
  않는다.
- launch와 runtime commit이 같은 private source-retarget helper를 사용하도록 기존 `resolve_overlay`를
  좁게 정리해도 된다. launch는 settings를 한 번 읽고, runtime commit은 이미 저장에 성공한 selected
  값을 직접 전달한다. frame마다 settings/filesystem을 읽지 않는다.
- `Some(true)`이고 geometry가 지원되면 현재 cart/platform/geometry source를 `set_sources`한다.
  다음 draw에서 이전 texture를 필요하면 한 번 free하고 새 source를 decode/upload한 뒤, game frame과
  OverlayMenu dim/panel 사이에 그림을 표시한다.
- `None`과 `Some(false)`, 미지원 geometry는 `clear_sources`한다. 다음 draw에서 기존 texture를 한 번
  free하고 overlay image 없이 game → OverlayMenu 순서로 그린다.
- 같은 setting/source를 다시 선택하면 `OverlayLayer::set_sources`의 동일값 no-op을 보존해 texture를
  free/redecode/reupload하지 않는다.
- source가 missing/corrupt/wrong-size이면 저장 성공과 selected row를 유지한다. 게임과 메뉴는 정상
  draw되고 overlay image만 없다. asset decode 실패를 settings rollback이나 failure toast로 바꾸지 않는다.
- 성공하면 Overlay 화면과 선택 행을 유지한다. success toast, core frame advance, Session/core restart,
  game texture 변경과 audio/sink 변경을 만들지 않는다.
- write 실패 시 card bytes와 기존 runtime source/texture를 유지하고 attempted row의 Overlay 화면에
  남는다. concise error를 한 번 log하고 `overlay-save-failed` toast를 표시한다. pending free/reupload도
  만들지 않는다.
- 신규 번역은 정확히 다음과 같다.
  - English: `overlay-save-failed = Could not save the overlay setting`
  - 한국어: `overlay-save-failed = 오버레이 설정을 저장하지 못했습니다`

### 4. draw와 lifecycle 정합성

- Overlay screen draw 순서는 정확히 game frame → 현재 runtime overlay image(있을 때) → OverlayMenu
  dim/panel → hold progress/toast다. parent Display/InGame menu, wallpaper, time-control badge와 새 clear를
  사이에 넣지 않는다.
- Overlay screen 추가 때문에 생기는 모든 exhaustive pause/draw/menu match를 같은 인게임 submenu
  범주에 정확히 추가한다.
- core switch/recovery와 stop/eject의 기존 overlay 수명을 바꾸지 않는다. 설정 변경은 현재 playthrough의
  source만 retarget하며 Session과 game texture를 소유하지 않는다.

## 테스트 계약

### Display UI

기존 `display_menu` 테스트를 새 row 계약으로 보강하되 단언을 약화하지 않는다.

- crop 없음은 기존 5행 뒤 Overlay를 추가한 6행/높이 316이고 양방향 wrap이 6행에서 동작함
- crop 있음은 기존 6행 뒤 Overlay를 추가한 7행/높이 352이고 양방향 wrap이 7행에서 동작함
- 두 mode에서 기존 행의 상대 순서와 scale constructor가 유지되고 Overlay가 마지막 행임
- 세 geometry·두 언어에서 두 높이 모두 safe-area, localized label, 단일 highlight와 warm redraw를 보존함

### App 집중 테스트

`crates/slot2/tests/overlay_menu_app.rs`를 새로 만들고 기존 두 fixture의 필요한 패턴만 좁게 재사용한다.
최소한 다음을 직접 검증한다.

- crop 없는 GBA와 crop 있는 합성 NES 또는 UI-state fixture 모두 Display 마지막 행에서 Overlay를 열며,
  현재 `None`/true/false가 정확한 row를 선택함
- B/MENU는 같은 Display Overlay 행으로 돌아오고 card bytes/runtime source를 바꾸지 않음
- 세 선택이 정확히 저장됨: None은 overlay key만 제거하고 overlay-only ini를 제거하며, true/false는
  canonical on/off로 남음
- core/scale/overscan/rewind/shader와 hand-written unknown key가 set/clear 뒤에도 보존됨
- exact-size card PNG가 있는 `Some(true)` 선택은 다음 draw에서 game → overlay image → OverlayMenu 순서로
  즉시 나타나고, false/None 선택은 다음 draw에서 texture를 정확히 한 번 free한 뒤 image를 그리지 않음
- off/None에서 다시 true를 고르면 새 source를 한 번 upload하고, 이미 true인 같은 source를 다시 저장하면
  free/redecode/reupload하지 않음
- setting 전환 중 core frame/state, Session identity, game texture upload/update/free, audio와 sink가 불변임
- missing/corrupt/wrong-size asset에서 true 저장은 성공하고 유지되며 game/menu는 정상 draw되고 overlay
  image와 save-failure toast만 없음
- write failure와 unreadable original은 prior exact bytes와 runtime texture/source를 보존하고 attempted row에
  남아 `overlay-save-failed` toast를 표시하며 free/upload를 만들지 않음
- 닫고 다시 열면 마지막 성공 저장값이 선택됨
- Overlay screen은 frame/audio를 pause하며 session/sink를 유지하고 draw에 parent menus, wallpaper,
  time-control badge와 clear가 없음
- 기존 launch/core switch/recovery/stop overlay tests와 Display scale/Shader/Overscan tests가 계속 통과함
- 영문/한글 failure key가 fallback 없이 직접 정의됨

PNG는 테스트 임시 card에 생성한다. repository asset을 추가하거나 texture id 숫자를 맞추기 위한 padding
upload/draw를 만들지 않는다. real core가 없으면 관련 case만 기존 형식으로 명시적 skip하고 정확한 수와
이유를 보고한다. 이 checkout에는 필요한 mGBA/FCEUmm library가 있으므로 정상 환경에서는 skip 0이어야
한다. ignore, 테스트 전용 production bypass와 단언 완화는 금지한다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-ui\src\display_menu.rs`
- `C:\SLOT2\crates\slot2-ui\tests\display_menu.rs`
- `C:\SLOT2\crates\slot2\src\app.rs`
- `C:\SLOT2\crates\slot2\tests\overlay_menu_app.rs` (신규)
- 새 Screen variant로 컴파일에 꼭 필요한 기존 `C:\SLOT2\crates\slot2\tests\overlay_app.rs`의 좁은 match 수정
- `C:\SLOT2\assets\lang\en.ftl`
- `C:\SLOT2\assets\lang\ko.ftl`
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`의 failure key 직접 정의 단언만
- `C:\SLOT2\tasks\78-overlay-menu-app-wiring.worker-result.md`

`overlay_menu.rs`, store, Session, runtime overlay resolver/layer, gfx, retro registry와 다른 production/test
파일은 수정하지 않는다. `slot2-ui/src/lib.rs`는 `DisplayChoice`와 `OverlayMenu`가 이미 export되어 있어
변경하지 않는다. 구현이 불가능하면 범위를 넓히지 말고 보고서에 이유를 적어 실패로 둔다. 관련 없는
미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- store ini 문법, platform default와 asset path/precedence 변경
- PNG decoder, Canvas, Session renderer, core switch/recovery와 stop/eject 재설계
- production `BUILT_IN_OVERLAYS` entry와 실제 `assets/overlays/` PNG 추가
- success toast, animation, sound, 새 gesture와 generic settings/menu framework
- 전체 workspace 테스트, 실제 GL 창, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-ui --test display_menu --test overlay_menu
cargo test -p slot2 --test display_menu_app --test overlay_app --test overlay_menu_app
cargo test -p slot2 -p slot2-ui -p slot2-i18n
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2 -p slot2-ui -p slot2-i18n --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 각 test 명령의 passed/failed/ignored 합계와 core-dependent skip 수를 보고한다.
검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test, 실제 GL test와 device 배포는
실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\78-overlay-menu-app-wiring.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- 두 Display row set/layout, Overlay screen 전이와 B/MENU 복귀 결과
- 세 setting의 저장 우선·runtime source 즉시 반영·동일 source no-op·write failure rollback 결과
- missing/broken asset의 선택 보존과 game/menu fallback 결과
- known/unknown key, 기존 menus, core/session/game texture/audio/sink와 lifecycle 보존 근거
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄과 core-dependent skip 수
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- actual built-in PNG와 실기 화질 확인이 범위 밖으로 남았다는 확인
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
