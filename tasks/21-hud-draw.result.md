# Task 21 — HUD draw: 검증 결과

대상 커밋 `2f4ccd2` (M3: the corner the machine talks from). HEAD 는 `0a690df`.
검증만 수행했습니다. 소스 수정·커밋·bai-gjc 실행 없음.

## cargo test

```
cargo test --workspace --features slot2-input/host --no-fail-fast
```

- 종료 코드: **0**
- 테스트 바이너리: **52개** (`test result:` 줄 기준)
- 합계: **369 passed / 0 failed / 0 ignored**
- 실패한 바이너리 없음 — 따라서 실패 `test result:` 줄도 없습니다.
- 가장 오래 걸린 것: 오디오 바이너리 1개가 `finished in 233.23s`, 그 다음이 10.19s.
  전체 실행은 약 5분.

## cargo clippy

```
cargo clippy --workspace --all-targets --features slot2-input/host -- -D warnings
```

- 종료 코드: **0**
- warning / error 줄 0개. 린트 없음.

## 작업 트리

`git status --short` 결과는 `?? .claude/` 한 줄뿐입니다. 추적 중인 파일에 변경 없음 —
지시받은 대로 깨끗합니다.

## 실기(RG SP)에서 확인해야 할 것

CI 에서는 원리상 확인이 불가능한 항목입니다. 여기서는 손대지 않았습니다.

1. **`/sys/class/power_supply` 실제 워크** — `crates/slot2-platform/src/battery.rs:74`
   `Gauge::probe()` 는 테스트에서 합성 sysfs 루트로만 검증됩니다. 실기에는
   `axp2202-battery` 와 `axp2202-usb` 두 항목이 같은 클래스에 있고, `type`=`Battery`
   이면서 `capacity` 가 있는 쪽만 골라야 합니다. 이름 정렬상 `-battery` 가 먼저 오지만,
   실제 노드 이름·`type` 파일 내용은 기기에서만 확인됩니다.
2. **`Charge::Unknown` 의 실제 빈도** — 같은 파일 주석대로 이 PMIC 는 `current_now` 가
   비어 나옵니다. `status` 속성이 실기에서 무엇을 내놓는지, 그리고 그때 HUD 가
   의도대로 보이는지 확인이 필요합니다.
3. **충전 볼트** — 볼트는 charging 일 때만 그려집니다. 케이블을 실제로 꽂았다 빼면서
   캡슐이 움직이지 않는지(예약 슬롯이 제 역할을 하는지) 눈으로 봐야 합니다.
4. **LOW_INK 경고색** — 배터리가 낮고 **방전 중**일 때만 경고색입니다. 실제로 배터리를
   낮은 상태까지 쓰면서 확인해야 합니다.
5. **시계 오프셋** — `SLOT2_UTC_OFFSET_MIN` 을 세팅하는 실행 스크립트가 저장소에
   없습니다(`crates/slot2-platform/src/clock.rs:20`). 기기에서 이 변수가 없으면 시계가
   UTC 로 나와 KST 기준 9시간 어긋납니다. 런처에 `540` 을 넣어야 하는지 확인 필요.
6. **죽은 RTC** — `clock::is_set` 은 2020-01-01 이전을 "안 맞춰짐"으로 봅니다
   (`clock.rs:84`). 콜드 부팅 직후 실기 RTC 가 그 아래로 나오는지, 그때 좌상단이
   비는지 확인.
7. **패널 위 배치** — HUD_MARGIN·28px 밴드·중앙 정렬은 픽셀 단언으로만 검증됩니다.
   AGS-102 화면에서 실제로 모서리에 잘리지 않고 읽히는지는 봐야 압니다.

## 판정

**성공.** 테스트·클리피 모두 통과, 트리 깨끗. 위 7개는 실패가 아니라 기기에서만
닫을 수 있는 확인 항목입니다.
