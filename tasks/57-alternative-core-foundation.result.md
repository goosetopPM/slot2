# Task 57 — Codex 최종 판정

## 판정

**통과. 누적 호출 1/2.** 추가 작업자 호출은 필요하지 않다.

## 확인한 내용

- gpSP와 Gambatte가 지정 pin에서 aarch64 device core와 Windows host core로 추가됐다. 공용
  `.meta`는 repo·commit·target·make 경로·makefile·추가 인자·triple·patch hash를 기록하며,
  gpSP는 arm64 dynarec 인자를 명시한다.
- registry는 여섯 `CoreId`와 플랫폼별 후보를 단일 계약으로 제공한다. GB/GBC는
  `[Mgba, Gambatte]`, GBA는 `[Mgba, Gpsp]`이고 나머지는 기존 기본 코어 하나다.
- `options_for_core`는 지원하지 않는 공식 조합을 거부하고, gpSP·Gambatte에는 mGBA 전용 option을
  넘기지 않는다. 기존 기본 코어 wrapper의 결과는 유지된다.
- Session은 실제 선택 dylib의 공식 identity를 보존한다. 지원하지 않는 공식 코어는 플랫폼 기본
  코어로 돌아가며, 알 수 없는 기존 외부 코어의 호환 경로는 유지된다.
- Gambatte와 gpSP의 치트 전달 정책은 `PassAllEntries`, 문법 판정은 `Unchecked`다. 기존
  mGBA·FCEUmm·SNES9x 검증 계약은 바뀌지 않았다.
- gpSP는 저장소의 MIT GBA ROM으로 실제 Session을 열어 10프레임을 실행했다. Gambatte도 사용자의
  gitignore 대상 로컬 GB/GBC ROM으로 실제 실행됐고 상용 ROM은 저장소 상태에 나타나지 않는다.

## 작업자 검증 증거

- 두 코어의 최초 device build 및 동일 stamp 재호출 skip — 종료 0
- `cargo fmt --all -- --check` — 종료 0
- registry — 17 passed / 0 failed
- cheat quirks — 21 passed / 0 failed
- cores — 6 passed / 0 failed
- Session — 29 passed / 0 failed
- `cargo clippy -p slot2-retro -p slot2 --all-targets -- -D warnings` — 종료 0
- device 배포 — 종료 0, `==> done`, 여섯 코어 포함

Codex는 사용자 운영 규칙에 따라 테스트를 다시 실행하지 않고 보고서와 구현·테스트 코드를
대조했다.

## 검토 중 정리한 문서 불일치

구현에는 영향이 없지만 Task 57의 6-core 결정과 충돌하던 이전 문구를 바로잡았다.

- D-21과 압축 ROM 보류 항목의 코어 수를 5에서 6으로 수정
- 보류 후보에서 이미 통합된 Gambatte 제거
- DESIGN의 소스 트리와 V-8, MILESTONES의 M2 제목을 6-core 구성으로 수정

문서 수정 뒤 `git diff --check`는 종료 0이다. 코드나 검증 대상은 바꾸지 않았다.

## 다음 방향

코어를 바꾸면 Resume과 numbered state의 직렬화 형식이 달라진다. Core 메뉴를 노출하기 전에
스테이트 파일을 코어별로 격리하고, 현재 코어의 Resume만 자동 복원하도록 저장소·Session 계약을
먼저 완성해야 한다. 그 다음 CorePicker UI와 App 재시작 배선을 분리한다.
