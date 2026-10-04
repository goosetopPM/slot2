# Task 64 — Codex 최종 판정

## 판정

**통과. 누적 호출 1/2.** 추가 작업자 호출은 필요하지 않다. Codex는 사용자의 운영 규칙에 따라
검증 명령을 다시 실행하지 않고 작업자 보고서와 변경 코드를 대조했다.

## 통과한 부분

- `GlobalSettings { pub volume: u8 }`, 기본값 70, 최대값 100과 `Card::read_global_settings` /
  `write_global_settings`가 추가됐다.
- 파일의 부재·읽기 실패·invalid UTF-8·잘못된 volume 값은 frontend를 막지 않고 기본 70으로 읽힌다.
  정상 값은 0..=100만 인정하며 clamp로 사용자의 결정을 바꾸지 않는다.
- API로 전달된 100 초과 값은 `Error::Invalid`로 거부되고 파일을 변경하지 않는다.
- write는 기존 ini를 fallible하게 읽은 뒤 `volume` key만 변경한다. unknown/future key와 mute,
  brightness, timezone, language 형태의 기존 key를 보존한다.
- 기본 70은 owned key를 제거하고, ini가 완전히 비었을 때만 `System/slot2.ini`를 제거한다. unreadable
  원본과 directory path는 error로 남기며 덮거나 지우지 않는다.
- mute는 파일 형식에 추가되지 않았고 runtime 상태로 남았다. App, UI, audio와 platform은 바뀌지 않았다.

## 검증 근거

- 작업자 검증: `cargo fmt --all -- --check` 종료 0.
- 작업자 검증: `cargo test -p slot2-store` **66 passed / 0 failed / 0 ignored**,
  신규 `global_settings` **11 passed**.
- 작업자 검증: `cargo clippy -p slot2-store --all-targets -- -D warnings` 종료 0.
- 작업자 추가 문서 검증도 경고 0이며 최종 검증 뒤 코드 변경이 없다. Codex의 변경 파일
  `git diff --check`에도 오류가 없었다.

## 보고된 우려 판정

`GlobalSettings::apply_to`가 공개라 직접 `Ini`에 범위 밖 값을 넣을 수 있다는 우려가 보고됐다. 그러나
`Ini::set` 자체가 이미 공개이고, 이번 태스크가 보장하는 Card mutation 경로는 `write_global_settings`에서
범위를 검사한다. 기존 `GameSettings::apply_to`와 같은 API 모양이므로 추가 호출 사유로 보지 않는다.

## 다음 방향

Task65에서 App 생성 시 stored volume을 적용하고, 메뉴·물리 키 변경을 안전한 시점에 저장해야 한다.
SD 카드에 키 입력마다 불필요한 write를 반복하지 않으면서 write 실패가 runtime volume 조작을 막지 않는
구체적인 flush 계약을 먼저 명세한다. mute는 계속 runtime으로 둔다.
