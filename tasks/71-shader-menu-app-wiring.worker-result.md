# Task 71 작업자 보고

## 판정

**성공. 누적 호출 1/2.** 완료 기준 5개 명령 전부 종료 0. 최종 검증 뒤 코드 변경 없음.

## 1. DisplayChoice와 화면 전이

- `slot2-ui::display_menu`에 `pub enum DisplayChoice { Scale(Option<ScaleMode>), Shader }`를 추가하고
  crate root에서 `DisplayMenu`와 함께 re-export. `ROWS: [DisplayChoice; 5]`는 정확히 Platform default,
  Integer, Aspect fit, Fill, Shader 순서이며, `DisplayMenu::new(scale)`는 그대로 해당 scale 행에서
  시작한다. 모든 행이 scale인 것처럼 보이던 `selected()`는 제거하고 `choice()`가 highlighted
  `DisplayChoice`를 돌려준다. Shader 행 label은 Task70의 `shader-title` key를 그대로 쓴다(중복 key 없음).
  `BOX_H`는 244 → 280, `BOX_W`/`PAD`/`ROW_H`/row 시작(`box_y + PAD + 28`)은 유지.
- `Screen::Shader(InGameMenu, DisplayMenu, ShaderMenu)` 추가. 세 menu가 Copy라 `Screen`의
  Copy/Debug/Eq 계약이 유지된다.
- 전이: Display의 Shader 행에서 A → active cart의 `GameSettings::shader`를 읽어 `ShaderMenu::new`로
  Shader 화면을 연다(세션 없으면 화면도 카드 I/O도 없다). Shader 화면에서 Up/Down은 여섯 행을 wrap
  이동, A는 저장 후 런타임 적용, B/MENU는 settings·runtime을 그대로 두고 **같은 Shader 행에 머문**
  Display 화면으로 돌아간다. 다른 ordinary button은 아무것도 하지 않는다.
- 집중 테스트: `None`/`Off`/explicit 세 카드 값 각각에서 Shader 화면이 그 행으로 열리고, B와 MENU
  각각으로 나간 뒤 A로 다시 들어가도 같은 행이라는 것, 카드 bytes가 나가는 길에 변하지 않는다는 것,
  세션 없는 상태에서 Shader 행이 화면을 열지 않는다는 것을 확인했다.

## 2. 여섯 의미의 저장 우선 · 즉시 적용 · 실패 rollback

- A는 항상 카드 전체를 read-modify-write하고 **shader 필드만** 바꾼다. 성공한 뒤에만 platform을
  `crate::session::retro_platform` 경계로 변환해 `Session::shader_effect_for(platform, selected)`를
  `set_shader_effect`로 적용한다(App에 mapping 사본 없음).
- 표 기반 테스트(`every_shader_choice_is_saved_exactly_and_applied_at_once`)가 GBA에서 여섯 값을 모두
  커밋하며 각각 확인한다: 카드가 그 의미 그대로(`None`은 key 제거, `Off`는 `shader = none`, 나머지 네
  값은 canonical 표기), 런타임은 `None`→`Lcd3x`(플랫폼 기본), `Off`→`None`(plain), 네 preset→동명 effect.
  성공 뒤 Shader 화면과 선택 행이 유지되고, `frames_run`·`last_frame` bytes·audio_health·texture
  upload/update/free count·sink 요청이 모두 불변이며 다음 draw만 정확한 `Image`/`ImageEffect`로 바뀐다.
  닫고 다시 열면 마지막 저장값이 선택된다.
- `None` 선택은 shader key만 지운다. shader-only ini였으면 store 계약대로 파일 자체가 제거되고 런타임은
  GBA `Lcd3x`로 돌아간다(테스트에서 Scanline을 먼저 적용해 "돌아감"이 실제 변화임을 확인).
- `Off`는 카드에 canonical `shader = none`을 남기고 런타임만 plain이 된다 — key 부재와 합치지 않는다.
- 실패: write가 실패하면 log 한 줄 + `shader-save-failed` toast, Shader 화면은 attempted 행에 그대로,
  카드 bytes와 런타임 effect는 손대지 않는다. write 불가(atomic temp 경로에 directory)와 원본이 읽히지
  않는 손상 파일 두 경우 모두 테스트했고, 후자는 store가 읽지 못한 파일을 덮어쓰지도 않는다.

## 3. 기존 동작 보존 근거

- known/unknown key: `core/scale/overscan/rewind`와 손으로 쓴 `shutdown = fast`가 shader set/clear 뒤에도
  모두 남고, clear는 `shader` key 하나만 지운다(테스트).
- scale 흐름: 기존 scale 선택·저장·실패·draw 테스트 4개를 다섯 행 계약으로만 갱신했고(단언 약화 없음,
  helper를 `choice()` 기반으로 바꾸고 Shader 행이 생긴 만큼 navigation 기대값을 명시적으로 고침), 전부
  통과한다. `commit_display_scale`도 `DisplayChoice::Scale`일 때만 쓰도록 좁혀졌다.
- pause/audio/sink: `audio_paused`에 `Screen::Shader(..)` 추가. Shader 화면에서 0.3초 동안 core frame과
  audio가 전혀 움직이지 않고 sink 요청도 없다(테스트).
- draw 순서: Shader 화면은 세션의 last frame을 **현재 effect로** 먼저 그리고 ShaderMenu overlay를
  그린다. 그 사이에 parent Display/InGame menu·wallpaper·time-control badge·switcher·clear가 없다
  (첫 mark가 game frame, ShaderMenu dim이 그 다음 전체 패널 rect, Display/InGame box·switcher card·HUD
  band 부재를 테스트로 확인). hold progress와 toast는 기존대로 ShaderMenu 뒤에 그린다.
- App module 설명에서 scale-only로 남아 있던 전이·draw 문장만 현재 동작에 맞게 고쳤고, overlay selector를
  구현했다고 쓰지 않았다. `DisplayMenu` 쪽 낡은 "shader는 아직 없다" 문장도 같은 범위에서 정정했다.

## 4. 완료 기준 명령

| 명령 | 종료 | 마지막 결과 줄 |
|---|---|---|
| `cargo fmt --all -- --check` | 0 | 출력 없음 |
| `cargo test -p slot2-ui --test display_menu` | 0 | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 18.57s` |
| `cargo test -p slot2 --test display_menu_app` | 0 | `test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 8.34s` |
| `cargo test -p slot2 -p slot2-ui -p slot2-i18n` | 0 | 마지막 target `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (46개 target 합계 **472 passed / 0 failed / 0 ignored**) |
| `cargo clippy -p slot2 -p slot2-ui -p slot2-i18n --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 31.96s` |

core-dependent skip: **0개**. vendor에 mgba가 있어 `display_menu_app`의 real-core 테스트 19개가 전부
실행됐고 어느 target에도 ignore/조기 return이 없다.

## 5. 생성·수정 파일

- 수정: `crates/slot2-ui/src/display_menu.rs`(DisplayChoice, ROWS 5행, `choice()`, BOX_H 280),
  `crates/slot2-ui/src/lib.rs`(re-export 한 줄), `crates/slot2-ui/tests/display_menu.rs`,
  `crates/slot2/src/app.rs`, `crates/slot2/tests/display_menu_app.rs`,
  `assets/lang/en.ftl`, `assets/lang/ko.ftl`(신규 `shader-save-failed`),
  `crates/slot2-i18n/tests/i18n.rs`(failure key 직접 정의 단언만)
- 생성: 이 보고서
- 최종 검증 뒤 코드 변경 없음. mtime 확인 결과 이번 호출에서 바뀐 production/test 파일은 위 목록뿐이며
  `shader_menu.rs`, Session, store, gfx, retro registry와 Cargo 파일은 무변경이다.

## 6. 범위 밖 확인

- overlay/overscan selector, shelf platform-default UI, generic settings framework, success toast,
  animation/sound, 새 gesture, Session/core restart는 건드리지 않았다.
- 실기 화질, GL 창, device 배포, workspace test, 네트워크는 실행하지 않았다.

## 7. 남은 위험 / 계약 소견

계약에서 틀린 부분은 없다. 다만 shader 선택은 이 화면을 통해 고른 뒤에만 실행 중 세션에 적용되므로,
손으로 ini를 고친 뒤 이미 실행 중인 게임은 다음 launch까지 예전 effect로 그려진다(테스트에 그 동작을
명시해 두었다) — launch 시점에 카드를 읽는 것이 SSOT라는 현재 계약과 일치한다.

## 8. 소요 시간

약 26분 (11:23 → 11:49 KST), 가재코드 호출 1회.
