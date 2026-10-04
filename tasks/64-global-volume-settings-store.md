# Task 64 — 전역 Volume 설정 저장 기반

현재 checkout에서 직접 작업한다. 카드의 전역 설정 파일 `System/slot2.ini`에 frontend volume level을
안전하게 읽고 쓸 수 있는 `slot2-store` 계약을 추가한다. 이번 태스크는 **store 한 크레이트만** 다룬다.
App 시작 시 적용, 메뉴·물리 버튼의 저장 시점, UI와 audio 변경은 Task65로 분리한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\64-global-volume-settings-store.md`
- `C:\SLOT2\tasks\47-safe-game-settings-write.result.md`
- `C:\SLOT2\crates\slot2-store\src\settings.rs`
- `C:\SLOT2\crates\slot2-store\src\ini.rs`
- `C:\SLOT2\crates\slot2-store\src\card.rs`의 `settings_path`와 path helper 부분만
- `C:\SLOT2\crates\slot2-store\src\lib.rs`의 module/export 부분만
- `C:\SLOT2\crates\slot2-store\tests\card.rs`의 settings·atomic write 테스트 부분만
- `C:\SLOT2\docs\DESIGN.md`의 게임별 설정, 설정 계층, `System/slot2.ini` 부분만
- `C:\SLOT2\docs\DECISIONS.md`의 D-23과 D-25만

직접 관련된 `Error`와 atomic-write 구현만 추가로 읽는다. App, Session, UI, audio, platform,
워커 로그와 저장소 이력은 읽지 않는다.

## 현재 계약과 결정

- `Card::settings_path()`는 이미 `<card>/System/slot2.ini`를 가리키지만 이 파일을 읽고 쓰는 공개 API는
  아직 없다.
- `System/slot2.ini`는 전역 설정 계층이다. 이번에는 frontend volume **level 하나만** 소유한다.
- 기본 volume level은 현재 `slot2_audio::Volume::default()`와 같은 **70**이다. store가 audio 크레이트에
  의존하지 않도록 공개 상수 `DEFAULT_VOLUME_LEVEL: u8 = 70`을 store 쪽 설정 계약에 둔다. Task65가 이
  상수를 사용해 기본값의 두 번째 사본을 만들지 않게 한다.
- 저장 범위는 `0..=100`이다. 빠진 key, 숫자가 아닌 값, 음수, 100 초과 값은 각각 독립적으로 기본 70으로
  떨어진다. 잘못된 설정이 frontend 시작을 막거나 암묵적으로 0/100으로 바뀌면 안 된다.
- **mute는 저장하지 않는다.** mute는 현재 runtime 토글이며 재시작한 기기가 이유를 알기 어려운 무음
  상태로 뜨지 않게 한다. `muted`/`mute` key를 새로 만들거나 기존 unknown key를 소유하지 않는다.
- brightness, blue light, timezone, language와 다른 전역 key는 이번 태스크가 해석하거나 변경하지 않는다.
  특히 D-25의 timezone key 이름이나 범위를 미리 정하지 않는다.

## 구현 계약

### 1. 공개 설정 타입

- `slot2-store`에서 공개 `GlobalSettings`를 제공한다.
- 기존 `GameSettings`와 같은 단순한 공개 shape으로 `pub volume: u8`을 제공하고, `Default`는
  `volume == DEFAULT_VOLUME_LEVEL`이다. 파일에서 parse된 값은 항상 `0..=100`이다.
- 호출자가 직접 `volume > 100`인 구조체를 만들어 write하면 `Error::Invalid`를 반환하고 파일을 전혀
  변경하지 않는다. silently clamp하거나 100을 저장하지 않는다. Task65의 `Volume::level()`은 이미 유효
  범위를 보장하므로 별도 fallible constructor나 호출자 `unwrap()`은 필요 없다.
- 소유 key는 정확히 `volume`이다. 파일 표기는 10진수 `0`부터 `100`까지다.

### 2. 읽기

- `Card::read_global_settings()`를 추가한다.
- 파일 부재는 `GlobalSettings::default()`다.
- 읽기 실패, directory가 파일 위치를 막는 경우, invalid UTF-8도 frontend를 막지 않고 default를 돌려준다.
- 읽을 수 있는 ini에서는 `volume`만 해석한다. 앞뒤 공백이 있는 정상 10진수는 허용한다.
- invalid value는 default 70이며 unknown key는 읽기 결과에 영향을 주지 않는다.
- 읽기는 파일을 만들거나 고치거나 삭제하지 않는다.

### 3. 안전한 쓰기

- `Card::write_global_settings(&GlobalSettings) -> Result<(), Error>` 의미의 공개 API를 추가한다.
- mutation 전에 현재 `System/slot2.ini`를 `Ini::load`로 **fallible하게** 읽는다. 기존 파일이 unreadable,
  invalid UTF-8 또는 directory면 error를 반환하고 원본 bytes/path를 그대로 둔다. 이를 빈 ini로 취급해
  덮거나 지우지 않는다.
- `volume != DEFAULT_VOLUME_LEVEL`이면 `volume = <decimal>`을 설정하고, 기본 70이면 소유 key를 제거한다.
- 자신이 소유하지 않은 모든 key를 그대로 보존한다. 예를 들어 미래의 `utc_offset_minutes`, `language`,
  손으로 쓴 `future_key`는 volume write 뒤 값과 spelling이 유지돼야 한다.
- 소유 key를 제거한 뒤 ini가 비면 파일을 제거한다. 파일이 이미 없으면 성공 no-op이다. unknown key가
  하나라도 남으면 파일을 보존한다.
- 저장은 기존 `Ini::save`/atomic write를 사용한다. 별도 임시파일 규칙이나 generic settings framework를
  만들지 않는다.
- 삭제 실패와 save 실패를 호출자에게 반환한다. 성공하지 않은 write를 성공으로 보고하면 안 된다.
- API로 만들어진 `volume > 100`의 write도 `Error::Invalid`이며 기존 파일과 unknown key를 그대로 둔다.

## 테스트 계약

`crates/slot2-store/tests/global_settings.rs`를 새로 만들고 최소한 다음을 직접 검증한다.

- 파일 부재의 default volume 70과 읽기가 파일을 만들지 않음
- `volume = 0`, 중간값, `100`의 parse와 write/read round trip
- `volume= 85` 같은 정상 공백 허용
- 빈 값, 문자, 음수, 101 이상이 각각 default 70으로 떨어지고 파일 원문은 읽기만으로 변하지 않음
- `GlobalSettings { volume: 101 }`의 write가 `Error::Invalid`이고 기존 파일 bytes를 바꾸지 않음
- write가 정확히 `System/slot2.ini`에 저장되고 `settings_path()`와 일치함
- unknown key가 volume 추가·변경·기본 복귀 뒤에도 값 그대로 남음
- 기본 70으로 돌아갈 때 volume key만 제거되고, ini가 완전히 비면 파일도 제거됨
- missing file에 default write는 파일을 만들지 않는 성공 no-op
- invalid UTF-8 기존 파일에 non-default write와 default write가 모두 error이며 exact bytes가 보존됨
- settings path가 directory일 때 두 write 모두 error이며 directory를 제거하지 않음
- `mute`/`muted`, brightness, blue light, timezone key를 생성하거나 소유하지 않음. 그런 unknown key가
  이미 있으면 그대로 보존됨
- 기존 game settings read/write와 unknown-key 보존 테스트가 계속 통과함

Windows에서 의미가 다른 permission bit로 failure를 만들지 않는다. invalid UTF-8과 directory path를
portable failure fixture로 사용한다. 테스트를 통과시키기 위한 fixture 전용 분기나 sleep/mtime 의존 검증은
추가하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-store\src\settings.rs`
- `C:\SLOT2\crates\slot2-store\src\lib.rs`
- `C:\SLOT2\crates\slot2-store\tests\global_settings.rs` (신규)
- 직접 필요한 경우에만 `C:\SLOT2\crates\slot2-store\tests\card.rs`
- `C:\SLOT2\tasks\64-global-volume-settings-store.worker-result.md`

`Card::settings_path()`가 이미 있으므로 `card.rs`, `ini.rs`, `atomic.rs` production 변경은 허용하지 않는다.
계약 구현이 불가능하다고 판단되면 범위를 넓히지 말고 보고서에 이유를 적어 실패로 둔다. 관련 없는
미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- App 시작 시 volume load, 버튼/menu 변경의 저장 시점, toast와 retry
- mute 영속성 또는 `mute`/`muted` key 정의
- brightness/blue-light/timezone/language/platform/game setting 추가
- `slot2-audio`, App, Session, UI, platform 변경
- `GameSettings`, `ScaleMode`, `Ini`, atomic-write와 `Error`의 공개 의미 변경
- 전체 workspace 테스트, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-store
cargo clippy -p slot2-store --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test나
device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\64-global-volume-settings-store.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- `GlobalSettings`, default/range/invalid parse 계약
- safe write, unknown-key 보존, 기본 복귀와 unreadable 원본 보존 결과
- mute와 다른 미래 key를 소유하지 않았다는 확인
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
