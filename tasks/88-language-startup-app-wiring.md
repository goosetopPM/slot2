# Task 88 — 저장 언어의 frontend 시작 적용

현재 checkout에서 직접 작업한다. Task87의 `System/slot2.ini` 언어 선택을 host/device frontend의
`UiCtx` 생성 전에 읽어 실제 시작 언어로 사용한다. 명시적인 `SLOT2_LANG`은 호스트 개발용 일시
override로 유지하고, 선택한 pack이 없거나 깨졌어도 저장값을 고치지 않은 채 내장 영어로 안전하게
부팅한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

필요한 부분만 읽는다.

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\87-global-language-settings-store.result.md`
- `C:\SLOT2\tasks\01-i18n.md`에서 load/fallback 계약 부분만
- `C:\SLOT2\docs\DECISIONS.md`의 D-12~D-14만
- `C:\SLOT2\docs\DESIGN.md`의 §8 언어 선택·폰트 부분만
- `C:\SLOT2\crates\slot2-store\src\settings.rs`의 언어 상수와 read API만
- `C:\SLOT2\crates\slot2-i18n\src\lib.rs`의 `FALLBACK`, `I18n::load`, `code` 공개 계약만
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`의 `UiCtx`와 `UiCtx::new`만
- `C:\SLOT2\crates\slot2\src\lib.rs`
- `C:\SLOT2\crates\slot2\src\main.rs`의 `Boot`, `boot`, `main`만
- `C:\SLOT2\crates\slot2\src\host_app.rs`와 `device_app.rs`의 Card/UiCtx/App 생성 순서만

App 화면 상태기계, Session/core, 메뉴 구현, 다른 store API, 전체 i18n 구현, 워커 로그와 저장소 이력은
읽지 않는다. 직접 깨지는 생성부 테스트만 위치를 찾기 위해 추가로 읽을 수 있다.

## 우선순위와 fallback 계약

시작 시 **요청 언어(requested)**는 다음 순서로 한 번 결정한다.

1. process에 `SLOT2_LANG`이 존재하면 그 값을 일시 override로 사용한다.
2. 환경변수가 없으면 `Card::read_language()`를 사용한다.
3. 카드 파일/key 부재·unreadable·unsafe 값은 store 계약에 따라 `en`이 된다.

- 환경변수가 존재하면 빈 문자열이나 알 수 없는 값이어도 카드 저장값으로 되돌아가지 않는다. 명시적
  override의 load가 실패하면 내장 영어가 **실제 언어(effective)**가 된다.
- BaseOS는 `SLOT2_LANG` 없이 frontend를 실행하므로 실기에서는 카드 저장값이 우선한다.
- 요청 언어는 `UiCtx::new`에 카드 `System/Lang` 경로와 함께 넘긴다. 기존 `UiCtx::new`의
  `I18n::load(requested, card_dir)` → 내장 `en` fallback 계약을 재사용한다.
- 저장 code가 현재 설치되지 않았거나 선택 pack/그 fallback pack의 FTL parse가 실패하면 frontend는
  panic하지 않고 내장 영어로 계속 실행한다. `slot2.ini`와 `.ftl` 원본은 변경·삭제하지 않는다.
- `I18n::available` 목록 포함 여부를 시작 판정으로 쓰지 않는다. 실제 `I18n::load` 성공이 authority다.
- `slot2_store::DEFAULT_LANGUAGE`와 `slot2_i18n::FALLBACK`이 둘 다 `en`임을 회귀 테스트로 봉인한다.
  store가 i18n에 의존하도록 만들지는 않는다.

## 구현 계약

### 1. 테스트 가능한 요청 언어 resolver

`crates/slot2/src/lib.rs`에 환경변수를 직접 읽지 않는 작은 public helper를 추가한다.

```rust
pub fn requested_language(card: &slot2_store::Card, env_override: Option<&str>) -> String;
```

- `Some(value)`는 철자·대소문자·빈 문자열을 그대로 소유 `String`으로 반환한다.
- `None`만 `card.read_language()`를 호출한다.
- 팩 탐색, FTL parse, 정규화, 파일 쓰기와 환경변수 읽기/변경을 하지 않는다.
- 환경변수 접근은 binary `boot()` 한 곳에 남기고 테스트는 helper 인자로 precedence를 검증한다.

### 2. Boot가 override 부재를 보존

- 현재 `Boot.lang: String`은 환경변수 부재를 `en`으로 바꿔 카드와 env precedence를 구분하지 못한다.
  이를 `lang_override: Option<String>`으로 바꾸거나 동등하게 **부재와 명시값을 구분**하도록 고친다.
- `boot()`는 `std::env::var_os("SLOT2_LANG")` 존재 여부를 보존한다. 비 Unicode 값은 panic하지 않는다.
  비 Unicode override는 사용할 수 없는 명시값으로 취급해 영어 fallback으로 가거나, 경고 후 빈 명시
  override로 전달해도 된다. 카드 저장값으로 조용히 내려가면 안 된다.
- App/Card 생성 전의 boot 진단 로그가 카드 언어를 최종 언어처럼 표시하지 않게 한다. 이 시점에는
  `lang_override=<값 또는 none>`만 표시한다. 비 Unicode 원문이나 민감한 환경 전체는 출력하지 않는다.

### 3. host/device의 단일 생성 순서

두 backend에서 다음 순서를 동일하게 사용한다.

1. `Card::new(&boot.root)`
2. 기존 위치에서 `card.ensure_layout()`
3. `requested_language(&card, boot.lang_override.as_deref())`
4. `UiCtx::new(profile, &requested, font_dirs, Some(System/Lang))`
5. 같은 `Card`를 `App::with_card`에 이동

- `Card`나 `App`을 언어 때문에 두 번 만들지 않는다. `read_language`를 backend당 한 번만 호출하는 구조로
  둔다. env override가 있으면 카드 언어 read 자체를 건너뛴다.
- device input probe도 같은 시작 `UiCtx`를 받아 저장 언어/override를 사용한다.
- 요청 언어와 `ctx.i18n.code()`가 다르면 `requested=<...> effective=en`을 알 수 있는 한 줄 진단을
  남긴다. 같을 때도 기존 boot line 또는 별도 한 줄에서 effective code를 확인할 수 있어야 한다.
  제어문자를 그대로 로그에 흘리지 말고 debug escaping을 사용한다.
- profile geometry 보정, surface/canvas 생성, probe, App 설정, clock/volume startup과 loop 순서는 언어
  초기화에 필요한 범위 밖에서 바꾸지 않는다.
- 기존 `UiCtx::new` fallback 구현을 중복하거나 `I18n::load`를 미리 호출해 같은 pack을 두 번 parse하지
  않는다.

## 테스트 계약

`crates/slot2/tests/language_startup.rs`를 새로 만들고 실제 `Card`, `requested_language`, `UiCtx::new`를
사용해 다음을 검증한다. process 환경변수를 설정/삭제하지 않는다.

- `None` + 설정 파일/key 부재 → requested/effective 모두 `en`, 시작 read가 파일을 만들지 않음
- 저장 `ko`, `PT-br`, 카드 전용 safe code가 requested에 철자 그대로 전달됨
- 저장 `ko` + `Some("en")` → requested/effective `en`; env override가 카드보다 우선
- 저장 `ko` + `Some("")` 또는 unknown override → 카드 `ko`로 내려가지 않고 effective `en`
- unknown 저장 code → requested는 보존되지만 effective는 `en`, 설정 bytes 불변
- `System/Lang/<code>.ftl`의 최소 valid 카드 전용 pack → effective code가 그 code이고 card pack의
  `lang-name`을 읽음
- 선택한 카드 pack이 malformed → effective `en`, pack과 설정 bytes 불변
- built-in `ko`를 선택하면 effective `ko`; built-in code와 같은 카드 pack이 valid하면 card text가
  우선하고, malformed이면 내장 영어로 안전하게 fallback
- invalid UTF-8/unsafe language 설정은 store 계약대로 requested/effective `en`, 원본 bytes 불변
- `DEFAULT_LANGUAGE == FALLBACK == "en"`

테스트용 FTL은 임시 카드 아래에만 만들고 저장소 asset을 추가하지 않는다. texture id, 실제 GL 창,
입력/audio/core, process env mutation에 의존하지 않는다. 기존 단언을 새 동작을 숨기도록 완화하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2\src\lib.rs`
- `C:\SLOT2\crates\slot2\src\main.rs`
- `C:\SLOT2\crates\slot2\src\host_app.rs`
- `C:\SLOT2\crates\slot2\src\device_app.rs`
- `C:\SLOT2\crates\slot2\tests\language_startup.rs` (신규)
- 시작 계약 변경으로 직접 깨지는 binary 생성부 테스트/setup만
- `C:\SLOT2\tasks\88-language-startup-app-wiring.worker-result.md`

그 밖의 production 파일은 수정하지 않는다. 특히 `slot2-store`, `slot2-i18n`, `slot2-ui`, FTL/폰트,
Shelf/App 화면 상태기계와 Cargo 파일을 고치지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지
않는다.

## 범위 밖 및 금지

- Shelf Language 행 활성화와 언어 선택 화면
- 실행 중 `UiCtx`/font chain 교체, texture cache 정리
- 언어 선택 저장, rollback, toast와 retry
- `System/Lang` 목록 UI와 `lang-font` 적용
- store validator/INI 형식 변경 또는 실패한 저장값 자동 수정
- 환경변수 mutation, process-global test lock, 새 dependency
- 전체 workspace 테스트, 실제 GL 창, device 배포
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 네트워크와 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2 --test language_startup
cargo test -p slot2
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2 --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 각 test 명령의 passed/failed/ignored 합계와 core-dependent skip 수를 보고한다.
검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test와 device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\88-language-startup-app-wiring.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- env override → card → `en` requested/effective 우선순위 결과
- built-in/card-only/override/unknown/malformed/invalid UTF-8 시나리오와 원본 보존
- host/device Card→requested→UiCtx→App 생성 순서 및 probe 적용
- boot 진단 로그의 override/effective 표시 방식
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored와 skip 수
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 언어 picker/runtime 교체·저장 실패 rollback이 후속이라는 남은 범위
- 계약이 틀려 보이는 부분이 있으면 그 내용
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
