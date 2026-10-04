# Task 60 — 플랫폼별 CorePicker UI

현재 checkout에서 직접 작업한다. 인게임 메뉴의 Core 행에서 열릴 코어 선택 화면을 `slot2-ui`에
구현한다. 이번 태스크는 UI 모델·탐색·그리기만 다룬다. 설치 core 검색, App 화면 전환, 설정 저장,
Session 종료·재시작은 Task61로 분리한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\60-core-picker-ui.md`
- `C:\SLOT2\tasks\45-display-scale-menu-ui.result.md`
- `C:\SLOT2\tasks\59-core-scoped-state-routing.result.md`
- `C:\SLOT2\crates\slot2-ui\src\display_menu.rs`
- `C:\SLOT2\crates\slot2-ui\tests\display_menu.rs`
- `C:\SLOT2\crates\slot2-ui\src\in_game_menu.rs`의 Core 행과 공용 시각 상수만
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`의 module/export 부분만
- `C:\SLOT2\crates\slot2-ui\Cargo.toml`
- `C:\SLOT2\crates\slot2-retro\src\registry.rs`의 `CoreId`, `Platform`, `supported_cores`만
- `C:\SLOT2\assets\lang\en.ftl`, `ko.ftl`의 인게임/Display 메뉴 부분
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`의 메뉴 message 직접 정의 테스트만

직접 관련된 drawing/span helper만 추가로 읽는다. App, Session, 저장소, 다른 UI 화면, 로그,
저장소 이력은 읽지 않는다.

## 기존 계약

- registry의 공식 후보 순서가 단일 진실이다: GB/GBC `[mGBA, Gambatte]`, GBA
  `[mGBA, gpSP]`, 나머지 플랫폼은 기본 core 하나다.
- App은 다음 태스크에서 실제 core 디렉터리를 검사해 설치된 `CoreId` 목록을 UI에 넘긴다. UI는
  파일시스템을 읽지 않는다.
- Task59에서 state는 실제 core별로 격리됐다. 코어 변경은 hot-swap이 아니라 Session 재시작이다.
- 현재 Session이 외부 core이면 `core_id()`가 `None`이다. 이때 공식 후보 중 아무것도 현재값이라고
  거짓 표시하면 안 된다.
- 메뉴는 정지된 마지막 게임 frame 위 overlay이며 canvas를 clear하지 않는다.

## 구현 계약

### 1. 타입과 후보 계산

- 공개 `core_picker` module과 `CorePicker`를 추가하고 `slot2-ui`에서 re-export한다.
- UI crate가 별도 core enum이나 문자열 ID를 만들지 않고 `slot2_retro::{CoreId, Platform}`을 직접
  사용한다. 필요한 최소 dependency를 `slot2-ui/Cargo.toml`에 추가한다.
- 생성자는 최소한 `(platform, installed official cores, current: Option<CoreId>)` 의미를 받는다.
- 표시 후보는 `supported_cores(platform)` 순서를 유지한 채 설치 목록에 있는 core만 남긴다.
  입력의 순서·중복·지원하지 않는 core가 화면 순서나 후보를 바꾸면 안 된다.
- 현재 공식 core가 후보에 있으면 그 행을 최초 highlight한다. 현재값이 `None`이거나 설치 목록과
  일치하지 않으면 첫 후보를 highlight하되 **현재 표시를 붙이지 않는다**.
- 후보가 없으면 selection은 `None`이고 empty 상태다. 한 개뿐인 경우도 안전하게 동작한다.
- 공개 query는 highlighted `Option<CoreId>`, current `Option<CoreId>`, 후보 수/empty 여부처럼 App에
  필요한 읽기 전용 정보만 제공한다. 내부 후보 벡터를 mutable로 노출하지 않는다.

### 2. 탐색

- Up/Down은 후보 행 사이를 순환한다. 빈 목록에서는 no-op, 한 행에서는 그 행을 유지한다.
- 입력 이벤트, A/B 의미, sound, timer는 UI model에 넣지 않는다. Task61의 App이 호출한다.
- 현재 행과 highlighted 행은 서로 다른 개념이다. 탐색해도 current 표시는 이동하지 않는다.

### 3. 그리기

- DisplayMenu와 같은 인게임 overlay 언어를 사용한다: 전체 물리 패널 dim, 안전 영역 중앙 panel,
  제목, 후보 행, select/back hint. `Canvas::clear`를 호출하지 않는다.
- 정확히 하나의 highlighted 행만 배경 강조한다(후보가 있을 때). 현재 실행 core 행에는 highlight와
  독립적인 localized `Current` / `현재` 표시를 둔다.
- 공식 표시 이름은 정확히 `mGBA`, `Gambatte`, `gpSP`, `FCEUmm`, `Snes9x`,
  `Genesis Plus GX`다. 화면에 base filename(`*_libretro`)을 노출하지 않는다.
- 후보가 없으면 localized empty 문장을 panel 안에 표시하고 행 highlight는 없다.
- 하단에는 기존 select/back hints와 `Changing core restarts the game` /
  `코어를 바꾸면 게임을 다시 시작합니다` 안내를 표시한다. 아직 실제 재시작이 구현됐다고 쓰지
  않고, 선택 동작의 성격만 알린다.
- 제목은 `Core` / `코어`이며 기존 `ingame-core` key를 재사용할 수 있다.
- 세 기기 프로필의 640×480 safe area 안에 panel, title, rows, current badge, 안내, hints가 모두
  들어가야 한다. dim만 전체 물리 패널을 덮는다.
- 같은 상태를 warm draw한 뒤 반복해서 그리면 새 glyph/image texture upload가 없어야 한다.

### 4. localized message

최소한 다음 의미를 영문·한글 pack에 직접 정의한다.

- 여섯 공식 core 표시 이름
- `Current` / `현재`
- `No cores available` / `사용 가능한 코어가 없습니다`
- `Changing core restarts the game` / `코어를 바꾸면 게임을 다시 시작합니다`

고유명사는 두 언어에서 같은 표기를 사용한다. 한국어 fallback으로 누락을 숨기지 않는다.

## 테스트 계약

`core_picker.rs` UI 테스트를 새로 만들고 최소한 다음을 직접 검증한다.

- GB/GBC/GBA와 단일-core 플랫폼에서 후보가 정확히 registry 순서이며 설치되지 않은 core,
  중복, 다른 플랫폼 core가 제거됨
- current 공식 core를 정확히 최초 highlight하고 current 표시가 탐색과 무관하게 고정됨
- current `None` 또는 후보 밖 값은 첫 행을 highlight하지만 어느 행에도 current 표시가 없음
- empty·single·two-row의 Up/Down wrap과 query가 안전함
- 모든 여섯 `CoreId`가 정확한 localized 표시 이름으로 연결되고 base filename은 그려지지 않음
- 세 프로필×영문/한글에서 overlay가 clear하지 않고 dim을 먼저 그리며, panel·title·rows·badge·안내·
  hints가 safe area 안에 있음
- 후보가 있을 때 highlight 정확히 하나, empty일 때 0개와 empty 문장
- warm draw 뒤 같은 상태 반복 및 highlight 이동 시 새 `UploadAlpha8`/`UploadRgba8`가 없음
- 영문과 한글이 모든 새 key를 직접 정의함

`RecordingCanvas` operation과 geometry를 사용한다. texture id나 golden screenshot에 의존하지 않는다.
기존 DisplayMenu/InGameMenu 테스트를 약화하거나 다시 쓰지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-ui\Cargo.toml`
- `C:\SLOT2\crates\slot2-ui\src\core_picker.rs` (신규)
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`
- `C:\SLOT2\crates\slot2-ui\tests\core_picker.rs` (신규)
- `C:\SLOT2\assets\lang\en.ftl`
- `C:\SLOT2\assets\lang\ko.ftl`
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`
- `C:\SLOT2\Cargo.lock` (dependency graph가 실제로 바뀌는 경우만)
- `C:\SLOT2\tasks\60-core-picker-ui.worker-result.md`

그 밖의 파일은 수정하지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- App `Screen` variant, 인게임 Core 행 연결, installed core 파일 검색
- `GameSettings.core` 읽기·쓰기, setting rollback, toast
- Session stop/start, save flush, Resume load, audio sink 교체, core hot-swap
- core option·치트·state namespace·save RAM 정책 변경
- `slot2-retro` registry 수정 또는 후보 순서 재정의
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

끝나기 전에 `C:\SLOT2\tasks\60-core-picker-ui.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- 후보 filtering/order, highlighted/current/empty/navigation 계약
- 최종 layout·localized message·warm draw 결과
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
