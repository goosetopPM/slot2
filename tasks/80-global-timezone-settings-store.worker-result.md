# Task 80 작업자 보고서

## 결과

성공. 누적 호출 1/2 (실행 호출 1회).

## 공개 계약

- 상수(`slot2_store` root에서 re-export):
  - `DEFAULT_UTC_OFFSET_MINUTES: i32 = 0`
  - `UTC_OFFSET_MINUTES_MIN: i32 = -720`
  - `UTC_OFFSET_MINUTES_MAX: i32 = 840`
  - `UTC_OFFSET_MINUTES_KEY: &str = "utc_offset_minutes"` (계약이 요구한 key를 read/write가 공유하는
    단일 출처로 두었다. 필수는 아니었지만 key 문자열이 두 곳에서 갈라지지 않게 한다.)
- 저장 key는 정확히 `utc_offset_minutes`, 기본값 0, 유효 범위는 -720..=840(양 끝 포함).
- parse 계약: `str::trim`(ASCII·Unicode 공백 모두) 뒤 `i32::from_str`로 10진 정수, 그 다음 범위 검사.
  - 허용: `0`, `540`, `-480`, `+60`, `-720`, `840`, 그리고 `utc_offset_minutes=540`,
    `utc_offset_minutes =540`, `  utc_offset_minutes  =  -480  `, NBSP로 감싼 값.
  - 거부(→ 기본 0): 빈 값, `abc`, `9.5`, `1e3`, `1 0`, `540 KST`, `540KST`, `-721`, `841`, `86400`,
    `2147483648`, `-2147483649`, `99999999999999999999`, `--60`, `0x21c`, `=` 없는 줄.
  - clamp·modulo·반올림 없음. 거부된 값은 원문 그대로 남는다.
- 읽기 `Card::read_utc_offset_minutes() -> i32`: 파일 부재, key 부재, 읽기 실패, invalid UTF-8,
  settings path가 directory인 경우 모두 0을 돌려주고 파일/path/다른 key를 건드리지 않는다.
- 쓰기 `Card::write_utc_offset_minutes(i32) -> Result<(), Error>`: 범위 밖은 파일을 읽기 전에
  `Error::Invalid`로 거부하고 bytes 불변. 그 밖에는 `Ini::load`를 fallible하게 먼저 읽고, non-default는
  `utc_offset_minutes = <signed decimal>`(`+` 없음), 0은 소유 key만 제거한 뒤 ini가 비면 파일 제거,
  파일이 원래 없으면 성공 no-op. 삭제 실패와 save 실패는 `Error::Io`로 반환한다.

## 검증 결과

- 독립 safe write: 시간대 write는 `volume`과 unknown key를 값 그대로 보존한다. 정상 volume과
  `volume = nope`(invalid) 모두 추가·변경·기본 복귀 세 단계에서 그대로 남았다.
- 기본 복귀: `utc_offset_minutes` key만 제거되고, 다른 key가 남으면 파일도 남고, ini가 완전히 비면
  파일이 제거된다(`utc_offset_minutes = nope`만 있던 파일도 제거 대상).
- unreadable 원본 보존: invalid UTF-8 파일과 directory path 모두에서 non-default(540)와 default(0)
  write가 `Error::Io`로 실패하고 bytes/path가 정확히 보존된다. 같은 파일에 대한 volume write도 같은
  이유로 거부되어 조용히 고쳐지지 않는다.
- 양방향 보존: volume write가 정상/invalid `utc_offset_minutes`를 그대로 남기고(빈 값 포함),
  시간대 write가 정상/invalid volume과 unknown key를 그대로 남긴다. 하나의 key를 기본값으로 되돌려도
  다른 key가 있으면 파일은 삭제되지 않는다.
- 환경/시계: `SLOT2_UTC_OFFSET_MIN`은 코드에서 읽지도 쓰지도 않으며, 테스트는 env를 변경하지 않는다.
  반환값이 항상 파일 내용만 따르는지(파일에 값이 생기고 사라지는 전환에서 같은 답을 반복하지 않음),
  호출 전후 `std::env::var_os("SLOT2_UTC_OFFSET_MIN")`이 동일한지, 그리고 `System`에 생기는 유일한
  파일이 `slot2.ini`인지(시각·mtime·local-time 사본 없음)를 확인했다. 실제 env 설정 검증은 계약의
  "테스트용 환경변수 변경 금지" 때문에 하지 않았고, 대신 값이 파일에서만 온다는 것을 직접 단언했다.
- volume 계약 회귀: `GlobalSettings`, `read_global_settings`, `write_global_settings`의 signature와
  default 70/최대 100은 변경하지 않았고 기존 `global_settings` 테스트 11개가 그대로 통과한다.
- `crates/slot2-store/tests/global_settings.rs`는 수정하지 않았다. 기존
  `unknown_keys_survive_every_write`가 이미 volume write의 `utc_offset_minutes` 보존을 덮고 있고,
  invalid 값 방향과 시간대 write 방향은 새 파일에서 검증했기 때문이다.

## 완료 기준 명령

| 명령 | 종료 코드 | 마지막 결과 |
|---|---|---|
| `cargo fmt --all -- --check` | 0 | 출력 없음 (아래 참고) |
| `cargo test -p slot2-store --test timezone_settings --test global_settings` | 0 | 12 + 11 = 23 passed / 0 failed / 0 ignored |
| `cargo test -p slot2-store` | 0 | 결과 줄 10개, 101 passed / 0 failed / 0 ignored |
| `cargo clippy -p slot2-store --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile ... in 5.04s` |

첫 `cargo fmt --all -- --check`는 새 테스트 파일의 줄바꿈 몇 곳만 지적했고, `cargo fmt --all`로 정리한
뒤 다시 실행한 최종 `--check`가 종료 0이다. 그 뒤로 코드를 바꾸지 않았고 나머지 명령은 모두 최종 상태
에서 실행했다. core-dependent skip은 없다(store에는 코어가 없다).

## 파일

수정:
- `crates/slot2-store/src/settings.rs` (상수 4개, parse helper, `read_utc_offset_minutes`,
  `write_utc_offset_minutes`, 그리고 파일이 이제 두 key를 소유한다는 모듈/`GlobalSettings` doc)
- `crates/slot2-store/src/lib.rs` (새 상수 re-export)

생성:
- `crates/slot2-store/tests/timezone_settings.rs` (12 tests)

`card.rs`, `ini.rs`, `atomic.rs` production과 `slot2-platform`·UI·App은 수정하지 않았다. 최종 검증 뒤
코드 변경 없음. 임시 `target/t80-*.txt`는 삭제했다.

## 후속 배선과 남은 위험

- 후속: App 시작 시 `read_utc_offset_minutes()`를 한 번 읽어 `slot2_platform::clock::set_utc_offset_min`에
  적용하고, 저장 크레이트와 platform의 범위 상수(`UTC_OFFSET_MINUTES_MIN/MAX` vs
  `clock::OFFSET_MIN/MAX`)가 같은지 검증하는 것, 그리고 시간대 선택 UI·즉시 반영·저장 실패 처리가
  남아 있다.
- 남은 위험 1줄: store와 platform이 각자 범위 상수를 들고 있어, 후속 배선이 둘의 일치를 검증하지 않으면
  카드가 받아들인 값(-720..=840)을 clock이 거부하는 상태가 생길 수 있다.

## 계약 의문

없음. 다만 "환경변수를 사용하지 않음"은 계약이 테스트에서 env 변경을 금지하므로 위와 같이
"값이 파일에서만 온다 + env가 호출로 변하지 않는다"로 검증했고, env 의존성 자체를 반증하는 테스트는
만들지 않았다(금지 조항 우선).

## 소요 시간

약 5분 (00:09 ~ 00:14 KST).
