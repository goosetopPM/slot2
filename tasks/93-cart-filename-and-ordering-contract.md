# Task 93 — 카트 파일명과 혼합 정렬 계약

현재 checkout에서 직접 작업한다. 카드 스캔의 한글·영문·일본어 혼합 정렬과 유효한 특수문자
파일명의 stem/부속 경로 보존을 자동 계약으로 고정하고, 이미 구현된 동작과 문서를 일치시킨다.

현재 `Card::scan`은 UTF-8 stem을 그대로 `Cart::stem`/`title`로 사용하고 Rust `str` 순서로 정렬한다.
이 태스크의 목적은 locale collation이나 파일명 변환 기능을 새로 만드는 것이 아니라, M5에서 요구한
정책의 정확한 경계를 테스트와 설계 문서로 봉인하는 것이다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

필요한 부분만 읽는다.

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\92-translation-contract-and-guide.result.md`
- `C:\SLOT2\docs\DESIGN.md`의 §8 정렬 한 줄과 §9 카드 레이아웃만
- `C:\SLOT2\docs\MILESTONES.md`의 M5만
- `C:\SLOT2\crates\slot2-store\src\card.rs`의 module 문서, `Cart`, `game_settings_path`,
  `cheat_path`, `label`, `scan`, `save_path`, `states_dir`만
- `C:\SLOT2\crates\slot2-store\src\platform.rs`의 extension 계약만
- `C:\SLOT2\crates\slot2-store\tests\card.rs`의 layout/scan/label/save 관련 테스트만

App/UI/session/core/i18n/font 내부, 다른 테스트 전체, 워커 로그와 저장소 이력은 읽지 않는다. 집중 테스트가
직접 깨질 때만 오류 위치 주변을 최소한으로 추가 확인한다.

## 확정할 정책

### 1. 혼합 정렬

- `Cart::title`의 Rust `str` 순서, 즉 Unicode scalar value 순서를 결정적으로 사용한다. UTF-8의
  lexicographic 순서는 scalar 순서를 보존한다.
- locale collation, 자연수 정렬, 대소문자 folding, 한글 초성 검색용 재배열을 하지 않는다.
- 따라서 ASCII 대문자 → ASCII 소문자 → 해당 code point의 가나·CJK·한글 순서가 그대로 드러난다.
  한글 완성형 음절 내부에서는 `가 < 각 < 나`가 성립한다.
- NFC/NFD 정규화를 하지 않는다. 파일시스템이 서로 다른 이름으로 제공한 두 UTF-8 stem은 서로 다른
  카트다. 테스트는 composed/decomposed 형태를 억지로 만들 필요가 없으며, production에 normalization
  dependency를 추가하지 않는다.
- 같은 title/stem의 복수 허용 확장자는 extension을 두 번째 key로 삼는 기존 deterministic tie-break를
  유지한다.

### 2. 파일명과 stem

- 스캔한 regular file의 마지막 허용 확장자 하나만 제거한다. 그 앞의 공백, 여러 점, 괄호, 대괄호,
  아포스트로피, `+`, `&`, `!`, `#`, `%`, `@`, `_`, `-`, 한글·가나·한자는 그대로 stem/title에
  보존한다.
- `._`를 포함해 `.`으로 시작하는 파일은 기존처럼 숨김/metadata 항목으로 보고 건너뛴다. directory와
  지원하지 않는 마지막 확장자도 건너뛴다.
- extension 판정만 ASCII 대소문자를 무시한다. stem의 대소문자와 Unicode spelling은 고치지 않는다.
- UTF-8로 표현할 수 없는 이름은 기존처럼 scan에서 건너뛴다. replacement character로 lossy 변환해
  서로 다른 파일을 충돌시키지 않는다. Windows에서 만들 수 없는 이름을 portable test로 흉내 내지
  않는다.
- `/`, `\`, NUL, Windows/VFAT 금지 문자, 끝의 점·공백처럼 대상 카드나 host가 애초에 허용하지 않는
  이름을 sanitize하거나 별도 이름으로 매핑하지 않는다. 이 태스크가 보장하는 것은 host와 카드에서
  유효한 UTF-8 파일명이다.

### 3. 부속 경로와 동일 stem

- exact stem을 다음 경로에 그대로 사용한다.
  - `Labels/<PLAT>/<stem>.png`
  - `Saves/<PLAT>/<stem>.sav`
  - `States/<PLAT>/<stem>/`
  - `System/games/<PLAT>/<stem>.ini`
  - `System/cheats/<PLAT>/<stem>.cht`
- 서로 다른 플랫폼의 같은 stem은 플랫폼 폴더로 격리한다.
- 같은 플랫폼에서 같은 stem을 가진 서로 다른 허용 확장자(예: `Twin.sfc`, `Twin.smc`)는 기존 카드
  레이아웃 호환을 위해 **카트 둘로 모두 표시하고 위 부속 경로를 공유한다**. 어느 하나를 숨기거나
  hash/확장자를 부속 경로에 더하지 않는다. 이 제한을 module 문서와 테스트에 명시한다.

## 자동 계약 테스트

`crates/slot2-store/tests/card_filename_contract.rs`를 새로 만들고, 서로 독립적인 작은 테스트로 다음을
검증한다. production dependency나 dev-dependency를 추가하지 않는다.

1. ASCII 대문자/소문자, 가나·CJK·한글을 섞은 ROM 목록이 위 scalar 순서로 나온다. 최소한
   `Alpha < alpha`, `가 < 각 < 나`를 직접 단언한다. directory 생성 순서와 무관함도 드러나야 한다.
2. portable한 특수문자를 모두 포함한 최소 두 이름을 사용해 마지막 확장자만 제거되고
   `Cart::stem == Cart::title`이며 원본 ROM path가 정확히 유지되는지 단언한다. 예:
   `10-in-1 [한글] + 日本語 (Rev A)!.gba`, `Alpha.beta's & more #50%@home.GBA`.
3. 그 special stem으로 label 탐색, save path, states dir, game settings path, cheat path가 exact stem을
   사용하며 root/platform 밖으로 벗어나지 않는지 단언한다. label은 실제 파일을 만들어 `Some(path)`를
   확인한다. save는 작은 byte round-trip까지 확인한다.
4. 숨김 이름, AppleDouble 이름, directory, unsupported final extension, `name.gba.bak`은 나타나지
   않고 지원 확장자의 ASCII case variant는 나타난다.
5. SNES의 동일 stem `.sfc`/`.smc` 두 파일은 모두 deterministic extension 순서로 나타나고, ROM path는
   서로 다르지만 label/save/states/settings/cheat path는 서로 같다는 legacy 계약을 명시적으로
   단언한다.
6. 같은 stem의 GB/GBA 카트는 모든 부속 경로가 플랫폼 폴더 때문에 다르다는 것을 단언한다.

테스트는 `std::env::temp_dir()` 아래 process-id와 test name을 포함한 고유 폴더를 쓰고 시작/끝에
best-effort 정리한다. 병렬 실행끼리 같은 폴더를 공유하지 않는다. OS별 금지 문자를 사용하거나
실행 순서·directory enumeration 순서에 의존하지 않는다.

기존 `card.rs` 테스트와 중복되는 단언은 새 파일에서 정책 전체를 한눈에 읽는 데 필요한 만큼만
허용한다. 기존 테스트를 약화하거나 지우지 않는다.

## production과 문서 변경

- 집중 테스트가 현재 구현에서 통과하면 `Card::scan`과 path production logic을 불필요하게 다시 쓰지
  않는다.
- 계약 불일치가 실제로 발견되면 위 정책을 만족하는 최소 수정만 허용한다. sanitize, locale library,
  normalization, metadata title DB, hash identity를 추가하지 않는다.
- `card.rs` module 문서의 정렬/스캔 설명을 위 정책과 같은 말로 정확히 다듬고, 같은 platform의 동일
  stem 복수 확장자가 부속 경로를 공유한다는 제한을 기록한다.
- `docs/DESIGN.md` §8 정렬 한 줄과 §9 카드 레이아웃 설명에 정책의 핵심만 간결하게 반영한다. 구현
  상세나 테스트 이름을 넣지 않는다.
- 모든 완료 기준이 통과한 뒤 `docs/MILESTONES.md` M5의
  `한·영 혼합 정렬 정책, 파일명 특수문자 처리` 한 항목만 `[x]`로 바꾼다.

## 수정 허용 파일

- `C:\SLOT2\crates\slot2-store\tests\card_filename_contract.rs` (신규)
- `C:\SLOT2\crates\slot2-store\src\card.rs` — module 문서, 필요 시 최소 scan/path 수정만
- `C:\SLOT2\docs\DESIGN.md` — §8 정렬과 §9 카드 filename/stem 설명만
- `C:\SLOT2\docs\MILESTONES.md` — 위 M5 checkbox 한 개만
- `C:\SLOT2\tasks\93-cart-filename-and-ordering-contract.worker-result.md`

그 밖의 production·test·문서 파일은 수정하지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지
않는다.

## 범위 밖 및 금지

- locale-aware collation, 자연수 정렬, 검색·필터 UI, title metadata DB
- Unicode normalization, 파일명 sanitizer/rename/migration, duplicate ROM 제거나 hash identity
- 카드 레이아웃 변경, 확장자를 save/state/settings key에 추가
- App/UI/session/core/i18n/font 변경
- Noto subset 생성, fontTools 설치, 네트워크 사용
- 전체 workspace 테스트, 실제 GL 창, device 배포
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 순서대로 실행한다.

```text
cargo fmt --all -- --check
cargo test -p slot2-store --test card_filename_contract
cargo test -p slot2-store --test card
cargo test -p slot2-store
cargo check -p slot2
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2-store -p slot2 --all-targets -- -D warnings
```

- 모두 종료 코드 0이어야 한다.
- 각 test command의 마지막 result 줄에서 passed/failed/ignored 수를 보고한다.
- 완료 기준 뒤 production/test 코드 변경 금지. 변경했으면 영향받는 명령부터 다시 실행한다.
- 문서만 마지막에 바꿨다면 코드 재실행은 필요 없지만 그 사실을 보고한다.

## 결과 보고서

`C:\SLOT2\tasks\93-cart-filename-and-ordering-contract.worker-result.md`에 다음만 간결하게 쓴다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- 최종 정렬 정책과 실제 테스트 순서
- special filename에서 보존한 stem/title/ROM 및 다섯 부속 경로 결과
- skip 대상과 extension case 처리 결과
- 동일 platform 동일 stem 복수 확장자의 표시 순서·공유 범위와 플랫폼 간 격리 결과
- production logic 변경 여부와 이유
- DESIGN/module 문서 및 M5 checkbox 반영 내용
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored 수
- 생성·수정 파일과 최종 검증 뒤 코드 변경 여부
- 계약이 틀려 보이는 부분이 있으면 그 내용
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
