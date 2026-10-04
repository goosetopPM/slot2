# Task 63 — 인게임 Device 메뉴 App 배선

현재 checkout에서 직접 작업한다. Task62의 `DeviceMenu`를 인게임 메뉴의 Device 행에 연결하고,
이미 App에 존재하는 `slot2_audio::Volume`의 level/mute를 같은 상태로 조작·표시한다. 현재 확정된
backend가 없는 Brightness와 Blue light는 `Unavailable`로 유지한다. 이번 태스크는 **실행 중 volume
상태와 UI 배선만** 다루며 platform 제어와 설정 파일 저장을 새로 만들지 않는다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\63-device-menu-app-wiring.md`
- `C:\SLOT2\tasks\62-device-menu-ui.result.md`
- `C:\SLOT2\crates\slot2-ui\src\device_menu.rs`의 공개 API와 derive
- `C:\SLOT2\crates\slot2-ui\src\in_game_menu.rs`의 Device 행 의미만
- `C:\SLOT2\crates\slot2-audio\src\volume.rs`
- `C:\SLOT2\crates\slot2\src\app.rs`의 Screen, input dispatch, volume, draw, audio pause와
  기존 Display/Cheats/Core submenu 배선 부분만
- `C:\SLOT2\crates\slot2\tests\display_scale_app.rs`와 직접 관련된 인게임 submenu test helper만
- `C:\SLOT2\docs\DECISIONS.md`의 D-23만

직접 관련된 test fixture와 Canvas operation helper만 추가로 읽는다. platform hardware 구현, store,
다른 마일스톤, 워커 로그와 저장소 이력은 읽지 않는다.

## 현재 계약과 범위 판단

- `App.volume`은 이미 실행 중 전역 volume 상태이며 기본 level은 70이다. 물리 `VolUp`/`VolDown`도
  모든 화면에서 이 객체를 변경한다.
- `Volume::STEP`은 5다. `step_up`은 mute를 해제하고 level을 5 올려 100에서 clamp하며,
  `step_down`은 mute 여부를 유지하고 level을 5 내려 0에서 clamp한다. mute는 기억된 level을 바꾸지 않는다.
- 현재 `slot2-platform`에는 검증된 brightness/blue-light API가 없다. RG SP 경로도 확정되지 않았으므로
  `DeviceMenu::new`에는 두 capability를 모두 `None`으로 넘긴다. 가짜 값, sysfs/PWM 추측, no-op backend를
  만들지 않는다.
- 기존 volume은 설정 파일에 저장되지 않는 runtime 상태다. 이번 태스크도 그 수명을 바꾸지 않는다.
  game settings, platform settings 또는 새 파일을 만들지 않는다.
- 인게임 overlay가 열린 동안 game frame과 audio는 정지한다. volume 변경을 미리 들려주기 위한 sample,
  sink 재생 또는 core frame 진행을 추가하지 않는다. 메뉴를 닫고 gameplay가 재개되면 기존 audio 경로가
  변경된 `App.volume`을 사용한다.

## 구현 계약

### 1. Screen과 열기

- `slot2_ui::DeviceMenu`를 App에 import하고 `Screen`에 parent `InGameMenu`와 Device menu snapshot을
  함께 보존하는 variant를 추가한다. 기존 Screen의 `Copy` 계약을 유지하기 위해 모든 필드가 Copy인
  `DeviceMenu`에 `Copy` derive를 추가해도 된다. 내부 구조를 바꾸거나 별도 duplicate model을 만들지 않는다.
- 인게임 Device 행에서 A를 누르면 다음 값으로 화면을 연다.
  - volume: 현재 `self.volume.level()`
  - muted: 현재 `self.volume.is_muted()`
  - brightness: `None`
  - blue light: `None`
- parent menu의 선택 위치를 그대로 보존한다. session, sink, toast, fast-forward latch와 card/store를
  열거나 닫거나 변경하지 않는다.
- app.rs 상단의 상태 전이 설명을 실제 Device 동작에 맞게 갱신한다.

### 2. Device 화면 입력과 단일 volume 상태

- Up/Down은 `DeviceMenu::up`/`down`을 호출한다. 이번 capability에서는 Volume만 available이므로 어느
  방향도 Volume에 머문다. 향후 capability를 예상해 App 쪽에 두 번째 탐색 규칙을 만들지 않는다.
- selected setting이 Volume일 때:
  - Left는 `self.volume.step_down()`과 같은 의미다.
  - Right는 `self.volume.step_up()`과 같은 의미다.
  - A는 `self.volume.toggle_mute()`와 같은 의미다.
- 각 변경 직후 열린 DeviceMenu snapshot의 level/muted를 실제 `self.volume`에서 다시 읽어 동기화한다.
  메뉴 snapshot을 먼저 바꾸거나 두 volume 상태를 독립적으로 계산하지 않는다.
- 방어적으로 unavailable setting이 선택된 상태라면 Left/Right/A는 아무 동작도 하지 않는다. capability를
  생성하거나 volume 대신 다른 setting을 바꾸지 않는다.
- B 또는 Menu tap은 `Screen::InGame(parent)`로 돌아가며 Device 행과 같은 parent 상태를 보존한다.
- Power의 기존 전역 동작과 그 외 버튼의 기존 의미를 바꾸지 않는다. Device 화면의 관련 없는 버튼은
  core/session으로 전달하지 않는다.
- 물리 `VolUp`/`VolDown`은 기존처럼 모든 화면에서 전역 volume을 변경한다. Device 화면이 열려 있을 때도
  그 뒤 snapshot을 실제 volume에서 동기화해 화면이 stale하지 않게 한다. 다른 화면에서의 기존 동작은
  그대로다. 같은 처리를 중복시키지 않도록 작은 App helper로 중앙화해도 된다.

### 3. pause와 draw

- Device 화면을 `audio_paused()`의 menu 집합에 포함한다.
- Device 화면에서는 기존 Display/Cheats/Core와 같이 session의 마지막 game frame을 먼저 그린 뒤
  Device overlay를 그린다. wallpaper, shelf, HUD, parent InGameMenu를 그리지 않고 clear하지 않는다.
- overlay는 hold progress와 toast보다 아래에 그린다. session이 없더라도 panic하지 않고 overlay는
  정직하게 그린다.
- draw 때문에 core frame이 진행되거나 sink request가 생기면 안 된다.

### 4. runtime 수명

- 메뉴에서 바꾼 level/mute는 B/Menu로 닫은 뒤와 Device 메뉴를 다시 열었을 때 유지된다.
- Playing으로 돌아간 뒤 다음 정상 gameplay frame/audio 처리에서 기존 `App.volume`이 그대로 사용된다.
- eject, 재시작, 새 App 생성 이후의 영속성은 이번 범위가 아니다. settings read/write나 migration을
  추가하지 않는다.

## 테스트 계약

`crates/slot2/tests/device_menu_app.rs`를 새로 만들고, 필요한 경우 app.rs의 직접 관련 기존 unit test를
새 Screen 계약에 맞게 고친다. 최소한 다음을 직접 검증한다.

- 실행 중 인게임 메뉴의 Device 행에서 열면 parent가 같은 Device 행이고 snapshot은 기본 70,
  unmuted, brightness/blue-light `None`, selected Volume이다.
- Device 화면은 audio paused이며 session/core frame을 진행하지 않고 sink request를 만들지 않는다.
- draw 순서는 마지막 game frame 뒤 Device dim/overlay이며 wallpaper, shelf, HUD, parent menu와 clear가 없다.
- brightness/blue-light가 unavailable인 현재 App에서 Up/Down은 Volume에 머문다.
- Left/Right는 정확히 `Volume::STEP`만큼 변경하고 0/100에서 clamp한다.
- muted 상태에서 Left는 mute를 유지하고 기억된 level만 내리며, Right는 unmute하고 level을 올린다.
- A는 level을 바꾸지 않고 mute만 왕복한다.
- Device 화면에서 물리 VolUp/VolDown을 눌러도 `App.volume`과 menu snapshot이 즉시 같고 위 mute 규칙을
  따른다. Device 밖에서의 전역 물리 volume 동작도 회귀하지 않는다.
- B와 Menu는 각각 sink/session을 건드리지 않고 같은 Device parent row로 돌아간다.
- 닫은 뒤 다시 열면 변경한 runtime level/mute가 보이며, Playing으로 돌아간 뒤에도 App state가 유지된다.
- Device 조작 중 settings/card 파일 write가 없다.

가능하면 기존 test fixture와 gitignore된 MIT `arm.gba`/vendored mGBA 패턴을 재사용한다. core-dependent
fixture가 실제로 없을 때만 그 테스트를 명시적으로 skip하고 보고한다. 테스트를 통과시키려고 가짜 draw,
padding upload, fixture 전용 분기를 만들지 않는다. 실기, 실제 audio sink, platform hardware는 테스트하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-ui\src\device_menu.rs` (`Copy` derive가 필요한 경우만)
- `C:\SLOT2\crates\slot2\src\app.rs`
- `C:\SLOT2\crates\slot2\tests\device_menu_app.rs` (신규)
- App의 새 Screen variant 때문에 직접 깨지는 기존 `crates\slot2\tests\*_app.rs` 한정
- `C:\SLOT2\tasks\63-device-menu-app-wiring.worker-result.md`

그 밖의 파일은 수정하지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- brightness/blue-light platform API, sysfs/PWM 탐색이나 hardware write
- volume의 settings/store persistence, 새 설정 key/file 또는 migration
- `slot2-audio`의 Volume 의미와 sink/audio pipeline 변경
- DeviceMenu layout, 문구, localization, upload 계약의 재설계
- core 전환, save state, display, cheat 동작 변경
- 전체 workspace 테스트, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2 -p slot2-ui
cargo clippy -p slot2 -p slot2-ui --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test나
device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\63-device-menu-app-wiring.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- Screen/open/parent 복귀와 brightness/blue-light unavailable 처리
- 메뉴 Left/Right/A 및 물리 VolUp/VolDown의 level/mute·snapshot 동기화 결과
- pause, frame/overlay draw, sink/session/store 비변경 결과
- runtime 유지와 재열기 결과
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄 및 core-dependent skip 수
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
