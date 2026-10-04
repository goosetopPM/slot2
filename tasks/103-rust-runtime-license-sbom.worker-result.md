# Task 103 worker result — Rust 런타임 notice/SBOM

## 1. 판정

**성공 (누적 시도 2/2, 최종).** 1차에서 발견 기계와 packager를 만들고, 계약의 “packaged text가 없으면
실패” 규칙 때문에 세 crate에서 차단됐다(원인과 정확한 패키지는 §2~§9에 보존). 2차에서 Codex가 지시한

- **declared-only 정책**(텍스트를 동봉하지 않는 crate를 투명하게 포함),
- **source-aware identity**(name+version만으로 접는 결함 수정),
- 나머지 Task 103 통합(dist/zip/CI/문서)

을 모두 구현했고, 완료 명령 4개가 종료 코드 0으로 끝났다. 번들 186개 파일 / 769,598 bytes, 66 패키지
(63 `packaged-text` + 3 `declared-only`), A/B byte 동일, dist·zip·CI 조립/검증 연결 완료.

- 1차 호출(§2~§9): 발견·검증 라이브러리와 packager 구현, preflight 보존·오프라인 실패 검증.
- 2차 호출(§10): 정책·identity 수정, 번들 생성, negative 6종, dist/zip/CI/문서 통합, 명령 4개 재실행.

## 2. 수정한 파일과 구현 (1차 호출)

| 파일 | 내용 |
|---|---|
| `build/rust-notices.ps1` (신규, 638줄) | 발견·생성·검증 라이브러리. `cargo tree`/`cargo metadata`를 offline으로 읽어 패키지 집합을 구하고, 번들 레이아웃을 쓰고, `Assert-RustNoticeBundle` postflight로 다시 검증한다 |
| `build/package-rust-notices.ps1` (신규) | `-OutputDir <dir>` CLI. 저장소 root는 `$PSScriptRoot`에서, 상대 OutputDir은 호출자 cwd에서 해석, repository/filesystem root 거부, 일반 디렉터리가 아닌 output 거부, staging에 완성·검증한 뒤 rename으로 교체(이전 출력은 유지) |

**발견 규칙 (하드코딩 없음, 네트워크 없음, 오프라인 전용):**

```text
cargo tree -p slot2 --edges normal --target aarch64-unknown-linux-gnu \
           --no-default-features --features device --prefix none --offline
cargo tree -p slot2 --edges normal --target aarch64-unknown-linux-gnu \
           --no-default-features --features device --prefix none \
           --format "{p}|{l}|{r}" --offline
cargo metadata --format-version 1 --offline --no-default-features \
           --features slot2/device --filter-platform aarch64-unknown-linux-gnu
```

- closure는 `cargo tree`를 **두 번** 다른 emission으로 읽고 각각 독립 파서로 정규화해 서로 같은지
  비교한다(하나라도 다르면 실패). `cargo metadata`는 workspace 전체를 feature 통합해 해석하므로
  superset이다(host 전용 winit/X11/wayland 계열이 들어온다) — 그래서 집합 비교에는 쓰지 않고,
  `license-file`·`repository`·`homepage`·추출된 package 디렉터리만 여기서 가져온다. closure에 있는데
  metadata에 없는 패키지는 실패다.
- 제외: path/workspace 패키지(SLOT2 자신), dev·build edge, 그리고 device selection에서 실제로
  링크되지 않는 패키지(자동으로 빠진다). 포함: registry/git 패키지.
- 각 패키지: name, version, 정규화 source, Cargo.lock checksum, 선언 license 식, license-file, repository,
  homepage, 복사한 notice 경로. notice 파일은 선언된 `license-file` + package root의 top-level
  `LICENSE*|LICENCE*|COPYING*|COPYRIGHT*|NOTICE*`(대소문자 무시, 비어 있지 않은 것)만, 바이트 그대로.
- stable key: `name-version`에서 `[^A-Za-z0-9._-]`를 `_`로 치환(`proc-macro-hack-0.5.20_deprecated`),
  **같은 name+version이 서로 다른 source에서 두 번 나올 때만** `-s<source sha256 앞 8자>`를 덧붙인다.
  이번 closure에서는 충돌 0건이라 접미사가 붙지 않는다. 경로 탈출·절대 경로·중복 출력 경로는 거부.

> 부수 발견: `cargo metadata --features`는 virtual workspace에서 `-p`를 받지 않는다. `--features
> slot2/device` 형태로만 device selection을 지정할 수 있다(구현에 반영).

## 3. 발견 결과 (오프라인, 실제 closure)

| 항목 | 값 |
|---|---|
| third-party 패키지 | **66** (workspace crate 10개 제외) |
| 그중 라이선스 텍스트 동봉 | 63 (복사 대상 notice 파일 **117**개) |
| **거부: metadata는 있으나 동봉 텍스트 0개** | **3** — `fluent-langneg-0.13.1`, `gl-0.14.0`, `intl_pluralrules-7.0.2` |
| metadata도 license도 없음 | 0 |
| stable key 충돌 | 0 |
| `Cargo.lock` SHA-256 | `dce2d3bfd41a666f10138bfcbf7d4e1bb2a87e84ef70eb86136a0d94f31de24e` |
| 거부 3개의 선언 license | `Apache-2.0 OR MIT` / `Apache-2.0` / `Apache-2.0/MIT` |

거부된 3개 패키지의 실제 상태(모두 `%USERPROFILE%\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\`
아래, `Cargo.toml`에 `license` 선언, 동봉 파일은 README·src·benches 등뿐):

```text
fluent-langneg 0.13.1 - declared 'Apache-2.0 OR MIT', packaged source ...\fluent-langneg-0.13.1,
                        repository https://github.com/projectfluent/fluent-langneg-rs
gl 0.14.0            - declared 'Apache-2.0', packaged source ...\gl-0.14.0,
                        repository https://github.com/brendanzab/gl-rs/
intl_pluralrules 7.0.2 - declared 'Apache-2.0/MIT', packaged source ...\intl_pluralrules-7.0.2,
                        repository https://github.com/zbraniecki/pluralrules
```

이 3개를 뺀 63개의 key·version·license 식·notice 파일 목록은
`C:\SLOT2\target\task103-negative\inventory.txt`에 있다(예: `adler2-2.0.1` = `0BSD OR MIT OR Apache-2.0`
+ `LICENSE-0BSD, LICENSE-APACHE, LICENSE-MIT`; `unicode-ident-1.0.26` =
`(MIT OR Apache-2.0) AND Unicode-3.0` + `LICENSE-APACHE, LICENSE-MIT, LICENSE-UNICODE`).

## 4. 왜 여기서 멈추는가 (계약 문장 vs 실제 데이터)

계약 Task 103의 두 문장이 이번 입력에서 서로 충돌한다.

1. "At least one original nonempty license/notice file is required for every third-party package."
2. "If a package has metadata but no packaged text, fail and report the exact package instead of
   inventing or downloading a license."

로컬에 존재하는 원문은 위 3개에 대해 **없다**(crate tarball에 없음, 같은 저장소의 다른 crate 텍스트를
빌려오는 것은 "그 패키지의 원문"이 아니다, 네트워크 금지). 그래서 (2)를 그대로 구현했고, 실패 메시지가
세 패키지를 **전부** 이름·선언 license·package 경로·repository와 함께 보고한다.

`AGENTS.md`의 "잘못된 계약은 수정으로 숨기지 말고 보고한다"에 따라 우회 구현을 제품 코드에 넣지 않았다.

## 5. 검증한 것 / 검증할 수 없는 것

검증한 것 (staging·임시 산출물만 사용, 실제 Cargo cache·Cargo.lock·코어 checkout·추적 license 무수정):

| 항목 | 결과 |
|---|---|
| 이전 output이 있는 상태에서의 preflight 실패 (계약 negative 5의 형태) | exit 1, 이전 output의 `marker.txt` SHA-256 **불변**, 파일 수 1→1, staging/backup 잔여물 **0** |
| 거부 패키지 보고 완전성 | 세 패키지 이름이 모두 메시지에 나타남 |
| 로컬 Cargo cache가 빈 상태(`CARGO_HOME`을 빈 디렉터리로) | exit 1, `error: no matching package named 'png' found` + 실행한 명령 + `cargo fetch --locked --target aarch64-unknown-linux-gnu` 안내. output 디렉터리 **생성되지 않음**, 자동 fetch 없음 |
| stable key 충돌 | 0건, 66 키 모두 유일·경로 안전 |

검증할 수 **없는** 것(번들이 존재하지 않으므로): `THIRD-PARTY-RUST.md`/`RUST-SBOM.json`/
`RUST-MANIFEST.txt`의 내용과 postflight(계약 negative 1~4, 6), A/B 결정성 비교, `System/licenses/rust`
배치, zip 포함 여부, CI parity. 코드 경로는 작성돼 있으나 **번들이 만들어지지 않아 한 번도 실행되지
않았다** — 완료로 문서화하지 않는다.

## 6. 완료 명령 결과

| # | 명령 | 종료 | 마지막 결과 줄 |
|---|---|---|---|
| 1 | `… build/package-rust-notices.ps1 -OutputDir target/task103-rust-notices-a` | **1** | `throw ("$($withoutText.Count) package(s) in the device depend …` — 3개 패키지 이름 열거 (아래 원문) |
| 2 | `… build/package-rust-notices.ps1 -OutputDir target/task103-rust-notices-b` | **1** | 1과 동일 |
| 3 | `… build/dist-device.ps1 -NoBuild -Zip` | **0** | `==> zipping dist\slot2-0.1.0-a8cb4af.zip` 뒤 `==> done` |
| 4 | `git diff --check` | **0** | stdout 진단 없음 (stderr에 기존 CRLF 경고만) |

명령 1의 원문(콘솔 wrap 제거):

```text
==> reading the device dependency closure from Cargo (offline)
3 package(s) in the device dependency closure declare a license and ship no license, copying,
copyright or notice text in their packaged source. This task copies original texts and never
invents, downloads or borrows one, so it stops here and names them:
  fluent-langneg 0.13.1 - declared 'Apache-2.0 OR MIT', packaged source <cache>\fluent-langneg-0.13.1, repository https://github.com/projectfluent/fluent-langneg-rs
  gl 0.14.0 - declared 'Apache-2.0', packaged source <cache>\gl-0.14.0, repository https://github.com/brendanzab/gl-rs/
  intl_pluralrules 7.0.2 - declared 'Apache-2.0/MIT', packaged source <cache>\intl_pluralrules-7.0.2, repository https://github.com/zbraniecki/pluralrules
```

명령 3은 **미수정된** 기존 스크립트이므로 여섯 core·core source bundle·기존 notice가 그대로 유지된
상태로 `==> done`까지 간다. `System/licenses/rust/`는 없다(packager가 산출물을 만들지 못했고, 실패하는
packager를 dist-device에 연결하면 카드 빌드 자체가 깨지므로 연결하지 않았다).

Rust 코드·Cargo 입력은 바뀌지 않아 workspace test·clippy·전체 device 빌드는 반복하지 않았다.
CI 전용 단계 중 로컬 실행 불가 항목: `ci.yml`의 device 조립/검증 body는 Linux runner의 `bash` +
`sha256sum`/`sed`/`diff`와 docker cross build를 전제로 하므로 이 Windows 호스트에서 그대로 실행할 수
없다. 로컬에서 실행 가능한 부분(동일 packager를 `pwsh`로 실행, postflight 재검증)은 위에 기록했다.

## 7. 변경/미변경 파일

- **신규**: `build/rust-notices.ps1`, `build/package-rust-notices.ps1` — 둘 다 untracked(커밋 안 함).
- **미변경(의도적)**: `build/dist-device.ps1`, `.github/workflows/ci.yml`, `CORE-NOTICES.md`,
  `docs/DESIGN.md`, `docs/MILESTONES.md`, `build/core-manifest.ps1`, Rust 소스·manifest·`Cargo.lock`.
  이유: 번들이 만들어지지 않았으므로 (a) 실패하는 packager를 dist-device/CI에 연결하면 지금 통과하는
  카드 조립·zip 경로가 깨지고, (b) "Rust notice가 아직 없다"는 `CORE-NOTICES.md` 문장은 **여전히
  사실**이며, (c) 미검증 기능을 완료로 문서화하지 않는다(`AGENTS.md`).
- 검증 뒤 code/content 변경 없음: 마지막 편집은 이 보고서 직전 `build/rust-notices.ps1`의 key 접미사
  수정(§3)이고, 그 뒤 완료 명령 1·4를 재실행해 같은 결과를 확인했다.

## 8. 계약 관련 의견 / 사용자 결정 필요

1. **결정 필요(차단)**: 텍스트를 동봉하지 않는 패키지 정책. 선택지:
   - (a) 계약 수정 — 그런 패키지는 **선언된 SPDX 식만** 기록하고 복사 파일 없이 포함하되,
     `PACKAGE.txt`/`THIRD-PARTY-RUST.md`에 "이 crate는 라이선스 텍스트를 동봉하지 않는다 + upstream
     repository URL"을 명시한다. 발명도 다운로드도 아니고, 세 패키지가 빠지지 않는다. → 66개 전부 포함.
   - (b) 해당 의존성 제거/대체(별도 조사·Cargo 변경 필요, 이 태스크 범위 밖).
   - (c) 릴리스에서 Rust notice 게이트를 요구하지 않음(권장하지 않음 — 카드에 Rust 의존성 고지가
     빠진 채로 나간다).
2. **설계상 한계(수용)**: `cargo metadata`는 workspace feature 통합 때문에 device closure의 superset을
   준다. 그래서 closure 판정은 `cargo tree -p slot2`(device selection 전용)로 하고, metadata는 부가
   metadata 조회에만 쓴다. 계약의 "compare with an independently normalized cargo tree result"는
   `cargo tree`를 두 emission(plain / `{p}|{l}|{r}`)으로 읽어 **독립 파서로 정규화**해 비교하는 방식으로
   만족시켰다 — 서로 다른 도구를 요구했다면 이 호스트에서는 `cargo metadata`로 device selection을
   정확히 재현할 수 없다(위 한계).
3. **남은 M7 작업**: 이 태스크가 끝나야 `dist-device`/zip/CI에 Rust notice가 실린다. 그 뒤에도 tag
   release, README, issue templates, migration 문서, M7 전체 acceptance는 미완이다(이번 지시서가 완료로
   표시하지 말라고 한 항목들).

## 9. 다음 정확한 명령

```text
# 차단 확인(같은 실패 재현)
powershell -NoProfile -ExecutionPolicy Bypass -File build/package-rust-notices.ps1 -OutputDir target/task103-rust-notices-a

# 이전 output 보존 확인(임시 디렉터리)
powershell -NoProfile -ExecutionPolicy Bypass -File build/package-rust-notices.ps1 -OutputDir target/task103-negative/prev-out
```

정책이 정해지면 packager의 텍스트 없음 처리를 그 결정대로 바꾸고, 그 뒤에 `dist-device.ps1`의 사전
빌드/사후 postflight 연결과 `ci.yml` 조립·검증을 추가한 다음 §6의 네 명령과 negative 1~6을 다시 돌려야
한다.

## 10. 2차 호출 (누적 2/2) — declared-only, source-aware identity, dist/zip/CI 통합

### 10.1 정책 변경 (Codex 지시)

1차의 “텍스트가 없으면 실패”를 투명한 포함으로 바꿨다. 모든 third-party 런타임 패키지는 패키지
집합과 SBOM에 남는다.

| `notice_status` | 조건 | 번들 내용 |
|---|---|---|
| `packaged-text` | crate 패키지에 비어 있지 않은 license/notice 파일이 하나 이상 | 그 원문을 byte 그대로 복사 |
| `declared-only` | 그런 파일이 없지만 Cargo `license` 선언은 비어 있지 않음 | 생성된 `PACKAGE.txt`만. notice 경로 목록은 비어 있음 |

- 발명·다운로드·정규화·`OR` 한쪽 선택·다른 crate 텍스트 차용 **없음**. 선언식은 Cargo가 보고한 그대로
  보존된다.
- `license`와 `license-file`이 모두 없으면 여전히 실패, 선언된 `license-file`이 없거나 비면 여전히 실패.
- **데이터 기반**: 세 crate 이름을 코드에 하드코딩하지 않았다. 상태는 매 실행 crate 패키지 내용에서
  다시 판단되므로, 나중에 그 crate가 텍스트를 동봉하면 자동으로 `packaged-text`가 된다.
- `declared-only`의 문구는 지시된 문장 그대로이며 `PACKAGE.txt`와 `THIRD-PARTY-RUST.md`에 들어간다:
  “The crate package declares this license expression but supplied no license/notice text. / Consult the
  recorded upstream repository. This inventory does not replace the license and is not legal advice.”
- `notice_status`는 `PACKAGE.txt`, `THIRD-PARTY-RUST.md`, `RUST-SBOM.json`의 모든 패키지 객체에 노출된다.
- postflight가 거부하는 불일치: `declared-only`인데 notice 파일/경로가 있음, `packaged-text`인데 notice
  목록이 빔, 상태가 현재 캐시된 crate 내용과 다름, declared-only 패키지가 디렉터리 집합·SBOM·inventory·
  manifest 어디서든 빠짐.

### 10.2 source-aware identity 수정

1차 결함: `cargo tree`/`cargo metadata` 결과를 `name version`으로 접어 두어, source가 다른 동명·동버전
패키지가 나중 키 분기 로직에 도달하기 전에 합쳐졌다. 수정 내용:

- 모든 파싱 단계가 **(name, version, source token)** 을 보존한다(`ConvertFrom-RustTreeLine`,
  `Get-RustNoticeIdentity`). source token은 `cargo tree`가 실제로 출력하는 source 텍스트이며, 아무것도
  출력하지 않는 기본 registry는 빈 token으로 표현된다.
- metadata는 사전(dictionary)이 아니라 **리스트**로 돌려주고(`Get-RustNoticeMetadataPackages`),
  `Resolve-RustNoticePackages`가 source 일치로 정확히 하나를 고른다. 후보가 여럿인데 하나로 좁혀지지
  않으면 **명확히 실패**하고 후보 목록을 출력한다(조용히 하나를 버리지 않는다).
- 같은 name+version이 서로 다른 source에서 오면 **둘 다 보존**되고, 디렉터리 키는 source 해시 접미사로
  갈린다(`Get-RustNoticeKeys`, 충돌할 때만 접미사). 체크섬도 `name version source`로 맞춘다.
- 독립 비교(두 `cargo tree` emission)도 접힌 이름이 아니라 source-aware identity로 비교한다.
- 현재 실제 closure에는 git·대체 registry source가 없다(있는 괄호 토큰은 `(*)`, `(proc-macro)`, workspace
  경로뿐) — 즉 모호성이 없다. 그래도 모호성 실패 처리는 구현돼 있다.

**중복 source fixture 결과** (순수 helper, 합성 패키지, 실제 crate·registry·네트워크 불필요):

| 케이스 | 결과 |
|---|---|
| A: `demo-crate 1.2.3` registry + git | entry 2개 해석, **키 2개** `demo-crate-1.2.3-s08e3d198` / `demo-crate-1.2.3-s857175b6`, source 둘 다 보존 |
| B: source 하나 | `demo-crate-1.2.3` (접미사 없음) |
| C: 같은 줄 반복(`(*)`) | identity 1개로 수렴 |
| D: source로도 구분 안 되는 동명·동버전 2개 | **명확히 실패**(하나 선택 안 함) |
| E: workspace path 패키지 / `(proc-macro)` 마커 | path 패키지는 제외, 마커는 source 없이 유지 |

### 10.3 최종 번들 수치

| 항목 | 값 |
|---|---|
| third-party 패키지 | **66** |
| `packaged-text` | **63** (원문 notice 파일 117개 복사) |
| `declared-only` | **3** — `fluent-langneg-0.13.1`(Apache-2.0 OR MIT), `gl-0.14.0`(Apache-2.0), `intl_pluralrules-7.0.2`(Apache-2.0/MIT) |
| 번들 파일 | **186** (769,598 bytes) = `THIRD-PARTY-RUST.md` + `RUST-SBOM.json` + `RUST-MANIFEST.txt` + 66×`PACKAGE.txt` + 117 원문 |
| `RUST-MANIFEST.txt` | 185줄 = 186 − 자기 자신 |
| `Cargo.lock` SHA-256 | `dce2d3bfd41a666f10138bfcbf7d4e1bb2a87e84ef70eb86136a0d94f31de24e` (SBOM 기록값과 일치) |
| A/B 결정성 | 186 vs 186 files, **byte 동일** (정렬 상대경로 + SHA-256), 목록 digest `16bd0a78c18e38fa996e9e20cece89a37dbbf45dbd5b1e218a56f239c44aaf73` |
| `RUST-MANIFEST.txt` SHA-256 | `4f87b75377434175dc011375179045ca53dcfd7f6b8fc10cc524e48316211753` |
| `RUST-SBOM.json` SHA-256 | `cda0b86cb1340c138dcb69422564180fcaf0df81058fc1ada2cdfd69b6556acf` |
| `THIRD-PARTY-RUST.md` SHA-256 | `747967459a29a2f355f0843e070fca4a4120eda07496c70ddd7d0c3d737b8245` |
| 기계별 경로·사용자명 유입 | SBOM·inventory·66개 `PACKAGE.txt` 전체에서 **0건** |
| 스키마 | `slot2-rust-sbom-v1` + `notice_status` (첫 릴리스 전 확장) |

### 10.4 negative 검증 6종 (전부 복사·staging, 실제 산출물 무수정)

| # | 시나리오 | 종료 | 핵심 메시지 |
|---|---|---|---|
| 1 | 패키지 notice 파일 하나 삭제 | 1 | `…\packages\adler2-2.0.1 is missing: LICENSE-APACHE` |
| 2a | notice 파일 1 byte 변경 | 1 | `packages/adler2-2.0.1/LICENSE-APACHE is not the bytes adler2 2.0.1 ships` |
| 2b | `PACKAGE.txt` 1 byte 변경 | 1 | `RUST-MANIFEST.txt records …` (manifest hash 불일치) |
| 3a | 패키지 디렉터리 추가 | 1 | `holds what the manifest does not: not-a-real-crate-9.9.9` |
| 3b | 패키지 디렉터리 삭제 | 1 | `is missing: adler2-2.0.1` |
| 3c | declared-only 디렉터리 누락 | 1 | `is missing: gl-0.14.0` |
| 4a | SBOM notice 경로를 `packages/../../outside.txt`로 | 1 | `RUST-SBOM.json has an unsafe notice path …` |
| 4b | SBOM JSON 손상 | 1 | `ConvertFrom-Json : Invalid object passed in` |
| 4c | 조립된 카드 트리(`System/licenses/rust`)의 SBOM을 위조 | 1 | dist/CI postflight가 같은 unsafe path로 거부 |
| 5 | packager **preflight** 실패(빈 `CARGO_HOME`) | 1 | 이전 output marker SHA-256 **불변**, 파일 1→1, parent 잔여물 **0**, 오류가 실행 명령과 `cargo fetch --locked …` 안내 포함 |
| 6 | dist **preflight** 실패(빈 `CARGO_HOME`) | 1 | 이전 `dist-device` 트리 242개 파일 그대로, 삽입한 marker 존재·SHA-256 **불변**, 빈 `CARGO_HOME` 오류 후 중단, 테스트 marker는 이후 제거 |

### 10.5 dist / zip / CI parity

- `build/dist-device.ps1`: 이전 트리를 지우기 **전에** `target/rust-notices`를 만들고 `Assert-RustNoticeBundle`
  로 검증한 뒤, `System\licenses\rust\`로 복사하고, 조립 후 **다시** postflight한다(패키지 집합·상태·
  복사 byte·SBOM 필드·manifest hash 전부). `-NoBuild`도 Cargo metadata와 로컬 cache source를 요구한다.
- `build/verify-rust-notices.ps1`(신규): CI·조립 단계용 postflight 진입점. 조립된 실제 트리에 대해
  로컬 실행 → `==> rust notice bundle checked: …`, exit 0.
- `.github/workflows/ci.yml`: cross build 뒤 `package rust notices` 단계, 조립 시 `System/licenses/rust`
  생성·복사, artifact 업로드 전 `verify-rust-notices.ps1` 검증을 추가했다. crate 목록을 YAML에 복제하지
  않았고 기존 여섯 core·license/source·artifact 검사는 그대로다.
- 여섯 core/source gate: `dist-device.ps1`이 내부에서 `Assert-CoreTree`·`Assert-LicensesTree`를 통과했고
  (exit 0), 트리에 `System/cores` 6개, `System/licenses/sources` 28개 파일, `VERSION.txt` =
  `SLOT2 0.1.0 (a8cb4af)`가 그대로 남아 있다.
- zip(`dist\slot2-0.1.0-a8cb4af.zip`): 244 entries / 241 files이고 그중 Rust notice 트리 187 entries
  (186 파일 + `packages\` 디렉터리), `RUST-SBOM.json` 1, `RUST-MANIFEST.txt` 1, 패키지 항목 184,
  core `.so` 6, source 트리 29.
- 로컬에서 실행할 수 **없는** CI 전용 단계: device job의 docker cross build, `cores.ps1` native 빌드,
  bash `sha256sum`/`sed`/`diff`/`ls` 기반 조립 검증, `actions/upload-artifact`. 이 호스트(Windows)에는
  해당 runner 환경이 없다. 실행 가능한 부분(같은 packager를 `pwsh`로 실행 + 조립 트리 postflight)은
  로컬에서 돌려 통과를 확인했다.

### 10.6 완료 명령 4개 (2차, 최종 코드 기준 14:15–14:18)

| # | 명령 | 종료 | 마지막 결과 줄 |
|---|---|---|---|
| 1 | `… package-rust-notices.ps1 -OutputDir target/task103-rust-notices-a` | **0** | `==> wrote 186 files (769598 bytes) to C:\SLOT2\target\task103-rust-notices-a` |
| 2 | `… package-rust-notices.ps1 -OutputDir target/task103-rust-notices-b` | **0** | `==> wrote 186 files (769598 bytes) to C:\SLOT2\target\task103-rust-notices-b` |
| 3 | `… build/dist-device.ps1 -NoBuild -Zip` | **0** | `==> zipping dist\slot2-0.1.0-a8cb4af.zip` 뒤 `==> done` |
| 4 | `git diff --check` | **0** | stdout 진단 없음(stderr에 기존 CRLF 경고만) |

Rust/Cargo 입력이 그대로여서 workspace test·clippy·전체 device 빌드는 반복하지 않았다.

### 10.7 변경 파일과 검증 뒤 변경 여부

- **신규**: `build/rust-notices.ps1`(825줄), `build/package-rust-notices.ps1`, `build/verify-rust-notices.ps1`.
- **수정**: `build/dist-device.ps1`(rust 번들 사전 빌드·검증, 복사, 사후 postflight), `.github/workflows/ci.yml`
  (device notice 단계·조립·검증), `CORE-NOTICES.md`(`System/licenses/rust/` 안내, 기기 런타임 범위,
  declared-only 한계, “not legal advice” 유지, “Rust notice가 없다”는 문장 제거), `docs/DESIGN.md`
  (카드 트리에 `licenses/rust/` 실제 경로), `docs/MILESTONES.md`(M7 진행: Rust 고지·SBOM 완료, tag release·
  README·이슈 템플릿·마이그레이션·M7 전체 acceptance는 **미완으로 유지**).
- 미변경: Rust 소스·`Cargo.toml`·`Cargo.lock`(mtime 2026-09-27 그대로), 코어 manifest/pin/patch, 기존
  license 텍스트, assets, README, release workflow, Task 101/102 파일.
- 마지막 code/content 변경은 `build/rust-notices.ps1` 14:13:17(SBOM 경로 안전성 검사를 집합 비교 앞으로
  이동)이고, 그 **뒤에** 완료 명령 4개를 모두 재실행해 위 결과를 얻었다. 그 이후 code/content write 없음
  (마지막 확인 14:18, 완료 명령 종료 14:18).
- 구현 중 고친 자체 결함(모두 검증으로 드러남): `(*)` 마커를 source로 오인하던 파서, SBOM 필드명과
  PowerShell 속성명 매핑, 빈 notice 목록에서 `Assert-SameNameSet`에 빈 배열을 넘기던 호출, postflight가
  scope 객체를 출력 스트림에 흘리던 문제, `&` 호출 시 `$LASTEXITCODE`가 호출자 값으로 남아 dist가
  오탐하던 문제(packager가 명시적으로 `exit 0`).

### 10.8 남은 한계 / M7

1. **declared-only 3건은 법적 공백이 아니라 상류 패키징 사실의 기록**이다. 세 crate는 저장소에는
   라이선스 파일이 있지만 crates.io tarball에 넣지 않았다. 번들에는 선언식과 upstream repository URL만
   있고 원문은 없다 — 이 한계가 `PACKAGE.txt`·`THIRD-PARTY-RUST.md`·`CORE-NOTICES.md`에 명시된다.
2. closure 판정은 `cargo tree -p slot2`(device selection)로 하고, `cargo metadata`는 workspace feature
   통합 때문에 superset이므로 부가 metadata 조회에만 쓴다. `cargo tree`가 기본 registry source를 출력하지
   않는 한계는 “모호하면 명확히 실패”로 방어했고, 현재 실제 closure에는 모호성이 없다.
3. 남은 M7: `release.yml` 태그 릴리스, README(en/ko), 이슈 템플릿·번역 안내, 원본 카드 마이그레이션
   가이드, M7 전체 acceptance. 이번 태스크는 그중 Rust 런타임 고지·SBOM과 카드/zip/CI 배선만 끝냈다.
