# Task 27 — 시계 런타임 기반 최종 검토

2026-09-25. 현재 판정: 구현·검토 및 최종 자동 검증 완료. 실기 확인은 사용자 범위.

## 변경

- `slot2-platform::clock`: AtomicI32 기반 런타임 오프셋. 환경은 초기값만 제공하고 기본 UTC 0 유지. 유효 범위 밖 setter 입력은 기존 값을 보존하며 Err 반환.
- 초기화는 sentinel compare_exchange로, 먼저 설정한 값을 환경값이 덮어쓰지 않음.
- HUD: 단일 UTC 샘플로 유효성을 판정한 뒤 표시 오프셋 적용. 산술 overflow는 None으로 처리.
- 부팅 배너: `utc_offset_min` 및 `utc_now` 기록.
- 두 통합 테스트 파일에 3개 테스트 추가: 런타임 변경·잘못된 값 거부, UTC 판정 경계·overflow, 첫 조회 전 setter.
- UTC 저장·표시 분리 계약 D-25 등재, DESIGN 및 M5 설정 파일 배선 계획 갱신.

## 워커와 독립 검토

사용자 승인된 OpenCodex B.AI `glm-5.3-flash` 워커로 구현. 워커는 플랫폼 테스트 및 fmt만 확인했고 성공으로 보고했다.
오케스트레이터 검토에서 main.rs의 부팅 로그 포맷 인자 누락을 발견해 추가하고 키를 utc_now로 맞췄다.
HUD의 i64 상한 산술 보호와 첫 조회 전 setter 테스트도 보완했다.
워커 보고서의 성공은 최종 워크스페이스·기기 빌드 통과를 의미하지 않는다.

## 검증 기록

- 변경 전 시작한 workspace 실행: 369 passed / 0 failed, 결과 줄 42개 후 doctest에서 E0463(slot2_ui 찾지 못함), 종료 코드 1.
  실행 후반에 워커 수정·플랫폼 빌드가 겹쳐 깨끗한 기준선으로 확정할 수 없음. 최종 검증은 모든 워커 종료 후 순차 수행.
- 최종 fmt: 종료 코드 0.
- 최종 workspace test: 종료 코드 0, 결과 줄 54개, 372 passed / 0 failed.
- 최종 clippy: `cargo clippy --workspace --all-targets -- -D warnings`, 종료 코드 0.
- 최종 dist: Windows 실행 정책과 샌드박스의 Docker 설정 접근 문제를 확인한 뒤 프로세스 한정 ExecutionPolicy Bypass 및 승인된 Docker 접근으로 재실행. 종료 코드 0, 마지막 줄 `==> done`.

## 범위

M3 뒤 시계 선행 작업 묶음 완료 대상. M4 메뉴 구현과 M5 설정 파일 읽기·쓰기·화면 연결은 이번 범위 밖.
시스템 시각·파일 mtime을 수정하지 않음. 설정 UI가 없어 현재 기기 기본 시계는 여전히 UTC.
실기 검증은 수행하지 않음. 커밋·푸시 없음. 기존 Task 24~26 변경 보존.
