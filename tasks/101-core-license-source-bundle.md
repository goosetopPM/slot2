# Task 101 — 코어 라이선스와 corresponding source 묶음

현재 checkout에서 직접 작업한다. Task100으로 카드·zip·CI가 여섯 device core를 정확히 요구하게 됐지만,
현재 `System/licenses`에는 font license와 `.so.meta`만 있고 core의 실제 license text와 source archive가
없다. 공개 배포 전에 SLOT2의 MIT 고지, 원작 slot 고지, 여섯 pinned core의 원문 license, pristine pinned
source와 SLOT2 build recipe/patch를 카드 tree와 zip에 실제로 동봉한다.

이 작업은 **core/source compliance 기반**이다. Rust crate 의존성 전수 고지, README, GitHub tag release는
후속으로 분리한다. 법률 해석을 새로 쓰지 않고 pinned source 안의 원문과 확인 가능한 사실만 기록한다.

누적 호출은 최대 2회다. 이번이 첫 호출이면 결과 보고서에 `누적 호출 1/2`를 기록한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`
- `C:\SLOT2\tasks\101-core-license-source-bundle.md`
- `C:\SLOT2\tasks\100-six-core-distribution-gate.worker-result.md`
- `C:\SLOT2\cores\required.txt`
- `C:\SLOT2\build\core-manifest.ps1`
- `C:\SLOT2\build\dist-device.ps1`
- `C:\SLOT2\.github\workflows\ci.yml`의 device core/assembly 단계
- `C:\SLOT2\cores\common.sh`
- 여섯 `cores/<name>/{commit,build.sh,*.patch}`
- 아래 local pinned checkout의 top-level license 파일만
- `C:\Users\gyuha\slot-2\LICENSE` — 원작 MIT, 읽기만
- `C:\SLOT2\assets\fonts\*-OFL.txt`
- `C:\SLOT2\assets\skins\PROVENANCE.md`, `assets\sfx\PROVENANCE.md`
- `C:\SLOT2\docs\DESIGN.md`의 카드 licenses/코어 build 문단만
- `C:\SLOT2\docs\MILESTONES.md` M7만

local checkout과 license path:

| core | checkout | top-level license |
|---|---|---|
| mgba | `target-device/cores/mgba/mgba` | `LICENSE` |
| gambatte | `target-device/cores/gambatte/src` | `COPYING` |
| gpsp | `target-device/cores/gpsp/src` | `COPYING` |
| fceumm | `target-device/cores/fceumm/src` | `Copying` |
| snes9x | `target-device/cores/snes9x/src` | `LICENSE` |
| genesis_plus_gx | `target-device/cores/genesis_plus_gx/src` | `LICENSE.txt` |

checkout을 통째로 읽거나 로그 전문을 읽지 않는다. license 원문과 git metadata, 필요한 script만 본다.
`CLAUDE.md`와 `AGENTS.md`의 오케스트레이터 역할 분리·bai-gjc 재위임 조항은 무시하고 직접 실행한다.
나머지 안전·품질 규칙은 유지한다.

## 추적할 고지 파일

### SLOT2와 원작

1. 루트 `LICENSE`를 표준 MIT 전문으로 만들고 copyright line은 정확히 다음으로 한다.

```text
Copyright (c) 2026 SLOT2 contributors
```

2. `licenses/upstream-slot/LICENSE`는 `C:\Users\gyuha\slot-2\LICENSE`의 **byte-identical copy**다.
   원작자 이름·연도·문구를 고치거나 요약하지 않는다.

3. 루트 `CORE-NOTICES.md`를 만든다. 다음 사실을 간결한 표와 문단으로 기록한다.

- SLOT2 frontend 자체는 root `LICENSE`의 MIT.
- 원작 `brandonkowalski/slot`에서 이식한 부분은 `licenses/upstream-slot/LICENSE`와 각 asset
  `PROVENANCE.md`를 따른다.
- 여섯 core 각각: display name, repository URL, `cores/<name>/commit`의 full pin, 동봉 license 경로,
  동봉 source zip과 recipe 경로.
- mGBA는 MPL-2.0, Gambatte/gpSP/FCEUmm은 GPL-2.0 전문, Snes9x와 Genesis Plus GX는 각 upstream의
  custom non-commercial 원문을 따른다는 **식별 정보**만 적는다. 권리·의무를 자체 문장으로 축약하거나
  “오픈소스”라고 뭉뚱그리지 않는다.
- Snes9x와 Genesis Plus GX binary를 판매·상업 활동에 사용하는 것이 upstream 원문상 제한된다는 점을
  눈에 띄게 안내하고 원문 license가 최종 기준이라고 적는다.
- archive가 pristine pinned source이고 `recipes/`의 patch/build script를 함께 적용해야 배포 binary와
  대응한다는 구조를 설명한다.
- fonts는 `System/licenses/fonts`, skin/sfx 출처는 repository의 PROVENANCE 문서를 가리킨다.
- “AI-assisted development” 항목에 Claude Code, OpenAI Codex, GajaeCode/OpenCodex와 B.AI 모델이 구현·
  검토에 사용됐고 최종 책임은 maintainers에게 있다는 사실만 적는다. 코드 저작권이나 license를 AI가
  소유한다고 쓰지 않는다.
- 이 문서는 법률 자문이 아니며 license 원문을 대체하지 않는다고 적는다.

### 여섯 core 원문 license

`licenses/cores/<name>/` 아래에 pinned checkout의 top-level license를 원래 파일명 그대로 복사한다.

- `licenses/cores/mgba/LICENSE`
- `licenses/cores/gambatte/COPYING`
- `licenses/cores/gpsp/COPYING`
- `licenses/cores/fceumm/Copying`
- `licenses/cores/snes9x/LICENSE`
- `licenses/cores/genesis_plus_gx/LICENSE.txt`

복사본은 `git show <pin>:<license-path>` 결과와 byte-identical이어야 한다. working tree는 autocrlf나
applied patch 때문에 dirty일 수 있으므로 단순 `Copy-Item` 결과만 믿지 말고 **git object의 pinned blob**을
기준으로 생성·검증한다. license text를 번역·재포맷·줄바꿈 정규화하지 않는다.

각 source archive 안에는 upstream이 추적하는 nested dependency license도 원래 경로로 들어간다.
top-level 복사본 외 nested license를 수동 선별해 별도 추적하지 않는다.

## source/recipe packager

`build/package-core-sources.ps1`을 추가한다.

```text
powershell -File build/package-core-sources.ps1 -OutputDir <absolute-or-relative-dir>
```

### 입력과 preflight

- repo root와 `cores/required.txt`는 script 위치 기준으로 찾고 `core-manifest.ps1`을 재사용한다.
- 각 core의 `target-device/cores/<name>/` 아래에서 `.git` checkout을 정확히 하나 찾는다. 현재 mGBA는
  `mgba/`, 나머지는 `src/`지만 이름별 path table을 새로 만들지 말고 실제 git root를 찾는다.
- git root가 0개/2개 이상이면 실패한다.
- checkout `HEAD`가 `cores/<name>/commit`의 40자리 pin과 exact match인지 확인한다.
- archive 전에 `git cat-file -e <pin>^{commit}`, top-level license blob 존재, tracked license 복사본과
  `git show <pin>:<path>` byte equality를 확인한다.
- 여섯 checkout·license·recipe 입력을 모두 검증한 뒤에만 기존 `OutputDir`을 교체한다. 실패가 직전 정상
  source bundle을 삭제하거나 부분 output을 남기면 안 된다.
- network fetch/clone은 하지 않는다. checkout이 없으면 `build/cores.ps1 -DeviceOnly`를 먼저 실행하라는
  core 이름·예상 경로 포함 오류로 끝낸다.

### 출력 구조

```text
<OutputDir>/
├─ SOURCE-MANIFEST.txt
├─ archives/
│  ├─ mgba-<40hex>.zip
│  └─ ... 여섯 개
├─ licenses/
│  └─ <name>/<원래 license filename>
└─ recipes/
   ├─ common.sh
   └─ <name>/
      ├─ commit
      ├─ build.sh
      └─ *.patch        # 있는 core만; 현재 mGBA 두 개
```

- archive는 `git archive --format=zip --prefix=<name>-<pin>/ <pin>`으로 **git object의 pristine pinned
  source 전체**를 만든다. working tree의 line-ending noise, build artifact, `.git`, 적용 후 dirty file을
  집어넣지 않는다.
- 여섯 archive 모두 해당 pin의 top-level license를 포함해야 한다.
- recipe는 repository의 `cores/common.sh`, 각 `commit`, `build.sh`, 모든 `*.patch`를 byte 그대로 복사한다.
- `SOURCE-MANIFEST.txt`는 UTF-8 BOM 없이 LF로 name, repository URL(`.so.meta`/build script와 일치),
  full pin, archive relative path, archive SHA-256, license relative path, patch filename+SHA-256을
  deterministic한 manifest 순서로 기록한다. 실행 시각·절대경로·machine name은 넣지 않는다.
- 같은 checkout에서 두 번 실행하면 archive bytes와 `SOURCE-MANIFEST.txt` SHA-256이 같아야 한다.
- staging directory를 사용해 성공 시에만 `OutputDir`을 교체한다. staging은 workspace의 `target/` 또는
  OS temp 아래 안전한 child를 사용하며 성공·실패 뒤 정리한다.

## `dist-device.ps1` 배선

- core preflight 뒤, 기존 `dist-device`를 지우기 전에 packager를 staging output으로 실행한다.
- root `LICENSE`, `CORE-NOTICES.md`, `licenses/upstream-slot`, tracked `licenses/cores`, 생성된 source bundle을
  모두 검증한 뒤 assembly를 시작한다.
- output은 다음과 같다.

```text
System/licenses/SLOT2-LICENSE
System/licenses/CORE-NOTICES.md
System/licenses/upstream-slot/LICENSE
System/licenses/fonts/*
System/licenses/cores/<name>/<license>
System/licenses/sources/{SOURCE-MANIFEST.txt,archives,licenses,recipes}
System/licenses/<name>_libretro.so.meta
```

- tracked core license와 generated source bundle의 `licenses/`는 같은 pinned blobs이므로 byte equality를
  postflight에서 확인한다.
- 기존 exact-six core/meta 검증을 유지한다. license/source 누락·빈 archive·hash mismatch도 zip/ADB 전에
  실패한다.
- `-Zip`은 이 전체 tree를 담는다. `-Adb`는 이미 `System/licenses/.`를 push하므로 새 별도 실기 명령을
  추가하지 않는다.
- `-NoBuild`에서도 local pinned checkout이 있어야 source bundle을 만들 수 있다. 없으면 기존 정상
  `dist-device`를 보존하고 명확히 실패한다.

## CI device assembly parity

`.github/workflows/ci.yml` device job에서 여섯 core build가 만든 checkout을 사용해
`pwsh -NoProfile -File build/package-core-sources.ps1`을 실행한다.

- CI card tree도 local dist와 같은 SLOT2 license, notice, upstream slot, core licenses, generated sources,
  font licenses, six meta를 포함한다.
- archive 6개, tracked/generated core license 6개, recipe 6개, mGBA patch 2개의 존재·nonempty와 source
  manifest/hash를 검증한다.
- 기존 exact-six core/meta 검증과 artifact upload를 유지한다.
- 이 태스크에서 `.github/workflows/release.yml`은 만들지 않는다. CI artifact가 release에 쓸 완전한 tree를
  먼저 만드는 단계다.

## 자동 계약 검증

새 Rust production 코드는 필요 없다. 검증은 packager와 배포 script의 실제 output을 대상으로 한다.

1. tracked six license가 각 checkout의 `git show <pin>:<path>`와 byte-identical.
2. source archive 정확히 6개, 각 이름/pin 일치, 해당 top-level license entry 포함.
3. recipe 정확히 6개, commit/build.sh 포함, 공용 common.sh 포함, mGBA patch 정확히 2개이며 repository의
   recipe와 byte-identical.
4. `SOURCE-MANIFEST.txt`의 archive/patch SHA-256이 실제 파일과 일치하며 절대경로·실행시각 없음.
5. packager 연속 2회 결과의 모든 relative path와 SHA-256 exact equality.
6. checkout 하나 또는 tracked license 하나가 없는 **staging copy**에서 nonzero로 실패하고 이전 output
   marker/tree 보존. 실제 target checkout, tracked license, 정상 dist를 rename/delete하지 않는다.
7. final `dist-device`와 zip에 위 license/source tree가 모두 있으며 core/meta exact-six도 유지.

PowerShell helper 검증을 `build/core-manifest.ps1`에 일반화해 추가해도 되지만 Task100의 기존 API와
negative behavior를 깨지 않는다. 테스트만을 위한 production switch나 환경변수는 만들지 않는다.

## 문서

- DESIGN 카드 layout을 위 실제 `System/licenses` tree로 갱신하고 pristine source + patch/build recipe가
  binary 대응 source를 이룬다는 점을 적는다.
- MILESTONES M7의 release 항목은 `[ ]` 유지: “Task101 core license/source bundle 완료, Rust dependency
  notices와 tag release 남음”으로 진행 메모를 갱신한다.
- M7 LICENSE/고지 항목도 Rust dependency 전수와 최종 README가 남으므로 `[ ]` 유지하고 root MIT,
  upstream/core/font/AI notices 기반 완료만 기록한다.
- 아직 release workflow나 공개 배포가 완료됐다고 쓰지 않는다.

## 수정 허용 파일

- `C:\SLOT2\LICENSE` — 신규
- `C:\SLOT2\CORE-NOTICES.md` — 신규
- `C:\SLOT2\licenses\upstream-slot\LICENSE` — 신규
- `C:\SLOT2\licenses\cores\<six>\<license>` — 신규 6개
- `C:\SLOT2\build\package-core-sources.ps1` — 신규
- `C:\SLOT2\build\core-manifest.ps1` — 공용 검증이 필요할 때만
- `C:\SLOT2\build\dist-device.ps1`
- `C:\SLOT2\.github\workflows\ci.yml`
- `C:\SLOT2\docs\DESIGN.md` — core source/license와 card layout 문단만
- `C:\SLOT2\docs\MILESTONES.md` — M7 관련 두 항목만
- `C:\SLOT2\tasks\101-core-license-source-bundle.worker-result.md`

다른 source, core pin/patch, Cargo metadata, README, workflow, asset, 문서는 수정하지 않는다. 계약이
틀렸거나 허용 파일 밖 변경이 필요하면 추측 구현하지 말고 실패 보고서에 이유를 쓴다.

## 범위 밖 및 금지

- core source/license 문구 수정·요약본으로 대체, pin/patch 변경
- Rust Cargo dependency license 전수·SBOM, BaseOS license bundle
- README 설치 문서, issue template, migration guide
- `release.yml`, tag, GitHub Release 생성·업로드
- source archive에 build output, `.git`, 상용 ROM, local test ROM, 절대경로 포함
- vendor/dist/target 산출물 커밋
- 네트워크 fetch/clone/download. local pinned checkout과 git object만 사용한다.
- 실제 target checkout, 정상 dist, tracked license를 파괴해 negative test 수행
- 커밋, push, 실기·ADB·Samba·SD 카드 접근, 공용 설정 변경

## 완료 기준

마지막 code/content 변경 뒤 아래 원문을 실행한다. 테스트와 패키징은 작업자인 가재코드가 수행한다.
Codex가 다시 실행할 필요가 없도록 종료 코드·hash·마지막 결과 줄을 보고서에 남긴다.

```text
cargo fmt --all -- --check
powershell -NoProfile -ExecutionPolicy Bypass -File build/package-core-sources.ps1 -OutputDir target/task101-core-sources-a
powershell -NoProfile -ExecutionPolicy Bypass -File build/package-core-sources.ps1 -OutputDir target/task101-core-sources-b
powershell -NoProfile -ExecutionPolicy Bypass -File build/dist-device.ps1 -NoBuild -Zip
cargo test --workspace
cargo check -p slot2 --no-default-features --features device
cargo clippy --workspace --all-targets -- -D warnings
powershell -NoProfile -ExecutionPolicy Bypass -File build/dist-device.ps1
git diff --check
```

- 모든 명령 종료 코드 0, 모든 test binary failed 0 / 예상 밖 ignored 0, clippy warning 0.
- 두 packager output의 relative file list와 SHA-256이 exact equality.
- source archive/core license/recipe 각각 6개, mGBA patch 2개, archive/patch hash 일치.
- negative staging은 nonzero이며 이전 output marker/tree를 보존한다.
- `dist-device`와 새 zip에 동일한 license/source 구조가 있고 six core/meta gate도 통과한다.
- workspace passed는 Task100의 973보다 줄지 않는다.
- full dist 마지막 줄 `==> done`.
- 완료 기준 뒤 code/content 변경 금지. 변경했으면 영향받는 명령부터 다시 실행한다.

첫 응답은 최대 5분, 전체는 최대 45분 기다린다. 진행 확인은 로그 크기가 아니라 `git status --short`다.

## 결과 보고서

`C:\SLOT2\tasks\101-core-license-source-bundle.worker-result.md`에 다음을 기록한다.

- 성공/실패와 누적 호출 수
- root/upstream/core license 파일 목록과 pinned blob byte-equality 결과
- 여섯 core의 repo, pin, license 식별, archive/recipe/patch 경로
- packager preflight·staging·determinism·hash 계약
- local dist와 CI license/source parity
- negative staging 명령·종료 코드·이전 output 보존 근거
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored 수
- 두 deterministic output 및 final dist/zip의 file counts와 hashes
- 생성·수정 파일, 최종 검증 뒤 code/content 변경 여부
- Rust dependency notice/tag release 등 남은 범위
- 계약이 틀려 보인 부분이 있으면 해당 줄과 이유

실패해도 보고서를 남긴다. 두 번째 호출까지 실패하면 더 시도하지 않는다. 커밋하지 않는다.
