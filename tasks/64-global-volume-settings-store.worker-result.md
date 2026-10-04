# Task 64 — 전역 Volume 설정 저장 기반 (결과)

- 결과: **성공**. 누적 호출 **1/2**(이번이 첫 호출이며 재시도하지 않았다).
- 범위: `slot2-store` 한 크레이트. `card.rs`/`ini.rs`/`atomic.rs` production 변경 없음(요구대로
  `settings_path()`와 `Ini`/`atomic_write` 재사용). App·Session·UI·audio·platform 변경 없음.

## GlobalSettings / default / range / invalid parse 계약

- `pub const DEFAULT_VOLUME_LEVEL: u8 = 70`, `pub const MAX_VOLUME_LEVEL: u8 = 100`을 store 계약으로
  두고 lib.rs에서 re-export했다. audio 크레이트 의존은 추가하지 않았다.
- `pub struct GlobalSettings { pub volume: u8 }`(`Clone, Copy, Debug, PartialEq, Eq`)와 수동 `Default`
  (`volume == 70`)를 제공한다. `KEY_VOLUME = "volume"` 하나만 소유한다.
- 읽기: `from_ini`가 `volume`만 보고, 없거나 해석 불가면 default 70이다. `parse_level`은 trim 후
  **전부 ASCII 숫자**이고 `<= 100`일 때만 값을 인정한다. 따라서 빈 값·문자·`-5`·`7.5`·`1 0`·`101`·
  `1000`·`+5`가 모두 default로 떨어지고, 0/100으로 몰래 바뀌지 않는다. `volume= 85` 같은 앞뒤 공백은
  정상 허용된다(테스트 4종 spelling).
- 쓰기: `write_global_settings(&GlobalSettings)`가 `volume > 100`을 **파일을 읽기도 전에**
  `Error::Invalid`로 거부한다(clamp·저장 없음). 그 외에는 `volume = <decimal>`을 설정하고, 값이 70이면
  소유 key를 제거한다.

## safe write / unknown key / 기본 복귀 / unreadable 원본

- mutation 전에 `Ini::load(&path)?`로 fallible하게 읽는다. invalid UTF-8 파일과 `settings_path()`가
  directory인 경우 모두 `Error::Io(path, _)`를 반환하며 원본 bytes/directory를 그대로 둔다(두 write
  경로 = 값 변경·기본 복귀 모두 확인).
- 비기본 write는 `ini.set("volume", decimal)`, 기본 write는 `ini.remove("volume")`만 한다.
  `future_key`, `utc_offset_minutes`, `language`, `mute`/`muted`/`brightness` 같은 소유하지 않은 key는
  값과 key spelling 그대로 남는다(정규화된 `key = value` 정렬 표기를 바이트 단위로 단언).
- 소유 key를 제거한 뒤 ini가 비면 파일을 지우고, unknown key가 남으면 파일을 보존한다. 파일 부재 +
  기본 write는 **성공 no-op**(파일 생성 없음). `System/` 폴더가 아예 없는 새 카드에 첫 비기본 write를
  하면 `atomic_write`가 폴더와 파일을 만든다.
- 삭제 실패(`remove_file`)와 save 실패는 `Error::Io`로 호출자에게 반환한다. 성공하지 않은 write를
  성공으로 보고하는 경로가 없다.
- 읽기는 관대하다: 부재·손상·invalid UTF-8 모두 default이며 파일을 만들거나 고치거나 지우지 않는다
  (읽기 전후 `System/` 항목 수 불변 단언).

## 소유하지 않은 key 확인

- 새로 쓰는 파일 텍스트는 정확히 `volume = <n>\n` 하나이고, `mute`/`muted`/`bright`/`blue`/`utc`/
  `offset`/`lang`/`time` 문자열이 나타나지 않음을 단언했다. mute는 저장하지 않으므로 재시작 뒤
  이유 없는 무음 상태가 생기지 않는다.
- 그런 모양의 key가 이미 파일에 있으면 `from_ini`는 읽지 않고 `apply_to`도 건드리지 않는다.
- 전역 파일 write가 게임별 `System/games/<PLAT>/<stem>.ini`를 건드리지 않음을 별도 테스트로 확인했다.

## 완료 기준 명령 (원문 순서, 마지막 코드 변경 뒤, 이후 코드 변경 없음)

1. `cargo fmt --all -- --check` → exit **0**, 출력 없음.
2. `cargo test -p slot2-store` → exit **0**. `test result:` 줄 7개, 합계 **66 passed / 0 failed /
   0 ignored**. 신규 `global_settings` **11 passed**, 기존 `card` 19, `cheats` 15, `core_states` 9,
   `state_undo` 11, lib 1(모두 불변). 마지막 결과 줄: `test result: ok. 0 passed; 0 failed; 0 ignored;
   0 measured; 0 filtered out; finished in 0.00s` (Doc-tests slot2_store).
3. `cargo clippy -p slot2-store --all-targets -- -D warnings` → exit **0**,
   `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.95s`.

- 추가 확인: `cargo doc -p slot2-store --no-deps` 경고 0(새 intra-doc link 3개 포함).
- 음성 대조: 검증 전에 (a) 범위 검사를 제거하고 (b) `Ini::load` 오류를 빈 ini로 취급하도록 일시
  변형했더니 신규 테스트 중 `an_out_of_range_level_is_refused_and_the_file_is_left_alone`과
  `an_unreadable_file_is_never_overwritten`가 각각 FAILED(2 failed / 9 passed)가 되었다. 원복 후
  66 passed / 0 failed 재확인.

## 생성·수정 파일

- 생성: `crates/slot2-store/tests/global_settings.rs`(11 tests),
  `tasks/64-global-volume-settings-store.worker-result.md`.
- 수정: `crates/slot2-store/src/settings.rs`(module doc + `DEFAULT_VOLUME_LEVEL`/`MAX_VOLUME_LEVEL`/
  `GlobalSettings`/`parse_level`/`read_global_settings`/`write_global_settings`),
  `crates/slot2-store/src/lib.rs`(re-export 1줄 확장).
- `src/card.rs`(+615)·`tests/card.rs`(+121)는 이번 세션 시작 시점과 동일해 손대지 않았음을
  `git diff --stat`으로 확인했다. 최종 검증 뒤 코드 변경 없음. 커밋·푸시·네트워크·실기·공용 설정
  변경·위임 없음.

## 계약 의견 / 남은 위험

- `GlobalSettings::apply_to`를 `GameSettings::apply_to`와 같은 공개 shape으로 두었으므로, `Ini`를 직접
  가진 호출자는 범위 검사 없이 `volume = 101`을 쓸 수 있다(문서에 명시했고 검증 경로는
  `write_global_settings`). 공개 범위를 줄이려면 `apply_to`를 crate-private로 내리고 그 테스트를
  Card 경로로 바꾸면 된다.

## 소요 시간

약 15분(22:08–22:22 KST). 누적 호출 1/2.
