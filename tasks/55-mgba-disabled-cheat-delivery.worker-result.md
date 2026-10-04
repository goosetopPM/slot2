# Task 55 워커 결과 — mGBA disabled 치트 전달 수정

**성공. 누적 호출 1/2.** 지정 검증 4종 모두 종료 0. Session 테스트는 mGBA 실코어로 실행됐다
(skip 없음: `vendor/mgba_libretro.dll`을 `core_dir()`이 찾았고 27개가 실제로 돌았다).

## `CheatDelivery` 정책과 실제 선택 CoreId 보존

- `slot2-retro::quirks`에 `pub enum CheatDelivery { PassAllEntries, EnabledEntriesOnly }`와
  `pub const fn cheat_delivery(core: CoreId) -> CheatDelivery`를 추가하고 crate root에서
  re-export했다. mGBA만 `EnabledEntriesOnly`이고 gpSP·FCEUmm·SNES9x·Genesis Plus GX는
  `PassAllEntries`다. 이유는 pinned mGBA adapter의 `retro_cheat_set`이 `UNUSED(index)`와
  `UNUSED(enabled)` 뒤 code를 enabled set에 넣는다는 한 가지만 주석으로 남겼다. 이 정책은 Task 54의
  `Validated`/`Unchecked` 판정과 독립이며 validator 결과는 바뀌지 않았다.
- `Session::start`가 named core 존재 여부와 기본 core fallback을 **모두 해결한 뒤** 실제 `dylib`에
  `CoreId::from_library_path`를 적용해 `core_id: Option<CoreId>`로 보존한다. 공식 다섯 core는
  `Some`, 카드가 지정한 알 수 없는 외부 library는 `None`이라 launch 동작이 그대로다. 읽기 전용
  `core_id() -> Option<CoreId>` accessor를 추가했고 `Debug`에는 경로를 넣지 않았다.

## 하나의 apply 경로

`apply_cheats(core, identity, cheats)`가 유일한 적용 경로다: 매번 `reset_cheats()` 한 번 →
`identity`의 정책이 `EnabledEntriesOnly`면 disabled entry를 건너뛰고 → 전달할 때는 **원래 file
index**를 그대로 쓴다(index 압축 없음). 초기 launch, `set_cheat_enabled`의 wanted 적용, 실패 뒤 이전
목록 rollback이 모두 이 함수와 `self.core_id`를 쓰므로 한 경로만 따로 필터링하는 곳이 없다. 알 수
없는 `None` core는 예전처럼 모든 entry와 enabled 값을 받는다.

## 실코어 상태 비교 (사용한 코드)

12 프레임 고정, 같은 카드 fixture, 버튼 없음·기본 볼륨으로 `core_state()`(serialize)를 비교했다.
안전한 writable EWRAM CodeBreaker 코드는 계약이 권장한 `32000000+00AA`를 그대로 썼고, 실제
fixture에서 관찰됐으므로(아래 enabled ≠ baseline) 다른 주소로 바꾸지 않았다. 누출 감지용 두 번째
코드는 `3203FFF0+00BB`다.

- 같은 fixture 두 번 → **동일**(비교 수단의 전제 확인)
- disabled-only `.cht` → baseline과 **동일**: off 항목이 core에 가지 않는다
- 같은 코드 enabled → baseline과 **다름**: 실제로 적용된다
- disabled로 시작해 첫 프레임 전 on 토글 → "처음부터 on" 실행과 **동일**
- enabled로 시작해 첫 프레임 전 off 토글 → 치트 없는 baseline과 **동일**
- 앞·중간이 disabled이고 마지막이 enabled인 3항목 파일(중간에 별도 코드 포함) → 그 enabled 항목만
  보낸 참조 실행과 **동일**하고 baseline과는 **다름**(disabled가 새지 않음)

## `.cht` 순서·index·bytes

Session의 `cheats()`는 전달 subset으로 줄지 않고 파일 전체를 순서·flag 그대로 보존한다(3항목 파일에서
len 3, `[false, false, true]`, description 순서 확인). 전달 index는 항상 file index다. 토글 후
`.cht` bytes 불변과 새 Session이 파일의 원래 enabled 값으로 복귀하는 계약은 기존 Task 50 테스트
(`toggling_a_cheat_leaves_the_file_alone_and_a_new_session_starts_from_it`)가 그대로 통과해 유지된다.
`.cht`를 다시 읽거나 수정하는 코드는 추가하지 않았다.

## 검증 (최종 코드 상태, 명세 순서)

- `cargo fmt --all -- --check` → 종료 **0**.
- `cargo test -p slot2-retro --test cheat_quirks` → 종료 **0**,
  `test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s`.
- `cargo test -p slot2 --test session` → 종료 **0**,
  `test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.40s`
  (실코어 실행, skip 없음).
- `cargo clippy -p slot2-retro -p slot2 --all-targets -- -D warnings` → 종료 **0**(경고·오류 없음).
- 최종 검증 후 코드 변경 없음.

## 생성·수정 파일

- `crates/slot2-retro/src/quirks.rs`: `CheatDelivery`와 `cheat_delivery`.
- `crates/slot2-retro/src/lib.rs`: 두 항목 re-export.
- `crates/slot2-retro/tests/cheat_quirks.rs`: 정책 2 테스트(＋validator 15개 회귀 유지).
- `crates/slot2/src/session.rs`: `core_id` 보존·accessor, `apply_cheats` 정책 적용.
- `crates/slot2/tests/session.rs`: core identity 1개, 상태 비교·토글 3개 테스트와 helper 3개.
- App/UI/i18n/toast와 `.cht` 경로는 건드리지 않았다.

## 계약이 틀려 보이는 부분 / 남은 위험

기존 `a_cheat_the_core_cannot_take_is_not_a_half_started_session`은 NUL 항목이 disabled였기 때문에
정책 변경 뒤에도 통과하려면 그 항목을 **enabled**로 바꿔야 했다(회귀 의도는 유지: 전달되는 코드가
C 문자열이 될 수 없으면 launch가 실패한다). 이는 새 정책이 만든 정당한 fixture 수정이지 계약 위반이
아니다. 남은 위험은 카드가 지정한 알 수 없는 외부 core(`None`)에는 이 정책이 적용되지 않아 예전처럼
disabled 항목까지 `enabled=false`로 전달된다는 점(계약대로 호환 우선)이다. 소요 약 10분(상한 45분 내).
