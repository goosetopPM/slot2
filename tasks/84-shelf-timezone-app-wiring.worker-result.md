# Task 84 — 작업자 결과

- **누적 호출 횟수:** 1/2
- **결과:** 성공 (완료 기준 6개 명령 모두 종료 0)
- **소요 시간:** 약 14분 (13:15 → 13:29 KST, 최종 검증 순차 실행 포함)

## 구현 요약

- `Screen`에 계약대로 `Shelf(ShelfMenu)`와 `Timezone(ShelfMenu, TimezoneMenu)`를 추가했다. 기존
  `Clone + Copy + Debug + PartialEq + Eq`는 그대로다(두 메뉴 모두 `Copy`).
- 입력: `Screen::List + Tap(Menu)`만 `Shelf(ShelfMenu::new(ShelfAvailability::timezone_only()))`로
  간다. 기존 `Hold(Menu) → PowerMenu`는 손대지 않았고, 홀드가 발화하면 릴리스가 탭이 되지 않는
  gesture 계약 때문에 두 동작이 서로를 만들지 않는다. Splash/Inserting/Ejecting/Playing의 Menu
  동작은 그대로다.
- Shelf: Up/Down은 `ShelfMenu`의 이동, A는 `selected() == Some(TimeZone)`일 때만 시간대 화면을 열고
  그 밖의 버튼은 no-op, B/Menu는 `List`로 돌아간다.
- Timezone: Left/Right/Up/Down은 `TimezoneMenu`를 먼저 움직인 뒤 `clock::set_utc_offset_min`으로 즉시
  runtime에 반영하고 같은 `Screen::Timezone(parent, menu)`에 다시 담는다(따라서 HUD가 같은 프레임부터
  미리보기 값을 읽는다). B/Menu는 write 없이 `original` runtime을 복원하고 parent Shelf로 돌아간다.
  A는 `!changed()`면 write 없이 parent Shelf, changed면 `write_utc_offset_minutes(selected)`를 정확히
  한 번 호출한다. 성공하면 runtime을 유지한 채 parent Shelf, 실패하면 `original` runtime과
  `TimezoneMenu::new(original)`을 복원하고 `Screen::Timezone`에 남아 `timezone-save-failed` toast를
  띄운다.
- 동작은 `open_shelf_menu`/`open_timezone_menu`/`move_timezone`/`apply_timezone`/`cancel_timezone`/
  `abandon_timezone` 여섯 private helper로 나눴다. setter 거부는 어느 경로에서도 `unwrap()`하지 않고
  한 줄 stderr 후 복원한다. `App`에 timezone 복사본이나 저장 scheduler는 추가하지 않았다(runtime
  source는 `clock::utc_offset_min()` 하나).
- draw: `Screen::List | Shelf(_) | Timezone(..)`가 **같은** wallpaper + shelf를 한 번 그리고,
  Shelf는 `ShelfMenu`만, Timezone은 `TimezoneMenu`만 그 위에 올린다(parent Shelf panel을 겹쳐 그리지
  않음). 두 메뉴 draw는 hold bar와 toast보다 먼저 호출되어 그 둘이 위에 남는다. HUD 표시 조건에
  Shelf/Timezone을 추가했고 `audio_paused`의 game-menu 집합은 그대로 뒀다.
- `timezone-save-failed`를 en/ko pack에 직접 추가하고 i18n 테스트에 정확 문구 검증을 넣었다.

## 변경 파일

- 수정: `crates/slot2/src/app.rs` (Screen variant 2개, 모듈 doc의 화면 map, List tap Menu 분기,
  Shelf/Timezone 입력 arm, helper 6개, draw의 List/Shelf/Timezone 공통 바탕 + 메뉴 layer, HUD 조건)
- 신규: `crates/slot2/tests/timezone_menu_app.rs` (`#[test]` 정확히 1개)
- 수정: `assets/lang/en.ftl`, `assets/lang/ko.ftl` (오류 key 1개씩)
- 수정: `crates/slot2-i18n/tests/i18n.rs` (timezone 테스트에 오류 문구 단언 추가)
- 신규: `tasks/84-shelf-timezone-app-wiring.worker-result.md`

`slot2-store`, `slot2-platform`, `slot2-ui`, host/device loop는 수정하지 않았다. 새 Screen variant
때문에 깨진 기존 테스트는 없어서(컴파일 1회로 확인) 기존 `*_app.rs`를 고치지 않았다.

## 완료 기준 명령 (마지막 코드 변경 뒤 순서대로)

| 명령 | 종료 코드 | 마지막 결과 줄 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | (출력 없음) |
| `cargo test -p slot2 --test timezone_menu_app` | 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s` |
| `cargo test -p slot2` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (Doc-tests slot2, 최종 줄) |
| `cargo test -p slot2-i18n` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (Doc-tests slot2_i18n, 최종 줄) |
| `cargo check -p slot2 --no-default-features --features device` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 5.60s` |
| `cargo clippy -p slot2 -p slot2-i18n --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 19.44s` |

- `timezone_menu_app`: **1 passed / 0 failed / 0 ignored** (파일당 `#[test]` 1개).
- `cargo test -p slot2`: **314 passed / 0 failed / 0 ignored** (lib 33, main bin 0, integration 281,
  doc-tests 0; 28개 result 줄). Task 81의 313에 새 test 1개가 더해진 값과 일치한다.
- `cargo test -p slot2-i18n`: **33 passed / 0 failed / 0 ignored**.
- core-dependent skip: **0** (실행 출력에 `skipping` 없음). `timezone_menu_app`는 core/ROM이 필요 없다.
- workspace test와 device 배포 이미지 생성은 실행하지 않았다.

## 검증 항목별 결과 (통합 test가 검증한 것)

1. List에서 Menu 탭 → `Screen::Shelf(ShelfMenu::new(timezone_only()))`, Up/Down 뒤에도 선택은
   `TimeZone` 하나.
2. List에서 Menu 홀드 → `PowerMenu`, 릴리스 뒤에도 그대로(Shelf를 거치지 않음).
3. Shelf에서 B와 Menu 탭 각각 → `Screen::List`.
4. Shelf A → `Screen::Timezone(parent, TimezoneMenu)`이고 `original == selected == runtime`, parent는
   `TimeZone` 행 유지.
5. Right/Down/Up/Left가 각각 555/495/555/540으로 menu와 runtime을 동시에 바꾸고, 매 입력마다 카드
   bytes 불변.
6. B cancel과 Menu cancel 각각 runtime을 original(540)로 되돌리고, 파일 무변경, toast 없음,
   `TimeZone` 행의 parent Shelf로 복귀.
7. unchanged A → 파일 무변경, toast 없음, parent Shelf 복귀.
8. changed A 성공 → `utc_offset_minutes = 555` 저장, runtime 555 유지, parent Shelf 복귀, toast 없음,
   기존 `volume = 30`/`language = ko`/`future_key = something` 보존, `App::volume`도 30 유지.
9. invalid UTF-8 `slot2.ini`(portable blocker)에서: write 실패 → runtime 0 복원, 카드 getter 0,
   화면 `Timezone`에 `original == selected == 0`, `timezone-save-failed` toast, 손상 bytes와
   `System` 디렉터리 entry 수 불변, volume runtime도 default 유지. 실패 뒤에도 화면이 살아 있어
   다시 미리보기(15)하고 B로 복귀(0) 가능.
10. `timezone-save-failed` 영문·한글이 pack 문구와 정확히 일치.
11. 세 프레임 모두 `Op::Clear` 없음. Shelf/Timezone 프레임의 첫 mark들은 List 프레임의 shelf 바탕
    mark들과 op 단위로 동일(upload는 제외해 비교). dim이 전체 패널에 그려지고 그 앞에 shelf 바탕이
    있다. Shelf 프레임에 시간대 panel 0개, Timezone 프레임에 parent Shelf panel 0개(각자 panel 1개).
    HUD quad 수가 List/Shelf/Timezone에서 동일하고 0이 아니다.
12. 위 모든 단계에서 `App::volume`과 카드의 volume은 변하지 않았고, 시간대만 설정한 카드의 파일은
    `volume = 30\n` 그대로다.

## 계약 검토

- 구현을 막는 계약 오류는 없었다. 다만 §2의 "예상 밖 setter 거부는 ... `timezone-save-failed`를
  표시한다"는 문구는 저장 실패용 문구("Could not save the time zone; restored the previous value")를
  preview 거부에 재사용하라는 뜻이라 문구와 상황이 완전히 맞지는 않는다. 명세대로 구현했고, 이 분기는
  store·menu·clock의 범위가 같아 실제로는 도달하지 않는다(방어적 경로).
- "clamp로 값이 변하지 않은 경계 입력은 파일을 쓰지 않는다"는 A 저장 조건으로 해석했다. 경계 방향
  입력은 같은 값으로 setter를 한 번 더 호출할 뿐 파일을 건드리지 않는다(멱등).

## 남은 위험

- `ShelfAvailability`는 아직 `timezone_only()`만 쓰인다. 언어/기본 화면/부팅 로고/연동/정보 화면이
  생기면 각 생성 지점에서 해당 필드를 켜야 하고, 그전까지 다섯 행은 계속 흐리게 남는다.
- HUD는 이 세 화면에서 계속 그려지므로 메뉴 panel 위에 시계가 겹쳐 보일 수 있다(코너 여백이 panel
  바깥이라 실제로는 겹치지 않지만, panel을 키우면 다시 확인해야 한다).
- 실기(Pi/RG SP) 검증과 실제 GL 창은 이 태스크에서 하지 않았다.
