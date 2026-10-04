# Task 87 — 작업자 결과

- **누적 호출 횟수:** 1/2
- **결과:** 성공 (완료 기준 6개 명령 모두 종료 0)
- **소요 시간:** 약 6분 (20:09 → 20:15 KST, 최종 검증 순차 실행 포함)

## 구현 요약

- `crates/slot2-store/src/settings.rs`에 `LANGUAGE_KEY = "language"`,
  `DEFAULT_LANGUAGE = "en"`, private `MAX_LANGUAGE_CODE_BYTES = 64`, private
  `parse_language_code(&str) -> Option<&str>`을 추가하고 `Card::read_language() -> String`,
  `Card::write_language(&str) -> Result<(), Error>`를 시간대 API 옆에 추가했다.
- `crates/slot2-store/src/lib.rs`의 settings re-export에 두 상수만 추가했다.
- 언어는 `GlobalSettings` field가 아니라 독립 read/write API다(volume debounce write가 language
  기본값을 덮어쓰는 구조를 만들지 않음). read/write가 같은 private validator를 쓴다.
- code 규칙은 *파일 이름* 규칙이다: 비어 있지 않고 UTF-8 byte 1..=64, `.`/`..` 불가, 그 외에는
  Unicode를 포함해 path separator(`/`, `\`)·제어문자(NUL 포함)·whitespace(앞뒤 포함)가 없어야 한다.
  BCP-47 해석·lowercase 정규화·locale 확장·설치 여부 판정은 하지 않는다.
- read: missing file/key, unreadable file, invalid UTF-8, unsafe code → `"en"`, 파일 무변경.
  write: unsafe code는 파일을 읽기 전 `Error::Invalid`, `en`은 key 제거 후 파일이 비면 파일도 제거,
  그 외에는 그대로 저장. safe-write temp/rename과 `Ini` 정렬은 기존 구현을 그대로 쓴다.
- `slot2-i18n`에 의존하지 않고 `SLOT2_LANG`/환경변수/`I18n::available`/`load`/font 선택을 호출하지
  않으며 `System/Lang`을 열지 않는다.
- 신규 `crates/slot2-store/tests/language_settings.rs` (11 test).

## 검증 항목별 결과

- **key/default:** `LANGUAGE_KEY == "language"`, `DEFAULT_LANGUAGE == "en"`. 파일/키 부재 카드에서
  `"en"`이고 read가 파일이나 `System` entry를 만들지 않는다. key가 없는 파일(`volume = 30`만)도
  같은 답이며 read가 파일을 rewrite하지 않는다.
- **safe stem / unknown code:** `ko`, `pt-BR`, `zh-Hant`, `ja_custom`, `xx-NotInstalled`, `a.b`,
  `1.2.3`, `한국어`, `pt-BR-x-private`, 64-byte code가 exact round-trip하고, 저장된 파일은
  `language = <code>` 한 줄이다. 65-byte는 read에서 `en`, write에서 거부. 설치되지 않은 code도
  보존됨(팩 존재 여부를 store가 판단하지 않음).
- **정규화 없음:** `PT-br`, `KO`, `zh-hant`, `ja_custom`, `ZH-HANT`, `Pt`가 철자·대소문자 그대로
  read/write된다.
- **unsafe 처리:** write는 `""`, `.`, `..`, `a/b`, `a\b`, `ko KR`, `ko\tKR`, NUL, ` ko`, `ko `,
  `\tko\t`, 65-byte를 모두 `Error::Invalid`로 거부하고 기존 bytes를 바꾸지 않는다. read는 `""`, `.`,
  `..`, `a/b`, `a\b`, `ko KR`, `ko\tKR`, NUL, 65-byte에서 `en`이고, INI delimiter 바깥 padding
  (`language =   ko   `, `  language  =  pt-BR`)은 parser가 제거해 각각 `ko`, `pt-BR`로 읽힌다.
- **양방향 보존:** language write가 `volume`, `utc_offset_minutes`, `future_key`를 보존하고,
  `write_global_settings`와 `write_utc_offset_minutes`가 `language = ko`를 보존한다(회귀 test로 봉인).
- **default removal:** `en` write는 `language` key만 제거하고 다른 key가 있으면 파일을 유지한다.
  language만 있던 파일은 `en` write 뒤 파일째 사라지고, missing file에 `en`을 반복해 써도 파일이
  생기지 않는다.
- **실패 경로:** invalid UTF-8 설정 파일과 `slot2.ini` 자리의 directory는 read에서 `en`, write에서
  `Error::Io`이며 bytes/directory와 entry 수가 불변이다.
- **temp/다른 파일:** language write 뒤 `System` entry 목록이 write 전과 동일하고 `.tmp`가 남지
  않는다. test가 만든 `System/Lang/ko.ftl`과 game settings `System/games/GBA/arm.ini`의 bytes가
  그대로다.
- **경계 확인:** `settings.rs`가 `std::env`, `env::var`, `slot2_i18n`, `I18n`, `"Lang"`,
  `available(`, `locale`을 포함하지 않음을 소스에서 확인하고, 파일에 `language = ko`가 있으면 그 값이
  반환됨을 결과로 확인했다.

## 완료 기준 명령 (마지막 코드 변경 뒤 순서대로)

| 명령 | 종료 코드 | 마지막 결과 줄 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | (출력 없음) |
| `cargo test -p slot2-store --test language_settings` | 0 | `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s` |
| `cargo test -p slot2-store` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (Doc-tests slot2_store, 최종 줄) |
| `cargo check -p slot2` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 12.99s` |
| `cargo check -p slot2 --no-default-features --features device` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 2.43s` |
| `cargo clippy -p slot2-store --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 3.66s` |

- `language_settings`: **11 passed / 0 failed / 0 ignored**.
- `cargo test -p slot2-store`: **112 passed / 0 failed / 0 ignored**
  (lib 1 + card 19 + cheats 15 + core_states 9 + global_settings 11 + language_settings 11 +
  overlay_settings 10 + shader_settings 13 + state_undo 11 + timezone_settings 12, doctests 0).
  Task 80 기준 101개에 신규 11개가 더해진 값과 일치한다.
- `fmt`가 재포맷한 파일은 이번에 추가/수정한 Rust 파일뿐이었다. 마지막 코드 변경(fmt) 뒤 위 6개
  명령을 순서대로 다시 실행했고 그 뒤 코드 변경은 없다.

## 변경 파일

- 수정: `crates/slot2-store/src/settings.rs`, `crates/slot2-store/src/lib.rs` (신규 상수 re-export만)
- 신규: `crates/slot2-store/tests/language_settings.rs`, `tasks/87-global-language-settings-store.worker-result.md`

`slot2`, `slot2-ui`, `slot2-i18n`, platform, Cargo 파일, FTL, dist script는 수정하지 않았다.

## 후속 / 남은 위험

- App/main 시작 시 `read_language` 적용, `SLOT2_LANG` precedence 결정, language picker와 Shelf
  Language 행 활성화, runtime `UiCtx`/font chain 교체, 저장 실패 toast는 다음 태스크 범위다. 이
  태스크는 store API만 제공한다.
- `DEFAULT_LANGUAGE == i18n fallback "en"` 동등성은 주석으로만 적혀 있고 compile-time 대조는 후속
  App 경계 몫이다(계약이 명시한 대로 store는 i18n에 의존하지 않는다).
- code가 실제 팩으로 존재하는지, FTL이 parse되는지는 이 store가 판정하지 않는다. 후속 경계가
  `I18n::available/load`로 판정하고, 실패 시 fallback과 사용자 안내를 정해야 한다.
- `...`처럼 점만 여러 개인 code는 이 규칙에서 safe로 통과한다(금지 목록이 `.`와 `..`뿐). Linux/FAT
  카드에서는 파일 이름이 될 수 있지만 Windows에서 trailing dot이 정리되는 차이가 있어, 후속 picker가
  이런 code를 만들지 않는 것이 안전하다. 계약 위반은 아니며 실제 팩이 없으면 `en`으로 떨어진다.
- 계약이 틀려 보이는 부분은 없었다. read 경로의 앞뒤 whitespace는 validator가 아니라 `Ini` parser가
  먼저 제거하므로 `en`이 아니라 trim된 code로 읽히는데, 이는 명세가 명시한 동작이다.
