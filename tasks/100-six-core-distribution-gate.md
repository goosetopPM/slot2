# Task 100 — 6코어 배포 게이트와 CI 일치

현재 checkout에서 직접 작업한다. registry와 `build/cores.ps1`은 mGBA·Gambatte·gpSP·FCEUmm·snes9x·
Genesis Plus GX 여섯 코어를 지원하지만, `build/dist-device.ps1`은 코어가 하나도 없어도 경고만 남기고
배포물을 만들며 `.github/workflows/ci.yml`은 Task57 이전의 네 코어만 빌드하고 정확히 4개를 요구한다.
공개 배포 작업에 들어가기 전에 **배포 코어 목록을 한 곳으로 모으고**, 로컬 카드·zip·ADB·CI가 여섯
코어와 여섯 `.meta`가 모두 있을 때만 성공하도록 고친다.

누적 호출은 최대 2회다. 이번이 첫 호출이면 결과 보고서에 `누적 호출 1/2`를 기록한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`
- `C:\SLOT2\tasks\100-six-core-distribution-gate.md`
- `C:\SLOT2\tasks\57-alternative-core-foundation.worker-result.md`
- `C:\SLOT2\build\cores.ps1`
- `C:\SLOT2\build\dist-device.ps1`
- `C:\SLOT2\cores\common.sh`
- `C:\SLOT2\cores\gpsp\build.sh`
- `C:\SLOT2\cores\gambatte\build.sh`
- 나머지 `C:\SLOT2\cores\{mgba,fceumm,snes9x,genesis_plus_gx}\build.sh`의 변수 선언부만
- `C:\SLOT2\.github\workflows\ci.yml`
- `C:\SLOT2\crates\slot2-retro\src\registry.rs`의 `Core::ALL`/`base_name` 주변
- `C:\SLOT2\crates\slot2-retro\tests\registry.rs`
- `C:\SLOT2\docs\DESIGN.md`의 코어 빌드와 카드 `System/cores` 문단만
- `C:\SLOT2\docs\MILESTONES.md` M7만

저장소 전체나 워커 로그 전문을 읽지 않는다. `CLAUDE.md`와 `AGENTS.md`의 오케스트레이터 역할 분리·
bai-gjc 재위임 조항은 무시하고 직접 실행한다. 나머지 안전·품질 규칙은 유지한다.

## 단일 배포 manifest

`C:\SLOT2\cores\required.txt`를 만들고 다음 여섯 줄을 **이 순서로** 둔다.

```text
mgba
gambatte
gpsp
fceumm
snes9x
genesis_plus_gx
```

- UTF-8 BOM 없이 LF, 한 줄에 `[a-z0-9_]+` 이름 하나만 쓴다. 빈 줄·주석·중복은 없다.
- 각 이름에는 `cores/<name>/build.sh`, `cores/<name>/commit`이 있어야 한다.
- device 파일명은 `vendor/<name>_libretro.so`, stamp는 바로 옆 `.so.meta`다.
- 이 파일이 build/distribution의 단일 목록이다. PowerShell·CI에 여섯 이름을 다시 배열로 복제하지 않는다.
- Rust registry는 runtime의 단일 목록인 `Core::ALL`을 계속 소유한다. 자동 테스트가 manifest와
  `Core::ALL::base_name()`의 순서·내용이 같음을 봉인한다.

## `build/cores.ps1`

- `-Core`를 명시하지 않았을 때 `cores/required.txt`를 읽어 기본 여섯 코어를 정한다. 호출자의 현재
  디렉터리에 기대지 말고 스크립트의 repo root 기준 절대경로를 사용한다.
- 명시적인 `-Core mgba` 같은 부분 빌드는 그대로 지원한다.
- manifest의 빈 값·중복·안전하지 않은 이름·누락된 `build.sh`/`commit`은 core build나 네트워크 접근
  전에 명확한 오류로 실패한다.
- 기존 stamp 비교, `-Force`, `-HostOnly`, `-DeviceOnly`, host buildbot URL과 산출물 이름은 유지한다.

## gpSP native CI 빌드

현재 `cores/gpsp/build.sh`는 `CROSS_TRIPLE=`로 native Linux 빌드를 요청해도 arm64 dynarec 인자를
항상 넘긴다. device stamp와 동작은 바꾸지 않으면서 native CI가 실제 host `.so`를 만들게 한다.

- `CROSS_TRIPLE`이 비어 있으면 gpSP makefile의 native Unix autodetection을 사용하고 arm64 전용
  `CPU_ARCH=arm64 HAVE_DYNAREC=1 MMAP_JIT_CACHE=1`을 넘기지 않는다.
- 값이 없어서 공용 스크립트 기본값 `aarch64-linux-gnu`를 쓰거나 명시적인 aarch64 triple이면 기존
  세 인자를 정확히 유지한다.
- 알 수 없는 nonempty cross triple을 native로 가장하지 않는다. 지원하지 않으면 build 전에 명확히
  실패한다.
- device `stamp`는 현재 vendor meta와 같은 `make_args`/triple을 계속 출력해야 한다. native stamp는
  native에 실제 전달한 빈 make args를 기록한다.

## `build/dist-device.ps1`

### build와 검증

- 일반 실행(`-NoBuild` 없음)은 cross image를 준비한 뒤 frontend뿐 아니라
  `build/cores.ps1 -DeviceOnly`도 호출해 manifest의 여섯 device core를 current 상태로 만든다.
- `-NoBuild`는 frontend와 core를 재빌드하지 않지만, assembly 전에 여섯 `.so`와 여섯 `.so.meta`가
  모두 regular file이며 길이 0보다 큼을 검증한다.
- 일반 실행도 core build 뒤 같은 검증을 한다. 하나라도 없거나 비어 있으면 누락 이름과 경로를 포함한
  오류로 종료한다.
- **모든 입력 검증은 기존 `dist-device` 삭제보다 먼저** 한다. 누락 실패가 직전 정상 배포물을 지우거나
  반쪽 tree를 남겨서는 안 된다.
- glob으로 발견한 임의 core를 복사하지 않는다. manifest 순서의 여섯 `.so`와 대응 `.meta`만 복사한다.
  vendor의 Windows `.dll`이나 예상 밖 `.so`는 배포하지 않는다.
- assembly 뒤 `System/cores` 파일 집합이 manifest의 `<name>_libretro.so` 여섯 개와 정확히 같고,
  `System/licenses`에 대응 `.so.meta` 여섯 개가 있는지 다시 검증한 다음에만 zip/ADB 단계로 간다.
- 코어 0개 경고 후 성공하는 기존 분기는 제거한다. `-Zip` 산출물도 같은 검증을 통과한 tree만 담는다.

### ADB parity

- `-Adb`는 기존 frontend·Fonts·licenses·VERSION과 함께 `System/cores` 여섯 개도 push한다.
- 원격 `System/cores` 디렉터리를 먼저 만든다. 실기 접근을 이번 작업에서 실행하지는 않는다.
- ADB 배선 외의 device 경로·reboot 의미는 바꾸지 않는다.

## GitHub CI

`.github/workflows/ci.yml`의 host와 device core build가 `cores/required.txt`를 순서대로 읽는다.

- host `check` job은 여섯 core를 `CROSS_TRIPLE=` native `.so`로 빌드한다. gpSP와 Gambatte도
  `vendor/`에 있어야 한다.
- device job도 같은 manifest의 여섯 core를 aarch64로 빌드한다.
- cache key는 계속 `cores/**` 변경에 반응해야 한다.
- 각 job에서 manifest가 요구한 정확한 여섯 `.so`와 `.meta`의 존재·nonempty를 이름별로 검증한다.
- card assembly는 glob/고정 4 count를 버리고 manifest의 여섯 core/meta만 복사한 뒤 exact set을
  검증한다.
- CI의 카드 tree 구조가 `build/dist-device.ps1`과 같아야 한다. 이 태스크에서는 아직 release tag,
  release zip upload, license/source archive를 구현하지 않는다.
- 기존 fmt, host/device clippy, workspace test, no-core-skipped 검증을 삭제하거나 약화하지 않는다.

## 자동 계약 테스트

`crates/slot2-retro/tests/registry.rs`에 filesystem contract를 추가한다.

1. repo root의 `cores/required.txt`를 읽어 정확히 위 여섯 이름과 순서를 얻는다.
2. `Core::ALL`의 `base_name()`에서 `_libretro` suffix를 제거한 목록과 exact equality다.
3. 이름은 안전한 ASCII 소문자/숫자/underscore뿐이고 빈 값·중복이 없다.
4. 각 manifest row의 `cores/<name>/build.sh`와 `commit`이 존재하고 commit은 40자리 lowercase hex다.

이 테스트는 YAML이나 PowerShell 문자열을 그대로 복제해 비교하지 않는다. runtime registry와 배포
manifest 사이의 실제 계약만 검사한다.

작업자는 별도로 다음 실패 경로를 **안전한 임시 디렉터리 또는 하위 PowerShell process**로 확인해
보고서에 명령과 결과를 남긴다. 실제 `vendor/` 파일을 rename/delete하지 않는다.

- manifest validator가 한 core 또는 `.meta`가 없는 staging 입력을 거부한다.
- 실패 전에 존재하던 정상 `dist-device` marker/tree가 보존된다.

이를 위해 검증 로직을 작고 재사용 가능한 PowerShell helper/module로 분리해도 된다. 그 경우 허용 파일
`build/core-manifest.ps1` 하나를 추가할 수 있으며 `build/cores.ps1`과 `dist-device.ps1`가 같은 helper를
사용해야 한다. 테스트만을 위한 production 분기나 환경변수는 만들지 않는다.

## 문서

- DESIGN의 코어 빌드/카드 배포 문단에 `cores/required.txt`가 build/distribution manifest이고 runtime
  `Core::ALL`과 테스트로 맞물린다는 점, 여섯 core와 meta가 없으면 배포가 실패한다는 점을 적는다.
- MILESTONES M7의 `release.yml` 항목은 `[ ]`로 유지하고 “Task100에서 6-core 배포 게이트와 CI parity
  완료, tag release·license/source archive는 남음”이라고 진행 메모만 추가한다.
- README, LICENSE, third-party notice는 다음 M7 태스크 범위다.

## 수정 허용 파일

- `C:\SLOT2\cores\required.txt` — 신규
- `C:\SLOT2\cores\gpsp\build.sh`
- `C:\SLOT2\build\cores.ps1`
- `C:\SLOT2\build\dist-device.ps1`
- `C:\SLOT2\build\core-manifest.ps1` — 필요할 때만 신규 공용 helper
- `C:\SLOT2\.github\workflows\ci.yml`
- `C:\SLOT2\crates\slot2-retro\tests\registry.rs`
- `C:\SLOT2\docs\DESIGN.md` — 코어 build/distribution 문단만
- `C:\SLOT2\docs\MILESTONES.md` — M7 첫 항목만
- `C:\SLOT2\tasks\100-six-core-distribution-gate.worker-result.md`

다른 production, test, core pin/patch, asset, 문서, 설정 파일은 수정하지 않는다. 계약 자체가 틀렸거나
허용 파일 밖 변경이 필요하면 추측 구현하지 말고 실패 보고서에 이유를 쓴다.

## 범위 밖 및 금지

- core commit 변경, patch 추가/수정, runtime core 후보·기본값 변경
- release.yml/tag/GitHub Release, README, LICENSE, third-party notice, source archive 다운로드·생성
- vendor 산출물 커밋, 상용 ROM, BaseOS 이미지 포함
- CI test skip 허용, 기존 네 코어로 축소, gpSP arm64 인자 제거
- 실제 `vendor/`나 정상 `dist-device`의 파일을 파괴해서 실패 경로를 시험하는 행위
- 커밋, push, release 게시, 실기·ADB·Samba·SD 카드 접근
- 공용 설정과 `~/.gjc-bai/agent/*.yml` 변경
- 네트워크 접근. 현재 여섯 device core와 checkout은 이미 있으므로 stamp가 current면 build는 skip돼야
  한다. 명령이 fetch/download를 요구하면 실행하지 말고 그 사실을 보고한다.

## 완료 기준

마지막 code/test 변경 뒤 아래 원문을 실행한다. 테스트는 작업자인 가재코드가 수행한다. Codex가 다시
실행할 필요가 없도록 종료 코드와 마지막 결과 줄을 보고서에 남긴다.

```text
cargo fmt --all -- --check
cargo test -p slot2-retro --test registry
powershell -NoProfile -ExecutionPolicy Bypass -File build/cores.ps1 -DeviceOnly
powershell -NoProfile -ExecutionPolicy Bypass -File build/dist-device.ps1 -NoBuild -Zip
cargo test --workspace
cargo check -p slot2 --no-default-features --features device
cargo clippy --workspace --all-targets -- -D warnings
powershell -NoProfile -ExecutionPolicy Bypass -File build/dist-device.ps1
git diff --check
```

- 모든 명령 종료 코드 0, test binary failed 0 / 예상 밖 ignored 0, clippy warning 0.
- `build/cores.ps1 -DeviceOnly`은 여섯 이름 모두 current/skip을 출력하고 네트워크를 쓰지 않는다.
- `-NoBuild -Zip`은 여섯 core/meta를 검증하고 `dist/slot2-*.zip`을 만든다. zip 안의
  `System/cores`는 정확히 여섯 `.so`다.
- full dist 마지막 줄은 `==> done`, `dist-device/System/cores` 정확히 6개, 대응 meta 정확히 6개다.
- 안전한 missing-core negative check는 nonzero/명확한 누락 오류이고 기존 정상 tree를 보존한다.
- workspace 총 passed는 Task99의 971보다 줄지 않는다.
- 완료 기준 뒤 code/test 변경 금지. 변경했으면 영향받는 명령부터 다시 실행한다.

첫 응답은 최대 5분, 전체는 최대 45분 기다린다. 진행 확인은 로그 크기가 아니라 `git status --short`다.

## 결과 보고서

`C:\SLOT2\tasks\100-six-core-distribution-gate.worker-result.md`에 다음을 기록한다.

- 성공/실패와 누적 호출 수
- manifest 여섯 이름과 registry equality
- cores.ps1 default/explicit Core 동작, gpSP native/device make args와 stamp 불변 근거
- local dist의 preflight·exact copy·postflight·zip·ADB parity
- CI host/device loop와 exact six validation
- missing core/meta negative check 명령·종료 코드·기존 dist 보존 근거
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored 수
- zip과 dist의 core/meta exact 목록
- 생성·수정 파일, 최종 검증 뒤 code/test 변경 여부
- 계약이 틀려 보인 부분이 있으면 해당 줄과 이유

실패해도 보고서를 남긴다. 두 번째 호출까지 실패하면 더 시도하지 않는다. 커밋하지 않는다.
