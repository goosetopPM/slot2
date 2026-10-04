# Task 65 — 전역 Volume App 영속 배선

현재 checkout에서 직접 작업한다. Task64의 `GlobalSettings`를 App에 연결해 시작 시 저장된 volume level을
적용하고, 메뉴·물리 버튼에서 바뀐 level을 입력이 잠잠해진 뒤 한 번 저장한다. App이 직접 처리하는
PowerOff/Reboot에서는 debounce를 기다리지 않고 마지막 level을 flush한다. mute는 계속 runtime 상태다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\65-global-volume-app-persistence.md`
- `C:\SLOT2\tasks\64-global-volume-settings-store.result.md`
- `C:\SLOT2\tasks\63-device-menu-app-wiring.result.md`
- `C:\SLOT2\crates\slot2-store\src\settings.rs`의 `GlobalSettings`와 Card 전역 설정 API만
- `C:\SLOT2\crates\slot2-audio\src\volume.rs`
- `C:\SLOT2\crates\slot2\src\app.rs`의 App fields/constructor/tick/action, power exit와 volume helper만
- `C:\SLOT2\crates\slot2\tests\device_menu_app.rs`의 volume·no-write 테스트만
- `C:\SLOT2\docs\DECISIONS.md`의 D-23만

직접 관련된 test input/time helper만 추가로 읽는다. host/device event loop, Session/core, UI drawing,
platform hardware, 다른 설정, 워커 로그와 저장소 이력은 읽지 않는다.

## 현재 계약과 결정

- Task64의 `Card::read_global_settings()`는 부재·unreadable·invalid volume을 안전하게 기본 70으로 읽는다.
  `write_global_settings`는 `volume`만 소유하고 unknown/future key를 보존한다.
- App의 유일한 runtime volume 상태는 `App.volume`이다. 메뉴와 물리 키는 이미 이 객체를 함께 쓴다.
- volume key press마다 즉시 atomic write하면 연속 조절 한 번이 여러 SD write가 된다. 마지막 **level 변경
  뒤 750ms** 동안 추가 level 변경이 없을 때 한 번만 저장한다.
- A mute/unmute는 저장 대상도 debounce 기준도 아니다. mute는 재시작 때 항상 false다.
- runtime volume 변경은 저장 성공보다 우선한다. 저장 실패 때문에 소리가 이전 level로 돌아가거나 입력,
  core, sink, 화면이 멈추면 안 된다.
- App이 직접 발생시키는 physical Power, Power menu Restart, Power menu PowerOff는 마지막 level을 즉시
  저장 시도한 뒤 기존 종료를 계속한다. host 창 X/Escape, process crash, 전원 강제 차단은 App action이
  아니므로 이번 범위의 강제 flush 계약에 포함하지 않는다. 평상시 750ms debounce가 저장한 값까지는 남는다.

## 구현 계약

### 1. 시작 시 load

- `App::with_card`는 struct를 만들기 전에 `card.read_global_settings()`를 한 번 읽고
  `slot2_audio::Volume::new(settings.volume)`으로 `App.volume`을 만든다.
- 저장값 0과 100을 포함해 그대로 적용한다. mute는 false로 시작한다.
- missing, unreadable, invalid 설정은 store가 돌려준 `DEFAULT_VOLUME_LEVEL` 70을 사용한다. App이 ini를
  다시 parse하거나 두 번째 fallback 규칙을 만들지 않는다.
- 현재 저장됐다고 알고 있는 level을 App private state로 기억한다. 시작 직후에는 pending write가 없다.
- `App::new`도 기존 `with_card` 경로를 유지한다. 테스트가 현재 작업 디렉터리의 우연한 설정에 기대지
  않도록 새 영속 동작 테스트는 모두 명시적 임시 Card를 사용한다.

### 2. level 변경 debounce

- 제품 계약 상수로 `VOLUME_SAVE_DELAY_MS: u64 = 750`을 App module에 둬 integration test가 같은 시간을
  숫자로 복사하지 않게 한다.
- 물리 VolUp/VolDown과 Device 화면 Left/Right가 실제 `App.volume.level()`을 바꾼 뒤 공통 helper에
  해당 action의 `now`를 넘긴다.
- 새 level이 마지막으로 성공 저장된 level과 다르면 due를 `now + 750ms`로 설정한다. 이미 pending이면
  **마지막 level 변경을 기준으로 다시 750ms** 뒤로 미룬다.
- 새 level이 저장된 level과 같아졌으면 pending을 취소한다. 예: 저장값 70에서 75로 올렸다가 deadline
  전에 70으로 내리면 파일 write가 없어야 한다.
- clamp 때문에 level이 실제로 바뀌지 않은 입력은 deadline을 새로 만들거나 미루지 않는다. 100에서
  VolUp, 0에서 VolDown은 storage activity가 아니다. 단 muted 100에서 VolUp으로 mute만 풀린 경우도
  level write가 아니다.
- A의 mute/unmute는 pending을 만들거나 기존 due를 미루지 않는다. Device menu snapshot 동기화는
  Task63과 같이 즉시 유지한다.
- `tick(now)`은 그 tick에서 생긴 gesture action을 모두 처리한 **뒤** due를 검사한다. `now >= due`면
  현재 level로 한 번 저장 시도한다. 같은 tick의 새 input을 저장했다가 다시 dirty로 만드는 순서를 피한다.

### 3. flush 성공·실패

- 저장은 `Card::write_global_settings(&GlobalSettings { volume: self.volume.level() })`만 사용한다.
  App이 `Ini`, path, temp file이나 unknown key를 직접 다루지 않는다.
- 성공하면 remembered persisted level을 갱신하고 pending을 지운다.
- 실패해도 runtime level과 mute, screen, session, sink, exit를 바꾸지 않는다. 다음 문맥을 포함한 한 줄을
  stderr에 남기고 pending을 지워 매 frame 재시도·로그 폭주를 막는다:
  `slot2: cannot save global volume: ...`
- 실패 뒤 level이 다시 실제로 바뀌면 새 debounce 저장을 시도할 수 있다. level 변경이 없어도 아래의
  명시적 power exit에서는 한 번 더 flush를 시도한다.
- 저장된 level과 현재 level이 같으면 flush는 filesystem을 건드리지 않는 no-op이다.

### 4. 종료 전 즉시 flush

- physical Power tap, Power menu Restart, Power menu PowerOff에서 `exit`를 설정하기 전에 pending 여부와
  deadline에 관계없이 현재 level을 flush한다. 성공·실패와 관계없이 기존 session stop과 Exit 의미는
  유지한다.
- 세 종료 경로가 서로 다른 저장 규칙을 갖지 않도록 작은 공통 helper를 사용한다.
- Eject, 인게임 메뉴 닫기, Device submenu 닫기, game/core 전환은 App 종료가 아니므로 강제 flush하지
  않는다. debounce는 화면과 session 전환 뒤에도 유효하다.
- 저장 실패 toast/i18n을 새로 만들지 않는다. 종료를 붙잡는 modal이나 retry loop도 만들지 않는다.

### 5. 기존 계약 정리

- app.rs 상단 상태 설명에 global volume load/debounce/power flush를 간결히 반영한다.
- Task63의 `using_the_device_menu_writes_nothing`처럼 volume이 영구히 저장되지 않는다고 가정하는 테스트는
  삭제하지 말고 새 계약으로 바꾼다: 각 입력 직후에는 write가 없고, 마지막 level 변경 뒤 750ms가
  지나면 volume key만 저장된다.
- brightness/blue-light는 계속 `None`/Unavailable이고 어떤 key도 쓰지 않는다.

## 테스트 계약

`crates/slot2/tests/global_volume_app.rs`를 새로 만들고, 직접 충돌하는 Task63 테스트만 좁게 갱신한다.
실제 core나 audio sink 없이 임시 Card와 `Screen::List`/인공 Device screen으로 최소한 다음을 검증한다.

- `with_card`가 저장된 0, 중간값, 100을 load하고 모두 unmuted로 시작함
- missing, invalid text, out-of-range, invalid UTF-8 설정이 level 70/unmuted로 시작하며 원본을 고치지 않음
- 물리 VolUp/VolDown과 Device Left/Right가 runtime level·menu snapshot은 즉시 바꾸지만 750ms 전에는
  파일을 쓰지 않음
- 여러 level 변경이 각각 deadline을 뒤로 미루고, 마지막 변경 749ms까지 write가 없으며 750ms 경계부터
  최종 level 하나가 저장됨
- 저장값으로 deadline 전에 되돌아오면 pending이 취소돼 이후 tick에도 write가 없음
- 0/100 clamp input과 mute toggle은 새 write를 만들거나 기존 deadline을 미루지 않음
- mute 상태에서 level을 내리면 mute를 유지한 채 level만 저장되고, level을 올려 unmute돼도 파일에는
  level만 있으며 mute key가 없음
- 기존 unknown/future key는 debounce save 뒤에도 보존됨
- invalid UTF-8/directory write 실패에서도 runtime volume과 Device snapshot은 바뀌고 원본은 보존되며,
  뒤의 tick들이 매번 파일을 재시도하지 않음. portable하게 재시도 횟수를 직접 관찰할 방법이 없다면
  원본 보존과 pending 소진 뒤 정상 파일로 교체해도 단순 tick만으로 write되지 않는 것으로 검증한다.
- 실패 원인을 제거하고 level을 다시 변경하면 새 저장이 성공함
- physical Power, Power menu Restart, Power menu PowerOff는 750ms 전에도 현재 level을 저장 시도하고 기존
  `Exit`을 그대로 설정함. unreadable 파일 때문에 flush가 실패해도 각 Exit은 그대로 설정됨
- Eject와 submenu close는 강제 flush하지 않으며 정상 debounce가 이후 tick에서 동작함
- global write가 game ini, session, sink request와 brightness/blue-light key를 건드리지 않음

filesystem mtime, sleep, wall clock에 의존하지 말고 test가 만든 `Instant`를 명시적으로 전진시킨다.
write 횟수를 증명하려고 production에 test-only counter나 filesystem wrapper를 넣지 않는다. 기존 App
테스트의 시간 간격 때문에 우연히 750ms를 넘는 경우에는 해당 테스트의 의도를 유지하면서 임시 Card를
명확히 하거나 최종 expectation만 새 계약에 맞춘다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2\src\app.rs`
- `C:\SLOT2\crates\slot2\tests\global_volume_app.rs` (신규)
- `C:\SLOT2\crates\slot2\tests\device_menu_app.rs` (Task63 no-write expectation 갱신만)
- volume load로 직접 expectation이 깨지는 기존 `C:\SLOT2\crates\slot2\tests\*_app.rs` 한정
- `C:\SLOT2\tasks\65-global-volume-app-persistence.worker-result.md`

그 밖의 production 파일은 수정하지 않는다. 특히 `slot2-store`, `slot2-audio`, UI, i18n, host/device loop를
고치지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- mute persistence, brightness/blue-light/timezone/language key 추가
- host 창 X/Escape와 process crash/강제 전원 차단의 sync flush
- toast/i18n 추가, modal retry, background thread와 generic scheduler
- store file format·atomic write·Volume 의미 변경
- core/session/sink, Display/Core/Cheats/state 기능 변경
- 전체 workspace 테스트, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2 --test global_volume_app --test device_menu_app
cargo test -p slot2 --lib
cargo clippy -p slot2 --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test나
device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\65-global-volume-app-persistence.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- startup load와 missing/invalid fallback 결과
- 750ms debounce, reschedule/cancel/clamp/mute 처리 결과
- save 성공·실패·재시도와 unknown-key 보존 결과
- 세 power exit의 즉시 flush와 non-exit 화면 전환 결과
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
