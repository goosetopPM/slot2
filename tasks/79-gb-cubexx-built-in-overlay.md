# Task 79 — GB × CubeXX 내장 오버레이 샘플

현재 checkout에서 직접 작업한다. D-11과 M5가 예시로 든 **Game Boy × RG CubeXX(720×720)** 조합에
재현 가능한 내장 bezel PNG 하나를 추가하고 production `BUILT_IN_OVERLAYS`에 등록한다. 기본 overlay
설정은 계속 off이며, 게임별 설정에서 On을 고른 경우에만 표시된다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\79-gb-cubexx-built-in-overlay.md`
- `C:\SLOT2\tasks\75-overlay-asset-layer.result.md`
- `C:\SLOT2\tasks\78-overlay-menu-app-wiring.result.md`
- `C:\SLOT2\docs\DECISIONS.md`의 D-09, D-11, D-23만
- `C:\SLOT2\docs\MILESTONES.md`의 M5 overlay 항목만
- `C:\SLOT2\crates\slot2\src\overlay.rs`의 `BuiltInOverlay`, production table, resolver/decode 부분만
- `C:\SLOT2\crates\slot2\tests\overlay_layer.rs`의 built-in resolution/decode/fallback 테스트만
- `C:\SLOT2\crates\slot2-gfx\src\fit.rs`의 Integer placement 함수와 GB 160×144 placement에 필요한
  부분만
- `C:\SLOT2\crates\slot2-platform\src\profile.rs`의 `W720H720` 크기만
- `C:\SLOT2\crates\slot2-retro\src\registry.rs`의 GB native size만

App/menu/store/Session/core, 다른 assets, 다른 태스크·로그와 저장소 이력은 읽지 않는다.

## 현재 계약

- D-11은 overlay를 플랫폼×geometry별 panel-size PNG로 정의하고 기본값을 없음/off로 둔다.
- GB native 160×144를 기본 `ScalePolicy::Integer`로 720×720 panel에 놓으면 4×인 **640×576**, origin은
  **(40,72)**다. 이 rectangle은 게임 화면이므로 sample PNG에서 전부 alpha 0이어야 한다.
- built-in은 card override의 fallback이다. `System/Overlays/GB/720x720.png`가 regular file이면 먼저
  시도하고, 없거나 decode가 실패할 때 내장 sample을 쓴다.
- overlay picture는 panel 전체에 straight RGBA8로 draw된다. aperture에 색을 칠하면 게임을 가리므로
  “거의 투명”이 아니라 alpha byte가 정확히 0이어야 한다.
- 사용자가 Aspect fit/Fill 등 다른 scale을 고를 수 있으나 이 sample은 **GB 기본 Integer 배치용**이다.
  그 사실을 README에 명시한다. scale에 따라 asset을 바꾸는 새 기능은 만들지 않는다.

## 구현 계약

### 1. 재현 가능한 asset 생성

- `build/generate-overlays.py`를 추가한다. Python 표준 라이브러리만 사용하고 Pillow, ImageMagick,
  network, 외부 font/tool에 의존하지 않는다.
- 인자 없이 repository root 어디에서 실행해도 정확히
  `assets/overlays/GB/720x720.png`를 생성한다. script 자신의 위치에서 repository root를 계산한다.
- PNG는 720×720, RGBA8, straight alpha, top row first다. timestamp, machine path, random 값 등 실행마다
  달라지는 metadata를 넣지 않는다. 같은 source에서 반복 실행한 bytes가 같아야 한다.
- artwork는 다음 제약 안에서 단순한 기하 도형으로 만든다.
  - aperture rectangle: `x=40..679`, `y=72..647`의 640×576 모든 pixel이 `(0,0,0,0)`
  - aperture 밖은 모두 alpha 255이며, game pixel과 섞이지 않는 어두운 graphite base
  - aperture 바로 바깥에 muted olive inner rim, 상·하 여백에 절제된 muted plum/olive accent line
  - text, logo, 상표, 캐릭터, 외부 artwork를 넣지 않는다
  - 번쩍이는 고채도색, 반투명 haze, noise와 pixel마다 달라지는 random texture를 넣지 않는다
- 정확한 RGB palette와 선 두께는 script 상단의 이름 붙은 상수로 둔다. 생성 코드를 읽으면 어느 영역에
  어떤 색이 들어가는지 알 수 있어야 하며, 주석은 디자인 의도만 설명한다.
- `assets/overlays/README.md`에 generated source, 대상 pair, 640×576 aperture/origin, 기본 Integer 전용,
  카드 override 경로, 제3자 asset을 쓰지 않았다는 사실과 재생성 명령을 짧게 적는다.

### 2. production 등록

- `assets/overlays/GB/720x720.png`를 repository asset으로 추가한다.
- `BUILT_IN_OVERLAYS`에 정확히 한 entry를 둔다.
  - `platform: Platform::Gb`
  - `geometry: Geometry::W720H720`
  - `id: "gb-cubexx-frame-v1"`
  - `png: include_bytes!("../../../assets/overlays/GB/720x720.png")`
- table/comment에서 “비어 있음/후속 artwork”라는 낡은 설명을 현재 사실로 고친다.
- `overlay_enabled(None) == false`와 `Some(false) == false`는 그대로 둔다. built-in entry가 생겼다는 이유로
  platform default를 On으로 바꾸지 않는다.
- 다른 6 platform과 GB의 640×480/720×480 pair에는 built-in을 추가하지 않는다. GBC를 GB와 자동 공유하지
  않으며, platform fallback이나 geometry stretching도 만들지 않는다.

### 3. asset/table 검증

기존 `overlay_layer.rs`에 production table/asset 테스트를 추가한다.

- production table 길이가 1이고 pair/id가 위 값과 정확히 일치하며 중복 pair/id가 없음
- built-in bytes가 production decoder에서 720×720 RGBA8로 성공하고 decoded byte 수가
  `720 * 720 * 4`임
- aperture 640×576의 모든 alpha가 0이고 RGB도 0임
- aperture 밖의 모든 alpha가 255이며 최소 세 개의 palette colour가 실제로 존재함
- `(40,72,640,576)`이 `slot2_gfx::place(Integer, (160,144), ..., (720,720))` 결과와 정확히 같음. 테스트에
  별도 magic placement를 하나 더 구현하지 않는다.
- `resolve`가 빈 card에서 GB×720×720에 built-in을 주고, 다른 20 platform×geometry pair에는 built-in을
  주지 않음
- 유효한 card override는 built-in보다 먼저 사용되고, corrupt/wrong-size card file은 production
  built-in으로 fallback하며 `OverlaySourceKind::BuiltIn`과 card failure를 함께 남김
- source가 production built-in뿐인 layer가 panel-size texture를 한 번 upload하고 full-panel image를
  frame마다 draw하되 재upload하지 않음
- `overlay_enabled(None/Some(false))`가 여전히 false이고 `Some(true)`만 true라 sample 추가가 기본 동작을
  바꾸지 않음
- generator를 연속 두 번 실행한 SHA-256이 같고, 두 번째 생성 뒤 tracked PNG가 달라지지 않음

binary PNG의 compressed bytes 전체를 테스트 상수로 복제하지 않는다. pixel/layout 계약과 deterministic
generation을 검사한다. 특정 texture id를 만들기 위한 padding upload/draw도 금지한다.

### 4. 문서 상태

- `docs/MILESTONES.md` M5의 “셰이더 프리셋 4종 + 플랫폼 기본값, 오버레이 로더 + GB·CubeXX용 샘플
  베젤” 항목을 `[x]`로 바꾼다. 이미 완료된 shader/loader와 이번 sample을 합쳐 이 항목이 닫힌다.
- 다른 milestone checkbox와 결정문은 변경하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\build\generate-overlays.py` (신규)
- `C:\SLOT2\assets\overlays\README.md` (신규)
- `C:\SLOT2\assets\overlays\GB\720x720.png` (신규)
- `C:\SLOT2\crates\slot2\src\overlay.rs`
- `C:\SLOT2\crates\slot2\tests\overlay_layer.rs`
- `C:\SLOT2\docs\MILESTONES.md`의 위 M5 checkbox 한 줄
- `C:\SLOT2\tasks\79-gb-cubexx-built-in-overlay.worker-result.md`

다른 production/test/docs 파일은 수정하지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- 다른 platform/geometry overlay와 GBC 공유 fallback
- platform default On 변경, 새 setting/menu/App/Session 동작
- scale-aware overlay 선택과 AspectFit/Fill용 별도 aperture
- PNG decoder, texture/cache/lifecycle, draw order와 card precedence 변경
- 외부 artwork/font/package 다운로드, network 사용
- 전체 workspace 테스트, 실제 GL 창, 실기 배포
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 asset/code 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
python build/generate-overlays.py
$first = (Get-FileHash assets/overlays/GB/720x720.png -Algorithm SHA256).Hash
python build/generate-overlays.py
$second = (Get-FileHash assets/overlays/GB/720x720.png -Algorithm SHA256).Hash
if ($first -ne $second) { throw "overlay generator is not deterministic" }
cargo fmt --all -- --check
cargo test -p slot2 --test overlay_layer --test overlay_app --test overlay_menu_app
cargo test -p slot2
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2 --all-targets -- -D warnings
powershell -File build/dist-device.ps1
```

모두 종료 0이어야 한다. test 명령의 passed/failed/ignored 합계와 core-dependent skip 수를 보고한다.
`dist-device.ps1` 마지막 줄은 `==> done`이어야 한다. 검증 뒤 generator/source/asset/code를 바꾸면
영향받는 명령부터 다시 실행한다. workspace test, 실제 GL test와 device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\79-gb-cubexx-built-in-overlay.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- asset path, SHA-256, byte size, geometry/aperture와 palette/alpha 결과
- generator의 두 번 실행 결정성, 표준 라이브러리만 사용한 근거
- production table pair/id와 다른 20 pair 부재 결과
- card precedence, corrupt fallback, decode/upload/cache와 default-off 보존 결과
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored와 skip 수
- `dist-device` 마지막 줄
- 생성·수정 파일 및 최종 검증 뒤 변경 여부
- 다른 geometry/scale와 실기 화질 확인이 범위 밖으로 남았다는 확인
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
