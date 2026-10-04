# Task 65 — Codex 최종 판정

## 판정

**통과. 누적 호출 1/2.** 추가 작업자 호출은 필요하지 않다. Codex는 사용자의 운영 규칙에 따라
검증 명령을 다시 실행하지 않고 작업자 보고서와 변경 코드를 대조했다.

## 통과한 부분

- `App::with_card`가 store의 전역 설정을 한 번 읽어 `Volume::new`에 적용한다. 저장값 0..=100을
  그대로 사용하고 missing/unreadable/invalid는 store의 기본 70을 따르며 mute는 false로 시작한다.
- 물리 VolUp/VolDown과 Device Left/Right가 같은 step helper를 사용한다. 실제 level이 바뀐 경우에만
  마지막 변경 시각부터 750ms 뒤로 저장을 예약한다.
- 연속 변경은 deadline을 다시 잡고, 저장값 복귀는 pending을 취소한다. clamp 입력과 mute toggle은
  새 write를 만들거나 기존 deadline을 미루지 않는다.
- tick은 gesture action 뒤 due를 검사하므로 같은 tick에서 이전 level을 먼저 저장하는 경로가 없다.
- 저장 성공 시 persisted level을 갱신한다. 실패 시 runtime volume·screen·session·sink를 유지하고
  pending을 소진해 frame별 retry/log 폭주를 막으며, 다음 실제 변경은 새 저장을 시도한다.
- physical Power, Power menu Restart와 PowerOff가 공통 helper에서 즉시 flush를 시도한 뒤 기존 Exit을
  설정한다. write 실패도 종료를 막지 않는다.
- unknown/future key와 game ini가 보존되고 mute, brightness, blue-light, timezone key가 추가되지 않았다.

## 검증 근거

- 작업자 검증: `cargo fmt --all -- --check` 종료 0.
- 작업자 집중 검증: `device_menu_app` **13 passed**, 신규 `global_volume_app` **16 passed**.
- 작업자 lib 검증: **33 passed / 0 failed**.
- 작업자 clippy: `cargo clippy -p slot2 --all-targets -- -D warnings` 종료 0.
- 작업자 추가 `cargo test -p slot2`: **219 passed / 0 failed / 0 ignored**.
- 최종 검증 뒤 코드 변경이 없고 Codex의 변경 파일 `git diff --check`도 오류가 없다.

## 보고된 한계 판정

저장값으로 복귀했을 때 pending 자체가 취소됐는지는 filesystem bytes만으로 no-op flush와 구분되지 않는다.
코드에서 `volume_due = None` 분기를 직접 확인했고 이후 tick의 observable 결과도 write 없음이므로 추가
instrumentation은 필요하지 않다.

## 다음 방향

전역 volume의 UI·runtime·store 경로가 닫혔다. Brightness와 blue light는 실기 경로가 확정될 때까지
Unavailable로 유지한다. 다음 비실기 작업은 D-23 Display의 남은 shader/overlay 기반을 크레이트 경계로
분리해 진행하는 것이 적절하다.
