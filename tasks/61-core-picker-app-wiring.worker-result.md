# Task 61 — worker result (CorePicker App 배선과 실패 안전 코어 전환) — 누적 2/2

## 결과
성공. 누적 호출 **2/2**(시도 1 = 구현·배선, 시도 2 = checkpoint 비단락 실행과 target Resume·save RAM
증거 보완). 시도 1의 구현·테스트는 되돌리지 않고 그대로 이어서 수정했다.

## 시도 1 (보존)
- `Screen::Core(InGameMenu)` + App 소유 `CorePicker`, Core 행 open 시 1회 설치 후보 계산, 단일 후보
  플랫폼은 `core-no-alternatives`, draw/입력/pause 배선.
- 전환 순서: checkpoint → `stop()` → `Session::start_named`(공용 `open`, fallback 없음) → settings 기록
  (플랫폼 기본 core는 `None`) → target 자신의 Resume만 load → Session+consumer 설치·`Open` 하나·
  undo/latch 삭제 → `Playing`.
- 실패별 토스트/복구(`core-checkpoint-failed`, `core-switch-failed`, `core-setting-save-failed`,
  `core-recovery-state-failed`, `core-recovery-failed`)와 영·한 key 6개, `core_picker_app.rs` 13 테스트.
- 시도 1의 문제: (a) checkpoint가 `flush_save().and_then(save_state)`라 flush 실패 시 Resume을 시도조차
  하지 않았고, (b) 플랫폼 기본 core로 돌아가는 전환에서 **유효한 target Resume**을 준비·비교하지 않았고,
  (c) save RAM flush를 증명하는 테스트가 없었다.

## 시도 2 delta
- **app.rs**: checkpoint를 두 단계로 분리했다. `flush_save`와 `save_state(Resume)`를 **각각 한 번씩**
  평가하고, 실패한 단계마다 개별 로그를 남긴 뒤(`…save RAM…`, `…resume…`) 실패 목록을 요약해
  `core-checkpoint-failed`로 남는다. 둘 다 성공한 경우에만 Session을 내려놓고 전환한다. 단계 실패를
  다른 단계 오류로 덮지 않는다. `Session::stop` 계약은 그대로다.
- **core_picker_app.rs**: `seat_to_playing`/`play_then_leave` helper를 추가하고, 위 (b)를
  `going_back_to_the_platforms_own_core_loads_its_own_resume_and_keeps_the_rest`로 확장, (c)를
  `a_checkpoint_that_cannot_write_its_resume_still_flushes_the_save`로 신규 작성(총 14 테스트).

## 최종 순서 (flush와 Resume 독립 실행)
1. 선택 core == 현재 core → 무변경 종료.
2. 살아 있는 Session에서 **save RAM flush 시도** → 실패해도 계속 진행, 실패 기록.
3. 같은 Session에서 **Resume 저장 시도** → 실패해도 위 결과를 덮지 않고 별도 기록.
4. 둘 중 하나라도 실패 → Session/settings/sink/screen 유지, `core-checkpoint-failed`.
5. 둘 다 성공 → `stop()`으로 old core 내려놓기 → `start_named` → settings 기록 → target Resume(자기
   namespace만) → Session+consumer 설치·`Open` 하나.

## 유효한 target Resume 증거
플랫폼 기본 core로 돌아가는 테스트에서: mGBA로 플레이 후 MENU hold로 나가 mGBA Resume을 만들고 그
bytes를 기록 → 카드 ini에 `core = gpsp_libretro`(+`scale`/`overscan`/미지 key)를 써서 gpSP로 0.4초
더 진행 → Core 행에서 Up+A로 mGBA 복귀. 전환 직후 `frames_run()==0`(코어 frame 미진행)인 상태에서
quick save한 slot 1 bytes가 **기록해 둔 mGBA Resume bytes와 동일**함을 확인했다. 동시에
`settings.core == None`, `scale`/`overscan`/`future_thing` ini key 보존, gpSP checkpoint Resume 존재
및 두 core Resume bytes가 서로 다름을 확인했다.

## save RAM round-trip: 확인됨 (한계 포함)
- `arm.gba`는 실제로 save RAM을 노출한다. vendored mGBA source에서 `retro_get_memory_data(SAVE_RAM)`이
  GBA에서 항상 non-null 매핑 버퍼를 돌려주고(`libretro.c:2053`에서 `GBA_SIZE_FLASH1M` 매핑),
  `retro_get_memory_size`는 AUTODETECT 상태에서 `GBA_SIZE_FLASH1M`을 보고한다. 별도 signature ROM을
  만들 필요가 없었다(저장소에 ROM을 추가하지 않았다).
- 신규 테스트: 게임을 나가 `Saves/GBA/arm.sav`(131072 bytes, 실측)를 만들고 그 bytes를 기록 →
  **save 파일과 Resume 파일을 모두 지운 뒤** 재입장 → Resume `.tmp`를 directory로 막아 checkpoint의
  Resume 단계만 실패시킴 → 전환이 시작되지 않고(`core-checkpoint-failed`, Session mGBA 유지, picker
  유지, Resume 파일 없음) **save 파일은 같은 bytes로 다시 기록됨**을 확인. 즉 flush는 Resume 실패와
  무관하게 실행된다.
- 한계: save RAM **내용의 변화**를 쓰는 게임이 아니므로 round-trip은 “코어가 들고 있는 버퍼를 카드에
  그대로 다시 쓴다”까지의 증명이다(코어 내부 값 변경→파일 반영까지는 아님). gpSP의 SaveRam 노출은
  이 경로에서 쓰이지 않아 확인하지 않았다.

## 완료 기준 (원문 순서, 마지막 코드 변경 뒤, 이후 코드 변경 없음)
1. `cargo fmt --all -- --check` → exit 0, 출력 없음.
2. `cargo test -p slot2 -p slot2-i18n` → exit 0. `test result:` 24줄, 합계 **217 passed / 0 failed /
   0 ignored**(시도 1의 216 + 신규 flush 테스트 1; 유효 target Resume 증거는 기존 테스트 확장이라 수 불변).
   마지막 줄: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in
   0.00s`(slot2_i18n doc-tests). `core_picker_app` 14 passed.
3. `cargo clippy -p slot2 -p slot2-i18n --all-targets -- -D warnings` → exit 0,
   `Finished \`dev\` profile … in 8.44s`.
- skip 여부: `core_picker_app`을 `--nocapture`로 재실행해 `skipping` 0건, `slot2: playing arm` 15줄,
  14 passed — mGBA↔gpSP 전환·복구 테스트가 실제 core로 실행됐다.

## 수정 파일
- 시도 1: `crates/slot2/src/app.rs`, `crates/slot2/src/session.rs`(start_named/공용 open/UnsupportedCore),
  `crates/slot2/tests/core_picker_app.rs`(신규), `assets/lang/en.ftl`·`ko.ftl`, `crates/slot2-i18n/tests/i18n.rs`.
- 시도 2: `crates/slot2/src/app.rs`(checkpoint 두 단계 분리), `crates/slot2/tests/core_picker_app.rs`
  (helper 2개, 테스트 1개 확장 + 1개 신규), `tasks/61-core-picker-app-wiring.worker-result.md`(이 보고서).
- 이번 시도에서 `session.rs`·i18n·store·UI·registry는 변경하지 않았다. 커밋·푸시·네트워크·실기·공용
  설정 변경 없음. 검증 뒤 코드 변경 없음.

## 남은 위험
- 복구 실패 경로의 `SinkRequest::Close`는 다음 tick의 eject SFX `Open`으로 곧바로 대체된다(기존 꺼내기
  경로와 동일). sink 한 칸을 게임 오디오와 SFX가 공유하는 구조 자체는 그대로다.
- `core-recovery-state-failed`는 “변경되지 않은 settings가 가리키는 core”가 checkpoint를 쓴 core와
  달라야 도달 가능하다(라이브러리 추가/삭제, 읽을 수 없게 된 settings). 정상 상태에서는 checkpoint가
  방금 쓴 Resume이 항상 읽히므로 다른 경로로는 만들 수 없다.

## 소요 시간
누적 약 44분(시도 1 약 34분 17:12–17:46, 시도 2 약 10분 17:57–18:07 KST). 누적 호출 2/2.
