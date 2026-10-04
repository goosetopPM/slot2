# Task 89 — 작업자 결과

- **누적 호출 횟수:** 1/2
- **결과:** 성공 (완료 기준 6개 명령 모두 종료 0)
- **소요 시간:** 약 22분 (22:40 → 23:02 KST, 최종 검증 순차 실행 포함)

## 구현 요약

- 신규 `crates/slot2-ui/src/language_picker.rs`: `LanguageOption`(code/name 소유, private field)과
  `LanguagePicker`(options/selected/current row/first visible window, `Clone + Debug + PartialEq + Eq`).
  crate root re-export 두 줄 추가.
- `assets/lang/en.ftl`, `ko.ftl`에 `language-code`, `language-code-current`, `language-picker-empty`,
  `language-picker-position` 네 key를 추가했다(기존 key·문구는 변경 없음).
- 신규 `crates/slot2-ui/tests/language_picker.rs` (11 test).
- picker는 파일·`System/Lang`·환경변수·`Card`·`I18n::available/load`·`UiCtx` 교체·설정 저장을
  호출하지 않는다. caller가 이미 load한 후보만 받는다(`I18n::name()` 결과를 표시 이름으로 사용).

## 검증 항목별 결과

- **model:** `LanguageOption`은 code/name의 대소문자·Unicode·앞뒤 공백까지 그대로 보존하고 caller
  순서를 유지한다(`pt-BR`, `한국어`, `xx-E🚀`, `"  日本語  "` 확인). exact code 중복은 첫 항목만 남고
  (`en`/`EN`은 다른 code라 둘 다 유지), 같은 표시 이름을 가진 다른 code도 둘 다 남는다.
- **current/highlight/changed:** current가 목록에 있으면 그 행에서 시작하고 `highlighted`/`current`가
  같은 행, `changed == false`. 이동하면 highlight만 움직이고 badge 행은 고정, `changed == true`.
  돌아오면 `changed == false`. current가 없으면 첫 행·`current() == None`·`changed == true`
  (첫 후보는 새 값이므로), 빈 목록은 `highlighted`/`current` 모두 `None`·`changed == false`·`up/down`
  no-op·`first_visible == 0`이다.
- **navigation/window:** `up/down`은 wrap하고 한 행 목록은 안정적이다. `MAX_VISIBLE_ROWS = 6`보다 긴
  9개 목록에서 9스텝을 돌며 selected가 항상 `first_visible..first_visible+6` 안에 있고 window가
  끝을 넘지 않음을 확인했다. 첫 행에서 up은 마지막 행 + tail window, 마지막 행에서 down은 첫 행 +
  top window로 돌아온다.
- **self-name:** 한국어 pack은 영어 화면에서도 `한국어`로 표시된다(행 이름은 그 pack의 `lang-name`,
  번역하지 않음). `code`/`current` 표시는 Fluent 완성 문장이며 en은 `en · Current`, ko는 `en · 사용 중`
  (`{ $code }` 메시지는 두 언어 모두 code 그대로). 빈/공백 이름은 code로 대체하고 원래 name 값은
  건드리지 않는다.
- **empty/position/hints:** 빈 목록은 첫 행 자리에 `language-picker-empty`를 그리고 highlight도
  position도 없다. 비어 있지 않으면 `language-picker-position`이 선택 행(1-based)/전체를 표시하고
  하단은 `hint-select` + `hint-back`이다.
- **말줄임:** 400자 ASCII 이름, 200자 한글 이름, 120개 emoji 이름, 긴 code, 빈/공백 이름 모두 panic
  없이 그려지고 좌우 column(이름 224px, code 160px, gap 12px) 안에서 잘린다. 잘림은 crate의 기존
  `cheat_menu::fit`(char boundary 보존, 이분 탐색)을 재사용했고, 그 결과가 실제로 panel 안에 그려지는
  지 측정 폭으로 확인했다.
- **세 geometry 안전영역:** `rgsp`/`rg35xxsp`/`rgcubexx`에서 dim은 panel 전체 1개(첫 mark), panel은
  safe area 중앙에 정확히 1개, 모든 rect/image가 safe area 안이며 clear가 없다. 세 패널에서 box의
  safe-area 상대 좌표가 동일함을 확인했다(D-09).
- **warm redraw:** 같은 picker/context의 두 번째 draw는 upload 0이고 mark 목록이 첫 프레임과 동일하다.
  window 안에서 highlight를 옮기면 새로 생기는 것은 position 숫자 face 하나뿐이고(≤1), 이미 그린
  숫자로 되돌아가면 다시 0이다.
- **경계:** picker source에 `std::fs`, `std::env`, `env::var`, `UiCtx::new`, `slot2_store`,
  `write_language`, `available(`, `read_dir`이 없음을 테스트가 source에서 직접 확인한다(파일·팩·카드·
  저장·context 교체에 접근하지 않음). 사용하는 것은 `Canvas`, `Arg`, `UiCtx`, `cheat_menu::fit`뿐이다.

## 완료 기준 명령 (마지막 코드 변경 뒤 순서대로)

| 명령 | 종료 코드 | 마지막 결과 줄 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | (출력 없음) |
| `cargo test -p slot2-ui --test language_picker` | 0 | `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.02s` |
| `cargo test -p slot2-ui` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (Doc-tests slot2_ui, 최종 줄) |
| `cargo test -p slot2-i18n` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (Doc-tests slot2_i18n, 최종 줄) |
| `cargo check -p slot2 --no-default-features --features device` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 10.58s` |
| `cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 18.86s` |

- `language_picker`: **11 passed / 0 failed / 0 ignored**.
- `cargo test -p slot2-ui`: **256 passed / 0 failed / 0 ignored** (lib 1 + integration 255, doctests 0;
  26개 result 줄). Task 85 시점의 ui 245에 신규 11개가 더해진 값과 일치한다.
- `cargo test -p slot2-i18n`: **34 passed / 0 failed / 0 ignored** (i18n.rs 34, lib 0, doctests 0).
- core-dependent skip은 이 두 crate에 없다(실행 출력에 `skipping` 없음). workspace test와 device 배포는
  실행하지 않았다.
- `fmt`가 재포맷한 파일은 이번에 건드린 Rust 파일뿐이었고, 그 뒤 위 6개 명령을 순서대로 다시 실행했다.
  **검증 후 코드 변경 없음.**

## 변경 파일

- 신규: `crates/slot2-ui/src/language_picker.rs`, `crates/slot2-ui/tests/language_picker.rs`
- 수정: `crates/slot2-ui/src/lib.rs` (module/re-export 두 줄), `assets/lang/en.ftl`, `assets/lang/ko.ftl`
- 신규: `tasks/89-language-picker-ui.worker-result.md`

`slot2`, `slot2-store`, `slot2-i18n`, Cargo 파일, Shelf/App 배선 파일은 수정하지 않았다.

## 계약 검토 / 판단한 부분

- **position의 `$current`:** picker의 `current()`는 “지금 쓰는 언어”이고 position은 “목록에서 어디인가”
  이므로, 1-based 선택 행(highlighted)을 `$current`로 넘겼다. 계약 문구(“1-based `$current`”)와
  모델 용어가 겹쳐 해석 여지가 있었고, 화면에 “n / m”으로 보이는 값은 선택 위치가 맞다.
- **오른쪽 column도 말줄임:** 계약이 좌우 column 모두 말줄임을 요구하므로, code 표시는 `spans` 대신
  `I18n::t_args`로 완성 문장을 얻어 `fit`한 뒤 한 Text로 그린다. `language-code`/`language-code-current`
  에는 BTN 자리표시자가 없어 그려지는 문자열은 pack 문장 그대로이고, code를 조각내어 잇는 코드는 없다.
- **`cheat_menu::fit` 재사용:** 말줄임 로직을 새로 만들지 않고 같은 crate의 기존 helper를 쓴다.
- **빈 목록의 `first_visible`:** `0`을 반환한다(`0..len`은 빈 목록에서 공허하게 참).
- `MAX_VISIBLE_ROWS`는 계약 범위 5..=7에서 6으로 정했다. 행/position/hint 위치는 후보 수와 무관하다.

## 남은 범위

- App discovery: `I18n::available`로 카드·내장 pack 목록을 만들어 picker에 넘기는 배선, Shelf Language
  행 활성화, A/B 입력 연결은 후속이다.
- 선택 즉시 runtime `UiCtx`/i18n bundle/font chain/face cache 교체, `write_language` 저장과 실패
  rollback/toast, `lang-font` 적용도 후속이다. 이 태스크는 picker 컴포넌트만 제공한다.
