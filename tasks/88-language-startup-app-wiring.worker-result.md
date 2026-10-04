# Task 88 — 작업자 결과 (누적 2/2)

- **누적 호출 횟수:** 2/2 (1차 구현 → 2차 escaping 수정)
- **결과:** 성공 (완료 기준 5개 명령 모두 종료 0)
- **소요 시간:** 2차 호출 약 5분 (22:02 → 22:07 KST). 1차 호출은 약 10분.

## 이번(2차) 수정

1차 보고서에서 `announce_language`가 `requested`만 `{:?}`로 escape하고 `effective`는 raw로 찍던
결함을 고쳤다. `SLOT2_LANG`은 계약상 제어문자를 포함한 값도 그대로 요청값이 되고, 그런 이름의 FTL
파일이 카드에 있으면 load가 성공해 `ctx.i18n.code()`에도 제어문자가 남을 수 있다. 그 경우
effective가 로그 줄을 쪼갤 수 있었다.

- 최종 형식(두 필드 모두 Rust debug escaping):
  `slot2: language requested={requested:?} effective={effective:?}`
- 정상 예: `slot2: language requested="ko" effective="ko"` — 두 값 모두 따옴표가 보인다.
- `\n`, `\t`, `"`, `\\`는 실제 제어문자로 출력되지 않고 각각 `\n`, `\t`, `\"`, `\\` 형태로 한 줄 안에
  남는다.
- 변경은 `crates/slot2/src/main.rs`의 `announce_language` 본문과 바로 위 설명 주석으로만 제한했다.
  언어 precedence, `requested_language` resolver, Card→requested→UiCtx→App 생성 순서, fallback,
  boot 로그의 `lang_override=...` escaping, FTL/store 파일, 테스트 시나리오는 그대로다. source-text
  assertion이나 새 로깅 abstraction도 만들지 않았다.

## 1차 구현 요약 (그대로 유지)

- `crates/slot2/src/lib.rs`: `pub fn requested_language(card, env_override) -> String`. `Some(v)`는
  철자·대소문자·빈 문자열을 그대로 반환하고(카드 read 생략), `None`만 `card.read_language()`를 쓴다.
  팩 탐색·parse·정규화·파일 쓰기·환경변수 접근 없음.
- `crates/slot2/src/main.rs`: `Boot.lang_override: Option<String>`으로 부재와 명시값을 구분.
  `var_os("SLOT2_LANG")`의 존재를 보존하고 비 Unicode 값은 경고 후 빈 명시 override로 전달(카드
  값으로 조용히 내려가지 않음). boot 로그는 `lang_override=none|<debug-escaped>`만 찍고 카드 언어를
  최종 언어처럼 표시하지 않는다.
- `host_app.rs` / `device_app.rs`: 두 backend 모두 Card 생성·`ensure_layout` → `requested_language`
  → `UiCtx::new(profile, &requested, font_dirs, Some(System/Lang))` → `announce_language` →
  `App::with_card(card, ..)`. Card는 한 번만 만들고 같은 인스턴스를 App에 넘기며 device probe는 같은
  시작 `UiCtx`를 받는다. surface/canvas·panel 보정 순서는 바꾸지 않았다.
- 신규 `crates/slot2/tests/language_startup.rs` (10 test).

## 검증 항목별 결과 (1·2차 공통, 회귀 없음)

- **우선순위:** env override가 카드보다 우선(`language = ko` + `Some("en")` → requested/effective
  `en`, 카드 값 `ko` 유지). override 부재면 카드 값. 카드 file/key 부재·unreadable·unsafe는 `en`.
- **override 실패:** `Some("")`/`Some("xx-unknown")` → requested는 각 값, effective는 `en`, 카드
  `ko`로 되돌아가지 않음. 다음 boot(override 없음)는 다시 `ko`.
- **built-in:** 저장 `ko` → requested/effective `ko`. 저장 `PT-br`/`ja_custom`은 철자 그대로
  requested, effective `en`.
- **card-only pack:** `System/Lang/xx-card.ftl` valid → effective `xx-card`, 카드 pack의 `lang-name`
  사용, 미정의 key는 내장 영어. pack bytes 불변.
- **같은 code 카드 pack:** 내장 `ko` 위에 valid 카드 pack이 있으면 카드 문구 우선(`power-off ==
  "카드 전원"`), 나머지는 내장 한국어(`resume == "계속하기"`). 그 카드 pack이 malformed면 effective
  `en`(`power-off == "Power off"`).
- **unknown/malformed/invalid UTF-8:** 설치되지 않은 저장 code는 requested 보존·effective `en`·
  bytes 불변(반복 boot에서도 되쓰지 않음). malformed pack과 invalid UTF-8 설정도 bytes 불변.
- **시작 read 부작용 없음:** 설정 파일/키 없는 카드에서 시작해도 `slot2.ini`가 생기지 않고 `System`
  entry 목록이 그대로.
- **두 기본값 일치:** `DEFAULT_LANGUAGE == FALLBACK == "en"`, `I18n::embedded(FALLBACK).code() == "en"`.
- **로그:** `lang_override=none|<debug-escaped>`(boot) + `requested=<escaped> effective=<escaped>`
  (양쪽 backend, UiCtx 생성 직후 한 줄).

## 완료 기준 명령 (2차 코드 변경 뒤 순서대로)

| 명령 | 종료 코드 | 마지막 결과 줄 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | (출력 없음) |
| `cargo test -p slot2 --test language_startup` | 0 | `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s` |
| `cargo test -p slot2` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (Doc-tests slot2, 최종 줄) |
| `cargo check -p slot2 --no-default-features --features device` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 2.55s` |
| `cargo clippy -p slot2 --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 9.72s` |

- `language_startup`: **10 passed / 0 failed / 0 ignored**.
- `cargo test -p slot2`: **325 passed / 0 failed / 0 ignored** (lib 33, main bin 0, integration 292,
  doc-tests 0; 30개 result 줄). 1차와 동일한 수치다.
- core-dependent skip: **0** (실행 출력에 `skipping` 없음).
- 전체 workspace 테스트와 device 배포는 실행하지 않았다.

## 변경 파일

- 2차: `crates/slot2/src/main.rs` (`announce_language` 본문 + 인접 설명 주석)
- 1차: `crates/slot2/src/lib.rs`, `crates/slot2/src/main.rs`, `crates/slot2/src/host_app.rs`,
  `crates/slot2/src/device_app.rs`, 신규 `crates/slot2/tests/language_startup.rs`
- 이번 호출에서 `slot2-store`, `slot2-i18n`, `slot2-ui`, FTL/폰트, App 화면 상태기계, Cargo 파일은
  수정하지 않았다.

## 최종 검증 뒤 코드 변경 여부 / 소요 시간

- 마지막 코드 변경은 2차의 `main.rs` 한 줄(+주석)이고, 그 뒤 위 5개 명령을 순서대로 실행했다.
  **검증 후 코드 변경 없음.** 2차 호출 소요 약 5분, 누적 약 15분.

## 남은 범위와 위험 (변경 없음)

- Shelf Language 행 활성화와 언어 picker, runtime `UiCtx`/font chain 교체와 texture cache 정리,
  선택 저장·실패 rollback/toast, `System/Lang` 목록 UI와 `lang-font` 적용은 후속 태스크 범위다.
- `requested_language`는 코드를 검증·정규화하지 않으므로 `SLOT2_LANG`의 제어문자가 그대로
  `UiCtx::new`까지 전달된다. load가 성공하면 effective code에도 남으며, 이제 두 값 모두 escaping되어
  로그가 쪼개지지는 않는다. “왜 영어/다른 언어인지”를 사용자에게 설명하는 것은 후속 picker/진단 몫이다.
- 계약이 틀려 보이는 부분은 없었다.
