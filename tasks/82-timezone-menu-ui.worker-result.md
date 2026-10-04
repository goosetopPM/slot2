# Task 82 — 작업자 결과 (호출 1/2)

- **결과:** 성공
- **누적 작업자 호출:** 1/2 (실패·중단 없음)
- **소요 시간:** 약 37분 (01:57 → 02:34 KST, 최종 명령 순차 실행 포함)

## 구현 요약

- 신규 `crates/slot2-ui/src/timezone_menu.rs`: `TimezoneMenu` model + `draw` + `format_offset` +
  번역 key 상수. `crates/slot2-ui/src/lib.rs`에 `pub mod timezone_menu;`와
  `pub use timezone_menu::TimezoneMenu;` 두 줄만 추가.
- `assets/lang/en.ftl`, `ko.ftl`에 `timezone-title/value/note/hint-adjust/hint-apply/hint-cancel`
  여섯 key를 명세 표의 문구 그대로 추가. 기존 key·문구는 변경하지 않음.
- `crates/slot2-i18n/tests/i18n.rs`에 신규 key 직접 정의 테스트 1개만 추가.
- 신규 `crates/slot2-ui/tests/timezone_menu.rs` (11 test).
- backend, store read/write, `slot2_platform::clock`, App/Screen/input, host/device loop은 읽기만
  했고 수정하지 않았다. 새 dependency, cache counter, input handler도 추가하지 않았다.

## model 계약

- `TimezoneMenu`는 `Clone + Copy + Debug + PartialEq + Eq`, private `original: i32`, `selected: i32`
  뿐이다. `new()`는 original/selected를 같은 값으로 시작하고, `changed()`는 두 값의 비교다.
- 범위는 store에서 그대로 가져왔다(`slot2-ui`는 이미 `slot2-store`에 의존). `MIN_MINUTES = -720`,
  `MAX_MINUTES = 840`, `DEFAULT_MINUTES = 0`이 각각
  `UTC_OFFSET_MINUTES_MIN/MAX/DEFAULT_UTC_OFFSET_MINUTES`와 같음을 test가 단언한다. UI가 범위를
  복사하지 않으므로 두 값이 어긋날 수 없다.
- constructor는 `541` 같은 hand-edited 값을 그대로 보존하고 quarter-hour로 맞추지 않는다. 범위 밖
  입력은 가까운 경계로 clamp하며 error 타입이나 App fallback을 만들지 않았다(`i32::MIN/MAX` 포함).
- `left/right` = ∓15분, `up/down` = ±60분. `up`이 큰 값(= 동쪽)이다. 모든 조정은 범위 안에서
  clamp되고 wrap하지 않는다. original은 navigation으로 변하지 않는다.
- `format_offset`은 입력을 같은 범위로 clamp해 `+00:00`, `+00:01`, `-00:01`, `+00:15`, `+09:00`,
  `-08:00`, `+05:45`, `-12:00`, `+14:00`을 만든다. 길이는 항상 6자이고 `UTC` 글자를 포함하지 않는다.
  0도 부호를 남긴다. `format_offset`만이 값을 문자열로 만들고, 화면에는 그 문자열이
  `timezone-value`의 `offset` argument로 들어간다.

## draw와 layout

- `Canvas::clear`를 호출하지 않는다. 첫 op는 physical panel 전체 dim(`BLACK 0.6`), 다음이 safe area에
  중앙 배치된 `BACKDROP` panel(440×244)이며 그 뒤 title → 현재 값 → note → 조정 hint →
  적용/취소 hint 순서다.
- 상수: `BOX_W = 440`, `BOX_H = 244`, `PAD = 16`. title/note는 `PX_BODY`, 현재 값은 `PX_TITLE`,
  hints는 `PX_HINT`. 현재 값은 `INK`, 나머지는 `INK_DIM`.
- `original != selected`를 색·별표·별도 문구로 표시하지 않는다. 값이 움직인 프레임과 열었을 때의
  프레임은 drawing op 종류 순서가 완전히 동일함을 test가 확인한다.
- 세 geometry 모두 panel·모든 text/button quad가 safe area 안에 있고, box는 panel 중심이 아니라
  safe area 중심(safe px/py 기준)에 놓인다. box와 각 문구는 panel 내부 폭(408px)에도 들어간다 —
  한국어 note가 440 panel 안에 들어가는 것을 실제 폰트 측정으로 확인했다.

## 문구와 warm draw

- 영문/한글 모두 여섯 key를 직접 정의한다(i18n test가 정확한 문구를, UI test가 fallback marker
  부재와 5개 key의 번역 차이를 확인). `timezone-value`는 두 pack이 같은 문구를 쓴다.
- FTL에 적은 대로 `timezone-hint-adjust`의 두 칸 공백이 렌더 결과에도 유지된다(`[←][→] 15 min␣␣[↑][↓] 1 hour`,
  한국어 `[←][→] 15분␣␣[↑][↓] 1시간`). 조정 hint는 `left/right/up/down` 네 버튼을 순서대로 만든다.
- warm 재드로: 첫 프레임만 upload가 있고 같은 값 8회 추가 draw는 upload 0. 새 값으로 이동하면 그
  문자열의 face 1개가 upload되고, 이미 그린 값으로 돌아오거나 다시 이동하면 upload 0(한국어 CJK
  face 포함). hint와 note는 값이 바뀌어도 재업로드되지 않는다.

## backend/store/clock/App 미접촉 확인

- 수정 파일은 아래 6개뿐이다. `slot2-store`, `slot2-platform`, `slot2/src/*`, host/device loop,
  `main.rs`, 다른 메뉴 파일은 diff가 없다(이번 작업에서 mtime이 바뀐 source는 신규/수정 6개 파일이
  전부). 카드 read/write, clock setter, 환경변수, 파일 시스템 접근 코드는 새로 만들지 않았고 test도
  RecordingCanvas와 in-memory pack만 쓴다.
- UI는 store의 범위 상수만 읽고 store API를 바꾸지 않았다.

## 완료 기준 명령 (마지막 코드 변경 뒤 순서대로)

| 명령 | 종료 코드 | 마지막 결과 줄 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | (출력 없음) |
| `cargo test -p slot2-ui --test timezone_menu` | 0 | `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.77s` |
| `cargo test -p slot2-ui -p slot2-i18n` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (Doc-tests slot2_ui, 최종 줄) |
| `cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 1.62s` |

- `timezone_menu`: **11 passed / 0 failed / 0 ignored**.
- `cargo test -p slot2-ui -p slot2-i18n`: **260 passed / 0 failed / 0 ignored**
  (slot2-ui lib 1 + integration 227, slot2-i18n i18n 32, doctests 0).
- 중간에 실패한 두 test(`a_moved_value_...`, `a_warm_menu_...`)는 test 코드 자체의 잘못된 기대
  (face cache를 warm한 upload를 프레임 비교에 포함, 아직 그린 적 없는 값에 upload 0을 기대)였고
  production 코드는 바뀌지 않았다. clippy 1건(`clippy::expect_fun_call`)도 test 코드에서 수정했다.
- workspace test, 실제 GL 창, device 배포는 실행하지 않았다.

## 생성·수정 파일

- 신규: `crates/slot2-ui/src/timezone_menu.rs`, `crates/slot2-ui/tests/timezone_menu.rs`,
  `tasks/82-timezone-menu-ui.worker-result.md`
- 수정: `crates/slot2-ui/src/lib.rs`(module/re-export 두 줄), `assets/lang/en.ftl`, `assets/lang/ko.ftl`,
  `crates/slot2-i18n/tests/i18n.rs`(신규 key 테스트 1개 추가)
- 최종 검증 4개 명령 실행 뒤 코드 변경 없음. `cargo fmt --all`이 재포맷한 파일은 이번에 건드린
  Rust 파일뿐이었다(check가 사전에 그 파일들만 diff로 보고).

## 후속

- `TimezoneMenu`를 선반/shelf menu에서 여는 경로와 Screen variant·input 배선은 아직 없다.
- runtime clock preview, 카드 read/write, A 적용과 B 취소, 저장 실패 시 `original` rollback과 toast는
  Task83 범위다. 이 컴포넌트는 `selected`/`original`만 제공하고 clock이나 파일을 건드리지 않는다.

## 계약 의문·남은 위험

없음. 다만 `timezone-value`가 두 pack에서 같은 문구라 "번역 차이"로는 검증할 수 없어, 해당 key는
fallback marker 부재와 정확한 문구로만 확인했다.
