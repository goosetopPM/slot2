# Task 62 — Codex 최종 판정

## 판정

**통과. 누적 호출 2/2.** 시도 1은 provider 400으로 중단됐고 시도 2에서 구현·검증을 마쳤다.
사용자 결정으로 동적 값/상태 문자열은 최초 표시 때 1회 texture upload를 허용하고, 같은 문자열의
반복 draw는 추가 upload 0회라는 계약으로 정정했다. 추가 작업자 호출은 하지 않는다.

## 통과한 부분

- 공개 `DeviceMenu`와 `DeviceSetting::{Volume, Brightness, BlueLight}`가 추가됐다.
- 값 clamp, `None` availability 보존, available 행만 순환하는 탐색과 setter 계약이 구현됐다.
- overlay는 game frame을 clear하지 않고 전체 panel dim, safe-area panel, 세 행·bar·상태·hint를 그린다.
- muted Volume은 기억된 bar를 dim 처리하고, unavailable 행은 선택·highlight되지 않는다.
- 세 geometry×영·한 layout, percentage Fluent argument와 message 직접 정의가 검증됐다.
- 작업자 검증: fmt 종료 0, UI+i18n 합계 223 passed / 0 failed / 0 ignored, clippy 종료 0.

## 정정된 warm-cache 계약

원 지시서는 warm draw 뒤 selection 이동뿐 아니라 **새 level·mute 문자열을 처음 표시할 때도** 새
`UploadAlpha8`/`UploadRgba8`가 0이어야 한다고 요구했다.

최종 테스트는 같은 상태 반복과 selection 이동은 0회로 검증하지만:

- 처음 `Muted`를 표시할 때 최대 1회 upload를 허용한다.
- 처음 `100%`를 표시할 때 정확히 1회 upload가 발생한다고 단언한다.

현재 공용 `FaceCache`는 완성 문자열 단위로 캐시하므로 처음 등장한 동적 percentage/status 문자열은
1회 upload되고 이후 재사용된다. 101개 percentage를 미리 올리는 방식보다 현재 동작이 효율적이므로
사용자가 이를 Task62의 최종 계약으로 수락했다.

최종 검증은 같은 상태 8회 반복과 selection 이동에서 upload 0회, 처음 `Muted` 표시에서 최대 1회,
처음 `100%` 표시에서 정확히 1회이며 이후 같은 문자열은 cache를 사용한다.

## 다음 방향

Task63에서 Device 행을 App에 연결한다. 현재 backend 계약이 있는 Volume level/mute만 실제로 적용하고,
brightness와 blue light는 platform backend가 확정될 때까지 `None`/Unavailable로 표시한다.
