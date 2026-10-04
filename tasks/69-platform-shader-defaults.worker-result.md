# Task 69 작업자 보고 (누적 2/2)

## 판정

**최종 성공. 누적 호출 2/2.** 1차에서 production 구현과 집중 테스트를 끝냈고(명세 완료 기준 6개 명령
통과), 1차 판정이 지목한 App 통합 테스트 8개 회귀를 2차에서 정리했다. 2차 완료 기준 5개 명령 모두
종료 0이며, 최종 검증 뒤 코드 변경 없음.

## 1차 (호출 1) — production 구현

- `slot2-retro`: `PlatformShader { SharpBilinear, Lcd3x, ZfastCrt, Scanline }`(Off 없음) 추가,
  `PlatformDef::shader_default`를 일곱 항목에 채움 — GB/GBC/GBA `Lcd3x`, NES/SNES/MD/SMS `ZfastCrt`.
  crate root re-export, registry 상단의 낡은 "shaders join in M2" 문장 정리. `slot2-retro` 의존은
  여전히 `libloading` 하나.
- `slot2::Session::shader_effect_for(platform, preset)` 하나로 확장: `Some(Off)` → `None`,
  네 explicit preset → 동명 gfx effect(플랫폼 기본값보다 우선), `None` → `def(platform).shader_default`.
  문자열·index·platform 이름 비교 없이 exhaustive match. `open`이 이미 계산한 platform과 이미 읽은
  settings를 넘기며 추가 I/O 없음. `start`/`start_named` 동일 경로. accessor/setter·draw routing은
  Task68 계약 유지.
- `docs/DESIGN.md` 카드 레이아웃 한 줄에서 "후일 셰이더"만 제거(휴대기/거치기 기본값 문장 불변).
- 1차 완료 기준: `cargo fmt --all -- --check` 0, `cargo test -p slot2-retro --test registry` 0
  (19 passed/0 failed), `cargo test -p slot2 --test session` 0 (41 passed/0 failed),
  `cargo test -p slot2-retro -p slot2 --lib` 0 (33 + 12 passed), `cargo check -p slot2 --tests` 0,
  `cargo clippy -p slot2-retro -p slot2 --all-targets -- -D warnings` 0. skip 0.

## 2차 (호출 2) — App 통합 테스트 8개의 game frame 식별

여덟 실패는 모두 "settings가 없는 GBA 세션의 game quad"를 `Op::Image`로만 찾던 자리다. 각 자리를
**패널 전체(720×480) + `effect == ShaderEffect::Lcd3x`** 라는 정확한 effect 단언으로 바꿨다.

| 파일 | 테스트 | 바꾼 것 |
|---|---|---|
| `cheat_menu_app.rs` | `drawing_shows_the_game_frame_under_the_cheats_menu` | `fn game_frame(&Op)` 신설: `Op::ImageEffect` + 전체 패널 + `Lcd3x`. `position(game_frame)`로 교체. 첫 mark 단언은 `Rect\|Image\|ImageEffect` 중 첫 op를 찾아 `.map(game_frame) == Some(true)` |
| `core_picker_app.rs` | `the_core_row_shows_the_platforms_cores_with_the_running_one_badged` | 첫 non-clear mark를 `Some(Op::ImageEffect { x:0,y:0,w:720,h:480, effect: Lcd3x })`로 match하고 effect까지 단언. 다른 variant는 이전처럼 panic |
| `device_menu_app.rs` | `drawing_shows_the_game_frame_under_the_device_overlay` | `game_frame` 신설(동일 조건), `position(game_frame)` + 첫 mark 단언을 `Rect\|Image\|ImageEffect` → `.map(game_frame)` |
| `display_menu_app.rs` | `drawing_shows_the_game_frame_under_the_submenu` | `game_frame` 신설, `position(game_frame)`, 첫 mark 단언 동일 방식 |
| `ingame_menu_app.rs` | `the_menu_opens_over_the_game_and_leaves_it_where_it_stopped` | 이전에는 아무 `Op::Image`나 game frame이었다. 이제 전체 패널 + `Lcd3x`만 인정(더 좁아짐), `game < dim` 순서 단언 유지 |
| `quick_state_app.rs` | `select_r1_writes_numbered_states_with_thumbnails_and_a_toast` | game frame을 `Op::ImageEffect { w: 720, effect: Lcd3x }`로 탐색. 그 뒤의 좁은 toast/thumbnail 단언은 `Op::Image { w < 200 }` 그대로 |
| `state_switcher_app.rs` | `the_save_state_row_opens_the_switcher_and_a_loads_the_greatest_slot` | 기존 `game_frame_at` helper만 좁게 수정(전체 패널 `Op::ImageEffect` + `Lcd3x`). 그 테스트의 첫 mark 탐색에 `ImageEffect`를 추가해 `first_mark == game_at`(game frame이 가장 먼저) 단언 유지 |
| `time_controls_app.rs` | `the_badge_follows_the_buttons_and_the_latch` | 기존 `game_frame_at` helper만 좁게 수정(game frame = 전체 패널 `ImageEffect` + `Lcd3x`). `game < plate < text` 순서 단언 유지 |

어느 자리에도 `Image | ImageEffect` 식의 느슨한 허용이나 settings의 explicit Off 주입, ignore/조기
return/단언 삭제를 쓰지 않았다. 여덟 fixture 모두 shader key가 없으므로 `Lcd3x`는 플랫폼 기본값이며,
effect 값 자체를 명시적으로 단언해 "기본값 상속"을 계속 검증한다.

## 일반 UI 단언 보존 근거

수정한 여덟 파일에서 `Op::Image`로 남은 자리는 전부 UI 텍스트/그림이고 손대지 않았다. helper를 새로
만든 곳은 game quad 조건과 effect를 함께 판별하도록 분리했고, UI image까지 effect-aware로 만들지 않았다.

- `device_menu_app.rs`: overlay 텍스트 존재 단언 `matches!(o, Op::Image { .. })`와 다른 테스트의 `Rect` 우선 단언 불변.
- `core_picker_app.rs`: 텍스트 quad 수집 helper `images()`(`Op::Image { x, y, w, .. }`) 불변.
- `time_controls_app.rs`: badge의 plate(`Op::Rect`)와 줄 텍스트(`Op::Image` in HUD band) 불변.
- `quick_state_app.rs`: thumbnail/toast 판정 `Op::Image { w < 200 }` 불변.
- `state_switcher_app.rs`: `has_capsule`(`Op::Rect`)와 switcher dim(`Op::Rect`) 단언 불변.
- `ingame_menu_app.rs`/`display_menu_app.rs`/`cheat_menu_app.rs`: dim `Op::Rect`, panel `Op::Rect`, HUD band 부재 단언 불변.

## 완료 기준 명령 (2차, 마지막 코드 변경 뒤 순서대로)

| 명령 | 종료 | 마지막 결과 줄 |
|---|---|---|
| `cargo fmt --all -- --check` | 0 | 출력 없음 |
| 8-target test (`cheat_menu_app` `core_picker_app` `device_menu_app` `display_menu_app` `ingame_menu_app` `quick_state_app` `state_switcher_app` `time_controls_app`) | 0 | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.79s` |
| `cargo test -p slot2 --tests --no-fail-fast` | 0 | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.47s` |
| `cargo check -p slot2 --tests` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 8.22s` |
| `cargo clippy -p slot2 --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 6.43s` |

- 8-target test 합계: target별 11 / 14 / 13 / 9 / 2 / 6 / 10 / 7 → **72 passed, 0 failed, 0 ignored**.
- `cargo test -p slot2 --tests --no-fail-fast` 합계: 22개 target(slot2 bin target 1개는 test 0개)
  → **231 passed, 0 failed, 0 ignored**.
- core-dependent skip: 두 명령 모두 **0개**. vendor에 mgba가 있어 real-core 테스트가 전부 실행됐고
  어느 target에도 ignore/조기 return이 없다. 1차 집중 테스트도 skip 0(registry 19, session 41).

## 수정 파일 (2차)

- `crates/slot2/tests/cheat_menu_app.rs`, `core_picker_app.rs`, `device_menu_app.rs`, `display_menu_app.rs`,
  `ingame_menu_app.rs`, `quick_state_app.rs`, `state_switcher_app.rs`, `time_controls_app.rs`
- `tasks/69-platform-shader-defaults.worker-result.md` (이 보고서)
- production 파일 무변경: 최근 40분 내 mtime이 바뀐 파일은 위 여덟 테스트뿐이며
  `crates/slot2/src/session.rs`, `crates/slot2-retro/src/{registry,lib}.rs`, store/gfx/UI 소스,
  `Cargo.toml`·lockfile, `docs/DESIGN.md`는 이번 호출에서 손대지 않았다. `git status`상 1차 변경분 외
  새 변경도 없다.
- 최종 검증 뒤 코드 변경 없음.

## 남은 위험

없음. 계약이 틀려 보이는 부분도 없다 — 다만 1차에서 "기본값 적용과 그 소비자(App 통합 테스트)의
시점 차이"를 위험으로 적었고, 그 후속이 이번 2차였다.

## 소요 시간

- 이번 호출(2차): 약 8분 (07:41 → 07:49 KST), 5개 command 실행 포함.
- 누적: 약 25분 (1차 17분 + 2차 8분), 호출 2회.
