# Task 68 — Session 게임 셰이더 라우팅

현재 checkout에서 직접 작업한다. Task66의 게임별 shader 설정을 Task67의 gfx effect로 변환해 Session의
게임 frame draw에 적용한다. 이번 태스크는 **Session과 그 집중 테스트만** 다룬다. 플랫폼별 기본값,
Display 메뉴 UI와 App의 즉시 변경·저장은 후속 태스크로 분리한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\68-session-shader-routing.md`
- `C:\SLOT2\tasks\66-game-shader-settings-store.result.md`
- `C:\SLOT2\tasks\67-gfx-shader-effects.result.md`
- `C:\SLOT2\crates\slot2\src\session.rs`의 `Session` 필드, `start`/`start_named`/`open`,
  `upload_video`, `draw`, scale accessor 주변만
- `C:\SLOT2\crates\slot2\tests\session.rs`의 fixture, settings, draw/scale 테스트 주변만
- `C:\SLOT2\crates\slot2-store\src\settings.rs`의 `ShaderPreset` 정의만
- `C:\SLOT2\crates\slot2-gfx\src\canvas.rs`의 effect draw와 `Op` 정의만

직접 필요한 test helper만 추가로 읽는다. App, UI, platform/retro registry, shader GLSL source,
다른 태스크·워커 로그와 저장소 이력은 읽지 않는다.

## 현재 계약과 의미

- `Session::open`은 이미 game settings를 한 번 읽고 scale, overscan, rewind를 초기화한다. shader 때문에
  두 번째 settings read나 별도 파일 I/O를 만들지 않는다.
- store의 `GameSettings::shader`는 다음 세 의미를 갖는다.
  - `None`: 플랫폼 기본값 상속
  - `Some(ShaderPreset::Off)`: 이 게임은 셰이더를 명시적으로 끔
  - 나머지 네 preset: 해당 효과를 명시적으로 선택
- 아직 `PlatformDef`에 shader 기본값이 없다. 그러므로 **이번 단계에서** `None`과 explicit Off는 둘 다
  gfx의 일반 image draw가 된다. 둘의 저장 의미를 합치거나 ini를 고치지 않으며, `None`을 특정 preset으로
  추측하지 않는다. 후속 플랫폼 기본값 태스크가 `None`만 별도로 해석한다.
- gfx의 `ShaderEffect`에는 Off가 없다. 일반 `Canvas::image_uv`가 no-effect 경로다.
- 셰이더는 game texture quad에만 적용한다. Session 바깥에서 뒤이어 그리는 HUD·메뉴·toast에는
  `Canvas::image_effect_uv`를 호출하지 않는다.

## 구현 계약

### 1. 단일 변환 경계

- store `Option<ShaderPreset>`을 runtime `Option<ShaderEffect>`로 바꾸는 Session 소유 helper를 하나만
  둔다. 이름은 의미가 분명해야 하며 launch와 후속 live apply가 같은 helper를 쓸 수 있게 공개한다.
- 매핑은 정확히 다음과 같다.
  - `Some(SharpBilinear)` → `Some(ShaderEffect::SharpBilinear)`
  - `Some(Lcd3x)` → `Some(ShaderEffect::Lcd3x)`
  - `Some(ZfastCrt)` → `Some(ShaderEffect::ZfastCrt)`
  - `Some(Scanline)` → `Some(ShaderEffect::Scanline)`
  - `Some(Off)` → `None`
  - 설정 key 부재의 `None` → `None` (플랫폼 기본값 미구현 상태의 임시 plain 결과)
- gfx나 store에 상호 dependency를 추가하지 않는다. 변환은 두 crate를 이미 아는 `slot2::session`에만 둔다.
- 문자열이나 enum 순서/index로 변환하지 말고 exhaustive match로 쓴다. 새 variant가 생기면 compile 시
  이 경계를 다시 검토하게 해야 한다.

### 2. Session 상태와 launch

- Session에 현재 runtime `Option<ShaderEffect>`를 보관한다.
- `Session::open`이 이미 읽은 `settings.shader`를 위 helper로 변환해 초기화한다. `start`와
  `start_named`는 모두 `open`을 거치므로 같은 설정을 사용해야 한다.
- 현재 effect를 읽는 accessor와 runtime에서 바꾸는 setter를 제공한다. 이름은 각각
  `shader_effect()` / `set_shader_effect(...)` 의미면 된다.
- setter는 renderer 상태만 바꾼다. 카드 설정을 읽거나 쓰지 않고, core restart/reset, frame advance,
  video texture upload/free, audio/sink 변경을 하지 않는다.
- Debug 출력에 shader를 추가할 필요는 없다. 다른 Session 설정과 수명 의미를 바꾸지 않는다.

### 3. 게임 frame draw

- 기존 scale placement, aspect와 overscan crop 계산을 그대로 한 번 수행한다.
- runtime effect가 `Some`이면 game texture를 `Canvas::image_effect_uv`로 그리고, `None`이면 기존
  `Canvas::image_uv`로 그린다.
- 두 경로는 texture id, destination x/y/w/h, crop UV와 white tint가 정확히 같아야 한다. effect를
  적용한다는 이유로 placement를 다시 계산하거나 full UV로 되돌리지 않는다.
- frame이 아직 없거나 texture가 upload되지 않은 경우는 기존처럼 아무 image도 그리지 않는다.
- draw는 frame을 진행하거나 settings를 다시 읽지 않는다. paused menu 아래 반복 draw에서도 마지막
  texture와 effect를 그대로 사용한다.
- Session은 `GlCanvas::shader_available`을 알 필요가 없다. Canvas가 지원하지 않거나 특정 program이
  실패한 경우의 plain fallback은 Task67의 Canvas 계약에 맡긴다.

## 테스트 계약

`crates/slot2/tests/session.rs`에 집중 회귀 테스트를 추가한다. 기존 real-core fixture와 명시적 skip 규칙을
재사용하되, pure mapping 검증은 core 유무와 무관하게 항상 실행한다.

- pure helper가 store의 여섯 입력(`None`, Off, 네 preset)을 위 표와 정확히 매핑함
- shader key가 없는 fresh Session은 accessor가 `None`이고 draw가 기존 `Op::Image` 하나를 사용함
- explicit Off Session도 accessor가 `None`이며 `Op::Image`를 사용하되 카드에는 `Some(Off)`가 그대로
  남아 있어 상속과 저장 의미가 합쳐지지 않음
- 네 preset 각각으로 시작한 Session이 정확한 `ShaderEffect`를 보유하고 game draw를 정확히 하나의
  `Op::ImageEffect`로 기록함
- effect op의 texture, destination geometry, overscan UV와 white tint가 같은 fixture의 plain
  `Op::Image`와 동일함
- scale override와 shader override가 함께 있을 때 scale placement와 effect가 동시에 적용됨
- `set_shader_effect`로 네 effect 및 `None`을 바꾸면 다음 draw op만 즉시 바뀌고 `frames_run`, core,
  session, texture upload/update/free count와 카드 settings bytes는 바뀌지 않음
- `start_named`도 같은 game shader setting을 읽어 새 Session에 적용함
- frame/texture가 없는 상태에서 effect가 설정돼도 image/effect op가 생기지 않음
- 기존 no-setting scale/draw와 texture-reuse 테스트가 계속 통과함

하나의 화면 결과를 확인하기 위해 GPU/GL window를 열지 않는다. `RecordingCanvas` op로 routing과
geometry를 검증한다. pass-through가 effect op인 것처럼 보이게 만드는 테스트 전용 분기는 금지한다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2\src\session.rs`
- `C:\SLOT2\crates\slot2\tests\session.rs`
- `C:\SLOT2\tasks\68-session-shader-routing.worker-result.md`

다른 production/test 파일은 수정하지 않는다. 구현이 불가능하다고 판단되면 범위를 넓히지 말고
보고서에 이유를 적어 실패로 둔다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- `slot2-store`, `slot2-gfx`, `slot2-ui`, App, platform/retro registry 변경
- 플랫폼별 shader 기본값 또는 `PlatformDef` 확장
- Display 메뉴 shader 선택 UI, 입력, 저장 실패 toast와 live persistence
- shader source·계수·GL program/fallback 수정, overlay/bezel 기능
- Session start 실패 정책, core/scale/overscan/rewind/cheat/state 동작 변경
- 전체 workspace 테스트, GL 창, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2 --test session
cargo test -p slot2 --lib
cargo check -p slot2 --tests
cargo clippy -p slot2 --all-targets -- -D warnings
```

모두 종료 0이어야 한다. core-dependent test가 skip되면 정확한 개수와 이유를 보고한다. 검증 뒤 코드를
바꾸면 영향받는 명령부터 다시 실행한다. workspace test, GL test와 device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\68-session-shader-routing.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- store preset → gfx effect 단일 변환과 None/Off 의미
- launch/start_named, accessor/setter와 game draw routing 결과
- geometry/UV/tint 보존, setter의 frame/I/O/texture/audio 무변경 근거
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄과 core-dependent skip 수
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 플랫폼 기본값과 App/UI가 범위 밖으로 남았다는 확인
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
