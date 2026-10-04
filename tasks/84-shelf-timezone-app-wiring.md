# Task 84 — 선반 설정 메뉴와 시간대 App 배선

현재 checkout에서 직접 작업한다. Task 80~83의 global 시간대 store, runtime clock,
`TimezoneMenu`, `ShelfMenu`를 `App`에 연결한다. 선반에서 Menu를 탭해 설정을 열고, 시간대 조정은
HUD clock에 즉시 미리보기로 반영하며, 적용·취소·저장 실패의 파일/runtime/screen 상태를 하나의
계약으로 완성한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

필요한 부분만 읽는다.

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\80-global-timezone-settings-store.result.md`
- `C:\SLOT2\tasks\81-timezone-startup-app-wiring.result.md`
- `C:\SLOT2\tasks\82-timezone-menu-ui.result.md`
- `C:\SLOT2\tasks\83-shelf-menu-ui.result.md`
- `C:\SLOT2\docs\DECISIONS.md`의 D-25만
- `C:\SLOT2\crates\slot2\src\app.rs`의 `Screen`, List input, draw, Display 계열
  open/commit/cancel 방식, startup clock 적용 부분만
- `C:\SLOT2\crates\slot2-ui\src\shelf_menu.rs`
- `C:\SLOT2\crates\slot2-ui\src\timezone_menu.rs`
- `C:\SLOT2\crates\slot2-store\src\settings.rs`의 `read_utc_offset_minutes`와
  `write_utc_offset_minutes` 계약만
- `C:\SLOT2\crates\slot2-platform\src\clock.rs`의 offset getter/setter만
- `C:\SLOT2\crates\slot2\tests\timezone_startup_app.rs`의 Card/App helper와 process-global
  격리 방식만
- `C:\SLOT2\crates\slot2\tests\display_menu_app.rs`의 tap/draw helper와 저장 실패 검증 방식만
- `C:\SLOT2\assets\lang\en.ftl`, `ko.ftl`의 timezone 구역

Session/core, host/device loop, 다른 메뉴 구현, 워커 로그와 저장소 이력은 읽지 않는다. 직접 깨지는
App test가 있으면 해당 setup/단언 주변만 추가로 읽는다.

## 확정 UX

- 선반 `Screen::List`에서 **Menu 탭**은 최상위 `ShelfMenu`를 연다.
- 기존 **Menu 홀드 → PowerMenu**는 그대로다. 탭과 홀드가 서로의 동작을 만들면 안 된다.
- 이번 빌드에서 `ShelfAvailability::timezone_only()`를 사용한다. 여섯 행은 보이지만 시간대만 선택·진입
  가능하다.
- `ShelfMenu`에서 B 또는 Menu 탭은 선반 List로 돌아간다.
- 시간대 행에서 A는 현재 runtime offset을 original로 가진 `TimezoneMenu`를 연다.
- 시간대 메뉴의 방향 입력은 menu 값을 바꾼 뒤 `clock::set_utc_offset_min`으로 즉시 runtime에 반영한다.
  따라서 뒤의 선반 HUD clock이 같은 프레임부터 미리보기 값을 사용한다.
- 시간대 메뉴의 B 또는 Menu 탭은 파일을 쓰지 않고 original runtime 값을 복원한 뒤, 선택 행을
  유지한 상위 `ShelfMenu`로 돌아간다.
- A 적용은 변경이 없으면 파일을 쓰지 않고 상위 메뉴로 돌아간다.
- A 적용은 변경이 있으면 `Card::write_utc_offset_minutes(selected)`를 호출한다. 성공하면 selected
  runtime을 유지하고 상위 메뉴로 돌아간다.
- 저장 실패면 original runtime을 복원하고 `TimezoneMenu::new(original)`로 메뉴 값도 원상 복원한다.
  시간대 메뉴에 그대로 남아 재시도할 수 있어야 하며 `timezone-save-failed` toast를 표시한다.
- 저장 실패 후 화면의 선택값, runtime clock, 카드에 남은 값이 서로 다르게 보이는 상태를 허용하지
  않는다.

## 구현 계약

### 1. Screen과 진입 경로

`Screen`에 다음 의미의 `Copy` 가능한 variant를 추가한다. 이름과 payload는 그대로 사용한다.

```rust
Shelf(ShelfMenu),
Timezone(ShelfMenu, TimezoneMenu),
```

- `Screen`의 기존 `Clone + Copy + Debug + PartialEq + Eq`를 유지한다.
- `Screen::List + Tap(Menu)`만 `Screen::Shelf(ShelfMenu::new(
  ShelfAvailability::timezone_only()))`로 전환한다.
- Splash, Inserting, Ejecting, Playing의 Menu 동작은 바꾸지 않는다.
- Shelf의 Up/Down은 `ShelfMenu::up/down`, A는 선택된 `TimeZone`만 연다. 비활성 choice를 위해
  가짜 동작이나 toast를 만들지 않는다.
- Shelf의 B/Menu는 `Screen::List`다.
- Timezone의 Left/Right/Up/Down/A/B/Menu만 아래 계약대로 처리하고 다른 버튼은 no-op이다.

### 2. preview, cancel, apply

동작을 작은 App private helper로 나눠 input match에 파일/clock 처리 전체를 복제하지 않는다.

- 시간대 진입은 `clock::utc_offset_min()`을 한 번 읽어 `TimezoneMenu::new`에 전달한다. App struct에
  별도 timezone 복사본이나 저장 scheduler를 추가하지 않는다.
- 방향 입력은 `TimezoneMenu`의 해당 메서드를 먼저 호출하고, 결과 `selected()`를 runtime setter에
  전달한 뒤 갱신된 menu를 같은 `Screen::Timezone(parent, menu)`에 둔다.
- clamp로 값이 변하지 않은 경계 입력은 파일을 쓰지 않는다.
- platform setter는 menu/store와 같은 범위라 정상적으로 거부할 수 없다. 그래도 `unwrap()`하지 않는다.
  예상 밖 거부는 한 줄 stderr를 남기고 original runtime과 `TimezoneMenu::new(original)`을 복원하며
  `timezone-save-failed`를 표시한다. store/platform 상수나 범위를 고쳐 맞추지 않는다.
- B/Menu cancel은 `menu.original()`을 runtime에 복원하고 parent Shelf로 돌아간다. 파일 read/write,
  toast 생성, 환경변수 변경을 하지 않는다.
- A에서 `!menu.changed()`면 write 없이 parent Shelf로 돌아간다.
- A에서 changed면 `write_utc_offset_minutes(menu.selected())`를 정확히 한 번 호출한다.
  - 성공: preview runtime을 유지하고 parent Shelf로 돌아간다. 성공 toast는 없다.
  - 실패: 오류와 거부 값을 stderr 한 줄로 남기고, original runtime을 복원하고,
    `Screen::Timezone(parent, TimezoneMenu::new(original))`에 남고,
    `timezone-save-failed` toast를 표시한다.
- rollback setter도 `unwrap()`하지 않는다. compile-time 범위 계약상 original은 유효하다. 예상 밖
  거부는 stderr에 추가하되 panic하지 않는다.
- write 전에 preview가 적용돼 있다는 사실 때문에 실패 rollback을 생략하면 안 된다.

### 3. draw와 기존 동작 보존

- `Screen::Shelf`와 `Screen::Timezone`의 바탕은 `Screen::List`와 정확히 같은 wallpaper + shelf다.
  게임 frame, session overlay, 빈 화면을 바탕으로 그리지 않는다.
- base shelf를 한 번 그린 뒤 Shelf 화면은 `ShelfMenu`만, Timezone 화면은 `TimezoneMenu`만 그린다.
  Timezone 뒤에 parent ShelfMenu를 겹쳐 그리지 않는다.
- hold progress와 toast는 기존처럼 메뉴보다 위에 그린다.
- shelf HUD의 battery/time은 List, Shelf, Timezone에서 계속 보인다. Timezone preview 직후 clock은
  같은 runtime offset을 읽어 표시한다. UTC validity 판정 순서는 바꾸지 않는다.
- 이 두 메뉴는 게임 위 메뉴가 아니므로 `audio_paused`의 game-menu 집합에 넣지 않는다.
- 모든 `Screen` match를 새 variant에 맞게 명시적으로 갱신하되, 기존 Power/InGame/Display/Device,
  insert/eject, volume, rescan, session 동작은 바꾸지 않는다.
- OS clock, RTC, process timezone, 환경변수, file mtime을 직접 조작하지 않는다.

### 4. 오류 문구

두 built-in pack에 다음 key를 직접 추가한다.

| key | English | 한국어 |
|---|---|---|
| `timezone-save-failed` | `Could not save the time zone; restored the previous value` | `시간대를 저장하지 못해 이전 값으로 돌아갔습니다` |

기존 timezone 문구를 변경하지 않는다. 코드에서 사용자 문구를 조합하지 않는다.

## 통합 테스트 계약

`crates/slot2/tests/timezone_menu_app.rs`를 새로 만든다. runtime clock이 process-global이므로 이 파일의
`#[test]`는 **정확히 한 개**만 두고, helper를 사용해 다음을 순차 검증한다. 마지막에는 테스트 시작 전
runtime offset을 복원한다.

1. List에서 Menu 탭은 `Shelf(timezone_only)`를 열고, Up/Down 뒤에도 TimeZone만 선택된다.
2. List에서 Menu 홀드는 기존 PowerMenu를 열며 Shelf를 거치지 않는다.
3. Shelf의 B와 Menu 탭 각각이 List로 돌아간다.
4. Shelf의 A가 `Timezone(parent, TimezoneMenu)`를 열고 original/selected가 현재 runtime과 같다.
5. 네 방향 입력이 Task 82의 ±15/±60 의미로 menu와 runtime을 같은 값으로 즉시 바꾼다. 이때 카드
   bytes는 불변이다.
6. B와 Menu cancel 각각이 original runtime을 복원하고, 파일을 쓰지 않으며, 같은 TimeZone 행의
   parent Shelf로 돌아간다.
7. unchanged A는 파일을 건드리지 않고 parent Shelf로 돌아가며 toast가 없다.
8. changed A 성공은 선택값을 카드와 runtime에 남기고 parent Shelf로 돌아간다. 기존 volume과
   unknown key가 보존되고 성공 toast가 없다.
9. original 0으로 시작한 별도 Card/App에 portable한 write blocker를 설치한 실패에서는 원본
   bytes/디렉터리가 손상되지 않고, card getter·runtime·화면 selected/original이 모두 이전 값 0으로
   복원되며 Timezone 화면에 남고 `timezone-save-failed`가 표시된다. Windows permission bit에는
   의존하지 않는다.
10. 저장 실패 toast의 영문·한글 문구가 정확하다.
11. `RecordingCanvas` draw에서 Shelf와 Timezone 모두 shelf 바탕 뒤 각자의 dim/panel이 그려지고,
    Timezone 화면에 parent Shelf panel이 중복되지 않는다. clear, GPU/GL 창, texture id 번호에는
    의존하지 않는다.
12. timezone preview/cancel/apply가 volume runtime과 volume 저장값을 바꾸지 않는다.

write 실패는 store 테스트가 이미 쓰는 invalid UTF-8 설정 파일 또는 `slot2.ini` 자리에 둔 디렉터리
방식을 사용한다. 테스트 전용 production hook, mock filesystem, unsafe, 새 dependency를 만들지 않는다.
기존 단언을 새 동작을 숨기도록 완화하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2\src\app.rs`
- `C:\SLOT2\crates\slot2\tests\timezone_menu_app.rs` (신규)
- `C:\SLOT2\assets\lang\en.ftl`
- `C:\SLOT2\assets\lang\ko.ftl`
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`의 신규 오류 key 직접 정의 검증만
- 새 Screen variant 때문에 직접 깨지는 기존 App test의 exhaustive match/setup만
- `C:\SLOT2\tasks\84-shelf-timezone-app-wiring.worker-result.md`

다른 production/test 파일은 수정하지 않는다. 특히 `slot2-store`, `slot2-platform`, `slot2-ui`,
host/device loop를 고치지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- 언어, 화면 기본값, 부팅 로고, 연동, 정보 행의 기능 구현 또는 활성화
- 도시/지역/DST database, 자동 감지, network time
- store 형식·범위·safe-write 구현 변경
- platform clock 구현·범위 변경
- boot 진단 로그 이동
- M4/M5 문서 checkbox 완료 처리
- 전체 workspace 테스트, 실제 GL 창, device 배포
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2 --test timezone_menu_app
cargo test -p slot2
cargo test -p slot2-i18n
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2 -p slot2-i18n --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 각 test 명령의 passed/failed/ignored 합계와 core-dependent skip 수를 보고한다.
검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test와 device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\84-shelf-timezone-app-wiring.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- Screen 진입/복귀와 tap/hold 분리 결과
- preview/cancel/unchanged apply/changed apply/save-failure rollback 결과
- 파일·runtime·menu 값 일치와 volume/unknown key 보존 결과
- shelf 바탕, 메뉴 layer, HUD preview draw 결과
- 오류 문구 영문·한글 직접 정의 결과
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored와 skip 수
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 계약이 틀려 보이는 부분과 남은 위험
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
