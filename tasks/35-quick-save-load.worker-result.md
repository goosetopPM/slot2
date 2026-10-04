# Task 35 결과 — 퀵 세이브/로드 배선

2026-09-25 19:38~19:52 (로컬, 약 14분). 판정: **성공.**

## 변경 파일

- `crates/slot2/src/app.rs` — `Action::Chord` 처리와 두 메서드.
  - `(Screen::Playing, Chord(R1)) → quick_save()`, `(Screen::Playing, Chord(L1)) → quick_load()`.
    화면이 정확히 `Playing`일 때만 매칭되므로 다른 화면에서는 chord가 아무 일도 하지 않는다.
  - `quick_save`: 세션의 cart를 읽어 `Card::next_state_number` → `StateKind::Numbered(n)` →
    `Session::save_state`. 성공 시 `state-saved` 토스트(title, n), 실패 시 로그 1줄과
    `state-save-failed` 토스트. 화면·세션·싱크 요청은 건드리지 않는다.
  - `quick_load`: `Card::list_states`에서 `Numbered`만 골라 최대값을 선택(Resume 제외, mtime 무시).
    없으면 `states-empty` 토스트만 띄우고 계속 플레이한다. 있으면 `Session::load_state`,
    성공 시 `state-loaded` 토스트(n), 실패 시 로그 1줄과 `state-load-failed` 토스트.
  - 모듈 주석의 `Screen::Playing` 항목에 두 chord와 토스트 키를 추가.
- `crates/slot2/src/session.rs` — `Session::load_state`가 `resampler.reset()`에 이어
  `rewind.clear()`를 호출한다. 상태 점프는 되감기 사슬의 끝이며, 로드 이전 미래의 캡처가
  남아 있으면 플레이하지 않은 게임으로 되돌아갈 수 있다.
- `assets/lang/en.ftl`, `assets/lang/ko.ftl` — 4개 키 추가: `state-loaded`, `states-empty`,
  `state-save-failed`, `state-load-failed`. 기존 `state-saved`는 그대로 재사용한다.
- `crates/slot2/tests/quick_state_app.rs` — 신규 통합 테스트 6개(실제 mgba 코어, 없으면 skip).
- `crates/slot2/tests/session.rs` — `loading_a_state_clears_the_rewind_chain` 1개 추가.
- `crates/slot2-i18n/tests/i18n.rs` — 새 키 4개의 en/ko 문구와 `$n` 보간 검증 1개 추가.
- `tasks/35-quick-save-load.worker-result.md` — 이 보고서.

`Session::cart()`가 이미 있어 새 접근자를 추가하지 않았다. `slot2-input`, `slot2-store`,
`slot2-ui`, 매니페스트, 문서, 런타임 루프는 수정하지 않았다. 기존 테스트는 약화·수정하지 않았다.

## 세이브/로드 선택과 피드백

- 저장: `next_state_number` = 기존 번호의 최대값+1(없으면 1). 성공 시 상태 파일과 썸네일 PNG를
  쓰고 `state-saved`("Saved {title} to slot {n}")를 띄운다. 화면·세션·오디오 요청 변화 없음.
- 로드: 번호가 있는 상태 중 최대 번호. Resume은 mtime이 가장 최신이어도 후보가 아니다.
  성공 시 `state-loaded`("{n}번 슬롯을 불러왔습니다"), 번호 상태가 없으면 `states-empty`,
  읽기/복원 실패 시 `state-load-failed`. 모두 화면은 Playing, 세션 유지, 싱크 요청 없음.
- 실패는 로그 한 줄(`slot2: quick save failed: …` / `quick load failed: …`)과 토스트로 끝나고
  게임은 계속된다. 실패 원인을 제거하면 다음 chord는 정상 동작한다(테스트로 확인).
- 두 chord 모두 Playing 이외 화면(셸프, 인서트/이젝트 애니메이션, 파워 메뉴, 인게임 메뉴)에서
  아무 동작도 하지 않는다.

## 검증

마지막 코드 변경 뒤 순서대로 실행했다.

1. `cargo fmt --all -- --check` — 종료 **0**, diff 없음.
2. `cargo test -p slot2 -p slot2-i18n` — 종료 **0**, 실패 0.
   - `tests/quick_state_app.rs` 6개 통과: 두 번 저장(1·2번 슬롯 + 썸네일 + 토스트 + 세션·싱크
     불변, 화면에 토스트가 게임 프레임 위로 그려짐), 최대 번호 선택(1번과 Resume을 조작된
     상태로 두고 2번만 정상 → `state-loaded`), 실제 코어 상태 복원(저장 시점 B와 로드 후 재저장
     바이트가 동일, 로드 직전 상태 C와는 다름), 번호 상태 없음(`states-empty` + 계속 플레이),
     저장·로드 실패 후 일관성 유지와 복구, 인게임 메뉴 위에서 두 chord 무시(세션 있는 상태).
   - `tests/session.rs` 17개 통과(신규 `loading_a_state_clears_the_rewind_chain` 포함:
     캡처가 있는 상태에서 로드하면 `rewind_state() == (0,0)`, `rewind_step()` false).
   - `slot2-i18n` 18개 통과(신규 메시지 4종 en/ko 및 `$n` 보간).
   - 기존 `slot2`/`slot2-i18n` 테스트 전부 통과(회귀 없음).
3. `cargo clippy -p slot2 -p slot2-i18n --all-targets -- -D warnings` — 종료 **0**, 경고 0.

최종 검증 이후 코드 변경 없음. 워크스페이스 전체 테스트와 `build/dist-device.ps1`은 실행하지
않았다. 실기·adb·SD 접근 없음.

## 남은 항목과 계약 의견

- 토스트 **인자**는 App의 공개 표면(`toast_key()`)으로 읽을 수 없다. 앱 테스트는 키를 확인하고,
  문구와 `$n` 보간은 `slot2-i18n` 테스트가 확인한다. 즉 "제목과 슬롯 번호가 토스트에 들어간다"는
  구성상 보장 + 메시지 테스트로 검증되며, 렌더된 문자열을 직접 읽어 확인하지는 않았다.
- 계약 1의 셸프/애니메이션/파워 메뉴 화면은 공개 API로 세션을 가질 수 없어(세션은 Playing과
  그 위의 InGame에만 존재) 실제 세션을 동반한 검증은 인게임 메뉴에서 했다. 나머지 화면은
  `Screen::Playing` arm과 세션 부재 시 조기 반환으로 구조적으로 막혀 있다.
- 상태 복원 테스트는 mGBA의 serialize→unserialize→serialize가 바이트 단위로 동일하다는 관찰에
  기댄다(이번에 확인됨). 다른 코어가 그 성질을 보장하지 않으면 그 테스트는 화면 기반 비교로
  바꿔야 한다.
- 저장 실패는 상태 폴더 자리에 파일을 놓아 `atomic_write`의 디렉터리 생성이 실패하게 만들어
  유도했다. 카드 쓰기 실패의 한 경로이며 모든 실패 원인을 덮지는 않는다.
- 새 키 이름(`states-empty`, `state-loaded`, `state-save-failed`, `state-load-failed`)은
  계약이 정하지 않아 기존 `state-saved`/`list-empty`/`core-missing` 명명을 따랐다.
- 상태 스위처 UI, 삭제, 30초 undo, 링 정책, Resume 동작, 상태 파일 이름, 썸네일 인코딩,
  번호 규칙은 손대지 않았다.
- 실행 시간 약 14분. 모델 토큰·비용 정보는 이 도구가 노출하지 않아 `unavailable`.
