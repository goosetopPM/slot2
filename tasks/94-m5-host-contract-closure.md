# Task 94 — M5 호스트 계약 마감

현재 checkout에서 직접 작업한다. Task80~84에서 이미 완성된 시간대 기능을 마일스톤과 설계 문서에
정확히 반영하고, M5 Acceptance의 카드 `ja.ftl` 3문자열 시나리오를 실제 App 통합 테스트로 봉인하며,
Task93 이후 낡아진 번역 가이드의 파일명 정책 문구를 고친다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

필요한 부분만 읽는다.

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\80-global-timezone-settings-store.result.md`
- `C:\SLOT2\tasks\81-timezone-startup-app-wiring.result.md`
- `C:\SLOT2\tasks\82-timezone-menu-ui.result.md`
- `C:\SLOT2\tasks\84-shelf-timezone-app-wiring.result.md`
- `C:\SLOT2\tasks\90-language-picker-app-wiring.result.md`
- `C:\SLOT2\tasks\93-cart-filename-and-ordering-contract.result.md`
- `C:\SLOT2\docs\MILESTONES.md`의 M5와 M3 시계 HUD 한 줄만
- `C:\SLOT2\docs\DESIGN.md`의 §8 정렬, §9 마지막 시각 계약 문단만
- `C:\SLOT2\docs\TRANSLATING.md`의 §1 카드 pack, §8 실패 동작, §9만
- `C:\SLOT2\crates\slot2\tests\language_picker_app.rs`의 상단 helper/pack 상수와 discovery/card-only
  테스트만

production App/i18n/store/platform/UI/font 내부, 다른 테스트 전체, 워커 로그와 저장소 이력은 읽지 않는다.
집중 테스트가 직접 깨질 때만 해당 오류 주변을 최소한으로 추가 확인한다.

## 1. 카드 `ja.ftl` Acceptance 통합 테스트

`crates/slot2/tests/language_picker_app.rs`에 독립 test 하나를 추가한다. 기존 helper를 재사용하고 기존
테스트나 `CARD_PACK` 의미를 바꾸지 않는다.

카드의 `System/Lang/ja.ftl`에는 정확히 다음 **3개 Fluent message**만 둔다. 주석·빈 줄은 message로
세지 않는다.

```ftl
lang-name = 日本語
power-off = 電源を切る
resume = 続ける
```

테스트는 다음 흐름을 실제 App 경계에서 순서대로 단언한다.

1. 영어로 시작한 shelf에서 Language picker를 연다.
2. picker option code가 `en`, `ja`, `ko`의 기존 simple code 순서이고, `ja`의 self-name은 `日本語`다.
3. `ja`를 highlight하고 A를 눌러 one-shot request를 만든 뒤 기존 `service_language_request`로 처리한다.
4. 처리 뒤 `UiCtx` effective code, `App::current_language`, `Card::read_language`가 모두 `ja`다.
5. 위 두 사용자 문구는 카드의 일본어이고, pack에 없는 최소 한 canonical key는 내장 영어로 fallback한다.
6. 화면은 기존 성공 계약대로 parent Shelf로 돌아가고 load/save failure toast가 없다.

App 내부를 직접 호출해 picker discovery를 우회하거나 `I18n::available`만 단독 검사해서 Acceptance를
대체하지 않는다. 새 production hook, dependency, 별도 대형 fixture를 만들지 않는다.

## 2. 시간대 완료 표기

Task80~84 결과를 근거로 다음 문서만 현재 구현과 맞춘다.

- `docs/MILESTONES.md` M5의 `시간대 설정` 한 항목을 `[x]`로 바꾸고 끝에
  `Task80~84: store, startup, UI, 즉시 preview·저장·실패 rollback` 정도의 짧은 근거를 남긴다.
- 같은 파일 M3의 HUD 줄에서 “시간대 배선 전까지 UTC”라는 오래된 미래형 설명을 제거하고, 현재는
  카드 설정의 표시 offset을 쓴다는 사실만 짧게 적는다.
- `docs/DESIGN.md` §9의 “M5 미구현” 문장을 현재 계약으로 바꾼다: `System/slot2.ini`의
  `utc_offset_minutes`, 기본 0, startup 적용, 메뉴 preview, apply/cancel, safe-write 실패 rollback,
  system clock/mtime UTC 유지. 구현 상세나 테스트 개수는 쓰지 않는다.

production clock/store/App 코드는 바꾸지 않는다. boot 진단 로그의 초기 표시 시점은 이 태스크의
기능 계약이 아니므로 손대거나 완료됐다고 적지 않는다.

## 3. 번역 가이드의 오래된 정책 정리

`docs/TRANSLATING.md` §9에서 “파일명 특수문자 정책은 정해지지 않았다”를 제거한다.

- 언어 **목록**은 locale collation 없이 기존 simple code 순서라는 사실을 유지한다.
- ROM 제목/파일명의 정렬과 특수문자 정책은 `DESIGN.md` §8~§9를 따른다고 요약한다: Unicode scalar
  value 순서, 마지막 허용 확장자 하나만 제거, 유효한 UTF-8 stem의 문자 exact 보존, normalization
  없음.
- 번역 pack 파일명 규칙(`System/Lang/<code>.ftl`)과 ROM 파일명 규칙을 섞지 않는다.
- Noto subset script가 아직 없고 실기 전 화면 확인이 남았다는 두 줄은 유지한다.

## 4. 마일스톤 Acceptance 표기

`docs/MILESTONES.md` M5 Acceptance의 카드 `ja.ftl` 줄 끝에 이 태스크의 host App 통합 테스트로 자동
검증된다는 짧은 근거를 붙인다. 실기에서 실제 글꼴·화면을 확인했다고 쓰지 않는다.

한국어 전 화면 순회 Acceptance는 그대로 남긴다. Noto subset 항목과 V-10도 닫지 않는다.

## 수정 허용 파일

- `C:\SLOT2\crates\slot2\tests\language_picker_app.rs` — 위 test 하나만
- `C:\SLOT2\docs\MILESTONES.md` — M3 HUD 한 줄, M5 시간대 한 줄, ja Acceptance 한 줄만
- `C:\SLOT2\docs\DESIGN.md` — §9 시각 계약 문단만
- `C:\SLOT2\docs\TRANSLATING.md` — §9의 정렬/파일명 bullet만
- `C:\SLOT2\tasks\94-m5-host-contract-closure.worker-result.md`

그 밖의 production·test·문서 파일은 수정하지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지
않는다.

## 범위 밖 및 금지

- production App/i18n/store/platform/UI/font 변경
- Noto subset 생성, font asset 교체, fontTools 설치, 네트워크 사용
- locale-aware language sorting, ROM sorting 변경, 파일명 sanitizer/normalization
- boot 진단 로그 변경
- 한국어 전 화면 또는 글꼴 실기 확인을 완료로 표시
- M4 또는 M5의 다른 checkbox 변경
- 전체 workspace 테스트, 실제 GL 창, device 배포
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 순서대로 실행한다.

```text
cargo fmt --all -- --check
cargo test -p slot2 --test language_picker_app
cargo test -p slot2 --test timezone_startup_app
cargo test -p slot2 --test timezone_menu_app
cargo test -p slot2-i18n
cargo test -p slot2-store --test timezone_settings
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2 -p slot2-i18n -p slot2-store --all-targets -- -D warnings
```

- 모두 종료 코드 0이어야 한다.
- 각 test command의 마지막 result 줄에서 passed/failed/ignored 수를 보고한다.
- 신규 test가 실제로 `ja.ftl`의 세 message, picker 표시, 선택·저장, 영어 fallback을 모두 통과해야 한다.
- 완료 기준 뒤 code/test 변경 금지. 변경했으면 영향받는 명령부터 다시 실행한다.
- 완료 기준 뒤 문서만 바꿨다면 코드 재실행은 필요 없지만 그 사실을 보고한다.

## 결과 보고서

`C:\SLOT2\tasks\94-m5-host-contract-closure.worker-result.md`에 다음만 간결하게 쓴다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- `ja.ftl`의 정확한 3 message와 picker order/self-name 결과
- 선택 후 effective/App/card code, 카드 번역 두 개, 영어 fallback, screen/toast 결과
- 시간대 완료 근거와 MILESTONES/DESIGN에서 바로잡은 문구
- TRANSLATING의 language list와 ROM filename 정책 구분 결과
- 닫은 M5 checkbox와 계속 남긴 Noto subset·V-10·실기 Acceptance
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored 수
- 생성·수정 파일과 최종 검증 뒤 code/test 변경 여부
- 계약이 틀려 보이는 부분이 있으면 그 내용
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
