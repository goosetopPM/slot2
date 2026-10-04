# Task 59 — worker result (Session/App 코어별 스테이트 배선)

## 결과
성공. 누적 호출 **2/2**: 시도 1은 코드 작업 도중 provider 400 `read body failed`로 중단되어 보고서를
남기지 못했고, 시도 2(이번)에서 그 working tree를 이어서 마무리했다.

## 시도 1 보존과 이번 delta
- 시도 1의 `session.rs` resolver/`Session.namespace`, `app.rs` scoped 배선, `tests/session.rs`·
  `resume_app.rs`·`quick_state_app.rs`·`state_switcher_app.rs`의 scoped 전환은 revert/clean/reset
  없이 그대로 사용했다. 재구현 없음.
- 이번에 고친 것:
  1. `quick_state_app.rs`의 `numbered()` helper가 평면 `card.list_states`를 읽던 것을 실행 중 core의
     `scoped_list_states(cart, &states_ns())`로 변경 — 시도 1 중단의 직접 원인.
  2. `session.rs`에 `#[cfg(test)]` 신규: resolver 단위 테스트 5개(코어 파일 없는 상태라 스텁 이름만 사용).
  3. `app.rs`의 draw-cache fixture(`the_shelf_hint_follows_the_scan_and_not_the_disk`)가 hint 대상을
     평면 경로가 아니라 scan이 실제로 보는 scoped 경로에 쓰고 지우도록 수정(제품 scoped 동작 fixture).
  4. `crates/slot2/tests/core_state_routing.rs` 신규(5 테스트).
  5. `docs/DESIGN.md` 카드 레이아웃: `States` 트리에 `<core>` 단계 추가 + 코어 격리·legacy 이전 문단.
- 남은 평면 호출은 `app.rs`의 `#[cfg(test)]`(1467행 이후)뿐이며, legacy migration·draw cache·평면
  `StateBackup` 자체를 검증하는 fixture라 의도적으로 유지했다.

## resolver와 namespace 규칙
- `session::resolve_core(card, cart, core_dir, quiet)` 한 곳을 `Session::start`와 `App::rescan`이 공유.
  반환은 최종 dylib 경로, `Option<CoreId>`, 유효 `StateNamespace`.
- 설정의 core 값은 core 디렉터리 안 **파일 이름**으로만 취급: 빈 문자열·`.`·`..`·`/`·`\`·`:`가 있으면
  경로로 보고 기본 core로 fallback. 없는 이름, 플랫폼 미지원 공식 core도 기본 core. 존재하는 미지원
  외부 library는 그대로 연다. rescan은 `quiet=true`로 fallback 로그를 반복하지 않는다.
- 공식 core namespace = `CoreId::base_name()` (`mgba_libretro`, `gpsp_libretro`, …).
- 외부 library = stem의 lowercase canonical이 `StateNamespace` 규칙을 만족하면 그대로, 아니면 고정
  변환으로 `external_<첫 19바이트 hex>_<fnv1a64 hex>`(≤64 bytes). `DefaultHasher`/seed 미사용.
  알고리즘은 테스트로 고정: `core!_libretro` → `external_636f7265215f6c6962726574726f_73fdf30215bf810f`,
  `My Core_libretro` → `external_4d7920436f72655f6c6962726574726f_be538c6f12df829e`.
- 플랫폼 변환(`retro_platform`)과 기본 core namespace(`default_namespace`)도 같은 helper 계층에 있어
  App이 registry 규칙을 복제하지 않는다.

## legacy 이전 시점과 실패 처리
- 시점: `App::rescan`에서 cart마다 (1) 플랫폼 기본 core namespace 계산 (2) `adopt_legacy_states`
  (3) resolver가 정한 namespace의 Resume 존재 캐시. 대체 core가 선택돼 있어도 legacy는 항상 기본 core
  소유로 간다.
- 성공/빈 no-op은 조용하다. 충돌·I/O 실패는 한 줄 로그 후 평면 원본을 그대로 두고 scoped 목적지를
  덮지 않는다(rename 전 전수 검사 + 부분 실패 rollback). 이후 hint와 제품 state 동작은 선택된 scoped
  namespace만 본다. draw는 계속 파일시스템을 읽지 않는다.

## Session/App scoped 전환 지점과 다른 core 보존 증거
- Session: `save_state`·`load_state`·`stop`의 Resume가 모두 `scoped_*`이고, `state_namespace()` accessor를
  제공한다. `core_id()==None`인 외부 core도 resolver가 정한 namespace를 쓴다.
- App: `resume_into`(scoped path 확인→load), `discard_resume`(scoped delete), switcher 열기/refresh,
  numbered load, X 삭제·실패 후 refresh, Y undo 후 refresh(`StateBackup`은 자기 namespace 소유),
  quick save(`scoped_next_state_number`)·quick load(`scoped_list_states`).
- 증거(`core_state_routing.rs`, 전부 통과):
  - 같은 GBA cart의 같은 slot 1·Resume에 mGBA/gpSP가 다른 bytes를 쓰고, 각 `state_namespace()`가 실제
    core와 일치하며, gpSP가 mGBA Resume를 load하려 하면 오류(바이트를 넘기지 않음).
  - 평면 legacy Resume/numbered가 rescan에서 mGBA namespace로 이동(gpSP 선택 상태), gpSP의 hint·
    numbered 목록에는 나타나지 않음.
  - 이동 충돌 시 평면 원본·scoped 목적지 모두 보존, launch는 scoped를 읽음(로드 성공 후 저장 bytes 일치).
  - Tap(A)는 대체 core Resume로만 resume, Hold(A) fresh는 그 core Resume와 png만 삭제하고 기본 core
    Resume/png는 보존.
  - quick save/switcher 목록·load·delete·undo가 현재 namespace만 다루고, mGBA slot 1과 png가 보존되며
    다시 mGBA로 돌아가면 자기 Resume/slot이 다시 보인다.
- 기존 회귀(Resume race·`resume-load-failed`·A-hold suppression·state load rewind reset·undo 만료·
  draw의 no-filesystem-access)는 기존 테스트가 그대로 통과한다.

## 완료 기준 명령 (원문 순서, 마지막 코드 변경 뒤 실행 / 이후 코드 변경 없음)
1. `cargo fmt --all -- --check` → exit 0, 출력 없음.
2. `cargo test -p slot2` → exit 0. `test result:` 줄 20개, 합계 **176 passed / 0 failed / 0 ignored**.
   마지막 줄: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`(doc-tests).
   - 실코어 skip 없음: `vendor/mgba_libretro.dll`·`vendor/gpsp_libretro.dll` 사용, `--nocapture` 로그에
     "skipping" 0건, `slot2: playing arm` 라인 다수 확인.
   - 이번 시도 개입 직전 같은 명령은 166 passed → +10(단위 5, routing 5).
3. `cargo clippy -p slot2 --all-targets -- -D warnings` → exit 0, `Finished \`dev\` profile … in 0.95s`.

## 생성·수정 파일
- 생성: `crates/slot2/tests/core_state_routing.rs`, `tasks/59-core-scoped-state-routing.worker-result.md`.
- 수정(이번 시도): `crates/slot2/tests/quick_state_app.rs`, `crates/slot2/src/session.rs`,
  `crates/slot2/src/app.rs`, `docs/DESIGN.md`.
- 수정(시도 1 유지): `crates/slot2/src/app.rs`·`session.rs`, `tests/session.rs`·`resume_app.rs`·
  `state_switcher_app.rs`.
- 커밋·푸시·네트워크·실기·공용 설정 변경 없음. 검증 뒤 코드 변경 없음.

## 계약 의문 / 남은 위험
- `external_namespace`가 stem을 lowercase로 접으므로 대소문자만 다른 두 라이브러리 stem은 같은 namespace를
  공유한다(계약이 소문자 canonical stem을 요구하므로 합치지만, 대소문자 구분 파일시스템에서는 이론적 충돌).
- rescan마다 cart별 평면 디렉터리 `read_dir`가 1회 추가된다(이동 후엔 부재라 비용 미미). draw 경로는
  여전히 FS 미접근이며 이번에 그 단언을 scoped 경로 기준으로 강화했다.

## 소요 시간
약 18분(13:31–13:49 KST). 누적 호출 2/2.
