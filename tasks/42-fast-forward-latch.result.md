# Task 42 최종 판정 — 통과

2026-09-26 Codex 검토 결과, 누적 호출 2/2 보고서와 실제 변경이 일치하고
`tasks/42-fast-forward-latch.md`의 계약을 충족했다. 시도 1에서 구현·검증을 완료했고 시도 2는
코드 변경 없이 현재 트리를 재확인하고 지정 검증을 다시 실행했다.

## 동작 확인

- App의 private `ff_latch`는 `Screen`과 분리되어 있고 기본값은 false다.
- R2 물리 hold는 즉시 `FAST_FORWARD` 4배속을 적용하고 release 뒤 정상 속도 1로 돌아간다.
- 실행 중 R2 더블탭은 latch를 켜 release 뒤에도 4배속을 유지하며, 다음 더블탭과 release로
  정상 속도로 돌아간다.
- 선반·삽입·배출·전원·인게임 메뉴·스위처 및 Session 없는 Playing의 더블탭은 latch를 켜지 않는다.
- L2가 눌리면 latch나 R2 hold보다 우선해 코어를 전진시키지 않고 rewind만 수행한다. release 뒤
  기존 latch 4배속이 재개된다.
- 같은 Session의 인게임 메뉴·스위처 왕복은 latch를 유지하며 메뉴가 열린 동안 프레임은 멈춘다.
- `stop_session`이 latch를 공통 초기화하므로 MENU hold, 메뉴 Eject, 물리 Power, 전원 메뉴의
  Restart/PowerOff 모두 해제되고 다시 실행한 Session은 속도 1로 시작한다.
- `slot2-input`, Session, UI, 오디오, Resume 및 번호 상태 계약은 수정하지 않았다.

## 테스트와 검증

- 코어 없는 단위 테스트 3개가 게임 밖 무장 방지, 메뉴·스위처 유지, 모든 종료 경로 초기화를
  직접 확인한다.
- 실제 mGBA 세션 테스트 3개가 순간 hold, latch on/off, 실제 추가 프레임, L2 우선, 메뉴 pause,
  배출 후 재실행 속도 초기화를 확인했다. 이번 환경에서는 skip되지 않고 실행됐다.
- 기존 `slot2-input` 테스트가 R2만 더블탭 대상으로 두고 MENU 탭을 지연하지 않으며 SELECT 코드를
  유지하는 계약을 계속 확인한다.

작업자가 시도 2에서 마지막 코드 변경이 없는 상태로 다음을 순서대로 실행했다.

- `cargo fmt --all -- --check`: 종료 0
- `cargo test -p slot2 -p slot2-input`: 종료 0, 실패 0; slot2 lib 28개,
  time_controls_app 3개, slot2-input 12개 포함
- `cargo clippy -p slot2 -p slot2-input --all-targets -- -D warnings`: 종료 0
- Codex 관련 파일 `git diff --check`: 종료 0

증거가 충분하므로 Codex에서 동일 테스트를 중복 실행하지 않았다. 커밋·푸시·실기·Pi·배포 빌드는
수행하지 않았다. 호출 상한 2/2를 사용했으므로 Task42 추가 자동 시도는 하지 않는다.
