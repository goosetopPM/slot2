# Task 86 — 작업자 결과

- **누적 호출 횟수:** 1/2
- **결과:** 성공 (완료 기준 6개 명령 모두 종료 0)
- **소요 시간:** 약 20분 (18:40 → 19:00 KST, 최종 검증 순차 실행 포함)

## 구현 요약

- `App`에 선반 설정 가용성을 한 곳에서 만드는 private helper `App::shelf_availability()`를 두고
  `time_zone: true, about: true, ..Default::default()`만 반환한다. `open_shelf_menu`가 이 값을 쓰며
  가용성 literal을 다른 production 위치에 복제하지 않았다. `slot2-ui::ShelfAvailability`는 수정하지
  않았다(이 빌드 전용 helper 추가 없음).
- `Screen::About(ShelfMenu, AboutSticker)`를 추가했다(`Clone + Copy + Debug + PartialEq + Eq` 유지).
- Shelf A 분기를 선택 choice로 명시적으로 나눴다: `Some(TimeZone)` → 기존 `open_timezone_menu`,
  `Some(About)` → 새 `open_about`(`Screen::About(parent, AboutSticker::new())`), 그 밖/None → no-op.
  비활성 행 활성화나 refusal/toast는 만들지 않았다.
- About 입력은 `Action::Tap(Button::B | Button::Menu) → Screen::Shelf(parent)` 하나뿐이고 나머지
  버튼·action은 상위 match의 catch-all로 no-op이다. About는 `audio_paused`의 game-menu 집합에 넣지
  않았다.
- draw: `Screen::List | Shelf(_) | Timezone(..) | About(..)`가 같은 wallpaper + shelf를
  `draw_overlay_nothing`과 함께 한 번 그리고, 그 위에 `AboutSticker`만 그린다(parent Shelf/Timezone
  panel을 겹쳐 그리지 않음). 정보는 draw 시점에 `AboutInfo { version: env!("CARGO_PKG_VERSION"),
  target: ctx.profile.target }`으로 전달하며 App struct/Screen payload에 문자열을 저장하지 않는다.
  hold bar·toast는 sticker보다 뒤(위)에 그려지고 HUD 표시 조건에 About을 추가했다.
- 기존 Screen match는 전부 컴파일 그대로 통과했고 insert/eject, session, volume, rescan, power,
  game menu, 시간대 진입/preview/apply/cancel/rollback 동작은 손대지 않았다.
- 신규 `crates/slot2/tests/about_sticker_app.rs`(`#[test]` 1개, core/ROM/GPU 불필요).
- `crates/slot2/tests/timezone_menu_app.rs`는 새 가용성 때문에 직접 깨지는 1번 섹션만 갱신했다
  (정확한 `timezone_only()` 기대 → TimeZone+About, “유일한 행” 단언 → 두 활성 행 양방향 순환).
  나머지 11개 시나리오와 단일 `#[test]`, preview/save/rollback 단언은 그대로다.

## 검증 항목별 결과 (신규 통합 test)

1. List → Menu 탭이 연 Shelf는 TimeZone과 About만 `is_available`이고 나머지 네 행은 false, 최초
   선택은 TimeZone. `Screen::Shelf(ShelfMenu::new(TimeZone+About 가용성))`과 정확히 일치.
2. Up/Down이 TimeZone ↔ About을 양방향 순환하고(Up으로 About, 다시 Up으로 TimeZone, Down도 동일),
   선택된 행은 항상 `is_available`이다.
3. About 행에서 A가 `Screen::About(parent, AboutSticker::new())`를 열고 parent는 About 행을 유지.
4. B와 Menu 탭 각각이 같은 About 행의 parent Shelf로 복귀하고 다시 A로 열 수 있다.
5. About에서 A/Up/Down/Left/Right 각각이 screen·clock offset·`App::volume`·카드 bytes·toast를
   바꾸지 않는다.
6. `RecordingCanvas`에서 About 프레임의 첫 mark들이 List 프레임의 shelf 바탕 mark와 동일(upload
   제외 비교), full-panel dim 1개(첫 mark 아님), About panel(440×260) 1개, Shelf panel(440×338) 0개,
   Timezone panel(440×244) 0개, `Op::Clear` 없음.
7. draw가 실제 값을 전달함을 face cache의 키(정확한 문자열)로 확인: `about-version`을
   `env!("CARGO_PKG_VERSION")`으로 렌더한 문장은 이미 캐시에 있어 재요청 시 항목이 늘지 않고 그
   텍스처가 프레임에 있다. 같은 검사를 `"9.9.9"`, `"rg-some-other"`, 다른 target(`rg35xxsp`)에
   대해서는 캐시가 늘어나므로, placeholder나 UI crate version으로는 통과하지 않는다. target 검사는
   App 생성과 다른 profile(`ctx = rgcubexx`)로 draw해 `ctx.profile.target`이 쓰임을 구분해 확인했다.
8. HUD quad 수가 List/Shelf/About에서 동일하고 0이 아니며, About 화면에서 `audio_paused() == false`.

## 기존 timezone_menu_app 보존

- 12개 시나리오, 단일 `#[test]`, preview/cancel/unchanged/changed apply/invalid UTF-8 rollback,
  카드 bytes·volume 보존, draw layer/HUD 단언을 모두 유지했다. 1번 섹션의 가용성 기대값과 “유일한
  행” 문구만 두 활성 행 계약으로 바꿨고, 실행 결과 1 passed / 0 failed로 통과한다.

## 완료 기준 명령 (마지막 코드 변경 뒤 순서대로)

| 명령 | 종료 코드 | 마지막 결과 줄 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | (출력 없음) |
| `cargo test -p slot2 --test about_sticker_app` | 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s` |
| `cargo test -p slot2 --test timezone_menu_app` | 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s` |
| `cargo test -p slot2` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (Doc-tests slot2, 최종 줄) |
| `cargo check -p slot2 --no-default-features --features device` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 1.68s` |
| `cargo clippy -p slot2 --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 6.62s` |

- `about_sticker_app`: **1 passed / 0 failed / 0 ignored**, `timezone_menu_app`: **1 passed / 0 failed /
  0 ignored**.
- `cargo test -p slot2`: **315 passed / 0 failed / 0 ignored** (lib 33, main bin 0, integration 282,
  doc-tests 0; 29개 result 줄). Task 84의 314에 신규 test 1개가 더해진 값과 일치한다.
- core-dependent skip: **0** (실행 출력에 `skipping` 없음). 신규 test는 core/ROM/GL이 필요 없다.
- clippy 지적 1건(`clippy::collapsible_match`)은 allow 없이 구조를 바꿔 해결했다(About 입력을
  `Action::Tap(Button::B | Button::Menu)` 단일 arm으로 두고 나머지는 상위 catch-all이 처리).
- workspace test와 device 배포는 실행하지 않았다.

## 변경 파일

- 수정: `crates/slot2/src/app.rs` (Screen::About, `shelf_availability`, `open_about`, Shelf A 분기,
  About 입력 arm, draw base/About layer/HUD 조건, 모듈 doc 화면 map)
- 신규: `crates/slot2/tests/about_sticker_app.rs`
- 수정: `crates/slot2/tests/timezone_menu_app.rs` (1번 섹션 가용성/순환 단언·문구만)
- 신규: `tasks/86-about-sticker-app-wiring.worker-result.md`

`slot2-ui`, i18n/FTL, store/platform, host/device loop, Cargo 파일은 수정하지 않았다. 최종 검증 6개
명령 실행 뒤 코드 변경 없음.

## 계약이 틀려 보이는 부분 / 남은 위험

- 계약 오류는 없었다. 다만 테스트 7번의 “UI crate version으로는 통과하지 않아야 한다”는 현재
  workspace에서 `slot2`와 `slot2-ui`의 package version이 둘 다 0.1.0(`version.workspace = true`)이라
  문자열로는 구분할 수 없다. 그래서 단언은 “draw된 문장이 `env!("CARGO_PKG_VERSION")`을 렌더한
  문장과 정확히 같다”로 세웠고, 이는 값이 다르면(placeholder 포함) 반드시 실패한다. 실제 값의 출처가
  `slot2` crate임은 production 코드가 `env!("CARGO_PKG_VERSION")`을 쓰는 것으로 봉인된다.
- About 화면은 정적이고 조작이 없으므로, 실기에서의 글자 크기·`System/licenses` 경로 가독성 확인은
  사용자 몫으로 남는다.
- Language/DisplayDefaults/BootLogo/Sync 네 행은 여전히 비활성이며, 각 화면이 생기면
  `App::shelf_availability` 한 곳만 바꾸면 된다.
