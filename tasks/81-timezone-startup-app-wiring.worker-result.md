# Task 81 — 작업자 결과 (호출 1/2)

- **결과:** 성공
- **누적 작업자 호출:** 1/2 (실패·중단 없음)
- **소요 시간:** 약 12분 (00:39 → 00:51 KST, 최종 명령 순차 실행 포함)

## 구현 요약

- `crates/slot2/src/app.rs` 상단(모듈 아이템, `#[cfg(test)]` 밖)에 production compile-time seal
  `const _: () = { assert!(...); }` 5줄 추가: store/platform min·max 동치, store default == 0,
  default가 platform 범위 안.
- `App::with_card`에서 struct 생성 전 `card.read_utc_offset_minutes()`를 정확히 한 번 읽어
  `clock::set_utc_offset_min`에 적용. `App::new`는 기존대로 `with_card`를 통과하므로 별도 경로 없음.
- setter가 거부하면 panic/unwrap 없이 `eprintln!("slot2: card utc offset {rejected} is outside the
  clock's range; using UTC")` 한 줄 후 `DEFAULT_UTC_OFFSET_MINUTES`(0) 적용을 시도. seal 때문에 정상
  source에서는 도달 불가.
- App struct·HUD draw·host/device loop·volume debounce·main.rs는 변경 없음. 새 struct 필드, 복사본,
  저장 scheduler 없음.
- `crates/slot2/tests/timezone_startup_app.rs` 신규. `#[test]` 정확히 1개.

## 검증 항목별 결과

### 1. compile-time seal (production)
- seal은 `slot2` 라이브러리 아이템이며 5개 명령 모두에서 컴파일된다(host/device 양쪽, test/비-test).
- 실제로 발화하는지 확인: `slot2-store`의 `UTC_OFFSET_MINUTES_MIN`을 `-720` → `-721`로 임시 변경 후
  `cargo check -p slot2` → **E0080 `assertion failed: slot2_store::UTC_OFFSET_MINUTES_MIN ==
  slot2_platform::clock::OFFSET_MIN` (app.rs:181)**, 종료 101. 즉시 `-720`으로 복원,
  sha256 `2c68568531adc0145ce30925e434f8d7696559a1c29f4f21ebe1f6c885640736`로 byte 동일 확인(임시 변경
  전후 동일), `cargo check -p slot2` 재실행 종료 0. 실패 값이 아니라 assert 위치가 원인 줄로 보고됨.
- 두 크레이트 상수는 변경하지 않았다. 현재 값은 실제로 일치한다(min -720, max 840, default 0).

### 2. startup 적용
단일 test 안에서 순차 검증, 마지막에 처음 runtime 값 복원.
- 시작 전 runtime을 카드와 다른 유효값(0 또는 60)으로 설정한 뒤 저장값 `540`, `-480`, `-720`, `840`,
  `0` 각각으로 `App::with_card` 생성 → getter가 즉시 그 값. 카드가 이전 값보다 우선함을 확인.
- card file/key 부재 → 이전 runtime `540`/`-480`을 0으로 덮음.
- invalid text(`KST`), fraction(`9.5`), 범위 밖(`900`), invalid UTF-8 각각 runtime 0, 원본 bytes 불변.
- read-only 보존: 정상 file 5종·key만 있는 file·무효 file 모두 startup 전후 bytes 동일. settings
  file 부재 카드에서는 생성 없음 + `System` 디렉터리 entry 수 불변.
- volume 계약: `volume=30` 카드에서 `App::volume.level() == 30`, 시간대만 있는 카드와 무파일 카드에서
  `DEFAULT_VOLUME_LEVEL`(70). 시간대 적용이 volume bytes를 바꾸지 않음.
- 상수 단언: store min/max/default == `-720`/`840`/`0`이고 각각 platform `OFFSET_MIN`/`OFFSET_MAX`와
  동일(test 이름과 단언으로 seal 의도 노출).

### 3. 순차 integration test / clock 격리
- 파일 안 `#[test]` 1개, 모든 시나리오를 한 함수에서 순차 실행. `SLOT2_UTC_OFFSET_MIN` env 변경,
  test-only reset API, sleep, wall clock 의존 없음. 마지막 줄에서 시작 시 읽은 runtime 값 복원
  (중간 panic 시 복원은 생략되며, 단일 test process 격리를 최종 안전망으로 둠 — 명세 허용 범위).
- 새 dependency·unsafe·global hook 없음.

### 4. D-25 HUD / visibility / volume 회귀
- `clock::hud_local` 순수 함수: `SET_AFTER + 3h + 1234` sample에 540 → `+540*60`, -480 → `-480*60`
  정확히 이동. `SET_AFTER - 1`은 `OFFSET_MIN`/`-480`/`0`/`540`/`OFFSET_MAX` 어느 offset에서도 `None`
  (판정이 shift 전에 이루어짐).
- App HUD draw 코드는 수정하지 않았다. `utc_now()` → `hud_local(utc, utc_offset_min())` 순서 유지.
- volume startup load/debounce, battery polling, Screen/Session 동작 변경 없음. `cargo test -p slot2`
  전 test target 통과(0 failed).

## 완료 기준 명령 (마지막 코드 변경 뒤 순서대로)

| 명령 | 종료 코드 | 마지막 결과 줄 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | (출력 없음) |
| `cargo test -p slot2 --test timezone_startup_app` | 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.68s` |
| `cargo test -p slot2` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (doc-tests, 최종 줄) |
| `cargo check -p slot2 --no-default-features --features device` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 8.69s` |
| `cargo clippy -p slot2 --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 21.24s` |

- `cargo test -p slot2` 합계: **313 passed / 0 failed / 0 ignored** (lib unittests 33, main bin 0,
  24개 integration test target 280, doc-tests 0; 27개 result 줄).
- core-dependent skip: **0**. 사전 조건이 모두 checkout에 존재한다(`vendor/mgba_libretro.dll` 등 6종 core,
  `assets/test/arm.gba`, `assets/test/local/*`). 실행 출력에 `skipping` 문구 없음.
- 집중 test skip/ignored: 0. core 불필요.
- `cargo fmt --all`이 재포맷한 파일은 신규 test 파일 1개뿐(`--check`가 사전에 그 1개 diff만 보고).
  그 밖의 미커밋 변경은 정리·복원·재포맷하지 않았다. workspace test와 device 배포는 실행하지 않았다.

## 생성·수정 파일

- 수정: `crates/slot2/src/app.rs` (seal 1블록 + `with_card` 시작 적용 + 이유 주석)
- 신규: `crates/slot2/tests/timezone_startup_app.rs`
- 신규: `tasks/81-timezone-startup-app-wiring.worker-result.md`
- 기존 App constructor 테스트 중 새 동작으로 깨진 것은 없어 `*_app.rs` setup/expectation을 고치지 않았다.
- 최종 검증 5개 명령 실행 뒤 코드 변경 없음(중간에 있었던 store 상수 임시 변경은 검증 전에 byte 동일
  복원했고, 그 뒤 5개 명령을 순서대로 다시 수행했다).

## 남은 위험과 후속

- `main.rs::boot()`가 App 생성 전에 `utc_offset_min`을 진단 로그로 찍는다. 기기에서 이 배너 값은 카드
  값이 아니라 환경 초기값(기기에서는 0)이다. 로그 이동·추가는 이번 범위 밖이며, 배너와 실제 시계 표시가
  다를 수 있다.
- 시간대 선택 UI, runtime 변경의 즉시 적용/저장/rollback/toast는 후속 태스크다. 이번 배선은 시작 시
  load/apply만 다루므로, 앱 실행 중에는 여전히 `slot2_platform::clock::utc_offset_min()`이 유일한
  source of truth이고 카드 재읽기는 없다.
- M5 시간대 항목 checkbox는 이번 태스크로 완료 처리하지 않았다(저장 UI 미구현).

## 계약 의문

없음. 명세의 store/platform 범위 상수는 실제로 일치하며, App 경계 seal 외에 두 크레이트를 고칠 필요가
없었다.
