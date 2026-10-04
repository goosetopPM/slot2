# Task 92 — 번역 pack 계약과 기여 가이드

현재 checkout에서 직접 작업한다. 내장 `en.ftl`과 `ko.ftl`의 전체 key/variable/button 계약을 자동으로
검증하고, 한국어 문구를 전수 감사하며, 카드 언어팩과 번역 기여자가 따라야 할 규칙을
`docs/TRANSLATING.md`로 문서화한다.

Task91 다음 작업으로 계획했던 Noto subset은 현재 host에 `pyftsubset`/fontTools가 없어 재현 검증할 수
없다. 네트워크 설치나 검증 생략으로 우회하지 않고 별도 후속으로 남긴다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

필요한 부분만 읽는다.

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\91-language-pack-preferred-font.result.md`
- `C:\SLOT2\docs\DECISIONS.md`의 D-12~D-14만
- `C:\SLOT2\docs\DESIGN.md`의 §8 다국어 부분만
- `C:\SLOT2\docs\MILESTONES.md`의 M5만
- `C:\SLOT2\assets\lang\en.ftl`, `ko.ftl`
- `C:\SLOT2\crates\slot2-i18n\src\lib.rs`의 public pack/fallback/font 계약만
- `C:\SLOT2\crates\slot2-i18n\src\button.rs`의 유효한 BTN id/label 계약만
- `C:\SLOT2\crates\slot2-i18n\src\josa.rs`의 지원 조사 pair 공개 계약만
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`의 pack별 문구·BTN/JOSA 테스트 목차와 관련 단언만

UI layout/renderer, App/session/core/store, font 내부, 다른 테스트 전체, 워커 로그와 저장소 이력은 읽지
않는다. 문구 변경으로 직접 깨진 i18n/UI 단언만 위치를 찾아 최소 수정할 수 있다.

## 내장 pack 계약

### 1. 기준과 key

- `en.ftl`이 canonical key 목록이다. `ko.ftl`은 metadata를 포함해 같은 message id를 정확히 한 번씩
  가져야 한다.
- 현재 기준은 en/ko 각각 **125개 key**다. 작업 중 실제 canonical 수가 달라졌다면 보고서에 이유와 최종
  수를 적는다. 숫자에 맞추기 위한 가짜 key나 padding은 금지한다.
- 빈 값은 `en`의 `lang-font = { "" }`만 허용한다. 다른 내장 값은 사람이 읽을 수 있는 비어 있지 않은
  번역이어야 한다.
- card-only/override pack은 일부 key만 가져도 된다. 내장 `ko`의 완전성 계약을 카드 pack에 강제하지
  않는다. 누락 key는 기존 영어 fallback 계약을 유지한다.

### 2. 변수와 버튼

- 같은 key의 en/ko는 `$variable` 이름의 **집합**이 같아야 한다. 어순과 사용 횟수는 언어가 정하므로
  같을 필요가 없지만, 번역이 변수를 잃거나 새 변수를 발명하면 실패한다.
- 같은 key의 en/ko는 `BTN("id")` id의 순서와 중복을 포함한 목록이 같아야 한다. 버튼 위치는 달라도
  되지만 다른 버튼으로 바꾸거나 literal `[A]`, `[MENU]` 등으로 대체하면 안 된다.
- 모든 BTN id는 `button.rs`가 아는 id여야 한다. unknown id를 보이게 렌더하는 runtime 안전망은
  card pack용이며, 내장 pack의 오타를 허용하는 장치가 아니다.
- `JOSA()`는 한국어 pack만 사용할 수 있고, 첫 인자는 해당 key에 이미 존재하는 변수여야 하며 두 번째
  인자는 `josa.rs`가 지원하는 pair여야 한다. 영어에 JOSA를 넣지 않는다.
- 한국어가 조사 선택이 필요한 변수 바로 뒤에 고정 조사(`은/는`, `이/가`, `을/를`, `으로/로` 등)를
  붙이지 않는다. 필요한 곳은 JOSA를 사용한다. 조사 자체가 필요 없는 자연스러운 재문장은 허용한다.

### 3. 문구 감사

`ko.ftl` 125개를 전수 확인한다.

- 메뉴/행/상태는 짧고 같은 개념에 같은 용어를 쓴다: 세이브 스테이트, 코어, 셰이더, 오버레이,
  오버스캔, 시간대, 이어하기, 새로 시작.
- toast/오류는 원인과 유지·복구 결과를 왜곡하지 않는다. 영어를 직역해 부자연스럽게 늘이지 않는다.
- 버튼 힌트는 버튼 cap 뒤에 자연스러운 동작어를 둔다. 코드가 문장 조각을 이어 붙이게 만들지 않는다.
- 제품명, core 이름, 경로, version/target/code 같은 기술 식별자는 번역하지 않는다.
- 의미가 이미 자연스럽고 정확하면 불필요하게 다시 쓰지 않는다. 문구 변경은 감사에서 발견한 구체적
  문제만 고친다.

## 자동 계약 테스트

새 `crates/slot2-i18n/tests/pack_contract.rs`를 만든다. production API나 Cargo dependency를 추가하지
말고 `slot2_i18n::EMBEDDED`의 source 문자열을 검사하는 작은 test-only parser/scanner를 사용한다.

### scanner 경계

- Fluent top-level message 시작(`id =`)과 그 뒤의 들여쓴 continuation/select variant를 한 message로
  묶는다. comment와 빈 줄은 값으로 세지 않는다.
- id는 현재 프로젝트 규칙인 ASCII 소문자 시작 + ASCII 소문자/숫자/`-`만 허용한다.
- 동일 pack의 duplicate id를 거부한다.
- `$variable`은 `$` 뒤의 Fluent identifier를 char 단위로 읽는다. set 비교하되 진단은 key와 양쪽
  목록을 정렬해 보여 준다.
- `BTN("...")`와 `JOSA(...)`는 whitespace를 허용해 추출한다. 단순 substring count로 우연한 comment나
  일반 문자열을 함수 호출로 오인하지 않는다.
- 새 regex/parser dependency를 추가하지 않는다. 이 프로젝트의 단순한 내장 pack 문법만 명시적으로
  다루고, scanner가 이해하지 못하는 함수 형태는 조용히 건너뛰지 말고 해당 key와 함께 실패한다.

### 최소 단언

1. en/ko key set이 같고 각 pack에 duplicate가 없으며 현재 125개다.
2. 허용된 빈 `en/lang-font` 외 모든 값이 비어 있지 않다.
3. 모든 key의 variable set이 en/ko에서 같다.
4. 모든 key의 BTN id 목록이 en/ko에서 같고 각각 유효한 built-in button id다.
5. en에는 JOSA가 없고, ko의 모든 JOSA 변수/pair가 유효하다.
6. 내장 pack에 literal button cap 표기(`[A]`, `[B]`, `[MENU]`, `[L1]`, `[R1]`, `[X]`, `[Y]` 등)가
   없다. 사용자에게 보이는 대괄호 일반 문구까지 광범위하게 금지하지 않는다.
7. `I18n::embedded("en")`와 `I18n::embedded("ko")`가 계속 parse/load되고, 모든 canonical key를
   각 언어에서 format했을 때 `[key]` missing marker가 되지 않는다. 인자가 필요한 message는 source
   계약 검사로 다루고 기존 `i18n.rs`의 concrete format 테스트를 유지한다.

테스트 helper가 production에 노출되거나 실제 card pack의 partial fallback을 막아서는 안 된다.

## `docs/TRANSLATING.md`

한국어로 간결하게 작성하되 외부 번역자가 그대로 따라 할 수 있어야 한다. 최소한 다음을 포함한다.

- Fluent UTF-8 파일 위치, en canonical/ko complete/card partial 관계
- 새 내장 언어와 카드 전용 `<code>.ftl`을 추가하는 방법 및 `System/Lang`, `System/Fonts` 경로
- `lang-name`은 자기 언어 이름, `lang-font`는 안전한 단일 파일명이며 없으면 기본 chain이라는 규칙
- 전체 문장 단위 번역, 변수 이름 보존, 어순 변경 허용, 문자열 조각 결합 금지
- BTN 사용법과 유효 id 표, literal cap 금지
- JOSA 사용법, 지원 pair, 필요 없는 문장은 자연스럽게 재작성 가능하다는 예시
- missing key의 영어 fallback, malformed pack의 load 실패, card override 우선순위
- 번역자가 실행할 정확한 검증 명령
- 폰트 OFL 고지 위치와 상용/무허가 font를 저장소에 넣지 않는 규칙

문서에 아직 없는 GUI, 자동 번역, locale sorting, font subset 완료나 실기 검증을 있는 것처럼 쓰지 않는다.

## 마일스톤 반영

모든 완료 기준이 통과한 뒤 `docs/MILESTONES.md` M5에서 다음만 `[x]`로 바꾼다.

- `ko.ftl` 전수 번역, JOSA·BTN 실사용, 문장형 문구 최소화 검토
- 언어팩 덮어쓰기(`System/Lang/`) + 폰트 지정 동작 확인 — Task88~91의 startup/picker/runtime/font
  검증 근거가 있으므로 닫는다.
- 번역 기여 가이드(`docs/TRANSLATING.md`)

Noto subset, 혼합 정렬/특수문자와 M5 Acceptance는 닫지 않는다.

## 수정 허용 파일

- `C:\SLOT2\assets\lang\en.ftl` — 감사에서 발견한 명확한 문구/주석 문제만
- `C:\SLOT2\assets\lang\ko.ftl` — 감사에서 발견한 명확한 번역/계약 문제만
- `C:\SLOT2\crates\slot2-i18n\tests\pack_contract.rs` (신규)
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs` — 문구 변경으로 직접 깨진 기대값만
- `C:\SLOT2\docs\TRANSLATING.md` (신규)
- `C:\SLOT2\docs\MILESTONES.md`의 위 세 checkbox만
- `C:\SLOT2\tasks\92-translation-contract-and-guide.worker-result.md`

그 밖의 production·test·문서 파일은 수정하지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지
않는다.

## 범위 밖 및 금지

- i18n production loader/fallback/JOSA/BTN 구현과 public API 변경
- UI/App/store/font code, Cargo/Cargo.lock, 새 dependency 변경
- Noto subset 생성, font asset 교체, fontTools 설치 또는 네트워크 사용
- locale-aware sorting, 파일명 정규화/특수문자 정책 구현
- 전체 workspace 테스트, 실제 GL 창, device 배포
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 순서대로 실행한다.

```text
cargo fmt --all -- --check
cargo test -p slot2-i18n --test pack_contract
cargo test -p slot2-i18n
cargo test -p slot2-ui
cargo test -p slot2
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2-i18n -p slot2-ui -p slot2 --all-targets -- -D warnings
```

- 모두 종료 코드 0이어야 한다.
- 각 test command의 result 줄 합계로 passed/failed/ignored 수를 센다.
- core 의존 skip이 있으면 수와 이유를 쓴다.
- 최종 en/ko key 수와 실제로 고친 문구 목록을 보고한다.
- 완료 기준 뒤 코드 변경 금지. 변경했으면 영향받는 명령부터 다시 실행한다.

## 결과 보고서

`C:\SLOT2\tasks\92-translation-contract-and-guide.worker-result.md`에 다음만 간결하게 쓴다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- 최종 en/ko key 수, duplicate/missing/extra 결과
- variable/BTN/JOSA/literal cap 계약 결과
- 한국어 전수 감사 결과와 실제 변경 문구 및 이유
- `docs/TRANSLATING.md`가 다루는 기여 흐름
- M5에서 닫은 세 checkbox와 남긴 항목
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored와 skip 수
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- Noto subset이 fontTools 부재로 별도 후속이라는 점
- 계약이 틀려 보이는 부분이 있으면 그 내용
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
