# Task 90 — Language picker App/runtime 배선

현재 checkout에서 직접 작업한다. Task89의 `LanguagePicker`를 Shelf Language 행에 연결하고, 실제
내장·카드 pack 중 load에 성공한 후보를 만들며, 선택한 언어의 새 `UiCtx`를 host/device loop에서
교체한다. 새 context 구성과 카드 저장이 모두 성공한 경우에만 전환하고, 어느 단계든 실패하면 기존
언어·context·카드 값을 유지하며 toast를 표시한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

필요한 부분만 읽는다.

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\88-language-startup-app-wiring.result.md`
- `C:\SLOT2\tasks\89-language-picker-ui.result.md`
- `C:\SLOT2\docs\DECISIONS.md`의 D-12~D-14만
- `C:\SLOT2\crates\slot2-i18n\src\lib.rs`의 `FALLBACK`, `available`, `load`, `embedded`, `code`,
  `name` 공개 계약만
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`의 `UiCtx::new`와 public field만
- `C:\SLOT2\crates\slot2-ui\src\language_picker.rs`의 공개 모델 계약만
- `C:\SLOT2\crates\slot2\src\app.rs`의 `Screen`, App field/constructor, Shelf/Timezone input·draw,
  toast와 request accessor 부분만
- `C:\SLOT2\crates\slot2\src\lib.rs`
- `C:\SLOT2\crates\slot2\src\host_app.rs`, `device_app.rs`의 startup context와 loop의
  `run_frame`→request→draw 구간만
- `C:\SLOT2\crates\slot2\tests\timezone_menu_app.rs`, `about_sticker_app.rs`의 Shelf availability와
  App/input helper 부분만
- `C:\SLOT2\assets\lang\en.ftl`, `ko.ftl`의 Language/실패 toast 인접 부분만

Session/core, 게임별 메뉴, renderer 구현, font internals, 다른 테스트 전체, 워커 로그와 저장소 이력은
읽지 않는다. 새 variant 때문에 직접 깨지는 exhaustive match만 위치를 찾아 최소 수정할 수 있다.

## 선택과 적용 순서

- App이 Language 화면을 열 때만 `I18n::available(Some(System/Lang))`을 한 번 호출한다. draw/frame마다
  card directory를 다시 읽지 않는다.
- available의 각 code를 같은 card lang dir로 `I18n::load`한다. 성공한 code만 `LanguageOption(code,
  loaded.name())`으로 picker에 넣는다. caller 순서, 즉 `available`의 정렬·dedup 결과를 유지한다.
- card의 `en.ftl` override가 malformed여도 frontend의 내장 영어는 실제 fallback으로 실행 가능하므로
  `en`은 `I18n::embedded(FALLBACK)`으로 복구해 후보에 반드시 남긴다. 다른 code의 load 실패는 한 줄
  debug-escaped 진단 후 제외한다.
- picker의 current는 backend가 알려 준 **현재 effective language**다. 저장된 requested code나
  `SLOT2_LANG` 문자열을 current로 추측하지 않는다.
- A에서 선택이 unchanged면 write/context rebuild 없이 Shelf parent로 돌아간다.
- changed 선택은 App이 code request만 queue한다. backend가 다음 draw 전에 새 `UiCtx`를 만든다.
- 새 context의 `i18n.code()`가 requested와 정확히 같고 `Card::write_language(requested)`도 성공해야
  context를 교체하고 current를 갱신한다. 이 순서를 지킨다: **load candidate → save → swap**.
- load mismatch/fallback이면 저장하지 않고 기존 context를 유지한다. save 실패면 새 candidate context를
  버리고 기존 context를 유지한다. 두 실패 모두 Language 화면에 남아 current badge/선택을 유지하고
  각각의 toast를 표시한다. runtime preview는 하지 않으므로 별도 rollback write가 필요 없다.
- 성공하면 Shelf parent의 Language 행으로 돌아간다. 새 `UiCtx`는 새 i18n/font chain/face cache를 가지며
  이전 언어의 face cache를 재사용하지 않는다.

## App 계약

### 1. Screen과 상태

- `Screen`에 `Language(ShelfMenu)` variant를 추가한다. `LanguagePicker`는 owned list라 `Screen`에 넣지
  않고 `App` private field로 한 인스턴스를 둔다. `Screen`의 `Copy` 계약을 유지한다.
- App private field에 `language_picker`, `current_language: String`, pending language request를 둔다.
  새 App의 기본 current는 `slot2_i18n::FALLBACK`이다.
- 다음 read-only/public 경계를 제공한다. 이름은 동등하게 명확하면 조정할 수 있다.

```rust
pub fn language_picker(&self) -> &LanguagePicker;
pub fn current_language(&self) -> &str;
pub fn set_current_language(&mut self, code: &str);
```

`set_current_language`는 backend가 startup `ctx.i18n.code()`를 App에 알려 주는 용도이며 pack load나
저장을 하지 않는다.

### 2. Shelf와 입력

- `shelf_availability()`는 Language, TimeZone, About 세 행을 true로 한다. ShelfMenu의 기존 순서상
  처음 선택은 Language다.
- Shelf Language에서 A → 후보를 새로 discovery하고 `Screen::Language(parent)`.
- Language에서 Up/Down → picker 이동, A → unchanged면 Shelf; changed면 request queue, B/Menu → write와
  request 없이 Shelf parent.
- pending request가 있는 동안 같은 Language 화면의 추가 입력은 무시한다. backend service가 동기적으로
  완료한 뒤 다시 입력받는다.
- 빈 picker에서 A는 write/request 없이 화면에 남는다.
- draw는 shelf 위에 LanguagePicker만 그린다. parent ShelfMenu를 겹쳐 그리지 않는다. hold bar, toast,
  HUD가 기존 z-order로 위에 남고 shelf-side screen 판정에 `Screen::Language`를 포함한다.

### 3. 실패와 toast

영문·한글 pack에 다음 완성 문장을 추가한다.

- `language-load-failed = Could not load that language; the previous language is still active`
  / `선택한 언어를 불러오지 못해 이전 언어를 유지합니다`
- `language-save-failed = Could not save the language; the previous language is still active`
  / `언어를 저장하지 못해 이전 언어를 유지합니다`

실패 로그의 code는 `{:?}`로 escape한다. toast는 기존 context에서 만들어지고 그 언어로 표시된다.

## Runtime service 계약

`crates/slot2/src/lib.rs`에 host/device와 integration test가 공유할 public helper를 추가한다.

```rust
pub fn service_language_request(
    app: &mut app::App,
    ctx: &mut slot2_ui::UiCtx,
    card_lang_dir: &std::path::Path,
) -> bool;
```

- pending request가 없으면 아무것도 하지 않고 false.
- `ctx.profile`, `ctx.font_dirs.clone()`, requested code와 `card_lang_dir`로 `UiCtx::new`를 정확히 한 번
  만든다. 기존 context를 먼저 move/drop하지 않는다.
- candidate effective code가 requested와 다르면 App에 load failure를 알리고 candidate를 버린 뒤 false.
- effective code가 같으면 App의 commit 경계를 호출한다. commit은 `write_language`를 먼저 수행하고,
  성공할 때만 App current/screen을 갱신한다.
- commit 성공 뒤에만 `*ctx = candidate`로 교체하고 true. save 실패면 candidate를 버리고 false.
- 요청·실제 code 진단은 둘 다 debug escaping한다.
- 이 helper에서 `I18n::load`를 별도로 호출해 pack을 두 번 parse하지 않는다.

host/device loop는 `app.run_frame()` 뒤, draw 전에 매 frame 이 helper를 한 번 호출한다. startup App 생성
직후에는 `app.set_current_language(ctx.i18n.code())`를 정확히 한 번 호출한다. 두 backend가 별도 적용
규칙을 복제하지 않는다.

## 저장·경계 계약

- 성공 시 Task87 `write_language`만 사용해 volume·timezone·unknown key를 보존한다. `en` 선택은 key
  부재로 저장된다.
- unchanged, B/Menu, load 실패에서는 설정 bytes와 mtime을 바꾸지 않는다.
- write 실패에서는 기존 bytes/directory가 그대로고 runtime context/current도 바뀌지 않는다.
- `SLOT2_LANG`은 startup override일 뿐이다. 실행 중 사용자가 성공적으로 선택하면 현재 process에서는
  선택값이 즉시 effective가 되고 카드에도 저장된다. 다음 시작에 환경변수가 여전히 있으면 Task88
  precedence가 다시 적용된다.
- unsafe하지만 load 가능한 card filename code는 picker에 나타날 수 있다. store write가 거부하면 save
  failure로 처리한다. store validator를 복제·공개하거나 임의 정규화하지 않는다.
- `lang-font` preferred font 적용은 이번 범위 밖이다. context 전체를 다시 만들기 때문에 현재 기본
  UI/CJK font chain과 face cache는 새 언어 기준으로 깨끗하게 재생성된다.

## 테스트 계약

새 `crates/slot2/tests/language_picker_app.rs`에서 실제 App, Card, UiCtx,
`service_language_request`와 `RecordingCanvas`를 사용해 최소한 다음을 검증한다.

- Shelf availability가 Language+TimeZone+About이고 첫 행 Language; 기존 navigation으로 세 행만 순환
- Language A가 built-in en/ko와 valid card-only pack을 available 순서·self-name으로 picker에 넣음
- malformed/unknown-extension pack은 후보에서 제외되고, malformed `en.ftl`이어도 embedded English가
  후보/current로 남음
- backend startup setter가 `ctx.i18n.code()`를 current badge로 사용함(SLOT2_LANG/stored requested를
  App이 재추측하지 않음)
- picker Up/Down, B/Menu round-trip은 write/request/context 변경 없음
- unchanged A는 write와 context rebuild 없이 Shelf로 복귀
- changed A 직후 request가 한 번만 service되고, valid built-in/card-only 선택은 save 후 context code와
  App current가 바뀌며 Shelf Language 행으로 복귀
- 성공 교체가 profile/font_dirs를 보존하고 기존 face cache를 버림. 새 context draw는 선택 언어 문구를
  사용함
- selected `en`은 language key만 제거하고 volume/timezone/unknown key를 보존
- 후보를 연 뒤 pack 삭제/손상 → service load mismatch, 설정 bytes·기존 context/current 불변,
  Language 화면 유지와 load-failed toast
- `slot2.ini`를 directory/invalid UTF-8로 만들어 write 실패 → 기존 context/current와 원본 불변,
  Language 화면 유지와 save-failed toast
- unsafe loadable code의 write 거부도 같은 save failure이며 panic하지 않음
- 빈 picker 경계는 가능하면 직접 모델 setup으로, A no-op을 확인
- Language 화면 draw가 picker panel 하나만 그리고 Shelf parent panel을 겹치지 않으며 HUD/toast z-order와
  warm redraw를 보존
- request가 없을 때 service는 false이고 `UiCtx`/App/card를 바꾸지 않음

process 환경변수를 설정/삭제하지 않는다. texture id 번호, 실제 GL 창, core/audio/session에 의존하지
않는다. 기존 테스트의 단언을 새 동작을 숨기도록 완화하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2\src\app.rs`
- `C:\SLOT2\crates\slot2\src\lib.rs`
- `C:\SLOT2\crates\slot2\src\host_app.rs`
- `C:\SLOT2\crates\slot2\src\device_app.rs`
- `C:\SLOT2\crates\slot2\tests\language_picker_app.rs` (신규)
- `C:\SLOT2\crates\slot2\tests\timezone_menu_app.rs`의 availability/navigation expectation만
- `C:\SLOT2\crates\slot2\tests\about_sticker_app.rs`의 availability/navigation expectation만
- 새 `Screen::Language`로 직접 깨지는 기존 `crates/slot2/tests/*_app.rs` exhaustive match/setup만
- `C:\SLOT2\assets\lang\en.ftl`
- `C:\SLOT2\assets\lang\ko.ftl`
- `C:\SLOT2\tasks\90-language-picker-app-wiring.worker-result.md`

그 밖의 production 파일은 수정하지 않는다. 특히 `slot2-ui`, `slot2-i18n`, `slot2-store`, font/text,
Session/core와 Cargo 파일을 고치지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- `UiCtx::new` 또는 i18n load/fallback 구현 변경
- 언어팩 `lang-font` preferred font 적용, 새 폰트 asset
- store validator/INI 형식 변경과 unsafe code 숨김
- 언어 이름 locale 정렬·검색·자동 번역
- 언어 전환 중 게임/session/core/audio 동작 변경
- M4/M5 checkbox 완료 처리
- 전체 workspace 테스트, 실제 GL 창, device 배포
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 네트워크와 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2 --test language_picker_app
cargo test -p slot2
cargo test -p slot2-i18n
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2 --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 각 test 명령의 passed/failed/ignored 합계와 core-dependent skip 수를 보고한다.
검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test와 device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\90-language-picker-app-wiring.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- discovery/filtering, embedded en 복구와 current effective source 결과
- Shelf/Language input·draw와 request one-shot 결과
- load→save→context swap 순서, 성공 시 face cache 교체와 실패 시 불변/각 toast 결과
- global key 보존, en key 제거, unsafe code save failure 결과
- host/device startup setter와 per-frame shared service 배선
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored와 skip 수
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- `lang-font` 적용이 후속이라는 남은 범위
- 계약이 틀려 보이는 부분이 있으면 그 내용
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
