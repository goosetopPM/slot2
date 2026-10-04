# Task 104 worker result — CI 라이선스 입력 준비와 cache-hit 경로 복구

## 1. 판정

**성공 (누적 시도 1/2).** Task 103 최종 리뷰가 지적한 CI 전용 결함 두 가지를 `.github/workflows/ci.yml`의
`device` job 안에서만 고쳤습니다. 제품 코드·packager·validator·Cargo 입력은 손대지 않았습니다.

1. **호스트 Cargo source 준비**: offline Rust notice packager 앞에 호스트 toolchain + Rust cache +
   명시적 `cargo fetch --locked` 를 넣어, 새 runner에서도 packager가 읽을 crate source가 존재하게 했습니다.
2. **device-core cache-hit 경로**: cache path에 `target-device/cores`를 추가하고 key를 versioning했으며,
   cache hit 여부와 무관하게 `cores/required.txt` 루프로 바이너리·`.meta`·pinned checkout 존재/핀 포함/
   HEAD 일치를 검사합니다.

호스팅 CI는 **실행하지 않았습니다.** 실제 GitHub Actions 실행은 사용자 수용 단계로 남습니다.

## 2. 변경한 YAML (최종 순서, `device` job)

파일: `.github/workflows/ci.yml` (279줄, mtime 2026-10-01 16:52:21). 신규/변경 step만 표시:

| 순서 | 줄 | step | 내용 |
|---|---|---|---|
| 1 | 84 | `actions/checkout@v4` | 그대로 |
| **2** | **92** | `dtolnay/rust-toolchain@stable` **(신규)** | `with: targets: aarch64-unknown-linux-gnu` — `check` job과 같은 action 스타일, `rust-toolchain.toml`의 `targets`와 동일 |
| **3** | **95** | `Swatinem/rust-cache@v2` **(신규)** | `check` job과 같은 Rust cache 스타일. `cache-targets: false`로 registry·git source만 담는다 |
| **4** | **100** | `device target on the host` **(신규)** | `rustup target list --installed \| grep -qx aarch64-unknown-linux-gnu` |
| 5 | 103 | `build cross image` | `docker build -t slot2-cross -f build/cross.Dockerfile build` — 그대로(Docker 유지) |
| **6** | **113** | `cache device cores` **(변경)** | `path: vendor` + `target-device/cores`, `key: cores-aarch64-v2-${{ hashFiles('cores/**') }}` |
| 7 | 125 | `build device cores` | 그대로(Docker) |
| **8** | **143** | `core files and checkouts present` **(변경)** | 기존 `core files present`를 확장: 바이너리·`.meta` + pinned checkout 검증 |
| 9 | 174 | `cross build (profile device)` | Docker 유지, target/profile/package/features/target-dir 플래그 동일 |
| 10 | 183 | `package core sources` | 그대로 |
| **11** | **192** | `fetch Cargo sources for the notice packager` **(신규)** | `cargo fetch --locked --target aarch64-unknown-linux-gnu` + `cargo fetch --locked` |
| 12 | 201 | `package rust notices` | 명령 그대로 (offline packager) |
| 13 | 208 | `assemble card tree` | 그대로 (여섯 core·source·Rust SBOM 검증 + artifact) |
| 14 | 275 | `actions/upload-artifact@v4` | 그대로 |

호스트 준비(2~4)가 packager(12)보다 앞서고, fetch(11)가 packager **바로 앞**이라는 점이 YAML 순서로
기계적으로 드러납니다(§6에서 파서로 단언).

## 3. 호스트 Cargo source 준비

- toolchain: `dtolnay/rust-toolchain@stable` + `targets: aarch64-unknown-linux-gnu` — 이 저장소의
  `rust-toolchain.toml`(`channel = "stable"`, `targets = ["aarch64-unknown-linux-gnu"]`)과 같은 채널·타깃,
  `check` job이 이미 쓰는 action 스타일 그대로.
- cache: `Swatinem/rust-cache@v2`(기존 스타일). `cache-targets: false`인 이유는 이 job이 만드는
  `target/core-sources`·`target/rust-notices`·`dist-device` 같은 산출물을 cache에 싣지 않기 위해서입니다.
- fetch: `cargo fetch --locked --target aarch64-unknown-linux-gnu` 로 device 타깃 의존성을 받고,
  이어서 `cargo fetch --locked` 한 번 더. 두 번째가 필요한 이유는 packager의 `cargo metadata`가
  **workspace 전체**를 해석하기 때문입니다(호스트 전용 feature를 켠 멤버가 끌어오는 winit/X11/wayland
  계열 source는 device 타깃 하나로는 받아지지 않습니다). `--locked`라서 `Cargo.lock`이 입력 그대로이고
  의존성이 흔들리면 fetch 단계에서 실패합니다.
- packager가 이 source를 보는 이유: packager는 수정하지 않았고 그대로 `--offline`이며, Cargo의 기본
  registry/git cache(`~/.cargo/registry`, `~/.cargo/git`)를 읽습니다. 위 fetch가 그 cache를 채우므로
  같은 프로세스 환경(러너 호스트)에서 곧바로 읽힙니다. **Docker 컨테이너의 Cargo home은 복사·마운트하지
  않았습니다** — 이미지 소유의 불투명한 디렉터리이고, crate 목록도 하드코딩하지 않았습니다.

## 4. device-core cache

| 항목 | 이전 | 이후 |
|---|---|---|
| `path` | `vendor` | `vendor` + `target-device/cores` |
| `key` | `cores-aarch64-${{ hashFiles('cores/**') }}` | `cores-aarch64-v2-${{ hashFiles('cores/**') }}` |

- `target-device/cores`를 넣은 이유: `build/core-manifest.ps1`의 `Get-CoreCheckout`이
  `target-device/cores/<core>` 아래에서 `.git`을 가진 디렉터리를 찾고, `package-core-sources.ps1`이 그
  checkout에서 `git archive`/`git cat-file`로 원문을 꺼냅니다. vendor만 담긴 cache가 hit하면 core 빌드를
  건너뛰고 source packaging이 실패합니다.
- key versioning: 이미 존재하는 cache는 내용을 다시 쓸 수 없으므로, vendor-only로 만들어진 옛 key는
  새 정의의 cache로 재사용될 수 없습니다. `v2`는 별도 cache이고 옛 key는 **파일에서 제거**했습니다.
- key 의존성은 그대로 `cores/**` — manifest(`required.txt`), pin(`<core>/commit`), recipe(`build.sh`,
  `common.sh`), patch(`*.patch`)를 모두 포함합니다.
- cache하지 **않은** 것: `target-device` 전체 트리, 완성된 bundle(`target/core-sources`,
  `target/rust-notices`), `dist-device` — 산출물입니다.

## 5. cache-hit 검증 body (`core files and checkouts present`)

`cores/required.txt` 루프(코어 이름을 YAML에 복제하지 않음)로 각 core에 대해:

1. `vendor/<core>_libretro.so`, `vendor/<core>_libretro.so.meta`가 nonempty(`-s`).
2. `target-device/cores/<core>`가 존재.
3. 그 아래 `.git`을 가진 디렉터리가 **정확히 1개**(`find -maxdepth 3 -name .git -printf '%h\n'`) —
   `Get-CoreCheckout`과 같은 규칙이라 mGBA의 `mgba/`, 나머지 다섯의 `src/` 차이가 YAML에 들어가지 않습니다.
4. 그 checkout이 추적 pin 객체를 포함(`git cat-file -e <pin>^{commit}`).
5. 그 checkout의 `HEAD`가 추적 pin과 같음(`git rev-parse HEAD`).

실패하면 core 이름, 경로, pin을 담은 `::error::`를 내고 `exit 1` 합니다. cache miss를 warning으로
낮추지 않고, hit를 받아들인 뒤 조용히 다시 빌드하지도 않습니다.

## 6. 검증 결과 (네트워크 없이)

### 6.1 YAML 검증 — 방법과 한계

이 호스트에는 actionlint·PyYAML·ruby·node·yq가 **없고**(설치는 네트워크 필요), 그래서 stdlib만 쓰는
안전한 전용 파서(`%TEMP%\slot2-task102-gate\ci-yaml-check.py`)를 작성해 workflow가 쓰는 subset
(block mapping/sequence, block scalar, plain/quoted scalar, 주석)을 파싱했습니다. 일반 YAML 구현이
아니며 스키마 검증도 하지 않습니다 — 그 한계를 여기 명시합니다. 파서가 단언한 것:

```text
jobs: check, device
devices steps: 1 checkout 2 rust-toolchain 3 rust-cache 4 device target on the host 5 build cross image
               6 cache device cores 7 build device cores 8 core files and checkouts present
               9 cross build (profile device) 10 package core sources
               11 fetch Cargo sources for the notice packager 12 package rust notices
               13 assemble card tree 14 upload-artifact
order: rust-toolchain(2) -> rust-cache(3) -> device target on the host(4) -> build cross image(5)
       -> fetch(11) -> package rust notices(12) -> assemble card tree(13) -> upload-artifact(14)
device cache paths: ['vendor', 'target-device/cores']
device cache key:   "cores-aarch64-v2-${{ hashFiles('cores/**') }}"
check job cache key untouched: "cores-x86_64-${{ hashFiles('cores/**') }}"
old device key absent from the file: yes
docker cross build still present: True ; cross build flags kept: True
```

추가로 YAML에서 뽑아낸 **모든 `run` body 9개를 `bash -n`으로 문법 검사**(git bash)했고 전부 rc=0입니다.

### 6.2 실제 트리에 대한 cache-hit 검증 body 실행

추출한 body를 실제 저장소에서 그대로 실행: **exit 0, `::error::` 0건.**

| core | pin | checkout | 관측 HEAD | 일치 |
|---|---|---|---|---|
| mgba | `e31759b24e7a4e3899285ff720d7b573ac328ae7` | `target-device\cores\mgba\mgba` | `e31759b2…` | ✓ |
| gambatte | `d9d6cd06382d1ced30de34d56d3609452323dab1` | `target-device\cores\gambatte\src` | `d9d6cd06…` | ✓ |
| gpsp | `5819380c2ffb0900219d700a382ee68c464ebb99` | `target-device\cores\gpsp\src` | `5819380c…` | ✓ |
| fceumm | `236ccdfc911e84c60fea6b9d0699c2d440a8de14` | `target-device\cores\fceumm\src` | `236ccdfc…` | ✓ |
| snes9x | `fae2fea08f74180759ef540ee94259213f503480` | `target-device\cores\snes9x\src` | `fae2fea0…` | ✓ |
| genesis_plus_gx | `c2838c7dc4236fc2fe94e5dbd08b41486067918e` | `target-device\cores\genesis_plus_gx\src` | `c2838c7d…` | ✓ |

### 6.3 negative 시뮬레이션 (임시 사본, 실제 checkout·pin·바이너리·cache 무수정)

`target/task104-negative/` 아래에 core manifest·pin·vendor 바이너리·`--shared` scratch checkout으로
가짜 루트를 만들고 **같은 body**를 실행했습니다.

| 케이스 | 종료 | 진단 |
|---|---|---|
| 대조군(충실한 사본) | **0** | 오류 0 |
| 1 checkout 없음 (`gpsp` 삭제) | 1 | `::error::gpsp has no source checkout under target-device/cores/gpsp (tracked pin 5819380c…)` |
| 2 HEAD가 pin이 아님 (`snes9x`) | 1 | `::error::target-device/cores/snes9x/src HEAD is c18b4203…, not the tracked pin fae2fea0…` |
| 3 vendor-only cache (디렉터리만, `.git` 없음) | 1 | 코어별 `has 0 git checkouts under … , not one (tracked pin …)` 6건 |
| 4 pin이 history에 없음 (`fceumm`, 빈 repo) | 1 | `::error::target-device/cores/fceumm/src does not contain the tracked pin 236ccdfc…` |
| 5 checkout 2개 (모호) | 1 | `::error::mgba has 2 git checkouts under target-device/cores/mgba, not one (tracked pin e31759b2…)` |

### 6.4 Rust notice packager (offline, 임시 출력)

```text
==> reading the device dependency closure from Cargo (offline)
==> 66 third-party packages, Cargo.lock dce2d3bfd41a
==> bundle checked (66 packages)
==> wrote 186 files (769598 bytes) to C:\SLOT2\target\task104-rust-notices-verify
```

exit 0, 패키지 66(63 `packaged-text` + 3 `declared-only`), 파일 186, `RUST-MANIFEST.txt` 185줄,
`RUST-SBOM.json` sha256 `cda0b86cb1340c138dcb69422564180fcaf0df81058fc1ada2cdfd69b6556acf` — Task 103의
최종 값과 **동일**(정책·키·레이아웃·결정성 변화 없음). 로컬 Cargo cache에 모든 입력이 있어 offline으로
끝났습니다.

### 6.5 `git diff --check`

exit 0 (stdout 진단 없음, 기존 CRLF stderr 경고만).

## 7. 변경 파일과 검증 뒤 변경 여부

- **변경**: `.github/workflows/ci.yml` 하나 (mtime 2026-10-01 16:52:21).
- **미변경(확인)**: `build/package-rust-notices.ps1`(14:10), `build/rust-notices.ps1`(14:13),
  `build/verify-rust-notices.ps1`(14:09), `build/package-core-sources.ps1`(10:45),
  `build/core-manifest.ps1`(09-30), `build/dist-device.ps1`(14:08 — Task 103 변경분),
  `build/cores.ps1`(09-30), `cores/common.sh`(09-27), `cores/required.txt`(09-30),
  `Cargo.lock`(09-27), `Cargo.toml`(09-23), `rust-toolchain.toml`(09-22). 이번 태스크에서 제품·packager·
  validator·Cargo·core recipe/pin/patch·asset·license·Task 103 파일을 건드리지 않았습니다.
- 마지막 파일 변경(16:52:21) **이후** YAML 파서 단언, `bash -n` 9건, 실제/가짜 트리 body 실행, packager
  실행, `git diff --check`를 모두 수행했고 그 뒤 파일 변경은 없습니다(마지막 확인 16:59).
- 커밋하지 않았습니다.

## 8. 호스팅 CI 미실행과 남은 수용 작업

- **GitHub Actions는 실행하지 않았습니다.** 이 호스트에서 workflow를 돌릴 수 없고(러너·Docker 교차 빌드·
  네트워크 필요), 이번 태스크는 네트워크 사용을 금지합니다. 따라서 "CI가 통과했다"고 주장하지 않습니다.
- 로컬에서 실행할 수 없었던 부분: `cargo fetch --locked …`의 실제 수행(네트워크 필요 — 로컬 cache에 이미
  모든 source가 있어 검증은 packager 성공으로 대신했습니다), Docker 교차 빌드, `dtolnay/rust-toolchain`,
  `Swatinem/rust-cache`, `actions/cache`, `actions/upload-artifact` 자체의 동작.
- 남은 수용 작업: (1) 실제 GitHub Actions `device` job 1회 실행으로 새 fetch 단계와 `v2` cache key가
  동작하는지 확인(첫 실행은 cache miss → 빌드, 두 번째 실행은 hit → checkout 검증 경로), (2) 그 실행의
  artifact에 `System/licenses/rust` 186 파일과 여섯 core·source 트리가 들어 있는지 확인, (3) 그래야 M7의
  artifact parity 주장이 가능합니다. Task 103/104 모두 릴리스 태그·README·M7 전체 acceptance는 미완입니다.

## 9. 계약 관련 의견

1. **`cargo fetch --locked`를 두 번 실행**한 것은 계약 문구("explicit locked Cargo fetch for
   `aarch64-unknown-linux-gnu`")보다 한 걸음 넓습니다. 근거: packager의 `cargo metadata`는 workspace
   전체를 해석하므로 device 타깃 fetch만으로는 호스트 전용 멤버의 source가 비어 offline metadata가
   실패할 수 있습니다(Task 103 리뷰의 실패 모드와 같은 종류). `--locked`는 유지했고 crate 목록 하드코딩은
   없습니다. 만약 CI에서 두 번째 fetch를 불필요하게 보신다면 그때 좁혀도 되지만, 먼저 지금 형태로 새
   runner에서 한 번 돌려 보는 편이 안전합니다.
2. **검증 body의 checkout 탐색**은 `Get-CoreCheckout`의 규칙(고유 `.git`, depth ≤ 3)을 shell로 옮긴
   것입니다. PowerShell validator를 YAML에서 호출하는 방식도 가능하지만, 기존 device 검증이 모두 bash이고
   `pwsh` 호출을 늘리면 실패 지점이 하나 더 생기므로 같은 규칙을 shell로 표현했습니다. 규칙이 바뀌면 두 곳을
   같이 고쳐야 한다는 점이 유일한 부채입니다.
3. **`cache-targets: false`**는 `Swatinem/rust-cache@v2`의 문서화된 입력입니다. 이 job의 산출물
   (bundle, dist-device)을 cache에 싣지 않기 위한 선택이며, 실제 CI 실행에서 입력명이 무시되면 경고만
   나고 동작에는 영향이 없습니다.
