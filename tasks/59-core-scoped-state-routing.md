# Task 59 — Session/App 코어별 스테이트 배선

현재 checkout에서 직접 작업한다. Task58의 코어별 state API를 실제 제품 경로에 연결한다. Session이
실제로 연 코어의 namespace에만 Resume와 numbered state를 읽고 쓰며, App의 선반 Resume 캐시·
fresh launch·퀵세이브/로드·스위처·삭제/undo도 같은 namespace만 보게 한다. 기존 평면 state는
플랫폼 기본 코어 namespace로 한 번 안전하게 이전한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\59-core-scoped-state-routing.md`
- `C:\SLOT2\tasks\57-alternative-core-foundation.result.md`
- `C:\SLOT2\tasks\58-core-scoped-state-store.result.md`
- `C:\SLOT2\crates\slot2\src\session.rs`의 core 선택, state save/load/stop, accessor 부분
- `C:\SLOT2\crates\slot2\src\app.rs`의 rescan, launch/Resume, state switcher, quick state,
  delete/undo 부분
- `C:\SLOT2\crates\slot2\tests\session.rs`의 state/core 선택 테스트
- `C:\SLOT2\crates\slot2\tests\resume_app.rs`
- `C:\SLOT2\crates\slot2\tests\quick_state_app.rs`
- `C:\SLOT2\crates\slot2\tests\state_switcher_app.rs`
- `C:\SLOT2\crates\slot2-store\src\card.rs`의 `StateNamespace`와 scoped/legacy API만
- `C:\SLOT2\crates\slot2-retro\src\registry.rs`의 `CoreId`와 플랫폼 기본 코어만

직접 관련된 helper만 추가로 읽는다. UI 구현, 다른 crate, 전체 문서·로그·저장소 이력은 읽지 않는다.

## 기존 동작과 문제

- Session의 core 선택은 설정된 파일 존재 여부, 공식 core의 플랫폼 지원 여부, 기본 fallback을
  처리한다. 이 최종 dylib가 실제 state 형식의 주인이다.
- App은 현재 평면 `state_path/list_states/next_state_number/take_state/delete_state`를 직접 호출한다.
- Session도 평면 `read_state/write_state`를 사용한다. Task58의 scoped API는 아직 제품에서 쓰이지
  않는다.
- 선반 Resume 힌트는 rescan 때만 캐시되며 draw에서 파일시스템을 보지 않는다. 이 성질을 유지한다.
- 코어 선택 UI는 아직 없다. 이번 태스크는 저장·복원 배선만 완성한다.

## 구현 계약

### 1. 코어 선택을 한 번만 해석한다

`session.rs`에 core 파일을 **로드하지 않고** 최종 선택을 계산하는 작은 resolver를 추출한다.
이름과 타입은 코드에 맞게 조정할 수 있다.

- `Session::start`와 App rescan이 같은 resolver를 사용한다. 설정 해석을 두 군데 복제하지 않는다.
- 결과에는 최종 dylib 경로, `Option<CoreId>`, 유효한 `StateNamespace`가 있어야 한다.
- 기존 선택 의미를 유지한다: 없는 이름은 기본 core, 지원하지 않는 공식 core도 기본 core, 존재하는
  알 수 없는 외부 core는 그대로 연다.
- rescan의 조용한 조회는 fallback 로그를 반복하지 않는다. 실제 launch는 기존처럼 잘못된 설정과
  fallback을 한 번 로그한다.
- 설정의 core 값은 core 디렉터리 안의 **파일 이름**으로만 취급한다. `/`, `\\`, `..` 등 경로
  성분이 들어간 값은 외부 경로를 열지 않고 기본 core로 fallback한다.
- 공식 core namespace는 `CoreId::base_name()`과 정확히 같다.
- 알 수 없는 기존 core도 state 기능을 잃지 않는다. 실제 library stem이 `StateNamespace` 규칙에
  맞으면 소문자 canonical stem을 사용한다. 맞지 않으면 고정된 결정적 변환을 사용해 64 bytes
  이하의 `external_<lower-hex>` namespace를 만든다. 같은 stem은 실행·호스트가 바뀌어도 항상 같은
  namespace, 서로 다른 대표 입력은 다른 namespace가 되게 하고 알고리즘을 테스트로 고정한다.
  임의 process seed의 `Hash`/`DefaultHasher`는 영속 경로에 사용하지 않는다.
- 플랫폼 변환과 기본 core namespace 계산도 같은 helper 계층에 둬 App이 registry 규칙을 복제하지
  않게 한다.

### 2. Session은 실제 core namespace를 소유한다

- `Session`에 최종 `StateNamespace`를 저장하고 읽기 전용 accessor를 제공한다.
- `save_state`, `load_state`, `stop`의 Resume는 모두 scoped API를 사용한다.
- save RAM 경로는 코어 간 공유하는 현재 계약을 유지한다. 이 태스크는 serialize state만 격리한다.
- state load 뒤 resampler/rewind 초기화, thumbnail, 실패 의미는 유지한다.
- `core_id() == None`인 외부 core도 resolver가 정한 namespace에서 정상 save/load한다.

### 3. legacy 평면 state는 기본 core로 한 번 이전한다

App의 기존 rescan 경계에서 각 cart에 대해 다음 순서를 수행한다.

1. 플랫폼 **기본** core namespace를 계산한다.
2. `adopt_legacy_states`로 평면 state를 그 namespace에 이전한다.
3. 현재 설정과 실제 core 파일을 resolver로 해석해 선택된 namespace의 Resume 존재 여부를 캐시한다.

- 설정이 대체 core를 가리켜도 legacy state는 항상 플랫폼 기본 core 소유다. 과거 버전은 대체 core를
  선택할 수 없었기 때문이다.
- 이전 성공/빈 no-op은 조용하다. 충돌·I/O 실패는 한 줄 로그를 남기고 평면 원본을 그대로 둔다.
  scoped 목적지를 덮거나 legacy를 삭제하지 않는다.
- 이전 실패 뒤 힌트와 제품 state 동작은 선택된 scoped namespace만 사용한다. 평면과 scoped를
  합치거나 어느 쪽이 최신인지 추측하지 않는다.
- rescan 뒤 draw는 계속 파일시스템을 읽지 않는다.

### 4. App의 모든 state 동작을 현재 Session namespace로 전환한다

다음 경로에서 평면 API 호출을 제거하고 현재 Session의 namespace를 사용한다.

- Tap(A) Resume 존재 확인과 load
- Hold(A) fresh launch의 Resume 삭제
- Session 종료 Resume 쓰기
- state switcher 열기/새로고침
- numbered state load
- X 삭제와 실패 후 목록 새로고침
- Y undo 뒤 목록 새로고침 (`StateBackup` restore 자체는 namespace를 이미 소유)
- quick save의 다음 번호와 write
- quick load의 최신 numbered 검색과 load

Fresh는 현재 선택 core의 Resume만 삭제한다. 다른 core의 Resume/numbered state와 평면 legacy는
건드리지 않는다. pending undo는 현재와 같이 Session 종료 시 폐기하며, 다른 namespace 목록에
섞이지 않는다.

제품 코드인 `crates/slot2/src`에는 migration 호출을 제외한 평면 state API 호출이 남지 않게 한다.

### 5. 오류와 수명주기

- resolver 실패로 기본 core 파일도 없으면 기존 `NoCore` launch 오류와 toast를 유지한다.
- Resume가 rescan 후 사라진 race는 조용히 fresh로 시작한다. 존재하지만 읽기/역직렬화가 실패하면
  기존 `resume-load-failed` toast를 표시하고 그 scoped 파일을 보존한다.
- launch 실패 전에는 어느 namespace의 Resume도 삭제하지 않는다.
- core를 바꾼 뒤 이전 core state가 새 core에 전달되는 fallback을 만들지 않는다.

## 테스트 계약

기존 테스트를 scoped 경로에 맞게 고치되 의미나 단언을 약화하지 않는다. 최소한 다음을 직접
검증한다.

- mGBA Session과 gpSP Session이 같은 GBA cart의 같은 numbered id·Resume에 서로 다른 bytes를
  저장하고 서로의 state를 보거나 load하지 않음; 각 `state_namespace()`가 실제 core와 일치
- 설정한 gpSP가 없거나 Gambatte/GBA처럼 unsupported 공식 조합이면 실제 기본 mGBA namespace를 사용
- 존재하는 알 수 없는 core의 namespace 생성은 안전하고 결정적이며, 경로 성분이 있는 설정은
  core_dir 밖을 선택하지 않음
- 평면 legacy Resume/numbered state가 rescan에서 기본 core namespace로 옮겨지고, 대체 core가 현재
  선택돼 있으면 그 Resume 힌트나 numbered 목록에 나타나지 않음
- legacy 이전 충돌은 원본과 scoped 목적지를 보존하며 App이 둘을 합치지 않음
- 대체 core의 Resume가 있을 때만 그 core로 Tap(A) Resume되고, Hold(A) fresh는 대체 core Resume만
  지워 기본 core Resume는 보존
- quick save/load와 switcher 목록·load·delete·undo가 현재 core namespace만 다루고 다른 core의
  같은 번호와 thumbnail을 보존
- core 변경 후 잘못된 state bytes를 새 core에 넘기지 않으며, 각 core로 돌아가면 자기 Resume와
  numbered state가 다시 보임
- 기존 Resume race/error toast, A-hold suppression, state load rewind reset, undo expiry와 draw의
  no-filesystem-access 회귀

실코어가 필요한 테스트는 기존 명시적 host core skip 패턴을 유지한다. gpSP DLL은 Task57에서
생성됐으므로 이 checkout에서는 mGBA/gpSP 격리 테스트가 skip 없이 돌아야 한다. 제품용 test hook를
추가하지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2\src\session.rs`
- `C:\SLOT2\crates\slot2\src\app.rs`
- `C:\SLOT2\crates\slot2\tests\session.rs`
- `C:\SLOT2\crates\slot2\tests\resume_app.rs`
- `C:\SLOT2\crates\slot2\tests\quick_state_app.rs`
- `C:\SLOT2\crates\slot2\tests\state_switcher_app.rs`
- `C:\SLOT2\crates\slot2\tests\core_state_routing.rs` (신규 권장; private resolver 단위 테스트는
  `session.rs`의 `#[cfg(test)]`에 둘 수 있음)
- `C:\SLOT2\docs\DESIGN.md`의 state 경로·core 선택 관련 문단만
- `C:\SLOT2\tasks\59-core-scoped-state-routing.worker-result.md`

그 밖의 파일은 수정하지 않는다. `slot2-store`와 `slot2-retro` API는 Task57/58 결과를 그대로
사용한다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- CorePicker UI, 인게임 Core 행, 설정 저장, core 변경 확인/재시작 흐름
- 실행 중 hot-swap, state format 변환·삭제·병합, core 간 state 호환 시도
- save RAM 코어별 분리, 치트·display·rewind 정책 변경
- Task58 API 수정 또는 legacy 파일을 대체 core로 이전
- 전체 workspace 테스트, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2
cargo clippy -p slot2 --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test나
device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\59-core-scoped-state-routing.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- resolver와 공식/외부 core namespace 규칙
- legacy 이전 시점과 실패 처리
- Session/App의 scoped 전환 지점과 다른 core 보존 증거
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄과 실코어 skip 여부
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.

## 시도 2/2 복구 계약

시도 1은 코드 작업 도중 provider가 400 `read body failed`로 대화형 요청을 중단했고 결과 보고서를
남기지 못했다. raw request에는 `developer` role이 없었고 메시지 1,038개가 누적돼 있었다. 공유 설정이나
모델을 바꾸지 말고 **새 대화형 세션**에서 현재 working tree를 이어서 처리한다. 이번이 마지막 허용
호출이다.

1. 현재 수정된 `session.rs`, `app.rs`, 테스트와 DESIGN을 보존한다. 처음부터 재구현하거나 revert,
   clean, reset하지 않는다.
2. 첫 시도의 마지막 실패 원인은 `quick_state_app.rs`의 `numbered(card, cart)` helper가 아직 평면
   `card.list_states(cart)`를 읽는 것이다. 이 helper를 해당 테스트들이 실행 중인 실제/default mGBA
   namespace의 `scoped_list_states`로 고쳐, quick save 뒤 `[1]`, 기존 slot 뒤 다음 번호, 실패 뒤 목록
   단언이 제품 경로와 같은 디렉터리를 보게 한다.
3. 아래 method 이름으로 남은 평면 호출을 다시 검색한다.
   `list_states`, `next_state_number`, `state_path`, `take_state`, `delete_state`, `read_state`,
   `write_state`. 제품 실행 경로에는 `adopt_legacy_states`의 입력 외 평면 state 접근이 없어야 한다.
   `app.rs`의 `#[cfg(test)]` 안에는 legacy migration·draw cache·평면 `StateBackup` 자체를 의도적으로
   검증하는 fixture가 있으므로 기계적으로 전부 치환하지 말고, 제품의 scoped 동작을 단언하는
   fixture/helper만 고친다.
4. 원래 계약의 resolver, 공식/외부 namespace, 경로 성분 거부, default legacy 이전, mGBA/gpSP
   격리, Resume/Fresh, quick state, switcher/delete/undo 테스트가 실제로 존재하는지 확인한다. 빠진
   계약만 현재 허용 파일 안에서 보완한다. 제품용 test hook나 새 공개 API를 테스트 때문에 만들지
   않는다.
5. 마지막 변경 뒤 원래 완료 기준 세 명령을 원문 순서로 실행한다. 실패하면 해당 실패 줄 주변만
   읽어 수정하고, 모두 종료 0이 될 때까지 이 마지막 호출 안에서 마친다.
6. `tasks/59-core-scoped-state-routing.worker-result.md`를 새로 작성해 누적 `2/2`, 시도 1의 400 중단,
   이번 수정 delta, 모든 계약 증거, 최종 명령 결과, 실코어 skip 여부, 검증 뒤 변경 여부를 기록한다.
   실패하더라도 보고서를 남기고 더 호출하지 않는다.
