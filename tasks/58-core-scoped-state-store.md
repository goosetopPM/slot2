# Task 58 — 코어별 스테이트 저장소 기반

현재 checkout에서 직접 작업한다. 게임별 코어 전환을 열기 전에 서로 호환되지 않는 libretro
serialize 데이터를 코어별 디렉터리로 격리할 저장소 API를 만든다. 이번 태스크는 `slot2-store`만
다룬다. Session/App은 아직 기존 평면 API를 사용하며, 후속 태스크에서 새 API로 전환한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\58-core-scoped-state-store.md`
- `C:\SLOT2\tasks\38-reversible-state-delete-store.result.md`
- `C:\SLOT2\tasks\57-alternative-core-foundation.result.md`
- `C:\SLOT2\crates\slot2-store\src\lib.rs`
- `C:\SLOT2\crates\slot2-store\src\card.rs`의 `StateKind`, `StateSlot`, `StateBackup`과 state API
- `C:\SLOT2\crates\slot2-store\src\atomic.rs`
- `C:\SLOT2\crates\slot2-store\tests\card.rs`의 state 테스트
- `C:\SLOT2\crates\slot2-store\tests\state_undo.rs`

직접 관련된 store 테스트만 추가로 읽는다. App, Session, UI, 워커 로그, 저장소 이력은 읽지 않는다.

## 기존 동작과 이유

- 현재 경로는 `States/<PLAT>/<stem>/resume.state`와 `<n>.state`이며 모든 코어가 같은 파일을 본다.
- Task57에서 GB/GBC는 mGBA·Gambatte, GBA는 mGBA·gpSP를 선택할 수 있게 됐다.
- libretro state는 코어 내부 직렬화 형식이므로 다른 코어에 넘길 수 없다. 같은 게임이라도 코어별
  Resume와 numbered state를 따로 보존해야 한다.
- 기존 카드에는 기본 코어가 만든 평면 state가 있을 수 있다. 새 버전이 이를 조용히 잃거나 임의의
  대체 코어에 귀속시키면 안 된다. 후속 App 태스크가 플랫폼 기본 코어 namespace로 한 번 이전한다.
- 현재 공개 평면 API와 테스트는 이번 태스크에서 제거하지 않는다. 새 API가 검증된 뒤 후속 태스크가
  제품 호출부를 옮긴다.

## 저장 경로 계약

새 scoped 경로는 다음과 같다.

```text
States/<PLAT>/<stem>/<core-base-name>/resume.state
States/<PLAT>/<stem>/<core-base-name>/<n>.state
```

`<core-base-name>`은 registry의 canonical base name이다. 예: `mgba_libretro`,
`gambatte_libretro`, `gpsp_libretro`. `slot2-store`는 `slot2-retro`에 의존하지 않으므로 코어 enum을
알지 않는다.

## 구현 계약

### 1. 검증된 namespace 값

공개 `StateNamespace`(이름은 동등하게 명확하면 조정 가능)를 추가하고 `lib.rs`에서 export한다.

- 생성 시 비어 있지 않은 ASCII 소문자·숫자·underscore만 허용한다.
- `/`, `\\`, `.`, `..`, 대문자, 공백, 비ASCII와 길이 65 이상을 `Error::Invalid`로 거부한다.
- 유효 길이는 1..=64 bytes다.
- 읽기 전용 문자열 accessor를 제공한다. 내부 문자열을 호출자가 바꿀 수 없게 한다.
- 이 타입이 검증되지 않은 경로 조각을 scoped API에 넘길 유일한 통로가 되게 한다.

### 2. scoped state API

현재 평면 API와 같은 의미를 namespace 안에서 제공한다. 이름은 Rust 관례에 맞게 다듬을 수 있지만
다음 기능이 모두 있어야 한다.

- scoped states directory와 state path
- scoped `list_states`, `next_state_number`, `read_state`, `write_state`, `delete_state`
- scoped `take_state`; 반환된 `StateBackup`은 namespace까지 origin으로 보존
- 기존 `restore_state(&StateBackup)`은 평면·scoped backup을 모두 원래 위치에만 복원

공통 내부 구현을 추출해 평면/scoped 경로가 같은 정렬, thumbnail, atomic-write, delete, undo
의미를 사용하게 한다. 공개 평면 API의 시그니처와 동작은 유지한다. scoped 목록에는 같은 게임의
다른 namespace나 평면 state가 섞이면 안 된다.

### 3. 기존 평면 state 이전 primitive

후속 App이 호출할 수 있는 `Card::adopt_legacy_states(cart, namespace)` 또는 동등한 좁은 API를
추가한다.

- 평면 디렉터리 바로 아래의 인식 가능한 `resume.state`, 양의 정수 `<n>.state`와 각 optional
  sibling PNG만 대상이다. namespace 하위 디렉터리는 건드리지 않는다.
- legacy state가 없으면 디렉터리를 만들지 않고 성공 no-op한다.
- 이전 전에 모든 목적지 state/PNG 충돌을 검사한다. 하나라도 존재하면 `Error::Invalid`로 전체를
  거부하고 원본·목적지를 하나도 바꾸지 않는다.
- orphan PNG, 알 수 없는 파일, 0·음수·overflow 번호, 다른 디렉터리는 그대로 둔다.
- 같은 카드 안의 rename으로 exact bytes와 mtime을 보존한다. 각 slot은 PNG를 먼저 옮기고
  `.state`를 마지막에 옮겨 state를 가시성 경계로 삼는다.
- 중간 rename이 실패하면 이번 호출에서 이미 옮긴 항목을 역순으로 원래 평면 경로에 되돌리도록
  best effort rollback한다. 실패를 성공으로 보고하지 않는다.
- 성공 후 평면 목록에서는 이전 항목이 사라지고 지정 namespace 목록에서 같은 kind·mtime·thumbnail
  존재 여부로 보인다.
- 어떤 namespace로 이전할지는 이 계층이 추측하지 않는다. 전달받은 namespace 하나에만 귀속한다.

### 4. 기존 불변식

- state와 PNG의 바이트 형식, PNG 인코딩, Resume의 삭제 금지(`take_state`), numbered 정렬과 다음
  번호 규칙을 바꾸지 않는다.
- `StateBackup`은 계속 opaque하며 `kind()` 외 state bytes를 노출하지 않는다.
- scoped backup을 다른 카드·카트·namespace 경로로 복원할 수 없어야 한다.
- 기존 평면 API의 모든 테스트가 그대로 통과해야 한다.

## 테스트 계약

새 `core_states.rs`를 우선 사용하고 최소한 다음을 직접 검증한다.

- namespace 허용·거부 경계와 path traversal 차단
- 같은 cart의 mGBA와 gpSP/Gambatte namespace에 같은 numbered id와 서로 다른 Resume를 써도
  목록·읽기·삭제·next number가 완전히 독립적임
- scoped thumbnail write/list/delete 및 평면 state와의 격리
- scoped take/restore가 exact state·raw PNG bytes를 보존하고, 다른 카드 복원과 목적지 충돌을
  기존처럼 거부하며 실패 후 재시도 가능함
- legacy Resume와 여러 numbered state(thumb 있음/없음)를 한 namespace로 이전하면 bytes, mtime,
  kind, 정렬, thumbnail 존재가 보존되고 평면 목록에서 사라짐
- 목적지 state 또는 PNG 하나의 충돌만 있어도 이전 전체가 시작되지 않음
- legacy state 없음은 디렉터리를 만들지 않는 no-op, orphan/unknown/0/overflow 파일은 불변
- 기존 `card`와 `state_undo` 테스트 전체 회귀

권한 오류나 실제 I/O 고장 주입용 제품 hook를 만들지 않는다. rollback 분기는 구조로 검토하고,
정상·충돌 경로를 결정적으로 테스트한다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-store\src\card.rs`
- `C:\SLOT2\crates\slot2-store\src\lib.rs`
- `C:\SLOT2\crates\slot2-store\tests\card.rs`
- `C:\SLOT2\crates\slot2-store\tests\state_undo.rs`
- `C:\SLOT2\crates\slot2-store\tests\core_states.rs` (신규 권장)
- `C:\SLOT2\tasks\58-core-scoped-state-store.worker-result.md`

그 밖의 파일은 수정하지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- Session/App/UI 호출부 전환, CorePicker, 코어 변경 저장, 재시작, 토스트
- 현재 선택 코어 추론 또는 플랫폼 기본 코어 결정
- legacy state 삭제, 복사본 유지, 자동 전체 카드 migration scan
- state format 변환, 다른 코어 state 호환 시도, save RAM 경로 변경
- 전체 workspace 테스트, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-store
cargo clippy -p slot2-store --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\58-core-scoped-state-store.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- 최종 namespace 검증과 scoped 경로/API
- legacy 이전의 대상·충돌·가시성·rollback 의미
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄
- 생성·수정 파일 목록과 최종 검증 뒤 코드 변경 여부
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.

## 시도 2/2 수정 계약

시도 1은 아직 최종 통과가 아니다. 아래 두 경계만 수정하고 같은 worker result를 누적 `2/2`로
갱신한다. API 이름, 경로 형식, 정상 이전, scoped CRUD/undo는 바꾸지 않는다.

1. `adopt_legacy_states`의 사전 충돌 검사는 **모든** 이전 대상 slot에 대해 목적지 `.state`와
   `.png`가 둘 다 비어 있는지 확인해야 한다. 원본 slot에 thumbnail이 없더라도 목적지에 orphan
   PNG가 있으면 전체 이전을 첫 rename 전에 `Error::Invalid`로 거부한다. 그렇지 않으면 이전된
   state가 다른 출처의 PNG와 결합된다.
2. 위 경우를 직접 재현하는 테스트를 추가한다. 평면 `1.state`에는 PNG를 만들지 않고 namespace의
   `1.png`만 미리 만든 뒤, 이전이 실패하고 평면의 모든 대상과 목적 orphan PNG가 byte-for-byte
   그대로이며 다른 slot도 하나도 이동하지 않았음을 확인한다. 기존 source-thumbnail + destination-
   PNG 충돌 테스트도 유지한다.
3. legacy 평면 디렉터리가 실제로 없을 때만 `Ok(0)` no-op한다. 경로가 존재하지만 디렉터리가
   아니거나 `read_dir`/entry 열람이 실패하면 성공으로 숨기지 말고 `Error::Io`를 반환한다. 제품용
   오류 주입 hook는 만들지 않는다. `states_dir(cart)` 위치에 일반 파일이 있는 결정적 테스트로
   `Ok(0)`이 아님을 확인한다.
4. 원래 완료 기준 세 명령을 마지막 변경 뒤 같은 순서로 다시 실행한다. 보고서에는 이번 수정 delta,
   두 회귀 테스트, 최종 결과, 검증 뒤 변경 없음만 간결하게 추가한다. 이번이 마지막 허용 호출이다.
