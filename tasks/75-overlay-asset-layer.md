# Task 75 — Geometry별 Overlay asset layer

현재 checkout에서 직접 작업한다. D-11의 플랫폼×geometry PNG를 찾고 검증·decode해 texture로 한 번만
올린 뒤 panel 전체에 alpha blend하는 독립 Overlay asset layer를 `slot2` crate에 추가한다. 이번
태스크는 **asset resolution과 renderer 수명만** 다룬다. Task74의 게임별 setting 해석, Session/App
draw 배선, Display 메뉴와 실제 샘플 bezel 제작은 후속 태스크로 분리한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\75-overlay-asset-layer.md`
- `C:\SLOT2\tasks\74-game-overlay-settings-store.result.md`
- `C:\SLOT2\crates\slot2\Cargo.toml`
- `C:\SLOT2\crates\slot2\src\lib.rs`
- `C:\SLOT2\crates\slot2-gfx\src\canvas.rs`의 `Canvas`, `TexId`, `RecordingCanvas`, `Op` 부분만
- `C:\SLOT2\crates\slot2-ui\src\image.rs` (기존 PNG decode 관례 참고만; 수정하지 않는다)
- `C:\SLOT2\crates\slot2-platform\src\profile.rs`의 `Geometry` 부분만
- `C:\SLOT2\crates\slot2-store\src\card.rs`의 `Card::root`, `Platform::folder` 부분만
- `C:\SLOT2\docs\DECISIONS.md`의 D-11만
- `C:\SLOT2\docs\DESIGN.md`의 표시 파이프라인과 overlay 경로 부분만

App/Session/UI menu, shader/overscan 구현, 실제 overlay 그림, build script, 워커 로그와 저장소
이력은 읽지 않는다. 구현에 필요한 `png 0.17` API는 현재 lockfile과 기존 UI decoder 사용법 안에서
확인한다. 네트워크나 새 버전 검색을 하지 않는다.

## 현재 계약과 경계

- 한 overlay asset은 정확히 `(slot2_store::Platform, slot2_platform::Geometry)` 한 쌍에 속한다.
- 카드 override 경로는 정확히
  `System/Overlays/<PLAT>/<geometry>.png`다.
  - `<PLAT>`은 `Platform::folder()`의 기존 대문자 표기 (`GB`, `GBC`, `GBA`, `NES`, `SNES`, `MD`,
    `SMS`)를 그대로 사용한다.
  - `<geometry>`는 `Geometry`의 기존 Display 표기 (`640x480`, `720x480`, `720x720`)다.
- 내장 asset은 runtime filesystem에서 `assets/`를 읽지 않는다. 기기 배포에는 repository의
  `assets/`가 통째로 복사되지 않으므로, 내장 PNG는 후속 샘플 태스크가 `include_bytes!`로 등록할
  정적 byte slice여야 한다.
- D-11의 현재 기본값은 없음이다. 따라서 이 태스크의 production 내장 표는 비어 있어도 맞다.
  다만 내장 asset 등록·조회·override 우선순위를 테스트할 수 있는 공개 계약은 완성한다.
- asset layer는 사용 여부를 결정하지 않는다. `overlay == None/Some(true)/Some(false)` 해석은 후속
  App/Session 경계가 맡는다. 이 layer는 호출자가 전달한 sources만 그린다.
- overlay는 game layer 위, UI layer 아래에 놓일 일반 RGBA image다. shader effect, crop, scale,
  dim, clear를 자체 적용하지 않는다.

## 구현 계약

### 1. 공개 asset descriptor와 resolver

`crates/slot2/src/overlay.rs`를 새로 만들고 `slot2::overlay`로 공개한다.

- `BuiltInOverlay`는 최소한 platform, geometry, stable id, `&'static [u8]` PNG bytes를 가진다.
  `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`가 가능하도록 설계한다.
- production 내장 목록 `BUILT_IN_OVERLAYS`를 공개한다. 이번 태스크에서는 실제 그림을 넣지 않고
  빈 slice다. 후속 샘플 태스크가 이 표만 채울 수 있어야 한다.
- 카드 경로를 만드는 공개 pure helper를 제공한다. 임의 문자열을 받지 말고 `Card`, `Platform`,
  `Geometry`로부터 위 정확한 경로를 만든다.
- resolver는 카드의 해당 path가 regular file이면 card source와 같은 platform/geometry의 내장
  fallback을 함께 돌려준다. file이 없으면 내장 source만, 둘 다 없으면 빈 sources를 돌려준다.
- production resolver는 `BUILT_IN_OVERLAYS`를 사용한다. 테스트와 후속 asset 등록 검증을 위해
  명시적인 built-in slice를 받는 동등 helper를 함께 제공해도 된다.
- 같은 platform/geometry로 중복 등록된 내장 entry가 있다면 **첫 entry**를 사용한다. panic이나
  nondeterministic 선택을 하지 않는다.
- 카드 파일은 내장 asset보다 우선한다. 다만 card PNG가 손상됐거나 규격이 틀리면 내장본을 decode해
  폴백한다. 손상된 사용자 파일 때문에 정상 내장 bezel까지 사라지게 하지 않는다.
- resolver는 파일을 열거나 decode하지 않는다. path 존재/regular-file 여부와 descriptor 선택까지만
  수행한다. frame마다 filesystem을 조회하는 API를 만들지 않는다.

### 2. 엄격한 PNG decode

- `png = "0.17"`을 `slot2`의 normal dependency로 둔다. 같은 crate가 dev-dependency에 이미 있으면
  중복 선언을 정리한다. 다른 dependency를 추가하지 않는다.
- path 또는 내장 bytes를 같은 내부 decoder로 처리한다.
- PNG의 width/height는 선택된 geometry의 panel size와 **정확히 같아야 한다**. 작거나 큰 그림을
  stretch/crop/downsample하지 않는다. geometry별 파일이라는 계약 위반은 decode 실패다.
- header에서 크기를 확인한 뒤에만 output pixel buffer를 할당한다. 거대한 잘못된 PNG의 선언 크기만
  보고 큰 allocation을 만들지 않는다.
- palette/transparency, RGB, RGBA, grayscale, grayscale+alpha와 16-bit 입력을 png crate의 안전한
  transformation으로 최종 straight RGBA8로 변환한다. 알파를 premultiply하거나 버리지 않는다.
- interlaced/지원 가능한 정상 PNG는 png crate가 해석하는 범위에서 허용한다.
- malformed/truncated/unsupported data는 panic하지 않고 설명 가능한 오류 문자열로 남긴다. 경로와
  전체 PNG bytes를 오류 문자열에 넣지 않는다.
- decode 결과의 길이는 항상 `width * height * 4`이며 overflow를 검사한다.

### 3. `OverlayLayer` texture 수명과 draw

- `OverlayLayer`는 현재 resolved sources, upload된 texture 하나, 실패 상태를 가진다. `Default`는
  sources도 texture도 없는 상태다.
- sources를 같은 값으로 다시 설정하면 texture를 버리거나 재decode하지 않는다.
- sources가 바뀌면 기존 texture는 다음 canvas 접근 시 정확히 한 번 `free`하고 새 source를 lazy
  decode/upload한다. sources가 비어도 이전 texture는 해제된다.
- `draw(&mut self, canvas)` 또는 동등 API는 다음을 지킨다.
  - canvas size가 sources의 geometry와 다르면 upload/draw하지 않고 오류를 남긴다.
  - card source가 정상이라면 그것만 한 번 decode/upload하고 내장 bytes는 건드리지 않는다.
  - card source가 실패하면 오류를 보존하고 정상 내장 source가 있으면 그것으로 한 번 폴백한다.
  - 두 source 모두 실패하거나 없으면 아무 image도 그리지 않는다.
  - 성공 texture는 `(0, 0, panel_w, panel_h)`, full UV, white tint의 일반 `Canvas::image`로 매 draw
    한 번 그린다. `clear`, rect, `image_effect`, game placement 변경을 하지 않는다.
  - 같은 sources의 다음 frame은 filesystem open/decode/upload 없이 기존 texture만 draw한다.
- 현재 실제로 사용한 source가 Card/BuiltIn/None 중 무엇인지, card fallback을 포함한 마지막 오류가
  무엇인지 테스트와 향후 log가 읽을 수 있는 좁은 accessor를 제공한다. 오류가 없으면 `None`이다.
- `release(&mut self, canvas)` 또는 동등 API로 texture를 즉시 한 번 free하고 빈 runtime 상태로 만들 수
  있어야 한다. `Drop`에서 canvas 없이 GL texture를 해제하려 하지 않는다.
- 실패한 sources는 같은 sources가 다시 설정되기 전까지 매 frame 재decode하지 않는다. source를
  다른 값 또는 빈 값으로 바꿨다가 되돌리면 새 시도로 본다.

## 테스트 계약

`crates/slot2/tests/overlay_layer.rs`를 새로 만들고 임시 파일과 생성한 작은 PNG fixture로 최소한 다음을
직접 검증한다. 실제 repository asset이나 실기 GL은 사용하지 않는다.

- 7개 플랫폼×3개 geometry의 카드 path가 정확한 대문자 platform/geometry 파일명으로 만들어짐
- regular file만 card source이고, missing path와 directory는 override로 취급되지 않음
- built-in lookup은 platform과 geometry가 모두 같은 첫 entry만 선택함
- valid card PNG가 valid built-in보다 우선하며 upload RGBA bytes가 card pixel임
- card가 없으면 built-in을 사용함
- corrupt/truncated 또는 wrong-size card는 built-in으로 폴백하고 오류를 조회할 수 있음
- fallback도 없거나 깨졌으면 upload/image가 없고 두 번째 draw에서도 재시도하지 않음
- RGB/RGBA/grayscale/grayscale-alpha/palette+transparency 및 16-bit fixture가 정확한 straight RGBA8로
  decode됨. 적어도 transparent pixel 하나가 alpha 0을 그대로 유지함
- header의 wrong/huge dimensions는 output-sized allocation 전에 거부됨
- 성공 draw가 panel 전체의 일반 `Op::Image` 하나이며 clear/rect/effect를 만들지 않음
- 같은 sources를 반복 설정·draw해도 upload는 한 번이고 frame마다 image만 하나씩 추가됨
- source 변경과 빈 sources 전환이 이전 texture를 정확히 한 번 free함
- `release`를 두 번 호출해도 double-free하지 않고 이후 draw도 없음
- canvas size 불일치는 upload/draw 없이 오류가 남음

`RecordingCanvas`가 upload pixel bytes를 보존하지 않으므로, 실제 decoded RGBA를 확인할 때는 테스트
안의 최소 capture Canvas를 사용해도 된다. production Canvas/Op에 pixel payload를 추가하지 않는다.
테스트는 파일 mtime, sleep, permission bit, 외부 프로그램과 네트워크에 의존하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2\Cargo.toml`
- `C:\SLOT2\crates\slot2\src\lib.rs`
- `C:\SLOT2\crates\slot2\src\overlay.rs` (신규)
- `C:\SLOT2\crates\slot2\tests\overlay_layer.rs` (신규)
- `C:\SLOT2\Cargo.lock` (`png`의 dependency 위치 변경으로 실제로 바뀌는 경우만)
- `C:\SLOT2\tasks\75-overlay-asset-layer.worker-result.md`

다른 production/test 파일 변경은 허용하지 않는다. 구현이 불가능하다고 판단되면 범위를 넓히지 말고
보고서에 이유를 적어 실패로 둔다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- 실제 `assets/overlays/` PNG 생성·추가, artwork 디자인과 라이선스 결정
- Task74의 `GameSettings::overlay` 해석, platform registry의 overlay 기본값
- Session/App 필드와 game draw 순서 배선, session start/eject 때 source 교체
- Display/Overlay 메뉴, 번역, 즉시 미리보기, 저장과 toast
- shader/scale/overscan/game texture 로직 변경
- `slot2-gfx`, `slot2-ui`, `slot2-store`, `slot2-platform`, `slot2-retro` 변경
- build/dist script와 배포 산출물 변경
- 전체 workspace 테스트, 실제 GL/device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2 --test overlay_layer
cargo check -p slot2 --tests
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2 --test overlay_layer -- -D warnings
```

모두 종료 0이어야 한다. 검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test,
실제 GL test와 device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\75-overlay-asset-layer.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- 정확한 card path, built-in 등록과 override/fallback 결과
- PNG size/color/alpha 검증과 oversized header 방어 결과
- lazy upload, cache, source 변경/free/release와 draw geometry 결과
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- App/Session/UI 배선과 실제 sample PNG는 구현하지 않았다는 명시
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
