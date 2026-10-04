# Task 86 — About 스티커 App 배선

현재 checkout에서 직접 작업한다. Task 85의 `AboutSticker`를 선반 설정 메뉴에 연결한다. 시간대와
About 두 행을 활성화하고, About 화면이 실제 frontend package version과 현재 profile target을 표시한
뒤 B/Menu로 같은 About 행에 복귀하게 한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

필요한 부분만 읽는다.

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\84-shelf-timezone-app-wiring.result.md`
- `C:\SLOT2\tasks\85-about-sticker-ui.result.md`
- `C:\SLOT2\crates\slot2\src\app.rs`의 `Screen`, Shelf/Timezone input helper와 draw/HUD 분기만
- `C:\SLOT2\crates\slot2\tests\timezone_menu_app.rs`의 Shelf availability 관련 단언만
- `C:\SLOT2\crates\slot2-ui\src\shelf_menu.rs`의 public availability/choice API만
- `C:\SLOT2\crates\slot2-ui\src\about_sticker.rs`의 public draw API와 layout 상수만
- `C:\SLOT2\crates\slot2-platform\src\profile.rs`의 `Profile::target` 필드만
- `C:\SLOT2\crates\slot2\Cargo.toml`의 package version 상속만

store/platform 구현, Session/core, host/device loop, 다른 메뉴, 번역 파일, 워커 로그와 저장소 이력은
읽지 않는다. 새 Screen variant 때문에 직접 깨지는 App test가 있으면 해당 setup/단언 주변만 추가로
읽는다.

## 확정 UX와 정보 출처

- 선반 설정 메뉴에서 사용 가능한 행은 **TimeZone과 About 두 개**다. 나머지 Language,
  DisplayDefaults, BootLogo, Sync는 계속 비활성이다.
- 메뉴 고정 순서상 처음 열면 TimeZone이 선택된다. Up/Down은 TimeZone ↔ About 사이를 양방향
  순환하며 비활성 행을 건너뛴다.
- About 행에서 A는 About 스티커를 연다.
- About 화면에서 B 또는 Menu 탭은 파일·clock·volume을 건드리지 않고 About 행이 선택된 parent
  Shelf로 돌아간다.
- About 화면에서 A와 모든 방향 버튼은 no-op이다. 가짜 링크, URL, filesystem viewer를 만들지 않는다.
- `version`은 **`slot2` crate의 `env!("CARGO_PKG_VERSION")`**, `target`은 draw 시점의
  **`ctx.profile.target`**을 `AboutInfo`로 전달한다.
- git SHA, host OS, 환경변수, 카드 파일에서 build 정보를 읽지 않는다.

## 구현 계약

### 1. Shelf availability

- `App` 안에 선반 설정의 현재 가용성을 한 곳에서 만드는 private helper를 둔다. 의미는 정확히 다음과
  같다.

```rust
ShelfAvailability {
    time_zone: true,
    about: true,
    ..Default::default()
}
```

- `open_shelf_menu`는 이 값을 사용한다. `slot2-ui::ShelfAvailability`에 이번 빌드 전용 helper를 새로
  추가하지 않는다.
- 동일 가용성 literal을 input/draw/test 편의를 위해 production 여러 곳에 복제하지 않는다.
- 기존 시간대 진입, preview, apply/cancel, 실패 rollback은 바꾸지 않는다.

### 2. Screen과 입력

`Screen`에 다음 variant를 추가한다. 이름과 payload는 그대로 사용한다.

```rust
About(ShelfMenu, AboutSticker),
```

- `Screen`의 `Clone + Copy + Debug + PartialEq + Eq`를 유지한다.
- Shelf의 A 분기는 선택 choice에 따라 명시적으로 나눈다.
  - `Some(ShelfChoice::TimeZone)` → 기존 `open_timezone_menu`
  - `Some(ShelfChoice::About)` → `Screen::About(parent, AboutSticker::new())`
  - 그 밖의 choice/None → no-op
- 비활성 행을 활성화하거나 비활성 선택을 위한 refusal/toast를 만들지 않는다.
- About의 `Action::Tap(Button::B | Button::Menu)`는 `Screen::Shelf(parent)`다.
- About의 다른 Tap/Hold action은 no-op이다. PowerMenu, List Menu tap/hold, 시간대 입력 동작을 바꾸지
  않는다.
- About는 선반에서만 열리므로 game audio pause 집합에 넣지 않는다.

### 3. draw와 build 정보 전달

- `Screen::About`의 바탕은 List/Shelf/Timezone과 정확히 같은 wallpaper + shelf다.
- base shelf를 한 번 그린 뒤 `AboutSticker`만 그린다. parent Shelf panel, Timezone panel을 아래에
  겹쳐 그리지 않는다.
- draw 호출은 정확히 다음 정보 의미를 전달한다.

```rust
AboutInfo {
    version: env!("CARGO_PKG_VERSION"),
    target: ctx.profile.target,
}
```

- App struct나 Screen payload에 version/target `String`을 저장하지 않는다. build/runtime source에서
  draw할 때 빌려 쓴다.
- hold progress와 toast는 기존처럼 AboutSticker보다 위에 그린다.
- shelf HUD의 battery/time은 About에서도 계속 보인다. UTC validity와 timezone runtime 값을 읽는
  기존 순서를 바꾸지 않는다.
- `draw_overlay_nothing`을 List/Shelf/Timezone과 같은 base 경로에서 호출해 이전 game overlay texture가
  남지 않게 한다.
- 기존 모든 `Screen` match의 exhaustive 분기를 새 variant에 맞게 갱신하되 insert/eject, session,
  volume, rescan, power와 game menu 동작은 바꾸지 않는다.

## 테스트 계약

`crates/slot2/tests/about_sticker_app.rs`를 새로 추가한다. core/ROM/GPU가 필요 없는 integration test로
다음을 검증한다.

1. List → Menu 탭으로 연 Shelf는 TimeZone과 About만 available이고 처음 선택은 TimeZone이다.
2. Up/Down이 두 활성 행 사이를 양방향 순환하고 네 비활성 행을 한 번도 선택하지 않는다.
3. About 선택에서 A가 `Screen::About(parent, AboutSticker::new())`를 열며 parent는 About 행을 유지한다.
4. B와 Menu 탭 각각이 같은 About 행의 parent Shelf로 돌아간다.
5. About의 A/방향 입력이 screen, clock, volume, 카드 bytes와 toast를 바꾸지 않는다.
6. `RecordingCanvas`에서 About 화면의 base marks가 List의 wallpaper+shelf와 같고, full-panel dim과
   About panel은 각각 하나다. Shelf/Timezone panel은 겹쳐 그리지 않으며 clear가 없다.
7. App draw가 `about-version`에 `env!("CARGO_PKG_VERSION")`, `about-target`에
   `ctx.profile.target`을 실제로 전달한다. 다른 placeholder 값이나 UI crate version을 넣어도 통과하는
   단언은 금지한다.
8. List/Shelf/About에서 HUD가 유지되고 About screen은 `audio_paused() == false`다.

기존 `crates/slot2/tests/timezone_menu_app.rs`는 새 가용성 때문에 직접 깨지는 최소 부분만 갱신한다.

- exact `timezone_only()` 기대를 TimeZone+About 가용성으로 바꾼다.
- “유일한 행”이라는 설명·단언을 두 활성 행 순환 계약으로 바꾼다.
- 기존 12개 시간대 시나리오와 단일 `#[test]`, preview/save/rollback 단언은 유지하고 완화하지 않는다.

RecordingCanvas만 사용한다. filesystem은 기존 Card 임시 디렉터리 외 사용하지 않고, 실제 GL 창,
core, ROM, wall-clock sleep, texture id 번호에 의존하지 않는다. 테스트 전용 production hook, unsafe,
새 dependency를 만들지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2\src\app.rs`
- `C:\SLOT2\crates\slot2\tests\about_sticker_app.rs` (신규)
- `C:\SLOT2\crates\slot2\tests\timezone_menu_app.rs`의 Shelf availability 관련 설명·단언만
- 새 Screen variant 때문에 직접 깨지는 기존 App test의 exhaustive match/setup만
- `C:\SLOT2\tasks\86-about-sticker-app-wiring.worker-result.md`

다른 production/test 파일은 수정하지 않는다. 특히 `slot2-ui`, i18n/FTL, store/platform,
host/device loop, Cargo 파일을 고치지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- 언어, 플랫폼 기본 화면, 부팅 로고, 연동 행 활성화·구현
- About 문구·layout 변경, license 파일 추가·수정
- git SHA/build.rs/새 환경변수, URL·QR code·license viewer
- 시간대 store/runtime/UI 계약 변경
- M4/M5/M7 문서 checkbox 완료 처리
- 전체 workspace 테스트, 실제 GL 창, device 배포
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2 --test about_sticker_app
cargo test -p slot2 --test timezone_menu_app
cargo test -p slot2
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2 --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 각 test 명령의 passed/failed/ignored 합계와 core-dependent skip 수를 보고한다.
검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test와 device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\86-about-sticker-app-wiring.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- TimeZone+About availability와 선택 순환 결과
- About Screen 진입/B·Menu 복귀/no-op 입력 결과
- frontend package version/profile target 전달 증거
- shelf base, 단일 About layer, HUD/audio 결과
- 기존 timezone_menu_app 12개 시나리오 보존 결과
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored와 skip 수
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 계약이 틀려 보이는 부분과 남은 위험
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
