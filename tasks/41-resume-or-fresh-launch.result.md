# Task 41 최종 판정 — 통과

2026-09-26 Codex 재검토 결과, 시도 1에서 남은 결함과 직접 증거 공백이 시도 2/2에서 보완됐고
`tasks/41-resume-or-fresh-launch.md`의 계약을 충족했다.

## 보완 확인

- `resume_into`는 상태 바이트를 존재 확인용으로 먼저 읽지 않는다. 경로 metadata의 NotFound만
  조용한 누락으로 처리하고, 존재하거나 조회 오류가 난 경로의 실제 로드 실패는 로그 1줄과
  `resume-load-failed` 토스트로 알린다. `Card`와 `Session` 계약은 바뀌지 않았다.
- fresh 실행은 코어 세션이 성공한 뒤 Resume 상태와 PNG를 함께 삭제한다. 시작 전에는 둘 다
  남고, 코어 시작 실패 때도 기존 Resume이 보존된다.
- Hold(A)의 A 입력은 release까지 코어 입력에서 제외되고, release 뒤 새 A 입력은 정상 전달된다.
- 카트와 Resume/Fresh 의도는 제스처 수락 시점에 고정되어 선택 이동·rescan 뒤에도 바뀌지 않는다.
- Resume 힌트는 rescan 때 만든 카트별 불린 캐시만 사용한다. 상태 파일이나 카드 루트가 draw
  사이에 바뀌어도 화면은 다음 rescan까지 유지되고 draw가 파일시스템 결과에 의존하지 않는다.
- 새 실패·힌트 키는 영어와 한국어 팩에 직접 정의되어 있으며 정확한 출력 테스트가 추가됐다.

## 전체 계약 확인

- Tap(A)는 Resume이 있으면 안착 시점에 정상 Session을 시작한 뒤 Playing 노출·첫 코어 프레임
  전에 정확한 상태를 복원한다. Resume 파일은 읽은 뒤에도 남고 오디오 sink는 하나만 열린다.
- Resume 없음과 표시 후 사라짐은 조용히 fresh로, 손상·읽기 불가는 usable fresh와 오류 피드백으로
  이어진다. 실패한 Resume은 자동 삭제하지 않는다.
- Hold(A)는 같은 삽입 애니메이션과 기존 사운드·거부·실패 경로를 유지하며 새 게임을 시작한다.
- 빈 선반은 힌트를 그리지 않고, 일반 카트는 play, Resume 카트는 resume/new-game 힌트를 그린다.
  영어·한국어와 rgsp·rg35xxsp·rgcubexx 안전영역 검사가 통과했다.
- 번호 상태, 스위처·undo, quick save/load, 전원·볼륨·플랫폼·HUD 동작은 유지됐다.

## 검증 증거

시도 2의 마지막 코드 변경 뒤 작업자가 다음 명령을 순서대로 실행했고 이후 코드 변경이 없다고
기록했다.

- `cargo fmt --all -- --check`: 종료 0
- `cargo test -p slot2 -p slot2-ui -p slot2-i18n`: 종료 0, 실패 0; slot2 lib 25개,
  resume_app 7개, shelf_resume 3개, i18n 21개 및 기존 관련 테스트 통과
- `cargo clippy -p slot2 -p slot2-ui -p slot2-i18n --all-targets -- -D warnings`: 종료 0
- Codex 관련 파일 `git diff --check`: 종료 0

보고서와 변경이 일치하고 필요한 직접 회귀 테스트가 있으므로 동일 검증을 중복 실행하지 않았다.
커밋·푸시·실기·Pi·배포 빌드는 수행하지 않았다. 누적 호출은 허용 상한인 2/2이며 Task41에 추가
자동 시도는 하지 않는다.
