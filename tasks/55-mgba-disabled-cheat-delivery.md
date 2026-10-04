# Task 55 — mGBA disabled 치트 전달 수정

## 목적

pinned mGBA adapter는 `retro_cheat_set(index, enabled, code)`의 `index`와 `enabled`를 무시한다.
현재 Session은 disabled entry도 전부 전달하므로 UI에서 off인 치트가 실제 core에서는 켜질 수 있다.

실제로 선택된 라이브러리에서 `CoreId`를 식별해 Session에 보존하고, core-specific 전달 정책은
`slot2-retro::quirks`에 둔다. mGBA에는 enabled entry만 전달하고, 다른 core의 기존 libretro
enabled 전달은 유지한다. 초기 적용·토글·실패 rollback이 모두 같은 정책을 사용해야 한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 모두 횟수에 포함한다.

## 먼저 읽을 범위

- `AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `docs/DECISIONS.md`의 D-03, D-21만
- `tasks/54-mgba-cheat-validator.result.md`
- `crates/slot2-retro/src/quirks.rs`의 공개 타입과 mGBA 부분만
- `crates/slot2-retro/src/registry.rs`의 `Core::from_library_path`만
- `crates/slot2/src/session.rs`의 `Session`, core 경로 선택, `apply_cheats`, 시작·토글 부분만
- `crates/slot2/tests/session.rs`의 core fixture와 치트 테스트 부분만
- `crates/slot2-retro/tests/cheat_quirks.rs`의 mGBA 부분만

## 고정된 근거

- pinned mGBA `retro_cheat_set`은 `UNUSED(index)`와 `UNUSED(enabled)` 뒤 전달된 code를 하나의
  enabled cheat set에 추가한다.
- 따라서 mGBA off의 올바른 구현은 reset 뒤 **enabled entry만 다시 전달**하는 것이다.
- 다른 네 core의 기존 계약은 `enabled` 인자를 포함해 file entry 전부를 순서대로 전달하는 것이다.
- 실제 선택 core는 설정 문자열이나 플랫폼 기본값을 다시 추측하지 않고, fallback까지 끝난 뒤 선택된
  `dylib`에 `CoreId::from_library_path`를 적용해 판정한다.

## 구현 계약

### 1. 코어별 전달 정책은 quirks에 둔다

`slot2-retro::quirks`에 작은 공개 정책을 추가한다. 이름은 Rust 관례에 맞게 조정할 수 있지만 다음
의미를 보존한다.

```rust
pub enum CheatDelivery {
    PassAllEntries,
    EnabledEntriesOnly,
}

pub const fn cheat_delivery(core: CoreId) -> CheatDelivery;
```

- mGBA는 `EnabledEntriesOnly`다.
- gpSP, FCEUmm, SNES9x, Genesis Plus GX는 `PassAllEntries`다.
- 이 정책은 code 형식 validator와 독립적이다. Task 54의 `Validated`/`Unchecked` 결과를 바꾸지 않는다.
- crate root에서 필요한 타입과 함수를 re-export한다.
- 왜 mGBA만 다른지는 pinned adapter가 `enabled`를 무시한다는 이유만 주석으로 남긴다.

### 2. 실제 선택 CoreId 보존

- `Session::start`가 named core 존재 여부와 기본 core fallback을 모두 해결한 뒤, 실제 `dylib`에서
  `CoreId::from_library_path`를 구한다.
- Session에 `Option<CoreId>`로 보존한다. 공식 다섯 core는 `Some`, 기존에 허용되던 알 수 없는 이름의
  외부 core는 `None`으로 두어 launch 동작을 깨지 않는다.
- 읽기 전용 accessor를 추가한다. 권장 이름은 `core_id() -> Option<CoreId>`다.
- `Debug`에 전체 경로나 설정값을 추가할 필요는 없다.
- 잘못된 named core가 없어 기본 core로 fallback한 경우 accessor는 fallback된 실제 core를 반환한다.

### 3. 하나의 apply 경로

`apply_cheats`가 core identity와 전달 정책을 받아 다음처럼 동작하게 한다.

- 매 적용 시 `reset_cheats`를 먼저 한 번 수행한다.
- `Some(Mgba)`의 `EnabledEntriesOnly`에서는 disabled entry를 `set_cheat`에 전달하지 않는다.
- enabled mGBA entry는 전달하며, index는 압축하지 않고 원래 `.cht` file index를 유지한다.
- 다른 알려진 core는 기존처럼 enabled/disabled entry를 모두 file 순서와 원래 index로 전달한다.
- 알 수 없는 `None` core도 호환성을 위해 기존처럼 모든 entry와 enabled 값을 전달한다.
- Session의 `cheats()`는 file 순서와 모든 entry를 그대로 보존한다. core에 보낸 subset으로 줄이지 않는다.
- `.cht`를 다시 읽거나 수정하지 않는다.

초기 launch, `set_cheat_enabled`의 wanted 적용, 실패 뒤 이전 상태 rollback이 모두 이 함수와 같은
identity/policy를 사용해야 한다. 한 경로만 별도 필터링하면 안 된다.

### 4. 토글과 오류 불변

- mGBA에서 off→on은 reset 후 새 enabled subset을 전달하고, on→off는 reset 후 해당 entry를 뺀
  subset을 전달한다.
- 성공 뒤에만 Session의 `enabled` 상태를 갱신한다.
- 적용 실패 시 기존 Session 상태를 유지하고 이전 subset을 같은 정책으로 복원한다. 복원도 실패하면
  기존 `Error::Cheat`가 두 오류를 보존한다.
- 이미 같은 값으로 요청한 no-op, 범위 밖 index, 파일 불변, 새 Session이 `.cht`의 enable 값으로
  다시 시작하는 계약은 유지한다.
- Task 54 validator는 이번 태스크에서 Session에 연결하지 않는다. 형식 오류와 localized toast 연결은
  나머지 core validator 이후의 별도 작업이다.

## 테스트 계약

### slot2-retro

`crates/slot2-retro/tests/cheat_quirks.rs`에서 다음을 확인한다.

- mGBA만 `EnabledEntriesOnly`
- gpSP/FCEUmm/SNES9x/Genesis Plus GX는 `PassAllEntries`
- 기존 validator 15개 회귀 유지

### Session

`crates/slot2/tests/session.rs`에서 기존 MIT GBA test ROM과 mGBA 실코어 fixture를 사용한다. core가
없는 환경은 기존 방식으로 명시적으로 skip하되 현재 개발 환경에서는 실제 실행한다.

제품용 mock hook나 호출 기록 accessor를 만들지 말고, 안전한 writable EWRAM CodeBreaker 코드가
core state에 만드는 변화를 동일 프레임 수의 baseline과 비교한다. 권장 코드는
`32000000+00AA`이며 실제 fixture에서 관찰되지 않으면 같은 형식의 다른 안전한 EWRAM 주소를 사용하고
보고서에 기록한다.

최소 다음을 직접 확인한다.

- 기본 launch와 명시적 `mgba` 설정 launch의 `core_id()`가 `Some(CoreId::Mgba)`
- 존재하지 않는 named core가 기본 mGBA로 fallback하면 `core_id()`도 mGBA
- 치트 없는 baseline과 disabled-only `.cht`를 같은 조건·프레임 수로 실행한 core state가 같음
- 같은 코드를 enabled로 시작하면 baseline과 core state가 달라 실제로 적용됨
- disabled로 시작해 첫 프레임 전에 on으로 토글하면 이후 state가 enabled 실행과 같음
- enabled로 시작해 첫 프레임 전에 off로 토글하면 이후 state가 baseline과 같음
- 여러 entry에서 중간 disabled entry가 있어도 뒤 enabled entry가 적용되며 Session 목록·원래 index와
  enabled 상태는 file 순서를 유지함
- 토글 뒤 `.cht` bytes 불변, 새 Session은 파일의 원래 enabled 값으로 복귀
- 기존 범위 오류·no-op·게임 실행 가능 테스트 회귀 유지

실코어 상태 비교가 주소 외의 비결정 상태 때문에 불가능하면 임의 제품 hook를 추가하지 않는다.
어떤 byte 범위가 달랐는지 조사하고, 두 번의 독립 baseline이 서로 같은지 먼저 확인한 뒤 계약이
틀렸다고 보고서에 적고 실패로 둔다.

## 수정 허용 범위

- `crates/slot2-retro/src/quirks.rs`
- `crates/slot2-retro/src/lib.rs`
- `crates/slot2-retro/tests/cheat_quirks.rs`
- `crates/slot2/src/session.rs`
- `crates/slot2/tests/session.rs`
- `tasks/55-mgba-disabled-cheat-delivery.worker-result.md`

그 밖의 파일은 수정하지 않는다.

## 금지·범위 밖

- 커밋·푸시
- 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- `~/.gjc-bai/agent/*.yml` 및 공용 설정 수정
- App/UI/i18n/toast 변경
- `.cht` 쓰기 또는 토글 영속화
- Task 54 validator의 Session 연결
- gpSP/FCEUmm/Genesis Plus GX 문법 validator
- 테스트 전용 제품 API, mock core, 호출 기록 hook
- 워크스페이스 전체 테스트와 배포 빌드

## 완료 기준

최종 코드 상태에서 아래를 원문 그대로 순서대로 실행한다.

```text
cargo fmt --all -- --check
cargo test -p slot2-retro --test cheat_quirks
cargo test -p slot2 --test session
cargo clippy -p slot2-retro -p slot2 --all-targets -- -D warnings
```

각 명령의 종료 코드와 마지막 결과 줄을 기록한다. Session 테스트의 실코어 실행 또는 skip 여부와
상태 비교 결과도 기록한다. 검증 뒤 소스를 고치면 네 명령을 다시 실행한다. 첫 응답까지 최대 5분,
전체 최대 45분 기다린다. 진행 신호는 로그 크기가 아니라 `git status --short`다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\55-mgba-disabled-cheat-delivery.worker-result.md`를 작성한다.

- 성공/실패와 누적 호출 횟수
- `CheatDelivery` 정책과 실제 선택 CoreId 보존 방식
- 초기 적용·양방향 토글·rollback이 같은 apply 경로를 쓰는지
- 실코어 baseline/disabled/enabled 상태 비교와 사용한 안전한 코드
- `.cht` file 순서·index·bytes 불변 및 새 Session 결과
- 네 검증 명령의 종료 코드와 마지막 결과 줄, 실코어 skip 여부
- 생성·수정 파일 목록
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄

실패해도 반드시 원인과 마지막 상태를 쓴다. 두 번째 호출도 실패하면 더 시도하지 않는다.
