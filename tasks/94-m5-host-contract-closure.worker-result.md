# Task 94 — 작업자 결과 (누적 호출 1/2)

## 결과

**성공 (1/2).** 완료 기준 8개 모두 종료 0. 신규 Acceptance 테스트 1개가 통과하고(11 tests), 시간대
완료 근거를 MILESTONES·DESIGN에 반영했고, 번역 가이드의 낡은 파일명 정책 문구를 교체했다.
production 코드는 한 줄도 바꾸지 않았다.

## 카드 `ja.ftl` Acceptance 결과

카드 pack의 message는 정확히 셋이다: `lang-name = 日本語`, `power-off = 電源を切る`,
`resume = 続ける`.

신규 테스트 `a_card_only_japanese_pack_is_offered_chosen_and_running`가 실제 App 경계에서 순서대로
단언한 것:

- picker option code가 `en`, `ja`, `ko`의 단순 code 순서이고, `ja` 행의 self-name은 `日本語`다
  (내장 en/ko가 먼저, 카드 pack이 그 사이 순서에 들어온다).
- `ja`를 highlight하고 A로 one-shot request를 만든 뒤 두 backend가 쓰는 공용
  `service_language_request`로 처리했다.
- 처리 뒤 `UiCtx` effective code, `App::current_language`, `Card::read_language`가 모두 `ja`.
- 카드가 정의한 `power-off`→`電源を切る`, `resume`→`続ける`, 정의하지 않은 canonical key
  `shelf-language`→`Language`(내장 영어) fallback.
- 화면은 성공 계약대로 parent Shelf(Language 행)로 복귀하고 `toast_key()`는 `None`(load/save failure
  toast 없음).

picker discovery를 우회하지 않았고 `I18n::available` 단독 검사가 아니며, production hook·dependency·
대형 fixture를 추가하지 않았다. 기존 테스트와 `CARD_PACK`은 손대지 않았다.

## 시간대 완료 근거와 바로잡은 문구

근거: Task80(store `utc_offset_minutes`, 기본 0, 범위 -720..=840, 무효값 UTC 0, atomic ini, volume·
unknown key 보존), Task81(startup에서 정확히 한 번 적용, compile-time 범위 계약), Task82(menu UI,
±15/±60분 clamp, safe-area draw, pack 6 key), Task84(Shelf 진입, 즉시 preview, A 저장·B 취소, 저장
실패 시 original rollback + `timezone-save-failed` toast) — 네 판정 모두 통과.

바로잡은 문구:

- MILESTONES M5 `시간대 설정`: `[ ]`→`[x]`, 끝에 `Task80~84: store, startup, UI, 즉시 preview·저장·
  실패 rollback` 근거 추가. 같은 줄에 남아 있던 미래형 단정 `BaseOS는 환경변수 없이 System/frontend를
  exec 하므로(DESIGN §2) 이 배선 전까지 실기 시계는 UTC다`는 이제 사실이 아니므로 그 절만 삭제하고,
  `잘못된 값의 처리도 명세한다`를 현재 사실인 `범위 밖·parse 실패도 UTC 0으로 처리한다`로 바꿨다
  (D-25의 system clock/저장 시각 UTC 유지 문구와 `SLOT2_UTC_OFFSET_MIN` 설명은 유지).
- MILESTONES M3 HUD 줄: `M5 설정 파일·시간대 화면 배선 전까지 실기 시계는 UTC로 보인다(D-25)` →
  `HUD 시계는 카드 System/slot2.ini의 표시 offset을 쓴다(D-25)`.
- DESIGN §9 시각 계약: `System/slot2.ini의 시간대 읽기·쓰기 배선은 M5 미구현 항목이다.` →
  `표시 offset의 영속 값은 System/slot2.ini의 utc_offset_minutes(기본 0) 하나이며, 시작 시 한 번
  적용되고 설정 메뉴에서 방향 입력으로 즉시 preview·A 적용·B 취소되고 저장 실패 시 이전 값으로
  되돌아간다.` (구현 상세·테스트 수 없음).

boot 진단 로그의 초기 표시 시점은 계약대로 손대지 않았고 완료로도 적지 않았다.

## 번역 가이드 정리

`docs/TRANSLATING.md` §9에서 “파일명 특수문자 정책은 정해지지 않았다”를 제거하고:

- 언어 **목록**은 locale collation 없이 언어 코드의 단순 순서라는 사실 유지(그리고 이 문서의
  `System/Lang/<code>.ftl` 파일명 규칙은 번역 pack의 것이며 ROM 파일명 규칙과 별개라고 명시).
- ROM 제목/파일명 정책은 `DESIGN.md` §8~§9를 정본으로 요약: 제목은 로케일 collation·자연수 정렬·
  대소문자 folding 없이 Unicode scalar value 순서, 파일명은 마지막 허용 확장자 하나만 제거, 유효한
  UTF-8 stem 문자 exact 보존, 정규화·sanitize·rename 없음.
- Noto subset 스크립트 부재와 실기 전 화면 확인이 남았다는 두 줄은 그대로 유지.

## M5 표기

- 닫은 항목: 시간대 설정(Task80~84 근거).
- Acceptance `카드에 System/Lang/ja.ftl(테스트용 3문자열)만 넣어도 언어 목록에 나타남`에
  `— host App 통합 테스트(crates/slot2/tests/language_picker_app.rs)로 자동 검증` 근거를 붙였다.
  실기 글꼴·화면 확인을 했다고 쓰지 않았고, 한국어 전 화면 순회 Acceptance도 그대로 두었다.
- 계속 남긴 것: Noto Sans KR 서브셋 생성 스크립트·지연 로딩, V-10 실기 로딩 시간·메모리, 실기 전
  화면 한국어 순회.

## 완료 기준 명령 (마지막 코드 변경 뒤 순서대로 실행)

| # | 명령 | 종료 | 마지막 result 줄 | passed/failed/ignored |
| --- | --- | --- | --- | --- |
| 1 | `cargo fmt --all -- --check` | 0 | (출력 없음) | — |
| 2 | `cargo test -p slot2 --test language_picker_app` | 0 | `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 36.12s` | 11 / 0 / 0 |
| 3 | `cargo test -p slot2 --test timezone_startup_app` | 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.65s` | 1 / 0 / 0 |
| 4 | `cargo test -p slot2 --test timezone_menu_app` | 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s` | 1 / 0 / 0 |
| 5 | `cargo test -p slot2-i18n` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (doc-tests) | 42 / 0 / 0 (4줄) |
| 6 | `cargo test -p slot2-store --test timezone_settings` | 0 | `test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.52s` | 12 / 0 / 0 |
| 7 | `cargo check -p slot2 --no-default-features --features device` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.77s`` | — |
| 8 | `cargo clippy -p slot2 -p slot2-i18n -p slot2-store --all-targets -- -D warnings` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.99s`` | warning 0줄 |

core 의존 skip 없음(모든 result 줄 `0 ignored`). 전체 workspace 테스트, 실제 GL 창, device 배포, 실기
접근은 하지 않았다.

## 생성·수정 파일

- `crates/slot2/tests/language_picker_app.rs` — 신규 test 1개(11번째)만 추가.
- `docs/MILESTONES.md` — M3 HUD 한 줄, M5 시간대 한 줄, M5 ja Acceptance 한 줄.
- `docs/DESIGN.md` — §9 시각 계약 문단의 마지막 문장.
- `docs/TRANSLATING.md` — §9의 정렬/파일명 bullet.
- `tasks/94-m5-host-contract-closure.worker-result.md` — 이 보고서.
- **최종 검증 뒤 code/test 변경 없음**: 완료 기준 실행 시작(10:48:18 KST)보다 모든 파일 mtime이
  앞선다. 문서는 그 전에 수정을 마쳤다. 출력 수집 폴더는 삭제했다. 위임·커밋·푸시·네트워크·실기·
  공용 설정 접근 없음.

## 계약이 틀려 보이는 부분

1. 명세는 M5 시간대 줄에서 `[x]` 전환과 근거 추가만 지시했지만, 그 줄 안에 “이 배선 전까지 실기
   시계는 UTC다”라는 **이미 거짓이 된 미래형 단정**이 남아 있었다. 완료 항목에 거짓 문장을 남길 수
   없어 같은 줄 안에서 그 절만 삭제하고 잘못된 값 처리 문구를 현재 사실로 바꿨다(다른 줄·다른
   checkbox는 건드리지 않았다).
2. 명세가 요구한 clippy 통과를 위해 신규 테스트의 `Option<&str>` 비교에서 `as_deref()`를 제거했다
   (`needless_option_as_deref`). 동작·단언 의미는 같다.

## 소요 시간

약 14분(10:35–10:49 KST). 대부분 `-p slot2` 빌드/테스트 대기와 clippy 재실행 1회였다.
