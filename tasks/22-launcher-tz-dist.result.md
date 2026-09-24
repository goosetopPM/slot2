# Task 22 결과 — 런처 타임존 주입 + dist-device 재빌드

실행 2026-09-24. 담당: 서브에이전트(워커).

## 1. (A)(B)(C) 판정

| 항목 | 결과 |
|---|---|
| (A) 런처에 `SLOT2_UTC_OFFSET_MIN=540` 주입 | **실패 (주입할 런처가 존재하지 않음 — 명세의 탈출 조항대로 실패로 남김)** |
| (B) `docs/MILESTONES.md` M5 에 "시간대 설정" 항목 기록 | **성공** |
| (C) `powershell -File build/dist-device.ps1` 재빌드 | **성공 (종료 코드 0)** |

## 2. 검증 명령 3개의 마지막 결과 줄

```
cargo test --workspace
```
→ 종료 코드 0. `test result:` 줄 **52개**, 합계 **369 passed / 0 failed**. 기준선(52 바이너리 / 369 passed / 0 failed)과 정확히 일치, 감소 없음.
마지막 줄: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (Doc-tests slot2_ui)

```
cargo clippy --workspace --all-targets -- -D warnings
```
→ 종료 코드 **0**.
마지막 줄: `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.50s`

```
powershell -File build/dist-device.ps1
```
→ 종료 코드 **0**.
마지막 줄: `==> done`

## 3. (A) 실패 원인 — 한 줄

BaseOS 가 `exec /lib/ld-linux-aarch64.so.1 System/frontend` 로 **동적 로더에 ELF 를 직접 먹여** 띄우고 **환경변수를 일절 주지 않으므로**, 저장소·카드 어디에도 env 를 export 할 쉘 런처가 없고 새로 만들 수도 없다.

### 근거 (조사 내역)

1. **저장소 전수 조사**: 추적 중인 `.sh` 파일은 `cores/common.sh`, `cores/{fceumm,genesis_plus_gx,mgba,snes9x}/build.sh` 5개뿐 — 전부 **호스트에서 libretro 코어를 크로스빌드하는 스크립트**이고 기기에서 도는 것이 아니다. `.desktop`·`launch.sh`·래퍼 생성 코드 없음.
2. **`build/dist-device.ps1` 이 카드에 싣는 것**: `System/frontend`(aarch64 ELF) · `System/cores/*.so` · `System/Fonts/*` · `System/licenses/*` · `System/VERSION.txt`. **실행 스크립트를 생성하지 않는다.** 실제 산출물 트리에도 `.sh` 가 0개다.
3. **`docs/DESIGN.md:37` (BaseOS 계약, `overlay/usr/sbin/frontend-session` 기준)**:
   > 실행 방식 | `cd $SD; exec /lib/ld-linux-aarch64.so.1 System/frontend`, 환경변수 없음, exec 비트 불필요 | `SLOT_ROOT` 같은 env 의존 금지

   `ld-linux-aarch64.so.1` 에 인자로 넘기므로 `System/frontend` 를 쉘 스크립트로 바꿔치기하는 우회는 **동작하지 않는다**(로더는 셔뱅을 해석하지 않고 ELF 가 아니면 죽는다). 실제로 이번 빌드가 찍은 파일 타입도 `ELF 64-bit LSB pie executable, ARM aarch64 ... interpreter /lib/ld-linux-aarch64.so.1` 이다.
4. **`docs/DESIGN.md:36` 의 `System/launch_frontend.sh`** 는 BaseOS 의 실행 우선순위에서 `System/frontend` **다음**이다. 카드에 `System/frontend` 가 있는 한 `launch_frontend.sh` 는 **영원히 실행되지 않는** 죽은 파일이 된다. 이걸 살리려면 바이너리를 다른 이름으로 옮기고 카드 레이아웃(`docs/DESIGN.md:281`, D-01 계약)을 바꿔야 하는데, 이는 태스크 범위를 넘는 설계 변경이라 **추측으로 새 파일을 만들지 말라**는 명세 지시(태스크 파일 33~34줄)에 따라 손대지 않았다.

### 대안 (승인 필요 — 이번에 구현하지 않음)

- **(a) 카드 레이아웃 변경**: 바이너리를 `System/slot2-bin` 으로 옮기고 `System/launch_frontend.sh` 를 런처로 세운다. D-01/DESIGN §2·§9 를 함께 고쳐야 하고, "`System/frontend` 배치"라는 명시적 결정을 뒤집는 것이라 사용자 승인이 필요하다. BaseOS 가 `launch_frontend.sh` 를 **셸로** 띄우는지(`sh` 경유인지 exec 비트 필요인지) 실기/소스 재확인도 선행돼야 한다.
- **(b) 코드 기본값**: `crates/slot2-platform/src/clock.rs` 의 `utc_offset_min()` 이 env 부재 시 `0` 대신 540 을 쓰게 한다. 런처 없이도 즉효지만, "카드는 UTC 를 들고 화면은 로컬을 보여준다 / 그 차이는 플레이어가 정하는 숫자"라는 모듈 주석의 설계와 충돌하고 KST 를 코드에 박는 셈이라 **권장하지 않는다**.
- **(c) 현상 유지**: M5 시간대 설정 화면이 이 문제의 정규 해법이다. (B)로 그렇게 기록했다.

## 4. 생성/수정한 파일

- `C:\SLOT2\docs\MILESTONES.md` — M5 "작업" 목록에 1줄 추가 (언어팩 항목과 번역 기여 가이드 사이):
  `- [ ] 시간대 설정 — 설정 메뉴에서 UTC 오프셋을 고르고, 그 값이 \`SLOT2_UTC_OFFSET_MIN\`을 대체한다. BaseOS는 환경변수 없이 \`System/frontend\`를 exec 하므로(DESIGN §2) 이 화면이 생기기 전까지 실기 시계는 UTC다`
  - 명세의 원안 마지막 문장("런처의 540 기본값은 그때 걷어낸다")은 **그런 기본값이 존재하지 않으므로 그대로 쓰지 않았다.** 대신 왜 없는지(BaseOS 가 env 를 주지 않음)를 같은 자리에 적었다. 없는 것을 있다고 적는 문서는 다음 사람을 또 속인다.
  - M3 절의 기존 문장(`docs/MILESTONES.md:82`)이 이미 같은 사실을 한 번 적고 있어 중복 서술이 되지 않도록 M5 항목은 "무엇을 만들 것인가"에 초점을 뒀다.
- `C:\SLOT2\dist-device\**` — 재빌드 산출물(gitignore 대상, 커밋 없음).
- `C:\SLOT2\tasks\22-launcher-tz-dist.result.md` — 이 보고서.

**소스 코드(`crates/**`)는 한 줄도 고치지 않았다.** 커밋/스테이징도 하지 않았다 (`git add`/`commit`/`push` 미실행).

## 5. (C) 산출물

- 경로: `C:\SLOT2\dist-device\` (내용물을 카드 루트로 복사)
- `System/VERSION.txt`: `SLOT2 0.1.0 (610a815)`
- 빌드에 들어간 커밋: **`610a815`** (빌드 시점 `HEAD`). 재빌드 전 이미지는 `38ccdc8` 기준이었으므로 Task 16~21(선반 UI·스킨·HUD)이 이제 포함된다.
- `System/frontend` 타임스탬프: **2026-09-24 17:57:33**, 크기 2,765,736 바이트
- BuildID: `c45a14f62cbe367eb6380922aeb3841643f4a463`, `ELF 64-bit LSB pie executable, ARM aarch64, for GNU/Linux 3.7.0, not stripped`
- 코어 4종 동봉: `fceumm`, `genesis_plus_gx`, `mgba`, `snes9x`
- 크로스 빌드: `device` 프로필 최적화, 1m 10s

## 6. 실기 확인 필요 항목 (사용자만 수행 — 6-4)

1. **시계는 UTC 로 나오는 것이 정상이다.** (A)가 실패했으므로 이번 이미지에서도 HUD 시계는 KST 대비 9시간 늦게 보인다. **버그로 오진하지 말 것.** 실기에서 KST 로 보고 싶으면 dropbear 로 들어가 수동으로 env 를 세워 띄우는 수밖에 없다(임시 확인용, 카드에는 남지 않음).
2. 새 카드 이미지(`610a815`)로 Task 16~21 결과물 확인: 선반 UI 이동/전환, 스킨 적용, **배터리·시계 HUD 가 패널 모서리에 고정**되는지, 저전력 시 잉크 색 변화, 충전 중 볼트 표시.
3. `System/frontend` 가 실제로 부팅되는지(ELF 교체 후 respawn 정상), `/tmp/frontend.log` 에 패닉 없는지.
4. 대안 (a)를 택할 경우: BaseOS 가 `System/launch_frontend.sh` 를 **어떤 방식으로** 띄우는지(exec 비트 필요 여부, `sh` 경유 여부) 실기 확인이 선행돼야 한다.

## 7. 계약이 틀려 보이는 줄

- **`tasks/22-launcher-tz-dist.md:23-29`** — "(A) 런처에 `SLOT2_UTC_OFFSET_MIN=540` 주입 … 그 런처에서 slot2 바이너리를 exec 하기 직전에 … `: "${SLOT2_UTC_OFFSET_MIN:=540}"` + `export` 형태로". **전제가 틀렸다.** 기기 실행 경로에 쉘이 없다. `docs/DESIGN.md:37` 이 "환경변수 없음"을 계약으로 못박고 있고, 로더 직접 exec 이라 쉘을 끼울 자리 자체가 없다. 명세 33~34줄의 탈출 조항이 정확히 이 경우를 예상하고 있어 그대로 따랐다.
- **`tasks/22-launcher-tz-dist.md:41`** — "런처의 540 기본값은 그때 걷어낸다". 걷어낼 기본값이 생기지 않으므로 (B)에서 이 문장을 옮겨 적지 않았다. 4절 참조.
- **`tasks/22-launcher-tz-dist.md:16`** — "`dist-device` 카드 이미지가 `38ccdc8` 기준"이라고 적혀 있는데, 재빌드 직전 기존 `VERSION.txt` 는 이미 확인하지 못한 채 덮어썼다. 다만 `dist/` 에 남아 있는 릴리즈 zip 은 `slot2-0.1.0-0d3dcd3.zip` 이라 `38ccdc8` 과 다르다 — 어느 쪽이든 Task 16~21 이전이라 재빌드 필요성 자체는 유효했다.
