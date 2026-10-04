# Task 100 — 6코어 배포 게이트와 CI 일치 (worker-result)

**상태: 성공. 누적 호출 1/2.**

완료 기준 9개 명령 전부 종료 코드 0. `cargo test --workspace` 973 passed / 0 failed / 0 ignored
(Task99의 971 + 이번에 추가한 registry 테스트 2개, 감소 없음). clippy warning 0. `-DeviceOnly`는
여섯 이름 모두 current/skip(네트워크 없음). full dist 마지막 줄 `==> done`, `dist-device/System/cores`
정확히 6개·대응 `.so.meta` 정확히 6개, zip도 같은 여섯. 안전한 staging negative check 8종 모두 실패
경로 확인 + 기존 tree 보존. 커밋·push·release·ADB·Samba·SD 카드·공용 설정 접근 없음. 위임 없이 직접 수행.

---

## 1. 단일 배포 manifest와 registry equality

신규 `cores/required.txt` (UTF-8, **BOM 없음, LF**, 49 bytes, 빈 줄·주석·중복 없음):

```text
mgba
gambatte
gpsp
fceumm
snes9x
genesis_plus_gx
```

- `build/cores.ps1`의 기본 목록, `build/dist-device.ps1`이 복사·검증하는 목록, CI 두 job이 순서대로
  읽는 목록이 모두 이 파일이다. PowerShell·workflow 어디에도 여섯 이름 배열을 복제하지 않았다
  (`build/core-manifest.ps1`이 유일한 reader).
- runtime 단일 목록은 계속 `Core::ALL`(slot2-retro)이며, 신규 테스트 2개가 계약을 봉인한다.
  - `the_cores_a_card_ships_are_one_list` — manifest를 파일에서 읽어 여섯 이름·순서를 그대로 단언하고,
    `Core::ALL::base_name()`에서 `_libretro`를 떼어낸 목록과 **exact equality**를 본다.
  - `every_manifest_row_names_a_core_this_repository_can_build` — 각 row가 `[a-z0-9_]+`, 빈 값·중복
    없음, `cores/<name>/build.sh` 존재, `commit`이 40자리 lowercase hex.
  - YAML·PowerShell 문자열을 복제해 비교하지 않는다: registry와 manifest만 본다.
- 검증: `cargo test -p slot2-retro --test registry` → 21 passed (기존 19 + 신규 2).

## 2. `build/cores.ps1`

- `param([string[]]$Core)` 기본값 제거. `-Core`가 없으면 `Get-CoreManifest -Root $root`로 manifest를
  읽는다. `$root`는 `Split-Path -Parent $PSScriptRoot`로 스크립트 위치 기준이라 호출자 cwd에 기대지
  않는다(파일 경로도 루트 절대경로 사용).
- **명시적 `-Core`는 그대로** 동작한다(`-Core mgba`는 rows 검증만 하고 그 코어만 빌드/확인). 컨테이너
  안 경로(`cores/$name/build.sh`, `vendor/<name>_libretro.so`)는 그대로, 호스트 검사만 절대경로.
- manifest 검증은 loop 이전에 수행하므로 빈 값·중복·안전하지 않은 이름·`build.sh`/`commit` 누락은
  core build·docker·네트워크 이전에 실패한다(§5 negative check 참조). `-Force`, `-HostOnly`,
  `-DeviceOnly`, stamp 비교, host buildbot URL·산출물 이름은 불변.
- Windows에서 `-match`/`-notmatch`가 대소문자 무시라 `Mgba`가 통과하던 문제를 구현 중 발견해
  `-cnotmatch` + `[StringComparer]::Ordinal` HashSet으로 고쳤다(대소문자만 다른 이름이 통과하면
  대소문자 무시 파일시스템에서 `vendor/Mgba_libretro.so`가 만들어지고 frontend가 이름을 못 찾는다).

## 3. gpSP native/device make args와 stamp

`cores/gpsp/build.sh`: `triple="${CROSS_TRIPLE-aarch64-linux-gnu}"` (common.sh와 동일한 기본값)로 분기.

| CROSS_TRIPLE | CORE_MAKE_ARGS | 근거 |
|---|---|---|
| 미설정 (device) | `CPU_ARCH=arm64 HAVE_DYNAREC=1 MMAP_JIT_CACHE=1` | 기존과 완전 동일 |
| `aarch64*` (명시) | 위 세 인자 그대로 | 명시적 aarch64 triple도 기존 동작 |
| `` (빈 값, CI native) | `` (빈 값) | 컨테이너/러너의 makefile unix autodetection |
| 그 외 nonempty | — | `exit 2`로 build·fetch 이전에 명확히 실패 |

컨테이너에서 직접 확인한 stamp (docker `sh cores/gpsp/build.sh stamp`):

- device: `make_args=CPU_ARCH=arm64 HAVE_DYNAREC=1 MMAP_JIT_CACHE=1`, `triple=aarch64-linux-gnu`,
  `repo/target/make_dir/makefile` 포함 — **`vendor/gpsp_libretro.so.meta`와 byte 동일**(335 bytes).
  그래서 `-DeviceOnly`가 gpsp를 current로 skip하고 재빌드하지 않는다.
- native(`-e CROSS_TRIPLE=`): `make_args=`(빈 값), `triple=` — native에 실제로 넘긴 인자를 기록한다.
- `-e CROSS_TRIPLE=arm-linux-gnueabihf`: exit 2,
  `gpsp: don't know how to build for 'arm-linux-gnueabihf'; only aarch64 cross toolchains and a native build are set up`.

## 4. `build/dist-device.ps1`

- 신규 `build/core-manifest.ps1`(함수만, dot-source, 읽기만 함)를 `cores.ps1`과 함께 사용.
  - `Get-CoreManifest` — 목록 읽기 + 검증
  - `Assert-CoreDeviceFiles` — 여섯 `.so`와 여섯 `.so.meta`가 regular file이고 길이 > 0인지, 없거나 빈
    파일의 **이름과 경로**를 모아 throw
  - `Assert-CoreTree` — 조립된 `System/cores`가 manifest의 `<name>_libretro.so` 여섯과 정확히 같은
    집합인지, `System/licenses`에 대응 `.meta` 여섯이 있고 다른 `*_libretro.so.meta`가 없는지,
    각 파일이 비어 있지 않은지
- 흐름: manifest 검증 → (일반 실행 시) cross image → frontend 빌드 → `cores.ps1 -DeviceOnly` →
  `Test-Path $bin` → **`Assert-CoreDeviceFiles`(preflight)** → 기존 `dist-device` 삭제 → 조립 →
  **`Assert-CoreTree`(postflight)** → VERSION.txt → 목록 출력 → `-Zip` → `-Adb` → `==> done`.
  모든 입력 검증이 `Remove-Item -Recurse -Force $out`보다 먼저라 실패가 직전 정상 배포물을 지우지 않는다.
- 복사는 glob이 아니라 manifest 순서의 여섯 `.so` + 대응 `.meta`만. `Get-ChildItem vendor\*_libretro.so`
  glob과 "코어 0개면 경고 후 성공" 분기는 제거했다(repo의 Windows `.dll`·예상 밖 `.so`는 배포되지 않음).
- `-Adb`: `adb shell 'mkdir -p /mnt/sdcard/System/Fonts /mnt/sdcard/System/licenses /mnt/sdcard/System/cores'`
  뒤 frontend·Fonts·licenses·**cores**·VERSION을 push. 원격 cores 디렉터리를 먼저 만든다. ADB 이외의
  device 경로·reboot 의미는 그대로이며 이번 작업에서 실행하지 않았다.

## 5. GitHub CI (`.github/workflows/ci.yml`)

- host `check`: `for c in $(tr -d '\r' < cores/required.txt)`로 여섯 코어를 `CROSS_TRIPLE=` native `.so`로
  빌드(gpSP·Gambatte 포함) → `cores present` 스텝이 이름별로 `.so`와 `.so.meta`의 존재·nonempty(`-s`)를
  검증하고 없으면 `::error::` + nonzero.
- device: 같은 manifest로 aarch64 빌드(동일 `docker run ... sh cores/$c/build.sh build ...`),
  cache hit에서도 항상 도는 `core files present` 스텝으로 여섯 `.so`/`.meta`를 이름별 검증.
- card assembly: glob·`-eq 4` count 제거 → manifest 여섯 `.so`/`.meta`만 복사 → `System/cores`와
  `System/licenses`(메타만 필터)의 exact set을 `diff`로 검증. tree 구조(Fonts/cores/frontend/licenses
  +VERSION.txt)는 `dist-device.ps1`과 같다. release tag·zip upload·license/source archive는 손대지 않았다.
- cache key는 계속 `hashFiles('cores/**')`라 manifest 변경도 캐시를 무효화한다. fmt, host/device clippy,
  workspace test, `no core was skipped` 스텝은 그대로다.
- 로컬 검증: 두 빌드 스텝을 제외한 CI 스텝 본문을 추출해 이 호스트의 `sh`로 그대로 실행 →
  `cores present` exit 0, `core files present` exit 0, `assemble card tree` exit 0(출력 트리는 임시
  디렉터리로 돌려 실제 `dist-device`를 건드리지 않았고, 그 결과 여섯 core + 여섯 meta + fonts가
  정확히 만들어졌다). 두 빌드 스텝은 소스 fetch/컴파일이 필요해 실행하지 않았고, 그 안의 docker
  호출·`CROSS_TRIPLE=` 형태는 §3과 §6에서 개별 검증했다.

## 6. 완료 기준 명령 (최종 검증, 18:22:24–18:34:17)

| # | 명령 | 종료 | 마지막 결과 줄 | passed/failed/ignored |
|---|---|---|---|---|
| 1 | `cargo fmt --all -- --check` | 0 | (출력 없음) | — |
| 2 | `cargo test -p slot2-retro --test registry` | 0 | `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s` | 21/0/0 |
| 3 | `powershell -NoProfile -ExecutionPolicy Bypass -File build/cores.ps1 -DeviceOnly` | 0 | `==> genesis_plus_gx device core is current (vendor/genesis_plus_gx_libretro.so)` (여섯 이름 모두 current, docker/네트워크 사용 없음) | — |
| 4 | `powershell -NoProfile -ExecutionPolicy Bypass -File build/dist-device.ps1 -NoBuild -Zip` | 0 | `==> done` (`dist\slot2-0.1.0-a8cb4af.zip`) | — |
| 5 | `cargo test --workspace` | 0 | 결과 줄 100개. 첫 줄 `ok. 33 passed; … in 0.36s`, 마지막 줄(doc-test) `ok. 0 passed; 0 failed; 0 ignored; … in 0.00s` | **973/0/0** |
| 6 | `cargo check -p slot2 --no-default-features --features device` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.77s`` | — |
| 7 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.06s`` (warning 0줄) | — |
| 8 | `powershell -NoProfile -ExecutionPolicy Bypass -File build/dist-device.ps1` | 0 | `==> done` (앞서 `==> {mgba,gambatte,gpsp,fceumm,snes9x,genesis_plus_gx} device core is current`, `frontend` 재빌드 1m30s) | — |
| 9 | `git diff --check` | 0 | (whitespace 진단 없음. stderr에 `CRLF will be replaced by LF…` 경고 13줄 — 이 checkout의 `core.autocrlf=true` 상태 때문이며 오류 아님) | — |

- `cargo test --workspace` 총 passed 973은 Task99의 971 + 신규 registry 테스트 2개다(감소·삭제 없음).
  실행된 모든 test binary failed 0, 예상 밖 ignored 0.
- zip과 `dist-device`의 core/meta 목록(둘 다 정확히 여섯, 내용 byte 동일):
  `System/cores/{fceumm,gambatte,genesis_plus_gx,gpsp,mgba,snes9x}_libretro.so`,
  `System/licenses/{같은 여섯}_libretro.so.meta`. `System/licenses/fonts`에는 폰트 라이선스 2개.
  zip은 18 entries = frontend + VERSION + fonts 2 + font licences 2 + cores 6 + metas 6.
  `dist-device/System/frontend` 3,061,968 bytes, `VERSION.txt` = `SLOT2 0.1.0 (a8cb4af)`.
  `dist/`에는 이번 zip 외에 이전(09-22, sha 0d3dcd3) zip이 남아 있다 — 그 zip에는 **core가 하나도
  없다**(7 entries, `System/cores` 없음). 기존 파일이라 지우지 않았다.

## 7. 안전한 negative check (실제 vendor/dist-device 파괴 없음)

staging: `%TEMP%\slot2-core-gate\`에 `build/{core-manifest,cores,dist-device}.ps1` 복사본,
`cores/required.txt` + `cores/<여섯>/{build.sh,commit}`, 더미 `vendor/` 여섯 `.so`+`.meta`,
`dist-device/marker.txt`(정상 tree 대표), 더미 `target-device/.../slot2`.

| 시나리오 | 명령 | 종료 | 결과 |
|---|---|---|---|
| `.so` 없음 (gpsp) | staged `dist-device.ps1 -NoBuild` | 1 | `the device cores the manifest asks for are not in vendor/: …\vendor\gpsp_libretro.so (missing)` + `run build/cores.ps1 -DeviceOnly first` |
| `.meta` 없음 (snes9x) | 동일 | 1 | `…\vendor\snes9x_libretro.so.meta (missing)` (이름·경로 지목) |
| `.so` 0바이트 (mgba) | 동일 | 1 | `…\vendor\mgba_libretro.so (empty)` |
| 기존 tree 보존 | 위 세 경우 각각 | — | `dist-device/marker.txt`가 **세 경우 모두 그대로 남음**(삭제 이전에 검증) |
| 중복 row | staged `cores.ps1 -DeviceOnly` | 1 | `cores\required.txt lists 'gpsp' twice`, docker 미실행 |
| 대문자 이름(`Mgba`) | 동일 | 1 | `has 'Mgba', which is not a core name`, docker 미실행 |
| 이름에 경로(`../etc/passwd`) | 동일 | 1 | `has '../etc/passwd', which is not a core name`, docker 미실행 |
| 빈 줄 | 동일 | 1 | `has '', which is not a core name`, docker 미실행 |
| `build.sh` 없음(`nosuchcore`) | 동일 | 1 | `names nosuchcore, which has no …\cores\nosuchcore\build.sh`, docker 미실행 |
| 지원하지 않는 triple | `CROSS_TRIPLE=arm-linux-gnueabihf` stamp | 2 | `only aarch64 cross toolchains and a native build are set up` (build/fetch 이전) |

실제 `vendor/`·`dist-device/`는 건드리지 않았고, **정상 tree도 보존**됐다(위 세 경우 실제 tree는
손대지 않았고, 마지막 배포 실행이 여섯 core/meta를 정상 재생성했다).

## 8. 생성·수정 파일

- 신규: `cores/required.txt`, `build/core-manifest.ps1`, `tasks/100-six-core-distribution-gate.worker-result.md`.
- 수정: `cores/gpsp/build.sh`, `build/cores.ps1`, `build/dist-device.ps1`, `.github/workflows/ci.yml`,
  `crates/slot2-retro/tests/registry.rs`(+2 테스트, 기존 19개 불변), `docs/DESIGN.md`(§6 코어 빌드 문단 +
  §9 카드 레이아웃 cores/licenses 두 줄), `docs/MILESTONES.md`(M7 첫 항목에 진행 메모, `[ ]` 유지).
- 산출물(비추적): `vendor/{fceumm,snes9x,genesis_plus_gx}_libretro.{so,so.meta}` 재생성(§아래),
  `dist-device/` 재조립, `dist/slot2-0.1.0-a8cb4af.zip`.
- **다른 파일은 수정하지 않았다.** core commit/patch, registry runtime 후보·기본값, README/LICENSE,
  release.yml, vendor 산출물 커밋 없음.
- **최종 검증 뒤 code/test 변경 없음**: 마지막 소스 변경 18:20:54(ci.yml), 검증 시작 18:22:24.
  검증 시작 이후 바뀐 것은 검증 자체의 산출물(`dist/slot2-0.1.0-a8cb4af.zip` 18:22:43,
  `dist-device/System/{frontend,VERSION.txt}` 18:34), 테스트가 다시 써낸 비추적
  `assets/overlays/GB/720x720.png`(생성기 테스트, Task99 보고서에도 기록된 기존 동작), 이 보고서뿐이다.
  추가 확인: 검증 뒤 `build/cores.ps1 -Core mgba -DeviceOnly`(명시적 `-Core`) exit 0, `vendor/mgba_libretro.so`
  는 그대로 current(파일 write 없음). 새 파일 줄바꿈은 전부 LF·BOM 없음(manifest 49 bytes 확인).

## 9. 계약과 다르게 진행된 부분 / 주의

1. **첫 `-DeviceOnly` 실행에서 세 코어(fceumm·snes9x·genesis_plus_gx)가 재빌드됐다.** 계약은 "stamp가
   current면 skip"을 기대했지만, 이 세 개의 기존 `.so.meta`는 **Task57이 확장하기 이전 `cores/common.sh`
   형식**(repo/target/make_dir/makefile/make_args 줄 없음, 198/189/198 bytes)이라 현재 stamp와 같아질 수
   없었다. 재빌드 소스는 로컬 체크아웃(`target-device/cores/<name>/src`)에서 나왔고 **네트워크 fetch는
   없었다**: `git fetch`가 일어나면 갱신될 `.git/FETCH_HEAD`의 mtime이 09-23(오늘 아님)이다. 결과 `.so`는
   이전과 같은 크기(4447528 / 2722056 / 12576832)의 aarch64 ELF(machine 183)이고 meta는 새 형식으로
   다시 쓰였다. 그 뒤 모든 실행(§6의 3·8번)에서 여섯 이름이 모두 current로 출력된다. 즉 이번 변경이
   만든 회귀가 아니라 Task57이 남긴 stamp 불일치가 이번에 처음 해소된 것이다.
2. **CI의 로컬 실행 범위**: GitHub Actions 자체는 이 환경에서 돌릴 수 없고(네트워크 금지), 두 빌드
   스텝(host native 6코어, device aarch64 6코어)은 소스 fetch가 필요해 실행하지 않았다. 대신 그 스텝들이
   공유하는 loop 형태를 나머지 세 스텝 본문으로 실행 검증했고, gpSP native 분기는 컨테이너에서
   `CROSS_TRIPLE=` stamp로 확인했다. 이 호스트의 셸에는 `read` 빌트인이 없어(`read: i/o error` os error 87)
   `while IFS= read` 형태를 로컬 검증할 수 없었기에, CI loop를 `for c in $(tr -d '\r' < cores/required.txt)`
   로 썼다(ubuntu bash에서 동일 동작, `\r` 제거 포함). YAML 자체는 parser가 없어(pyyaml 미설치·네트워크
   금지) 기계 검증하지 못했고, 들여쓰기·블록 스칼라 구조를 눈으로 확인 + 본문 실행으로 대체했다.
3. `build/core-manifest.ps1`의 이름 검증만 계약이 "빈 값·중복·안전하지 않은 이름 거부"라고 적었고
   commit 형식(40자리 hex) 검사는 계약이 자동 테스트 항목으로만 요구했기에, hex 검사는 Rust 테스트에만
   둔다(PowerShell은 존재 여부만 본다). 중복 판정은 대소문자 구분(Ordinal)으로 구현했다.
4. `Assert-CoreTree`는 `System/licenses`에서 `*_libretro.so.meta`만 집계한다(같은 디렉터리에 폰트
   라이선스가 함께 들어가므로). 계약의 "대응 meta 정확히 6개"를 그 기준으로 해석했다.
5. 이 checkout은 `core.autocrlf=true`라 `git diff` 기준으로 다수 파일이 modified로 보이고
   `docs/DESIGN.md`에는 기존 CRLF가 섞여 있다(내가 추가한 줄은 LF). `git diff --check`는 통과한다.

## 10. 남은 것 (다음 M7 태스크)

- `release.yml`(태그 → zip + 라이선스/소스 아카이브), README·LICENSE·third-party notice, 이슈 템플릿,
  원본 slot 카드 마이그레이션 가이드. 실기 카드 검증(6코어로 7개 선반 부팅)은 사용자 몫이다.
