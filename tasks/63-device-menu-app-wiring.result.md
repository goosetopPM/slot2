# Task 63 — Codex 최종 판정

## 판정

**통과. 누적 호출 1/2.** 추가 작업자 호출은 필요하지 않다. Codex는 사용자의 운영 규칙에 따라
검증 명령을 다시 실행하지 않고 작업자 보고서와 변경 코드를 대조했다.

## 통과한 부분

- `Screen::Device(InGameMenu, DeviceMenu)`가 추가됐고 Device 행에서 현재 runtime volume 70,
  unmuted snapshot으로 열린다. parent의 Device 선택 위치를 보존한다.
- brightness와 blue light는 검증된 backend가 없으므로 계속 `None`/Unavailable이며 hardware 경로나
  가짜 값을 만들지 않았다.
- Left/Right/A와 물리 VolUp/VolDown이 모두 하나의 `App.volume`을 먼저 변경하고 열린 menu snapshot을
  그 결과에서 다시 만든다. step, clamp, mute 유지·해제 의미가 `slot2_audio::Volume`과 일치한다.
- Device 화면은 core와 audio를 pause하고 sink/session/store를 변경하지 않는다. 마지막 game frame 뒤
  overlay를 그리며 wallpaper, HUD, parent menu와 clear를 추가하지 않는다.
- B/Menu가 같은 Device parent row로 돌아가며 level/mute는 gameplay 복귀와 메뉴 재진입 동안 runtime으로
  유지된다. 설정 파일 영속성은 추가하지 않았다.
- `DeviceMenu`에는 `Screen: Copy`를 유지하는 데 필요한 `Copy` derive만 추가됐다.

## 검증 근거

- 작업자 검증: `cargo fmt --all -- --check` 종료 0.
- 작업자 검증: `cargo test -p slot2 -p slot2-ui` 합계 **398 passed / 0 failed / 0 ignored**,
  신규 `device_menu_app` **13 passed**, core-dependent skip 0.
- 작업자 검증: `cargo clippy -p slot2 -p slot2-ui --all-targets -- -D warnings` 종료 0.
- 최종 검증 뒤 코드 변경 없음. Codex의 `git diff --check`도 오류가 없었다.

## 계약 해석

작업자는 인공적으로 session 없는 `Screen::InGame` 상태에서도 Device 화면을 열도록 했다. 실제 제품
진입은 실행 중 게임에서만 가능하고 volume은 frontend 전역 상태이며, session 없는 overlay도 panic하지
않아야 한다는 명세와 충돌하지 않는다. 별도 수정 사유로 보지 않는다.

## 남은 범위

Brightness와 blue light는 RG SP의 실제 제어 경로와 안전한 적용 계약이 확인될 때까지 Unavailable이다.
Task63은 runtime volume만 다루므로 재시작 뒤 volume 영속성도 아직 없다. 다음 태스크는 실기 경로를
추측하지 않고 M4의 남은 비실기 기능 중 하나를 분리해 진행해야 한다.
