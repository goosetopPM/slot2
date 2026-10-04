# Task 87 — 전역 언어 설정 store

현재 checkout에서 직접 작업한다. 선반 Language 행의 기반으로 `System/slot2.ini`에 선택 언어 코드를
독립적으로 읽고 안전하게 쓰는 `slot2-store` API를 추가한다. 기본 언어는 `en`이며 키 부재로 표현한다.
volume·시간대·미래 키를 양방향 보존하고, 카드 전용 `System/Lang/<code>.ftl`을 막지 않도록 store는
팩 존재 여부를 판단하지 않는다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

필요한 부분만 읽는다.

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\64-global-volume-settings-store.result.md`
- `C:\SLOT2\tasks\80-global-timezone-settings-store.result.md`
- `C:\SLOT2\docs\DESIGN.md`의 전역 설정 계층과 `System/Lang` 부분만
- `C:\SLOT2\docs\MILESTONES.md`의 M4 Language와 M5 언어팩 항목만
- `C:\SLOT2\crates\slot2-store\src\settings.rs`의 global volume/timezone API와 safe-write 방식만
- `C:\SLOT2\crates\slot2-store\src\lib.rs`의 settings re-export 부분만
- `C:\SLOT2\crates\slot2-store\tests\global_settings.rs`
- `C:\SLOT2\crates\slot2-store\tests\timezone_settings.rs`
- `C:\SLOT2\crates\slot2-i18n\src\lib.rs`의 `FALLBACK`, `I18n::available/load` 공개 계약만
- `C:\SLOT2\crates\slot2-i18n\src\imp.rs`의 `<code>.ftl` 경로 조립 부분만

App/main/UI, i18n 구현 변경, Session/core, 다른 store 기능, 워커 로그와 저장소 이력은 읽지 않는다.

## 저장 계약

- 파일: `System/slot2.ini`
- key: `language`
- default: `en`
- default `en`은 key 부재로 저장한다. untouched card와 명시적으로 English로 되돌린 카드는 같은
  상태다.
- store는 언어팩이 현재 설치돼 있는지 판단하지 않는다. syntactically safe한 `pt-BR`, `zh-Hant`,
  `ja_custom` 같은 코드를 보존한다. 실제 사용 가능성과 FTL parse 성공 여부는 후속 i18n/App 경계가
  `I18n::available/load`로 판정한다.
- code는 `System/Lang/<code>.ftl`의 **단일 파일 stem**으로 안전해야 한다.
  - 비어 있지 않고 UTF-8 byte 길이 1..=64
  - `/`, `\`, NUL, 제어문자, code 내부 공백이 없음
  - `.` 또는 `..`가 아님
  - 그 밖의 Unicode, ASCII 영숫자, `-`, `_`, `.`은 허용한다. store가 BCP-47을 새로 구현하지 않는다.
- read에서 invalid/unsafe code는 default `en`으로 돌아가며 원본을 고치지 않는다.
- write에서 invalid/unsafe code는 `Error::Invalid`로 파일을 읽기 전 거부한다.
- read/write는 `System/Lang` 파일을 생성·삭제·수정하지 않는다.

## 구현 계약

### 1. 공개 상수와 API

`slot2-store` crate root에서 다음을 re-export한다.

```rust
pub const LANGUAGE_KEY: &str = "language";
pub const DEFAULT_LANGUAGE: &str = "en";
```

`Card`에 다음 API를 추가한다.

```rust
pub fn read_language(&self) -> String;
pub fn write_language(&self, code: &str) -> Result<(), Error>;
```

- `read_language`는 missing file/key, unreadable file, invalid UTF-8, unsafe code에서
  `DEFAULT_LANGUAGE.to_owned()`를 반환한다. panic하거나 파일을 변경하지 않는다.
- valid code는 철자와 대소문자를 그대로 반환한다. lowercase 정규화, locale 확장, 설치 여부 검사를
  하지 않는다. `language = ko  `처럼 INI 구문이 값 바깥에 허용하는 padding은 기존 `Ini` parser가
  제거하므로 `ko`로 읽는다. `language = ko KR`처럼 code 내부에 남는 whitespace는 invalid다.
- `write_language`는 safe code를 그대로 쓴다. `en`이면 key를 제거하고, 파일이 비면 파일도 기존
  safe-write 계약대로 제거한다.
- 언어 API를 `GlobalSettings` field에 넣지 않는다. volume debounce write가 language 기본값을
  덮어쓰는 구조를 만들지 않고, 시간대처럼 독립 read/write API로 둔다.
- parsing/validation helper는 private로 두고 read와 write가 같은 기준을 사용한다.

### 2. 다른 global key 보존

- language write는 `volume`, `utc_offset_minutes`, unknown/future key와 그 값을 보존한다.
- volume write와 timezone write도 이미 있는 `language` key를 보존해야 한다. 기존 구현을 중복하거나
  재작성하지 말고 회귀 테스트로 봉인한다.
- language default write는 language key만 제거한다. 다른 key가 있으면 파일을 유지한다.
- language만 있던 파일에서 `en`을 쓰면 빈 파일을 제거한다.
- missing file에 `en`을 쓰면 파일을 만들지 않는다.
- safe-write temp/rename과 `Ini` 정렬 규칙은 기존 구현을 그대로 재사용한다. 별도 serializer나
  language 전용 파일을 만들지 않는다.
- unreadable/invalid UTF-8 설정 파일 또는 `slot2.ini` 자리에 directory가 있으면 덮어쓰거나 제거하지
  않고 기존 `Error::Io`로 실패한다.

### 3. 책임 경계

- `slot2-store`는 `slot2-i18n`에 의존하지 않는다. `DEFAULT_LANGUAGE` 값은 현재 fallback `en`과 같지만
  crate dependency나 compile-time cross-assertion은 후속 App 경계에서 다룬다.
- `SLOT2_LANG` 환경변수를 읽거나 변경하지 않는다.
- `I18n::load`, `I18n::available`, FTL parse, font 선택을 호출하지 않는다.
- App 시작 적용, language picker, runtime `UiCtx` 교체와 저장 실패 toast는 후속 태스크다.

## 테스트 계약

`crates/slot2-store/tests/language_settings.rs`를 새로 추가해 최소한 다음을 검증한다.

1. missing file/key는 `en`이고 read가 파일·디렉터리 entry를 만들거나 바꾸지 않는다.
2. `ko`, `pt-BR`, `zh-Hant`, `ja_custom`, safe Unicode stem이 exact round-trip한다.
3. valid code의 대소문자·점·하이픈·밑줄 철자를 read가 정규화하지 않는다.
4. empty, 65-byte 초과, `.`, `..`, slash, backslash, 내부 whitespace, newline/control, NUL code는
   read에서 `en`, write에서 `Error::Invalid`이며 기존 bytes가 불변이다. direct write의 앞뒤 whitespace도
   invalid이고, INI delimiter 바깥 padding은 기존 parser 규칙대로 무시돼야 한다.
5. syntactically safe하지만 설치되지 않은 code도 exact round-trip한다.
6. language write가 volume, utc offset, unknown key를 보존한다.
7. volume write와 timezone write가 language를 보존한다.
8. `en` write는 language key만 제거하고, 다른 key가 없을 때만 파일을 제거한다.
9. missing file에 `en`을 반복해서 써도 파일이 생기지 않는다.
10. invalid UTF-8 file과 settings path directory는 read에서 `en`, write에서 실패하며 bytes/directory와
    entry 수가 불변이다.
11. write 뒤 `.tmp`가 남지 않고 `System/Lang`의 파일과 다른 game settings 파일이 불변이다.
12. environment, OS locale, i18n pack 목록을 전혀 보지 않는다는 계약을 결과와 소스에서 확인한다.

Windows permission bit에 의존하지 않는다. 기존 portable invalid UTF-8/directory failure pattern을
사용한다. sleep, mtime 정밀도, network, 새 dependency, test-only production hook에 의존하지 않는다.
테스트를 통과시키기 위해 unknown code를 `en/ko` 목록으로 제한하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-store\src\settings.rs`
- `C:\SLOT2\crates\slot2-store\src\lib.rs`의 신규 상수 re-export만
- `C:\SLOT2\crates\slot2-store\tests\language_settings.rs` (신규)
- 새 공개 상수/API로 직접 깨지는 기존 `slot2-store` test의 import/단언만
- `C:\SLOT2\tasks\87-global-language-settings-store.worker-result.md`

다른 production/test 파일은 수정하지 않는다. 특히 `slot2`, `slot2-ui`, `slot2-i18n`, platform,
Cargo 파일과 FTL을 고치지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- App/main startup language 적용, `SLOT2_LANG` precedence 변경
- language picker UI와 Shelf Language 행 활성화
- runtime UiCtx/i18n/font chain 교체
- FTL pack discovery/parse/fallback 변경
- System/Lang 파일 생성·편집·삭제
- M4/M5 문서 checkbox 완료 처리
- 전체 workspace 테스트, 실제 GL 창, device 배포
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-store --test language_settings
cargo test -p slot2-store
cargo check -p slot2
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2-store --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 각 test 명령의 passed/failed/ignored 합계를 보고한다. 검증 뒤 코드를 바꾸면
영향받는 명령부터 다시 실행한다. workspace test와 device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\87-global-language-settings-store.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- key/default, safe stem validation과 unknown pack code 보존 결과
- volume/timezone/unknown key 양방향 보존 결과
- default removal, missing/unreadable/directory와 temp cleanup 결과
- store가 i18n/environment/System/Lang을 건드리지 않았다는 확인
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored 합계
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 후속 startup/picker/runtime 적용이 남았다는 확인
- 계약이 틀려 보이는 부분과 남은 위험
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
