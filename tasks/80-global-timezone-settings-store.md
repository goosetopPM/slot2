# Task 80 — 전역 시간대 오프셋 저장 기반

현재 checkout에서 직접 작업한다. D-25의 표시용 UTC 오프셋을 카드의 전역 설정 파일
`System/slot2.ini`에서 안전하게 읽고 쓸 수 있는 `slot2-store` 계약을 추가한다. 이번 태스크는
**store 기반만** 다룬다. 부팅 시 `slot2_platform::clock` 적용, 설정 화면과 즉시 반영은 후속 태스크로
분리한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\80-global-timezone-settings-store.md`
- `C:\SLOT2\tasks\64-global-volume-settings-store.result.md`
- `C:\SLOT2\docs\DECISIONS.md`의 D-25만
- `C:\SLOT2\docs\MILESTONES.md`의 M5 시간대 항목만
- `C:\SLOT2\docs\DESIGN.md`의 `System/slot2.ini`, 설정 계층, 시각 계약 부분만
- `C:\SLOT2\crates\slot2-store\src\settings.rs`의 `GlobalSettings`, parse helper와 global read/write 부분만
- `C:\SLOT2\crates\slot2-store\src\ini.rs`의 load/set/remove/save 공개 계약만
- `C:\SLOT2\crates\slot2-store\src\lib.rs`의 settings export 부분만
- `C:\SLOT2\crates\slot2-store\tests\global_settings.rs`
- `C:\SLOT2\crates\slot2-platform\src\clock.rs`의 `OFFSET_MIN`, `OFFSET_MAX`, `parse_offset_min` 계약만

App, UI, HUD, main loop, 다른 store 기능, 워커 로그와 저장소 이력은 읽지 않는다. 직접 관련된
`Error`와 atomic-write 구현만 필요할 때 추가로 읽는다.

## 현재 계약과 결정

- 시스템 시각과 파일 mtime은 UTC로 유지한다. 설정값은 표시 직전에 더하는 **분 단위 정수**일 뿐이며
  OS clock이나 timezone을 변경하지 않는다.
- 저장 key는 정확히 `utc_offset_minutes`다. 기본값은 UTC인 **0**이다.
- 유효 범위는 `slot2_platform::clock`과 같은 **-720..=840**분이다. 저장 크레이트가 platform
  크레이트에 새로 의존하지 않게 `slot2-store`에 이 파일 형식의 공개 상수
  `UTC_OFFSET_MINUTES_MIN`, `UTC_OFFSET_MINUTES_MAX`, `DEFAULT_UTC_OFFSET_MINUTES`를 둔다. 후속 App
  배선에서 platform 범위와 일치함을 검증한다.
- `GlobalSettings { volume }`의 공개 shape와 기존 호출자를 깨지 않는다. 시간대를 그 구조체 필드로
  추가하면 기존 volume 저장이 읽지 않은 시간대 값을 기본값으로 덮거나 제거할 수 있으므로, 이번에는
  `Card`의 독립 read/write API로 둔다.
- 기존 `Card::write_global_settings`는 volume key만 소유한다. 시간대 key가 정상·잘못된 값 어느 쪽이든
  volume 저장으로 key나 값이 바뀌면 안 된다.
- 환경변수 `SLOT2_UTC_OFFSET_MIN`은 host 개발용 초기값이다. 이 태스크가 읽거나 쓰지 않는다.

## 구현 계약

### 1. 공개 상수와 읽기

- `slot2-store`에서 다음 상수를 공개한다.
  - `DEFAULT_UTC_OFFSET_MINUTES: i32 = 0`
  - `UTC_OFFSET_MINUTES_MIN: i32 = -720`
  - `UTC_OFFSET_MINUTES_MAX: i32 = 840`
- `Card::read_utc_offset_minutes() -> i32` 의미의 공개 API를 추가한다.
- 파일 부재, key 부재, 읽기 실패, invalid UTF-8과 settings path가 directory인 경우 모두 기본 0을
  반환하며 파일/path를 변경하지 않는다.
- 정상 값은 앞뒤 ASCII/Unicode 공백을 허용한 10진 정수다. `0`, `540`, `-480`, `+60`, 경계값
  `-720`과 `840`을 허용한다.
- 빈 값, 소수, 내부 공백, 숫자 뒤 문구, 범위 밖, `i32` overflow는 기본 0으로 떨어진다. clamp하거나
  modulo로 감지 않는다.
- 읽기는 volume이나 unknown key를 해석·수정하지 않는다.

### 2. 독립적인 안전 쓰기

- `Card::write_utc_offset_minutes(i32) -> Result<(), Error>` 의미의 공개 API를 추가한다.
- 입력이 `-720..=840` 밖이면 파일을 읽기 전 `Error::Invalid`로 거부하고 기존 bytes/path를 그대로 둔다.
- mutation 전에 현재 `System/slot2.ini`를 `Ini::load`로 fallible하게 읽는다. 기존 파일이 unreadable,
  invalid UTF-8 또는 directory면 error를 반환하고 원본을 덮거나 지우지 않는다.
- non-default 값은 `utc_offset_minutes = <signed decimal>`로 저장한다. 양수에는 `+`를 붙이지 않는다.
- 기본 0은 소유 key만 제거한다. 제거 뒤 ini가 비면 파일을 제거하고, 파일이 원래 없으면 성공 no-op다.
- `volume`과 그 밖의 모든 key를 값 그대로 보존한다. 정상 또는 잘못된 기존 volume을 시간대 저장이
  고치거나 제거하지 않는다.
- 저장은 기존 `Ini::save`/atomic write를 사용한다. 별도 temp 규칙이나 generic settings framework를
  만들지 않는다.
- 삭제 실패와 save 실패를 호출자에게 반환한다. 성공하지 않은 write를 성공으로 보고하지 않는다.

### 3. 기존 volume 계약 회귀 방지

- `GlobalSettings`, `read_global_settings`, `write_global_settings`의 공개 signature와 volume default/range는
  바꾸지 않는다.
- volume write는 `utc_offset_minutes`를 unknown key처럼 보존한다. 특히 invalid
  `utc_offset_minutes = nope`도 volume 저장 때문에 0으로 정리되거나 삭제되면 안 된다.
- 시간대 write는 volume key를 unknown key처럼 보존한다. invalid `volume = nope`도 그대로 남긴다.
- 하나의 key를 기본값으로 되돌려도 다른 key가 남아 있으면 파일을 삭제하지 않는다.

## 테스트 계약

`crates/slot2-store/tests/timezone_settings.rs`를 새로 만들고 최소한 다음을 직접 검증한다.

- 파일과 key 부재의 기본 0, 그리고 read가 파일을 만들지 않음
- `0`, `540`, `-480`, `+60`, `-720`, `840`의 parse; 앞뒤 공백 허용
- 빈 값, 소수, 내부 공백, trailing text, `-721`, `841`, i32 overflow가 각각 기본 0이며 원문 불변
- `-720`, 중간 음수, 0, 중간 양수, `840`의 write/read round trip과 canonical decimal 표기
- 범위 밖 API write가 `Error::Invalid`이고 기존 file bytes가 정확히 불변
- write 경로가 정확히 `System/slot2.ini`이며 missing parent를 필요한 경우 생성함
- 시간대 추가·변경·기본 복귀 동안 정상/invalid volume과 unknown key가 그대로 보존됨
- volume write가 정상/invalid 시간대 key를 그대로 보존함
- 기본 0으로 돌아갈 때 시간대 key만 제거되고, ini가 완전히 비면 파일도 제거됨
- missing file에 기본 0 write는 파일을 만들지 않는 성공 no-op
- invalid UTF-8 기존 파일과 directory path에서 non-default/default write가 모두 error이며 bytes/path 보존
- read/write 어느 쪽도 `SLOT2_UTC_OFFSET_MIN`, system clock, mtime 보정을 사용하지 않음
- 기존 `global_settings` 테스트 전체가 계속 통과함

Windows에서 의미가 다른 permission bit로 failure를 만들지 않는다. invalid UTF-8과 directory path를
portable failure fixture로 사용한다. 테스트용 환경변수 변경, wall-clock 단언, sleep/mtime 의존 검증,
특정 테스트만 위한 production 분기를 추가하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-store\src\settings.rs`
- `C:\SLOT2\crates\slot2-store\src\lib.rs`
- `C:\SLOT2\crates\slot2-store\tests\timezone_settings.rs` (신규)
- 직접 필요한 회귀 단언만 `C:\SLOT2\crates\slot2-store\tests\global_settings.rs`
- `C:\SLOT2\tasks\80-global-timezone-settings-store.worker-result.md`

`card.rs`, `ini.rs`, `atomic.rs` production 변경은 허용하지 않는다. 계약 구현이 불가능하면 범위를 넓히지
말고 보고서에 이유를 적어 실패로 둔다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- App 시작 시 card offset load와 `slot2_platform::clock::set_utc_offset_min` 호출
- 시간대 선택 UI, shelf menu, 즉시 반영, 저장 실패 toast/rollback
- OS clock, system timezone, RTC, 파일 mtime 변경
- 언어·폰트·부팅 로고·네트워크·platform/game 설정 추가
- `GlobalSettings` field/signature, 기존 volume 동작과 저장 시점 변경
- `slot2-platform`, `slot2-ui`, App, HUD production 변경
- M5 checkbox 완료 처리
- 전체 workspace 테스트, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-store --test timezone_settings --test global_settings
cargo test -p slot2-store
cargo clippy -p slot2-store --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 각 test 명령의 passed/failed/ignored 합계를 보고한다. 검증 뒤 코드를 바꾸면
영향받는 명령부터 다시 실행한다. workspace test, device check와 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\80-global-timezone-settings-store.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- 공개 상수, key, default/range와 accepted/rejected parse 계약
- 독립 safe write, 기본 복귀, unreadable 원본 보존 결과
- 양방향 volume/timezone key 보존과 unknown-key 보존 결과
- 환경변수·system clock·mtime을 건드리지 않았다는 확인
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄과 passed/failed/ignored 합계
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 후속 App/UI 배선과 남은 위험 1줄
- 계약이 틀려 보이는 부분이 있으면 그 내용
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
