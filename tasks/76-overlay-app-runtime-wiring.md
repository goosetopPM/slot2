# Task 76 — Overlay App runtime 배선

현재 checkout에서 직접 작업한다. Task74의 게임별 overlay 설정과 Task75의 `OverlayLayer`를 실제 플레이
경로에 연결한다. App이 session의 game frame 직후 overlay를 그리고 그 뒤에 HUD·인게임 UI를 그리며,
launch/core switch/recovery/eject의 source·texture 수명을 일관되게 유지한다. 이번 태스크는 **runtime
배선만** 다룬다. Display/Overlay 메뉴, 설정 변경·저장, 실제 내장 sample PNG는 후속 태스크로 분리한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\76-overlay-app-runtime-wiring.md`
- `C:\SLOT2\tasks\74-game-overlay-settings-store.result.md`
- `C:\SLOT2\tasks\75-overlay-asset-layer.result.md`
- `C:\SLOT2\crates\slot2\src\overlay.rs`
- `C:\SLOT2\crates\slot2\src\app.rs`에서 `App` fields/constructor, `start_launch`, `stop_session`,
  core switch/recovery와 `draw` 부분만
- `C:\SLOT2\crates\slot2\src\session.rs`에서 `cart`, `upload_video`, `draw` 부분만
- `C:\SLOT2\crates\slot2\tests\display_menu_app.rs`의 합성 GBA/NES App·draw fixture 패턴만
- `C:\SLOT2\crates\slot2\tests\core_picker_app.rs`의 core switch 성공·복구 fixture 패턴만
- `C:\SLOT2\docs\DECISIONS.md`의 D-11과 D-23만
- `C:\SLOT2\docs\DESIGN.md`의 표시 파이프라인과 인게임 Display 부분만

UI menu 구현, 실제 overlay artwork, PNG decoder 내부, shader source, store safe-write 구현, worker 로그와
저장소 이력은 읽지 않는다.

## 현재 계약과 책임

- `GameSettings::overlay`의 세 값은 이번 태스크에서 한 곳에서 runtime 의미로 바뀐다.
  - `None`: 플랫폼 기본값 상속. D-11의 현재 플랫폼 기본값은 **없음**이므로 현재는 disabled.
  - `Some(true)`: 이 게임에서 overlay enabled.
  - `Some(false)`: 명시적 disabled.
- 이 변환을 공개 pure helper 하나로 둔다. `unwrap_or(false)`로 뜻을 숨기지 말고 exhaustive match로
  세 상태를 적는다. 향후 registry 기본값이 생기면 이 한 경계만 바뀌어야 한다.
- Session은 core frame의 upload, shader, crop과 placement만 계속 소유한다. overlay texture는
  `OverlayLayer`가 App에서 소유한다. Session에 PNG/path/texture 필드를 추가하지 않는다.
- App의 실제 device/host geometry는 `tuning.geometry`의 `(640,480)`, `(720,480)`, `(720,720)` 중
  하나다. 이 tuple을 `slot2_platform::Geometry`로 바꾸는 pure exhaustive helper를 한 곳에 둔다.
  지원하지 않는 크기는 overlay disabled로 안전하게 처리하며 game launch 자체를 실패시키지 않는다.
- production `BUILT_IN_OVERLAYS`는 아직 비어 있으므로 이 태스크 뒤 실제로 보이는 overlay는 카드의
  `System/Overlays/<PLAT>/<geometry>.png`가 있을 때뿐이다. 테스트는 카드 PNG를 생성해 사용한다.

## 구현 계약

### 1. App 소유와 설정 해석

- `App`에 private `OverlayLayer` field를 하나 추가하고 constructor에서 `Default`로 초기화한다.
- `overlay_enabled(setting: Option<bool>) -> bool` 또는 동등한 공개 pure helper를 `overlay.rs`에 둔다.
  exhaustive match 결과는 `None => false`, `Some(true) => true`, `Some(false) => false`다.
- `(u32,u32) -> Option<Geometry>` helper도 한 곳에 둔다. 정확히 세 지원 크기만 variant로 바꾸고 다른
  값은 `None`이다. 문자열 format/parse나 profile 이름 추측을 사용하지 않는다.
- 성공한 **최초 launch**에서 이미 확정된 `cart`의 settings를 한 번 읽는다.
  - enabled이고 geometry가 지원되면 `overlay::resolve(&self.card, cart.platform, geometry)`를
    `OverlayLayer::set_sources`에 전달한다.
  - disabled, unsupported geometry이면 `clear_sources`한다.
  - 이 resolution은 launch 경계에서 한 번만 한다. frame마다 settings나 filesystem을 읽지 않는다.
- Session start가 실패하면 overlay source를 활성화하지 않는다.
- core switch 성공과 recovery 성공은 같은 cart/settings/geometry의 같은 playthrough이므로 기존
  `OverlayLayer` sources와 texture를 유지한다. core를 바꿨다는 이유로 PNG를 free/redecode/reupload하지
  않는다.
- core switch가 완전히 복구되지 않아 session이 사라지는 경로, 일반 eject/stop과 process exit 경로는
  overlay sources를 비운다. canvas가 없는 stop 함수에서 texture id를 직접 free하려 하지 않는다.

### 2. draw 순서와 lazy free

- game-bearing 화면의 공통 draw 경계를 만든다. 최소한 다음 화면에서 순서는 같다.
  - `Playing`
  - `InGame`, `Display`, `Shader`, `Overscan`, `Cheats`, `Switcher`, `Core`, `Device`
  - running session 위의 `Power`
- 순서는 정확히 다음과 같다.
  1. `Session::upload_video`
  2. `Session::draw` — clear + game image/effect
  3. `OverlayLayer::draw` — panel 전체의 일반 alpha image
  4. time-control HUD(표시되는 화면에서만)
  5. 현재 menu/dim/hold progress/toast 등 UI
- overlay를 Session draw보다 먼저 그려 clear에 지워지게 하거나, menu/dim/HUD 뒤에 그려 UI를
  가리지 않는다. overlay 자체에는 dim/clear/effect를 적용하지 않는다.
- `Inserting` 동안 session이 먼저 준비돼도 overlay를 카트 삽입 애니메이션 위에 그리지 않는다.
  `Playing`으로 전환되어 game frame을 실제로 그린 뒤부터 보인다.
- session이 없는 draw에서는 먼저 layer sources를 빈 상태로 맞춘 뒤 `OverlayLayer::draw`를 한 번
  호출해 pending texture를 소유 canvas에서 free할 기회를 준다. 그 호출은 image를 그리지 않아야
  한다. 이후 shelf/splash/eject UI에는 overlay가 없어야 한다.
- session이 예외적으로 없는 game-bearing screen에서도 overlay만 단독으로 그리지 않는다.
- 같은 running session의 매 frame은 overlay image만 다시 그리고 upload는 최초 한 번이다.
- invalid/missing card PNG는 Task75 계약대로 game을 막지 않고 overlay 없이 계속한다.

### 3. core switch와 종료 수명

- core switch를 시작하려고 기존 Session을 잠시 내리는 것은 overlay 종료가 아니다. 성공한 새 core나
  성공한 기존 core recovery가 같은 cart로 돌아오면 현재 texture를 그대로 쓴다.
- core setting save 실패 뒤 recovery도 같은 overlay를 유지한다.
- recovery까지 실패해 `session = None`/Ejecting이 되면 sources를 비우고 다음 draw에서 texture를 한 번
  free한다.
- MENU hold eject, InGame Eject, power Restart/PowerOff와 다른 기존 `stop_session` 호출은 모두 source를
  비운다. 기존 save/Resume flush, sink request, screen 전환과 오류 처리를 바꾸지 않는다.
- App/OverlayLayer의 `Drop`에서 canvas 없이 free하려는 새 코드를 만들지 않는다. 정상 frame의 lazy
  free와 기존 GlCanvas drop이 안전망이다.

## 테스트 계약

`crates/slot2/tests/overlay_app.rs`를 새로 만들고 기존 합성 ROM/core fixture를 좁게 재사용하거나 같은
패턴의 최소 fixture를 만든다. `vendor/mgba_libretro.dll`이 없으면 성공으로 skip하지 말고 테스트를
실패시켜 보고서에 환경 문제를 명시한다. 최소한 다음을 직접 검증한다.

- pure setting helper가 `None`, `Some(true)`, `Some(false)`를 각각 false/true/false로 해석함
- geometry helper가 세 지원 크기를 정확한 variant로 바꾸고 다른 크기는 `None`임
- GBA game에 `overlay = on`과 exact 720x480 card PNG가 있으면 최초 Playing draw에서 game
  Image/ImageEffect 뒤에 panel 전체 일반 Image가 그려짐
- 다음 Playing frame은 overlay upload 없이 같은 texture의 overlay image만 다시 그림
- key 부재와 explicit off는 PNG가 있어도 overlay upload/image를 만들지 않음
- enabled지만 file missing/corrupt/wrong-size인 경우 game frame은 정상 draw되고 overlay image는 없음
- InGame과 Display 같은 menu screen에서 game < overlay < dim/menu text 순서이며 overlay가
  ImageEffect가 아닌 일반 Image임
- running game 위 Power menu에서도 game < overlay < Power UI 순서임
- session이 준비된 Inserting frame에는 overlay image가 없음
- MENU hold 또는 기존 stop 경로 뒤 Ejecting의 다음 draw가 overlay texture를 정확히 한 번 free하고
  overlay image를 그리지 않으며, 그 다음 frame에 double-free하지 않음
- no-session game-bearing screen을 직접 만들거나 failure fixture로 도달해도 overlay만 단독 draw하지 않음
- 가능하면 기존 core-switch fixture로 성공/recovery 경로가 overlay를 free/reupload하지 않는 것을
  고정한다. 환경상 두 core 실행이 필요한 검증이 불안정하면 production 경로를 바꾸지 말고 그 이유를
  보고서에 남기며, 동일 cart에 `set_sources`를 다시 하지 않는 좁은 App 단언으로 대체한다.
- overlay 기능 추가가 game texture의 shader effect, UV crop, placement, frame/core/audio/sink 동작을
  바꾸지 않음

PNG는 테스트 중 임시 card에 생성하고 repository `assets/`에 넣지 않는다. `RecordingCanvas`의 texture
id와 op 순서로 검증하며 특정 id 숫자를 맞추기 위한 padding upload나 불필요한 draw를 추가하지 않는다.
sleep/mtime, 실제 창, GL, network와 실기에 의존하지 않는다.

기존 app/session/display 테스트의 draw 순서 단언은 overlay가 disabled인 기본 설정에서 이전 결과가
그대로여야 한다. 기존 단언을 느슨하게 하거나 삭제하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2\src\overlay.rs`
- `C:\SLOT2\crates\slot2\src\app.rs`
- `C:\SLOT2\crates\slot2\tests\overlay_app.rs` (신규)
- overlay field 추가로 컴파일에 꼭 필요한 기존 `App` literal/test helper의 좁은 수정
- `C:\SLOT2\tasks\76-overlay-app-runtime-wiring.worker-result.md`

다른 production crate, manifest, lockfile와 asset 변경은 허용하지 않는다. 기존 테스트 파일 수정이
필요하면 보고서에 파일과 이유를 정확히 적고 overlay-disabled 기존 단언을 유지한다. 관련 없는 미커밋
변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- Display 메뉴 Overlay 행, 별도 Overlay menu, 번역, 설정 write와 save-failure toast
- runtime에서 overlay on/off를 바꾸는 공개 UI 경로와 즉시 preview
- production `BUILT_IN_OVERLAYS` entry 또는 실제 `assets/overlays/` PNG 추가
- platform registry 기본 overlay 변경
- PNG decoder/resolver/Canvas/Session game renderer 재설계
- shader/scale/overscan/core/cheat 동작 변경
- build/dist script와 배포 산출물 변경
- 전체 workspace 테스트, 실제 GL/device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2 --test overlay_layer --test overlay_app
cargo test -p slot2
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2 --all-targets -- -D warnings
```

모두 종료 0이어야 한다. `cargo test -p slot2`의 총 passed/failed/ignored와 core-dependent skip 수를
보고한다. 검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test, 실제 GL test와
device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\76-overlay-app-runtime-wiring.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- setting/geometry 해석과 launch resolution 횟수
- game→overlay→HUD/UI draw 순서와 Inserting/no-session 제외 결과
- cache, core switch/recovery 보존, stop/eject free 결과
- missing/broken asset fallback과 기존 shader/UV/placement/core/audio 보존 결과
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄과 core skip 수
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 메뉴/저장 배선과 실제 내장 sample은 구현하지 않았다는 명시
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
