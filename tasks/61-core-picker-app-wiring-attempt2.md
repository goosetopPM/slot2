# Task 61 — 시도 2/2: checkpoint 비단락 실행과 target Resume 증거 보완

현재 working tree의 Task61 구현을 그대로 이어서 수정한다. 시도 1의 구현을 되돌리거나 다시 만들지
않는다. 이번이 **누적 마지막 2/2 호출**이다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시한다.
- `C:\SLOT2\tasks\61-core-picker-app-wiring.md`의 핵심 안전 계약과 테스트 계약
- `C:\SLOT2\tasks\61-core-picker-app-wiring.worker-result.md`
- `C:\SLOT2\crates\slot2\src\app.rs`의 `switch_core` checkpoint 구간만
- `C:\SLOT2\crates\slot2\tests\core_picker_app.rs`의 fixture, 성공 전환 테스트만
- 필요하면 `crates/slot2/tests/session.rs`의 Session 생성·state/save helper만

## Codex 검토에서 확인된 미완료

### 1. checkpoint가 두 작업을 모두 시도하지 않는다

현재 코드는 다음 의미다.

```rust
session
    .flush_save(&self.card)
    .and_then(|_| session.save_state(&self.card, StateKind::Resume))
```

따라서 save RAM flush가 실패하면 Resume 저장은 실행되지 않는다. 원 명세는 살아 있는 Session에서
두 작업을 **모두 시도**하고, 둘 중 하나라도 실패하면 Session을 유지하도록 요구했다.

- `flush_save`와 `save_state(Resume)`를 각각 한 번씩 평가한다.
- 둘 다 성공한 경우에만 기존 Session을 내려놓고 전환한다.
- 하나 또는 둘 다 실패하면 Session/settings/sink를 그대로 유지하고 `core-checkpoint-failed`로 남는다.
- 실패 로그는 실제 실패한 단계가 무엇인지 숨기지 않는다. 한 단계 실패를 다른 단계의 오류로 덮지 않는다.
- 일반 `Session::stop` 계약은 바꾸지 않는다.

### 2. 유효한 target Resume load 테스트가 없다

시도 1 보고서는 gpSP→mGBA 전환에서 “target mGBA Resume을 load하고 저장 byte가 일치했다”고 했지만,
현재 `going_back_to_the_platforms_own_core...` 테스트는 mGBA Resume을 미리 만들지 않고 load 결과도
비교하지 않는다. 깨진 target Resume과 Resume 없음은 검증됐지만 **유효한 target Resume** 경로가 빠졌다.

- 같은 core가 만든 유효한 target Resume을 전환 전에 준비한다.
- 다른 core에서 충분히 다른 시점으로 실행한 뒤 target으로 전환한다.
- 전환 직후 core frame을 진행시키지 않은 상태에서 state를 다시 저장하거나 동등한 observable 방법으로
  target Resume의 bytes/state가 실제로 load됐음을 확인한다.
- target core namespace만 읽었고, 떠난 core의 checkpoint Resume은 보존됐음을 함께 확인한다.
- 플랫폼 기본 core로 돌아가는 경우 settings `core == None`과 기존/미지 ini key 보존 단언은 유지한다.

### 3. save RAM 증거를 정확히 한다

기존 `arm.gba`는 save RAM이 없을 수 있으므로 시도 1 테스트는 save RAM flush를 직접 증명하지 못한다.
가능하면 테스트의 임시 ROM 복사본에 libretro core가 인식하는 SRAM signature를 넣어 save memory를
노출하고, Session 시작 전에 넣은 save를 시작 후 카드에서 제거한 다음 core 전환 checkpoint가 같은
save를 다시 기록하는 방식으로 flush 순서를 직접 검증한다. 저장소에 새 ROM이나 상용 ROM을 추가하지
않는다.

이 방식이 두 실제 core에서 신뢰성 있게 동작하지 않으면 테스트를 억지로 맞추지 말고, 보고서에서
“호출 순서는 코드로 확인했지만 MIT ROM은 SaveRam을 노출하지 않아 파일 round-trip은 확인 못 함”이라고
정확히 적는다. 확인하지 않은 save RAM round-trip을 확인했다고 쓰지 않는다.

## 수정 범위

- `C:\SLOT2\crates\slot2\src\app.rs` — checkpoint의 독립 실행/오류 집계만
- `C:\SLOT2\crates\slot2\tests\core_picker_app.rs` — 위 누락 증거만
- 직접 필요할 때만 `C:\SLOT2\crates\slot2\tests\session.rs`
- `C:\SLOT2\tasks\61-core-picker-app-wiring.worker-result.md` — 누적 2/2 최종 보고로 갱신

다른 제품 동작, UI, i18n, registry, store 계약은 바꾸지 않는다. 관련 없는 미커밋 변경을 정리·복원·
재포맷하지 않는다.

## 완료 기준

마지막 코드 변경 뒤 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2 -p slot2-i18n
cargo clippy -p slot2 -p slot2-i18n --all-targets -- -D warnings
```

모두 종료 0이어야 한다. mGBA/gpSP 전환 테스트가 skip되지 않았는지 확인한다. 검증 뒤 코드를 바꾸면
영향받는 명령부터 다시 실행한다.

## 최종 보고서

기존 `C:\SLOT2\tasks\61-core-picker-app-wiring.worker-result.md`를 누적 **2/2** 보고서로 갱신한다.

- 시도 1 결과를 보존하고 이번 delta를 분리
- flush와 Resume을 독립적으로 실행하는 최종 순서
- 유효한 target Resume을 준비·load·비교한 구체적 증거
- save RAM round-trip의 실제 확인 여부와 한계
- 세 완료 기준의 종료 코드·마지막 결과 줄, skip 여부
- 수정 파일, 검증 뒤 코드 변경 여부, 남은 위험, 소요 시간

코드·로그 전문은 넣지 않는다. 네트워크·실기·공용 설정·위임·커밋·푸시는 금지한다. 실패해도 보고서를
남기고, 이번 호출 뒤에는 세 번째 시도를 하지 않는다.
