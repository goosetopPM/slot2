# Task 52 워커 결과 — 치트 메뉴 App 연결

**성공. 누적 호출 1/2.** 지정 검증 4종 모두 종료 0. mGBA 실코어가 있는 환경에서 신규 테스트가
skip 없이 실행됐다(로그에 `slot2: playing arm`).

## Screen 상태와 열기·닫기·재진입

- `Screen::Cheats(InGameMenu, CheatMenu)` 추가(Copy 유지). 인게임 메뉴에서 `InGameChoice::Cheats`에
  A를 누르면 `Session::cheats().len()`으로 `CheatMenu::new(len)`을 만들어 연다. Session이 없으면
  화면을 바꾸지 않는다(빈 Session·가짜 목록 없음).
- 열 때 선택은 index 0에서 시작한다(Task 51 계약). B/MENU는 보존한 부모 인게임 메뉴로 돌아가며
  Cheats 행을 유지하고, 같은 행에서 A를 누르면 선택 0으로 새로 열리되 표시 상태는 Session 값을
  읽는다.
- Up/Down은 `CheatMenu::up/down`만 호출하고 화면을 유지한다. 다른 버튼은 무시하며 입력이 core로
  가거나 프레임을 전진시키지 않는다(`run_frame`은 `Screen::Playing`에서만 동작).

## 토글·rollback·toast

- A는 `selected_index()`가 있을 때 그 index의 Session `enabled`를 읽어 반대로 뒤집고
  `Session::set_cheat_enabled`를 정확히 한 번 호출한다. 성공하면 같은 화면·같은 선택에 머물고,
  draw가 Session slice를 읽으므로 새 on/off가 즉시 반영된다.
- 빈 목록에서 A는 no-op이며 toast를 만들지 않는다.
- 범위 밖 index(화면과 Session 길이 불일치)는 Session을 건드리지 않고 stderr에 cart/index와 이유를
  남기고 localized `cheat-toggle-failed`(`$title`) 토스트를 띄운다. 메뉴는 열린 채 같은 선택에
  머문다. 세션 내부의 재적용 실패·복구 실패 rollback은 Task 50의 구현을 그대로 쓰며, 그 오류도 같은
  토스트로 분류된다.
- 토글·탐색·닫기·재열기는 Session 메모리만 건드린다. `.cht` bytes는 start 이후 읽지도 쓰지도
  않아 전후 동일함을 테스트로 확인했다.

## launch 오류 분류

`Session::start`의 `Error::Store`(Task 50에서 손상/판독 불가 `.cht`가 내는 유일한 store 경로)를
`core-missing`/`cart-broken`과 분리해 localized `cheat-load-failed`(`$title`)로 분류한다. 손상
파일로 launch하면 Session 없이 토스트가 `cheat-load-failed`로 뜨고 화면은 Ejecting/List로 돌아간다
(`cart-broken` 아님).

## pause·draw 순서

- 치트 화면은 게임 overlay 계열에 포함: wallpaper/shelf 미표시, Session 마지막 frame → `CheatMenu`
  순서, HUD 시간 배지 숨김, hold progress·toast는 메뉴보다 위. `clear` 없음, 부모 메뉴·스위처·상단
  band 미표시를 좌표로 확인했다.
- core frame·audio 정지: 치트 화면에서 `run_frame`을 0.3초 반복 호출해도 `frames_run`과 audio
  생산량이 변하지 않고 sink 요청도 없다. `audio_paused()`에 `Screen::Cheats`를 포함했다.
- Session이 사라진 상태에서도 A/draw가 panic하지 않는다(draw는 빈 slice로 그린다).

## 검증 (최종 코드 상태, 명세 순서)

- `cargo fmt --all -- --check` → 종료 **0**.
- `cargo test -p slot2 --test cheat_menu_app` → 종료 **0**, 마지막 결과 줄
  `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.42s`.
- `cargo test -p slot2-i18n --test i18n` → 종료 **0**,
  `test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s`.
- `cargo clippy -p slot2 --all-targets -- -D warnings` → 종료 **0**(경고·오류 없음).
- 최종 검증 후 코드 변경 없음.

## 생성·수정 파일

- `crates/slot2/src/app.rs`: `Screen::Cheats`, 열기/토글 helper, 입력 arm, `audio_paused`, draw
  경로, launch 오류 분류, 모듈 doc.
- 신규 `crates/slot2/tests/cheat_menu_app.rs`(11 테스트).
- `assets/lang/en.ftl`·`ko.ftl`: `cheat-toggle-failed`, `cheat-load-failed`($title).
- `crates/slot2-i18n/tests/i18n.rs`: 두 문구의 영·한 치환 테스트.
- `session.rs`·`slot2-ui`·`slot2-store`·`slot2-retro`는 수정하지 않았다.

## 계약 우려 / 남은 위험, 그리고 작업 중 처리한 잠금

- 남은 위험: `Error::Store`를 치트 파일 전용으로 분류하는 근거는 `Session::start`의 store 호출이
  `read_cheats` 하나뿐이라는 점이다. 나중에 start가 다른 store 실패를 내면 그 오류도
  `cheat-load-failed`로 표시된다(코드 주석에 명시).
- 작업 중 처리: 1회차(중단된 Task 46)가 남긴 빈 잠금 디렉터리 `crates/slot2/src/app.rs.lock` 때문에
  `edit` 도구가 `owner_mismatch`로 계속 실패했다. 해당 디렉터리가 비어 있고 app.rs가 5시간 동안
  변경되지 않은 것을 확인한 뒤(해시 동일) 문서화된 복구 절차대로 디렉터리를 제거했고, 이후 정상
  편집됐다. 실행 중인 다른 프로세스는 종료하지 않았다.
- 소요 약 13분(상한 45분 내).
