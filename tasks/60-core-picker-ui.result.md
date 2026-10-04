# Task 60 — Codex 최종 판정

## 판정

**통과. 누적 호출 1/2.** 추가 작업자 호출은 필요하지 않다.

## 확인한 내용

- `CorePicker`는 별도 core enum을 만들지 않고 `slot2_retro::{CoreId, Platform}`과
  `supported_cores(platform)`을 직접 사용한다.
- 후보는 registry 순서를 유지하며 설치 목록과 교집합을 취한다. 설치 입력의 순서·중복·타 플랫폼
  core가 후보나 순서를 바꾸지 않는다.
- 실행 중인 공식 core가 후보에 있으면 최초 highlight와 별도의 Current badge를 가진다. 탐색해도
  badge는 고정된다.
- 외부 core, 타 플랫폼 core, 설치되지 않은 current는 첫 후보만 highlight하고 어느 행에도 current
  표시를 붙이지 않는다.
- empty·single·two-row 탐색이 안전하며 Up/Down은 두 후보 사이에서 순환한다.
- 여섯 core의 화면 이름은 정확한 고유명사이고 `*_libretro` 파일 이름은 노출하지 않는다.
- overlay는 frame을 clear하지 않고 물리 패널 dim 뒤 safe area 중앙 panel에 title·행·badge·재시작
  안내·select/back hint를 그린다.
- 세 기기 프로필과 영문·한글에서 geometry와 message를 확인했고 warm draw 및 highlight 이동은 새
  texture upload를 만들지 않는다.

## 작업자 검증 증거

- `cargo fmt --all -- --check` — 종료 0
- `cargo test -p slot2-ui -p slot2-i18n` — 211 passed / 0 failed / 0 ignored
- 신규 `core_picker` 9 passed, i18n 26 passed
- `cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings` — 종료 0
- 최종 검증 뒤 코드 변경 없음

Codex는 사용자 운영 규칙에 따라 테스트를 다시 실행하지 않고 보고서와 구현·RecordingCanvas·i18n
테스트를 대조했다.

## 다음 방향

다음 태스크는 인게임 Core 행을 `CorePicker`에 연결한다. 실제 설치 core를 수집하고, 선택 변경 시
기존 Session의 save RAM과 Resume을 안전하게 기록한 뒤 `GameSettings.core`를 원자적으로 저장하고
새 core Session을 시작해야 한다. 설정 저장이나 새 Session 시작이 실패하면 기존 설정과 플레이
가능 상태를 잃지 않는 rollback 계약이 필요하다.
