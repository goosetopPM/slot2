# Task 70 작업자 보고

## 판정

**성공. 누적 호출 1/2.** 완료 기준 4개 명령 전부 종료 0. 최종 검증 뒤 코드 변경 없음. 이번 태스크는
1차에서 끝났고 2차 호출은 쓰지 않았다.

## 1. 선택 모델 (여섯 값/행 순서)

`crates/slot2-ui/src/shader_menu.rs` 신규. 새 enum을 만들지 않고 기존
`Option<slot2_store::ShaderPreset>`을 그대로 쓴다.

`pub const ROWS: [Option<ShaderPreset>; 6]`, 정확한 순서:

| 행 | 값 | 표시 key |
|---|---|---|
| 0 | `None` | `display-platform-default` (기존 key 재사용) |
| 1 | `Some(Off)` | `shader-off` |
| 2 | `Some(SharpBilinear)` | `shader-sharp-bilinear` |
| 3 | `Some(Lcd3x)` | `shader-lcd3x` |
| 4 | `Some(ZfastCrt)` | `shader-zfast-crt` |
| 5 | `Some(Scanline)` | `shader-scanline` |

- `ShaderMenu::new(current)`는 `ROWS`에서 같은 값을 찾아 그 행을 선택하고, `selected()`는 highlighted
  row의 정확한 `Option<ShaderPreset>`을 돌려준다. 여섯 값 각각에서 constructor→query 왕복이 일치함을
  테스트로 고정했다.
- `None`(상속)과 `Some(Off)`(명시적 끄기)는 별도 행이며, 서로 다른 행에 놓인다는 것을 별도 단언으로
  고정했다. UI는 `None`을 계산된 preset으로 바꾸지 않는다.
- `up()`/`down()`은 `ROWS.len()` 모듈로로 양 끝에서 wrap한다. 테스트는 고정 순서대로 한 바퀴 이동,
  마지막에서 down → 행 0, 첫 행에서 up → 행 5, 다시 한 바퀴로 원위치를 확인한다.
- 입력 event 처리, 파일 I/O와 App 상태는 컴포넌트에 없다(파라미터는 `Option<ShaderPreset>`뿐).
- `key()`는 exhaustive match이며 enum 순서·index·Debug 문자열을 쓰지 않는다. 여섯 행의 key 중복 없음도
  테스트로 확인한다.

## 2. draw 결과

`DisplayMenu`/`InGameMenu`/`PowerMenu`와 같은 팔레트·text 크기·safe-area·face cache를 쓴다.

- `Canvas::clear`를 호출하지 않는다(테스트에서 `Op::Clear` 0개 확인).
- 순서: physical panel 전체 `BLACK alpha 0.6` dim → safe area 중앙 panel(`BACKDROP`) → title →
  여섯 행 → select/back hint. 첫 mark가 dim이고 그 색이 `DIM`임을 단언한다.
- layout 상수는 계약대로 `BOX_W = 380`, `BOX_H = 316`, `PAD = 16`, `ROW_H = 36`, row 시작
  `box_y + PAD + 28`. 여섯 행(44..260), title(16..37), hint(row 286)가 box 안에서 겹치지 않는다.
- highlight는 선택된 행에만 `INK alpha 0.15`로 그린다. 테스트는 여섯 행 각각에서 highlight 여부를
  검사해 선택된 행 하나에서만 참임을 확인한다.
- `rgsp`/`rg35xxsp`/`rgcubexx` × `en`/`ko` 여섯 조합에서 box와 모든 op(전체 패널 dim 제외)가
  640×480 safe area 안에 있고, title이 box 상단 중앙, hint가 hint line 위에 있음을 확인했다.

## 3. 문구와 warm redraw

- `assets/lang/en.ftl`/`ko.ftl`에 `shader-title`/`shader-off`/`shader-sharp-bilinear`/`shader-lcd3x`/
  `shader-zfast-crt`/`shader-scanline` 여섯 key를 표의 문구 그대로 추가했다. 첫 행은 기존
  `display-platform-default`를 재사용한다.
- 검증: UI 테스트가 `en`/`ko` 각각의 정확한 문자열과, 여섯 label이 서로 다른 문구임을 확인한다.
  `crates/slot2-i18n/tests/i18n.rs`에 `shader_menu_messages_read_naturally_in_both_packs` 하나를
  추가해 두 pack이 fallback 없이 직접 정의함을 고정했다(여섯 문구 + `display-platform-default` +
  label 중복 없음).
- safe-area: 위 조합 전부에서 box·title·여섯 label·highlight·hint가 safe area 안(테스트로 확인).
- warm redraw: 한국어(`ko`, CJK face 포함)로 첫 draw 뒤 8회 반복 draw와 highlight 이동 뒤 draw에서
  `UploadAlpha8`/`UploadRgba8`가 0개임을 확인했다.

## 4. 완료 기준 명령

| 명령 | 종료 | 마지막 결과 줄 |
|---|---|---|
| `cargo fmt --all -- --check` | 0 | 출력 없음 |
| `cargo test -p slot2-ui --test shader_menu` | 0 | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.07s` |
| `cargo test -p slot2-ui -p slot2-i18n` | 0 | 마지막 target `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (23개 target 합계 231 passed / 0 failed / 0 ignored) |
| `cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 2.89s` |

skip/ignore는 어느 target에도 없다(0 ignored). 계약의 test 목록 8항목은 모두 `shader_menu` 테스트
7개와 i18n 테스트 1개로 덮인다.

## 5. 생성·수정 파일

- 생성: `crates/slot2-ui/src/shader_menu.rs`, `crates/slot2-ui/tests/shader_menu.rs`
- 수정: `crates/slot2-ui/src/lib.rs`(module 선언 1줄 + re-export 1줄), `assets/lang/en.ftl`,
  `assets/lang/ko.ftl`, `crates/slot2-i18n/tests/i18n.rs`(신규 key 직접 정의 테스트 1개)
- 생성: 이 보고서
- 최종 검증 뒤 코드 변경 없음. mtime 확인 결과 이번 호출에서 바뀐 파일은 위 목록뿐이며
  `display_menu.rs`·다른 UI 화면·store/gfx/retro/Session·`Cargo.toml`은 무변경이다.

## 6. 범위 밖 확인

- App screen variant, Display 진입 행, 입력 배선, live preview와 settings 저장은 구현하지 않았다.
  셰이더 선택은 Session/store에 전혀 연결되지 않았고, 컴포넌트는 값을 읽고 그리기만 한다(Task71 소관).
- 기존 `DisplayMenu`의 scale 행·layout·API와 그 테스트는 손대지 않았고, `-p slot2-ui` 전체 실행에서
  여전히 통과한다.
- GPU/GL 창, 실기, device 배포, workspace test, 네트워크는 실행하지 않았다.

## 7. 남은 위험 / 계약 소견

계약에서 틀린 부분은 없다. 남은 위험은 이 화면이 아직 어디에서도 열리지 않는다는 것뿐이며, 그 배선은
명세가 Task71로 분리한 범위다.

## 8. 소요 시간

약 22분 (10:29 → 10:51 KST), 가재코드 호출 1회.
