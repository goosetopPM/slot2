# Task 58 — Codex 최종 판정

## 판정

**통과. 누적 호출 2/2.** 추가 호출은 허용되지 않으며 필요하지 않다.

## 확인한 내용

- `StateNamespace`는 1..=64 byte의 소문자 ASCII·숫자·underscore만 허용해 검증되지 않은 경로
  조각이 scoped state 경로에 들어가지 못한다.
- 코어별 경로는 `States/<PLAT>/<stem>/<core-base-name>/`이며 목록·다음 번호·읽기·쓰기·삭제·
  take/restore가 다른 namespace와 평면 legacy state에서 분리된다.
- `StateBackup`은 namespace를 비공개 origin으로 보존하므로 삭제 취소가 원래 카드·카트·코어
  디렉터리로만 돌아간다. 기존 평면 API와 undo 동작은 유지된다.
- `adopt_legacy_states`는 canonical Resume와 양의 numbered state만 전달받은 namespace로 옮긴다.
  bytes·mtime을 보존하고 PNG를 먼저, state를 마지막에 rename하며 중간 실패는 역순 rollback한다.
- 이전 전 모든 대상 slot의 목적 state와 PNG를 검사한다. 원본에 PNG가 없어도 목적 orphan PNG가
  있으면 첫 rename 전에 전체를 거부하므로 서로 다른 출처의 state와 그림이 결합되지 않는다.
- legacy 경로가 없을 때만 빈 성공으로 처리한다. 경로가 디렉터리가 아니거나 디렉터리·entry 열람이
  실패하면 `Error::Io`를 반환한다.

## 2/2 수정 확인

시도 1에서 발견한 두 문제는 모두 수정됐다.

- 목적 orphan PNG 충돌: 평면 state와 목적 PNG가 그대로 남고 다른 slot도 이동하지 않는 직접
  회귀 테스트가 추가됐다.
- legacy 열람 실패 은닉: state 디렉터리 자리에 일반 파일을 둔 테스트가 `Error::Io`와 원본 불변을
  확인한다.

기존 source-thumbnail/목적 PNG 충돌, 정상 이전, namespace 독립, scoped undo 테스트도 유지된다.

## 작업자 검증 증거

- `cargo fmt --all -- --check` — 종료 0
- `cargo test -p slot2-store` — core_states 9 포함 전체 0 failed
- `cargo clippy -p slot2-store --all-targets -- -D warnings` — 종료 0
- 최종 검증 뒤 코드 변경 없음

Codex는 사용자 운영 규칙에 따라 테스트를 다시 실행하지 않고 누적 보고서와 구현·회귀 테스트를
대조했다.

## 다음 방향

다음 태스크는 Session과 App의 Resume·퀵세이브·스위처·삭제 취소 경로를 실제 선택 CoreId의
namespace로 전환한다. 카드의 기존 평면 state는 플랫폼 기본 코어 namespace로 한 번 이전하고,
코어 전환 UI는 이 배선이 끝난 뒤에 노출한다.
