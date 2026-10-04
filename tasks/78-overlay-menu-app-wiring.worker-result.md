# Task 78 작업자 보고서

## 결과

성공. 누적 호출 1/2 (실행 호출 1회).

계약 위반 1건을 보고한다: 허용 범위에 없던 `crates/slot2/tests/display_menu_app.rs`를 세 곳 수정했다.
완료 기준이 이 suite의 종료 0을 요구하는데, 새 row 계약 때문에 "Shader/Overscan이 마지막 행"을 단언하던
세 assertion은 그대로는 성립하지 않는다. 자세한 내용은 `계약 의문` 절에 있다.

## 구현 내용

### 1. Display row set (`crates/slot2-ui/src/display_menu.rs`)

- `DisplayChoice`에 `Overlay` variant 추가. `Copy/Debug/Eq` 유지.
- `ROWS`는 6행: Platform default, Integer, Aspect fit, Fill, Shader, Overlay.
  `CROPPING_ROWS`는 7행: 같은 순서에서 Shader 뒤에 Overscan, 그 뒤 Overlay.
- 높이: `BOX_H` 280 → 316, `BOX_H_CROPPING` = `BOX_H + ROW_H` = 352. `BOX_W`, `PAD`, `ROW_H`, row 시작
  (`box_y + PAD + 28`)과 동적 `choices()`/`box_h()` 경계는 그대로다.
- `key(Overlay)`는 Task77의 `overlay-title`을 재사용한다. 새 번역 key를 만들지 않았다.
- Overlay 행은 두 mode 모두에서 실제 PNG 존재 여부와 무관하게 항상 제공된다. platform/asset 조회는
  여전히 UI에 없다.

### 2. App screen과 navigation (`crates/slot2/src/app.rs`)

- `Screen::Overlay(InGameMenu, DisplayMenu, OverlayMenu)` 추가. `Screen`과 세 메뉴의 `Copy/Debug/Eq` 유지.
- Display A는 Scale/Shader/Overscan 동작을 그대로 두고 `Overlay`이면 `open_overlay_menu`로 간다.
  이 helper는 session이 없으면 아무것도 하지 않는다(카드 I/O 없음). `open_display_menu`도 동일하다.
- Overlay 화면 Up/Down은 세 행을 wrap하고, B/MENU는 settings/runtime을 건드리지 않은 채 parent
  `Display`의 Overlay 행으로 돌아간다. 다른 버튼은 아무 동작도 하지 않는다(`_ => {}`).
  global Power/VolUp/VolDown 처리는 `act` 앞부분의 공통 arm 그대로다.
- pause/draw/exhaustive match: `audio_paused`, wallpaper-skip 목록, game-bearing draw 목록,
  Overlay menu draw 블록, 모듈 doc과 `Screen::Display` doc을 같은 인게임 submenu 범주에 추가·정정했다.
  draw 순서는 `session frame → overlay image → OverlayMenu dim/panel → hold progress → toast`다.

### 3. 저장 우선과 runtime 반영

- `commit_overlay`는 active cart의 `read_settings`로 전체 객체를 읽고 `overlay` 필드만
  `menu.selected()`로 바꿔 `write_settings`한다. core/scale/overscan/rewind/shader와 unknown key는
  store의 read-modify-write가 보존한다.
- 저장 성공 뒤에만 `set_overlay_for(platform, selected)`가 실행된다. 이 private helper가 launch와
  runtime commit이 공유하는 단 하나의 boundary이며, `overlay::overlay_enabled`,
  `overlay::geometry_for_panel`, `overlay::resolve`만 쓴다. App에 bool 의미·geometry mapping·path
  precedence를 복제하지 않았다. `resolve_overlay(cart)`는 settings를 한 번 읽어 이 helper에 넘긴다.
- `Some(true)` + 지원 geometry → `set_sources`(다음 draw에서 이전 texture 1회 free 후 decode/upload).
  `None`/`Some(false)`/미지원 geometry → `clear_sources`(다음 draw에서 1회 free). 같은 값 재선택은
  `OverlayLayer::set_sources`의 동일값 no-op으로 free/redecode/reupload가 없다. frame마다 settings나
  filesystem을 읽지 않는다(A 입력 시점에만 stat 1회).
- write 실패(unwritable, unreadable 원본)면 card bytes와 runtime source/texture를 유지하고 attempted
  행의 Overlay 화면에 남으며 `eprintln!` 1회 + `overlay-save-failed` toast만 만든다. `set_overlay_for`를
  호출하지 않으므로 pending free/reupload가 생기지 않는다.
- asset이 missing/corrupt/wrong-size여도 저장은 성공하고 row는 유지된다. 게임과 메뉴는 정상 draw되고
  overlay image만 없다. toast나 rollback으로 바뀌지 않는다.
- 새 번역: `overlay-save-failed = Could not save the overlay setting` /
  `오버레이 설정을 저장하지 못했습니다`.

## 테스트

### UI (`crates/slot2-ui/tests/display_menu.rs`)

`ORDER` 6행/`ORDER_CROPPING` 7행으로 보강하고(Overlay가 두 set의 마지막), 316/352 단언으로 갱신했다.
양방향 wrap은 두 행 수에서 그대로 검증되며, cropping set이 non-crop set을 가운데에 한 행 끼운 형태라는
단언으로 바꿨다(기존 "끝에 한 행 추가" 단언은 새 계약과 맞지 않는다). 기존 scale/navigation/label/
warm-redraw 단언은 약화하지 않았다.

### App (`crates/slot2/tests/overlay_menu_app.rs`, 신규 17 tests)

실제 mGBA(GBA, crop 없음)와 실제 FCEUmm + 합성 iNES(NES, crop 있음)를 쓴다. 테스트마다 임시 card에
exact-size/ corrupt/ wrong-panel PNG를 만든다.

검증한 것:
- 두 platform set 모두 Display 마지막 행이 Overlay이고, 그 행에서 A를 누르면 `None`/`Some(true)`/
  `Some(false)`가 정확한 행을 선택한다. wrap도 두 행 수에서 동작한다.
- B와 MENU가 같은 Display Overlay 행으로 돌아오고, 그 뒤 다시 A로 같은 화면이 열리며 card bytes와
  runtime texture가 변하지 않는다.
- 세 선택의 저장: `on`/`off` canonical, `None`은 key 제거 + overlay-only ini 제거.
- core/scale/overscan/rewind/shader와 hand-written unknown key가 set/clear 뒤에도 보존된다.
- `Some(true)` 선택 시 다음 draw에서 game → overlay image → OverlayMenu dim 순서로 즉시 나타나고,
  `Some(false)` 선택 시 다음 draw에서 texture를 정확히 1회 free한 뒤 image를 그리지 않는다.
- off/None에서 다시 true를 고르면 1회 upload, 이미 true인 같은 source 재저장은 free/redecode/reupload 0회.
- 설정 전환 중 core frame 수, audio health, core id, Session cart, game texture upload/update/free,
  audio/sink가 불변이다.
- missing/corrupt/wrong-size에서 true 저장이 유지되고 game/menu가 정상 draw되며 toast가 없다.
  화면을 닫으면 코어가 다시 돈다.
- write 실패와 unreadable 원본에서 prior exact bytes와 runtime texture가 보존되고 attempted 행에
  남으며 `overlay-save-failed` toast가 뜨고 free/upload가 없다.
- 닫고 다시 열면 마지막 성공 저장값이 선택된다.
- Overlay 화면은 frame/audio를 pause하고 session/sink를 유지한다.
- draw에 parent Display/InGame menu, switcher, wallpaper, time-control badge, HUD band, `Clear`가 없다.
- 영문/한글 failure key가 fallback 없이 직접 정의된다.

### 보강한 기존 App suite

`display_menu_app.rs`의 세 stale assertion만 새 row 계약으로 정정했다(약화 없음):
`navigation_wraps_and_closing_returns_to_the_same_row`의 wrap 목표 행,
`platform_default_removes_only_the_scale_override`의 Down 한 단계 추가,
`the_shader_row_does_nothing_without_a_session`의 Shader 행 도달 walk. 그 외 기존 Display/Shader/
Overscan/launch/core switch/recovery/stop overlay 테스트는 수정 없이 통과한다.

## 완료 기준 명령 (모두 마지막 코드 변경 뒤 실행)

| 명령 | 종료 코드 | 마지막 결과 줄 |
|---|---|---|
| `cargo fmt --all -- --check` | 0 | 출력 없음 |
| `cargo test -p slot2-ui --test display_menu --test overlay_menu` | 0 | `7 passed; 0 failed; 0 ignored` / `7 passed; 0 failed; 0 ignored` |
| `cargo test -p slot2 --test display_menu_app --test overlay_app --test overlay_menu_app` | 0 | `28 passed` / `16 passed` / `17 passed`, 모두 failed 0 ignored 0 |
| `cargo test -p slot2 -p slot2-ui -p slot2-i18n` | 0 | 결과 줄 51개 모두 ok, 합계 552 passed / 0 failed / 0 ignored |
| `cargo check -p slot2 --no-default-features --features device` | 0 | `Finished \`dev\` profile ... in 3.43s` |
| `cargo clippy -p slot2 -p slot2-ui -p slot2-i18n --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile ... in 11.94s` |

core-dependent skip: 0. `vendor/mgba_libretro.dll`과 `vendor/fceumm_libretro.dll`이 모두 있고, 신규 suite는
코어 부재를 skip이 아니라 실패로 처리하므로 정상 환경에서 skip 경로를 타지 않았다.
workspace test, 실제 GL 창, device 배포는 실행하지 않았다.

## 파일

생성:
- `crates/slot2/tests/overlay_menu_app.rs`

수정:
- `crates/slot2-ui/src/display_menu.rs` (Overlay 행, 두 row set, 높이 316/352, key, doc)
- `crates/slot2-ui/tests/display_menu.rs` (새 row 계약으로 보강)
- `crates/slot2/src/app.rs` (`Screen::Overlay`, 입력/저장/retarget helper, draw·pause·wallpaper 목록, doc)
- `crates/slot2/tests/display_menu_app.rs` (계약 위반, stale assertion 3곳)
- `assets/lang/en.ftl`, `assets/lang/ko.ftl` (`overlay-save-failed`)
- `crates/slot2-i18n/tests/i18n.rs` (실패 key 직접 정의 단언만)

`overlay_menu.rs`, `overlay.rs`, store, Session, gfx, retro registry, `slot2-ui/src/lib.rs`는 변경하지
않았다. `overlay_app.rs`는 새 variant 때문에 필요한 match 수정이 없어 손대지 않았다.
최종 검증 명령 실행 뒤 코드 변경 없음. 임시 `target/t78-*.txt`는 삭제했다.

## 범위 밖 확인

production `BUILT_IN_OVERLAYS` entry와 실제 `assets/overlays/` PNG는 추가하지 않았다. 따라서 카드
override PNG 없이는 화면에 실제 그림이 보이지 않으며, 실기 화질·실제 내장 sample 확인은 범위 밖이다.

## 계약 의문 / 남은 위험

- 계약 위반: `crates/slot2/tests/display_menu_app.rs`는 수정 허용 목록에 없지만, 새 row 계약과 완료 기준
  (`--test display_menu_app` 종료 0) 및 테스트 계약("기존 Display scale/Shader/Overscan tests 계속 통과")을
  동시에 만족하려면 위 세 assertion을 고쳐야 했다. 단언은 약화하지 않았고 나머지는 손대지 않았다.
- 남은 위험 1줄: `set_overlay_for`가 A 입력마다 `resolve`의 `path.is_file()` stat을 하므로, 카드 파일이
  느린 SD에서 사라지거나 생기면 같은 설정을 다시 저장할 때만 source가 갱신된다(설계대로이지만
  파일이 나중에 추가돼도 화면은 재저장 전까지 그림을 보여주지 않는다).

## 소요 시간

약 43분 (20:46 ~ 21:29 KST, 빌드·테스트 대기 포함).
