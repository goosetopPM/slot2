# Task 58 워커 결과 — 코어별 스테이트 저장소 기반

**성공. 누적 호출 2/2(마지막 허용 호출).** 완료 기준 세 명령 모두 종료 0이고, 최종 검증 뒤 코드
변경은 없다.

## 시도 2/2 delta

- **충돌 검사 강화**: `adopt_legacy_states`가 이제 **모든** 이전 대상 slot에 대해 목적지 `.state`와
  `.png`를 둘 다 검사한다 — 원본에 thumbnail이 없어도 namespace에 orphan PNG가 있으면 첫 rename
  전에 `Error::Invalid`로 전체를 거부한다. 그렇지 않으면 다른 출처의 그림이 도착한 state의
  thumbnail처럼 목록에 보이게 된다.
- **디렉터리 읽기 실패 은닉 제거**: `Ok(0)` no-op은 평면 경로가 실제로 존재하지 않을 때만이다.
  경로가 있는데 디렉터리가 아니면 `Error::Io`(NotADirectory), `read_dir`나 entry 열람이
  실패해도 `Error::Io`를 반환한다(이전에는 `flatten()`이 entry 오류를 삼켰다).
- **회귀 테스트 2개 추가**(`core_states` 7 → 9):
  `an_orphan_picture_in_the_namespace_stops_the_move` — 평면 `1.state`에는 PNG가 없고 namespace에
  `1.png`만 있는 경우 이전이 거부되고 평면 `resume.state`/`1.state`와 그 orphan PNG가 byte-for-byte
  그대로이며, PNG를 치우면 같은 이전이 두 slot 모두 성공하고 PNG는 생기지 않는다.
  `a_path_that_is_not_a_directory_is_an_error_not_a_no_op` — `States/GBA/arm` 자리에 일반 파일을
  두면 `Ok(0)`이 아니라 `Error::Io`이고 그 파일은 그대로다.
  기존 source-thumbnail + destination-PNG 충돌 테스트(`one_destination_in_the_way_stops_the_whole_move`)는
  유지된다.

## namespace 검증과 scoped API

- `StateNamespace`(신규, `lib.rs`에서 export)는 1~64 bytes의 ASCII 소문자·숫자·underscore만
  허용한다. 빈 문자열, 65 bytes, 대문자·혼합 대소문자, 공백, hyphen, dot, `/`, `\`, `.`, `..`,
  `../mgba`, `mgba/../gambatte`, 비ASCII(`코어`), 내부 NUL을 모두 `Error::Invalid`로 거부한다.
  검증된 문자열은 `as_str()`로만 읽을 수 있어 생성 뒤 바뀌지 않는다(검증되지 않은 조각이 scoped
  경로에 닿는 유일한 통로가 이 타입이다).
- scoped 경로는 계약대로 `States/<PLAT>/<stem>/<namespace>/resume.state`와 `<n>.state`다.
  API: `scoped_states_dir` · `scoped_state_path` · `scoped_list_states` · `scoped_next_state_number` ·
  `scoped_read_state` · `scoped_write_state` · `scoped_delete_state` · `scoped_take_state`.
- 평면/scoped는 같은 내부 구현(`state_file`·`states_in`·`next_number_in`·`write_state_files`·
  `delete_state_files`·`take_state_from`)을 공유해 정렬·thumbnail·atomic write·undo 의미가 하나다.
  기존 평면 API 시그니처와 동작은 불변이다(`StateBackup::kind()` 외에는 여전히 opaque).
- `StateBackup`이 namespace까지 origin으로 보존하고 `restore_state`는 그 디렉터리에만 복원한다.
  API가 `&backup`만 받으므로 다른 카드(루트 검사로 거부)·다른 카트·다른 namespace로 돌릴 통로가
  없다.

## legacy 이전 의미

`Card::adopt_legacy_states(cart, namespace) -> Result<usize, Error>`(신규).

- **대상**: 평면 디렉터리 바로 아래의 `resume.state`, 정규 양의 정수 `<n>.state`(선행 0·부호 없음)와
  각 sibling PNG. 그 밖은 그대로 둔다 — orphan PNG, 다른 확장자, `0.state`, u32 초과, `-1.state`,
  `01.state`, 디렉터리, 그리고 namespace 하위 디렉터리.
- **충돌**: 첫 rename 전에 모든 slot의 목적지 state와 PNG를 검사하고 하나라도 있으면
  `Error::Invalid`로 전체를 거부한다(테스트에서 state 충돌·PNG 충돌·orphan PNG 충돌 확인, 평면
  파일 전부 그대로).
- **가시성**: 같은 카드 안의 rename이라 bytes와 mtime이 보존되고, 각 slot은 PNG를 먼저 옮기고
  `.state`를 마지막에 옮긴다. 이전 뒤 평면 목록에서는 사라지고 namespace 목록에서 같은 kind·bytes·
  mtime·PNG bytes로 보이며 다음 번호도 이어받는다(2 → 3).
- **rollback**: 중간 rename 실패 시 이번 호출에서 옮긴 항목을 역순으로 되돌리는 best-effort 함수로
  복구하고 실패를 성공으로 보고하지 않는다(구조로만 보장; I/O 고장 주입 hook는 만들지 않았다).
- **no-op**: 평면 경로가 실제로 없거나 인식 가능한 파일이 없으면 `Ok(0)`이고 namespace 디렉터리를
  만들지 않는다. 경로가 있는데 읽을 수 없으면(디렉터리가 아니거나 entry 열람 실패) `Error::Io`다 —
  성공으로 숨기지 않는다. 어떤 namespace로 귀속할지는 추측하지 않고 넘겨받은 하나에만 옮긴다.

## 완료 기준 명령 (최종 상태, 명세 순서)

- `cargo fmt --all -- --check` → 종료 **0**.
- `cargo test -p slot2-store` → 종료 **0**: lib 1 / card 19 / cheats 15 / **core_states 9** /
  state_undo 11 / doc 0, 0 failed(마지막 결과 줄 `ok. 0 passed; 0 failed`).
- `cargo clippy -p slot2-store --all-targets -- -D warnings` → 종료 **0**(경고·오류 없음).
- 최종 검증 뒤 코드 변경 없음(두 파일 mtime이 검증 실행보다 앞선다).

## 생성·수정 파일

- 수정 `crates/slot2-store/src/card.rs`(namespace 타입, 공유 내부 helper, scoped API, 이전 primitive,
  모듈 doc의 레이아웃 한 줄), `crates/slot2-store/src/lib.rs`(`StateNamespace` export).
- 신규 `crates/slot2-store/tests/core_states.rs`(9 테스트: namespace 경계, 두 코어 독립, scoped
  thumbnail 격리, take/restore·충돌 재시도, legacy 이전, state/thumbnail 충돌 all-or-nothing,
  orphan PNG 충돌, 디렉터리 자리 파일 오류, no-op·비정규 파일 불변).
- `tests/card.rs`와 `tests/state_undo.rs`는 손대지 않았고 전부 통과한다.

## 계약이 틀려 보이는 부분 / 남은 위험

`01.state`·`+1.state` 같은 비정규 이름은 이전 대상이 아니라 평면에 남는데, 기존 `list_states`는
그 파일도 numbered 슬롯으로 보여주므로(경로는 정규 이름과 다름) 후속 App이 이전할 때 그런 파일은
스스로 판단해야 한다. 그 밖에는 namespace가 소문자만 허용하므로 registry base name이 모두
소문자라는 전제에 기대고 있다. 시도 2/2 소요 약 3분(전체 상한 45분 내).
