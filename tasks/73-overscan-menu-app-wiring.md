# Task 73 — Display 오버스캔 메뉴 App 배선

현재 checkout에서 직접 작업한다. registry에 실제 overscan crop이 있는 platform에서만 기존 Display
화면에 Overscan 진입 행을 추가하고 Task72의 `OverscanMenu`를 App 상태기계에 연결한다. 선택한 게임별
overscan 설정은 **카드 저장 성공 뒤에만** 실행 중 Session에 즉시 적용한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\73-overscan-menu-app-wiring.md`
- `C:\SLOT2\tasks\71-shader-menu-app-wiring.result.md`
- `C:\SLOT2\tasks\72-overscan-menu-ui.result.md`
- `C:\SLOT2\crates\slot2-ui\src\display_menu.rs`
- `C:\SLOT2\crates\slot2-ui\src\overscan_menu.rs`
- `C:\SLOT2\crates\slot2-ui\tests\display_menu.rs`
- `C:\SLOT2\crates\slot2\src\app.rs`의 `Screen`, pause/audio guard, Display/Shader helper와 draw 주변만
- `C:\SLOT2\crates\slot2\src\session.rs`의 platform 변환, launch overscan 초기화, draw와 setter 주변만
- `C:\SLOT2\crates\slot2\tests\display_menu_app.rs`
- `C:\SLOT2\crates\slot2\tests\session.rs`의 settings/overscan draw helper 주변만
- `C:\SLOT2\crates\slot2-retro\src\registry.rs`의 `Overscan`과 platform def 주변만
- `C:\SLOT2\crates\slot2\tests\core_picker_app.rs`의 합성 NES ROM/FCEUmm fixture helper만
- `C:\SLOT2\assets\lang\en.ftl`, `ko.ftl`과 i18n 직접 정의 테스트의 overscan key 주변만

직접 필요한 test helper만 추가로 읽는다. gfx/GL 구현, 다른 App 테스트, 다른 태스크·워커 로그와 저장소
이력은 읽지 않는다.

## 현재 계약

- `GameSettings::overscan`의 `None`은 platform default, `Some(true)`는 explicit crop,
  `Some(false)`는 full image다. Session launch는 현재 `None`을 crop enabled로 해석한다.
- registry에서 실제 nonzero crop을 가진 platform은 NES 하나이며 상하 8줄이다. 다른 platform에
  Overscan row를 보여주면 선택해도 화면이 바뀌지 않는 거짓 기능이 된다.
- Task72의 `OverscanMenu`는 세 stored meaning을 정확히 표시하지만 아직 어디에서도 열리지 않는다.
- 기존 Display에는 네 scale 선택과 Shader 진입 행이 있고 GBA/App 검증이 완료됐다. 비-cropping
  platform의 이 5행 순서·layout·navigation을 회귀시키지 않는다.
- 저장은 전체 `GameSettings`의 read-modify-write다. write 실패 전에 runtime crop을 바꾸지 않는다.

## 구현 계약

### 1. 조건부 Display 행과 동적 layout

- `DisplayChoice`에 `Overscan` variant를 추가한다.
- Display rows는 두 고정 배열로 표현한다.
  - crop 없음: Platform default, Integer, Aspect fit, Fill, Shader — 기존 5행
  - crop 있음: 위 5행 뒤 Overscan — 6행
- `DisplayMenu` constructor가 current scale과 `overscan_available: bool`을 받아 정확한 scale row에서
  시작한다. menu가 자기 availability를 보관하고 `choices()`가 현재 public row slice를 반환하며
  `choice()`/Up/Down/draw는 반드시 이 slice 하나를 사용한다.
- platform enum이나 NES 이름을 `slot2-ui`에 넣지 않는다. availability는 App이 registry로 계산한다.
- non-crop menu 높이는 기존 **280**, crop menu 높이는 **316**이다. menu-owned `box_h()`와 instance
  `box_origin(ctx)`/`row_y(ctx, i)`가 현재 row set의 높이를 사용하게 한다. `BOX_W`, `PAD`, `ROW_H`와
  row 시작은 유지한다.
- Overscan label은 Task72의 `overscan-title`을 재사용한다. 별도 중복 key를 만들지 않는다.
- `DisplayChoice`와 menu는 Copy/Debug/Eq를 유지한다.

### 2. App 진입과 Screen 전이

- `open_display_menu`는 active cart platform을 기존 `retro_platform`으로 변환하고
  `slot2_retro::def(platform).overscan != Overscan::NONE`으로 availability를 계산한다. NES를 문자열이나
  store enum으로 hard-code하지 않는다.
- `Screen`에 `Overscan(InGameMenu, DisplayMenu, OverscanMenu)`를 추가한다. parent menu 둘을 보존해
  B/MENU가 같은 Overscan 행의 Display 화면으로 돌아오게 하고 `Screen` Copy/Debug/Eq를 유지한다.
- Display A는 `Scale`이면 기존 commit, `Shader`이면 기존 Shader screen, `Overscan`이면 active cart의
  current `settings.overscan`으로 Overscan screen을 연다.
- non-crop platform에서는 `choices()`에 Overscan이 없어 normal navigation으로 screen을 열 수 없다.
  active Session이 없으면 open/I/O를 하지 않는다.
- Overscan screen에서 Up/Down은 세 행을 wrap한다. B/MENU는 settings/runtime 변경 없이 parent
  Display의 같은 Overscan 행으로 돌아간다. 다른 ordinary button은 아무 동작도 하지 않는다.
- Overscan screen은 core frame/audio를 pause하고 sink request를 만들지 않는다. global power/volume은
  기존 전역 처리 그대로 유지한다.

### 3. Session-owned setting 해석과 즉시 적용

- Session에 platform과 `Option<bool>`을 실제 `slot2_retro::Overscan`으로 해석하는 단일 public helper를
  둔다. 정확한 의미는 `None | Some(true)` → registry default crop, `Some(false)` → `Overscan::NONE`이다.
- launch initialization과 새 runtime apply API가 모두 그 helper를 사용하게 한다. 기존
  `set_overscan(bool)` API는 필요한 기존 호출/테스트를 위해 유지해도 되지만 App이 `unwrap_or(true)`나
  registry mapping을 다시 쓰지 않게 한다.
- Overscan A는 전체 settings에서 overscan만 selected 값으로 바꿔 카드에 write한다. core, scale,
  rewind, shader와 unknown key를 보존한다.
- write 성공 뒤에만 Session runtime apply API를 호출한다. 성공하면 Overscan screen/selected row를
  유지하고 다음 draw가 same last frame을 새 crop/UV/placement로 보여준다.
- `None`은 overscan key만 제거하고 platform default를 즉시 재적용한다. NES에서는 상하 8줄 crop이다.
  overscan-only ini였다면 store 계약에 따라 파일을 제거한다.
- `Some(true)`는 canonical `overscan = on`을 남기고 default와 같은 crop을 쓰되 저장 의미는 구분한다.
- `Some(false)`는 canonical `overscan = off`를 남기고 full UV를 쓴다.
- success toast, frame advance, Session/core restart, texture upload/update/free와 audio 변경을 만들지 않는다.
- write failure는 card bytes/prior crop을 유지하고 attempted row에 남아 error 한 줄과
  `overscan-save-failed` toast를 표시한다.
- 신규 번역은 정확히 다음과 같다.
  - English: `overscan-save-failed = Could not save the overscan setting`
  - 한국어: `overscan-save-failed = 오버스캔 설정을 저장하지 못했습니다`

### 4. draw와 문서 정합성

- Overscan screen은 Session last frame을 current shader와 crop으로 먼저 그리고 OverscanMenu overlay만
  올린다. parent Display/InGame menu, wallpaper, HUD badge, switcher와 clear가 사이에 없어야 한다.
- hold progress/toast는 기존대로 OverscanMenu 뒤에 그린다.
- App 상태기계·draw 주석과 Display menu 설명에서 fixed five rows 또는 overscan 미구현이라는 부분만
  현재 사실에 맞게 고친다. overlay selector가 구현됐다고 쓰지 않는다.
- 새 Screen variant를 모든 exhaustive pause/draw/menu match에 정확히 추가한다.

## 테스트 계약

### Display UI

기존 `display_menu` 테스트를 동적 row 계약으로 보강하되 기존 단언을 약화하지 않는다.

- crop 없음은 기존 5행/높이 280/양방향 wrap을 정확히 유지함
- crop 있음은 같은 5행 뒤 Overscan을 추가한 6행/높이 316이며 navigation이 여섯 행에서 wrap함
- constructor의 네 scale 값은 두 mode 모두 정확한 scale row에서 시작함
- 세 geometry·두 언어에서 두 높이 모두 safe-area, label, 단일 highlight와 warm redraw를 보존함

### Session 집중 테스트

- 일곱 platform에서 `None`과 `Some(true)`가 registry default를, `Some(false)`가 NONE을 반환함
- NES default는 top/bottom 8, 다른 여섯 default는 NONE임
- launch와 runtime apply가 같은 helper 결과를 사용하며 setter가 frame/core/texture/audio를 건드리지 않음

### App 집중 테스트

- GBA 등 crop 없는 platform Display는 기존 5행이며 Overscan에 도달하거나 screen을 열 수 없음
- 합성 NES ROM + FCEUmm session은 6행 Display에서 Overscan을 열고 current `None`/true/false 행을 정확히
  선택함. B/MENU는 같은 Display Overscan 행으로 돌아오며 bytes/runtime을 바꾸지 않음
- NES에서 세 선택 모두 정확히 저장되고 즉시 적용됨: default/true는 crop UV
  `[0, 8/240, 1, 232/240]`, false는 full UV `[0, 0, 1, 1]`; placement도 256×224와 256×240 visible
  size에 맞게 재계산됨
- setting 전환은 같은 last frame에서 일어나며 frames/core state/texture lifecycle/audio/sink가 불변임
- Platform default는 overscan key만 제거하고 overscan-only ini를 제거함. explicit true/false는 canonical
  on/off로 남음
- core/scale/rewind/shader와 hand-written unknown key가 set/clear 뒤에도 보존됨
- write failure와 unreadable original은 prior bytes/crop을 보존하고 attempted row에 남아 localized toast
- 닫고 다시 열면 last successful value가 선택됨
- Overscan screen의 pause/audio/sink와 game frame → overlay draw 순서가 유지됨
- 기존 GBA scale/Shader 선택·저장·failure tests가 동적 constructor 뒤에도 계속 통과함
- 영문/한글 failure key가 fallback 없이 직접 정의됨

FCEUmm가 없으면 real NES cases만 기존 형식으로 명시적 skip하고 정확한 개수와 이유를 보고한다. 이
checkout에는 `vendor/fceumm_libretro.dll`과 `.so`가 있으므로 정상 환경에서는 skip 0이어야 한다.
ignore, 테스트 전용 production bypass와 단언 완화는 금지한다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-ui\src\display_menu.rs`
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`의 기존 export 조정이 필요한 경우만
- `C:\SLOT2\crates\slot2-ui\tests\display_menu.rs`
- `C:\SLOT2\crates\slot2\src\session.rs`
- `C:\SLOT2\crates\slot2\src\app.rs`
- `C:\SLOT2\crates\slot2\tests\session.rs`
- `C:\SLOT2\crates\slot2\tests\display_menu_app.rs`
- `C:\SLOT2\assets\lang\en.ftl`
- `C:\SLOT2\assets\lang\ko.ftl`
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`의 failure key 직접 정의 단언만
- `C:\SLOT2\tasks\73-overscan-menu-app-wiring.worker-result.md`

`overscan_menu.rs`, store, gfx, retro registry와 다른 production/test 파일은 수정하지 않는다. 합성 NES
helper는 `display_menu_app.rs` 안에 필요한 만큼만 복제한다. 구현이 불가능하면 범위를 넓히지 말고
보고서에 이유를 적어 실패로 둔다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- registry crop 값, core option, store ini 문법과 gfx crop math 변경
- overlay selector, generic settings framework와 shelf platform-default UI
- success toast, animation, sound, 새 gesture와 Session/core restart
- 전체 workspace 테스트, GL 창, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-ui --test display_menu
cargo test -p slot2 --test session --test display_menu_app
cargo test -p slot2 -p slot2-ui -p slot2-i18n
cargo clippy -p slot2 -p slot2-ui -p slot2-i18n --all-targets -- -D warnings
```

모두 종료 0이어야 한다. core-dependent test가 skip되면 정확한 개수와 이유를 보고한다. 검증 뒤 코드를
바꾸면 영향받는 명령부터 다시 실행한다. workspace test, GL test와 device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\73-overscan-menu-app-wiring.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- registry 기반 조건부 Display rows/layout과 Screen 전이 결과
- 세 settings 의미의 저장 우선·runtime crop/UV 즉시 적용·failure rollback 결과
- 기존 known/unknown key, scale/Shader, pause/audio/sink/texture/draw 순서 보존 근거
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄과 core-dependent skip 수
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- overlay selector와 실기 화질이 범위 밖으로 남았다는 확인
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
