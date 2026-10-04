# Task 81 — 저장 시간대의 App 시작 적용

현재 checkout에서 직접 작업한다. Task80이 `System/slot2.ini`에서 읽은 표시용 UTC 오프셋을
`App::with_card` 시작 경로에서 `slot2_platform::clock`에 적용해 HUD와 이후 runtime clock 읽기가 카드
설정을 사용하게 한다. 이번 태스크는 **시작 시 load/apply만** 다룬다. 시간대 선택 UI, runtime 변경과
저장은 후속 태스크로 분리한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\81-timezone-startup-app-wiring.md`
- `C:\SLOT2\tasks\80-global-timezone-settings-store.result.md`
- `C:\SLOT2\tasks\27-clock-runtime.result.md`
- `C:\SLOT2\docs\DECISIONS.md`의 D-25만
- `C:\SLOT2\docs\MILESTONES.md`의 M5 시간대 항목만
- `C:\SLOT2\crates\slot2-store\src\settings.rs`의 시간대 상수와 Card read API만
- `C:\SLOT2\crates\slot2-platform\src\clock.rs`의 offset 상수, getter/setter와 HUD 변환 부분만
- `C:\SLOT2\crates\slot2\src\app.rs`의 `App::new`, `App::with_card`와 HUD clock draw 부분만
- `C:\SLOT2\crates\slot2\tests\global_volume_app.rs`의 임시 Card/App 생성 helper만
- `C:\SLOT2\crates\slot2\tests\hud_app.rs`의 App/HUD helper와 clock 관련 단언만

host/device loop, Session/core, 메뉴/UI 구현, 다른 App action, 워커 로그와 저장소 이력은 읽지 않는다.
직접 깨지는 App constructor 테스트만 위치를 찾기 위해 추가로 읽을 수 있다.

## 현재 계약과 결정

- `Card::read_utc_offset_minutes()`는 key 부재·파일 부재·invalid/unreadable을 기본 0으로 돌리고, 정상
  값은 `-720..=840`만 반환한다. App은 ini를 다시 parse하거나 별도 fallback 규칙을 만들지 않는다.
- `slot2_platform::clock::set_utc_offset_min`은 같은 범위의 값을 process-global atomic에 적용한다.
  `now_local()`과 App HUD는 이 runtime offset을 읽는다. 시스템 시각은 UTC로 유지된다.
- `App::with_card`는 host/device 실행과 명시적 Card를 쓰는 테스트가 공유하는 생성 경로다. 적용은 이
  한 곳에서 한다. host/device loop나 HUD draw마다 파일을 다시 읽지 않는다.
- 카드 설정이 runtime 값보다 우선한다. key가 없거나 invalid이면 store의 기본 0을 적용한다. 따라서
  이미 초기화된 환경변수/이전 runtime 값이 있더라도 App 생성 뒤에는 카드가 돌려준 값이 현재값이다.
- `SLOT2_UTC_OFFSET_MIN`은 App 배선 전 개발용 초기값이었다. 이 태스크는 환경변수를 읽거나 변경하지
  않는다. 카드가 없는 독립 `slot2-platform` 사용자는 기존 환경 fallback을 계속 쓸 수 있다.
- clock offset은 process-global이므로 같은 test binary 안에서 서로 다른 값을 쓰는 여러 테스트를
  병렬 실행하면 결과가 섞인다. 새 integration test는 **한 개의 test 함수 안에서 순차적으로** 모든
  시나리오를 검증하고 마지막에 이전 값을 복원한다.

## 구현 계약

### 1. 범위 계약을 compile-time으로 봉인

- `slot2` App 경계에서 다음 일치를 compile-time assertion으로 검증한다.
  - `slot2_store::UTC_OFFSET_MINUTES_MIN == slot2_platform::clock::OFFSET_MIN`
  - `slot2_store::UTC_OFFSET_MINUTES_MAX == slot2_platform::clock::OFFSET_MAX`
  - `slot2_store::DEFAULT_UTC_OFFSET_MINUTES == 0`이며 platform setter가 받는 범위 안
- test-only 단언만 두지 않는다. 한쪽 상수가 미래에 바뀌면 production `slot2` compile이 실패해 카드가
  받아들인 값을 clock이 거부하는 조합이 출하되지 않아야 한다.
- 두 크레이트의 상수나 범위를 이번 태스크에서 고쳐 맞추지 않는다. 현재 값이 다르면 계약 오류로
  보고하고 실패로 둔다.

### 2. App 시작 시 한 번 적용

- `App::with_card`가 struct를 만들기 전에 `card.read_utc_offset_minutes()`를 정확히 한 번 읽고
  `slot2_platform::clock::set_utc_offset_min`으로 적용한다.
- `App::new`도 기존처럼 `with_card`를 통과하므로 같은 동작을 얻는다. 별도 초기화 경로를 만들지 않는다.
- 저장된 0, 양수, 음수와 양 경계값을 그대로 적용한다.
- missing key/file, invalid text, 범위 밖 text, invalid UTF-8는 store가 반환한 0을 적용한다. App이 원본을
  고치거나 새 파일을 만들지 않는다.
- setter가 예상 밖으로 값을 거부하면 panic/unwrap하지 않는다. 한 줄 stderr로 rejected value를 남기고
  기본 0 적용을 시도해 이전 App의 offset이 남지 않게 한다. compile-time range assertion 때문에 정상
  source에서는 이 분기에 들어가지 않아야 한다.
- App struct에 별도 timezone 복사본이나 저장 scheduler를 추가하지 않는다. 현재 runtime source of truth는
  `slot2_platform::clock::utc_offset_min()`이다.

### 3. D-25와 기존 동작 보존

- system clock, RTC, process timezone, 환경변수와 파일 mtime을 변경하지 않는다.
- 설정 파일은 시작 시 읽기 전용이다. `write_utc_offset_minutes`나 직접 file write를 호출하지 않는다.
- volume startup load/debounce, HUD 표시 조건, battery polling, Screen/Session 동작을 바꾸지 않는다.
- App HUD는 기존처럼 동일 UTC sample의 유효성을 먼저 판단한 뒤 runtime offset을 표시 직전에 더한다.
  offset 때문에 `SET_AFTER` visibility 판정이 달라지면 안 된다.
- `main.rs::boot()`의 진단 로그는 App 생성 전 찍혀 환경 초기값을 보여줄 수 있다. 로그 이동·추가·삭제는
  이번 범위 밖이며 결과 보고서의 남은 위험에 기록한다.

## 테스트 계약

`crates/slot2/tests/timezone_startup_app.rs`를 새로 만든다. process-global clock의 병렬 오염을 피하려고
파일 안의 `#[test]`는 **정확히 하나**만 두고 다음을 순차 검증한다.

- 시작 전 runtime offset을 카드와 다른 유효값으로 설정한 뒤, 저장된 `540`, `-480`, `-720`, `840`,
  `0` 각각으로 `App::with_card`를 만들면 getter가 즉시 그 값이 됨
- card file/key 부재가 이전 runtime 값을 0으로 덮음
- invalid text, 범위 밖 text, invalid UTF-8가 각각 runtime 0이 되고 원본 bytes가 불변
- startup read가 missing file을 만들지 않고 정상 file을 rewrite하지 않음
- 같은 test 안에서 마지막으로 처음 runtime 값을 복원함. 중간 assertion panic에도 안전하게 복원하려고
  새 dependency/unsafe/global test hook을 만들지는 않는다; 단일 test process 격리가 최종 안전망이다.
- store/platform min/max/default 상수가 기대한 숫자이며 서로 같음. production compile-time assertion의
  의도를 test 이름과 단언으로도 드러냄
- `clock::hud_local`의 순수 함수로 같은 valid UTC sample에 `540`과 `-480`을 적용한 결과가 각각 정확히
  `offset * 60`만큼 이동하고, `SET_AFTER - 1`은 어느 offset에서도 `None`임
- 모든 생성에서 volume은 해당 카드의 기존 계약대로 load되며 시간대 적용이 volume file bytes를
  변경하지 않음

실제 wall clock, sleep, 환경변수 변경과 test-only reset API에 의존하지 않는다. 특정 순서의 texture id,
core/audio/GL 창도 사용하지 않는다. 기존 테스트의 단언을 새 동작을 숨기도록 완화하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2\src\app.rs`
- `C:\SLOT2\crates\slot2\tests\timezone_startup_app.rs` (신규)
- 시작 적용으로 직접 깨지는 기존 `C:\SLOT2\crates\slot2\tests\*_app.rs`의 setup/expectation만
- `C:\SLOT2\tasks\81-timezone-startup-app-wiring.worker-result.md`

그 밖의 production 파일은 수정하지 않는다. 특히 `slot2-store`, `slot2-platform`, `main.rs`, host/device
loop, UI/i18n을 고치지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- 시간대 선택 화면, shelf menu와 입력 동작
- runtime 선택값의 즉시 저장, rollback, toast와 retry
- `System/slot2.ini` 형식·범위·safe-write 변경
- OS clock/timezone/RTC/mtime 변경
- 환경변수 precedence를 위한 새 분기 또는 환경변수 mutation
- boot 진단 로그 변경
- M5 checkbox 완료 처리
- 전체 workspace 테스트, 실제 GL 창, device 배포
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 네트워크와 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2 --test timezone_startup_app
cargo test -p slot2
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2 --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 각 test 명령의 passed/failed/ignored 합계와 core-dependent skip 수를 보고한다.
검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test와 device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\81-timezone-startup-app-wiring.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- compile-time store/platform 범위 일치 봉인 결과
- saved/boundary/default/invalid/unreadable startup 적용 결과와 read-only 보존
- 단일 순차 integration test 및 process-global clock 격리 방식
- D-25 HUD 변환·visibility와 volume 회귀 결과
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored와 skip 수
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- boot 진단 로그가 App 전 초기값일 수 있다는 남은 위험과 후속 UI 배선
- 계약이 틀려 보이는 부분이 있으면 그 내용
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
