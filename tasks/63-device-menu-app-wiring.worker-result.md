# Task 63 — 인게임 Device 메뉴 App 배선 (결과)

- 결과: **성공**. 누적 호출 **1/2**(이번이 첫 호출이며 재시도하지 않았다).
- 시도 1의 provider 400 같은 중단은 없었고, 이전 Task62가 남긴 `DeviceMenu`를 그대로 배선했다.

## Screen / 열기 / parent 복귀

- `slot2_ui::{DeviceMenu, DeviceSetting}`을 import하고 `Screen::Device(InGameMenu, DeviceMenu)`를
  추가했다. `Screen`의 `Copy/Debug/PartialEq/Eq` 계약을 지키려고 `DeviceMenu`에 `Copy` derive만
  추가했다(내부 구조·duplicate model 없음).
- 인게임 메뉴 Device 행 A → `open_device_menu`가 `DeviceMenu::new(volume.level(), volume.is_muted(),
  None, None)`으로 연다. 즉시 실행 중 기본값 **level 70, unmuted, brightness/blue-light `None`,
  selected Volume**이다. 세션 유무와 무관하게 열린다(볼륨은 게임이 아니라 프론트엔드 소유이고,
  draw 계약이 "session이 없더라도 overlay를 정직하게 그린다"를 요구한다).
- 부모 `InGameMenu`는 값 그대로 보존된다(테스트가 `selected_index`·`choice` 동일을 단언). 열기는
  session·sink·toast·store·card·ff_latch를 건드리지 않는다(`take_sink_request()` None, session 유지).
- brightness/blue light는 어느 경로에서도 값을 만들지 않는다: `new(..., None, None)` 고정, setter도
  unavailable 행을 되살리지 않으므로 두 행은 화면에 보이되 선택 불가다. sysfs/PWM/no-op backend 없음.
- 기존 7행이 전부 배선돼 `InGame` A 매치의 `_ => {}`(미배선 행 안내 주석)가 도달 불가가 되어 제거했다.

## Left/Right/A 와 물리 VolUp/VolDown 동기화

- Left = `self.volume.step_down()`, Right = `self.volume.step_up()`, A = `self.volume.toggle_mute()`.
  변경 직후 `device_menu()`가 `self.volume`에서 snapshot을 **다시 만들어** 넣는다. 메뉴를 먼저 바꾸거나
  두 상태를 독립 계산하지 않는다.
- selected가 Volume이 아니면 `change_volume`이 즉시 반환한다(no-op). capability 생성이나 volume 대신
  다른 setting 변경 없음.
- 물리 VolUp/VolDown은 기존 전역 arm에서 volume을 바꾼 뒤 `refresh_device_menu()`로 열린 Device 화면의
  snapshot만 다시 읽는다. 중앙화된 한 곳이며 다른 화면 동작은 그대로다.
- 결과: level은 0/100에서 clamp, mute는 Volume에만 속하고 step_down은 mute 유지·step_up은 해제,
  A는 level을 바꾸지 않고 mute만 왕복한다. 물리 키도 같은 규칙을 따른다(테스트 확인).
- Up/Down은 `DeviceMenu::up/down`만 호출한다. App 쪽 두 번째 탐색 규칙 없음 → Volume에 머문다.

## pause / draw / 비변경

- `audio_paused()`에 `Screen::Device(..)`를 포함. 실제 session으로 `audio_paused()==true`, 메뉴 조작
  후에도 `frames_run()` 불변, `take_sink_request()` None(열기·닫기 모두)을 확인했다.
- draw: wallpaper 제외 목록과 frame-arm에 Device 추가 → session의 마지막 프레임(`upload_video`+`draw`)
  뒤에 `DeviceMenu::draw`. clear 없음, wallpaper/shelf/HUD/parent InGameMenu 없음. overlay는 hold
  progress와 toast보다 아래, HUD는 원래 Playing 이외 화면에 없으므로 Device에서도 없음.
- session이 없는 Device 화면도 panic 없이 dim/panel/행/hint를 그린다(첫 mark가 full-panel dim).
- store 비변경: Device 조작(Left/Right/A/VolUp/VolDown/Up/Down) 전후 카드 전체 파일 트리(path+bytes)가
  동일하고 `read_settings`가 default 그대로임을 단언했다.

## runtime 유지와 재열기

- B와 Menu는 각각 `Screen::InGame(parent)`로 돌아가고 Device 행·부모 상태를 보존한다. session/sink 불변.
- 메뉴에서 level 60 + mute로 바꾼 뒤 B→B로 Playing 복귀: `App.volume`이 60/muted 그대로이고 코어가
  다시 진행한다(`frames_run()` 증가). 다시 열면 60/muted가 보이며 selected는 Volume이다.
- 새 설정 key/file·settings 저장·migration 없음. `slot2_audio::Volume` 의미도 변경 없음.

## 완료 기준 명령 (원문 순서, 마지막 코드 변경 뒤, 이후 코드 변경 없음)

1. `cargo fmt --all -- --check` → exit **0**, 출력 없음.
2. `cargo test -p slot2 -p slot2-ui` → exit **0**. `test result:` 줄 41개, 합계 **398 passed / 0 failed /
   0 ignored**. 신규 `device_menu_app` **13 passed**(1.34s), 기존 `device_menu`(slot2-ui) 10 passed,
   slot2 lib unit 33 passed. 마지막 결과 줄: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured;
   0 filtered out; finished in 0.00s` (slot2_ui doc-tests).
3. `cargo clippy -p slot2 -p slot2-ui --all-targets -- -D warnings` → exit **0**,
   `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 7.61s`.

- core-dependent skip: **0**. 같은 명령을 `--nocapture`로 재실행해 확인했다(398 passed 동일,
  `slot2: playing` 82줄). "skipping" 2건은 이번 변경과 무관한 기존 GL 창 검사
  (`shelf_shot.rs::the_shelf_holds_up_on_every_panel`, `splash.rs::splash_renders_for_real`,
  `SLOT2_GFX_TEST` 미설정)이다.
- 새 variant 때문에 직접 깨진 기존 app.rs unit test 1개를 새 계약에 맞게 고쳤다:
  `the_rows_with_nothing_behind_them_stay_in_the_menu`의 case 목록에서 Device 제거(주석으로 이유와
  대체 커버리지 명시). `menus_pause_the_audio_and_a_running_game_does_not`에 Device 화면 1줄 추가.
  기존 테스트 약화·재작성은 없다.

## 생성·수정 파일

- 생성: `crates/slot2/tests/device_menu_app.rs`(13 tests), `tasks/63-device-menu-app-wiring.worker-result.md`.
- 수정: `crates/slot2/src/app.rs`(module 상태 전이 doc, Screen variant, import, audio_paused,
  전역 VolUp/VolDown, InGame Device arm, `Screen::Device` arm, `open_device_menu`/`device_menu`/
  `refresh_device_menu`/`change_volume`, draw 3곳, 관련 unit test 2개),
  `crates/slot2-ui/src/device_menu.rs`(`Copy` derive + doc 한 줄).
- 그 밖의 파일은 건드리지 않았다. 최종 검증 뒤 코드 변경 없음. 커밋·푸시·네트워크·실기·공용 설정
  변경·위임 없음.

## 계약 의견 / 남은 위험

- 계약은 "Device 행이 세션 없이도 열리는지"를 명시하지 않는다. 볼륨이 프론트엔드 소유이고 draw 계약이
  session 없는 overlay를 언급하므로 **연다**로 구현하고 기존 unit test를 그에 맞게 고쳤다. 만약 판정이
  "세션 없으면 행에 머문다"라면 `open_device_menu`에 session 가드 1줄과 그 테스트 되돌리기가 필요하다.

## 소요 시간

약 22분(20:14–20:36 KST). 누적 호출 1/2.
