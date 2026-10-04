# Task 32 — M4 인게임 메뉴 순수 UI

## 목적

D-23의 인게임 메뉴를 `slot2-ui`의 독립 컴포넌트로 구현한다. 이번 태스크는 메뉴의 상태·탐색·그리기·en/ko 번역까지만 다룬다. 앱 화면 전환, 코어 정지, 오디오 뮤트, 실제 하위 기능 실행은 다음 태스크다.

## 현재 상태와 계약

- 작업 트리에는 Task 24~31 및 Task 27 시계 변경이 미커밋 상태로 있다. 지정 파일 밖의 기존 변경을 수정·정리·되돌리지 않는다.
- 관련 확정 계약: `docs/DECISIONS.md` D-23, `docs/DESIGN.md` 인게임 메뉴 문단, `docs/MILESTONES.md` M4.
- 기존 구현 참고: `crates/slot2-ui/src/power_menu.rs`. 화면 위에 dim과 패널을 그리며 `RecordingCanvas`로 검사할 수 있다.
- 메뉴 항목과 순서는 고정: 계속하기 / 세이브 스테이트 / 치트 / 화면 / 코어 / 기기 / 꺼내기.
- 640x480 안전 영역을 기준으로 하고 더 큰 패널에서는 중앙 정렬한다(D-09). 7개 항목과 제목·힌트가 잘리지 않아야 한다.

## 수정 허용 파일

- `crates/slot2-ui/src/in_game_menu.rs` 신규
- `crates/slot2-ui/src/lib.rs`
- `assets/lang/en.ftl`
- `assets/lang/ko.ftl`
- 필요한 경우 `crates/slot2-ui/tests/` 또는 `crates/slot2-i18n/tests/`의 Task32 전용 테스트 파일
- `tasks/32-ingame-menu-ui.worker-result.md`

이외 파일은 수정하지 않는다. 기존 파일에 이미 있는 사용자 변경은 보존한다.

## 구현 요구

1. `InGameChoice` enum과 7개 항목의 고정 배열을 둔다. 각 선택지는 안정적인 Fluent 키를 반환한다.
2. `InGameMenu`는 선택 인덱스를 소유하고 `Default`, 위/아래 순환, 현재 선택 조회를 제공한다. 외부 호출자가 잘못된 인덱스를 만들 수 없게 한다.
3. `draw`는 현재 게임 프레임 위에 그리는 오버레이다. canvas를 clear하지 않는다.
4. 전체 패널을 반투명 검정으로 dim하고, 안전 영역 중앙에 불투명 패널·제목·7개 행·선택 강조·선택/뒤로 힌트를 그린다. 640x480에서 모든 요소가 안전 영역 안에 있어야 한다.
5. 색상과 타이포그래피는 기존 `PowerMenu` 및 `splash` 팔레트를 재사용한다. 새 렌더링 추상화나 범용 메뉴 프레임워크를 만들지 않는다.
6. en/ko에 제목과 7개 항목의 자연스러운 번역 키를 추가한다. 기존 키를 재사용해 의미가 모호해지면 전용 키를 사용한다.
7. 공개 타입은 `slot2_ui::{InGameChoice, InGameMenu}`로 re-export한다.
8. 테스트는 최소한 항목 순서/키, 위·아래 wrap, 기본 선택, 640x480에서 선택 행 강조 및 모든 행의 y 경계가 안전 영역 안임을 검증한다. 구현 내부의 정확한 draw-op 개수에 과도하게 결합하지 않는다.

## 범위 밖

- `crates/slot2/src/app.rs` 및 `Screen` 변경
- `Session` 정지/재개, 오디오 뮤트, 입력 배선
- 세이브 스테이트·치트·화면·코어·기기 하위 화면 구현
- 꺼내기 동작
- 설정 ini 읽기·쓰기
- 스크린샷 골든 파일 생성
- 실기, adb, Samba, SD 카드 접근
- 공용 B.AI YAML, 사용자 Codex/OpenCodex 설정 변경
- 재귀 에이전트 위임, 커밋, 푸시

## 완료 기준

아래 명령을 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-ui -p slot2-i18n
cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings
```

세 명령 모두 종료 코드 0이어야 한다. 마지막 검증 뒤 코드를 수정했다면 영향받는 검증을 다시 실행한다. workspace 전체 테스트와 device 배포 빌드는 이번 작업자가 실행하지 않는다. 최종 판정 단계에서 필요성을 결정한다.

## 시도 및 보고

- 이번 태스크 누적 시도: 1/2. 첫 시도는 한국어 셸 출력 깨짐과 후속 provider connect timeout으로 코드 변경 없이 실패했다. 다음 실행은 두 번째이자 마지막 시도다.
- 첫 응답 최대 5분, 전체 최대 45분. 빈 출력은 실패 근거가 아니다.
- 결과는 `C:\SLOT2\tasks\32-ingame-menu-ui.worker-result.md`에 작성한다.
- 결과 형식: 성공/실패/부분 완료, 누적 시도, 변경 파일과 이유, 각 검증 명령·종료 코드·핵심 결과, 마지막 검증 뒤 수정 여부, 남은 항목/계약 의문, 실행 시간 및 확인 가능한 input/output/cached 토큰(모르면 미확인).
- 테스트를 만족시키기 위한 패딩·불필요한 draw·lint allow 금지. 계약이 잘못됐다고 판단하면 실패로 남기고 해당 지점을 보고한다.
