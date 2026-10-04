# Task 100 — Codex 최종 판정

## 판정

**통과 (누적 호출 1/2).** 추가 작업자 호출은 필요하지 않다.

## 확인한 내용

- `cores/required.txt`가 mGBA·Gambatte·gpSP·FCEUmm·snes9x·Genesis Plus GX의 단일 build/distribution
  manifest가 됐다. Rust 계약 테스트가 그 순서와 `Core::ALL`을 exact equality로 비교하고 각 core의
  build script와 40자리 pin을 확인한다.
- `build/cores.ps1`은 기본 호출에서 manifest를 사용하고 명시적 `-Core` 부분 빌드를 유지한다. 공용
  helper는 안전하지 않은 이름·빈 줄·중복·누락 build script/commit을 build나 fetch 전에 거부한다.
- gpSP는 native `CROSS_TRIPLE=`에서 arm64 전용 make args를 제거하고, device/aarch64 build에서는 기존
  `CPU_ARCH=arm64 HAVE_DYNAREC=1 MMAP_JIT_CACHE=1`과 stamp를 유지한다. 다른 nonempty cross triple은
  명확히 거부한다.
- `dist-device.ps1`은 core build 또는 preflight 뒤 manifest의 여섯 `.so`와 여섯 `.meta`가 모두
  nonempty인지 확인한 다음에만 기존 `dist-device`를 교체한다. 조립 뒤 exact set을 다시 확인하고
  zip/ADB로 진행하며 ADB도 `System/cores`를 push한다.
- missing/empty core와 meta, 잘못된 manifest를 staging에서 검증했고 모두 nonzero로 실패했다. 기존
  정상 tree marker는 core preflight 실패 뒤에도 보존됐다.
- CI host/device job은 4코어 고정 목록을 제거하고 같은 manifest의 여섯 core와 meta를 빌드·검증한다.
  card assembly도 manifest exact set만 복사한다. 기존 fmt/clippy/test/no-core-skipped 검증은 유지된다.
- DESIGN과 M7 진행 메모는 6-core gate 완료와 tag release·license/source archive 미완료를 구분한다.
- Codex 검토 중 `dist-device.ps1` synopsis의 과거 “one per shelf” 표현을 실제 여섯 manifest core로
  정정했다. 주석만 수정했으며 동작·테스트는 바뀌지 않았다.

## 검증 근거

- `slot2-retro --test registry`: **21 passed / 0 failed / 0 ignored**
- workspace 전체: **973 passed / 0 failed / 0 ignored**
- device core 확인: 여섯 core 모두 최종 current/skip, 네트워크 사용 없음
- `-NoBuild -Zip`: 종료 코드 0, zip의 core/meta 각각 정확히 6개
- device check, fmt, workspace clippy, `git diff --check`: 모두 종료 코드 0
- full device 배포: 종료 코드 0, 마지막 줄 `==> done`, core/meta 각각 정확히 6개
- negative staging: missing/empty core/meta 및 잘못된 manifest가 모두 nonzero, 기존 tree 보존

위 수치는 작업자 보고서의 최종 code/test 변경 후 실행 결과다. Codex는 프로젝트 규칙에 따라 테스트를
다시 실행하지 않고 manifest helper, PowerShell 실행 순서, gpSP 분기, CI loop와 계약 테스트를 대조했다.
이후 Codex 변경은 배포 스크립트 설명 한 줄 정정뿐이다.

## 남은 범위

- M7 tag `release.yml`과 release zip 게시
- SLOT2 및 여섯 core의 라이선스·corresponding source·patch 동봉
- README en/ko, third-party/AI 고지, 이슈 템플릿과 마이그레이션 가이드
- 실기에서 여섯 core로 일곱 플랫폼 선반 부팅 확인
