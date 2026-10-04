# Task 61 — Codex 최종 판정

## 판정

**통과. 누적 호출 2/2.** 시도 1의 전환·복구 구현을 유지하고, 시도 2에서 checkpoint 단락 처리와
누락된 target Resume/save RAM 증거를 보완했다. 추가 작업자 호출은 하지 않는다.

## 확인한 구현

- `CorePicker`는 App이 소유하고 `Screen::Core(InGameMenu)`는 부모 메뉴만 보관해 `Screen: Copy`를
  유지한다. 후보는 Core 행을 열 때 registry 순서와 설치 파일의 교집합으로 한 번 만든다.
- 전환은 old Session save RAM·Resume checkpoint → settings를 바꾸지 않은 explicit target start →
  target 성공 뒤 settings commit → target namespace Resume load 순서다.
- target start 또는 setting write 실패는 변경 전 resolver로 old core와 Resume을 복구한다. recovery
  state 실패는 fresh old Session을 유지하고, recovery core 시작 실패는 Ejecting으로 빠진다.
- 플랫폼 기본 core는 `GameSettings.core = None`으로 저장하고 다른 알려진 설정과 미지 ini key를
  보존한다. 성공·복구 시 새 consumer 하나와 `SinkRequest::Open` 하나만 설치한다.

## 시도 2 보완 확인

- `flush_save`와 `save_state(Resume)`가 각각 한 번씩 독립 실행되며, 둘 다 성공한 경우에만 old
  Session을 내려놓는다.
- 유효한 mGBA Resume을 만든 뒤 gpSP에서 다른 시점까지 실행하고 mGBA로 돌아왔다. 전환 직후 저장한
  state bytes가 기존 mGBA Resume과 같고 gpSP checkpoint Resume과는 다르다.
- Resume 쓰기를 막아도 mGBA의 131072-byte save RAM이 같은 bytes로 다시 기록되고 전환은 중단된다.

## 작업자 검증 증거

- `cargo fmt --all -- --check` — 종료 0
- `cargo test -p slot2 -p slot2-i18n` — **217 passed / 0 failed / 0 ignored**
- `core_picker_app` 14 passed, 별도 `--nocapture`에서 skip 0건
- `cargo clippy -p slot2 -p slot2-i18n --all-targets -- -D warnings` — 종료 0
- 최종 검증 뒤 코드 변경 없음

Codex는 사용자 운영 규칙에 따라 테스트를 다시 실행하지 않고 누적 보고서, 제품 코드와 통합 테스트를
대조했다.

## 남은 제한

save RAM 검증은 코어가 가진 버퍼를 같은 bytes로 카드에 다시 쓰는 round-trip까지다. 테스트 ROM이
save 내용을 변경하는 게임은 아니므로 게임 내부 변경값 반영까지 증명하지는 않는다.

## 다음 방향

인게임 메뉴에서 아직 동작이 없는 행은 Device다. D-23의 밝기·블루라이트·볼륨 중 현재 platform
계약으로 구현 가능한 범위를 정하고 UI와 App 배선을 나눈다.
