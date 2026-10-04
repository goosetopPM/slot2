# Task 83 — 작업자 결과

- **누적 호출 횟수:** 1/2
- **결과:** 성공 (검증 4개 명령 모두 종료 0)
- **소요 시간:** 약 20분 (07:46 → 08:06, 최종 검증 순차 실행 포함)

## 구현 요약

- 신규 `crates/slot2-ui/src/shelf_menu.rs`: `ShelfChoice`, `SHELF_CHOICES`, `ShelfAvailability`,
  `ShelfMenu`와 draw/layout. `lib.rs`에 module + re-export(`ShelfAvailability, ShelfChoice,
  ShelfMenu, SHELF_CHOICES`) 한 블록 추가.
- 행 순서는 `SHELF_CHOICES` 한 곳에만 있다(계약 순서 그대로). 여섯 행을 항상 같은 순서로 그리고,
  열 수 있는 행만 강조·선택되며 나머지는 흐린 색 + `device-unavailable` 재사용 문구로 표시된다.
  시간대 행은 `timezone_menu::TITLE_KEY`를 그대로 쓴다.
- 선택: 생성 시 첫 열 수 있는 행, 위/아래는 열 수 없는 행을 건너뛰고 양방향 순환, 열 수 있는 행이
  하나면 그대로 유지, 하나도 없으면 `None` + 안전한 no-op.
- draw: clear 없음 → `canvas.size()` 전체 dim → safe area 중앙 panel(440×338) → 제목(PX_BODY,
  INK_DIM) → 6행(PX_TITLE, 열 수 있으면 INK·없으면 INK_DIM, 선택 행 1개만 INK alpha 0.15 강조,
  비활성 행 오른쪽에 PX_BODY INK_DIM으로 사용 불가 문구) → hint line(PX_HINT, INK_DIM;
  선택 가능 행이 있을 때만 `hint-select`, `hint-back`은 항상).
- 영문·한글 pack에 여섯 key를 표의 문구 그대로 직접 추가. 기존 key/문구는 변경하지 않음.
- `crates/slot2-i18n/tests/i18n.rs`에 신규 key 직접 정의·정확 문구·중복 없음 검증 1개 추가.
- 신규 `crates/slot2-ui/tests/shelf_menu.rs` (9 test). App/Screen/input, store, platform, backend는
  건드리지 않았고 새 dependency·cache counter·입력 handler도 추가하지 않았다.

## 변경 파일

- 신규: `crates/slot2-ui/src/shelf_menu.rs`, `crates/slot2-ui/tests/shelf_menu.rs`
- 수정: `crates/slot2-ui/src/lib.rs` (module/re-export), `crates/slot2-i18n/tests/i18n.rs` (테스트 1개)
- 수정(계약 경로 정정, 아래 참조): `assets/lang/en.ftl`, `assets/lang/ko.ftl`
- 신규: `tasks/83-shelf-menu-ui.worker-result.md`

## 검증 명령 (마지막 코드 변경 뒤 순서대로)

| 명령 | 종료 코드 | 마지막 결과 줄 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | (출력 없음) |
| `cargo test -p slot2-ui --test shelf_menu` | 0 | `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.90s` |
| `cargo test -p slot2-ui -p slot2-i18n` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (Doc-tests slot2_ui, 최종 줄) |
| `cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 17.72s` |

### 테스트 수

- `shelf_menu`: **9 passed / 0 failed / 0 ignored**
- `cargo test -p slot2-ui -p slot2-i18n`: **270 passed / 0 failed / 0 ignored**
  (slot2-ui lib 1 + integration 236, slot2-i18n i18n.rs 33, doctests 0)
- workspace 전체 테스트와 배포 이미지 생성은 실행하지 않았다.

## 계약이 잘못됐거나 모호한 부분 (모두 보고 대상)

1. **`draw` 시그니처가 이 크레이트에서 구현 불가.** 계약의
   `pub fn draw(&self, canvas: &mut impl Canvas, i18n: &I18n)`로는 텍스트를 한 글자도 그릴 수 없다.
   텍스트는 `UiCtx`의 `FontChain` + `FaceCache`를 통해 rasterize되고(`face::face`/`draw_spans`),
   `slot2_i18n::I18n`에는 폰트도 safe area도 없다. 또한 `Canvas` trait에는 `safe()`가 아예 없다
   (`slot2-gfx` 확인). 게다가 계약 스스로 "기존 `PowerMenu`, `DeviceMenu`, `TimezoneMenu`의 스타일을
   따른다"고 하며, 그 셋은 모두 `draw(&self, canvas: &mut dyn Canvas, ctx: &mut UiCtx)`다.
   → **구현: `pub fn draw(&self, canvas: &mut dyn Canvas, ctx: &mut UiCtx)`** (다른 메뉴와 같은 형태).
   그 외 공개 API(`new`/`selected`/`is_available`/`up`/`down`)와 타입 이름·derive는 계약 그대로다.
2. **번역 파일 경로가 존재하지 않음.** 계약의 `crates/slot2-i18n/locales/en/main.ftl`,
   `.../ko/main.ftl`은 이 저장소에 없다(디렉터리 자체가 없음). 실제 built-in pack은
   `I18n::EMBEDDED`가 `include_str!`로 묶는 `assets/lang/en.ftl`, `assets/lang/ko.ftl`이며 Task 82도
   같은 파일을 썼다. → **허용 목록 밖이지만 기능상 같은 파일이므로 이 두 파일에만 여섯 key를
   추가했다**(그 외 변경 없음).
3. **1280×720은 이 플랫폼에 없는 해상도.** `slot2_platform::Geometry`는 640×480, 720×480, 720×720
   세 가지뿐이다(`crates/slot2-platform/src/profile.rs`). → 세 번째 케이스를 **합성 패널**로 검증했다:
   `SafeArea { x: 320, y: 120, panel_w: 1280, panel_h: 720 }`(모든 geometry가 safe area를 중앙에
   두는 규칙 그대로)와 1280×720 `RecordingCanvas`. 이를 위해 `draw`의 dim은 `ctx.profile.geometry`
   대신 **`canvas.size()`**를 쓴다(같은 패널이면 같은 값이고, 캔버스가 더 넓을 때 dim이 잘리지 않는다).
   이 두 가지(합성 패널, dim 기준)가 위 1번과 함께 계약 대비 의도적 최소 편차다.
4. 모호했던 점: 계약 §4의 "선택 가능한 행이 하나 이상일 때만 `hint-select`를 표시"는 "열 수 있는 행"
   기준으로 해석했다(선택 강조와 같은 조건). 사용 가능 행이 0개일 때 `selected()`는 `None`이라는 §3과
   일관된다.

## 남은 위험과 다음 태스크로 넘길 것

- 선반에서 이 메뉴를 여는 key/Screen variant·input 배선은 아직 없다. `ShelfMenu`를 `App::Screen`에
  넣고 열고 닫는 경로, 그리고 `TimezoneMenu`로 들어가는 하위 경로가 Task 84 대상이다.
- 시간대 행을 실제로 열었을 때의 runtime 미리보기, A 적용과 safe-write, B 취소, 저장 실패 시
  `original` rollback과 toast는 여전히 미구현이다(이 태스크 범위 밖).
- `ShelfAvailability::timezone_only()`가 현재 유일한 실사용 값이다. 언어/화면 기본값/부팅 로고/연동/
  정보 화면이 생기면 각 생성 지점에서 해당 필드를 켜야 하고, 켜지 않은 행은 계속 흐리게 남는다.
- 1280×720 케이스는 합성 패널이라 실제 기기 검증이 아니다. 향후 그런 패널이 등록되면
  `Geometry`/`PROFILES`가 바뀌고 이 테스트는 실제 패널로 대체하는 편이 좋다.
