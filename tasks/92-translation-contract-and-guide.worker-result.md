# Task 92 — 작업자 결과 (누적 호출 1/2)

## 결과

**성공 (1/2).** 완료 기준 7개 모두 종료 0이고, 내장 pack 계약 테스트·한국어 전수 감사·기여 가이드·
M5 반영을 모두 마쳤다. 코드·문구·문서 변경 뒤 영향받는 명령은 다시 돌리지 않았다(아래 "최종 검증 뒤
코드 변경 없음").

## pack 계약 결과 (en/ko)

- **key 수: en 125, ko 125.** 실제 canonical 수가 명세의 125와 일치해 이유를 적을 것이 없다.
- **duplicate 0**(양쪽), **ko에 missing 0 / extra 0**(key 집합 완전 일치).
- `$변수`: 125개 key 전부에서 en/ko의 변수 **집합이 동일**하다. 어순·사용 횟수는 언어마다 다르게
  두었다(예: `state-saved`는 en `Saved { $title } to slot { $n }`, ko `{ $n }번 슬롯에 { JOSA($title,
  "을/를") } 저장했습니다`).
- `BTN`: 모든 key에서 en/ko의 BTN id **목록(순서·중복 포함)이 동일**하고, 사용된 id는 `a`, `b`, `x`,
  `y`, `l1`, `r1`, `left`, `right`, `up`, `down`, `menu`로 전부 `Button::from_id`가 아는 값이다.
  unknown id 0.
- `JOSA`: en에는 0건. ko는 5개 key(`cart-inserted`, `cart-ejected`, `cart-broken`,
  `platform-switched`, `state-saved`)에서 사용하며, 첫 인자가 모두 그 문장의 변수이고 쌍은 `을/를`,
  `으로/로`로 런타임이 구현한 것이다. 조사가 필요한 변수 뒤에 고정 조사(`은/는`·`이/가`·`을/를`·
  `으로/로`)를 직접 붙인 곳은 없다(`{ $title }의`처럼 받침에 따라 변하지 않는 `의`는 허용).
- **literal cap 0건.** `[A]`/`[MENU]`/`[L1]` 같은 표기가 없고 전부 `BTN("...")`이다.
- 빈 값은 `en/lang-font = { "" }` 하나뿐이다.
- load: `I18n::embedded("en")`, `I18n::embedded("ko")`가 모두 parse/load되고, 125개 key를 각 언어에서
  format했을 때 `[key]` missing marker가 되는 key가 없다(기준 확인용으로 `no-such-key` → `[no-such-key]`
  를 함께 단언).

새 테스트가 비어 있지 않다는 증거로 변이 검사를 했다(임시 수정 후 원복, 최종 상태 재검증 완료):
en 변수 이름 변경 → 변수 테스트 실패, ko literal `[A]` → literal/버튼 테스트 실패, duplicate id 삽입 →
전 테스트 실패, ko key 삭제 → key 집합 테스트 실패, 미지원 함수 `MAGIC()` → 함수 테스트 실패.

## 한국어 전수 감사 결과

125개를 전부 확인했고, **사용자에게 보이는 문구는 한 줄도 바꾸지 않았다.** 용어(세이브 스테이트·코어·
셰이더·오버레이·오버스캔·시간대·이어하기·새로 시작)는 화면 간에 일관되고, toast/오류는 원인과 유지·
복구 결과를 정확히 말하며(`코어를 바꾸지 않았습니다`, `이전 코어로 돌아갔습니다`, `이전 값으로 돌아갔
습니다`, `이전 언어를 유지합니다`), 힌트는 캡 뒤 자연스러운 동작어를 두고, 제품명·코어명·경로·버전·
기기·code는 번역하지 않았다. 의미가 이미 정확한 문장을 다시 쓰지 않는다는 규칙에 따라 고친 것은
아래 두 곳뿐이다(둘 다 주석이며 화면에 나오지 않는다).

| 파일 | 고친 것 | 이유 |
| --- | --- | --- |
| `ko.ftl` | `core-missing`/`cart-broken` 위의 영어 주석 2줄을 한국어로 번역 | 이 파일에서 유일하게 영어로 남아 있던 주석. ko pack은 번역자가 그대로 읽고 따라 하는 참조 문서라 언어가 섞이면 안 된다 |
| `en.ftl` | 헤더의 `Variables:` 목록을 실제 변수 12개로 교정 | `$lang`은 어떤 문장에도 없고, 실제로 쓰이는 `$target`·`$panel`·`$version`·`$code`·`$current`·`$total`·`$value`·`$speed`·`$offset`이 빠져 있었다 |

## `docs/TRANSLATING.md`

기여자가 그대로 따라 할 수 있게 다음을 다룬다: 파일 위치와 en canonical / ko complete / 카드 partial
관계(카드가 같은 key를 덮어쓰고, 없는 key는 내장 → 영어 순), 새 카드 언어를 넣는 절차(코드 변경 불필요)
와 새 내장 언어를 넣는 절차(`assets/lang/` + `EMBEDDED` 한 줄), `lang-name` 자기 이름 규칙,
`lang-font`의 안전한 단일 파일명·탐색 순서(`System/Fonts` → 빌드 폰트)·없으면 기본 체인·지연 로딩,
문장 통째 번역·변수 이름 보존·어순 변경 허용, `BTN` 사용법과 **id→캡 표 18개**, literal 캡 금지,
`JOSA` 사용법·지원 7쌍·받침에 따라 변하지 않는 조사 구분·조사가 필요 없는 재문장 예시, `[key]` 영어
fallback / malformed pack load 실패 / 카드 override 우선순위, 번역자가 돌릴 검증 명령 두 줄, 폰트 OFL
고지 위치(`assets/fonts/*-OFL.txt` → 배포 `System/licenses/`)와 상용·무허가 폰트 반입 금지, 그리고
§9 "아직 없는 것"(GUI·자동 번역·locale 정렬·Noto subset·실기 검증)을 없는 그대로 적었다.

## M5 반영

`docs/MILESTONES.md` M5에서 다음 세 항목만 `[x]`로 닫았다.
- `ko.ftl` 전수 번역, `JOSA`·`BTN` 실사용, 문장형 문구 최소화 검토
- 언어팩 덮어쓰기(`System/Lang/`) + 폰트 지정 동작 확인 (Task88~91 startup/picker/runtime/font 검증 근거)
- 번역 기여 가이드(`docs/TRANSLATING.md`)

닫지 않은 항목: Noto Sans KR 서브셋 생성 스크립트·지연 로딩, 한·영 혼합 정렬 정책과 파일명 특수문자
처리, 시간대 설정 배선 줄, M5 Acceptance(실기 전 화면 순회)와 V-10.

## 완료 기준 명령 (마지막 코드 변경 뒤 순서대로 실행)

| # | 명령 | 종료 | 마지막 결과 줄 | passed/failed/ignored |
| --- | --- | --- | --- | --- |
| 1 | `cargo fmt --all -- --check` | 0 | (출력 없음) | — |
| 2 | `cargo test -p slot2-i18n --test pack_contract` | 0 | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` | 8 / 0 / 0 |
| 3 | `cargo test -p slot2-i18n` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | 42 / 0 / 0 (4줄: 34 + 8 + 0 + 0) |
| 4 | `cargo test -p slot2-ui` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | 263 / 0 / 0 (27줄) |
| 5 | `cargo test -p slot2` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | 335 / 0 / 0 (31줄) |
| 6 | `cargo check -p slot2 --no-default-features --features device` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.36s`` | — |
| 7 | `cargo clippy -p slot2-i18n -p slot2-ui -p slot2 --all-targets -- -D warnings` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 18.83s`` | warning 0줄 |

- core 의존 skip: 모든 result 줄이 `0 ignored`이고 런타임 skip 보고가 없다. core library가 필요한
  slot2-retro 테스트는 이 명령 범위가 아니다.
- 전체 workspace 테스트, 실제 GL 창, device 배포, 실기 접근은 하지 않았다.

## 생성·수정 파일

- `crates/slot2-i18n/tests/pack_contract.rs` (신규, 8 tests) — `EMBEDDED`의 source 문자열을 검사하는
  test-only scanner. regex/parser dependency 없이 `std`만 쓰고, production API·helper를 노출하지 않는다.
- `assets/lang/en.ftl` — 헤더 `Variables:` 주석 교정(문구 변경 없음).
- `assets/lang/ko.ftl` — 영어로 남아 있던 주석 번역(문구 변경 없음).
- `docs/TRANSLATING.md` (신규).
- `docs/MILESTONES.md` — M5 세 checkbox.
- `tasks/92-translation-contract-and-guide.worker-result.md` — 이 보고서.
- `crates/slot2-i18n/tests/i18n.rs`는 **수정하지 않았다**(문구를 바꾸지 않아 깨진 기대값이 없었다).
- **최종 검증 뒤 코드 변경 없음**: 완료 기준 실행 시작(09:33:40 KST)보다 모든 소스 mtime이 앞선다.
  변이 검사용 임시 `.bak` 파일도 남기지 않았고 출력 수집 폴더는 삭제했다. 위임·커밋·푸시·네트워크·
  실기·공용 설정 접근 없음.

## Noto subset

`pyftsubset`/fontTools가 이 host에 없어 재현 검증이 불가능하므로, 이번 범위에서 손대지 않고 별도
후속으로 남겼다(`docs/TRANSLATING.md` §9에도 "아직 없다"고 적었다). 네트워크 설치나 검증 생략으로
우회하지 않았다. M5의 서브셋 항목도 닫지 않았다.

## 계약이 틀려 보이는 부분

없다. 다만 계약 해석 두 가지와 한계를 적어 둔다.

1. `josa.rs`에는 지원 pair 목록을 공개하는 API가 없다. 계약이 production API 추가를 금지하므로 테스트가
   josa.rs의 규칙과 ko pack 헤더에 적힌 7쌍을 상수로 갖는다. josa.rs에 새 pair가 생기면 테스트 목록을
   사람이 함께 늘려야 한다(현재 사용 중인 `을/를`·`으로/로`는 검증된다).
2. 계약의 scanner 경계는 "들여쓴 continuation/select variant"만 message에 묶는다고 적었지만, 실제
   `states-count` select는 닫는 `}`가 들여쓰이지 않는다. 그 줄을 message의 continuation으로 처리했고
   (그러지 않으면 두 pack 모두 parse 실패로 보고된다), 그 밖의 비들여쓰기 줄은 여전히 실패로 보고한다.
3. 계약 단언 7(`t(key)`가 `[key]`가 아니다)만으로는 ko에 key가 **없는** 경우를 잡지 못한다(없으면 영어로
   fallback해 문장이 나온다). ko의 완전성은 단언 1의 key 집합 일치로 검증한다 — 두 단언이 함께 있어야
   "빠진 번역"이 잡힌다. 테스트에 두 검사를 모두 두었다.

## 소요 시간

약 18분(09:26–09:44 KST). 대부분 `cargo test -p slot2-ui`/`-p slot2` 실행(각 3–5분)과 변이 검사 6회였다.
