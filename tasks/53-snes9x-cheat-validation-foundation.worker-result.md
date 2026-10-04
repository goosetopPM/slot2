# Task 53 워커 결과 — SNES9x 치트 검증 기반

**성공. 누적 호출 1/2.** 지정 검증 4종 모두 종료 0.

## library path 식별 계약

- `registry::Core::from_library_path(path: &Path) -> Option<Self>` 추가(권장 시그니처 그대로).
  파일명에서 마지막 확장자를 떼어낸 stem을 각 variant의 `base_name()`과 **ASCII
  case-insensitive**로 비교하고, 확장자는 `.dll`/`.so`/`.dylib`을 호스트 OS와 무관하게 인정한다.
  확장자가 없는 이름도 인정하되, 확장자가 있으면서 라이브러리 확장자가 아니면 거부한다. 부모
  디렉터리 이름은 판정에 쓰지 않는다.
- `Core::ALL`(5개) 상수를 함께 추가했고 `base_name`·`file_name`·platform registry 결과는 그대로다.
- 테스트(`tests/registry.rs`, 14 passed): 5개 core × 3개 확장자, `MGBA_LIBRETRO.DLL` 같은 전대문자와
  `sNeS9x_LIBretro.So` 혼합 대소문자, `core.file_name()` 왕복, 디렉터리 무관(`gpsp_libretro/
  snes9x_libretro.so` → Snes9x), `libmgba_libretro.so`·`mgba_libretro.so.1`·`mgba_libretro.dll.bak`·
  `mgba.so`·`mgba_libretro.txt`·`nosuch_libretro.so`·`.so`·`System/cores/` 거부.

## SNES9x validator

- `quirks::validate_cheat(core: CoreId, code: &str) -> Result<CheatValidation, CheatValidationError>`
  (공개, crate root re-export). 순수 함수이며 파일·코어·전역 상태를 읽지 않는다.
- **인정**: 8자리 ASCII hex Pro Action Replay(`12345678`, `abcdef12`), `AAAAAA:VV`(`7E0000:01`),
  `XXXX-XXXX` Game Genie(`DF47-0915`, alphabet `DF4709156BC8A23E` 대소문자 무관), 그리고 이들을
  `+`, `,`, `.`, `;`, space 다섯 separator로 이은 다중 token. 연속·선행·후행 separator는 건너뛴다.
- **거부**: 7자리/9자리 hex, colon·hyphen 위치 오류, colon/hyphen 뒤 자릿수 오류, `_`,
  alphabet 밖 문자(`DF47-091G`, `GGGG-0915`), 자릿수 부족(`DF-0915`), tab/newline(separator가 아니라
  token 안으로 들어간다), 전각 숫자, separator만 있는 문자열, 빈 문자열, 내부 NUL, 256 bytes 이상.
- **255-byte 경계**: 24개의 PAR token + 4개의 `AAAAAA:VV` token을 `+`로 이어 정확히 255 bytes인
  문자열은 `Validated`, 같은 문자열에 1 byte를 더한 256 bytes는 길이 오류(reason에 256 명시).
  길이는 Unicode scalar가 아니라 `code.as_bytes().len()`이라 200자 한글(600 bytes)도 길이 오류로
  거부되고 reason이 600을 알린다.
- 8자리 Game Genie alphabet은 hex의 부분집합이라 hyphen을 뺀 8문자는 PAR로도 성립한다. 형식이
  겹치는 것은 core도 같은 방식으로 순서대로 시도하기 때문이며, 그래서 인정하는 쪽이 맞다(테스트에
  근거 주석을 남겼다).

## 나머지 네 코어

- mGBA·gpSP·FCEUmm·Genesis Plus GX는 non-empty·NUL-free 입력이면 형식과 무관하게
  `CheatValidation::Unchecked`다. SNES9x가 거부하는 코드도 이 네 코어에는 `Unchecked`로 남는다(남의
  문법을 빌려 거부하지 않는다).
- 빈 문자열과 내부 NUL 오류는 코어별 질문보다 **먼저** 적용되어 5개 코어 모두에서 같은 오류가 난다.
- 오류 타입은 `core()`와 `reason()`을 보존하고 `Debug`·`Display`·`std::error::Error`를 제공한다.
  reason에는 cheat code 전체를 복사하지 않으며(테스트로 고정), `Display`는
  `snes9x_libretro: ...` 형태로 코어를 밝힌다.

## 검증 (최종 코드 상태, 명세 순서)

- `cargo fmt --all -- --check` → 종료 **0**.
- `cargo test -p slot2-retro --test registry` → 종료 **0**,
  `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s`.
- `cargo test -p slot2-retro --test cheat_quirks` → 종료 **0**,
  `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s`.
  (이 파일은 순수 함수만 다루므로 코어나 카드가 없어도 skip하지 않는다.)
- `cargo clippy -p slot2-retro --all-targets -- -D warnings` → 종료 **0**(경고·오류 없음).
- 최종 검증 후 코드 변경 없음.

## 생성·수정 파일

- 신규 `crates/slot2-retro/src/quirks.rs`: `CheatValidation`, `CheatValidationError`,
  `validate_cheat`, SNES9x token/length 규칙.
- 신규 `crates/slot2-retro/tests/cheat_quirks.rs`: 9 테스트.
- `crates/slot2-retro/src/registry.rs`: `Core::ALL`과 `Core::from_library_path`.
- `crates/slot2-retro/src/lib.rs`: `pub mod quirks`, re-export 3개, 모듈 doc 한 줄.
- `crates/slot2-retro/tests/registry.rs`: library path 테스트 2개.
- `slot2`·`slot2-store`·`slot2-ui`·Session·App은 건드리지 않았다. `Core::set_cheat`/`reset_cheats`
  동작과 `.cht` 읽기·쓰기도 그대로다.

## 계약 우려 / 남은 위험

SNES9x token 형식이 서로 겹칠 수 있어(8자리 Genie alphabet ⊂ hex) `-` 없는 Genie 코드는 PAR로
인정되는데, pin한 adapter가 같은 순서로 시도하므로 실질 위험은 없다. 나머지 네 코어 문법 검증은
D-21의 후속 몫이며, 그때까지 `Unchecked`는 "확인하지 않음"이지 "문제 없음"이 아니다.
소요 약 14분(상한 45분 내).
