# Task 62 — 인게임 Device 메뉴 UI (누적 복구 결과)

- 결과: **성공**. 누적 호출 **2/2**.
- 시도 1 중단 원인은 provider 400 `read body failed`(약 2.13MB·622 messages 누적)이며, 설정·role
  문제가 아니다. 시도 1의 변경은 working tree에 그대로 남아 있었다.
- 보존한 시도 1 변경: `crates/slot2-ui/src/device_menu.rs`, `crates/slot2-ui/tests/device_menu.rs`,
  `crates/slot2-ui/src/lib.rs`(module + re-export), `assets/lang/en.ftl`·`ko.ftl`(device block),
  `crates/slot2-i18n/tests/i18n.rs`(device message 테스트). reset/revert/재구현하지 않았다.
- 이번 delta(1건): `crates/slot2-ui/tests/device_menu.rs`의 헬퍼 `drawn(...)`이 인자 8개라
  `cargo clippy … -D warnings`가 `clippy::too_many_arguments`로 실패했다. 마지막 두 인자
  `y: f32, h: f32`를 `band: (f32, f32)` 하나로 묶어 7개로 줄이고 호출부 7곳을 함께 고쳤다.
  `#[allow]`는 쓰지 않았다. 그 외 미완 코드·컴파일 오류는 없었다.

## 원 Task62 계약 결과

- 타입/상태: 공개 `device_menu` module과 `DeviceMenu`, `DeviceSetting`(정확히 `Volume`,
  `Brightness`, `BlueLight`, `Copy`)을 `slot2-ui`에서 re-export. 행 순서는 `ROWS`로 고정, 최초
  선택은 Volume. 필드는 전부 private이고 query는 `selected()`, `value()`, `volume_muted()`,
  `is_available()`뿐 — 내부 상태를 mutable reference로 노출하지 않는다.
- availability/clamp: 생성자는 `volume, muted, brightness: Option<u8>, blue_light: Option<u8>`을
  받고 100 초과를 clamp하며 `None`을 0으로 바꾸지 않는다(테스트: `new(120, false, Some(200), None)`
  → 100/100/`None`). Volume은 항상 available. `set_value`는 100으로 clamp하고 unavailable 항목은
  값을 받아도 `None`·unavailable 그대로다. `set_muted`/`toggle_muted`는 Volume에만 속하고 level을
  건드리지 않는다.
- navigation: Up/Down은 available 행만 `rem_euclid`로 순환. 둘 다 `None`이면 Volume에 고정되고
  panic하지 않는다. brightness `None`+blue light `Some`이면 Volume ↔ BlueLight로 건너뛴다.
- draw: `Canvas::clear` 없음. 맨 먼저 전체 물리 패널 dim(`DIM`, BLACK alpha 0.6), 이어 safe area
  중앙 panel(`BOX_W 340` × `BOX_H 264`, BACKDROP), 제목 `ingame-device`(PX_BODY, INK_DIM), 세 행
  `row_y = by + PAD(16) + 28 + i*ROW_H(56)`, 선택된 available 행만 INK alpha 0.15 highlight 정확히
  1개(테스트가 3행 각각 확인), 라벨 PX_TITLE 좌측·값 text PX_BODY 우측, bar는 track 전체 폭
  (`BOX_W - 2*(PAD+INSET)`) `BAR_H 6`, fill 폭 = level/100(0에서 0, 100에서 track 전체, 초과 없음).
  muted Volume은 기억된 level의 bar를 alpha 0.35로 dim 처리해 유지하고 값 text는 `Muted`/`음소거`.
  unavailable 행은 track alpha 0.07 + fill 없음 + label·값 모두 INK_DIM, `Unavailable`/`사용 불가`
  표시이며 highlight되지 않는다. hints는 `by + BOX_H - PAD - PX_HINT` 한 줄에 PX_HINT·INK_DIM,
  Volume 선택일 때만 mute hint가 붙는다.
- 세 profile × en/ko layout: 3 target(rgsp/rg35xxsp/rgcubexx) × 2 lang = 6 case 모두 첫 op이
  full-panel dim, dim을 뺀 모든 `Rect`/`Image` op이 `ctx.safe.contains` 안에 있고, box/title/각 행
  label·값·bar가 자기 위치에 실제로 그려짐을 폭 측정으로 확인. hints 폭 합계가 `BOX_W - 2*PAD` 이하.
- i18n: 두 pack에 `device-volume`, `device-brightness`, `device-blue-light`, `device-percent`
  (`{ $value }%` — 코드에서 문장을 조립하지 않고 Fluent argument로 전달), `device-muted`,
  `device-unavailable`, `hint-adjust`, `hint-mute`를 직접 정의했고, 제목 `ingame-device`와
  `hint-back`은 재사용했다. i18n 테스트가 영·한 각각의 문자열과 `device-percent`의
  0%/42%/100% 포맷을 실제로 단언한다.
- warm draw: 한국어 ctx에서 첫 프레임 뒤 같은 상태 8회 반복, down/up 3회씩 이동, mute
  toggle 3회 + level setter 적용에서 새 `UploadAlpha8`/`UploadRgba8`는 0이 아니라 **정확히 1**이다.
  그 1은 그 프레임에서 처음 등장한 level text(`100%`)의 texture다. 아래 계약 의문 참고.

## 완료 기준 명령 (원문 순서, 마지막 코드 변경 뒤 실행, 이후 코드 변경 없음)

1. `cargo fmt --all -- --check` → exit **0**, 출력 없음.
2. `cargo test -p slot2-ui -p slot2-i18n` → exit **0**. `test result:` 줄 22개, 합계 **223 passed /
   0 failed / 0 ignored**. 신규 `device_menu` suite 10 passed(`a_warm_menu_uploads_nothing_while_the_screen_says_the_same_thing`,
   `drawing_dims_then_panels_and_keeps_every_thing_in_the_safe_area` 포함), `i18n` 28 passed.
   마지막 결과 줄: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
   finished in 0.00s` (slot2_ui doc-tests).
3. `cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings` → exit **0**,
   `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.78s`.

workspace 전체 테스트와 device 배포는 실행하지 않았다.

## 생성·수정 파일

- 생성(시도 1): `crates/slot2-ui/src/device_menu.rs`, `crates/slot2-ui/tests/device_menu.rs`,
  `tasks/62-device-menu-ui.worker-result.md`(본 보고서).
- 수정(시도 1): `crates/slot2-ui/src/lib.rs`(module·re-export 2줄), `assets/lang/en.ftl`·`ko.ftl`
  (device block 8줄), `crates/slot2-i18n/tests/i18n.rs`(device message 테스트).
- 수정(이번 delta): `crates/slot2-ui/tests/device_menu.rs`의 `drawn` 헬퍼와 호출 7곳.
- App/platform/store/audio crate와 무관한 기존 미커밋 변경은 건드리지 않았다. 최종 검증 뒤 코드
  변경 없음. 커밋·푸시·네트워크·실기·공용 설정 변경·위임 없음.

## 계약 의문 / 남은 위험

- 원 계약의 warm-cache 문장은 "value가 바뀌어도 새 glyph/image texture upload가 없어야 한다"인데,
  공용 `FaceCache`가 `(문자열, px)` 단위로 캐시하므로 화면에 처음 등장하는 level 문자열(예: `100%`)은
  texture 1개를 새로 업로드한다. 테스트는 이를 명시적으로 1로 단언하고 있고, 0으로 만들려면 생성 시
  101개 level text를 미리 업로드해야 해서 더 나쁘다 — 계약 문장과 구현 해석이 갈리는 지점으로 남긴다.

## 소요 시간

약 19분(19:24–19:43 KST). 누적 호출 2/2 — 세 번째 시도는 하지 않는다.
