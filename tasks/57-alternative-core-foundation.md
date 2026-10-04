# Task 57 — gpSP·Gambatte 대체 코어 기반

## 목적

게임별 코어 전환을 구현하기 전에 실제 선택 후보를 빌드·배포하고 registry와 Session의 코어별
계약을 완성한다.

- GB/GBC: 기본 mGBA, 대체 Gambatte
- GBA: 기본 mGBA, 대체 gpSP
- NES/SNES/MD/SMS: 현재 각 기본 코어 하나

gpSP와 Gambatte를 pinned device core 및 Windows host core로 추가하고, 플랫폼별 지원 코어 목록을
registry의 단일 진실로 만든다. Session은 실제 선택된 공식 코어가 해당 플랫폼을 지원하는지 확인하고
그 코어에 맞는 option만 넘겨야 한다. 인게임 코어 선택 화면과 재시작·상태 처리 배선은 후속 태스크다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 모두 횟수에 포함한다.

## 먼저 읽을 범위

- `AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `docs/DECISIONS.md`의 D-03, D-05, D-21, D-23만
- `docs/DESIGN.md`의 코어 registry·코어 빌드·게임별 ini·인게임 메뉴 부분만
- `docs/MILESTONES.md`의 M2 코어 빌드와 M4 코어 선택 항목만
- `tasks/55-mgba-disabled-cheat-delivery.result.md`
- `tasks/56-fceumm-cheat-validator.result.md`
- `cores/common.sh`, 기존 네 core의 `build.sh`와 `commit`
- `build/cores.ps1`, `build/dist-device.ps1`
- `crates/slot2-retro/src/registry.rs`, `src/quirks.rs`, `src/lib.rs`
- `crates/slot2-retro/tests/registry.rs`, `tests/cores.rs`, `tests/cheat_quirks.rs`
- `crates/slot2/src/session.rs`의 core 선택·fallback·option 생성 부분
- `crates/slot2/tests/session.rs`의 core fixture와 시작 테스트 부분만

## 확정 결정과 upstream pin

사용자가 이번 태스크에서 D-05를 다음 6-core 구성으로 확장했다.

- mGBA: GB/GBC/GBA 기본
- Gambatte: GB/GBC 대체
- gpSP: GBA 대체
- FCEUmm: NES
- SNES9x: SNES
- Genesis Plus GX: MD/SMS

아래 pin을 그대로 사용한다. 임의로 최신 commit이나 tag로 바꾸지 않는다.

### gpSP

- repository: `https://github.com/libretro/gpsp`
- commit: `5819380c2ffb0900219d700a382ee68c464ebb99`
- commit date: 2026-09-19
- target: `gpsp_libretro.so`
- build file: repository root `Makefile`
- H700은 aarch64이므로 cross build에서 `CPU_ARCH=arm64`, `HAVE_DYNAREC=1`,
  `MMAP_JIT_CACHE=1`을 명시한다. build container의 x86_64 `uname` 추론에 맡기면 x86 dynarec을
  선택하므로 안 된다.

### Gambatte

- repository: `https://github.com/libretro/gambatte-libretro`
- commit: `d9d6cd06382d1ced30de34d56d3609452323dab1`
- commit date: 2026-08-21
- target: `gambatte_libretro.so`
- build file: repository root `Makefile.libretro`(root `Makefile`은 이를 include할 뿐이다)
- 일반 `platform=unix`와 cross `CC/CXX/AR/RANLIB`을 사용한다. 근거 없이 기기별 최적화 flag를
  추가하지 않는다.

source provenance:

- gpSP commit: `https://github.com/libretro/gpsp/commit/5819380c2ffb0900219d700a382ee68c464ebb99`
- Gambatte commit: `https://github.com/libretro/gambatte-libretro/commit/d9d6cd06382d1ced30de34d56d3609452323dab1`
- pinned Gambatte adapter의 `retro_cheat_set`은 index별 code/enabled를 보존하고 enabled entry만
  재적용한다. 따라서 전달 정책은 `PassAllEntries`다. 이번 태스크에서 문법 validator는 만들지 않는다.

## 구현 계약

### 1. 재현 가능한 core build

다음을 추가한다.

- `cores/gpsp/commit`
- `cores/gpsp/build.sh`
- `cores/gambatte/commit`
- `cores/gambatte/build.sh`

기존 `cores/common.sh`를 사용하되 gpSP의 Makefile과 arm64 dynarec 인자는 위 계약대로 지정한다.
Gambatte는 불필요한 인자 없이 고정 Makefile을 사용한다.

`cores/common.sh stamp`가 최소한 다음을 기록하도록 보완한다.

- repository와 commit
- target
- make directory와 makefile
- make arguments
- triple 및 기존 patch hashes

빌드 인자를 바꿨는데 기존 `.meta`가 current로 판정되는 일이 없어야 한다. 이 변경으로 기존 core의
다음 전체 빌드에서 stamp가 갱신되는 것은 정상이다.

`build/cores.ps1`의 기본 core 목록에 `gpsp`, `gambatte`를 추가한다. host DLL은 기존 정책대로
libretro buildbot의 `gpsp_libretro.dll.zip`, `gambatte_libretro.dll.zip`에서 받는다. host DLL은
Windows 개발 테스트용이며 배포물은 위 pin에서 빌드한 aarch64 `.so`라는 구분을 유지한다.

### 2. 6-core registry와 플랫폼별 후보

`slot2-retro::CoreId`에 `Gambatte`를 추가하고 다음을 모두 반영한다.

- `CoreId::ALL`, `base_name`, `file_name`, `from_library_path`
- Gambatte base name: `gambatte_libretro`
- `supports_platform`: Gambatte는 GB/GBC만, gpSP는 GBA만
- mGBA는 GB/GBC/GBA를 계속 지원하고 세 플랫폼의 기본값으로 유지
- FCEUmm/SNES9x/GPGX 기존 관계 불변

플랫폼의 **지원 후보를 순서까지 포함해** 한 곳에서 돌려주는 공개 API를 추가한다. 이름은 Rust
관례에 맞게 조정할 수 있지만 의미는 다음과 같다.

```rust
pub fn supported_cores(platform: Platform) -> &'static [CoreId];
```

- GB: `[Mgba, Gambatte]`
- GBC: `[Mgba, Gambatte]`
- GBA: `[Mgba, Gpsp]`
- NES: `[Fceumm]`
- SNES: `[Snes9x]`
- MD/SMS: `[GenesisPlusGx]`
- 각 slice의 첫 항목은 `PlatformDef::default_core`와 반드시 같다.
- `supports_platform`과 이 목록이 서로 어긋나지 않도록 테스트한다.

후속 UI는 이 API에서 후보를 얻고 `System/cores`에 실제 파일이 있는 항목만 표시할 예정이다.
이번 태스크에서는 UI를 만들지 않는다.

### 3. 선택 코어별 option

현재 `options_for(platform, ...)`은 플랫폼 기본 core를 기준으로 mGBA option을 만든다. 대체 core를
선택한 Session에 mGBA option을 넘기지 않도록 core-aware API를 추가한다.

- 기존 `options_for(platform, ...)`은 기본 core용 호환 wrapper로 유지한다.
- 새 API는 `(core, platform, bios_present, tuning)`을 받고 지원하지 않는 공식 조합을 표현할 수
  있어야 한다(`Option`/`Result` 중 코드에 맞는 쪽).
- mGBA는 기존 option 결과를 완전히 유지한다.
- gpSP와 Gambatte에는 mGBA 이름의 option을 하나도 넘기지 않는다. 이번 태스크에서 검증되지 않은
  gpSP/Gambatte 전용 option을 추측해 추가하지 않는다.
- 나머지 네 core의 기존 option 결과를 유지한다.
- aspect와 overscan은 계속 frontend가 소유한다.

### 4. Session의 공식 core 선택 경계

Session은 설정된 core 파일을 찾은 뒤 `CoreId::from_library_path`로 공식 core인지 확인한다.

- 알려진 공식 core가 현재 플랫폼을 지원하면 그 core를 사용한다.
- 알려진 공식 core가 현재 플랫폼을 지원하지 않으면 기본 core로 fallback하고 로그를 남긴다.
  예: GBA 게임에 `gambatte_libretro`를 설정해도 Gambatte를 열지 않는다.
- 설정된 파일이 없을 때의 기존 기본 fallback을 유지한다.
- 저장소 밖에서 추가한 알 수 없는 existing core는 `None` identity로 두고 기존 호환 동작을 유지한다.
- 최종적으로 실제 선택된 공식 core에는 새 core-aware option API의 결과를 넘긴다. unknown core에는
  기존처럼 플랫폼 기본 option을 넘긴다.
- `core_id()`는 gpSP/Gambatte를 포함해 실제 선택 dylib를 반환한다.
- Task55의 공통 치트 적용 경로, save RAM, launch 실패 원자성은 유지한다.
- `Session::cheats()`/필드의 “core가 보유한 목록”이라는 오래된 설명은 이번 Session 수정에서 실제
  의미인 desired/file-order 전체 목록으로 바로잡는다. 동작은 바꾸지 않는다.

### 5. quirks의 새 core exhaustiveness

- Gambatte + GB/GBC의 non-empty·NUL-free code는 이번 태스크에서 `Unchecked`다.
- pinned Gambatte가 index/enabled를 처리하므로 `cheat_delivery(Gambatte)`는 `PassAllEntries`다.
- gpSP도 기존 `PassAllEntries`, `Unchecked`를 유지한다.
- mGBA/FCEUmm/SNES9x의 validator 결과를 바꾸지 않는다.
- Task56 비차단 항목을 정리한다. 8자리 FCEUmm token은 Game Genie와 PAR 양쪽 문법에 겹칠 수 있고
  pinned adapter가 Game Genie를 먼저 판정한다고 주석·테스트 설명을 고친다. boolean validator의
  동작은 바꾸지 않는다.

### 6. 문서 갱신

- D-05를 위 6-core 구성으로 갱신한다. 이 변경은 사용자가 명시적으로 승인했다.
- DESIGN의 core 수·목록과 플랫폼별 대체 core 설명을 실제 구현과 맞춘다.
- M2의 gpSP 후순위/미구현 설명을 gpSP·Gambatte 통합 완료로 고친다.
- M4 코어 선택 화면의 “gpSP 빌드가 선행이라 막힘”은 해제하되, 화면 자체는 아직 미완료로 둔다.
- 없는 UI나 즉시 전환 기능을 완료했다고 쓰지 않는다.

## 테스트 계약

### build와 배포

- gpSP와 Gambatte의 첫 device build가 각각 pinned source에서 성공한다.
- 두 `.so`는 ELF 64-bit LSB, ARM aarch64 shared object다.
- `.meta`에 각 pin, source, target, makefile, gpSP arm64 dynarec arguments가 기록된다.
- 같은 명령을 다시 실행하면 stamp match로 device build가 skip된다.
- Windows host DLL 두 개가 `vendor/`에 존재한다.
- `dist-device/System/cores/`에 기존 네 core와 두 새 core, 총 여섯 `.so`가 들어간다.

### registry와 quirks

- 여섯 core의 base/file/path round trip과 혼합 대소문자, 잘못된 suffix 거부
- 일곱 플랫폼의 `supported_cores` 목록과 기본-first 불변
- 모든 `supports_platform` 결과와 후보 목록의 양방향 일치
- Gambatte는 GB/GBC만, gpSP는 GBA만
- 기본 `options_for` 결과 회귀
- 대체 core option에는 `mgba_` key가 없고 unsupported 공식 pair는 거부됨
- Gambatte GB/GBC와 gpSP GBA는 `Unchecked`; Gambatte/gpSP delivery는 `PassAllEntries`
- 기존 세 validator와 FCEUmm Game Genie 우선 설명 회귀

### 실제 host core

- 기존 MIT `assets/test/arm.gba`로 named gpSP Session을 시작하고 `core_id() == Some(Gpsp)`, 한 frame
  이상 실행, 종료까지 확인한다. `vendor/gpsp_libretro.dll`이 이번 태스크에서 생성되므로 skip 금지다.
- 같은 GBA fixture의 기본 mGBA 회귀를 유지한다.
- GB/GBC ROM은 Nintendo logo를 포함해야 하므로 새 ROM을 생성·커밋하지 않는다. 사용자의
  `assets/test/local` 또는 `$SLOT2_TEST_ROMS`에 합법적 fixture가 있으면 Gambatte GB/GBC도 실행하고,
  없으면 그 두 content smoke만 명시적으로 skip해 보고한다. DLL·registry·배포 검증은 skip할 수 없다.
- 알려진 unsupported core 설정이 기본 core로 fallback하고 unknown existing core 호환 경로가 유지됨을
  filesystem 단위로 검증한다. 테스트 전용 제품 hook는 만들지 않는다.

## 수정 허용 범위

- `cores/common.sh`
- `cores/gpsp/commit`, `cores/gpsp/build.sh`
- `cores/gambatte/commit`, `cores/gambatte/build.sh`
- `build/cores.ps1`
- `crates/slot2-retro/src/registry.rs`, `src/quirks.rs`, `src/lib.rs`
- `crates/slot2-retro/tests/registry.rs`, `tests/cores.rs`, `tests/cheat_quirks.rs`
- `crates/slot2/src/session.rs`
- `crates/slot2/tests/session.rs`
- `docs/DECISIONS.md`, `docs/DESIGN.md`, `docs/MILESTONES.md`
- `tasks/57-alternative-core-foundation.worker-result.md`
- 빌드 산출물: `vendor/gpsp_libretro.*`, `vendor/gambatte_libretro.*`, 각 `.meta`,
  `target-device/cores/{gpsp,gambatte}/`, `dist-device/`의 재생성 파일

그 밖의 소스·문서 파일은 수정하지 않는다.

## 금지·범위 밖

- 커밋·푸시
- 위 pinned GitHub repository와 기존 libretro buildbot 이외의 네트워크 사용
- pin 변경, 임의 patch, 근거 없는 compiler flag 추가
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- `~/.gjc-bai/agent/*.yml` 및 공용 설정 수정
- 상용 ROM 다운로드·복사·커밋, Nintendo logo를 넣은 새 GB/GBC fixture 생성
- 인게임 Core 메뉴 UI, App의 Core 행 배선, 실행 중 즉시 core hot-swap
- core 변경 시 Resume/numbered state 정책 결정 또는 state 파일 삭제
- gpSP/Gambatte 치트 문법 validator
- 플랫폼 기본 core 변경

## 완료 기준

최종 상태에서 아래를 원문 그대로 순서대로 실행한다. 첫 두 명령에 한해 위 공식 source/buildbot
네트워크 사용을 허용한다.

```text
powershell -NoProfile -ExecutionPolicy Bypass -File build\cores.ps1 -Core gpsp
powershell -NoProfile -ExecutionPolicy Bypass -File build\cores.ps1 -Core gambatte
powershell -NoProfile -ExecutionPolicy Bypass -File build\cores.ps1 -Core gpsp -DeviceOnly
powershell -NoProfile -ExecutionPolicy Bypass -File build\cores.ps1 -Core gambatte -DeviceOnly
cargo fmt --all -- --check
cargo test -p slot2-retro --test registry
cargo test -p slot2-retro --test cheat_quirks
cargo test -p slot2-retro --test cores
cargo test -p slot2 --test session
cargo clippy -p slot2-retro -p slot2 --all-targets -- -D warnings
powershell -NoProfile -ExecutionPolicy Bypass -File build\dist-device.ps1
```

- 각 명령의 종료 코드와 마지막 결과 줄을 기록한다.
- 두 번째 device-only 호출들은 반드시 stamp skip을 보고해야 한다.
- `cores` 테스트에서 각 core/platform의 실행 또는 skip 이유를 개별 기록한다.
- 배포 명령의 마지막 줄은 `==> done`이고 여섯 core 이름이 출력되어야 한다.
- 검증 뒤 소스를 고치면 영향을 받는 명령부터 다시 실행한다.
- 첫 응답까지 최대 5분, 전체 최대 45분 기다린다. 진행 신호는 로그 크기가 아니라
  `git status --short`다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\57-alternative-core-foundation.worker-result.md`를 작성한다.

- 성공/실패와 누적 호출 횟수
- gpSP/Gambatte pin, build args, `.so` architecture, `.meta`, second-run skip 결과
- 여섯 core registry와 플랫폼별 후보 목록
- 실제 선택 core의 option/fallback 처리
- gpSP 실제 Session 결과와 Gambatte GB/GBC 실행 또는 정확한 skip 이유
- quirks exhaustiveness와 FCEUmm 주석 정리
- 모든 완료 기준 명령의 종료 코드와 마지막 결과 줄
- 생성·수정 파일 및 재생성된 build artifact 목록
- core 전환 UI 전에 남은 Resume/numbered state 호환 위험 1줄
- 계약이 틀려 보이는 부분 또는 추가 위험 1줄

실패해도 반드시 원인과 마지막 상태를 쓴다. 두 번째 호출도 실패하면 더 시도하지 않는다.
