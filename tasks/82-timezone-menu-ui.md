# Task 82 — 시간대 선택 UI

현재 checkout에서 직접 작업한다. Task80/81의 분 단위 UTC 표시 오프셋을 사람이 조정할 수 있는 독립
`TimezoneMenu` UI 컴포넌트를 `slot2-ui`에 추가한다. 이번 태스크는 **model·draw·번역만** 다룬다.
선반에서 여는 경로, runtime preview, 저장·취소·실패 rollback은 Task83으로 분리한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\82-timezone-menu-ui.md`
- `C:\SLOT2\tasks\80-global-timezone-settings-store.result.md`
- `C:\SLOT2\tasks\81-timezone-startup-app-wiring.result.md`
- `C:\SLOT2\docs\DECISIONS.md`의 D-25만
- `C:\SLOT2\docs\MILESTONES.md`의 M4 선반 메뉴와 M5 시간대 항목만
- `C:\SLOT2\crates\slot2-ui\src\device_menu.rs`의 model, layout, draw/hint 방식만
- `C:\SLOT2\crates\slot2-ui\src\overlay_menu.rs`의 독립 submenu와 warm draw 방식만
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`의 module/re-export 부분만
- `C:\SLOT2\crates\slot2-ui\tests\device_menu.rs`의 geometry·localized draw helper만
- `C:\SLOT2\assets\lang\en.ftl`과 `ko.ftl`의 menu/hint 구역만
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`의 직접 정의 key 검사 부분만

App, Screen enum, input event 처리, store read/write 구현, platform clock, HUD, host/device loop, 다른
메뉴와 워커 로그·저장소 이력은 읽지 않는다.

## 현재 계약과 UX 결정

- 값은 지역 이름이나 DST 규칙이 아니라 **UTC에 더할 고정 분 수**다. 범위는 store가 소유한
  `UTC_OFFSET_MINUTES_MIN..=UTC_OFFSET_MINUTES_MAX`, 즉 `-720..=840`이다.
- 표시 형식은 항상 `UTC±HH:MM`이다. 0도 `UTC+00:00`으로 표시해 부호와 조정 결과가 모호하지 않게
  한다. 지역명, GMT 약어, 도시 목록과 DST 자동 전환은 만들지 않는다.
- Left/Right는 15분씩, Up/Down은 60분씩 조정한다. 15분은 현재 실제 고정 offset의 quarter-hour를
  표현하고, 60분은 한국처럼 큰 값까지 수십 번 누르지 않기 위한 빠른 조정이다.
- 양 끝에서는 clamp하고 반대 끝으로 wrap하지 않는다. `+14:00` 다음이 `-12:00`이 되는 26시간 점프는
  시간 설정으로 읽히지 않는다.
- 카드 파일은 정수 분 전체를 허용하므로 constructor가 `541` 같은 기존 값을 받으면 그대로 표시한다.
  조정도 그 값에서 ±15/±60으로 움직인다. UI가 기존 hand-edited 값을 몰래 quarter-hour에 맞추지 않는다.
- 컴포넌트는 열었을 때의 `original`과 현재 `selected`를 함께 보존한다. Task83은 selected를 runtime
  preview에 쓰고, B/저장 실패 때 original로 rollback한다. UI가 직접 clock이나 파일을 바꾸지 않는다.

## 구현 계약

### 1. TimezoneMenu model

- `crates/slot2-ui/src/timezone_menu.rs`를 추가하고 crate root에서 `TimezoneMenu`를 re-export한다.
- `TimezoneMenu`는 `Clone + Copy + Debug + PartialEq + Eq`이고 private `original: i32`,
  `selected: i32`만 가진다.
- `TimezoneMenu::new(minutes)`는 유효 범위 값을 그대로 original/selected에 둔다. 방어적으로 범위 밖
  호출은 가까운 경계로 clamp하되, production caller가 범위 밖 값을 만들었다고 가장하는 별도 error나
  App fallback을 만들지 않는다.
- 다음 의미의 공개 API를 제공한다.
  - `original() -> i32`
  - `selected() -> i32`
  - `changed() -> bool`
  - `left()` / `right()`: -15 / +15분, 경계 clamp
  - `up()` / `down()`: +60 / -60분. 화면상 Up은 동쪽/큰 값, Down은 서쪽/작은 값으로 일관되게 한다.
- 조정은 i32 overflow 없이 store 범위에서 끝나며 wrap하지 않는다. original은 navigation으로 바뀌지
  않는다.
- `format_offset(minutes) -> String` 의미의 공개 pure helper를 둔다. 입력은 같은 범위로 clamp해
  `+00:00`, `+09:00`, `-08:00`, `+05:45`, `-12:00`, `+14:00`처럼 sign과 두 자리 hour/minute를 만든다.
  `UTC` 글자는 helper에 넣지 않고 번역 key가 소유한다.

### 2. draw와 safe area

- 기존 menu palette, face cache, BTN span과 safe-area 방식을 재사용한다. 새 generic menu framework,
  animation, filesystem/backend 접근을 만들지 않는다.
- `draw`는 밑 화면 위에 올라갈 컴포넌트이므로 `Canvas::clear`를 호출하지 않는다.
- 순서는 physical panel 전체 dim(`BLACK alpha 0.6`) → safe area 중앙 panel → title → 큰 현재 값 → D-25
  설명 → 조정 hint → 적용/취소 hint다.
- layout 상수는 `BOX_W = 440`, `BOX_H = 244`, `PAD = 16`으로 둔다. 현재 값에는 `PX_TITLE`, 설명에는
  `PX_BODY`, hints에는 `PX_HINT`를 사용한다. 640×480 safe area에서도 모든 텍스트와 panel이 안에 있어야
  하고, 720×720에서는 panel safe-area 중심이 물리 panel 중심으로 잘못 이동하면 안 된다.
- current value는 `timezone-value`에 `offset = format_offset(selected)` argument를 넣어 그린다. code에서
  `UTC` 문자열을 이어 붙이지 않는다.
- `original != selected`를 색·별표·별도 문구로 표시하지 않는다. Task83이 A/B 의미를 처리하며 화면은
  현재 preview 값과 명시적 apply/cancel hints만 보여준다.
- 같은 language/value의 warm draw 8회와 navigation 후 이미 사용한 glyph 조합의 redraw에서 새
  `UploadAlpha8`/`UploadRgba8`가 없어야 한다. 새로운 숫자 glyph를 처음 그리는 navigation까지 upload 0을
  요구하지 않는다.

### 3. 영문·한글 문구

두 built-in pack에 다음 key를 직접 정의한다.

| key | English | 한국어 |
|---|---|---|
| `timezone-title` | `Time zone` | `시간대` |
| `timezone-value` | `UTC{ $offset }` | `UTC{ $offset }` |
| `timezone-note` | `Display only; the system clock stays on UTC` | `표시에만 적용하며 시스템 시각은 UTC로 유지합니다` |
| `timezone-hint-adjust` | `{ BTN("left") }{ BTN("right") } 15 min  { BTN("up") }{ BTN("down") } 1 hour` | `{ BTN("left") }{ BTN("right") } 15분  { BTN("up") }{ BTN("down") } 1시간` |
| `timezone-hint-apply` | `{ BTN("a") } apply` | `{ BTN("a") } 적용` |
| `timezone-hint-cancel` | `{ BTN("b") } cancel` | `{ BTN("b") } 취소` |

- 문자열 조각을 code에서 결합하지 않는다. `offset`만 완성된 numeric token으로 전달한다.
- Korean fallback이 누락을 숨기지 않게 여섯 key의 영문·한글 직접 정의를 i18n 테스트에 추가한다.
- 기존 번역 key와 문구를 변경하지 않는다.

## 테스트 계약

`crates/slot2-ui/tests/timezone_menu.rs`를 추가해 최소한 다음을 검증한다.

- constructor가 0, 양수·음수, 양 경계와 `541`을 exact round-trip하고 original/selected가 같아 시작함
- 범위 밖 constructor의 방어적 clamp
- Left/Right ±15, Up/Down ±60의 방향과 누적 결과, navigation 뒤 original 불변과 changed 상태
- 양 경계에서 반복 조정해도 clamp되고 wrap/overflow하지 않음
- `format_offset`의 0, ±1, quarter-hour, KST, 양 경계와 두 자리 padding
- draw가 clear 없이 dim → panel 순서이고 title/value/note/두 hint line을 그림
- `rgsp`, `rg35xxsp`, `rgcubexx`의 영문/한글에서 panel과 모든 text/button span이 safe area 안에 있음
- 세 geometry에서 value가 `timezone-value`의 localized span으로 그려지고 code 조합 문자열이 없음
- 같은 menu warm redraw 8회와 이미 그린 두 값 사이 navigation/redraw에서 glyph/image upload가 없음
- 영문/한글 pack이 여섯 신규 key를 fallback 없이 직접 정의함

RecordingCanvas만 사용한다. wall clock, environment, filesystem, GPU/GL window, screenshot golden과 texture
id 순서에 의존하지 않는다. 테스트를 위해 production에 cache counter나 input handler를 추가하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-ui\src\timezone_menu.rs` (신규)
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`의 module/re-export 두 줄
- `C:\SLOT2\crates\slot2-ui\tests\timezone_menu.rs` (신규)
- `C:\SLOT2\assets\lang\en.ftl`
- `C:\SLOT2\assets\lang\ko.ftl`
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`의 신규 key 직접 정의 테스트만
- `C:\SLOT2\tasks\82-timezone-menu-ui.worker-result.md`

다른 production/test 파일은 수정하지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- App Screen variant와 input wiring, ShelfMenu/PowerMenu row 추가
- runtime clock preview, 카드 read/write, confirm/cancel/save-failure rollback과 toast
- OS clock/timezone/RTC/mtime와 환경변수 변경
- store/platform offset 범위와 API 변경
- 지역·도시·DST database, 자동 timezone 감지와 network
- 다른 설정 UI, M5 checkbox와 boot 진단 로그 변경
- 전체 workspace 테스트, GL 창, device 배포
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-ui --test timezone_menu
cargo test -p slot2-ui -p slot2-i18n
cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 각 test 명령의 passed/failed/ignored 합계를 보고한다. 검증 뒤 코드를 바꾸면
영향받는 명령부터 다시 실행한다. workspace test, 실제 GL test와 device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\82-timezone-menu-ui.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- original/selected/changed, step 방향·크기·경계 clamp와 format 결과
- 영문/한글 문구, 세 geometry safe-area와 warm redraw 결과
- backend/store/clock/App을 건드리지 않았다는 확인
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄과 passed/failed/ignored 합계
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 후속 Shelf/App preview·apply/cancel·save-failure rollback이 남았다는 확인
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
