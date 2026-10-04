# Task 70 — 게임별 셰이더 선택 메뉴 UI

현재 checkout에서 직접 작업한다. Task66의 게임별 `ShaderPreset` 여섯 의미를 표시하고 선택할 수 있는
독립적인 `ShaderMenu` UI를 `slot2-ui`에 추가한다. 이번 태스크는 **UI 컴포넌트와 번역만** 다룬다.
Display 메뉴에서 여는 행, App의 즉시 preview와 저장은 Task71로 분리한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\70-shader-menu-ui.md`
- `C:\SLOT2\crates\slot2-ui\src\display_menu.rs`
- `C:\SLOT2\crates\slot2-ui\tests\display_menu.rs`
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`의 module 선언과 re-export 주변만
- `C:\SLOT2\crates\slot2-store\src\settings.rs`의 `ShaderPreset` 정의만
- `C:\SLOT2\assets\lang\en.ftl`과 `ko.ftl`의 Display/힌트 관련 key 주변만
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`의 Display 직접 번역 테스트 주변만

직접 필요한 shared draw/span helper만 추가로 읽는다. App, Session, gfx shader 구현, 다른 태스크·보고서·
로그와 저장소 이력은 읽지 않는다.

## 현재 계약

- `GameSettings::shader`는 `Option<ShaderPreset>`이다. `None`은 플랫폼 기본값 상속,
  `Some(Off)`는 명시적 끄기, 나머지 네 값은 명시적 effect override다.
- 플랫폼 기본값의 실제 `Lcd3x`/`ZfastCrt` 해석은 registry와 Session 소관이다. UI는 `None`을 계산된
  preset으로 바꾸지 않고 **Platform default**라는 저장 의미 그대로 보여준다.
- explicit Off와 Platform default는 화면 결과가 우연히 같을 수 있어도 다른 카드 설정이므로 별도 행이다.
- 기존 `DisplayMenu`는 scale 네 선택을 직접 고르는 화면이다. 이를 10행짜리 혼합 목록으로 바꾸지 않는다.
  이번 컴포넌트는 후속 App 태스크가 별도 screen으로 열 수 있는 shader 전용 선택 화면이다.

## 구현 계약

### 1. public 선택 모델

- `slot2-ui`에 public `shader_menu` module과 main `ShaderMenu` type을 추가하고 crate root에서 re-export한다.
- 새 shader enum을 만들지 않는다. 행의 값과 public selected query는 기존
  `Option<slot2_store::ShaderPreset>`을 사용한다.
- 선택지는 다음 여섯 개를 정확히 이 순서로 둔다.
  1. `None` — Platform default
  2. `Some(ShaderPreset::Off)` — Off
  3. `Some(ShaderPreset::SharpBilinear)` — Sharp bilinear
  4. `Some(ShaderPreset::Lcd3x)` — LCD 3x
  5. `Some(ShaderPreset::ZfastCrt)` — zfast CRT
  6. `Some(ShaderPreset::Scanline)` — Scanlines
- 여섯 값을 public 고정 배열로 제공해 draw, navigation과 테스트가 같은 순서를 사용하게 한다.
- `ShaderMenu::new(current)`는 정확히 해당 행을 선택한다. `selected()`는 highlighted row의 정확한
  `Option<ShaderPreset>`을 반환한다.
- Up/Down은 양 끝에서 wrap한다. 입력 event 처리, 파일 I/O와 App 상태는 컴포넌트에 넣지 않는다.

### 2. overlay draw

- 기존 Display/InGame/Power 메뉴의 색, text 크기, safe-area와 face cache 방식을 재사용한다.
- game frame 위에 그릴 컴포넌트이므로 `Canvas::clear`를 호출하지 않는다.
- draw 순서는 physical panel 전체의 `BLACK alpha 0.6` dim, safe area 중앙 panel, title, 여섯 행,
  select/back hint다. 정확히 한 행만 highlight한다.
- layout 상수는 `BOX_W = 380`, `BOX_H = 316`, `PAD = 16`, `ROW_H = 36`으로 둔다. row 시작은 기존
  Display 메뉴와 같은 `box_y + PAD + 28`이다. title/rows/highlight/hints가 640×480 safe area 안에
  들어가야 한다.
- 같은 menu와 language를 warm 상태에서 반복 draw해도 새 glyph texture upload가 없어야 한다.

### 3. 표시 문구

영문/한글 pack에 직접 정의한다. 기존 `display-platform-default` key는 같은 의미이므로 재사용하고,
나머지는 다음 문구를 정확히 사용한다.

| key | English | 한국어 |
|---|---|---|
| `shader-title` | `Shader` | `셰이더` |
| `shader-off` | `Off` | `끄기` |
| `shader-sharp-bilinear` | `Sharp bilinear` | `선명한 이중선형` |
| `shader-lcd3x` | `LCD 3x` | `LCD 3배` |
| `shader-zfast-crt` | `zfast CRT` | `zfast CRT` |
| `shader-scanline` | `Scanlines` | `스캔라인` |

key lookup은 exhaustive match로 작성한다. enum 순서/index나 Debug 문자열로 message key를 만들지 않는다.
Korean fallback이 누락을 숨기지 않도록 i18n 직접 정의 테스트를 추가한다.

## 테스트 계약

`crates/slot2-ui/tests/shader_menu.rs`에 집중 테스트를 추가한다.

- `None`, Off와 네 effect 각각의 constructor가 정확한 행을 선택하고 `selected()`가 같은 값을 반환함
- 고정 순서대로 Up/Down이 이동하며 양 끝에서 wrap함
- draw가 clear하지 않고 dim → panel 순서이며 정확히 한 행만 highlight함
- `rgsp`, `rg35xxsp`, `rgcubexx`의 영문/한글에서 box, title, 여섯 labels, highlight와 hints가
  640×480 safe area 안에 있음
- 여섯 행이 위 표의 올바른 localized label을 각자 자기 row에 그림
- 동일 menu warm redraw 및 highlight 이동 뒤 `UploadAlpha8`/`UploadRgba8`가 0개임
- public row 배열에 여섯 값이 중복 없이 정확한 순서로 있고 key mapping도 중복되지 않음
- 영문과 한글이 신규 key를 fallback 없이 정확히 직접 정의함

RecordingCanvas와 geometry 단언만 사용한다. GPU/GL window, screenshot golden과 texture id 단언은 쓰지
않는다. 기존 Display menu 테스트를 약화하거나 재작성하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-ui\src\shader_menu.rs` (신규)
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`의 module/re-export 두 줄
- `C:\SLOT2\crates\slot2-ui\tests\shader_menu.rs` (신규)
- `C:\SLOT2\assets\lang\en.ftl`
- `C:\SLOT2\assets\lang\ko.ftl`
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`의 신규 key 직접 정의 테스트만
- `C:\SLOT2\tasks\70-shader-menu-ui.worker-result.md`

다른 production/test 파일은 수정하지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- 기존 `DisplayMenu`의 scale 행·layout·API 변경
- App screen variant, Display 진입 행, 입력 배선, live shader preview와 settings 저장
- Session, store, gfx, retro registry와 shader GLSL/program/fallback 변경
- overlay/overscan UI, generic settings/menu framework, animation과 새로운 input gesture
- 전체 workspace 테스트, GL 창, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-ui --test shader_menu
cargo test -p slot2-ui -p slot2-i18n
cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test, GL test와
device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\70-shader-menu-ui.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- 여섯 값/행 순서, constructor/query/navigation과 draw 결과
- 영문/한글 문구, safe-area와 warm redraw 검증
- 각 완료 기준 명령, 종료 코드와 마지막 결과 줄
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- App/Session 저장·preview가 범위 밖으로 남았다는 확인
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
