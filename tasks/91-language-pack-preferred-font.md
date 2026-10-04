# Task 91 — 언어팩 preferred font 적용

현재 checkout에서 직접 작업한다. Task90으로 완성된 startup/runtime `UiCtx` 생성 경계에서 언어팩의
`lang-font`를 실제 UI 본문 선호 폰트로 적용한다. 카드 `System/Fonts`를 우선하고 host asset을 그다음으로
찾되, 잘못되거나 없거나 깨진 선호 폰트 때문에 언어 전환이나 기본 UI/CJK 렌더가 실패해서는 안 된다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

필요한 부분만 읽는다.

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\90-language-picker-app-wiring.result.md`
- `C:\SLOT2\docs\DECISIONS.md`의 D-12~D-14만
- `C:\SLOT2\docs\DESIGN.md`의 §8 다국어·폰트 부분만
- `C:\SLOT2\crates\slot2-i18n\src\lib.rs`의 `I18n::load`, `code`, `font` 공개 계약만
- `C:\SLOT2\crates\slot2-text\src\lib.rs`의 `FontChain` 공개 API만
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`의 font 상수, `UiCtx`, `UiCtx::new`만
- `C:\SLOT2\crates\slot2-ui\tests\splash.rs`의 font id/지연 로딩 관련 테스트만
- `C:\SLOT2\crates\slot2\tests\language_startup.rs`, `language_picker_app.rs`의 `UiCtx` 생성과 언어
  전환 성공 테스트 부분만
- `C:\SLOT2\assets\lang\en.ftl`, `ko.ftl`의 `lang-name`, `lang-font` 부분만
- `C:\SLOT2\crates\slot2\src\main.rs`의 `font_dirs`만 읽어 search order를 확인한다.

App 상태기계, 다른 UI 화면, renderer, session/core/store, fontdue 내부 구현, 다른 테스트 전체, 워커 로그와
저장소 이력은 읽지 않는다. 기존 font id를 숫자로 가정해 직접 깨진 테스트만 위치를 찾아 최소 수정할 수
있다.

## 확정 계약

### 1. 선호 폰트의 의미

- source는 요청 문자열이 아니라 **성공적으로 load된 effective `I18n::font()`**다. 요청 언어가 실패해
  내장 영어로 fallback했다면 영어의 빈 preference를 사용한다.
- `lang-font`는 UI 본문에서 가장 먼저 시도할 폰트다. 성공적으로 찾은 경우 폴백 체인은 다음 순서다.

```text
language preferred font (lazy)
→ embedded OpenSans UI font (eager)
→ Noto Sans KR CJK fallback (lazy)
```

- preference가 없거나 사용할 수 없으면 기존 순서인 `embedded OpenSans → CJK fallback`을 유지한다.
- 선호 폰트가 일부 문자만 가지거나 load/parse에 실패하면 다음 폰트가 그 문자를 그린다. 폰트 문제는
  언어팩 load 실패, 언어 선택 저장 실패 또는 toast 사유가 아니다.
- preferred와 CJK가 정규화 없이도 **같은 최종 PathBuf**라면 같은 파일을 두 번 등록하지 않는다.
  이 경우 preferred가 앞에 한 번만 있고, 그 자체가 CJK fallback도 담당한다.
- `FontChain`과 `FaceCache`는 계속 `UiCtx`별 소유다. 전역 캐시나 이전 언어의 font/face cache를
  재사용하지 않는다.

### 2. 파일명과 탐색 경계

- `lang-font`는 **비어 있지 않은 단일 파일명**만 허용한다. `/`, `\`, absolute/prefix/root,
  `.`/`..` 또는 둘 이상의 path component가 들어간 값은 사용하지 않는다.
- 확장자를 임의로 `.ttf`/`.otf`로 바꾸거나 대소문자를 보정하지 않는다. pack에 적힌 정확한 파일명을
  쓴다.
- 안전한 이름은 `font_dirs` 순서대로 `dir.join(name)`을 확인해 **처음 존재하는 regular file**을 쓴다.
  production 순서는 이미 `System/Fonts`가 먼저이고 host checkout asset이 뒤다. 이 순서를 바꾸지 않는다.
- 언어팩 파일이 있는 `System/Lang`에서 폰트를 찾거나 임의 경로를 직접 열지 않는다.
- 찾은 preferred는 `push_lazy_file`로 등록한다. context 생성만으로 파일 bytes를 읽거나 fontdue parse를
  시작하지 않는다.
- missing/unsafe preferred는 등록하지 않고 기본 체인으로 진행한다. 진단을 추가한다면 이름과 경로는
  반드시 debug escaping하고 한 줄만 출력한다. 정상 fallback에 toast를 추가하지 않는다.
- corrupt regular file은 lazy slot으로 등록될 수 있다. 첫 glyph resolve 때 `slot2-text`의 기존 once-log
  및 다음 slot fallback 계약에 맡긴다. font parser나 `FontChain` 내부를 바꾸지 않는다.

### 3. 내장 한국어 metadata와 기존 CJK 계약

- `assets/lang/ko.ftl`의 `lang-font`를 실제 배포 파일명인
  `NotoSansKR-Regular.otf`로 고친다. 해당 i18n 테스트의 기대값도 함께 고친다.
- `CJK_FONT`는 그대로 `NotoSansKR-Regular.otf`다. 언어 preference가 없는 경우에도 게임 제목의
  한글·가나·한자는 기존처럼 CJK lazy fallback을 사용한다.
- 내장 OpenSans bytes와 Noto asset을 새로 추가·교체하지 않는다. Cargo dependency도 추가하지 않는다.
- 기존 splash 테스트가 특정 `FontId(1)`을 구현 순서로 가정해 깨지면, `ko`에서는 Noto가 preferred라
  앞 slot이 되는 새 의미를 검증하도록 최소 수정한다. 단언을 지우거나 렌더 검증을 약화하지 않는다.

## 구현 위치

- production 결정은 `crates/slot2-ui/src/lib.rs`의 `UiCtx::new`와 작고 private한 filename/path helper에
  둔다. i18n·text API에 App 전용 상태를 추가하지 않는다.
- `I18n`을 먼저 load해 effective pack을 확정한 뒤 그 `font()`로 chain을 만든다.
- preferred/CJK path 선택은 `font_dirs`의 caller 순서를 보존한다. preferred와 CJK dedup 비교에는 실제
  선택된 `PathBuf`를 사용하며 canonicalize나 filesystem mutation을 요구하지 않는다.
- public API 추가는 필요하지 않다. 테스트 전용 public accessor를 만들지 말고 기존 `FontChain`의
  `len`, `is_loaded`, `resolve`로 결과를 확인한다.

## 테스트 계약

새 `crates/slot2-ui/tests/language_font.rs`를 중심으로 실제 임시 `Lang`/`Fonts` 디렉터리와 현재 font
asset을 사용해 최소한 다음을 검증한다.

1. 내장 영어: preference 없음, OpenSans eager + CJK lazy 순서 유지, Latin은 OpenSans이고 한글은 CJK.
2. 내장 한국어: 수정된 `.otf`가 preferred로 lazy 등록되고 OpenSans는 eager fallback이며, 같은 Noto를
   CJK slot으로 중복 등록하지 않는다. Latin과 한글이 preferred Noto에서 resolve된다.
3. card-only 언어가 `lang-font = CardPreferred.ttf`를 지정하고 첫 font dir에 유효한 파일을 두면
   preferred가 첫 lazy slot이며 해당 glyph를 담당한다.
4. 같은 이름의 파일이 여러 font dir에 있으면 첫 dir의 파일이 선택된다. 서로 다른 glyph coverage를
   가진 기존 OpenSans/Noto asset 복사본으로 결과를 구분한다.
5. preference 없음, 존재하지 않는 이름, `/`와 `\`가 든 하위 경로, absolute/prefix, `.`과 `..`는
   기본 체인을 유지하고 바깥 파일을 읽지 않는다. 플랫폼별 path separator 차이에도 같은 정책이다.
6. 존재하지만 깨진 preferred font는 context 생성을 막지 않고, Latin은 OpenSans로, 한글은 CJK로
   fallback하며 panic하지 않는다.
7. malformed/unknown 요청이 내장 영어로 fallback하면 requested pack의 font가 아니라 effective 영어
   기본 체인을 쓴다.
8. Task90 runtime 언어 전환 테스트에 card-only preferred font 적용을 한 경로 보강한다. 전환 성공 뒤
   새 context가 preferred를 사용하고 이전 face cache가 남지 않으며, missing/corrupt preference도
   language load/save/swap 성공 자체를 막지 않는다.

테스트용 font는 기존 `assets/fonts` 파일을 임시 디렉터리에 복사한다. 새 binary font asset을 저장소에
추가하거나 실제 시스템 폰트를 참조하지 않는다. 임시 파일·디렉터리는 테스트 종료 시 정리한다.

## 수정 허용 파일

- `C:\SLOT2\crates\slot2-ui\src\lib.rs`
- `C:\SLOT2\crates\slot2-ui\tests\language_font.rs` (신규)
- `C:\SLOT2\crates\slot2-ui\tests\splash.rs`의 font-order 단언만
- `C:\SLOT2\crates\slot2\tests\language_picker_app.rs`의 runtime preferred font 테스트만
- `C:\SLOT2\assets\lang\ko.ftl`의 `lang-font` 파일명만
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`의 해당 기대값만
- `C:\SLOT2\docs\DESIGN.md` §8의 오래된 `font =` 표기를 실제 `lang-font` key로 바로잡는 부분만
- `C:\SLOT2\docs\DECISIONS.md` D-14의 오래된 `font=` 표기를 실제 `lang-font` key로 바로잡는 부분만
- `C:\SLOT2\tasks\91-language-pack-preferred-font.worker-result.md`

그 밖의 production·test·문서 파일은 수정하지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지
않는다.

## 범위 밖 및 금지

- `slot2-text` 내부, font parser/cache/rasterizer 변경
- font subset 생성 스크립트, 기존 font bytes 교체, 새 font asset·dependency 추가
- `I18n::font` 형식 또는 Fluent loader/fallback 변경
- Language picker/App 상태 전이, store, host/device loop 변경
- 한국어 문구 전수 교정, 정렬 정책, label renderer 변경
- M4/M5 checkbox 완료 처리
- 전체 workspace 테스트, 실제 GL 창, device 배포
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 네트워크와 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

아래를 **마지막 코드 변경 뒤** 순서대로 실행한다.

```text
cargo fmt --all -- --check
cargo test -p slot2-ui --test language_font
cargo test -p slot2-ui
cargo test -p slot2-i18n
cargo test -p slot2 --test language_startup --test language_picker_app
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2-ui -p slot2 --all-targets -- -D warnings
```

- 모든 명령 종료 코드 0.
- focused/전체 test result의 passed/failed/ignored 수를 각각 센다.
- core 의존 테스트 skip이 있으면 수와 이유를 쓴다.
- 완료 기준 뒤 코드 변경 금지. 변경했으면 영향받는 명령부터 다시 실행한다.

## 결과 보고서

`C:\SLOT2\tasks\91-language-pack-preferred-font.worker-result.md`에 다음만 간결하게 쓴다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- effective `I18n::font` source, 안전한 filename 판정과 font_dirs 우선순위 결과
- preferred→OpenSans→CJK 순서, lazy 상태와 preferred/CJK dedup 결과
- missing/unsafe/corrupt preferred fallback과 language 전환 성공 보존 결과
- 한국어 `.otf` metadata 및 기존 splash/runtime 회귀 결과
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored와 skip 수
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 남은 범위: subset 생성/실기 load 시간과 한국어·혼합 정렬 검토
- 계약이 틀려 보이는 부분이 있으면 그 내용
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
