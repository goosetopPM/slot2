# Task 85 — About 스티커 UI

현재 checkout에서 직접 작업한다. M4 선반 설정 메뉴의 `About` 행이 열 독립 `AboutSticker` UI를
`slot2-ui`에 추가한다. 화면은 SLOT2 버전, 실행 target, SLOT2의 MIT 표시와 배포 카드의
`System/licenses` 안내를 정직하게 보여준다. 이번 태스크에서는 App 진입이나 Shelf 가용성을 바꾸지
않는다. 다음 태스크가 About 행을 활성화하고 `App::Screen`에 연결한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

필요한 부분만 읽는다.

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\83-shelf-menu-ui.result.md`
- `C:\SLOT2\tasks\84-shelf-timezone-app-wiring.result.md`
- `C:\SLOT2\docs\MILESTONES.md`의 M4 About 항목만
- `C:\SLOT2\docs\DESIGN.md`의 UI 화면 상태기계와 배포 카드 `System/licenses` 부분만
- `C:\SLOT2\Cargo.toml`의 workspace version/license만
- `C:\SLOT2\build\dist-device.ps1`의 version과 `System/licenses` 조립 부분만
- `C:\SLOT2\crates\slot2-ui\src\timezone_menu.rs`의 layout/draw 방식만
- `C:\SLOT2\crates\slot2-ui\src\shelf_menu.rs`의 palette/hint 방식만
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`의 module/re-export 부분만
- `C:\SLOT2\crates\slot2-ui\tests\timezone_menu.rs`의 safe-area와 warm redraw helper만
- `C:\SLOT2\assets\lang\en.ftl`, `ko.ftl`의 shelf/timezone 구역만
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`의 직접 정의 key 검사 부분만

App, Screen/input, store/platform backend, Session/core, 다른 메뉴와 워커 로그·저장소 이력은 읽지
않는다.

## 제품 계약

- 이 화면은 법률 문서 전문 뷰어가 아니라 **스티커 형태의 빌드 정보와 라이선스 위치 안내**다.
- 표시하는 동적 값은 frontend 호출자가 넘기는 `version`과 `target` 두 개뿐이다.
- UI 크레이트 자신의 `CARGO_PKG_VERSION`이나 host OS를 읽어 App 버전·기기 target인 것처럼 표시하지
  않는다.
- git SHA는 현재 바이너리에 compile-time으로 주입되지 않으므로 만들거나 추측하지 않는다.
- `System/licenses`는 배포 스크립트가 실제 생성하는 경로다. “모든 라이선스 전문이 완비됐다”처럼
  아직 M7에서 완성할 내용을 약속하지 않고 **코어·글꼴 고지 위치**라고만 안내한다.
- SLOT2 workspace의 license가 MIT이므로 `SLOT2 · MIT`는 고정 제품 정보로 표시한다.

## 구현 계약

### 1. 공개 타입과 API

`crates/slot2-ui/src/about_sticker.rs`를 추가하고 crate root에서 타입을 re-export한다.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AboutInfo<'a> {
    pub version: &'a str,
    pub target: &'a str,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AboutSticker;
```

최소한 다음 API를 제공한다.

```rust
impl AboutSticker {
    pub const fn new() -> Self;
    pub fn draw(
        &self,
        canvas: &mut dyn Canvas,
        ctx: &mut UiCtx,
        info: AboutInfo<'_>,
    );
}
```

- `AboutSticker`는 상태가 없는 `Copy` 값이다. 입력 처리, filesystem, clock, 환경변수 접근을 넣지 않는다.
- `AboutInfo`의 문자열을 코드에서 `Version ...`처럼 조합하지 않는다. Fluent argument로 전달한다.
- version/target이 빈 문자열이어도 panic하거나 safe area를 벗어나지 않는다.

### 2. 스티커 draw

기존 overlay menu와 일관된 draw 순서를 사용한다.

- `Canvas::clear`를 호출하지 않는다.
- physical panel 전체에 검정 alpha 0.6 dim을 먼저 그린다.
- safe area 중앙에 `BOX_W = 440`, `BOX_H = 260`, `PAD = 16`의 panel을 그린다.
- panel 안에는 위에서 아래로 다음을 표시한다.
  1. `about-title`
  2. 큰 `SLOT2` 워드마크
  3. `about-version`에 `version` argument
  4. `about-target`에 `target` argument
  5. `about-license`
  6. `about-notices`
  7. 기존 `hint-back`
- SLOT2, 경로, MIT 같은 제품 토큰을 Rust에서 여러 조각으로 이어 붙이지 않는다. 모두 FTL 문장 안에
  둔다. 동적 argument는 version과 target만 허용한다.
- title과 hint는 기존 메뉴의 `INK_DIM`, 핵심 워드마크는 `INK`, 나머지 정보는 읽기 쉬운 기존 palette를
  사용한다. 새 색 시스템이나 이미지 asset을 만들지 않는다.
- 모든 텍스트는 실제 측정값으로 가로 중앙 정렬하거나 명시적 inset에 배치한다. 글자 수 기반 위치와
  잘림을 숨기는 clip은 금지한다.
- `rg35xxsp` 640×480, `rgsp` 720×480, `rgcubexx` 720×720의 영문·한글에서 panel과 모든 glyph/button
  span이 safe area 안에 있어야 한다. 720×720에서 physical panel 중심으로 잘못 이동하면 안 된다.
- 같은 language/info의 warm redraw 8회에서 새 glyph/image upload가 없어야 한다.

### 3. 번역

두 built-in pack에 다음 key를 직접 정의한다.

| key | English | 한국어 |
|---|---|---|
| `about-title` | `About` | `정보` |
| `about-wordmark` | `SLOT2` | `SLOT2` |
| `about-version` | `Version { $version }` | `버전 { $version }` |
| `about-target` | `Device { $target }` | `기기 { $target }` |
| `about-license` | `SLOT2 · MIT` | `SLOT2 · MIT` |
| `about-notices` | `Core and font notices: System/licenses` | `코어 및 글꼴 고지: System/licenses` |

뒤로 힌트는 기존 `hint-back`을 재사용한다. 새 문구는 code fallback이 아니라 각 FTL에 직접 있어야
한다. 기존 `shelf-about` 행 이름은 변경하지 않는다.

## 테스트 계약

`crates/slot2-ui/tests/about_sticker.rs`를 추가해 최소한 다음을 검증한다.

1. `AboutInfo`가 영문·한글 draw에서 전달받은 version/target을 정확한 localized span으로 표시한다.
2. 화면에 title, wordmark, version, target, license, notices, 뒤로 힌트가 각각 한 번 존재한다.
3. draw가 clear 없이 full-panel dim을 첫 mark로 그리고 그 뒤 safe-area 중앙 panel을 그린다.
4. 세 실제 geometry × 두 언어에서 panel과 모든 text/button span이 safe area 안에 있다.
5. 720×720 panel은 safe area 중심을 사용한다.
6. 긴 방어 입력(`version = "1234567890.1234567890"`, `target = "unknown-long-target"`)도 safe area를
   벗어나지 않는다. 필요하면 정보 글꼴 크기를 낮추되 생략·ellipsis·clip으로 테스트를 피하지 않는다.
7. 빈 version/target이 panic하지 않고 FTL의 나머지 문구를 정상 표시한다.
8. 같은 화면 warm redraw 8회에서 upload가 없다.
9. en/ko pack에 여섯 신규 key가 직접 존재하고 표의 정확한 문자열로 해석된다.

RecordingCanvas만 사용한다. GPU/GL 창, filesystem, wall clock, environment, texture id 순서에 의존하지
않는다. 테스트를 위해 production에 cache counter나 input handler를 추가하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-ui\src\about_sticker.rs` (신규)
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`의 module/re-export만
- `C:\SLOT2\crates\slot2-ui\tests\about_sticker.rs` (신규)
- `C:\SLOT2\assets\lang\en.ftl`
- `C:\SLOT2\assets\lang\ko.ftl`
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`의 신규 key 직접 정의 검증만
- `C:\SLOT2\tasks\85-about-sticker-ui.worker-result.md`

다른 production/test 파일은 수정하지 않는다. 특히 `slot2` App, `ShelfAvailability`, store/platform,
dist script와 license 파일을 고치지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- Shelf의 About 행 활성화, App Screen/input/draw 연결
- git SHA 생성·주입, build.rs, 새 dependency
- 라이선스 전문 viewer, 스크롤, URL/QR code, network
- 실제 license 파일 추가·수정 또는 M7 배포 계약 완성 처리
- 언어 선택, 플랫폼 기본 화면, 부팅 로고, 연동 기능 구현
- 전체 workspace 테스트, 실제 GL 창, device 배포
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-ui --test about_sticker
cargo test -p slot2-ui -p slot2-i18n
cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 각 test 명령의 passed/failed/ignored 합계를 보고한다. 검증 뒤 코드를 바꾸면
영향받는 명령부터 다시 실행한다. workspace test, 실제 GL test와 device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\85-about-sticker-ui.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- 공개 API와 state-free/Copy 결과
- version/target argument, 고정 license/notices 문구와 영문·한글 결과
- 세 geometry safe-area, 긴/빈 입력, warm redraw 결과
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄과 passed/failed/ignored 합계
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 후속 App About 행 배선이 남았다는 확인
- 계약이 틀려 보이는 부분과 남은 위험
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
