# Task 62 — 인게임 Device 메뉴 UI

현재 checkout에서 직접 작업한다. 인게임 메뉴의 Device 행에서 열릴 밝기·블루라이트·볼륨 화면을
`slot2-ui`에 구현한다. 이번 태스크는 **UI model·탐색·표시만** 다룬다. App 화면 전환, 실제 Volume 변경,
platform brightness/blue-light backend와 설정 저장은 Task63 이후로 분리한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\62-device-menu-ui.md`
- `C:\SLOT2\tasks\45-display-scale-menu-ui.result.md`
- `C:\SLOT2\tasks\60-core-picker-ui.result.md`
- `C:\SLOT2\crates\slot2-ui\src\display_menu.rs`
- `C:\SLOT2\crates\slot2-ui\src\core_picker.rs`의 overlay geometry/draw 방식만
- `C:\SLOT2\crates\slot2-ui\src\in_game_menu.rs`의 Device 행과 공용 시각 상수만
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`의 module/export 부분만
- `C:\SLOT2\crates\slot2-audio\src\volume.rs`의 0..100, mute, step 의미만
- `C:\SLOT2\assets\lang\en.ftl`, `ko.ftl`의 인게임 submenu/hint 부분
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`의 직접 정의 테스트만
- `C:\SLOT2\docs\DECISIONS.md`의 D-23만

직접 관련된 drawing/span helper만 추가로 읽는다. App, platform backend, store, 다른 화면, 워커 로그와
저장소 이력은 읽지 않는다.

## 현재 계약과 범위 판단

- D-23의 Device 항목은 밝기·블루라이트·볼륨을 다룬다.
- App의 `slot2_audio::Volume`은 0..100 level, mute와 5단계 step을 이미 제공한다.
- 현재 `slot2-platform`에는 brightness/blue-light API가 없다. RG SP는 `/sys/class/backlight`도 없고
  PWM 경로만 관찰돼 실제 제어 계약이 아직 확정되지 않았다. 이번 UI가 sysfs 경로나 지원 여부를
  추측하면 안 된다.
- 따라서 App이 다음 태스크에서 volume은 실제 값, brightness와 blue light는 backend capability에 따라
  `Some(0..=100)` 또는 `None`으로 넘긴다. `None`은 0이 아니라 **지원 여부 미확인/사용 불가**다.
- 이 설정은 게임별 설정이 아니다. UI는 파일시스템이나 hardware를 읽거나 저장하지 않는다.

## 구현 계약

### 1. 공개 타입과 상태

- 공개 `device_menu` module과 `DeviceMenu`, `DeviceSetting`을 추가하고 `slot2-ui`에서 re-export한다.
- `DeviceSetting`은 registry/string ID 없이 정확히 `Volume`, `Brightness`, `BlueLight` 세 항목을
  구분하는 작고 `Copy` 가능한 enum이다. 화면 순서는 이 순서로 고정한다.
- 생성자는 최소한 다음 의미의 snapshot을 받는다.
  - volume level `0..=100`과 muted 여부 — volume은 항상 available
  - brightness `Option<u8>`
  - blue light `Option<u8>`
- 들어온 값은 100을 넘지 않게 clamp한다. `None`을 0으로 바꾸지 않는다.
- 처음 선택은 Volume이다.
- App이 다음 태스크에서 사용할 query를 제공한다: selected setting, 각 값, volume muted, availability.
  내부 배열/상태를 mutable reference로 노출하지 않는다.
- 성공적으로 외부에 적용된 값을 화면에 반영할 좁은 setter를 제공한다. unavailable 항목을 setter가
  available로 몰래 바꾸면 안 되며, 모든 level은 0..100으로 clamp한다. mute는 Volume에만 속한다.

### 2. 탐색

- Up/Down은 **available 항목만** 순환한다. unavailable 행은 화면에는 보이지만 선택되지 않는다.
- Volume은 항상 available이므로 selection은 항상 유효하다.
- 예: brightness `None`, blue light `Some`이면 Volume ↔ BlueLight로 순환한다. 둘 다 `None`이면
  Up/Down을 눌러도 Volume에 머문다.
- Left/Right/A/B의 제품 의미, 실제 값 변경, sound, timer는 UI model에 넣지 않는다. Task63 App이
  selected/query/setter를 이용해 처리한다.

### 3. 그리기

- Display/Core submenu와 같은 인게임 overlay 언어를 사용한다: 전체 물리 패널 dim, 안전 영역 중앙
  panel, 제목, 세 행, 하단 hints. `Canvas::clear`를 호출하지 않는다.
- 세 행에는 localized label과 현재 상태를 함께 표시한다.
  - available + unmuted: 0..100 bar와 localized percentage
  - Volume muted: 기억된 level의 bar는 유지하고 값 text는 localized `Muted` / `음소거`
  - unavailable: bar를 활성 상태처럼 그리지 않고 localized `Unavailable` / `사용 불가`를 dim color로
    표시한다.
- selected available 행은 정확히 하나만 배경 강조한다. unavailable 행은 절대 highlight하지 않는다.
- level bar의 채움 폭은 0에서 0, 100에서 track 전체이며 범위를 벗어나지 않는다. muted Volume bar는
  기억된 level을 보여주되 dim 처리해 현재 출력이 0임을 값 text와 함께 정직하게 나타낸다.
- 제목은 `Device` / `기기`, 행은 `Volume` / `볼륨`, `Brightness` / `밝기`,
  `Blue light` / `블루라이트`다.
- 하단에는 localized Left/Right adjust와 B back hint를 둔다. Volume 행이 선택됐을 때만 A mute/unmute
  hint도 표시한다. unavailable 항목은 선택되지 않으므로 거짓 조작 hint가 나오지 않는다.
- 세 기기 프로필의 640×480 safe area 안에 panel, title, rows, bars, value text와 hints가 모두 들어가야
  한다. dim만 전체 물리 패널을 덮는다.
- 같은 상태를 warm draw한 뒤 반복하거나 selection/value/mute가 바뀌어도 새 glyph/image texture
  upload가 없어야 한다.

### 4. localized message

최소한 다음 의미를 영문·한글 pack에 직접 정의한다.

- Device / 기기
- Volume / 볼륨
- Brightness / 밝기
- Blue light / 블루라이트
- `{ $value }%`에 해당하는 percentage 표시
- Muted / 음소거
- Unavailable / 사용 불가
- Left/Right adjust hint / 좌우 조절 hint
- A mute/unmute hint / A 음소거 hint

percentage는 코드에서 완성 문장으로 조립하지 않고 Fluent argument로 넘긴다. 기존 `ingame-device`와
`hint-back`은 의미가 맞으면 재사용한다. 한국어 fallback으로 누락을 숨기지 않는다.

## 테스트 계약

`device_menu.rs` UI 테스트를 새로 만들고 최소한 다음을 직접 검증한다.

- 생성 시 값 clamp, `None` 보존, Volume 최초 선택과 세 항목 순서
- 세 항목 모두 available일 때 Up/Down wrap
- 일부 unavailable일 때 그 행을 건너뛰고 available 항목만 순환
- brightness/blue-light가 모두 unavailable이면 Volume에 고정
- setter가 available 값만 갱신하고 clamp하며 unavailable을 available로 바꾸지 않음; mute는 Volume만
- 0/중간/100 level의 bar geometry가 정확하고 track 범위를 벗어나지 않음
- muted Volume은 기억된 bar를 dim으로 유지하고 Muted text를 표시함
- unavailable 행은 Unavailable text, 비활성 시각, highlight 0이며 available selected highlight는 하나
- Volume 선택 때만 mute hint가 있고 다른 available 행에는 adjust/back만 있음
- 세 프로필×영문/한글에서 overlay가 clear하지 않고 dim을 먼저 그리며 모든 요소가 safe area 안에 있음
- warm draw 뒤 반복, selection 이동, level·mute setter 적용 시 새 `UploadAlpha8`/`UploadRgba8`가 없음
- 영문과 한글이 모든 새 key를 직접 정의하고 percentage argument를 실제로 포맷함

`RecordingCanvas` operation과 geometry를 사용한다. texture id나 golden screenshot에 의존하지 않는다.
기존 DisplayMenu/CorePicker/InGameMenu 테스트를 약화하거나 다시 쓰지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-ui\src\device_menu.rs` (신규)
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`
- `C:\SLOT2\crates\slot2-ui\tests\device_menu.rs` (신규)
- `C:\SLOT2\assets\lang\en.ftl`
- `C:\SLOT2\assets\lang\ko.ftl`
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`
- `C:\SLOT2\tasks\62-device-menu-ui.worker-result.md`

그 밖의 파일은 수정하지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- App `Screen` variant와 인게임 Device 행 연결
- `slot2_audio::Volume` 변경, 실제 volume/mute 적용
- brightness/blue-light platform API, sysfs/PWM 경로 추측 또는 hardware write
- 전역/platform/game settings 파일 추가나 저장
- 전체 workspace 테스트, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-ui -p slot2-i18n
cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test나
device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\62-device-menu-ui.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- 타입, availability, clamp, selection/navigation 계약
- bar/muted/unavailable/hint와 세 geometry layout 결과
- 영·한 message와 warm draw 결과
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
