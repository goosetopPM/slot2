# Task 67 작업자 결과 — GFX 내장 셰이더 효과 기반

## 판정

**성공.** 누적 호출 1/2 (이번이 첫 호출). 완료 기준 5개 명령 모두 종료 0이며, 최종 검증 뒤 코드
변경 없음.

## 공개 계약

- **`slot2_gfx::ShaderEffect`** 공개. variant는 `SharpBilinear`, `Lcd3x`, `ZfastCrt`, `Scanline`
  정확히 네 개, `Clone, Copy, Debug, PartialEq, Eq, Hash` 파생. `ShaderEffect::ALL`은
  `[SharpBilinear, Lcd3x, ZfastCrt, Scanline]` 고정 순서이며, 프로그램 배열 index도 이 순서다.
  `Off`는 넣지 않았다(계약대로 기본 image draw이므로 renderer enum에 없다).
- **`Canvas::image_effect_uv(tex, x, y, w, h, [u0,v0,u1,v1], tint, effect)`** 추가와 full-UV 편의
  `Canvas::image_effect`. trait 기본 구현은 같은 geometry/UV/tint로 `image_uv`를 호출한다.
- **`Op::ImageEffect { tex, x, y, w, h, uv, tint, effect }`** 추가. `RecordingCanvas`는 effect draw를
  `Op::Image`로 뭉개지 않고 effect까지 기록하며, origin은 일반 draw와 같은 방식으로 더한다. 기존
  `Op::Image`/`image`/`image_uv`의 의미와 기록은 그대로다.
- **`GlCanvas::shader_available(effect) -> bool`**, **`GlCanvas::shader_error(effect) -> Option<&str>`**
  공개. 성공 effect의 error는 `None`이다.
- store 의존·`Cargo.toml` 변경 없음. gfx는 여전히 `slot2_store`를 모른다.

## 네 effect의 동작

공통: 모든 fragment source가 GLSL ES 1.00(`attribute`/`varying`/`texture2D`, `#ifdef GL_ES`
precision, `#ifdef GL_FRAGMENT_PRECISION_HIGH`로 highp 선택)이며, `u_src`(원본 texture 전체 크기),
`u_dst`(destination 크기), `u_uv_rect`(UV crop) uniform을 모두 선언하고 실제로 사용한다. alpha는
`c.a * v_col.a`로만 바뀌어 transparent가 불투명해지지 않고, tint RGB는 기존 `image`와 같은 곱셈
의미다. 모든 neighbor tap은 crop 안으로 clamp하는 `crop_uv()`를 지난다.

- `SharpBilinear`: texel 내부는 nearest, seam만 bilinear. seam 폭을 destination 1.5px로 잡고
  `pixels_per_texel`로 나눠 clamp(1..8)하므로 확대 배율에 따라 좁아지고 1:1에서는 일반 동작이다.
  scanline/mask 없음.
- `Lcd3x`: destination 3픽셀 주기 stripe(1.30/0.85/0.85 세 조합, 평균 1.0이라 단색 밝기 보존)와
  source cell 경계에서 최대 20% 어두워지는 약한 grid. alpha는 stripe·grid에 곱하지 않는다.
- `ZfastCrt`: 수평 이웃 0.35 beam blend(1:1에서는 0)와 source row 1주기 `cos` scan modulation.
  multi-pass·history·curvature 없음.
- `Scanline`: source row 하단 절반을 35% 어둡게 하는 band만. color mask·blur·hue 변화 없음. 원본
  row가 destination 1px 이하일 때만 세기를 줄여 moire를 막고, band는 항상 source row에 고정된다.
- 네 source는 서로 다르며 서로 다른 수학을 쓴다(stripe phase/`mod`, seam `mix`, `cos`, band).

## effect 실패 fallback과 GL object 정리

- `link_program()` 하나로 compile/link를 처리하며, vertex compile 실패·fragment compile 실패(vertex
  shader 삭제)·link 실패(program 삭제)와 성공(양쪽 shader 삭제) 모든 경로에서 object를 정확히
  삭제한다. 기존 기본 program의 link 실패 시 program 잔존도 이 helper로 정리됐다.
- 기본 program 실패는 기존대로 `GfxError::Shader`로 canvas 생성을 실패시킨다. effect 하나의 실패는
  canvas를 실패시키지 않고 오류 문자열만 보존한다. `Drop`은 `effects.iter().flatten()`으로 성공한
  program만 정확히 한 번 삭제한다.
- **검증(임시 probe 후 삭제, 최종 코드 아님)**: Scanline source에 (a) 문법 오류를 넣으면
  `available=false`, error=드라이버 log(`ERROR: 0:47: '}' : syntax error …`)이고 canvas 생성과 나머지
  세 effect는 정상이며 Scanline draw는 기본 image draw와 **byte 단위 동일(diff 0)**. (b) vertex에 없는
  varying을 넣어 link 실패를 유발해도(`Out of resource error` log) canvas는 생성되고 fallback draw는
  diff 0. 즉 compile 실패와 link 실패 양쪽 모두에서 격리와 fallback 동작을 확인했다.
- freed/unknown `TexId`는 effect 경로와 기본 경로 모두 panic 없이 같은 결과(diff 0)였다.

## program state가 새지 않는 근거

- effect draw는 이전 batch를 먼저 flush하고, quad 하나를 그린 뒤 `UseProgram(self.program)`으로 기본
  program을 복원한다. 이후 `rect`/`image`/`mask`는 `image_uv`가 다시 기본 program과 `u_panel`을
  설정해 그린다. `present`는 손대지 않은 기본 program 경로 그대로다.
- GL phase에서 effect 뒤에 그린 작은 불투명 rect 두 점이 base 실행과 정확히 같은 색([0,200,255])임을
  네 effect 모두에서 확인했다. `RecordingCanvas` 쪽에서도 effect op 뒤의 rect/image/image_uv가 각각
  `Op::Rect`/`Op::Image`로 남는 것을 테스트로 고정했다.

## 완료 기준 명령 (최종 코드에서 순서대로 1회)

| 명령 | 종료 코드 | 마지막 결과 줄 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | (출력 없음) |
| `cargo test -p slot2-gfx` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`(doc-tests) — lib **19**, screenshot **1**, shader_effects **5** = **25 passed / 0 failed** |
| `cargo check -p slot2-gfx --no-default-features --features device` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.32s`` |
| `cargo check -p slot2 --tests` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 17.94s`` |
| `cargo clippy -p slot2-gfx --all-targets -- -D warnings` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.03s`` |

**GL phase**: 위 `cargo test -p slot2-gfx`는 `SLOT2_GFX_TEST`가 없어 **skip**됐다(screenshot 1 passed
0.02s). 별도로 `SLOT2_GFX_TEST=1`로 실행했을 때는 **실제로 실행**됐고 1 passed(1.24s)였다. 그 실행의
context는 `OpenGL ES 2.0 - Build 32.0.101.7088` / `Intel(R) UHD Graphics`, `glfn::is_es() == true`로
확인했다(임시 probe로 확인 후 제거). 즉 `#ifdef GL_ES` + highp precision 분기가 실제 GLES2
드라이버에서 compile·link됐다.

- 네 effect 모두 `shader_available == true`, `shader_error == None`.
- 8x8 checker를 400x300으로 확대한 quad에서 base 대비 panel 전체 채널 차이 합계:
  SharpBilinear 4,295,442 / Lcd3x 8,522,656 / ZfastCrt 26,292,600 / Scanline 8,880,000. shipped
  테스트는 quad 영역 기준 200,000 초과를 요구하며(통과), pass-through(0)와는 자릿수가 다르다. 네
  effect 상호간 차이도 각각 200,000 초과를 통과했다.
- crop 경계 sentinel: `[0.25,0.25,0.75,0.75]` crop 바깥을 강한 red ring으로 채우고 네 effect를
  적용했을 때 crop 내부 어떤 픽셀도 ring 색이 새지 않았다(`r - max(g,b) < 100`, 회색 실측 최대
  57).

## 생성·수정 파일

- 신규: `crates/slot2-gfx/src/shader.rs` (허용된 신규 내부 module 1개; `ShaderEffect`, 네 GLSL source)
- 신규: `crates/slot2-gfx/tests/shader_effects.rs` (5 tests)
- 수정: `crates/slot2-gfx/src/lib.rs`, `crates/slot2-gfx/src/canvas.rs`,
  `crates/slot2-gfx/src/gl_canvas.rs`, `crates/slot2-gfx/tests/screenshot.rs`
- `Cargo.toml`과 다른 crate는 수정하지 않았다. 임시 probe 파일(`tests/probe_fallback.rs`)과 임시
  GL version probe는 검증 후 제거했고 최종 tree에 남아 있지 않다.
- 최종 검증 뒤 코드 변경 없음. 커밋·푸시·네트워크·실기 접근 없음.

## 계약 재량 사항 두 가지 (보고)

- source 형태 검사(`#ifdef GL_ES`/precision/uniform/main/GLES2 금지 construct)는 계약이 허용한
  "동등한 집중 테스트"로 판단해 `src/shader.rs`의 unit test에 넣었다. source는 `pub(crate)`이고
  공개 API로 노출할 이유가 없어 통합 테스트에서 볼 수 없다. enum/기록/origin/전파/fallback은
  `tests/shader_effects.rs`에 있다.
- **Mali G31 compile과 실제 화질은 확인하지 않았다.** GL phase가 실행된 것은 이 Windows 호스트의
  Intel GLES2 드라이버이며, 720x480 패널에서의 실제 화질·계수 적정성은 실기 검증 대상이다.

## 남은 위험 (1줄)

effect의 `u_src`는 "원본 texture 전체 크기"라는 뜻이므로 후속 Session 배선에서 crop 크기를 넘기면
neighbor tap clamp가 한 texel씩 밀리고, 수치 계수(beam 0.35/scan 0.22/band 0.35/stripe 1.30)는
호스트 GLES2 한 종류에서만 확인된 값이다.

## 소요 시간

약 35분.
