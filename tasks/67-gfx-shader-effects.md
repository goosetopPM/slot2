# Task 67 — GFX 내장 셰이더 효과 기반

현재 checkout에서 직접 작업한다. `slot2-gfx`에 D-10의 네 내장 단일 패스 셰이더를 개별 이미지 draw에
적용할 수 있는 renderer 기반을 추가한다. 이번 태스크는 **gfx 한 크레이트만** 다룬다. Session이 게임
텍스처에 효과를 선택하는 배선, 게임별 store 설정 변환, 플랫폼 기본값과 Display 메뉴는 후속으로
분리한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\67-gfx-shader-effects.md`
- `C:\SLOT2\tasks\66-game-shader-settings-store.result.md`
- `C:\SLOT2\crates\slot2-gfx\src\lib.rs`
- `C:\SLOT2\crates\slot2-gfx\src\canvas.rs`
- `C:\SLOT2\crates\slot2-gfx\src\gl_canvas.rs`
- `C:\SLOT2\crates\slot2-gfx\tests\screenshot.rs`
- `C:\SLOT2\docs\DECISIONS.md`의 D-10과 D-23만

현재 game texture draw 경계를 이해하는 데 필요하면
`C:\SLOT2\crates\slot2\src\session.rs`의 `upload_video`와 `draw` 주변만 **읽을 수 있다**. 그 파일을
수정하지 않는다. UI/App/store/platform, shader 관련 외부 저장소와 워커 로그는 읽지 않는다.

## 현재 구조와 핵심 경계

- `GlCanvas`는 모든 UI와 game frame을 panel 크기의 같은 offscreen framebuffer에 그린 뒤 `present`에서
  최종 panel texture를 surface로 복사한다.
- 따라서 `present` 전체에 shader를 적용하면 HUD, 인게임 메뉴, 글자와 dim layer까지 함께 왜곡된다.
  셰이더는 **게임 texture의 한 draw에만** 적용해야 한다.
- 일반 `image`/`image_uv`, rect, text mask와 최종 present는 지금의 기본 texture program을 계속 쓴다.
- Task66의 `slot2_store::ShaderPreset`은 카드 형식 타입이다. gfx가 store에 의존하지 않도록 renderer
  쪽에는 `ShaderEffect`를 두고, 두 타입의 변환은 후속 Session/App 태스크의 한 경계에서 한다.
- `Off`는 renderer 효과가 아니라 기본 image draw이므로 gfx enum에는 넣지 않는다.

## 구현 계약

### 1. 공개 renderer 효과와 Canvas API

- `slot2_gfx::ShaderEffect`를 공개한다. variant는 정확히 `SharpBilinear`, `Lcd3x`, `ZfastCrt`,
  `Scanline` 네 개이며 `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `Hash`를 제공한다.
- 순회가 필요하면 `ShaderEffect::ALL`을 위 순서의 `[ShaderEffect; 4]`로 제공한다.
- `Canvas`에 UV crop과 tint를 받는 effect draw API를 추가한다. 이름은 명확하면 되지만 다음 의미를
  모두 보존해야 한다.
  - texture, destination `x/y/w/h`, `[u0,v0,u1,v1]`, tint, `ShaderEffect`를 한 호출에 받는다.
  - 편의상 full-UV 버전을 함께 제공해도 된다.
  - effect를 지원하지 못하는 Canvas의 기본 구현은 같은 geometry/UV/tint의 일반 `image_uv`로
    안전하게 fallback한다.
- `RecordingCanvas`는 effect draw를 일반 `Op::Image`로 잃지 않고 새 `Op` variant에 effect,
  texture, geometry, UV, tint를 모두 기록한다. origin도 일반 draw와 똑같이 적용한다.
- 기존 `image`, `image_uv`, `Op::Image`의 공개 의미와 기존 호출 기록을 바꾸지 않는다.

### 2. 내장 source 계약

- 네 fragment shader source는 저장소 안 Rust source에 내장한다. 외부 파일·런타임 파일 읽기·새 crate
  의존성을 만들지 않는다.
- GLSL ES 1.00 문법만 사용한다. `attribute`/`varying`/`texture2D`와 `#ifdef GL_ES` precision을
  유지하고 GLES2에서 없는 derivative, textureSize, bitwise op, dynamic loop를 사용하지 않는다.
- 모든 효과는 source texture size, destination draw size와 UV crop 경계를 uniform으로 받아야 한다.
  neighbor sample은 지정된 UV 영역 밖으로 나가 overscan crop 바깥 픽셀을 다시 끌어오면 안 된다.
- 모든 효과는 tint RGB와 alpha를 기존 image draw와 같은 의미로 적용한다. transparent source의 alpha를
  불투명하게 만들지 않는다.
- 단색 입력은 경계 artifact 없이 같은 계열의 단색을 유지하고, NaN/Inf나 UV wrap을 만들지 않는다.
- 네 효과는 이름만 다른 pass-through나 단순 상수 밝기 네 벌이면 안 된다.
  - `SharpBilinear`: source texel 중심은 선명하게 유지하면서 확대 경계만 좁게 bilinear 보간한다.
    nearest의 큰 block과 보통 linear의 전면 blur 사이 동작이어야 하며 scanline/mask는 넣지 않는다.
  - `Lcd3x`: destination의 3-pixel RGB subpixel 주기와 약한 cell separation을 적용한다. 세 stripe의
    평균 밝기가 원본에서 크게 벗어나지 않도록 보정하고 alpha는 유지한다.
  - `ZfastCrt`: 가벼운 수평 neighbor blend와 한 source row 주기의 부드러운 scan modulation을 함께
    적용한다. multi-pass, history texture, curvature는 넣지 않는다.
  - `Scanline`: color channel mask나 blur 없이 한 source row마다 반복되는 어두운 band만 적용한다.
    destination 해상도가 달라도 band가 source row에 고정돼 흔들리지 않아야 한다.
- 수치 계수는 가독성 있는 상수로 두고 source 주석에 효과의 이유만 짧게 적는다. 외부 shader 코드를
  복사하지 않는다.

### 3. GlCanvas 프로그램 선택과 격리

- 기존 기본 vertex/fragment program은 UI와 일반 image용으로 유지한다.
- `GlCanvas::new`에서 네 effect program을 준비한다. 공통 program compile/link helper는 성공과 모든
  실패 경로에서 shader/program GL object를 정확히 삭제해야 한다. 현재 기본 program의 link 실패
  경로에 program 삭제가 빠져 있다면 같은 helper로 정리한다.
- 기본 program compile/link 실패는 기존처럼 `GfxError::Shader`로 canvas 생성을 실패시킨다.
- 특정 **선택 effect**의 compile/link만 실패하면 canvas 전체를 실패시키지 않는다. 해당 effect의
  오류 문자열을 보존하고 그 effect draw만 기본 image program으로 fallback한다.
- `GlCanvas::shader_available(effect) -> bool`과 실패한 effect의 driver log를 읽을 수 있는
  `shader_error(effect) -> Option<&str>` 의미의 공개 조회를 제공한다. 성공 effect의 error는 `None`이다.
- effect draw 전에는 이전 일반 batch를 flush한다. effect quad는 source texture의 실제 크기,
  destination 크기와 UV crop uniform을 설정해 한 번 그린 뒤, 다음 일반 draw가 반드시 기본 program을
  쓰도록 상태를 복구한다. 효과가 다음 HUD/메뉴/text/rect로 새면 안 된다.
- effect draw가 texture batching을 깨는 것은 허용한다. 현재 화면당 game texture 한 장에 쓰는 경로라
  effect별 복잡한 batch key를 추가하지 않는다.
- unknown/freed `TexId`는 기존 일반 draw처럼 crash/panic하지 않는다. fallback과 성공 경로의 동작이
  일치해야 한다.
- `Drop`은 기본 program과 성공적으로 만든 모든 effect program을 한 번씩 삭제한다. fallback entry나
  실패한 object를 다시 삭제하지 않는다.
- 최종 `present`는 기본 program을 사용하며 panel 전체에 effect를 다시 적용하지 않는다.

## 테스트 계약

`crates/slot2-gfx/tests/shader_effects.rs`를 새로 만들거나 동등한 집중 테스트를 추가해 GPU 없이 다음을
직접 검증한다.

- 공개 enum의 네 variant와 `ALL` 순서
- `RecordingCanvas`가 네 effect 각각의 texture, geometry, UV, tint, effect를 정확히 기록함
- `set_origin`이 effect op에도 일반 image와 같은 방식으로 적용됨
- effect op 뒤의 일반 image/rect가 일반 op로 남아 효과가 논리적으로 전파되지 않음
- default Canvas fallback을 쓰는 최소 test canvas가 동일 geometry/UV/tint의 일반 image draw를 받음
- 네 source가 서로 다르고, NUL이 없으며, GLES2 계약에 필요한 공통 uniform/precision/main을 포함함
  (문자열 존재만으로 화질을 통과시키는 테스트는 만들지 않는다)

기존 `tests/screenshot.rs`의 단일 GL context phase도 확장한다.

- `SLOT2_GFX_TEST=1`일 때 네 effect가 모두 compile돼 `shader_available == true`이고 error가 없음
- 고대비 checker/색 패턴에 네 effect를 각각 적용해 read-back 결과가 base와 서로 구분됨
- effect 뒤에 그린 작은 일반 image 또는 rect의 색은 base 경로와 같아 program state가 새지 않음
- UV sub-rect 가장자리 밖의 강한 sentinel color가 effect 결과 안으로 유입되지 않음
- pixel 비교는 driver rounding을 감안한 tolerance/checksum 범위를 쓰되, pass-through shader가 통과할
  정도로 느슨하게 만들지 않는다.

환경변수가 없으면 기존처럼 GL phase가 skip되는 것은 허용한다. 보고서에는 실제 실행/skip을 정확히
쓴다. GPU 없는 기본 테스트만 통과했다고 실제 Mali compile을 확인했다고 적지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-gfx\src\lib.rs`
- `C:\SLOT2\crates\slot2-gfx\src\canvas.rs`
- `C:\SLOT2\crates\slot2-gfx\src\gl_canvas.rs`
- `C:\SLOT2\crates\slot2-gfx\src\shader.rs` 또는 의미가 같은 신규 내부 module 1개
- `C:\SLOT2\crates\slot2-gfx\tests\shader_effects.rs` (신규)
- `C:\SLOT2\crates\slot2-gfx\tests\screenshot.rs`
- `C:\SLOT2\tasks\67-gfx-shader-effects.worker-result.md`

`Cargo.toml`과 다른 crate는 수정하지 않는다. 구현이 불가능하다고 판단되면 범위를 넓히지 말고
보고서에 이유를 적어 실패로 둔다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- `slot2-store::ShaderPreset` 변경 또는 gfx가 store에 의존하게 만들기
- Session/App의 game texture에 effect 적용, 설정 load/write, Display 메뉴 변경
- 플랫폼별 기본 effect 결정과 registry 변경
- overlay/bezel, 사용자 GLSL, `.glslp`, `.slang`, multi-pass shader
- 외부 shader 다운로드·복사, 새 dependency, build script와 asset pipeline
- 전체 workspace 테스트, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-gfx
cargo check -p slot2-gfx --no-default-features --features device
cargo check -p slot2 --tests
cargo clippy -p slot2-gfx --all-targets -- -D warnings
```

모두 종료 0이어야 한다. `cargo test -p slot2-gfx`에서 GL phase가 skip됐는지 실행됐는지 보고한다.
검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test나 device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\67-gfx-shader-effects.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- 공개 `ShaderEffect`와 Canvas/RecordingCanvas 계약
- 네 effect의 실제 동작과 UV/tint/source·destination size 처리
- effect compile 실패 fallback, error 조회와 GL object 정리 결과
- 일반 UI/present program으로 effect가 새지 않는 근거
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄과 GL phase 실제 실행/skip
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 실제 Mali compile·화질은 확인하지 않았다면 그대로 명시
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
