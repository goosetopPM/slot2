# Task 50 — Session 치트 로드·즉시 재적용

## 목적

Task 48의 `Card::read_cheats`와 Task 49의 `Core::reset_cheats` / `set_cheat`를 실행 Session에서
연결한다. 게임 시작 전에 파일의 전체 ordered set을 적용하고, 후속 인게임 메뉴가 항목을 읽고
세션 안에서 on/off를 바꾸면 즉시 전체 재적용할 수 있는 API를 만든다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 모두 횟수에 포함한다.

## 먼저 읽을 범위

- `AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `tasks/48-retroarch-cheat-file-loader.result.md`
- `tasks/49-libretro-cheat-bridge.result.md`
- `crates/slot2/src/session.rs`에서 시작·오류·공개 설정 메서드 주변
- `crates/slot2/tests/session.rs`의 fixture와 실코어 skip 규약
- `crates/slot2-store/src/cheats.rs`의 공개 `Cheat` 계약
- 필요한 공개 API 확인 범위에서만 `crates/slot2-retro/src/host.rs`

## 구현 계약

### 1. 시작 시 읽기와 적용

- `Session::start`는 해당 cart의 `Card::read_cheats`를 정확히 한 번 읽는다.
- 파일이 없거나 `cheats = 0`이면 정상적으로 빈 목록으로 시작한다.
- 읽기·파싱 오류는 조용히 무시하지 말고 기존 `session::Error::Store` 경로로 시작을 실패시킨다.
  App의 오류 토스트 분류는 이번 범위가 아니다.
- Core가 게임을 로드한 뒤 첫 `run_frame` 전에 다음 순서로 적용한다.
  1. `reset_cheats()` 한 번
  2. 파일 순서 `0..N` 그대로 모든 항목에 `set_cheat(index, enabled, code)` 한 번씩
- disabled 항목도 생략하지 않는다. 파일 index와 libretro index가 항상 같아야 한다.
- set 도중 오류가 나면 다시 `reset_cheats()`해 부분 적용을 남기지 않고 Session 시작을 실패시킨다.
- 코드 문자열을 trim·분해·대소문자 변환·정규화하지 않는다.

### 2. Session 상태와 공개 API

- Session은 적용에 성공한 ordered `Vec<slot2_store::Cheat>`를 소유한다.
- 후속 UI가 복사 없이 읽을 수 있는 읽기 전용 accessor를 제공한다. 권장 형태:

```rust
pub fn cheats(&self) -> &[slot2_store::Cheat];
```

- 세션 내 항목의 enable 상태를 바꾸는 API를 제공한다. 권장 형태:

```rust
pub fn set_cheat_enabled(&mut self, index: usize, enabled: bool) -> Result<(), Error>;
```

- 유효한 변경은 `reset -> 전체 ordered set 재적용`으로 즉시 Core에 반영한다. 변경 대상만 별도로
  보내지 않는다.
- 범위 밖 index는 core를 건드리거나 상태를 바꾸지 않고 명확한 오류를 반환한다. 기존 Error 구조에
  최소한의 적절한 variant를 추가해도 된다.
- 같은 값 요청은 상태가 이미 충족됐으므로 성공 no-op이어도 된다.
- Session의 `Cheat.enabled`는 전체 재적용에 성공한 뒤에만 갱신한다.
- 재적용이 실패하면 core를 reset하고 이전 목록을 복구 적용하며, 메모리의 enabled 상태도 이전
  값으로 유지한다. 복구까지 실패하면 원래 오류와 복구 오류를 모두 잃지 않는 메시지를 반환한다.

### 3. 수명과 저장 정책

- 토글은 현재 Session 메모리와 실행 Core에만 적용한다.
- `.cht`를 쓰거나 `cheatN_enable`을 바꾸지 않는다. D-21에 따라 카드 치트 파일 작성·갱신은
  데스크탑 도구의 책임이다.
- 새 Session은 항상 카드 파일의 enable 값에서 다시 시작한다.
- 세이브 스테이트 load, rewind, display 설정, 종료 저장 로직은 변경하지 않는다.

## 테스트 계약

기존 MIT `assets/test/arm.gba`와 mGBA 실코어 fixture를 사용하고 기존 명시적 skip 규약을 유지한다.
코어가 있는 현재 개발 환경에서는 다음을 실제로 실행해 확인한다.

- 치트 파일이 없으면 빈 목록으로 정상 시작한다.
- 여러 항목 파일은 설명·코드·enabled·순서가 Session에 그대로 보이고, 첫 프레임 전에 시작된다.
- disabled 항목을 포함한 목록으로 프레임 실행이 계속 가능하다.
- 유효 index를 on/off하면 accessor가 즉시 바뀌고 이후 프레임도 실행된다.
- 토글은 원본 `.cht` bytes를 바꾸지 않으며, 새 Session은 파일의 원래 enabled 값으로 돌아온다.
- 범위 밖 index는 오류이고 목록과 실행 가능한 Session을 그대로 유지한다.
- 내부 NUL 코드 또는 다른 `Core::set_cheat` 준비 오류가 시작 중 발생하면 부분 Session을 반환하지
  않는다.
- 손상된 `.cht`는 `Error::Store`로 시작 실패하고, 없는 파일과 혼동되지 않는다.

호출 순서 자체를 관찰하려고 제품 코드에 테스트 전용 branch나 mock hook를 추가하지 않는다.
실제 Core 통합과 공개 상태·파일 불변으로 계약을 검증한다.

## 수정 허용 범위

- `crates/slot2/src/session.rs`
- `crates/slot2/tests/session.rs`
- 꼭 필요한 경우에만 `crates/slot2/src/lib.rs`
- `tasks/50-session-cheat-application.worker-result.md`

그 밖의 소스는 수정하지 않는다. 특히 `slot2-store`, `slot2-retro`, `slot2-ui`, `app.rs`, 언어
리소스와 공용 GJC 설정은 건드리지 않는다.

## 금지·범위 밖

- 커밋·푸시
- 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- `~/.gjc-bai/agent/*.yml` 및 공용 설정 수정
- `.cht` 쓰기·수정·생성 API
- 인게임 치트 메뉴와 App 입력 연결
- 코어별 코드 형식 validator/quirks — D-21의 이 항목은 별도 후속 태스크
- libretro FFI 재설계
- 워크스페이스 전체 테스트와 배포 빌드

## 완료 기준

최종 코드 상태에서 아래를 원문 그대로 순서대로 실행한다.

```text
cargo fmt --all -- --check
cargo test -p slot2 --test session
cargo clippy -p slot2 --all-targets -- -D warnings
```

각 명령의 종료 코드와 마지막 결과 줄을 기록한다. 검증 뒤 소스를 고치면 세 명령을 다시 실행한다.
첫 응답까지 최대 5분, 전체 최대 45분 기다린다. 진행 신호는 로그 크기가 아니라
`git status --short`다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\50-session-cheat-application.worker-result.md`를 작성한다.

- 성공/실패와 누적 호출 횟수
- 시작 시 읽기·reset·ordered 전체 적용 동작
- accessor와 토글 API의 실제 최종 형태
- 오류 시 부분 적용 제거와 rollback 결과
- 파일 불변·새 Session 초기화 검증
- 실코어 테스트 실행 또는 skip 여부
- 세 검증 명령의 종료 코드와 마지막 결과 줄
- 생성·수정 파일 목록
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄

실패해도 반드시 원인과 마지막 상태를 쓴다. 두 번째 호출도 실패하면 더 시도하지 않는다.
