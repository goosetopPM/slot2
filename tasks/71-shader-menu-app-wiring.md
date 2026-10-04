# Task 71 — Display 셰이더 메뉴 App 배선

현재 checkout에서 직접 작업한다. 기존 scale Display 화면에 Shader 진입 행을 추가하고 Task70의
`ShaderMenu`를 App 상태기계에 연결한다. 선택한 게임별 shader 설정은 **카드 저장 성공 뒤에만** 실행 중
Session에 즉시 적용한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\71-shader-menu-app-wiring.md`
- `C:\SLOT2\tasks\69-platform-shader-defaults.result.md`
- `C:\SLOT2\tasks\70-shader-menu-ui.result.md`
- `C:\SLOT2\crates\slot2-ui\src\display_menu.rs`
- `C:\SLOT2\crates\slot2-ui\src\shader_menu.rs`
- `C:\SLOT2\crates\slot2-ui\tests\display_menu.rs`
- `C:\SLOT2\crates\slot2\src\app.rs`의 `Screen`, pause/audio guard, Display 입력/helper와 draw 주변만
- `C:\SLOT2\crates\slot2\src\session.rs`의 platform 변환, shader helper/accessor/setter 주변만
- `C:\SLOT2\crates\slot2\tests\display_menu_app.rs`
- `C:\SLOT2\crates\slot2-store\src\settings.rs`의 `GameSettings`와 `ShaderPreset` 정의만
- `C:\SLOT2\assets\lang\en.ftl`, `ko.ftl`과 i18n 직접 정의 테스트의 Display/shader key 주변만

직접 필요한 test helper만 추가로 읽는다. gfx GLSL/GL 구현, 다른 App 테스트, 다른 태스크·워커 로그와
저장소 이력은 읽지 않는다.

## 현재 계약

- 기존 `DisplayMenu`는 scale 네 값을 직접 선택하고 A로 저장하는 화면이다. scale persistence와 live
  application은 이미 동작하며 회귀시키지 않는다.
- Task70의 `ShaderMenu`는 `Option<ShaderPreset>` 여섯 의미를 정확히 표시하지만 아직 어디에서도 열리지
  않는다.
- Session의 `shader_effect_for(platform, preset)`이 explicit Off, 네 override와 key 부재의 플랫폼
  기본값을 gfx effect로 바꾸는 유일한 경계다. App에 중복 mapping을 만들지 않는다.
- 카드 저장은 전체 `GameSettings`를 read-modify-write하여 소유하지 않는 known/unknown key를 보존한다.
  write 실패 시 runtime을 먼저 바꾸면 다음 launch와 현재 화면이 어긋나므로 반드시 저장이 먼저다.

## 구현 계약

### 1. Display 화면의 Shader 진입 행

- `slot2-ui::display_menu`에 public Copy enum을 추가한다.

```rust
pub enum DisplayChoice {
    Scale(Option<ScaleMode>),
    Shader,
}
```

- 기존 네 scale 선택 뒤에 `DisplayChoice::Shader`를 다섯 번째 행으로 추가한다. 행 순서는 정확히
  Platform default, Integer, Aspect fit, Fill, Shader다.
- public `ROWS`는 `[DisplayChoice; 5]`가 되고 `DisplayMenu::choice()`가 highlighted choice를 반환한다.
  더는 모든 행이 scale인 것처럼 보이는 `selected() -> Option<ScaleMode>` API를 유지하지 않는다.
- `DisplayMenu::new(current_scale)`은 계속 정확한 scale row에서 시작한다. Shader row는 navigation으로
  도달하며 Up/Down은 다섯 행에서 wrap한다.
- Shader 행 label은 Task70의 `shader-title` key를 재사용한다. 별도 중복 번역 key를 만들지 않는다.
- 한 행 증가에 맞춰 `BOX_H`를 244에서 **280**으로 바꾼다. `BOX_W`, `PAD`, `ROW_H`와 row 시작은
  유지하며 세 device geometry·두 언어에서 safe area를 지킨다.
- `DisplayChoice`를 crate root에서 `DisplayMenu`와 함께 re-export한다.

### 2. App screen과 navigation

- `Screen`에 `Shader(InGameMenu, DisplayMenu, ShaderMenu)` variant를 추가한다. parent in-game menu와
  Shader row에 머문 Display menu를 함께 보존해 B/MENU가 한 단계 전의 같은 행으로 돌아오게 한다.
  세 menu가 Copy이므로 `Screen`의 Copy/Debug/Eq 계약을 유지한다.
- Display 화면에서 Up/Down은 기존처럼 이동한다. A는 choice에 따라 다음과 같이 분기한다.
  - `Scale(value)`: 기존 scale 저장·즉시 적용을 그대로 실행하고 Display 화면에 남는다.
  - `Shader`: active cart의 현재 `GameSettings::shader`를 읽어 `ShaderMenu::new`로 Shader 화면을 연다.
- active Session이 없으면 Shader 화면을 열거나 카드 I/O를 하지 않는다.
- Shader 화면에서 Up/Down은 여섯 행을 이동한다. B/MENU는 settings/runtime을 바꾸지 않고 parent
  Display 화면의 Shader 행으로 돌아간다. 다른 ordinary button은 아무 동작도 하지 않는다.
- Shader 화면도 in-game/Display menu처럼 core frame과 audio를 pause하고 sink open/close 요청을 만들지
  않는다. power와 global volume 동작은 기존 전역 처리 그대로 유지한다.

### 3. 저장 후 즉시 적용

- Shader 화면의 A는 active cart의 전체 settings를 읽고 **shader 필드만** highlighted 값으로 바꿔
  `Card::write_settings`를 호출한다. core, scale, overscan, rewind와 unknown key를 보존한다.
- 저장 성공 뒤에만 cart platform을 기존 `retro_platform` 경계로 변환하고
  `Session::shader_effect_for(platform, selected)` 결과를 `set_shader_effect`로 적용한다.
  store/retro/gfx mapping을 App에 다시 쓰지 않는다.
- 성공하면 Shader 화면을 선택한 행에 그대로 둔다. success toast, core frame advance, Session restart,
  texture upload/free와 audio 변경을 만들지 않는다. 다음 draw가 같은 last frame에 새 effect를 보여준다.
- `None` 선택은 shader key만 제거하고 현재 platform default를 즉시 다시 적용한다. GBA fixture에서는
  `Lcd3x`다. shader-only ini였다면 기존 store 계약에 따라 파일 자체가 제거된다.
- `Some(Off)`는 카드에 canonical `shader = none`을 남기고 runtime은 plain draw가 된다. key 부재와
  explicit Off를 합치지 않는다.
- write 실패 시 prior runtime effect와 카드 bytes를 유지하고 attempted row의 Shader 화면에 남는다.
  concise error를 한 번 log하고 `shader-save-failed` toast를 표시한다.
- 신규 번역은 정확히 다음과 같다.
  - English: `shader-save-failed = Could not save the shader setting`
  - 한국어: `shader-save-failed = 셰이더 설정을 저장하지 못했습니다`

### 4. draw와 상태기계 정합성

- Shader 화면에서는 Session의 last frame을 현재 effect로 먼저 그린 뒤 `ShaderMenu` overlay를 그린다.
  parent Display/InGame menu, wallpaper, time-control badge, state switcher와 clear를 사이에 넣지 않는다.
- hold progress와 toast는 기존 순서대로 ShaderMenu 뒤에 그린다.
- App module의 화면 전이·pause·draw 설명에서 scale-only라는 낡은 부분만 현재 동작에 맞게 고친다.
  overlay selector가 구현됐다고 쓰지 않는다.
- 새 Screen variant 때문에 생기는 모든 exhaustive match를 같은 pause/draw/menu 범주에 정확히 추가한다.

## 테스트 계약

### `slot2-ui` Display menu

기존 `display_menu` 테스트를 새 다섯 행 계약에 맞게 고치되 단언을 약화하지 않는다.

- constructor의 네 scale 값은 여전히 정확한 row에서 시작함
- 다섯 행 순서, Shader choice와 Up/Down 양방향 wrap
- BOX_H 280, 정확히 한 highlight, safe-area, 영문/한글 label과 warm redraw 보존

### App 집중 테스트

`crates/slot2/tests/display_menu_app.rs`에 다음을 검증한다.

- Display의 Shader 행에서 A가 현재 카드 shader 값(`None`, Off 또는 explicit preset)을 선택한
  Shader 화면을 열고, B/MENU가 같은 Shader 행으로 돌아옴
- Shader Up/Down wrap과 닫기만으로 settings bytes/runtime effect가 바뀌지 않음
- explicit Off와 네 effect 중 적어도 각 의미가 정확히 저장되고 즉시 runtime에 적용됨. 가능한 한 표
  기반으로 여섯 선택 전체를 검증함
- GBA에서 Platform default를 선택하면 shader override만 제거되고 runtime은 `Lcd3x`로 돌아가며,
  shader-only ini는 제거됨
- scale/core/overscan/rewind와 hand-written unknown key가 shader set/clear 뒤에도 보존됨
- 성공 A 뒤 Shader 화면과 선택 행이 유지되고 frames/core state/texture upload-update-free/audio/sink가
  바뀌지 않으며 다음 draw만 정확한 `Image` 또는 `ImageEffect`로 바뀜
- write failure와 unreadable original은 prior card bytes/runtime effect를 보존하고 attempted row에 남아
  `shader-save-failed` toast를 표시함
- 닫고 다시 Shader 화면을 열면 마지막 성공 저장값이 선택됨
- Shader 화면은 frame/audio를 pause하며 session과 sink를 유지함
- draw는 game frame → Shader dim/panel 순서이고 parent Display/InGame menu, wallpaper, HUD badge와 clear가 없음
- 기존 네 scale 선택·저장·실패·draw 테스트가 다섯 번째 Shader 행 뒤에도 계속 정확히 통과함
- 영문/한글 failure key가 fallback 없이 직접 정의됨

real-core fixture가 명시된 기존 사유로 없으면 해당 테스트만 skip하고 정확한 수와 이유를 보고한다.
UI/state/persistence의 pure 검증은 가능한 범위에서 core와 독립적으로 실행한다. 테스트용 bypass, ignore와
단언 완화는 금지한다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-ui\src\display_menu.rs`
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`의 `DisplayChoice` re-export만
- `C:\SLOT2\crates\slot2-ui\tests\display_menu.rs`
- `C:\SLOT2\crates\slot2\src\app.rs`
- `C:\SLOT2\crates\slot2\tests\display_menu_app.rs`
- `C:\SLOT2\assets\lang\en.ftl`
- `C:\SLOT2\assets\lang\ko.ftl`
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`의 failure key 직접 정의 단언만
- `C:\SLOT2\tasks\71-shader-menu-app-wiring.worker-result.md`

`shader_menu.rs`, Session, store, gfx, retro와 다른 production/test 파일은 수정하지 않는다. 구현이
불가능하면 범위를 넓히지 말고 보고서에 이유를 적어 실패로 둔다. 관련 없는 미커밋 변경을 정리·복원·
재포맷하지 않는다.

## 범위 밖 및 금지

- shader source/program/fallback, 플랫폼 기본값과 store ini 문법 변경
- overlay/overscan selector, generic settings framework와 shelf platform-default UI
- 성공 toast, animation, sound, 새로운 gesture, Session/core restart
- 전체 workspace 테스트, GL 창, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-ui --test display_menu
cargo test -p slot2 --test display_menu_app
cargo test -p slot2 -p slot2-ui -p slot2-i18n
cargo clippy -p slot2 -p slot2-ui -p slot2-i18n --all-targets -- -D warnings
```

모두 종료 0이어야 한다. core-dependent test가 skip되면 정확한 개수와 이유를 보고한다. 검증 뒤 코드를
바꾸면 영향받는 명령부터 다시 실행한다. workspace test, GL test와 device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\71-shader-menu-app-wiring.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- DisplayChoice/Shader screen 전이와 B/MENU 복귀 결과
- 여섯 settings 의미의 저장 우선·runtime 즉시 적용·실패 rollback 결과
- 기존 known/unknown key, scale 흐름, pause/audio/sink/draw 순서 보존 근거
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄과 core-dependent skip 수
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- overlay selector와 실기 화질이 범위 밖으로 남았다는 확인
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
