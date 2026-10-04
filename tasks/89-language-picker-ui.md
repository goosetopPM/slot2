# Task 89 — Language picker UI

현재 checkout에서 직접 작업한다. Shelf의 Language 행에서 열 독립 `LanguagePicker` UI를
`slot2-ui`에 추가한다. caller가 이미 load에 성공한 언어의 code와 자기 이름(`lang-name`)을 넘기며,
picker는 현재값·선택값·스크롤과 안전한 렌더링만 담당한다. 파일 검색, FTL parse, runtime 언어 교체와
저장은 후속 App 배선으로 분리한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

필요한 부분만 읽는다.

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\88-language-startup-app-wiring.result.md`
- `C:\SLOT2\docs\DECISIONS.md`의 D-09, D-12~D-14만
- `C:\SLOT2\docs\DESIGN.md`의 §8 언어 선택·폰트 부분만
- `C:\SLOT2\crates\slot2-ui\src\core_picker.rs`의 owned rows, navigation, draw 방식만
- `C:\SLOT2\crates\slot2-ui\src\timezone_menu.rs`의 safe-area panel과 hint draw 방식만
- `C:\SLOT2\crates\slot2-ui\src\shelf_menu.rs`의 `ShelfChoice::Language` key만
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`의 module/re-export와 `UiCtx` 공개 계약만
- `C:\SLOT2\crates\slot2-ui\tests\core_picker.rs`와 `timezone_menu.rs`의 geometry/warm redraw
  검증 helper 부분만
- `C:\SLOT2\assets\lang\en.ftl`, `ko.ftl`의 generic hint와 Shelf 문구 부분만

App/main/host/device, store, `slot2-i18n` 구현, Session/core, 다른 메뉴 전체, 워커 로그와 저장소 이력은
읽지 않는다.

## 책임 경계

- picker는 파일 시스템, `System/Lang`, 환경변수, `Card`, `I18n::available/load`, `UiCtx` 교체와
  설정 저장을 호출하지 않는다.
- caller가 **실제로 load에 성공한** 후보들을 안정된 순서로 넘긴다. 각 후보의 표시 이름은 그 후보
  pack의 `I18n::name()` 결과다. 따라서 한국어는 현재 UI가 영어여도 `한국어`로 표시할 수 있다.
- picker는 caller 순서를 보존한다. 언어 이름 정렬이나 locale collation을 새로 구현하지 않는다.
- code 비교는 대소문자를 포함한 정확한 문자열 비교다. 정규화·BCP-47 해석을 하지 않는다.
- A/B 동작, runtime preview/commit, 저장 실패 rollback/toast와 Shelf 행 활성화는 후속이다.

## 공개 모델 계약

새 `crates/slot2-ui/src/language_picker.rs`를 만들고 crate root에서 다음 두 타입을 re-export한다.

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguageOption { /* private fields */ }

impl LanguageOption {
    pub fn new(code: impl Into<String>, name: impl Into<String>) -> Self;
    pub fn code(&self) -> &str;
    pub fn name(&self) -> &str;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LanguagePicker { /* private fields */ }

impl LanguagePicker {
    pub fn new(options: Vec<LanguageOption>, current: &str) -> Self;
    pub fn options(&self) -> &[LanguageOption];
    pub fn highlighted(&self) -> Option<&LanguageOption>;
    pub fn current(&self) -> Option<&LanguageOption>;
    pub fn changed(&self) -> bool;
    pub fn is_empty(&self) -> bool;
    pub fn len(&self) -> usize;
    pub fn first_visible(&self) -> usize;
    pub fn up(&mut self);
    pub fn down(&mut self);
    pub fn draw(&self, canvas: &mut dyn Canvas, ctx: &mut UiCtx);
}
```

- `LanguageOption`은 code/name 철자를 그대로 소유한다. name이 빈 문자열이거나 whitespace뿐이면 draw에서
  code를 표시 이름으로 사용하되 원래 name 값 자체는 바꾸지 않는다.
- `LanguagePicker::new`는 exact code 중복을 **처음 것만 남겨** 제거한다. 같은 표시 이름을 가진 서로
  다른 code는 모두 남긴다.
- `current` code가 후보에 있으면 그 행에서 시작하고 current badge도 그 행에 고정한다.
- current가 후보에 없으면 첫 행에서 시작하고 `current()`는 `None`이다. 빈 목록이면 highlighted/current
  모두 `None`, `changed()`는 false다.
- 비어 있지 않고 current가 목록에 없으면 첫 후보가 선택 가능한 새 값이므로 `changed()`는 true다.
  current가 있으면 highlighted code가 current code와 다른지만 반환한다.
- `up`/`down`은 wrap한다. 빈 목록과 한 행은 안전하게 그대로다.
- 목록은 임의 길이를 받을 수 있다. 선택 행은 항상 visible window에 들어오며 `first_visible`은
  `0..len` 범위다.

## 그리기 계약

- 다른 메뉴처럼 panel 전체 dim → safe area 중앙 panel → title → visible rows → position → hints 순서로
  overlay하고 화면을 clear하지 않는다.
- title은 기존 `shelf-language` key를 재사용한다.
- panel은 640×480 safe area 안에 완전히 들어가고 세 geometry(`rgsp`, `rg35xxsp`, `rgcubexx`)에서 같은
  safe-area 좌표를 사용한다.
- `MAX_VISIBLE_ROWS`는 최소 5, 최대 7의 고정값으로 둔다. 전체 후보가 더 많으면 selected를 따라 window가
  움직인다. 행 위치와 panel/hint 위치는 전체 후보 수에 따라 흔들리지 않는다.
- 각 행은 자기 언어 이름을 본문 크기로 왼쪽에, code 또는 current 표시를 hint 크기로 오른쪽에 그린다.
  current 표시는 Fluent 완성 문장 `language-code-current`에 `$code`를 넘겨 만들고, 나머지는
  `language-code`에 `$code`를 넘긴다. 코드에서 `· Current` 같은 문장을 조립하지 않는다.
- 선택 행만 기존 메뉴와 같은 반투명 highlight를 갖는다. current badge는 navigation과 함께 움직이지
  않는다.
- 목록이 비면 highlight 없이 `language-picker-empty`를 첫 행 영역에 표시한다.
- position은 `language-picker-position`에 1-based `$current`, `$total`을 넘긴 완성 메시지로 표시한다.
  빈 목록에서는 position을 그리지 않는다.
- 하단은 기존 `hint-select`, `hint-back`을 사용한다.
- 카드 pack의 name/code는 길이 제한을 신뢰하지 않는다. 각 좌우 column의 실제 측정 폭 안에 들어오도록
  Unicode char boundary를 지키며 말줄임한다. 말줄임 결과도 panel/safe area 밖으로 나가면 안 된다.
  이름의 빈 값은 code로 대체하고, 긴 ASCII·한글·emoji에서 panic하지 않는다.
- warm redraw는 같은 `UiCtx`에서 새 alpha/RGBA upload를 만들지 않는다.

## 번역 계약

`assets/lang/en.ftl`, `ko.ftl`에 다음 key를 추가한다. 한국어는 자연스러운 완성 문장으로 번역한다.

- `language-code = { $code }`
- `language-code-current = { $code } · Current` / `{ $code } · 사용 중`
- `language-picker-empty = No languages available` / `사용 가능한 언어가 없습니다`
- `language-picker-position = { $current } / { $total }`

기존 key를 바꾸거나 code/name을 번역하지 않는다.

## 테스트 계약

새 `crates/slot2-ui/tests/language_picker.rs`에서 최소한 다음을 검증한다.

- option이 code/name의 대소문자·Unicode를 그대로 보존하고 caller 순서를 유지함
- exact duplicate code는 첫 항목만 남고, 같은 name의 다른 code는 남음
- current가 있으면 그 행이 highlighted/current이고 `changed == false`; 이동 후 current badge는 고정되고
  changed가 true, 다시 돌아오면 false
- current가 없을 때 첫 행·current None·changed true, 빈 목록에서는 모든 accessor/navigation이 안전함
- up/down wrap, 한 행 안정성, `MAX_VISIBLE_ROWS`보다 긴 목록에서 selected가 항상 visible하고 window가
  양 끝 wrap 뒤 올바르게 이동함
- 영어·한국어 context에서 self-name이 그대로 표시되고 code/current/empty/position/hints 번역 key가
  빠지지 않음
- 세 geometry에서 dim/panel/rows/position/hints와 모든 text/image rect가 safe area 안에 있음
- 매우 긴 ASCII·한글·emoji name/code, 빈/whitespace name이 panic 없이 column 안에서 말줄임·fallback됨
- selected highlight와 current 표시는 서로 독립이고, 빈 목록에는 highlight/position이 없음
- 같은 menu/context의 두 번째 draw에서 upload 수가 0인 warm redraw

texture id의 정확한 번호, 실제 GL 창, 파일 시스템, 환경변수, Card/App/input에 의존하지 않는다. 기존
테스트의 단언을 새 동작을 숨기도록 완화하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-ui\src\language_picker.rs` (신규)
- `C:\SLOT2\crates\slot2-ui\src\lib.rs` (module/re-export만)
- `C:\SLOT2\crates\slot2-ui\tests\language_picker.rs` (신규)
- `C:\SLOT2\assets\lang\en.ftl`
- `C:\SLOT2\assets\lang\ko.ftl`
- `C:\SLOT2\tasks\89-language-picker-ui.worker-result.md`

그 밖의 production/test 파일은 수정하지 않는다. 특히 `slot2`, `slot2-store`, `slot2-i18n`, Cargo 파일,
Shelf/App 배선을 고치지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- `I18n::available/load` 호출과 pack filtering/discovery
- Shelf Language 행 활성화, `Screen`/input/App 배선
- 선택 즉시 `UiCtx`, i18n bundle, font chain, face/texture cache 교체
- `write_language`, 저장 실패 rollback/toast와 env override 변경
- `lang-font` 적용과 새 폰트 asset
- M4/M5 checkbox 완료 처리
- 전체 workspace 테스트, 실제 GL 창, device 배포
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 네트워크와 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-ui --test language_picker
cargo test -p slot2-ui
cargo test -p slot2-i18n
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 각 test 명령의 passed/failed/ignored 합계를 보고한다. 검증 뒤 코드를 바꾸면
영향받는 명령부터 다시 실행한다. workspace test와 device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\89-language-picker-ui.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- option/current/highlight/changed/dedup/navigation/window model 결과
- self-name, code/current/empty/position/hints와 긴 문자열 말줄임 결과
- 세 geometry·영어/한국어 safe-area와 warm redraw 결과
- picker가 파일/I18n load/store/App를 건드리지 않았다는 경계 확인
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄과 passed/failed/ignored 합계
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- App discovery/runtime context 교체/저장·rollback이 후속이라는 남은 범위
- 계약이 틀려 보이는 부분이 있으면 그 내용
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
