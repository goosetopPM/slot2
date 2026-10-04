# Task 84 — Codex 최종 판정

## 판정

**통과 (누적 호출 1/2).** 두 번째 작업자 호출은 필요하지 않다.

## 확인 내용

- `Screen::Shelf(ShelfMenu)`와 `Screen::Timezone(ShelfMenu, TimezoneMenu)`가 추가됐고 `Screen`의
  `Copy` 계약을 유지한다.
- List의 Menu 탭만 Shelf 설정을 열며 기존 Menu 홀드의 PowerMenu 동작과 분리돼 있다.
- Shelf는 시간대 행만 선택·진입 가능하고 B/Menu로 List에 복귀한다.
- 시간대 방향 입력은 menu 이동 뒤 runtime clock에 즉시 preview를 적용하며 파일을 쓰지 않는다.
- B/Menu cancel은 original runtime을 복원하고 같은 TimeZone 행의 parent Shelf로 돌아간다.
- unchanged A는 write 없이 복귀하고, changed A 성공은 safe-write 뒤 preview runtime을 유지한다.
- write 실패는 original runtime과 `TimezoneMenu::new(original)`을 함께 복원하고 시간대 화면에 남아
  `timezone-save-failed`를 표시한다. `unwrap()`이나 App의 별도 timezone 복사본은 없다.
- Shelf와 Timezone은 List와 같은 wallpaper+shelf를 한 번 그린 뒤 각자의 modal만 그린다. parent
  Shelf panel 중복이 없고 HUD는 preview runtime을 읽는다.
- 영어·한국어 오류 문구가 각 built-in pack에 직접 정의돼 있다.

## 작업자 검증 증거

- `cargo fmt --all -- --check`: 종료 0
- `cargo test -p slot2 --test timezone_menu_app`: **1 passed / 0 failed / 0 ignored**
- `cargo test -p slot2`: **314 passed / 0 failed / 0 ignored**, core-dependent skip 0
- `cargo test -p slot2-i18n`: **33 passed / 0 failed / 0 ignored**
- `cargo check -p slot2 --no-default-features --features device`: 종료 0
- `cargo clippy -p slot2 -p slot2-i18n --all-targets -- -D warnings`: 종료 0

Task 81 기준 313개에 신규 순차 통합 테스트 1개가 추가된 314개로 테스트 수가 일치한다. Codex는
프로젝트 규칙에 따라 테스트를 재실행하지 않고 보고서와 production·test 코드를 교차 검토했다.

## 통합 테스트 범위

- Menu tap/hold 분리와 Shelf 열기·닫기
- 네 방향 preview와 카드 bytes 불변
- B/Menu cancel과 unchanged A의 무쓰기
- changed A 저장, volume·unknown key 보존
- invalid UTF-8 portable blocker에서 runtime/menu/card 기본값 복원, toast와 재시도 가능 상태
- List/Shelf/Timezone의 공통 shelf 바탕, modal 단일 layer와 HUD 유지
- process-global clock을 정확히 한 test에서 순차 사용하고 종료 시 시작값 복원

## 남은 사항

- 언어, 플랫폼 기본 화면, 부팅 로고, 연동, 정보 행은 아직 비활성이다.
- boot 진단 로그는 App 생성 전 offset을 표시할 수 있다.
- 실기에서 시간대 메뉴 가독성과 HUD clock preview는 사용자 확인이 필요하다.
