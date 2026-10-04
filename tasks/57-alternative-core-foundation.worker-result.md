# Task 57 워커 결과 — gpSP·Gambatte 대체 코어 기반

**성공. 누적 호출 1/2.** 완료 기준 열한 개 명령이 모두 종료 0이고, 두 번째 device-only 호출은 각각
stamp skip을 보고했다. 배포 마지막 줄은 `==> done`, 여섯 코어 이름이 출력됐다.

## gpSP·Gambatte pin, build, 산출물

| | gpSP | Gambatte |
| --- | --- | --- |
| repo / pin | `libretro/gpsp` / `5819380c2ffb0900219d700a382ee68c464ebb99` | `libretro/gambatte-libretro` / `d9d6cd06382d1ced30de34d56d3609452323dab1` |
| makefile | root `Makefile` | root `Makefile.libretro` (기본값) |
| make args | `CPU_ARCH=arm64 HAVE_DYNAREC=1 MMAP_JIT_CACHE=1` (공용 경로가 `ARCH=arm64`도 넘긴다) | 없음 |
| `.so` | `vendor/gpsp_libretro.so` 934,856 B | `vendor/gambatte_libretro.so` 4,805,824 B |
| file(1) | ELF 64-bit LSB shared object, ARM aarch64, stripped | 같음 |
| host DLL | `vendor/gpsp_libretro.dll` 848,384 B (buildbot) | `vendor/gambatte_libretro.dll` 4,137,001 B |

- `.meta`가 repo·commit·source·target·make_dir·makefile·make_args·triple·patch hash를 모두 기록한다
  (`cores/common.sh stamp` 확장). gpSP meta에 arm64 dynarec 인자가 그대로 남아 있고, 인자가 바뀌면
  같은 pin이라도 재빌드된다.
- 첫 device build는 pinned source에서 성공했고(`==> building … device core (aarch64, LTO)`),
  같은 명령 재실행은 각각 `device core is current (vendor/…_libretro.so)`를 출력하며 skip했다.

## registry와 후보 목록

- `CoreId`에 `Gambatte` 추가(ALL 여섯): mGBA(GB/GBC/GBA), Gambatte(GB/GBC), gpSP(GBA),
  FCEUmm(NES), SNES9x(SNES), GenesisPlusGX(MD/SMS). base name `gambatte_libretro`, `file_name`,
  `from_library_path` 라운드트립이 여섯 모두 통과한다.
- `supported_cores(platform)` 추가: GB/GBC `[Mgba, Gambatte]`, GBA `[Mgba, Gpsp]`, NES `[Fceumm]`,
  SNES `[Snes9x]`, MD/SMS `[GenesisPlusGx]`. 각 목록 첫 항목이 `default_core`와 같고,
  `supports_platform`과 양방향으로 일치함을 테스트가 확인한다(기본 core 변경 없음).

## 실제 선택 core의 option과 fallback

- `options_for_core(core, platform, bios_present, tuning) -> Option<Vec<..>>` 추가.
  `options_for(platform, …)`은 기본 core용 호환 wrapper로 남아 결과가 동일하다.
- gpSP·Gambatte는 아직 자체 option이 없어 platform option만 받고 `mgba_*` 키는 하나도 받지 않는다
  (기존 `match def.default_core { Mgba | Gpsp => mGBA option }`이 gpSP에 mGBA option을 넘기던 결함이
  사라졌다). 지원하지 않는 공식 조합은 `None`이다.
- Session은 파일을 고른 뒤(존재/기본 fallback) `CoreId::from_library_path`로 공식 core를 확인하고,
  그 core가 플랫폼을 지원하지 않으면 기본 core로 fallback하며 로그를 남긴다. GBA 게임에
  `gambatte_libretro`를 설정한 케이스가 mGBA로 열리는 것을 테스트로 확인했다. 저장소 밖 알 수 없는
  library는 `core_id() == None`으로 남고 platform 기본 option과 기존 치트 전달 정책(모든 entry)을
  유지한다(Task 55/56 회귀 29개 통과). `cheats()`/필드 설명은 “core가 보유한 목록”에서 실제 의미인
  file-order 전체 desired 목록으로 바로잡았다(동작 불변).

## 실코어 결과

- Session: named `gpsp` + MIT `arm.gba` → `core_id() == Some(Gpsp)`, 10 프레임 실행, 프레임 존재,
  resume state 기록까지 성공(skip 없음). mGBA 기본 회귀도 그대로다.
- `cores` 테스트: 일곱 플랫폼 기본 core 모두 실행(skip 메시지 0건), 대체 코어 테스트에서 gpSP+GBA는
  저장소 ROM으로, Gambatte GB/GBC는 사용자 `assets/test/local`(Pac-Man `.gb`, Galacard `.gbc`)로
  실제 실행됐다 — 이 두 ROM이 없는 환경에서는 그 두 행만 “no test rom available”로 skip된다.
- quirks: Gambatte는 `PassAllEntries`(pinned adapter가 index/enabled를 보존)이고 GB/GBC 코드는
  `Unchecked`, mGBA/FCEUmm/SNES9x 판정은 불변. FCEUmm 8자리 token은 Game Genie와 PAR alphabet이
  겹치고 adapter가 Game Genie를 먼저 본다는 사실로 주석·테스트 설명을 고쳤다(boolean 판정 불변).

## 완료 기준 명령 (최종 상태, 명세 순서)

| 명령 | 종료 | 마지막 결과 줄 |
| --- | --- | --- |
| `build\cores.ps1 -Core gpsp` | 0 | `==> gpsp device core is current`, `==> gpsp host core present` |
| `build\cores.ps1 -Core gambatte` | 0 | `==> gambatte device core is current`, `==> gambatte host core present` |
| `build\cores.ps1 -Core gpsp -DeviceOnly` | 0 | `==> gpsp device core is current (vendor/gpsp_libretro.so)` |
| `build\cores.ps1 -Core gambatte -DeviceOnly` | 0 | `==> gambatte device core is current (vendor/gambatte_libretro.so)` |
| `cargo fmt --all -- --check` | 0 | diff 없음 |
| `cargo test -p slot2-retro --test registry` | 0 | `ok. 17 passed; 0 failed; … finished in 0.02s` |
| `cargo test -p slot2-retro --test cheat_quirks` | 0 | `ok. 21 passed; 0 failed; … finished in 0.06s` |
| `cargo test -p slot2-retro --test cores` | 0 | `ok. 6 passed; 0 failed; … finished in 1.49s` |
| `cargo test -p slot2 --test session` | 0 | `ok. 29 passed; 0 failed; … finished in 14.00s` |
| `cargo clippy -p slot2-retro -p slot2 --all-targets -- -D warnings` | 0 | 경고·오류 없음 |
| `build\dist-device.ps1` | 0 | `==> done`, 여섯 코어(fceumm·gambatte·genesis_plus_gx·gpsp·mgba·snes9x) |

검증 뒤 소스 변경 없음. `dist-device/System/cores/`에 여섯 `.so`, `System/licenses/`에 여섯 `.meta`가
들어갔다.

## 생성·수정 파일과 재생성 artifact

- 추가: `cores/gpsp/{commit,build.sh}`, `cores/gambatte/{commit,build.sh}`.
- 수정: `cores/common.sh`(stamp 확장), `build/cores.ps1`(기본 목록 여섯),
  `crates/slot2-retro/src/{registry.rs,quirks.rs,lib.rs}`,
  `crates/slot2-retro/tests/{registry.rs,cores.rs,cheat_quirks.rs}`,
  `crates/slot2/src/session.rs`, `crates/slot2/tests/session.rs`,
  `docs/{DECISIONS.md,DESIGN.md,MILESTONES.md}`(D-05 6-core, 코어 빌드·카드 목록, M2 통합 완료,
  M4 코어 선택 화면 막힘 해제·화면은 미완료).
- artifact: `vendor/gpsp_libretro.{so,so.meta,dll}`, `vendor/gambatte_libretro.{so,so.meta,dll}`,
  `target-device/cores/{gpsp,gambatte}/`(체크아웃·빌드 트리), `dist-device/` 재생성 파일.

## 남은 위험 (core 전환 UI 전)

Resume·numbered state는 코어별 직렬화 형식이라, 게임별 코어를 바꾸면 이전 코어가 쓴 state는 새
코어에서 실패하거나 잘못 복원된다 — Core 메뉴가 코어 변경 시 state 정책(삭제 또는 코어별 구분)을
정하기 전까지 전환을 열면 안 된다(이번 태스크 범위 밖).

## 계약이 틀려 보이는 부분 / 추가 위험

계약의 gpSP build 인자 목록(`CPU_ARCH/HAVE_DYNAREC/MMAP_JIT_CACHE`)만으로는 공용 cross 경로가 이미
넘기는 `ARCH=arm64`가 빠지는데, `ARCH`는 mGBA를 제외한 makefile들이 `uname -m` 대신 읽는 값이라
둘 다 필요하다(제거하지 않고 그대로 두었다). 또한 GB/GBC 실코어 검증은 저장소가 커밋할 수 없는
사용자 `assets/test/local` ROM에 의존하므로, 그 파일이 없는 환경에서는 그 두 행만 skip되고 나머지
(DLL·registry·배포)는 그대로 검증된다. 소요 약 20분(상한 45분 내).
