# Task 65 — 전역 Volume App 영속 배선 (결과)

- 결과: **성공**. 누적 호출 **1/2**(이번이 첫 호출이며 재시도하지 않았다).
- 범위: `crates/slot2`만. `slot2-store`(settings.rs diffstat은 Task64 종료 시점과 동일), `slot2-audio`,
  UI, i18n, host/device loop은 변경하지 않았다.

## startup load와 fallback

- `App::with_card`가 struct를 만들기 전에 `card.read_global_settings()`를 한 번 읽고
  `Volume::new(settings.volume)`으로 `App.volume`을 만든다. mute는 항상 false로 시작한다.
- 저장값 0·42·100이 그대로 적용되고, 시작 직후에는 pending write가 없다(부팅 뒤 tick 5회에도 ini
  bytes 불변).
- missing·`volume = abc`·`101`·`-5`·빈 값·invalid UTF-8은 store가 돌려준 `DEFAULT_VOLUME_LEVEL` 70으로
  시작하며 원본 bytes/부재 상태를 그대로 둔다. App 쪽에 두 번째 parse나 복구 규칙은 없고, 손상 파일을
  몰래 고치지도 않는다.
- `App::new`는 그대로 `with_card` 경로를 쓰고, 새 영속 테스트는 모두 명시적 임시 Card를 쓴다(작업
  디렉터리에 우연한 설정이 생기지 않음).

## debounce / reschedule / cancel / clamp / mute

- `pub const VOLUME_SAVE_DELAY_MS: u64 = 750`을 App module에 두고 test가 같은 상수를 쓴다.
- 물리 VolUp/VolDown과 Device Left/Right가 `step_volume(button, now)` 한 곳을 지나고, level이 실제로
  바뀐 경우에만 `defer_volume_save(now)`가 due를 `now + 750ms`로 잡는다.
- `tick(now)`은 그 tick의 gesture action을 모두 처리한 뒤 due를 검사해 `now >= due`면 `flush_volume()`을
  한 번 시도한다.
- 재예약: 두 번째 변경이 deadline을 뒤로 미룬다(첫 deadline에 write 없음, 마지막 변경 +750ms에 최종
  level 하나 저장). 경계는 press의 release 기준 749ms 무기록 / 750ms 기록으로 직접 확인했다.
- 취소: 저장값 30에서 35로 올렸다가 deadline 전에 30으로 내리면 이후 tick에도 파일이 그대로다.
- clamp: 100에서 VolUp, 0에서 VolDown은 write를 만들지 않고, **이미 pending인 write를 미루지도 않는다**
  (70→100으로 6회 올린 뒤 deadline 100ms 전에 100에서 VolUp → 원래 deadline에 `volume = 100` 저장).
- mute: Device의 A는 pending을 만들지도, 기존 due를 미루지도 않는다. muted에서 Left는 mute를 유지한 채
  기억된 level만 저장하고, Right로 unmute된 뒤 저장되는 파일에도 `mute`/`muted` key가 없다.

## 저장 성공·실패·재시도 / unknown key

- 저장은 `Card::write_global_settings(&GlobalSettings { volume: self.volume.level() })` 하나만 쓴다.
  App은 `Ini`·path·temp file·unknown key를 직접 다루지 않는다.
- 성공 시 `volume_saved`를 갱신하고 pending을 지운다. 저장된 level과 현재 level이 같으면 flush는
  파일을 건드리지 않는 no-op이다.
- 실패(invalid UTF-8, path가 directory)에서도 runtime level·mute·screen·session·sink·exit가 그대로이고
  원본 bytes/directory가 보존된다. stderr 한 줄 `slot2: cannot save global volume: …` 뒤 pending을
  비우므로 이후 tick이 매번 재시도하지 않는다(손상 파일을 정상 파일로 바꿔도 단순 tick으로는 write되지
  않음을 확인). 손상을 고친 뒤 level을 다시 바꾸면 새 저장이 성공한다.
- unknown/future key(`future_key`, `language`)는 debounce 저장 뒤에도 값·spelling 그대로 남는다.
- 전역 write가 게임별 ini bytes, session, sink request, screen을 건드리지 않고 brightness/blue-light/
  timezone key를 만들지 않는다.

## 종료 전 즉시 flush와 non-exit 전환

- `exit_now(exit)` 공통 helper가 `flush_volume()` → `stop_session()` → `exit` 순서를 담당하고, physical
  Power tap / Power menu Restart / Power menu PowerOff 세 경로가 모두 이를 쓴다. deadline 전에 눌러도
  현재 level이 즉시 저장되고 각 `Exit` 값은 그대로 설정된다.
- unreadable 파일로 flush가 실패해도 세 종료 경로 모두 `Exit`이 설정된다(원본 bytes 보존).
- Eject와 in-game menu 닫기는 강제 flush하지 않는다: 화면 전환 직후 write가 없고, 이후 tick에서 정상
  debounce가 최종 level을 저장한다.
- 저장 실패 toast/i18n·modal·retry loop·background thread 없음.

## 완료 기준 명령 (원문 순서, 마지막 코드 변경 뒤, 이후 코드 변경 없음)

1. `cargo fmt --all -- --check` → exit **0**, 출력 없음.
2. `cargo test -p slot2 --test global_volume_app --test device_menu_app` → exit **0**.
   `device_menu_app` **13 passed**(1.23s), 신규 `global_volume_app` **16 passed**(0.32s).
   마지막 결과 줄: `test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
   finished in 0.32s`.
3. `cargo test -p slot2 --lib` → exit **0**,
   `test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s`.
4. `cargo clippy -p slot2 --all-targets -- -D warnings` → exit **0**,
   `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 5.16s`.

- 추가 회귀 확인(모든 production 변경 뒤): `cargo test -p slot2`가 `test result:` 23줄,
  **219 passed / 0 failed / 0 ignored**. Task63 시점의 203 + 신규 16이며 잃은 테스트는 없다.
- 음성 대조 2건: (a) `step_volume`이 level 변화 여부와 무관하게 defer하도록 일시 변형 →
  `a_clamped_press_is_not_storage_activity` FAILED(15 passed/1 failed), (b) 실패한 flush가 pending을
  남기도록 변형 → `a_failed_write_leaves_the_running_volume_alone_and_is_not_retried` FAILED
  (15 passed/1 failed). 원복 후 16 passed / 0 failed 재확인.

## 생성·수정 파일

- 생성: `crates/slot2/tests/global_volume_app.rs`(16 tests),
  `tasks/65-global-volume-app-persistence.worker-result.md`.
- 수정: `crates/slot2/src/app.rs`(module 상태 설명 + volume 영속 문단, `VOLUME_SAVE_DELAY_MS`,
  `volume_saved`/`volume_due` field, `with_card`에서 global 설정 load, `tick`의 due flush,
  `step_volume`/`defer_volume_save`/`flush_volume`/`exit_now`, 전역 VolUp/VolDown과 Device
  `change_volume`에 `now` 전달, 세 power exit를 helper로 통합).
- 수정(계약 §5에 따른 Task63 기대 갱신만): `crates/slot2/tests/device_menu_app.rs` —
  `using_the_device_menu_writes_nothing`을 `the_device_menu_writes_only_the_volume_after_the_delay`로
  바꿔 "입력 직후에는 write 없음 + 마지막 level 변경 뒤 750ms에 volume key만 저장"을 검증하고,
  `tree` 헬퍼 주석과 import 1줄을 갱신했다. 테스트를 삭제하거나 약화하지 않았다.
- 최종 검증 뒤 코드 변경 없음. 커밋·푸시·네트워크·실기·공용 설정 변경·위임 없음.

## 계약 의견 / 남은 위험

- "저장값으로 되돌아오면 pending 취소" 분기는 `flush_volume`의 no-op guard와 관찰 가능한 파일 동작이
  완전히 같아(둘 다 파일을 건드리지 않음) bytes만으로는 두 경로를 구분할 수 없다 — 계약은 지켰지만 그
  부분만은 테스트가 아니라 코드 독해로 확인했다.

## 소요 시간

약 17분(23:05–23:21 KST). 누적 호출 1/2.
