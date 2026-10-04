# Task 83 — Codex 최종 판정

## 판정

**통과 (누적 호출 1/2).** 두 번째 작업자 호출은 필요하지 않다.

## 확인 내용

- `ShelfChoice`와 공개 `SHELF_CHOICES`가 M4의 여섯 행을 확정 순서로 한 곳에 정의한다.
- `ShelfAvailability`와 `ShelfMenu`가 첫 사용 가능 행 선택, 비활성 행 건너뛰기, 양방향 순환,
  단일/0개 가용 행을 안전하게 처리한다.
- `timezone_only()`는 시간대만 활성화하고 `TimeZone`을 선택한다.
- draw는 기존 메뉴와 같은 `UiCtx` 계약을 사용하며 clear 없이 전체 dim 뒤 safe-area 중앙 패널을
  그린다. 비활성 행, 선택 강조, 선택/뒤로 힌트 조건이 분리돼 있다.
- 영문·한글 여섯 키가 실제 내장 pack인 `assets/lang/en.ftl`, `assets/lang/ko.ftl`에 직접 존재한다.
- UI 테스트가 행 순서, 이동, 비활성 표시, 0개 가용 상태, safe area, draw 순서와 warm redraw를
  검증한다. i18n 테스트가 두 언어의 정확한 문자열과 행 라벨 중복 없음을 검증한다.

## 작업자 검증 증거

- `cargo fmt --all -- --check`: 종료 0
- `cargo test -p slot2-ui --test shelf_menu`: **9 passed / 0 failed / 0 ignored**
- `cargo test -p slot2-ui -p slot2-i18n`: **270 passed / 0 failed / 0 ignored**
- `cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings`: 종료 0

Codex는 프로젝트 규칙에 따라 테스트를 다시 실행하지 않고 보고서와 코드·테스트 계약을 교차
검토했다.

## 승인한 명세 편차

1. 명세의 `draw(..., &I18n)`는 이 UI 크레이트에서 폰트 렌더링을 할 수 없으므로 기존 메뉴와 같은
   `draw(..., &mut UiCtx)`로 구현했다. 올바른 저장소 계약이다.
2. 명세에 적힌 `crates/slot2-i18n/locales/...`는 존재하지 않아 실제 내장 번역 경로인
   `assets/lang/*.ftl`을 변경했다. Task 82와 같은 구조다.
3. 등록된 실제 세 번째 기하가 1280×720이 아니므로 해당 요구는 합성 safe area/canvas로 검증했다.
   런타임 코드는 특정 합성 해상도에 의존하지 않는다.

## 남은 작업

- 선반의 Menu 입력에서 `ShelfMenu` 열기와 B 복귀
- 시간대 행에서 `TimezoneMenu` 진입
- 이동 중 runtime preview, A safe-write 적용, B 취소 롤백
- 저장 실패 시 원래 runtime 값 복원과 사용자 오류 표시

이 항목들은 Task 84 범위다. 실기 검증은 아직 하지 않았다.
