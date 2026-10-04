# Task 101 — 코어 라이선스와 corresponding source 묶음 (worker-result, 누적 2/2)

**최종 상태: 성공. 누적 호출 2/2 (두 번째이자 마지막 호출).**

1차 호출에서 라이선스·corresponding source 기능 전체를 구현했고(§1~§11), Codex 판정이
`build/package-core-sources.ps1`의 최종 output 교체가 실패 시 이전 bundle을 지울 수 있다는 결함을
지적했다. 2차 호출(`tasks/101-core-license-source-bundle-attempt2.md`)에서 그 교체를
candidate + rollback 방식으로 바꾸고, 교체 구간 실패를 주입해 이전 output 보존을 확인한 뒤,
완료 기준 9개 명령을 전부 다시 실행했다(§12).

- 최종 검증(2차): 9개 명령 전부 종료 코드 0, workspace **973 passed / 0 failed / 0 ignored**,
  clippy warning 0, full dist `==> done`.
- 두 packager output은 28 files / 40,524,361 bytes로 relative path·SHA-256이 **완전 동일**하며,
  1차 호출 때의 값(manifest `622b9a70…`, bundle digest `c6e0cf89…`)과도 같다 — 교체 방식 변경이
  산출물 바이트를 바꾸지 않았다.
- 최종 `dist-device` 55 files(그중 `System/licenses` 45), zip 47,121,707 bytes에 archive 6 +
  recipe 6 + mGBA patch 2 + core license 6 + `.so.meta` 6 + SOURCE-MANIFEST가 들어 있다.
- 교체 구간 실패 주입 2종(move-aside 실패, 승격 실패)과 preflight 실패 4종 모두 nonzero이며
  이전 output/marker tree가 byte 단위로 보존되고 candidate/backup 쓰레기가 남지 않았다(§12).
- 커밋·push·release·ADB·Samba·SD 카드·공용 설정 접근 없음, 위임 없이 직접 수행.

---

## 1. 고지 파일 목록과 pinned blob byte-equality

| 파일 | bytes | pinned blob과 byte 동일 | 비고 |
|---|---|---|---|
| `LICENSE` (신규) | 1,075 | — | 표준 MIT, `Copyright (c) 2026 SLOT2 contributors` (LF·BOM 없음) |
| `licenses/upstream-slot/LICENSE` | 1,097 | `C:\Users\gyuha\slot-2\LICENSE`와 byte 동일 | 원문 그대로(CRLF 21줄 포함, 재포맷하지 않음) |
| `licenses/cores/mgba/LICENSE` | 16,726 | E | `git show <pin>:LICENSE` |
| `licenses/cores/gambatte/COPYING` | 17,987 | E | `git show <pin>:COPYING` |
| `licenses/cores/gpsp/COPYING` | 18,092 | E | `git show <pin>:COPYING` |
| `licenses/cores/fceumm/Copying` | 18,010 | E | `git show <pin>:Copying` |
| `licenses/cores/snes9x/LICENSE` | 7,047 | E | `git show <pin>:LICENSE` |
| `licenses/cores/genesis_plus_gx/LICENSE.txt` | 63,857 | E | `git show <pin>:LICENSE.txt` |
| `CORE-NOTICES.md` (신규) | 7,805 | — | 고지·대응 소스 설명 (아래 §3) |

"E"는 `git -C <checkout> cat-file blob <pin>:<license>` 결과와 byte 동일을 뜻한다. 세 위치(추적
`licenses/cores/…`, packager가 만든 `…/licenses/<name>/<file>`, 최종 카드
`System/licenses/cores/<name>/<file>`)가 모두 같은 blob과 일치함을 확인했다. 복사본은 작업 트리가
아니라 **git object에서** 만들었다(여섯 체크아웃 모두 patches가 적용돼 genesis_plus_gx는 dirty).

## 2. 여섯 core의 repo·pin·license·archive·recipe

| core | repository (build script = `.so.meta`) | pin (`cores/<name>/commit`) | 라이선스 식별 | archive | recipe |
|---|---|---|---|---|---|
| mgba | https://github.com/libretro/mgba | `e31759b24e7a4e3899285ff720d7b573ac328ae7` | MPL-2.0 | `sources/archives/mgba-e31759b24e7a4e3899285ff720d7b573ac328ae7.zip` (18,767,861 B) | `sources/recipes/mgba/` (+patch 2) |
| gambatte | https://github.com/libretro/gambatte-libretro | `d9d6cd06382d1ced30de34d56d3609452323dab1` | GPL-2.0 전문 | `sources/archives/gambatte-d9d6cd06382d1ced30de34d56d3609452323dab1.zip` (1,180,238 B) | `sources/recipes/gambatte/` |
| gpsp | https://github.com/libretro/gpsp | `5819380c2ffb0900219d700a382ee68c464ebb99` | GPL-2.0 전문 | `sources/archives/gpsp-5819380c2ffb0900219d700a382ee68c464ebb99.zip` (536,440 B) | `sources/recipes/gpsp/` |
| fceumm | https://github.com/libretro/libretro-fceumm | `236ccdfc911e84c60fea6b9d0699c2d440a8de14` | GPL-2.0 전문 | `sources/archives/fceumm-236ccdfc911e84c60fea6b9d0699c2d440a8de14.zip` (1,665,446 B) | `sources/recipes/fceumm/` |
| snes9x | https://github.com/libretro/snes9x | `fae2fea08f74180759ef540ee94259213f503480` | Snes9x custom (비상업) | `sources/archives/snes9x-fae2fea08f74180759ef540ee94259213f503480.zip` (760,336 B) | `sources/recipes/snes9x/` |
| genesis_plus_gx | https://github.com/libretro/Genesis-Plus-GX | `c2838c7dc4236fc2fe94e5dbd08b41486067918e` | Genesis Plus GX custom (비상업) | `sources/archives/genesis_plus_gx-c2838c7dc4236fc2fe94e5dbd08b41486067918e.zip` (17,453,267 B) | `sources/recipes/genesis_plus_gx/` |

- repository URL은 `cores/<name>/build.sh`에서 읽고(`$`가 남은 로그 템플릿 URL은 건너뛴다 — mgba의
  `source=https://github.com/libretro/mgba/tree/$commit` 같은 줄), `vendor/<name>_libretro.so.meta`의
  `repo=` 또는 `source=<url>/tree/<pin>` 및 `commit=<pin>`과 일치함을 preflight에서 확인한다.
- `CORE-NOTICES.md`의 Snes9x·Genesis Plus GX 항목은 원문 문장을 그대로 인용해 식별만 한다:
  `Permission to use, copy, modify and/or distribute Snes9x … for non-commercial purposes`,
  `Snes9x is freeware for PERSONAL USE only. Commercial users should seek permission…`,
  `(Under no circumstances will commercial rights be given)`,
  `Redistributions may not be sold, nor may they be used in a commercial product or activity.`
  그리고 판매·상업 사용 제한을 눈에 띄게 안내하고 "원문 license가 최종 기준"이라고 적었다.

## 3. `CORE-NOTICES.md` 내용

SLOT2 frontend = root MIT / 원작 slot에서 이식한 부분 = `licenses/upstream-slot/LICENSE` + asset별
`PROVENANCE.md`(skins·sfx 명시) / 여섯 core 표(표시 이름, repo, full pin, 동봉 license 경로, archive,
recipe) / 라이선스 "식별 정보"만(축약·"오픈소스" 뭉뚱그리기 없음) / Snes9x·Genesis Plus GX 비상업
경고 블록 / archive는 pristine pinned source이고 `recipes/`의 patch·build script를 함께 적용해야
배포 binary에 대응한다는 구조 설명 / 폰트는 `System/licenses/fonts`, 스킨·효과음은 repository의
PROVENANCE 문서 / AI-assisted development(Claude Code, OpenAI Codex, GajaeCode/OpenCodex, B.AI 모델이
구현·검토에 쓰였고 최종 책임은 maintainers, AI가 저작권을 가진다는 서술 없음) / 법률 자문이 아니며
원문을 대체하지 않는다는 문장.

## 4. packager (`build/package-core-sources.ps1`)

- 입력: repo root와 `cores/required.txt`는 script 위치 기준(`$PSScriptRoot`)이고 `core-manifest.ps1`의
  `Get-CoreManifest`·`Get-CoreLicenseName`·`Get-CorePin`·`Get-CoreCheckout`을 재사용한다. `-OutputDir`
  은 상대경로면 **호출자 cwd** 기준으로 절대화한다.
- preflight(모두 archive 생성 전): git checkout을 `target-device/cores/<name>` 아래에서 실제로 찾아
  **정확히 하나**인지(`.git`이 있는 디렉터리; mgba만 `mgba/`, 나머지는 `src/` — 이름별 path table 없음),
  `git cat-file -e <pin>^{commit}`, `HEAD == pin`, `licenses/cores/<name>/<license>` 존재·비어 있지 않음,
  repo URL의 `.so.meta`/build script 일치, `cores/<name>/{commit,build.sh,*.patch}` 존재.
- staging: workspace `target/package-core-sources-<guid>/`에서 조립하고, 성공 시에만 `OutputDir`을
  교체한다(교체 전 기존 output 삭제, 실패 시 `finally`로 staging 정리). repo root·파일시스템 root·
  repo 상위를 OutputDir로 지정하면 거부한다.
- 출력 28 files: `SOURCE-MANIFEST.txt`, `archives/<name>-<pin>.zip` ×6, `licenses/<name>/<license>` ×6,
  `recipes/common.sh` + `recipes/<name>/{commit,build.sh,*.patch}` ×6(mgba만 patch 2).
- archive는 `git archive --format=zip --prefix=<name>-<pin>/ <pin>` 전체 tree(pristine: build output·
  `.git`·적용 패치·working tree line-ending noise 없음). **`-c core.autocrlf=false -c core.eol=lf`를
  강제한다** — 이걸 빼면 이 호스트에서 archive가 CRLF로 변환돼 pinned blob이 아닌 소스가 들어간다(§9-1).
- 라이선스는 `git cat-file blob`을 raw stdout으로 받아 byte 그대로 쓴다(PowerShell 파이프라인은 텍스트로
  디코드해 줄바꿈을 바꾼다).
- `SOURCE-MANIFEST.txt`는 UTF-8 BOM 없이 LF, key=value 블록(빈 줄 구분), manifest 순서, 실행 시각·
  절대경로·machine name 없음(백슬래시·드라이브 경로를 validator가 거부). patch는 `patch=<file> sha256:<hex>`.
- 두 번 실행 결과(§8): relative path 집합과 모든 파일 SHA-256이 동일 → archive bytes와 manifest hash가
  결정적이다.

## 5. local dist와 CI parity

- `build/dist-device.ps1`: core preflight 뒤, **기존 `dist-device` 삭제 전에** packager를 실행해
  `target/core-sources`를 만들고 `Assert-SourceBundle`로 검증한다(실패하면 이전 tree 보존). 그 뒤
  `System/licenses/{SLOT2-LICENSE,CORE-NOTICES.md,upstream-slot/LICENSE,fonts/,cores/<name>/<license>,
  sources/,<name>_libretro.so.meta}`를 조립하고 postflight로 `Assert-CoreTree`(기존 exact-six core/meta)와
  `Assert-LicensesTree`(notices·6 core license·6 archive·6 recipe·2 patch·manifest hash·tracked vs
  generated license byte 동일)를 통과한 뒤에만 zip/ADB로 간다. `-Adb`는 기존 `System/licenses/.` push로
  새 tree가 함께 올라가므로 별도 실기 명령을 추가하지 않았다.
- CI device job: core build 뒤 `pwsh -NoProfile -File build/package-core-sources.ps1 -OutputDir
  "$PWD/target/core-sources"`를 실행하고, assembly에서 같은 tree를 만든 뒤 exact-six core/meta 검증 +
  `diff -r licenses/cores …/sources/licenses`, `diff cores/common.sh …/recipes/common.sh`,
  core별 `diff -r cores/<c> …/recipes/<c>`, archive nonempty, manifest의 `archive_sha256`과
  `sha256sum` 비교, mGBA patch 2개를 검사한다. 기존 exact-six·artifact upload는 유지하고
  `release.yml`은 만들지 않았다.
- 로컬 검증: CI assembly 스텝 본문을 추출해 임시 output으로 그대로 실행 → exit 0, cores 6 / licenses
  `SLOT2-LICENSE,CORE-NOTICES.md,upstream-slot,cores,fonts,sources,*.meta` / sources
  `SOURCE-MANIFEST.txt,archives(6),licenses(6),recipes(6+common.sh)` 확인. 두 빌드 스텝은 실행하지 않았고
  (동일 내용이 Task100에서 검증됨), YAML 자체는 parser 부재로 기계 검증하지 못했다(Task100과 동일).

## 6. 완료 기준 명령 (1차 호출, 2026-09-30 21:11:42–21:25:48)

| # | 명령 | 종료 | 마지막 결과 줄 | passed/failed/ignored |
|---|---|---|---|---|
| 1 | `cargo fmt --all -- --check` | 0 | (출력 없음) | — |
| 2 | `powershell … package-core-sources.ps1 -OutputDir target/task101-core-sources-a` | 0 | `==> wrote 28 files (40524361 bytes) to C:\SLOT2\target\task101-core-sources-a` | — |
| 3 | `powershell … -OutputDir target/task101-core-sources-b` | 0 | `==> wrote 28 files (40524361 bytes) to C:\SLOT2\target\task101-core-sources-b` | — |
| 4 | `powershell … dist-device.ps1 -NoBuild -Zip` | 0 | `==> done` (`==> zipping dist\slot2-0.1.0-a8cb4af.zip` 뒤) | — |
| 5 | `cargo test --workspace` | 0 | 결과 줄 100개. 첫 줄 `ok. 33 passed; … in 0.57s`, 마지막 줄(doc-test) `ok. 0 passed; 0 failed; 0 ignored; … in 0.00s` | **973/0/0** |
| 6 | `cargo check -p slot2 --no-default-features --features device` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.47s`` | — |
| 7 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 12.34s`` | — |
| 8 | `powershell … dist-device.ps1` | 0 | `==> done` (frontend 재빌드 + 여섯 core current + packager 재실행) | — |
| 9 | `git diff --check` | 0 | (whitespace 진단 없음. stderr에 CRLF 경고 13줄 — `core.autocrlf=true` 상태, 오류 아님) | — |

- 모든 test binary failed 0, 예상 밖 ignored 0. workspace passed 973은 Task100과 동일(감소 없음).
- packager 각 실행은 `==> bundle checked (6 cores)`와 `==> wrote …`로 끝나고, 그 사이 `Assert-SourceBundle`이
  checkout/pin/license/hash/recipe를 전부 재검증한다.

## 7. two deterministic output / final dist·zip counts·hashes

두 output(`target/task101-core-sources-a`, `-b`)은 **relative path 28개와 모든 파일 SHA-256이 동일**하다.
총 40,524,361 bytes(파일 28개).

| 항목 | SHA-256 |
|---|---|
| `SOURCE-MANIFEST.txt` (두 output 동일) | `622b9a70f3b52e846b54e270b11c020898ce4b812c8172bf8e7765fbba42133d` |
| bundle digest (정렬한 `path sha256` 목록의 SHA-256) | `c6e0cf89e9a70d702a76a078f5de77368aef580917fbae623f291a109bc4c64c` |
| archives/fceumm-236ccdfc…zip | `a1d03521ab88966bc415ba66afdf27debdeb2ab0c58c99e8af0cb890759bf1b8` |
| archives/gambatte-d9d6cd06…zip | `948572a6d7ee53d8604176e18cadf6ff9726f3c4e329bc8821ef231575ddbce5` |
| archives/genesis_plus_gx-c2838c7d…zip | `7b851571aaf1873e07eba8546599d2d66e761fe476648c075067e7d19606bdd8` |
| archives/gpsp-5819380c…zip | `26814482a144e459c676d1a45511dceb1df3c72388ed4f9fcf8ba09b91d91381` |
| archives/mgba-e31759b2…zip | `aab7bc7bf731559b74d82413710b5b4da681e6446d35b4fc3453422a9c99f4de` |
| archives/snes9x-fae2fea0…zip | `a16b1ca45bf9279704f172335ef81c41eec0ce649a6a7503c7382628397b516a` |
| licenses/mgba/LICENSE | `fab3dd6bdab226f1c08630b1dd917e11fcb4ec5e1e020e2c16f83a0a13863e85` |
| licenses/gambatte/COPYING | `ab15fd526bd8dd18a9e77ebc139656bf4d33e97fc7238cd11bf60e2b9b8666c6` |
| licenses/gpsp/COPYING | `8177f97513213526df2cf6184d8ff986c675afb514d4e68a404010521b880643` |
| licenses/fceumm/Copying | `a6996dcf0c334281f734560926e079b2dbbd5b78e81c0ca00a413ec01e1cd2fb` |
| licenses/snes9x/LICENSE | `70efeee282d82a6e9d26aeed5466d08c632369858371dc6a4644c8dcedc2be78` |
| licenses/genesis_plus_gx/LICENSE.txt | `642c163624269243d1f6b29d759d4e3a2d161bdc272c90d82ecbeec82ae26755` |

- 최종 `dist-device`: 파일 55개, 그중 `System/licenses/` 아래 45개 — archive 6, recipe file 15
  (common.sh + 6×(build.sh, commit) + patch 2), core license 6, `.so.meta` 6. `System/cores` 정확히 6.
  frontend 3,061,968 B, `VERSION.txt` = `SLOT2 0.1.0 (a8cb4af)`.
- 최종 zip `dist/slot2-0.1.0-a8cb4af.zip` 47,121,707 B, entries 57(디렉터리 entry 2개
  `System/licenses/cores/`, `System/licenses/sources/licenses/` 포함): cores 6, archives 6, core licenses 6,
  recipe files 15, patches 2, 그 밖에 frontend·VERSION·fonts·licenses. dist-device와 모든 `System/…`
  파일이 byte 동일(differences none). 참고로 기존 `dist/slot2-0.1.0-0d3dcd3.zip`(core 0개, Task100 이전
  산출물)은 그대로 두었다.

## 8. negative staging (실제 checkout/license/dist 파괴 없음)

staging repo: `%TEMP%\slot2-lic-gate\`에 `build/{core-manifest,package-core-sources,dist-device}.ps1`,
`cores/`(required.txt·common.sh·각 commit/build.sh/patch), `licenses/`(upstream-slot + cores 6),
`LICENSE`, `CORE-NOTICES.md`, `vendor/*.so.meta` 복사본. 실제 checkout은 **junction**으로 연결
(`target-device/cores/<name>` → 실제 경로, 읽기만). 이전 정상 output 대표로
`target/negative-out/{marker.txt,keep.txt}`, 이전 정상 카드 대표로 `dist-device/{marker.txt,frontend}`.

| 시나리오 | 명령 | 종료 | 결과 |
|---|---|---|---|
| tracked license 없음 (gpsp) | staged `package-core-sources.ps1 -OutputDir …/negative-out` | 1 | `no tracked license at …\licenses\cores\gpsp\COPYING`, marker/tree hash 불변 |
| tracked license가 pinned blob과 다름 | 동일 | 1 | `…:COPYING differs: 23 bytes against 18092`, 보존 |
| checkout 없음 (mgba junction 제거) | 동일 | 1 | `no checkout for mgba under …\target-device\cores\mgba — run build/cores.ps1 -DeviceOnly`, 보존 |
| tracked license 없음 (snes9x) + `dist-device.ps1 -NoBuild` | staged `dist-device.ps1 -NoBuild` | 1 | 이전 `dist-device`(marker·frontend) hash 불변, 삭제되지 않음 |

네 경우 모두 OutputDir/`dist-device`의 파일 목록과 SHA-256이 실행 전후 동일했다. 실제
`target-device/cores/*`, `licenses/cores/*`, 정상 `dist-device`는 rename/delete하지 않았다.

## 9. 생성·수정 파일과 검증 뒤 변경 여부

- 신규: `LICENSE`, `CORE-NOTICES.md`, `licenses/upstream-slot/LICENSE`, `licenses/cores/<six>/<license>`,
  `build/package-core-sources.ps1`, `tasks/101-core-license-source-bundle.worker-result.md`.
- 수정: `build/core-manifest.ps1`(Task100 API 그대로 두고 license/source helper 추가),
  `build/dist-device.ps1`(packager 배선 + licenses tree 조립/검증), `.github/workflows/ci.yml`(device job),
  `docs/DESIGN.md`(§6 코어 빌드 문단 + §9 카드 layout), `docs/MILESTONES.md`(M7 두 항목 진행 메모, `[ ]` 유지).
- 산출물(비추적, gitignore): `target/task101-core-sources-{a,b}`, `target/core-sources`, `dist-device/`(재조립),
  `dist/slot2-0.1.0-a8cb4af.zip`.
- **최종 검증 뒤 code/content 변경 없음**: 검증 시작 21:11:42 이후 바뀐 파일은 검증 자체의 산출물
  (`target/**`, `dist/slot2-0.1.0-a8cb4af.zip`, `dist-device/System/**`)과 테스트가 다시 써낸 비추적
  `assets/overlays/GB/720x720.png`(기존 동작)뿐이다. 새 파일은 모두 LF이고 `licenses/upstream-slot/LICENSE`
  만 원본 그대로 CRLF를 유지한다(contract가 byte-identical copy를 요구).
- 참고: 이 checkout은 `core.autocrlf=true`이고 Task57~100의 미커밋 변경이 쌓여 있어 `git status`에
  많은 파일이 M/??로 보인다(예: `cores/common.sh`는 Task57의 stamp 확장, `licenses/`·`build/core-manifest.ps1`
  은 이전 태스크 산출물). 이번 태스크에서 손댄 것은 위 목록이 전부이고, `licenses/`·`LICENSE`·
  `CORE-NOTICES.md`·`build/package-core-sources.ps1`는 ignore되지 않는 추적 대상이다.

## 10. 남은 범위

- Rust crate 의존성 전수 고지/SBOM(이 태스크 범위 밖으로 명시됨), 최종 README(en/ko), issue template,
  migration guide, `release.yml`·tag·GitHub Release. `CORE-NOTICES.md` 마지막 절에도 "아직 없다"고 적었다.

## 11. 계약이 틀려 보이거나 다르게 구현한 부분

1. **`git archive`는 pristine하지 않을 수 있다 — 가장 중요한 발견.** 이 호스트는 `core.autocrlf=true`라
   그냥 `git archive`를 쓰면 내보내는 파일의 line ending이 변환된다. 실측: mgba `LICENSE`의 pinned
   blob은 16,726 bytes인데 첫 구현의 archive 안 entry는 **17,099 bytes**(373줄 × CRLF)였다. 계약 문구
   (`git archive …`로 pristine pinned source)만 그대로 따르면 조용히 다른 소스가 동봉된다. packager는
   `-c core.autocrlf=false -c core.eol=lf`로 강제하고, validator가 **archive 안 license entry와 pinned
   blob의 byte 동일성**을 검사해 이 변환이 다시 생기면 실패하게 했다.
2. packager가 `vendor/<name>_libretro.so.meta`를 요구한다(계약은 manifest의 URL이 `.so.meta`/build
   script와 일치해야 한다고만 적었다). 그래야 URL 대조가 실재하고, 어차피 checkout이 필요하므로 core
   build 없이는 못 도는 것은 같다. 메타가 없으면 core 이름·경로를 찍고 실패한다.
3. `-OutputDir`의 상대경로는 **호출자 cwd** 기준으로 해석한다(계약은 "absolute-or-relative"만 명시).
   repo root 기준 상대경로로 임의 해석하지 않는다.
4. `CORE-NOTICES.md`의 pin은 `cores/<name>/commit` 값을 옮겨 적은 것이라 pin을 움직이면 문서가
   낡을 수 있다(계약이 문서에 full pin을 요구한다). 자동 검증 대상이 아니며, 단일 source of truth는
   여전히 `cores/<name>/commit`과 `SOURCE-MANIFEST.txt`다.
5. `System/licenses/sources/licenses/`(bundle의 licenses)와 `System/licenses/cores/`(추적 복사본)가 같은
   blob을 두 번 담는다. 계약의 tree에 둘 다 있고 byte equality를 요구하므로 그대로 두었고, 각각
   40 KB 미만이다.
6. CI의 `read` 빌트인이 이 호스트 셸에 없어(Task100에서 확인) CI loop는 계속
   `for c in $(tr -d '\r' < cores/required.txt)` 형태로 썼고, hash 대조는 `grep -A1`+`sed`+`sha256sum`으로
   pairing까지 검사한다(로컬 실행으로 검증). YAML 기계 파싱은 pyyaml 부재·네트워크 금지로 하지 못했다.
7. 파일럿 성격의 두 사실: ①`git archive`는 `export-ignore`/`export-subst` attribute를 존중하므로(이는
   "pinned source"의 정의 그대로다) 해당 attribute가 있는 파일은 archive에서 빠질 수 있다 — 이 여섯
   저장소에서는 top-level license가 모두 archive에 들어 있음을 확인했다. ②`git cat-file blob`의 exit
   code를 `Process.Dispose()` 뒤에 읽으면 성공한 실행도 실패로 보인다(구현 중 발견해 수정).

---

## 12. attempt 2 — output 교체의 트랜잭션 보장 (누적 2/2)

### 12.1 결함과 원인

1차 packager의 마지막 구간은 `if (Test-Path $OutputDir) { Remove-Item -Recurse -Force }` →
`New-Item $OutputDir` → `Copy-Item $stage/*` 순서였다. 검증된 staging을 다 만든 뒤라도, 복사가
디스크·권한·I/O 오류로 중간에 실패하면 **직전 정상 bundle은 이미 삭제된 뒤**이고 부분 output만 남는다.
1차 보고서의 negative 4종은 모두 이 구간보다 앞선 preflight 실패라서 이 위험을 건드리지 못했다.

### 12.2 수정 내용 (`build/package-core-sources.ps1`만 변경)

최종 교체를 **같은 parent의 완성 candidate + rollback 가능한 directory rename**으로 바꿨다.

```
$parent = Split-Path -Parent $OutputDir          # repo 밖·다른 volume이어도 이 parent에 만든다
$candidate = $parent\.<leaf>.candidate-<guid>
$backup    = $parent\.<leaf>.backup-<guid>
Copy-Item $stage -> $candidate                    # 기존 OutputDir은 손대지 않는다
Assert-SourceBundle -Bundle $candidate            # candidate 자체를 승격 전에 재검증
[System.IO.Directory]::Move($OutputDir, $backup)  # 기존 bundle을 옆으로 (원자적 rename)
try   { [System.IO.Directory]::Move($candidate, $OutputDir) }   # 승격 (원자적 rename)
catch { if ($replaced) { [System.IO.Directory]::Move($backup, $OutputDir) }; throw }  # rollback
finally { candidate·backup 삭제 }                 # 성공·실패 모두 쓰레기 없음
```

- 기존 `OutputDir`은 candidate 복사와 candidate 검증이 **모두 끝날 때까지** 건드리지 않는다.
- candidate는 항상 `OutputDir`의 parent에 만들어지므로 최종 rename이 같은 filesystem 안에서 일어난다
  (repo 밖·다른 volume의 OutputDir도 `Copy-Item`이 아니라 그 위치에서 만들고 거기로 rename한다).
- repo root·filesystem root·repo 상위 보호, deterministic bytes, archive/license/recipe/hash 계약은
  그대로다. 테스트 전용 switch·환경변수는 추가하지 않았다.

**수정 중 발견한 두 번째 결함**: 처음 구현은 PowerShell `Move-Item`으로 옮겼는데, `Move-Item`은
디렉터리를 **한 번의 rename으로 옮기지 않는다.** 자식 단위로 처리하다 실패하면 일부만 옮겨진 상태로
끝난다. 주입 실험(수정 전 코드, 기존 output 안의 `marker.txt`를 다른 프로세스가 연 상태로 실행):
exit 1로 실패했지만 이전 tree가 보존되지 않았다 — `keep.txt`가 backup으로 옮겨진 뒤 `finally`의 backup
삭제로 사라져 남은 파일은 `marker.txt`뿐이었다. 같은 실험에서 `[System.IO.Directory]::Move`
(`MoveFile`)는 예외만 던지고 원본 디렉터리를 **모든 자식과 함께** 그대로 남긴다(별도 probe: 잠긴 자식이
있을 때 IOException + `x`에 `a.txt`·`locked.txt` 모두 보존, 목적지 미생성). 그래서 이동 3곳을
`[System.IO.Directory]::Move`로 통일했다.

### 12.3 검증 방법과 결과 (안전한 staging)

실제 `target/core-sources`, `dist-device`, checkout, tracked license는 rename/delete하지 않았다.
staging(`%TEMP%\slot2-lic-gate`)은 실제 checkout과 `vendor`를 junction으로 **읽기만** 하고(스크립트는 이
둘을 수정하지 않는다), tracked license·`LICENSE`·`CORE-NOTICES.md`·`cores/*`는 실제 저장소에서 복사한
사본이다. 각 실패 주입 뒤에는 손상시킨 사본을 원래 바이트로 되돌렸다.

| # | 주입/시나리오 | 방법 | 종료 | 결과 |
|---|---|---|---|---|
| 1 | 정상 교체 (기존 marker tree 위) | `-OutputDir …\negative-out`을 연속 2회 실행 | 0 | 28 files, 옛 marker 사라짐, candidate/backup 없음, manifest sha256 `622b9a70…`(실제 bundle과 동일) |
| 2 | **move-aside 실패** | 기존 output 안 `marker.txt`를 다른 프로세스가 연 채 실행 | 1 | 이전 output(marker.txt+keep.txt) 파일·hash 완전 동일, candidate/backup 없음 |
| 3 | **승격 실패** | candidate 안 `SOURCE-MANIFEST.txt`를 연 채 실행(폴링으로 candidate 포착) | 1 | 이전 marker tree 복구·hash 동일, backup 없음; candidate는 **주입한 handle 때문에** 남았다가 handle을 놓은 뒤 정리(스크립트가 남긴 쓰레기 아님) |
| 4 | preflight: tracked license 누락 (gpsp) | staging에서 파일 제거 | 1 | `no tracked license at …`, 이전 output 보존, 쓰레기 없음 |
| 5 | preflight: tracked license 변조 (gpsp) | staging 파일에 다른 바이트 기록 | 1 | `…:COPYING differs: 23 bytes against 18092`, 보존 |
| 6 | preflight: checkout 누락 (mgba) | junction 제거 | 1 | `no checkout for mgba under … — run build/cores.ps1 -DeviceOnly`, 보존 |
| 7 | dist-device: core license 누락 | staged `dist-device.ps1 -NoBuild` | 1 | `no tracked license at …`, 이전 `dist-device`(marker.txt+frontend) hash 동일 |

2차 종료 시점에 `target/`·`dist/`·`dist-device/` 어디에도 `.candidate-*`/`.backup-*`가 없다.

### 12.4 아홉 명령 재실행 (2차, 2026-09-30 23:49:27 – 2026-10-01 00:17:47)

| # | 명령 | 종료 | 마지막 결과 줄 | passed/failed/ignored |
|---|---|---|---|---|
| 1 | `cargo fmt --all -- --check` | 0 | (출력 없음) | — |
| 2 | `powershell … package-core-sources.ps1 -OutputDir target/task101-core-sources-a` | 0 | `==> wrote 28 files (40524361 bytes) to C:\SLOT2\target\task101-core-sources-a` | — |
| 3 | `powershell … -OutputDir target/task101-core-sources-b` | 0 | `==> wrote 28 files (40524361 bytes) to C:\SLOT2\target\task101-core-sources-b` | — |
| 4 | `powershell … dist-device.ps1 -NoBuild -Zip` | 0 | `==> done` | — |
| 5 | `cargo test --workspace` | 0 | 결과 줄 100개. 첫 줄 `ok. 33 passed; … in 0.72s`, 마지막 줄(doc-test) `ok. 0 passed; 0 failed; 0 ignored; … in 0.00s` | **973/0/0** |
| 6 | `cargo check -p slot2 --no-default-features --features device` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.83s`` | — |
| 7 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 13.08s`` (warning 0줄) | — |
| 8 | `powershell … dist-device.ps1` | 0 | `==> done` | — |
| 9 | `git diff --check` | 0 | (whitespace 진단 없음. stderr에 CRLF 경고 13줄) | — |

추가 확인:
- 두 packager output: 28 files / 40,524,361 bytes, relative path 집합과 모든 SHA-256 **동일**,
  manifest sha256 `622b9a70f3b52e846b54e270b11c020898ce4b812c8172bf8e7765fbba42133d`,
  bundle digest `c6e0cf89e9a70d702a76a078f5de77368aef580917fbae623f291a109bc4c64c` — **1차 호출과 같은 값**.
- 최종 `dist-device`: 55 files / 76,544,488 bytes (그중 `System/licenses` 45: archive 6, recipe 15,
  core license 6, `.so.meta` 6, 기타 12), `System/cores` 정확히 6, `VERSION.txt` = `SLOT2 0.1.0 (a8cb4af)`.
- 최종 zip `dist/slot2-0.1.0-a8cb4af.zip` 47,121,707 bytes / entries 57(dir 2): cores 6, archives 6,
  core licenses 6, recipe files 15, patches 2. 표본 6개(core `.so`, SLOT2-LICENSE, CORE-NOTICES.md,
  SOURCE-MANIFEST.txt, snes9x LICENSE, mgba patch)의 zip entry SHA-256이 `dist-device` 파일과 동일.
- 라이선스 재확인(수정 뒤): 여섯 core의 추적/generated/카드 복사본 모두 `git cat-file blob <pin>:<path>`와
  byte 동일, `licenses/upstream-slot/LICENSE`는 원본과 동일.

### 12.5 최종 변경 파일과 검증 뒤 변경 여부

- 2차 호출에서 바뀐 code/content는 `build/package-core-sources.ps1` 하나와 이 보고서뿐이다. `LICENSE`,
  `CORE-NOTICES.md`, `licenses/**`, `build/core-manifest.ps1`, `build/dist-device.ps1`, `ci.yml`,
  `docs/DESIGN.md`, `docs/MILESTONES.md`는 1차 상태 그대로다(마지막 write 20:50–21:05, 2차 검증 시작
  23:49:27보다 앞선다).
- **검증 뒤 code/content 변경 없음**: 검증 시작(2026-09-30 23:49:27) 이후 mtime이 바뀐 것은 검증 자체의
  산출물(`target/**`, `dist/slot2-0.1.0-a8cb4af.zip`, `dist-device/System/**`)과 테스트가 다시 써낸 비추적
  `assets/overlays/GB/720x720.png`(기존 동작)뿐이다. `build/package-core-sources.ps1`의 마지막 write는
  23:38:00이다.

### 12.6 남은 한계 / 위험

- **삭제는 원자적일 수 없다**: candidate/backup 정리는 `Remove-Item -Recurse -Force`인데, 외부
  프로세스가 그 안의 파일을 연 채로 잡고 있으면 OS가 삭제를 거부해 숨은 `.candidate-*`/`.backup-*`가
  남을 수 있다(주입 3의 관측된 현상). 이 경우에도 **현재 `OutputDir`의 bundle은 온전하고 이전 bundle을
  잃지 않는다** — 남는 것은 이름이 `.`로 시작하는 중간 산출물뿐이다. 열린 handle은 이 스크립트가
  통제할 수 없다.
- 실제 다른 volume의 `OutputDir`에서는 candidate가 그 volume의 parent에 만들어져 rename이 같은
  filesystem 안에서 일어난다(설계 의도). 별도 volume을 마운트해 실측하지는 않았다 — 로직상 보장이고,
  이 호스트에서는 확인할 수 없는 부분이다.
- `rc` 같은 Windows reparse point가 `OutputDir` 자리에 있는 경우(깨진 junction 등) `Test-Path`가
  false를 돌려주어 "기존 output 없음"으로 판단한다. 이 실험에서 promotion이 junction을 실제 디렉터리로
  대체해 exit 0으로 끝나는 것을 관측했다 — 일반적인 bundle 경로에서는 생기지 않지만, 링크가 섞인
  경로를 `-OutputDir`로 지정하는 것은 의도된 용법이 아니다.
- 나머지 남은 범위는 §10과 같다(Rust dependency 전수 고지/SBOM, README, `release.yml`·tag release).

### 12.7 하네스 잔여물

- 하네스가 만든 scratch 스크립트(`target/task101-{probe,probe2,move-probe,rename-probe,stage-setup}.ps1`)는
  검증 뒤 `target/`(gitignore된 빌드 디렉터리)에서 삭제했다. `target/task101-core-sources-{a,b}`와
  `target/core-sources`는 완료 기준 명령의 산출물이라 그대로 두었다.
- staging 하네스 자체(`%TEMP%\slot2-lic-gate`)는 **일부러 남겨 두었다**: 그 안의 `target-device/cores/*`와
  `vendor`는 실제 저장소를 가리키는 junction이라, `shutil.rmtree`나 `rd /s`로 통째로 지우면 junction을
  따라가 실제 pinned checkout을 지울 수 있다. 지우려면 junction을 `rmdir <link>`로 하나씩 끊은 뒤
  디렉터리를 지워야 하는데, 임시 폴더 정리를 위해 실제 checkout을 잃을 위험을 감수하지 않았다.
  이 저장소 밖 폴더는 빌드·테스트·배포 어느 경로에도 영향이 없다.
