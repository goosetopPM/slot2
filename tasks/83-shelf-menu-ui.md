# Task 83 — 선반 설정 메뉴 UI

## 목적

선반 화면에서 여는 최상위 설정 메뉴의 UI와 선택 계약을 `slot2-ui`에 추가한다.

M4의 최종 메뉴 순서를 지금 확정한다. 아직 구현되지 않은 항목은 가용성 값으로 비활성화하고,
현재 사용할 수 있는 시간대 항목만 활성화할 수 있어야 한다. 이 태스크에서는 `App` 입력 연결이나
설정 저장을 하지 않는다. 다음 태스크가 이 메뉴에서 Task 82의 `TimezoneMenu`를 열고 미리보기,
저장, 취소 롤백을 연결한다.

## 먼저 읽을 파일

필요한 부분만 읽는다.

- `C:\SLOT2\AGENTS.md`
- `C:\SLOT2\docs\DESIGN.md`
- `C:\SLOT2\docs\DECISIONS.md`
- `C:\SLOT2\docs\MILESTONES.md`의 M4 부분
- `C:\SLOT2\tasks\82-timezone-menu-ui.md`
- `C:\SLOT2\tasks\82-timezone-menu-ui.result.md`
- `C:\SLOT2\crates\slot2-ui\src\device_menu.rs`
- `C:\SLOT2\crates\slot2-ui\src\power_menu.rs`
- `C:\SLOT2\crates\slot2-ui\src\timezone_menu.rs`
- `C:\SLOT2\crates\slot2-ui\src\lib.rs`
- `C:\SLOT2\crates\slot2-i18n\locales\en\main.ftl`
- `C:\SLOT2\crates\slot2-i18n\locales\ko\main.ftl`

## 구현 계약

### 1. 공개 타입

`slot2_ui`에 다음 공개 타입을 추가한다. 이름은 그대로 사용한다.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShelfChoice {
    Language,
    DisplayDefaults,
    BootLogo,
    Sync,
    TimeZone,
    About,
}

pub const SHELF_CHOICES: [ShelfChoice; 6] = [/* 아래 고정 순서 */];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShelfAvailability {
    pub language: bool,
    pub display_defaults: bool,
    pub boot_logo: bool,
    pub sync: bool,
    pub time_zone: bool,
    pub about: bool,
}
```

`ShelfAvailability`에는 최소한 다음 생성 편의 함수를 둔다.

```rust
pub const fn timezone_only() -> Self;
```

`ShelfMenu`는 `Clone + Copy + Debug + PartialEq + Eq`여야 하며 최소한 다음 API를 제공한다.

```rust
pub fn new(availability: ShelfAvailability) -> Self;
pub fn selected(&self) -> Option<ShelfChoice>;
pub fn is_available(&self, choice: ShelfChoice) -> bool;
pub fn up(&mut self);
pub fn down(&mut self);
pub fn draw(&self, canvas: &mut impl Canvas, i18n: &I18n);
```

필요한 내부 도우미와 상수는 자유롭게 추가한다. 새 타입은 `slot2_ui` 크레이트 루트에서 다시
내보낸다.

### 2. 고정 행 순서

메뉴 행은 위에서 아래로 정확히 다음 순서다.

1. `Language`
2. `DisplayDefaults`
3. `BootLogo`
4. `Sync`
5. `TimeZone`
6. `About`

이 순서는 공개 `SHELF_CHOICES` 상수 한 곳에 정의하고 드로잉과 이동이 같은 상수를 사용한다.
테스트가 순서와 중복 없음을 봉인해야 한다.

### 3. 선택과 이동

- 생성 시 첫 번째 사용 가능 행을 선택한다.
- 위/아래 이동은 사용 불가 행을 건너뛴다.
- 위/아래 모두 끝에서 반대편으로 순환한다.
- 사용 가능한 행이 하나뿐이면 위/아래 후에도 그 행을 유지한다.
- 사용 가능한 행이 하나도 없으면 `selected()`는 `None`이고 위/아래는 안전한 no-op이다.
- 비활성 행이 선택 상태가 되는 순간이 없어야 한다.
- `ShelfAvailability::timezone_only()`로 만든 메뉴의 선택은 `Some(ShelfChoice::TimeZone)`이다.

### 4. 드로잉

기존 `PowerMenu`, `DeviceMenu`, `TimezoneMenu`의 스타일을 따른다.

- 화면을 `clear`하지 않는다.
- 먼저 전체 화면에 반투명 검정 dim을 그리고, 그 위에 중앙 패널을 그린다.
- 패널과 모든 텍스트는 `canvas.safe()` 안에 있어야 한다.
- 제목은 `shelf-menu-title` 번역을 사용한다.
- 6개 행을 항상 고정 순서로 표시한다.
- 사용 가능한 행은 보통 텍스트 색, 사용 불가 행은 명확히 흐린 색으로 표시한다.
- 사용 불가 행 오른쪽에는 기존 `device-unavailable` 번역을 재사용해 사용할 수 없음을 표시한다.
- 사용 가능한 선택 행이 있으면 정확히 한 행만 강조한다.
- 선택 가능한 행이 하나 이상일 때만 `hint-select`를 표시한다.
- `hint-back`은 항상 표시한다.
- 버튼 힌트는 기존 메뉴와 같은 표기 관례를 사용한다.
- 영문과 한글 모두 640×480, 720×480, 1280×720에서 잘림 없이 안전영역 안에 있어야 한다.
- 같은 인스턴스를 다시 그렸을 때 결과가 같아야 한다.

### 5. 번역

영문과 한글에 다음 키를 직접 추가한다. 한 언어가 다른 언어 파일을 상속해 통과하면 안 된다.

| 키 | 영어 | 한국어 |
|---|---|---|
| `shelf-menu-title` | `Settings` | `설정` |
| `shelf-language` | `Language` | `언어` |
| `shelf-display-defaults` | `Display defaults` | `기본 화면 설정` |
| `shelf-boot-logo` | `Boot logo` | `부팅 로고` |
| `shelf-sync` | `Sync` | `연동 모드` |
| `shelf-about` | `About` | `정보` |

시간대 행은 Task 82의 `timezone-title`을 재사용한다. `device-unavailable`, `hint-select`,
`hint-back`도 기존 키를 재사용한다.

## 테스트 계약

`crates/slot2-ui/tests/shelf_menu.rs`를 추가하고 최소한 다음을 검증한다.

1. 행 순서가 계약과 정확히 같고 중복이 없다.
2. 생성 시 첫 사용 가능 행을 선택한다.
3. 위/아래가 비활성 행을 건너뛰고 양방향 순환한다.
4. 사용 가능 행 하나 및 0개인 경우가 안전하다.
5. `timezone_only()`가 시간대만 활성화하고 선택한다.
6. 세 해상도 × 두 언어에서 패널과 텍스트가 safe 영역 안에 있다.
7. 활성 선택 행 하나만 강조되고 비활성 행은 흐리며 `Unavailable`/`사용 불가`를 표시한다.
8. 사용 가능 행이 없을 때 선택 강조와 선택 힌트가 없고 뒤로 힌트는 있다.
9. draw가 clear하지 않고 dim 뒤에 패널을 그린다.
10. warm redraw 결과가 동일하다.

`slot2-i18n` 테스트에는 새 여섯 키가 en/ko 파일에 직접 존재하고 기대 문자열로 해석되는 검증을
추가한다.

테스트를 통과시키기 위한 패딩 draw나 입력별 상수는 금지한다. 레이아웃 테스트는 실제 사용자가
보는 요소의 경계를 검사해야 한다.

## 허용 변경 파일

- `crates/slot2-ui/src/shelf_menu.rs` (신규)
- `crates/slot2-ui/src/lib.rs`
- `crates/slot2-ui/tests/shelf_menu.rs` (신규)
- `crates/slot2-i18n/locales/en/main.ftl`
- `crates/slot2-i18n/locales/ko/main.ftl`
- 기존 `crates/slot2-i18n/tests/*.rs` 중 직접 번역 검증 파일
- `tasks/83-shelf-menu-ui.worker-result.md`

허용 목록 밖의 변경이 꼭 필요하면 먼저 보고서에 이유를 적고 최소 범위만 변경한다.

## 금지 범위

- `slot2-app`, `slot2-store`, `slot2-platform`, `slot2-bin` 변경
- `App::Screen` 변경 또는 실제 입력 연결
- 시간대 미리보기, 설정 저장, 저장 실패 롤백 구현
- 언어, 화면 기본값, 부팅 로고, 연동, 정보 화면의 실제 기능 구현
- 실기, SD 카드, Samba, `adb` 접근
- 공용 `~/.gjc-bai/agent/*.yml` 변경
- 커밋 또는 푸시

## 완료 검증

아래 명령을 원문 그대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-ui --test shelf_menu
cargo test -p slot2-ui -p slot2-i18n
cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings
```

전체 워크스페이스 테스트와 배포 이미지 생성은 이 태스크에서 실행하지 않는다.

## 워커 실행 규칙

- 누적 호출은 최대 2회다. 같은 태스크의 세 번째 호출은 금지한다.
- 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
- 로그가 비어 있어도 종료 신호가 아니다. 진행 확인은 `git status --short`로 한다.
- `CLAUDE.md`와 `AGENTS.md`의 오케스트레이터 역할 분리 조항만 무시하고 직접 구현한다.
- 나머지 안전, 품질, 실기 접근 금지 규칙은 유지한다.
- 커밋과 푸시는 금지한다.

## 결과 보고서

`C:\SLOT2\tasks\83-shelf-menu-ui.worker-result.md`에 다음을 작성한다.

- 누적 호출 횟수 `1/2` 또는 `2/2`
- 성공/실패
- 구현 요약
- 변경 파일 목록
- 각 검증 명령의 종료 코드와 마지막 결과 줄
- 테스트 수
- 계약이 잘못됐거나 모호해 보인 부분
- 남은 위험과 다음 태스크에 넘길 사항
