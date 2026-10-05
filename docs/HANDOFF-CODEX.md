# SLOT2 — Codex 인수인계

## 현재 상태 (2026-10-05)

- 사용자 승인으로 Task114 6개 파일을 커밋 `2941b07`로 `main`에 푸시했다. hosted run
  `37256229710`은 Task114의 ALSA 설치와 host/device clippy를 통과했지만 workspace test에서
  `core_picker_app` 14개 중 2개가 실패해 `device`가 skipped됐다. 두 실패는 integration fixture가 외부
  코어를 `mystery_libretro.dll`로 하드코딩해 Linux product resolver가 찾는 `mystery_libretro.so`를 만들지
  않은 하나의 테스트 이식성 결함이다.
- Task115 누적 1/2 완료, Codex 최종 검토 통과. `core_picker_app`에 표준 라이브러리의 platform DLL
  extension을 쓰는 external core filename helper 하나를 추가하고 두 Windows 전용 fixture path를 이를
  사용하도록 바꿨다. 집중 테스트 14 passed/0 failed, 집중 clippy, fmt, 구조·diff 검증이 통과했다. 최종
  판정은 `tasks/115-linux-core-picker-fixture-portability.result.md`다.
- 사용자가 GitHub username을 `goosetopPM`으로 변경했고 새 canonical repository URL은
  `https://github.com/goosetopPM/slot2`다. 루트 Cargo repository metadata를 새 주소로 변경했으며 local
  `origin`도 같은 주소로 맞춘 뒤 접근성을 확인한다. 다음은 사용자가 Task115 기록과 metadata 변경의
  commit/push를 승인하는 단계이며, push 뒤 hosted `check`와 `device`를 다시 판정한다.
- 사용자가 비공개 GitHub 저장소 `https://github.com/gyuhangcho/slot2`를 만들었고, Codex가 workspace
  repository metadata를 해당 주소로 바로잡아 커밋 `a9f57e2`로 `main`/`origin/main`에 푸시했다. 로컬과
  원격 추적 ref는 일치하고 푸시 직후 작업 트리는 clean이었다.
- 첫 hosted CI run `37254253979`는 `check`의 `clippy (host)`에서 실패했다. Ubuntu runner에
  `libasound2-dev`가 없어 `slot2 -> slot2-audio -> cpal -> alsa -> alsa-sys` 빌드 중 `alsa.pc`를 찾지
  못했다. 선행 core build·presence·fmt는 통과했고, `device`는 `check` 의존성 때문에 skipped다.
  `crates/slot2-i18n/Cargo.toml`의 유일한 UTF-8 BOM도 Rust cache parser 경고를 냈다.
- Task114 누적 1/2 완료, Codex 최종 검토 통과. `.github/workflows/ci.yml`은 core cache 조건과 분리된
  무조건 step에서 `libasound2-dev`를 host clippy 전에 설치하고, 유일하게 BOM이 있던
  `crates/slot2-i18n/Cargo.toml`은 선두 3바이트만 제거했다. 11개 manifest BOM scan, offline metadata,
  fmt와 diff check가 통과했다. 최종 판정은 `tasks/114-hosted-ci-linux-prerequisites.result.md`다. 다음은
  사용자가 Task114 변경 묶음의 commit/push를 승인하는 단계다. 새 hosted run에서 `check`와 `device`가
  모두 통과한 뒤 같은 revision을 한 번 재실행해 core-cache hit와 device artifact를 확인해야 한다.

## 현재 상태 (2026-10-04)

- Task111 1/1 완료, Codex 최종 검토 통과. staged gate에서 드러난 71개 whitespace finding을 닫았다.
  42개 task history EOF blank line과 Task78 trailing space는 각 1바이트만 제거했고, 두 upstream core
  licence는 byte/hash를 보존한 채 exact path에만 `-whitespace` attribute를 적용했다. 대체 manifest
  `tasks/111-staged-whitespace-gate-closure.manifest.txt`는 `.gitattributes`와 Task111 기록을 포함한 454개
  경로이며 예약 판정 파일 생성 후 실제 status와 완전히 일치한다. staged entry와 commit은 여전히 0이다.
  이전 449-path Task110 manifest는 사용하지 않는다. 다음은 사용자가 새 454개 전체의 staging/검사/단일
  한국어 commit을 다시 명시 승인하는 단계다. 최종 판정은
  `tasks/111-staged-whitespace-gate-closure.result.md`다.

- 사용자 승인으로 Task110 manifest 449개를 정확히 staging했으나 `git diff --cached --check`가 Task109/
  110의 pre-stage 검사에서 보이지 않던 71개 오류를 발견해 commit 전에 중단했다. 42개 task history의
  redundant EOF blank line, Task78 report의 trailing space 1개, byte-preserved upstream core licence 2개의
  trailing-space 28개다. 실제 index는 다시 staged 0으로 복구했고 작업 내용은 보존했다. 다음 지시서
  `tasks/111-staged-whitespace-gate-closure.md`는 task history를 exact byte cleanup하고 두 licence에는 좁은
  `.gitattributes` 예외를 둔 뒤 전체 commit set을 새 manifest로 다시 고정한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\111-staged-whitespace-gate-closure.md exactly. Treat it as the complete contract. Work directly without delegation. Do not stage, commit, push, publish, create or move a tag, configure a remote, use the network, access hardware or a card, or change shared configuration. Before stopping, write C:\SLOT2\tasks\111-staged-whitespace-gate-closure.worker-result.md.`

- Task110 1/1 완료, Codex 최종 검토 통과. `tasks/110-public-commit-set-freeze.manifest.txt`에 현재 전체
  개발분 449개 경로를 고정했고, 예약해 둔 Codex 판정 파일까지 생성한 뒤 실제 status 경로와 완전히
  일치한다. task/operations history 270개도 기존 추적 전례와 구현 provenance를 유지하기 위해 공개
  snapshot에 포함하는 권고안이다. staged entry는 0이며 아직 commit하지 않았다. 다음은 사용자가
  manifest 전체를 한국어 메시지의 단일 개발 snapshot으로 commit하도록 명시 승인하는 단계다. 이후
  clean committed rebuild, remote/push/tag, hosted acceptance가 각각 별도 단계로 남는다. 최종 판정은
  `tasks/110-public-commit-set-freeze.result.md`다.

- Task110 작업 지시서 `tasks/110-public-commit-set-freeze.md`를 작성했다. Task109를 통과한 전체 개발분과
  기존부터 추적된 task/operations history를 한 공개 스냅샷에 포함하는 권고안을 검증하고, 실제 staging
  없이 정확한 경로 manifest를 만든다. 이전 빌드·테스트는 반복하지 않으며 commit/push/tag/network/실기는
  금지한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\110-public-commit-set-freeze.md exactly. Treat it as the complete contract. Work directly without delegation. Do not stage, commit, remove files, push, publish, create or move a tag, configure a remote, use the network, access hardware or a card, or change shared configuration. Before stopping, write C:\SLOT2\tasks\110-public-commit-set-freeze.worker-result.md.`

- Task109 누적 2/2 완료, Codex 최종 검토 통과. 현재 커밋 후보에서 금지 파일·비밀·링크·ignored
  output이 없음을 확인하고 fmt, 973 workspace tests, host/device clippy, six-core tests, full dist,
  release package suite를 오프라인으로 통과했다. 빈 untracked 루트 `$env`는 fail-closed 조건을 모두
  만족해 제거했다. 2차에서 local/hosted `System/VERSION.txt`를 BOM 없는 strict UTF-8·LF 3줄로
  통일하고 packager의 BOM/CRLF/no-final-LF/invalid-UTF8 거부 회귀를 추가했다. 최종 판정은
  `tasks/109-offline-release-candidate-audit.result.md`다. 다음은 사용자가 public commit에 포함할
  task/operations history 정책을 정하고 커밋을 명시 승인해야 한다. 그 뒤 clean committed rebuild,
  remote 설정·push/tag와 hosted acceptance가 남는다. 다음 작업 지시서는 아직 작성하지 않았다.

- Task109 누적 1/2은 수정 필요다. 커밋 후보 위생과 8개 오프라인 게이트(fmt, 973 tests, host/device
  clippy, 6 core tests, full dist, package suite, diff check)는 모두 통과했고 빈 `$env`도 정확한 조건에서
  제거됐다. 그러나 로컬 `dist-device.ps1`의 `VERSION.txt`는 BOM+마지막 CRLF이고 hosted artifact는
  BOM 없는 LF라 생산자 byte parity가 깨진다. 마지막 호출은
  `tasks/109-offline-release-candidate-audit-attempt2.md`다. 로컬 출력을 normalize하고 release validator와
  regression을 엄격하게 만든 뒤 worker result를 누적 2/2로 갱신한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\109-offline-release-candidate-audit-attempt2.md exactly. This is cumulative attempt 2/2. Work directly without delegation. Do not commit, stage, push, publish, create or move a tag, configure a remote, use the network, access hardware or a card, or change shared configuration. Update C:\SLOT2\tasks\109-offline-release-candidate-audit.worker-result.md as the cumulative 2/2 report before stopping.`

- 다음 작업 지시서 `tasks/109-offline-release-candidate-audit.md`를 작성했다. 현재 HEAD 이후 430개
  변경 후보가 누적돼 있어 커밋·태그 전에 전체 오프라인 게이트와 커밋 후보 위생을 감사한다. 정확히
  0바이트·untracked 일반 파일인 루트 `$env`만 fail-closed 조건에서 제거하고, fmt·workspace test·host/
  device clippy·core test·full dist·release packager test를 순차 실행한다. dirty HEAD 산출물은 validation
  build로만 취급하며 commit/stage/push/tag/network/실기는 금지한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\109-offline-release-candidate-audit.md exactly. Treat it as the complete contract. Work directly without delegation. Do not commit, stage, push, publish, create or move a tag, configure a remote, use the network, access hardware or a card, or change shared configuration. Before stopping, write C:\SLOT2\tasks\109-offline-release-candidate-audit.worker-result.md.`

- Task108 누적 2/2 완료, Codex 최종 검토 통과. 원본 slot 데이터 카드용 영문 canonical
  `docs/MIGRATION.md`와 동등한 `docs/MIGRATION.ko.md`를 추가하고 README의 관련 경고·문서 링크를
  연결했다. 카드 밖 전체 백업, `System/` 비병합 완전 교체, 기본 코어 평면 스테이트 이동, 충돌
  preflight, best-effort 역방향 rename, `States/` 복구를 실제 소스와 대조했다. 최종 판정은
  `tasks/108-original-slot-migration-guide.result.md`다. M7에는 hosted release/cache/tag/draft,
  hosted issue form render, fresh-card first-user, 광범위 실기 acceptance가 남았다. 다음 작업 지시서는
  아직 작성하지 않았다.

- Task108 누적 1/2은 수정 필요다. 영문/한글 마이그레이션 안내와 README 범위는 대체로 맞지만,
  스테이트 이동 오류가 `System/slot2-diag.txt`에도 남는다고 잘못 안내했고 충돌한 평면 스테이트를
  SLOT2 UI에서 선택할 수 있는 것처럼 썼다. 또한 부분 rename 실패의 역방향 복구는 best-effort인데
  평면 원위치를 보장했다. 마지막 호출은 `tasks/108-original-slot-migration-guide-attempt2.md`다. 양쪽
  가이드의 해당 문단만 고치고 worker result를 누적 2/2로 갱신한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\108-original-slot-migration-guide-attempt2.md exactly. This is cumulative attempt 2/2. Work directly without delegation. Do not commit, push, publish, create a tag, open an issue or pull request, use the network, access hardware, or change shared configuration. Update C:\SLOT2\tasks\108-original-slot-migration-guide.worker-result.md as the cumulative 2/2 report before stopping.`

- 다음 작업 지시서 `tasks/108-original-slot-migration-guide.md`를 작성했다. 기존 원본 slot 데이터
  카드의 외부 전체 백업을 검증한 뒤 `System/`을 완전 교체하고 호환되는 루트 사용자 데이터를 유지하는
  영문/한글 안내를 만든다. 첫 SLOT2 스캔이 평면 스테이트를 기본 코어 namespace로 이동할 수 있으므로
  충돌 중단과 `States/`를 포함한 롤백을 필수로 다룬다. 코드·빌드·workflow·실기·네트워크는 범위 밖이다.
  다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\108-original-slot-migration-guide.md exactly. Treat it as the complete contract. Work directly without delegation. Do not commit, push, publish, create a tag, open an issue or pull request, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\108-original-slot-migration-guide.worker-result.md.`

- Task107 누적 2/2 완료, Codex 최종 검토 통과. 영·한 병기 bug/device/translation GitHub issue
  form과 blank issue 설정을 추가하고, 기존 번역 문서를 영문 canonical `docs/TRANSLATING.md`와 동등한
  `docs/TRANSLATING.ko.md`로 분리했다. 2차에서 translation form의 여덟 가이드 링크를 GitHub issue
  문맥용 `../blob/main/docs/...` 형식으로 고쳤으며, 전체 offline 검증 863개와 YAML 구조·문서 parity·
  링크·인코딩·diff check가 통과했다. 최종 판정은
  `tasks/107-issue-forms-translation-contribution.result.md`다. M7의 이슈 템플릿/번역 기여 안내는
  완료됐고 hosted UI render, release acceptance, fresh-card acceptance, migration guide가 남았다. 다음
  작업 지시서는 아직 작성하지 않았다.

- Task107 1차는 이슈 필드·안전 안내, 영문/한글 번역 가이드, README 링크 범위와 로컬 구조 검증이
  맞지만 누적 1/2 수정 필요다. 번역 form의 `docs/TRANSLATING*.md` 링크는 저장소 파일 문맥에서는
  맞아도 GitHub issue URL 문맥에서는 저장소 루트로 해석되지 않는다. 마지막 호출은
  `tasks/107-issue-forms-translation-contribution-attempt2.md`다. form 안의 가이드 링크만 GitHub가
  issue/comment에 안내하는 `../blob/main/docs/...` 형식으로 고치고 기존 보고서를 누적 2/2로 갱신한다.
  다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\107-issue-forms-translation-contribution-attempt2.md exactly. This is cumulative attempt 2/2. Work directly without delegation. Do not commit, push, publish, create a tag, open an issue or pull request, use the network, access hardware, or change shared configuration. Update C:\SLOT2\tasks\107-issue-forms-translation-contribution.worker-result.md as the cumulative 2/2 report before stopping.`

- 다음 작업 지시서 `tasks/107-issue-forms-translation-contribution.md`를 작성했다. GitHub 이슈 form을
  버그·실기 호환성·번역 조율 세 종류로 추가하고, 기존 한국어 번역 가이드를 영문 canonical
  `docs/TRANSLATING.md`와 동등한 `docs/TRANSLATING.ko.md`로 분리한다. 한국어 README의 번역 링크만
  한글 문서로 바꾸며 public release/support를 가정하지 않고 ROM·BIOS·비밀·무검토 로그 첨부를
  요구하지 않는다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\107-issue-forms-translation-contribution.md exactly. Treat it as the complete contract. Work directly without delegation. Do not commit, push, publish, create a tag, open an issue or pull request, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\107-issue-forms-translation-contribution.worker-result.md.`

- Task106 누적 2/2 완료, Codex 최종 검토 통과. 오래된 README를 영문 공개 가이드로 교체하고 동등한
  `README.ko.md`를 추가했다. 지원 기기·플랫폼·코어, 신규 1장/2장 설치, 카드 구조, 기기/PC 조작,
  문제 해결, 번역·라이선스를 실제 소스와 대조했으며 공개 릴리스·fresh-card·광범위 실기와 기존 카드
  마이그레이션은 완료로 주장하지 않는다. 2차에서 전체 쓰기 원자성/카드 제거 보장을 제거하고 검증된
  `slot2-store` 저장 종류만 정확히 기술했다. 집중 검증 68개와 전체 README 검증 223개, diff check가
  통과했다. 최종 판정은 `tasks/106-public-readme-en-ko.result.md`다. M7의 README와 공개 라이선스 안내는
  완료됐고 hosted release acceptance, issue template/번역 기여 진입점, migration guide가 남았다. 다음
  작업 지시서는 아직 작성하지 않았다.

- Task106 1차는 영문/한글 구조, 설치 경계, 기기·플랫폼·코어·조작 표, 링크, 인코딩과 라이선스
  안내가 모두 맞지만 누적 1/2 수정 필요다. README가 SLOT2의 모든 쓰기를 원자적이라고 단정하지만
  `diag.rs`와 `probe.rs`의 카드 보고서는 직접 쓰기이므로 공개 데이터 안전성 계약을 확대했다. 마지막
  호출은 `tasks/106-public-readme-en-ko-attempt2.md`다. 두 README의 해당 문단에서 전체 쓰기/카드 제거
  보장을 제거하거나 검증된 store 저장 종류로만 좁히고 기존 보고서를 누적 2/2로 갱신한다. 다음 정확한
  전달 문구:
  `Read and execute C:\SLOT2\tasks\106-public-readme-en-ko-attempt2.md exactly. This is cumulative attempt 2/2. Work directly without delegation. Do not commit, push, publish, create a tag, use the network, access hardware, or change shared configuration. Update C:\SLOT2\tasks\106-public-readme-en-ko.worker-result.md as the cumulative 2/2 report before stopping.`

- 다음 작업 지시서 `tasks/106-public-readme-en-ko.md`를 작성했다. 오래된 설계 단계 README를 영문
  공개 안내로 교체하고 동일 계약의 `README.ko.md`를 추가한다. 구현된 기기·플랫폼·코어와 신규
  1장/2장 설치, 카드 구조, 기기/PC 조작, 문제 해결, 번역·라이선스를 실제 소스와 대조한다. 공개
  릴리스·hosted CI·광범위 실기·기존 카드 마이그레이션은 완료로 주장하지 않으며 코드·빌드·workflow는
  변경하지 않는다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\106-public-readme-en-ko.md exactly. Treat it as the complete contract. Work directly without delegation. Do not commit, push, publish, create a tag, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\106-public-readme-en-ko.worker-result.md.`

- Task105 누적 2/2 완료, Codex 최종 검토 통과. CI와 tag release는 하나의 reusable device artifact
  workflow를 사용하며, guarded release workflow는 검증된 `System/` zip과 SHA-256 sidecar로 draft
  GitHub Release만 만든다. 2차에서 zip/sidecar의 순차 file move를 제거하고 fresh OutputDir로 staging
  directory를 한 번 rename하도록 수정했다. 기존 output·late collision 보존, staging ownership,
  241-entry archive·sidecar·전체 음성 검증과 workflow/shell/PowerShell syntax가 성공했다. 최종 판정은
  `tasks/105-tag-release-workflow.result.md`다. 실제 hosted cache miss/hit, tag run, draft asset 확인과
  수동 게시, README·issue template·migration 문서는 남아 있다. 다음 작업 지시서는 아직 작성하지 않았다.

- Task105 1차는 누적 1/2 수정 필요다. reusable device workflow, draft release 권한/그래프, tag/version/
  commit/tree 검증, 241-entry zip과 sidecar 검증은 성공했다. 그러나 `package-release.ps1`이 staged zip과
  sidecar를 최종 위치로 두 번 따로 이동해 두 번째 이동 실패 시 zip만 남길 수 있고, 음성 검증은 이
  승격 중간 실패를 다루지 않았다. 마지막 호출은 `tasks/105-tag-release-workflow-attempt2.md`다. fresh
  OutputDir만 허용하고 검증된 두 파일이 든 staging directory를 한 번의 same-parent rename으로 승격하며
  late destination collision 보존을 검증한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\105-tag-release-workflow-attempt2.md exactly. This is cumulative attempt 2/2. Work directly without delegation. Do not commit, push, publish, create a tag, use the network, access hardware, or change shared configuration. Update C:\SLOT2\tasks\105-tag-release-workflow.worker-result.md as the cumulative 2/2 report before stopping.`

- Task105 작업 지시서 `tasks/105-tag-release-workflow.md`를 작성했다. CI device 구현을
  `device-artifact.yml` 재사용 workflow로 단일화하고, tag/Cargo version/commit/card tree를 검증해
  `System/` 루트 zip과 SHA-256 sidecar를 만드는 offline packager 및 최소 권한의 draft `release.yml`을
  추가한다. 실제 tag·push·hosted workflow·release 게시와 Task104 cache miss/hit acceptance는 금지하고
  후속 사용자 통제 검증으로 남긴다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\105-tag-release-workflow.md exactly. Treat it as the complete contract. Work directly without delegation. Do not commit, push, publish, create a tag, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\105-tag-release-workflow.worker-result.md.`

- Task104 누적 호출 1/2 완료, Codex 최종 검토 통과. CI device job은 host Rust toolchain/cache와 locked
  Cargo fetch를 offline notice packager 전에 수행하고, device core cache는 `vendor`와
  `target-device/cores`를 새 `v2` key로 함께 보존한다. cache 복원 뒤 여섯 core의 binary·stamp·유일 Git
  checkout·tracked pin·HEAD를 manifest loop로 검증한다. 로컬 workflow 구조, shell syntax, 실제 checkout,
  음성 시뮬레이션, 66-package/186-file offline packager 검증은 성공했다. 최종 판정은
  `tasks/104-ci-license-input-cache.result.md`다. 실제 GitHub Actions의 첫 cache miss, 후속 cache hit,
  artifact 내용 확인은 사용자 통제 acceptance로 남아 있으며 그 전에는 M7 artifact parity를 완료로
  표시하지 않는다. 다음 작업 지시서는 아직 작성하지 않았다.

- Task104 작업 지시서 `tasks/104-ci-license-input-cache.md`를 작성했다. Task103 최종 판정에서 확인한
  두 CI 전용 차단 요인을 별도 후속으로 고친다. device job host에서 locked Cargo fetch/cache를 notice
  packager보다 먼저 수행하고, device core cache에 `target-device/cores`의 pinned checkout도 포함하며
  기존 vendor-only cache를 받지 않도록 key를 갱신한다. 실제 hosted CI 성공은 push 없이 주장하지 않고
  후속 사용자 통제 acceptance로 남긴다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\104-ci-license-input-cache.md exactly. Treat it as the complete contract. Work directly without delegation. Do not commit, push, publish, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\104-ci-license-input-cache.worker-result.md.`

- Task103은 누적 2/2로 **최종 실패**, 자동 재시도 중단 상태다. 로컬에서는 device runtime crate 66개
  (63 packaged-text + 3 declared-only)의 186-file deterministic bundle, dist·zip·negative 검증이
  성공했다. 그러나 CI device job은 Rust build를 Docker 안에서 수행한 뒤 host에서 offline notice
  packager를 실행하면서 host Cargo cache를 채우지 않아 fresh runner에서 실패한다. 기존 device core
  cache도 `vendor`만 저장해 cache hit 시 `target-device/cores` checkout이 없어 core source packager가
  실패하는 문제가 있다. 최종 판정은 `tasks/103-rust-runtime-license-sbom.result.md`다. 세 번째 Task103
  지시서는 만들지 않는다. 별도 후속으로 host Cargo fetch/cache와 core checkout cache-hit 경로를 고치고
  실제 CI run을 확인해야 한다.

- Task103 1차는 누적 1/2 실패다. device normal closure 66개 중 63개는 원문 117개를 동봉하지만
  `fluent-langneg 0.13.1`, `gl 0.14.0`, `intl_pluralrules 7.0.2`는 license 선언만 있고 crate package에
  원문 파일이 없어 원 계약대로 중단했다. Codex 검토에서 name+version만 쓰는 discovery map이 서로
  다른 source의 동명·동버전을 조용히 합칠 수 있는 문제도 찾았다. 마지막 호출은
  `tasks/103-rust-runtime-license-sbom-attempt2.md`다. 원문 없는 crate는 투명한 `declared-only`로 포함하고
  source-aware identity를 고친 뒤 dist·zip·CI 연결과 전체 negative 검증을 끝낸다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\103-rust-runtime-license-sbom-attempt2.md exactly. This is cumulative attempt 2/2. Work directly without delegation. Do not commit, push, publish, use the network, access hardware, or change shared configuration. Update C:\SLOT2\tasks\103-rust-runtime-license-sbom.worker-result.md as the cumulative 2/2 report before stopping.`

- 다음 작업 지시서 `tasks/103-rust-runtime-license-sbom.md`를 작성했다. aarch64 device feature의 normal
  runtime dependency closure만 Cargo에서 발견해 각 crate의 원본 license/notice 파일, 선언 metadata,
  결정적 `slot2-rust-sbom-v1` inventory와 hash manifest를 만들고 local dist·zip·CI의
  `System/licenses/rust/`에 동봉한다. dev/build/host-only crate와 release.yml은 범위 밖이다. 다음 정확한
  전달 문구:
  `Read and execute C:\SLOT2\tasks\103-rust-runtime-license-sbom.md exactly. Treat it as the complete contract. Work directly without delegation. Do not commit, push, publish, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\103-rust-runtime-license-sbom.worker-result.md.`

- Task102는 누적 2/2로 최종 통과했다. same-parent candidate, post-promotion validation,
  backup/rollback 보존 상태 기계에 더해 recursive copy 중간 실패의 부분 candidate도 실제 GUID 경로
  존재 여부로 정리한다. 잠금 때문에 정리할 수 없으면 원래 오류를 유지하고 정확한 경로를 경고한다.
  failure harness 8종과 packager 2회, `dist-device -NoBuild -Zip`, diff check가 성공했고 두 bundle은
  Task101의 28개 파일과 byte 동일하다. 최종 판정은
  `tasks/102-safe-source-bundle-rollback.result.md`다. Task101의 license/source bundle은 이 후속으로
  안전 교체까지 닫혔다. 다음은 M7의 Rust dependency notice/SBOM 또는 tag release이며 지시서는 아직
  작성하지 않았다.

- Task102 1차는 rollback/backup 보존과 여섯 failure harness, deterministic bundle, dist 검증이
  성공했지만 최종 판정은 수정 필요다. candidate recursive copy가 중간 실패하면
  `$candidateHolds`가 아직 false라 부분 candidate를 정리·경고하지 않는 경로가 남았다. 마지막 호출은
  `tasks/102-safe-source-bundle-rollback-attempt2.md`이며 이 ownership/cleanup 경로만 고치고 기존
  worker result를 누적 2/2로 갱신한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\102-safe-source-bundle-rollback-attempt2.md exactly. This is cumulative attempt 2/2. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Update C:\SLOT2\tasks\102-safe-source-bundle-rollback.worker-result.md as the cumulative 2/2 report before stopping.`

- Task101 실패의 안전 계약을 조정한 별도 후속 `tasks/102-safe-source-bundle-rollback.md`를 작성했다.
  기존 정상 bundle 보존을 최우선으로 하며 rollback 실패 시 backup을 절대 삭제하지 않는다. Windows
  잠금으로 candidate/backup 정리가 불가능하면 정확한 복구 경로를 경고하고 안전한 잔여물을 허용한다.
  Rust 코드는 바뀌지 않으므로 Task101에서 성공한 workspace test/clippy는 반복하지 않고 packager 2회,
  safe failure harness, `dist-device -NoBuild -Zip`, diff check만 검증한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\102-safe-source-bundle-rollback.md exactly. Treat it as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\102-safe-source-bundle-rollback.worker-result.md.`

- Task101은 누적 2/2로 **최종 실패**, 자동 재시도 중단 상태다. 2차에서 same-parent candidate와
  directory rename을 도입했고 workspace 973 passed 등 일반 검증은 성공했다. 그러나 rollback rename
  실패 시 바깥 `finally`가 이전 정상 bundle이 든 backup까지 무조건 삭제할 수 있다. 또한 승격 실패
  주입에서 잠긴 candidate가 실제로 남아 “성공·실패 모두 잔여물 없음” 계약도 충족하지 못했다. 최종
  판정은 `tasks/101-core-license-source-bundle.result.md`다. 세 번째 지시서는 만들지 않는다. 사용자가
  안전한 잔여물 허용을 포함한 계약 조정 또는 별도 후속 태스크를 지시해야 한다.

- Task101 1차 구현은 license/source 내용과 973개 workspace 검증은 통과했지만 최종 판정은 수정 필요다.
  `build/package-core-sources.ps1`가 기존 output을 먼저 삭제하고 staging을 복사해, 교체 중 I/O 실패 시
  직전 정상 bundle 소실·부분 output이 가능하다. 마지막 호출은
  `tasks/101-core-license-source-bundle-attempt2.md`이며 같은 parent candidate + backup/rollback rename으로
  교체만 고치고 기존 보고서를 누적 2/2로 갱신해야 한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\101-core-license-source-bundle-attempt2.md exactly. This is cumulative attempt 2/2. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Update C:\SLOT2\tasks\101-core-license-source-bundle.worker-result.md as the cumulative 2/2 report before stopping.`

- 다음 작업 지시서 `tasks/101-core-license-source-bundle.md` 작성 완료. root/upstream/core license 원문과
  pinned pristine source archive, SLOT2 build recipe/patch를 deterministic하게 만들어 local dist·zip·CI
  artifact에 함께 넣는다. Rust dependency notice와 tag release는 후속이다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\101-core-license-source-bundle.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\101-core-license-source-bundle.worker-result.md.`

- Task100 누적 호출 1/2 완료, Codex 최종 검토 통과. `cores/required.txt`를 여섯 core의 단일
  build/distribution manifest로 만들고 cores.ps1·local dist/zip/ADB·CI host/device/card assembly가
  core와 meta exact six를 요구하도록 맞췄다. gpSP native CI는 host autodetection, device는 기존 arm64
  dynarec args/stamp를 유지한다. 작업자 검증은 registry 21, workspace 973 passed로 모두 실패·ignored
  0이며 device check·fmt·clippy·full dist·zip·negative staging이 성공했다. Codex는 배포 스크립트의
  오래된 “one per shelf” 설명 한 줄만 정정했다. 최종 판정은
  `tasks/100-six-core-distribution-gate.result.md`다. 다음은 M7 license/source archive와 tag release
  기반이며 작업 지시서는 아직 작성하지 않았다.

- 다음 작업 지시서 `tasks/100-six-core-distribution-gate.md` 작성 완료. registry/build가 지원하는 여섯
  core를 `cores/required.txt` 단일 manifest로 묶고 local dist·zip·ADB·CI가 core/meta exact six를
  요구하도록 맞춘다. gpSP native CI와 device arm64 인자를 분리하며 release.yml·license/source archive는
  후속이다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\100-six-core-distribution-gate.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\100-six-core-distribution-gate.worker-result.md.`

- Task99 누적 호출 1/2 완료, Codex 최종 검토 통과. 기존 MIT PCM 두 개를 보존한 채 7개 플랫폼의
  insert/eject speed·gain profile 14개, styled render와 transformed lead cue를 App에 연결했다.
  작업자 검증은 audio sfx 14, skin 26, sfx_app 11, insert_app 16, workspace 전체 971 passed로 모두
  실패·ignored 0이며 device check·fmt·workspace clippy·device 배포가 종료 0이다. Codex는 M3의 오래된
  “효과음은 남음” 문구 한 줄만 현재 상태로 정정했다. 최종 판정은
  `tasks/99-platform-insert-eject-sfx.result.md`다. M6의 cart/port/curve/sfx 구현은 완료됐고 실기 청감과
  나머지 M6 실기 Acceptance가 남았다. 다음 작업 지시서는 아직 작성하지 않았다.

- 다음 작업 지시서 `tasks/99-platform-insert-eject-sfx.md` 작성 완료. 기존 MIT PCM 두 개는 그대로
  보존하고 `PlatformSkin`의 7×insert/eject speed·gain profile, audio styled render, App의 transformed
  lead cue를 연결한다. 성공하면 M6의 cart/port/curve/sfx 복합 항목을 닫고 workspace·device 배포까지
  검증한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\99-platform-insert-eject-sfx.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\99-platform-insert-eject-sfx.worker-result.md.`

- Task98 누적 호출 1/2 완료, Codex 최종 검토 통과. 공용 duration/load/SFX clock을 유지하면서 7개
  플랫폼별 insert/eject curve 14개를 `PlatformSkin`, travel, ShelfView/App 방향 경로에 연결했다.
  작업자 검증은 skin 23, insert 26, shelf_draw 13, insert_app 16, shelf_shot 1, `slot2-ui` 287,
  `slot2 --tests` 339 passed로 모두 실패·ignored 0이며 device check·fmt·clippy 종료 0이다. Codex는
  과거 공용 hesitation 설명 한 문단만 현재 설계에 맞게 정정했다. 최종 판정은
  `tasks/98-platform-insert-eject-curves.result.md`다. 플랫폼별 sfx와 실기 확인이 남았다.

- 다음 작업 지시서 `tasks/98-platform-insert-eject-curves.md` 작성 완료. 공용 duration·core load·기존
  SFX clock은 유지하고 `PlatformSkin`에 7개 플랫폼별 insert/eject normalized travel curve를 연결한다.
  ShelfView/App 방향 선택과 실패 전환 endpoint 연속성을 검증하며 플랫폼별 효과음은 후속이다. 다음
  정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\98-platform-insert-eject-curves.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\98-platform-insert-eject-curves.worker-result.md.`

- Task97 누적 호출 1/2 완료, Codex 최종 검토 통과. 7개 플랫폼별 port trim SVG를 `PlatformSkin`과
  `ShelfView` front/cache 경로에 연결했다. 중앙 opening과 기존 procedural occlusion은 유지된다.
  작업자 검증은 skin 21, shelf_draw 13, insert 21, `slot2-ui` 전체 280 passed로 모두 실패·ignored
  0이며 device check·fmt·clippy 종료 0이다. 최종 판정은 `tasks/97-platform-port-skins.result.md`다.
  플랫폼별 insert/eject curve와 sfx, 실기 확인이 남았으며 다음 작업 지시서는 아직 작성하지 않았다.

- 다음 작업 지시서 `tasks/97-platform-port-skins.md` 작성 완료. 기존 procedural slot의 삽입 가림
  구조를 유지하면서 7개 플랫폼별 투명 opening port trim SVG를 `PlatformSkin`과 `ShelfView` cache에
  연결한다. 빈 선반 전환·cache·front occlusion 계약까지 검증하며 curve/sfx는 후속이다. 다음 정확한
  전달 문구:
  `Read and execute C:\SLOT2\tasks\97-platform-port-skins.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\97-platform-port-skins.worker-result.md.`

- Task96 누적 호출 1/2 완료, Codex 최종 검토 통과. MD·SMS 독립 shell/detail SVG를 추가해 7개
  플랫폼 모두 고유 카트 자산을 갖게 했고 마지막 GBA fallback helper를 제거했다. 작업자 검증은
  skin 16, shelf_draw 10, insert 18, label 20, `slot2-ui` 전체 269 passed로 모두 실패·ignored 0이며
  device check·fmt·clippy 종료 0이다. 최종 판정은 `tasks/96-md-sms-cartridge-skins.result.md`다.
  플랫폼별 port, 삽입 곡선·효과음과 실기 확인이 남았으며 다음 작업 지시서는 아직 작성하지 않았다.

- 다음 작업 지시서 `tasks/96-md-sms-cartridge-skins.md` 작성 완료. 남은 GBA fallback인 MD·SMS에
  독립 shell/detail SVG 네 개를 추가해 7개 플랫폼의 카트 자산을 모두 고유하게 만든다. port API,
  삽입 곡선과 효과음은 후속으로 유지한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\96-md-sms-cartridge-skins.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\96-md-sms-cartridge-skins.worker-result.md.`

- Task95 누적 호출 1/2 완료, Codex 최종 검토 통과. NES·SNES에 독립 shell/detail SVG를 추가하고
  `PlatformSkin`의 fallback을 해제했다. MD·SMS만 GBA fallback으로 남았고 port API·삽입 곡선·효과음은
  후속 범위다. 작업자 검증은 skin 16, shelf_draw 10, insert 18, label 20, `slot2-ui` 전체 269 passed로
  모두 실패·ignored 0이며 device check·fmt·clippy 종료 0이다. 최종 판정은
  `tasks/95-nes-snes-cartridge-skins.result.md`다. 다음 작업 지시서는 아직 작성하지 않았다.

- 다음 작업 지시서 `tasks/95-nes-snes-cartridge-skins.md` 작성 완료. GBA fallback이던 NES·SNES에
  독립 shell/detail SVG 네 개와 skin table·raster/coverage/aspect 계약을 추가한다. MD/SMS fallback과
  공용 slot mouth는 유지하며 플랫폼 port 구조와 효과음은 후속으로 분리한다. lid 기능은 입력 신호가
  이미 있으나 RG SP 백라이트 경로가 확정되지 않아 추측 구현하지 않는다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\95-nes-snes-cartridge-skins.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\95-nes-snes-cartridge-skins.worker-result.md.`

- Task94 누적 호출 1/2 완료, Codex 최종 검토 통과. 카드 `ja.ftl` 정확히 3 message의 picker 표시,
  App 선택·저장, 영어 fallback과 Shelf 복귀를 신규 통합 테스트로 봉인했다. Task80~84 근거로 M5
  시간대 항목을 완료 처리하고 DESIGN/MILESTONES/TRANSLATING의 오래된 설명을 현재 계약에 맞췄다.
  작업자 검증은 language picker 11, timezone startup 1, timezone menu 1, i18n 42, timezone store 12
  passed로 모두 실패·ignored 0이고 device check·fmt·clippy 종료 0이다. 최종 판정은
  `tasks/94-m5-host-contract-closure.result.md`다. Noto subset·V-10·한국어 전 화면 실기는 남아 있으며
  다음 작업 지시서는 아직 작성하지 않았다.

- 다음 작업 지시서 `tasks/94-m5-host-contract-closure.md` 작성 완료. 카드 `ja.ftl` 정확히 3문자열의
  picker 표시·선택·저장·영어 fallback을 App 통합 테스트로 봉인하고, Task80~84로 이미 완성된 시간대
  항목과 Task93 이후 낡아진 DESIGN/TRANSLATING/MILESTONES 문구를 현재 계약에 맞춘다. Noto subset과
  실기 Acceptance는 닫지 않는다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\94-m5-host-contract-closure.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\94-m5-host-contract-closure.worker-result.md.`

- Task93 누적 호출 1/2 완료, Codex 최종 검토 통과. 기존 production을 바꾸지 않고 Unicode scalar value
  혼합 정렬, portable 특수문자 stem exact 보존, 숨김/확장자 경계, 다섯 부속 경로, 동일 stem 복수
  확장자 공유와 플랫폼 격리를 신규 6개 테스트로 봉인했다. 작업자 검증은 신규 6, 기존 card 19,
  `slot2-store` 전체 118 passed로 모두 실패·ignored 0이고 host/device check·fmt·clippy 종료 0이다.
  Codex는 DESIGN의 UTF-8/Unicode 순서 설명만 정확히 고쳤다. 최종 판정은
  `tasks/93-cart-filename-and-ordering-contract.result.md`다. 다음 작업 지시서는 아직 작성하지 않았다.

- 다음 작업 지시서 `tasks/93-cart-filename-and-ordering-contract.md` 작성 완료. 기존 scalar-value 정렬과
  UTF-8 stem exact 보존을 별도 계약 테스트로 봉인하고, portable 특수문자·다중 점·숨김 파일·확장자
  case·동일 stem 복수 확장자 및 label/save/state/settings/cheat 경로를 검증한다. 동일 platform의 같은
  stem은 legacy 부속 경로를 공유하고 플랫폼 간에는 격리한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\93-cart-filename-and-ordering-contract.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\93-cart-filename-and-ordering-contract.worker-result.md.`

- Task92 누적 호출 1/2 완료, Codex 최종 검토 통과. en/ko 각 125개 key의 동등성, 변수·BTN·JOSA와
  literal cap/function 계약을 test-only scanner로 봉인하고 한국어 문구 전수 감사 및
  `docs/TRANSLATING.md`를 완성했다. 작업자 검증은 `pack_contract` 8, `slot2-i18n` 42,
  `slot2-ui` 263, `slot2` 335 passed로 모두 실패·ignored 0이고 device check·fmt·clippy 종료 0이다.
  Codex는 가이드의 변수/버튼 규칙과 새 내장 언어 절차 문구만 정정했다. 최종 판정은
  `tasks/92-translation-contract-and-guide.result.md`다. Noto subset은 fontTools 실행 환경이 없어서
  계속 후속이며, 다음 작업 지시서는 아직 작성하지 않았다.

- 다음 작업 지시서 `tasks/92-translation-contract-and-guide.md` 작성 완료. en canonical/ko complete의
  125-key 동등성, variable·BTN·JOSA 계약을 test-only scanner로 봉인하고 한국어 문구 전수 감사와
  `docs/TRANSLATING.md`를 작성한다. Task88~91 근거로 언어팩 override/font 항목도 닫는다. Noto subset은
  일반 PATH와 Codex 번들 Python 모두 fontTools/`pyftsubset`이 없어 검증 가능한 실행 환경이 준비될
  때까지 후속으로 남긴다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\92-translation-contract-and-guide.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\92-translation-contract-and-guide.worker-result.md.`

- Task91 누적 호출 2/2 완료, Codex 최종 검토 통과. effective pack의 안전한 `lang-font`를 card-first
  preferred(lazy) → OpenSans(eager) → CJK(lazy) chain에 연결했고 preferred/CJK 중복을 제거했다.
  2차에서는 failed first slot의 line metrics/bitmap fallback과 Noto line box에 맞춘 Cheat 메뉴 여백을
  고쳤다. 작업자 검증은 `slot2-text` 12, `slot2-ui` 263, `slot2-i18n` 34,
  language startup/picker 20 passed로 모두 실패·ignored 0이고 device check·fmt·clippy 종료 0이다.
  최종 판정은 `tasks/91-language-pack-preferred-font.result.md`다. 다음은 Noto Sans KR subset 생성
  스크립트와 host 검증이며 지시서는 아직 작성하지 않았다.

- Task91 1차 호출은 부분 성공(누적 1/2)이다. preferred font 구현과 집중 테스트는 들어갔지만
  `slot2-ui` 전체에서 한국어 Noto의 24px line box 때문에 Cheat 메뉴 upper bar clearance 1건이
  실패했고, corrupt preferred가 첫 slot이면 glyph fallback과 달리 line metrics가 0이라 실제 bitmap이
  사라지는 회귀를 발견했다. 마지막 수정 지시서는
  `tasks/91-language-pack-preferred-font-attempt2.md`이며 기존 worker-result를 누적 2/2로 갱신해야 한다.
  다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\91-language-pack-preferred-font-attempt2.md exactly. This is cumulative attempt 2/2. Continue the existing implementation; do not restart it. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Update C:\SLOT2\tasks\91-language-pack-preferred-font.worker-result.md before stopping.`

- 다음 작업 지시서 `tasks/91-language-pack-preferred-font.md` 작성 완료. effective language pack의
  `lang-font`를 안전한 단일 파일명으로 해석해 card-first preferred → embedded OpenSans → CJK lazy
  chain에 연결한다. missing/unsafe/corrupt preference는 기본 체인으로 fallback하고 언어 전환 성공을
  막지 않으며, 한국어의 잘못된 `.ttf` metadata를 실제 `.otf`로 고친다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\91-language-pack-preferred-font.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\91-language-pack-preferred-font.worker-result.md.`

- Task90 누적 호출 2/2 완료, Codex 최종 검토 통과. 실제 load 성공 pack discovery, Shelf Language
  입력, one-shot request와 load→save→`UiCtx` swap을 host/device 공용 경계에 연결했다. load/save
  실패는 기존 context·카드·current를 유지하며 toast한다. 작업자 집중 테스트 9 passed, `slot2`
  전체 334 passed / 0 failed / 0 ignored, `slot2-i18n` 34 passed / 0 failed / 0 ignored이고 device
  check·fmt·clippy 종료 0이다. 최종 판정은 `tasks/90-language-picker-app-wiring.result.md`다.
  다음은 언어팩 `lang-font` preferred font 적용이며 지시서는 아직 작성하지 않았다.

- Task90 1차 호출은 구현과 622줄 집중 테스트를 만든 뒤 대화형 세션 요청이 869 messages / 약
  2.32MB로 커져 provider `400001 read body failed`로 중단됐다. role/tool-call 구조 이상은 없고 결과
  보고서는 아직 없다. 기존 가재코드 인스턴스를 완전히 종료하고 새 인스턴스에서
  `tasks/90-language-picker-app-wiring-recovery.md`를 누적 2/2 마지막 호출로 수행해야 한다. 다음 정확한
  전달 문구:
  `Read and execute C:\SLOT2\tasks\90-language-picker-app-wiring-recovery.md exactly. This is cumulative attempt 2/2. Continue the existing implementation; do not restart it. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\90-language-picker-app-wiring.worker-result.md.`

- 다음 작업 지시서 `tasks/90-language-picker-app-wiring.md` 작성 완료. load 성공 pack discovery,
  Shelf Language 입력, one-shot runtime request와 load→save→`UiCtx` swap을 host/device 공용 경계에
  연결한다. load/save 실패에서는 기존 context·카드·current를 유지하고 toast한다. `lang-font` 적용은
  후속으로 분리한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\90-language-picker-app-wiring.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\90-language-picker-app-wiring.worker-result.md.`

- Task89 누적 호출 1/2 완료, Codex 최종 검토 통과. caller가 넘긴 load 성공 언어의 code/self-name을
  표시하는 독립 `LanguagePicker`에 current/highlight 분리, wrap, 6행 window, Unicode 말줄임과 세
  geometry safe-area를 구현했다. 작업자 집중 테스트 11 passed, `slot2-ui` 256 passed,
  `slot2-i18n` 34 passed로 모두 실패 0이고 device check·fmt·clippy 종료 0이다. 최종 판정은
  `tasks/89-language-picker-ui.result.md`다. 다음은 pack discovery, Shelf/App 입력, runtime context 교체와
  저장 rollback을 연결하는 작업이며 지시서는 아직 작성하지 않았다.

- 다음 작업 지시서 `tasks/89-language-picker-ui.md` 작성 완료. caller가 넘긴 load 성공 언어의 code와
  self-name을 표시하는 독립 `LanguagePicker`를 추가하고 current/highlight, wrap, 긴 목록 window,
  긴 문자열 말줄임과 세 geometry safe-area를 구현한다. pack discovery, Shelf/App 배선, runtime
  `UiCtx` 교체와 저장은 후속으로 분리한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\89-language-picker-ui.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\89-language-picker-ui.worker-result.md.`

- Task88 누적 호출 2/2 완료, Codex 최종 검토 통과. `SLOT2_LANG` override → 카드 저장 언어 → `en`
  요청 우선순위를 host/device 시작 `UiCtx`에 연결했고 unknown·malformed pack은 원본을 보존한 채 내장
  영어로 fallback한다. 1차 검토에서 발견한 effective code 로그 escaping도 2차에 수정했다. 작업자
  집중 테스트 10 passed, `slot2` 전체 325 passed / 0 failed / 0 ignored, core skip 0이며 device check,
  fmt, clippy 종료 0이다. 최종 판정은 `tasks/88-language-startup-app-wiring.result.md`다. 다음 작업
  지시서는 아직 작성하지 않았으며 Shelf Language picker UI가 다음 단계다.

- Task88 1차 구현은 기능·테스트가 통과했으나 Codex 검토에서 language 진단의 `effective` code가 raw
  출력되는 계약 위반을 발견했다. 제어문자 로그 삽입을 막기 위한 마지막 2/2 수정 지시서는
  `tasks/88-language-startup-app-wiring-attempt2.md`다. 수정 후 기존 worker-result를 누적 2/2로 갱신해야
  하며 아직 최종 통과 판정하지 않았다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\88-language-startup-app-wiring-attempt2.md exactly. This is cumulative attempt 2/2. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Update C:\SLOT2\tasks\88-language-startup-app-wiring.worker-result.md before stopping.`

- 다음 작업 지시서 `tasks/88-language-startup-app-wiring.md` 작성 완료. 명시적인 `SLOT2_LANG` override,
  카드 저장 언어, `en` 순으로 요청 언어를 정하고 host/device `UiCtx` 시작에 적용한다. unknown·malformed
  pack은 원본을 보존한 채 내장 영어로 부팅한다. picker/runtime 교체·저장은 후속으로 분리한다. 다음
  정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\88-language-startup-app-wiring.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\88-language-startup-app-wiring.worker-result.md.`

- Task87 누적 호출 1/2 완료, Codex 최종 검토 통과. `System/slot2.ini`의 독립 `language` key를
  기본 `en`으로 read/safe-write하고 volume·시간대·unknown key를 양방향 보존한다. 작업자 집중
  테스트는 11 passed, `slot2-store` 전체는 112 passed / 0 failed / 0 ignored이고 host/device check,
  fmt, clippy 종료 0이다. 최종 판정은 `tasks/87-global-language-settings-store.result.md`다. 다음은
  저장 언어와 `SLOT2_LANG`의 시작 우선순위, pack load 실패 fallback을 App/i18n 경계에 연결하는
  태스크이며 지시서는 아직 작성하지 않았다.

- 다음 작업 지시서 `tasks/87-global-language-settings-store.md` 작성 완료. `System/slot2.ini`의
  독립 `language` key를 기본 `en`으로 read/safe-write하고 volume·시간대·unknown key를 양방향
  보존한다. safe 단일 stem을 허용해 카드 전용 pack code를 막지 않으며 App/startup/picker는
  후속으로 분리한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\87-global-language-settings-store.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\87-global-language-settings-store.worker-result.md.`

- Task86 누적 호출 1/2 완료, Codex 최종 검토 통과. Shelf의 TimeZone+About 두 행을 활성화하고
  About Screen/input/draw를 연결해 frontend package version과 현재 profile target을 표시한다.
  작업자 About/Timezone 집중 테스트는 각각 1 passed, `slot2` 전체는 315 passed / 0 failed /
  0 ignored이고 device check·fmt·clippy 종료 0이다. Codex는 stale 주석 두 곳만 현재 동작에 맞게
  정정했다. 최종 판정은 `tasks/86-about-sticker-app-wiring.result.md`다. 다음 작업 지시서는 아직
  작성하지 않았다.

- 다음 작업 지시서 `tasks/86-about-sticker-app-wiring.md` 작성 완료. Shelf의 활성 행을 TimeZone과
  About으로 확장하고 `Screen::About`에서 frontend package version과 현재 profile target을
  `AboutSticker`에 전달한다. 기존 시간대 계약은 유지한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\86-about-sticker-app-wiring.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\86-about-sticker-app-wiring.worker-result.md.`

- Task85 누적 호출 1/2 완료, Codex 최종 검토 통과. 호출자가 넘기는 frontend version·target과
  `SLOT2 · MIT`, `System/licenses` 안내를 표시하는 상태 없는 `AboutSticker`를 추가했다. 작업자
  집중 테스트는 8 passed, `slot2-ui`+`slot2-i18n` 전체는 279 passed / 0 failed / 0 ignored이고
  fmt/clippy 종료 0이다. 최종 판정은 `tasks/85-about-sticker-ui.result.md`다. 다음은 Shelf의 About
  행을 활성화하고 App Screen/input/draw에 연결하는 Task 86이다.

- 다음 작업 지시서 `tasks/85-about-sticker-ui.md` 작성 완료. 실제 배포 정보인 frontend version,
  target, `SLOT2 · MIT`, `System/licenses` 안내를 표시하는 독립 `AboutSticker` UI를 추가한다.
  Shelf/App 진입 배선은 Task 86으로 분리한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\85-about-sticker-ui.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\85-about-sticker-ui.worker-result.md.`

- Task84 누적 호출 1/2 완료, Codex 최종 검토 통과. List Menu 탭의 Shelf 설정 진입과 시간대
  runtime preview, cancel/unchanged apply, safe-write 성공 및 실패 rollback을 App에 연결했다.
  작업자 집중 테스트는 1 passed, `slot2` 전체는 314 passed / 0 failed / 0 ignored, i18n은
  33 passed이고 device check·fmt·clippy 종료 0이다. 최종 판정은
  `tasks/84-shelf-timezone-app-wiring.result.md`다. 다음 작업 지시서는 아직 작성하지 않았다.

- 다음 작업 지시서 `tasks/84-shelf-timezone-app-wiring.md` 작성 완료. List의 Menu 탭에서 ShelfMenu를
  열고 TimezoneMenu의 runtime preview, cancel rollback, safe-write apply와 실패 rollback을 App에
  연결한다. process-global clock 검증은 한 integration test에서 순차 실행한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\84-shelf-timezone-app-wiring.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\84-shelf-timezone-app-wiring.worker-result.md.`

- Task83 누적 호출 1/2 완료, Codex 최종 검토 통과. M4 최종 여섯 행의 `ShelfMenu`, 가용성에
  따른 선택·순환·비활성 표시, safe-area와 warm redraw를 구현했다. 작업자 집중 테스트는 9 passed,
  `slot2-ui`+`slot2-i18n` 전체는 270 passed / 0 failed / 0 ignored이고 fmt/clippy 종료 0이다.
  최종 판정은 `tasks/83-shelf-menu-ui.result.md`다. 다음은 App에서 선반 메뉴와 시간대 메뉴를
  연결하고 preview·apply/cancel·safe-write 실패 rollback을 구현하는 Task 84다.

- 다음 작업 지시서 `tasks/83-shelf-menu-ui.md` 작성 완료. M4 최종 순서의 선반 설정 메뉴를
  `slot2-ui`에 추가하고, 아직 없는 기능은 비활성 상태로 표시하며 현재 시간대만 활성화할 수 있는
  가용성 계약을 둔다. Shelf/App 입력 연결과 시간대 preview·apply/cancel·저장 실패 rollback은
  Task 84로 분리한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\83-shelf-menu-ui.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\83-shelf-menu-ui.worker-result.md.`

- Task82 누적 호출 1/2 완료, Codex 최종 검토 통과. 독립 `TimezoneMenu`가 `UTC±HH:MM`, 15분/60분
  clamp 조정과 original/selected를 보존하며 세 geometry의 영문·한글 safe-area와 warm redraw를 지킨다.
  작업자 집중 테스트는 11 passed, `slot2-ui`+`slot2-i18n` 전체는 260 passed / 0 failed / 0 ignored이고
  fmt/clippy 종료 0이다. 최종 판정은 `tasks/82-timezone-menu-ui.result.md`다. Shelf/App 진입,
  runtime preview, apply/cancel, safe-write 실패 rollback이 남았다.

- 다음 작업 지시서 `tasks/82-timezone-menu-ui.md` 작성 완료. 분 단위 고정 UTC offset을
  `UTC±HH:MM`으로 표시하고 Left/Right 15분, Up/Down 60분으로 clamp 조정하는 독립 `TimezoneMenu`를
  추가한다. original/selected를 함께 보존하며 safe-area·영문/한글·warm redraw를 검증한다. Shelf/App
  진입과 preview·apply/cancel·저장 실패 rollback은 후속이다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\82-timezone-menu-ui.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\82-timezone-menu-ui.worker-result.md.`

- Task81 누적 호출 1/2 완료, Codex 최종 검토 통과. `App::with_card`가 카드 UTC offset을 시작 시 한 번
  runtime clock에 적용하고 store/platform 범위를 production compile-time assertion으로 봉인한다.
  단일 순차 integration test와 `slot2` 전체 313 passed / 0 failed / 0 ignored, core skip 0이며 device
  check·fmt·clippy 종료 0이다. 최종 판정은 `tasks/81-timezone-startup-app-wiring.result.md`다. 다음
  지시서는 아직 작성하지 않았으며 시간대 선택 UI와 즉시 적용·저장·rollback이 남았다. boot 진단 로그는
  App 생성 전 초기값을 표시할 수 있어 후속 마무리 대상이다.

- 다음 작업 지시서 `tasks/81-timezone-startup-app-wiring.md` 작성 완료. `App::with_card`가 카드의 표시용
  UTC offset을 시작 시 한 번 runtime clock에 적용하고 store/platform 범위를 compile-time으로 봉인한다.
  process-global clock 검증은 단일 순차 integration test로 격리하며 UI·저장·boot 진단 로그는 후속이다.
  다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\81-timezone-startup-app-wiring.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\81-timezone-startup-app-wiring.worker-result.md.`

- Task80 누적 호출 1/2 완료, Codex 최종 검토 통과. `System/slot2.ini`의
  `utc_offset_minutes`를 기본 0·범위 -720..=840으로 독립 read/safe-write하며 volume과 unknown key를
  양방향 보존한다. invalid UTF-8·directory·범위 밖 입력에서도 원본을 지킨다. 작업자 집중 테스트는
  23 passed, `slot2-store` 전체는 101 passed / 0 failed / 0 ignored이고 fmt/clippy 종료 0이다. 최종
  판정은 `tasks/80-global-timezone-settings-store.result.md`다. 다음 지시서는 아직 작성하지 않았으며
  App 부팅 적용과 store/platform 범위 일치 검증이 다음 기반 작업이다.

- 다음 작업 지시서 `tasks/80-global-timezone-settings-store.md` 작성 완료. D-25의 표시용
  `utc_offset_minutes`를 `System/slot2.ini`에서 독립적으로 읽고 안전하게 쓰며, 기존 volume과 unknown
  key를 양방향 보존한다. 기본 0, 범위 -720..=840이고 OS clock·mtime은 건드리지 않는다. App 부팅
  적용과 설정 UI는 후속으로 분리한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\80-global-timezone-settings-store.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\80-global-timezone-settings-store.worker-result.md.`

- Task79 누적 호출 1/2 완료, Codex 최종 검토 통과. 결정적 generator가 만든 720×720 RGBA8 GB
  CubeXX sample을 production table의 `gb-cubexx-frame-v1` 한 항목으로 등록했다. Integer aperture
  `(40,72,640,576)`는 완전 투명하고 카드 asset 우선·손상 시 내장 fallback·기본 off가 유지된다.
  작업자 overlay 집중 검증은 60 passed, `slot2` 전체는 312 passed / 0 failed / 0 ignored이며 Python
  skip 0, device check와 clippy 종료 0, 배포 빌드 마지막 줄은 `==> done`이다. Codex가 PNG도 직접 열어
  장식과 aperture를 확인했다. 최종 판정은 `tasks/79-gb-cubexx-built-in-overlay.result.md`다. 다음 작업
  지시서는 아직 작성하지 않았다. RG SP 실기 화질·정렬 확인은 사용자 항목이다.

- 다음 작업 지시서 `tasks/79-gb-cubexx-built-in-overlay.md` 작성 완료. GB 기본 Integer 배치의
  640×576 aperture를 완전 투명하게 둔 720×720 CubeXX sample을 deterministic generator로 만들고,
  production table에 `gb-cubexx-frame-v1` 하나만 등록한다. 기본값은 계속 off이며 다른 pair와 scale-aware
  선택은 범위 밖이다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\79-gb-cubexx-built-in-overlay.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\79-gb-cubexx-built-in-overlay.worker-result.md.`

- Task78 누적 호출 1/2 완료, Codex 최종 검토 통과. Display의 마지막에 Overlay 행을 추가하고 세 저장
  의미를 저장 성공 뒤 runtime source에 즉시 반영한다. write 실패는 card bytes와 texture를 보존하고
  asset 실패는 On 선택을 유지한 채 그림만 생략한다. 작업자 관련 suites는 552 passed / 0 failed /
  0 ignored, core skip 0이며 device check와 clippy 종료 0이다. 최종 판정은
  `tasks/78-overlay-menu-app-wiring.result.md`. 다음 지시서는 아직 작성하지 않았으며 production 내장
  overlay asset과 실기 화질 검증이 남았다.

- Task77 작업자 호출 1/2의 구현·검증은 최종 통과. 세 저장 의미, safe-area, 번역과 warm redraw가
  정확하다. 최초 검토에서 찾은 `Some(true)` 설명 오류 두 곳은 Codex가 동작 변경 없이 정정하고
  `git diff --check`로 확인했다. 최종 판정은 `tasks/77-overlay-menu-ui.result.md`.
- 다음 작업 지시서 `tasks/78-overlay-menu-app-wiring.md` 작성 완료. 두 Display row set 마지막에 Overlay를
  추가하고 저장 성공 뒤 runtime source를 즉시 retarget한다. write 실패는 기존 texture까지 보존하고,
  asset 실패는 On 선택을 유지한 채 그림만 생략한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\78-overlay-menu-app-wiring.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\78-overlay-menu-app-wiring.worker-result.md.`

- 다음 작업 지시서 `tasks/77-overlay-menu-ui.md` 작성 완료. overlay의 `None`/`Some(true)`/
  `Some(false)`를 Platform default/On/Off 세 행으로 보존하는 독립 `OverlayMenu`와 영문·한글 번역만
  구현한다. Display 진입, App 저장·rollback·toast·즉시 preview와 실제 PNG는 후속으로 분리한다.
  다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\77-overlay-menu-ui.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\77-overlay-menu-ui.worker-result.md.`

- Task76 누적 호출 1/2 완료, Codex 최종 검토 통과. 최초 launch에서 overlay setting/source를 한 번
  해석하고 App이 game frame 뒤, HUD·UI 앞에 그린다. Inserting/no-session은 제외되고 core switch와
  recovery는 texture를 유지하며 stop/eject는 다음 canvas frame에서 한 번 free한다. 작업자 `slot2`
  전체는 287 passed / 0 failed / 0 ignored, core skip 0이고 device check와 clippy 종료 0이다. 최종
  판정은 `tasks/76-overlay-app-runtime-wiring.result.md`. 다음 태스크 지시서는 아직 작성하지 않았으며
  Overlay 선택 UI와 runtime 저장 배선이 다음 단계다.

- 다음 작업 지시서 `tasks/76-overlay-app-runtime-wiring.md` 작성 완료. Task74의 overlay 설정을 최초
  launch에서 해석하고 App 소유 layer를 game frame 뒤, HUD·UI 앞에 그린다. core switch/recovery는
  texture를 유지하고 stop/eject/no-session draw는 lazy free한다. 메뉴·설정 저장과 실제 내장 sample은
  후속으로 분리한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\76-overlay-app-runtime-wiring.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\76-overlay-app-runtime-wiring.worker-result.md.`

- Task75 누적 호출 1/2 완료, Codex 최종 검토 통과. 카드 geometry PNG와 compile-time 내장 PNG의
  resolver·fallback, exact-size straight RGBA8 decode, lazy upload/cache/free와 panel 전체 일반 image
  draw가 독립 layer로 완성됐다. 작업자 집중 테스트는 19 passed / 0 failed / 0 ignored이고 host tests
  check, device feature check와 clippy 종료 0이다. 최종 판정은 `tasks/75-overlay-asset-layer.result.md`.
  다음 태스크 지시서는 아직 작성하지 않았으며 Task74 setting 해석과 Session/App draw·수명 배선이
  다음 단계다.

- 다음 작업 지시서 `tasks/75-overlay-asset-layer.md` 작성 완료. 카드
  `System/Overlays/<PLAT>/<geometry>.png`와 compile-time 내장 PNG의 우선순위·fallback, exact-size
  RGBA decode, lazy upload/cache/free와 panel 전체 일반 image draw를 독립 layer로 구현한다. 실제
  sample PNG와 Session/App/UI 배선은 후속으로 분리한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\75-overlay-asset-layer.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\75-overlay-asset-layer.worker-result.md.`

- Task74 누적 호출 1/2 완료, Codex 최종 검토 통과. 게임별 ini의 overlay key가 플랫폼 기본값
  상속/사용/명시적 Off 세 상태를 보존하고 safe-write가 기존 setting·unknown key·unreadable 원본을
  지킨다. 작업자 store 전체는 89 passed / 0 failed / 0 ignored, downstream test check와 clippy 종료
  0이다. 최종 판정은 `tasks/74-game-overlay-settings-store.result.md`. 다음 태스크 지시서는 아직
  작성하지 않았으며 geometry별 PNG resolver와 runtime renderer가 다음 기반 작업이다.

- 다음 작업 지시서 `tasks/74-game-overlay-settings-store.md` 작성 완료. 게임별 ini에 overlay key의
  플랫폼 기본값 상속/사용/명시적 Off 세 상태를 `Option<bool>`로 보존하고 기존 setting·unknown key와
  unreadable 원본을 지키는 safe-write를 추가한다. PNG 탐색·decode·renderer·UI/App은 후속으로
  분리한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\74-game-overlay-settings-store.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\74-game-overlay-settings-store.worker-result.md.`

- Task73 누적 호출 1/2 완료, Codex 최종 검토 통과. registry crop이 있는 플랫폼에만 Display
  Overscan 행을 표시하며 카드 저장 성공 뒤 Session crop을 즉시 적용한다. `None`/explicit crop/full
  image 의미, 실패 rollback, 비-NES 5행 보존과 합성 NES/FCEUmm의 실제 UV·visible placement를
  검증했다. 작업자 세 crate 전체는 492 passed / 0 failed / 0 ignored, core skip 0이고 clippy 종료
  0이다. 최종 판정은 `tasks/73-overscan-menu-app-wiring.result.md`. 다음 태스크 지시서는 아직
  작성하지 않았으며 Overlay 선택은 runtime/store 계약부터 후속으로 설계해야 한다.

- 다음 작업 지시서 `tasks/73-overscan-menu-app-wiring.md` 작성 완료. registry crop이 실제로 있는
  platform에서만 Display에 Overscan 행을 추가하고 별도 screen에서 카드 저장 성공 뒤 Session crop을
  즉시 적용한다. 합성 NES/FCEUmm로 default/explicit crop/full-image의 UV와 rollback을 검증하며 비-NES
  5행 Display와 기존 scale/Shader를 보존한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\73-overscan-menu-app-wiring.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\73-overscan-menu-app-wiring.worker-result.md.`

- Task72 누적 호출 1/2 완료, Codex 최종 검토 통과. 독립 `OverscanMenu`가 Platform default, explicit
  crop과 full image를 `Option<bool>` 그대로 세 행으로 제공한다. 세 geometry·두 언어의 safe-area,
  warm glyph cache와 직접 번역을 검증했고 작업자 `slot2-ui`+`slot2-i18n` 전체는 239 passed / 0 failed /
  0 ignored, clippy 종료 0이다. 최종 판정은 `tasks/72-overscan-menu-ui.result.md`. 다음 태스크 지시서는
  아직 작성하지 않았으며 방향은 NES 조건부 Display 진입과 App 저장/live crop이다.

- 다음 작업 지시서 `tasks/72-overscan-menu-ui.md` 작성 완료. store의 `Option<bool>`을 그대로 쓰는 독립
  `OverscanMenu`가 Platform default, explicit crop, full image를 별도 세 행으로 제공한다. safe-area,
  영문/한글 직접 정의와 warm glyph cache를 검증하고 NES 조건부 App 진입·저장·live crop은 Task73으로
  분리한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\72-overscan-menu-ui.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\72-overscan-menu-ui.worker-result.md.`

- Task71 누적 호출 1/2 완료, Codex 최종 검토 통과. Display의 Shader 행에서 별도 Shader screen을 열고
  카드 저장 성공 뒤에만 Session effect를 즉시 적용한다. Platform default, explicit Off와 네 preset,
  failure rollback, known/unknown key 보존과 기존 scale 흐름을 검증했다. 작업자 `slot2`+`slot2-ui`+
  `slot2-i18n` 전체는 472 passed / 0 failed / 0 ignored, core skip 0이고 clippy 종료 0이다. 최종 판정은
  `tasks/71-shader-menu-app-wiring.result.md`. 다음 태스크 지시서는 아직 작성하지 않았다.

- 다음 작업 지시서 `tasks/71-shader-menu-app-wiring.md` 작성 완료. 기존 scale Display 화면에 Shader
  진입 행을 추가하고 별도 Shader screen에서 카드 저장 성공 뒤에만 Session effect를 즉시 바꾼다.
  Platform default, explicit Off와 네 effect를 보존하며 저장 실패는 card/runtime을 유지한다. 다음
  정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\71-shader-menu-app-wiring.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\71-shader-menu-app-wiring.worker-result.md.`

- Task70 누적 호출 1/2 완료, Codex 최종 검토 통과. 독립 `ShaderMenu`가 Platform default, explicit Off와
  네 effect를 store 의미 그대로 여섯 행으로 제공한다. 세 geometry·두 언어의 safe-area, warm glyph
  cache와 직접 번역을 검증했고 작업자 `slot2-ui`+`slot2-i18n` 전체는 231 passed / 0 failed / 0 ignored,
  clippy 종료 0이다. 검토 중 혼동 가능한 NES 기본값 주석만 정확한 ZfastCrt로 정정했다. 최종 판정은
  `tasks/70-shader-menu-ui.result.md`. 다음 태스크 지시서는 아직 작성하지 않았으며 방향은 Display의
  Shader 진입 행과 App live apply/persistence다.

- 다음 작업 지시서 `tasks/70-shader-menu-ui.md` 작성 완료. store의 `Option<ShaderPreset>`을 그대로 쓰는
  독립 `ShaderMenu`에 Platform default, Off와 네 effect를 별도 여섯 행으로 표시한다. 640×480 safe
  area, 영문/한글 직접 정의와 warm glyph cache를 검증하며 App 진입·즉시 preview·저장은 Task71로
  분리한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\70-shader-menu-ui.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\70-shader-menu-ui.worker-result.md.`

- Task69 누적 호출 2/2 완료, Codex 최종 검토 통과. retro registry의 플랫폼 기본 shader는
  GB/GBC/GBA Lcd3x, NES/SNES/MD/SMS ZfastCrt이며 Session은 key 부재만 이를 상속하고 explicit Off와
  네 preset override를 우선한다. 1차에서 드러난 App 통합 테스트 8개도 정확한 GBA Lcd3x effect 단언으로
  정리했다. 작업자 전체 `slot2 --tests`는 231 passed / 0 failed / 0 ignored, core skip 0이고 check/clippy
  종료 0이다. 최종 판정은 `tasks/69-platform-shader-defaults.result.md`. 다음 태스크 지시서는 아직
  작성하지 않았으며 방향은 Display 메뉴 shader 선택과 App live apply/persistence다.

- Task69 1차 호출은 production 집중 검증을 모두 통과했지만 작업자가 추가 실행한 `slot2 --tests`에서
  기존 App 통합 테스트 8개가 GBA 기본 `ImageEffect(Lcd3x)`를 인식하지 못해 실패했다. Codex 판정은
  누적 1/2 미통과이며 `tasks/69-platform-shader-defaults-attempt2.md`에 마지막 수정 지시서를 작성했다.
  production은 유지하고 여덟 테스트의 game-frame 단언만 정확한 Lcd3x effect로 갱신한다. 다음 정확한
  전달 문구:
  `Read and execute C:\SLOT2\tasks\69-platform-shader-defaults-attempt2.md exactly. Treat that file as the complete contract. Work directly without delegation. This is cumulative attempt 2/2; do not make a third attempt. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, update C:\SLOT2\tasks\69-platform-shader-defaults.worker-result.md as the cumulative 2/2 report.`

- 다음 작업 지시서 `tasks/69-platform-shader-defaults.md` 작성 완료. retro registry가 store/gfx와
  독립적인 `PlatformShader` 계약을 소유하고 GB/GBC/GBA는 Lcd3x, NES/SNES/MD/SMS는 ZfastCrt를
  기본값으로 둔다. Session은 key 부재만 플랫폼 기본값으로 해석하고 explicit Off와 네 게임별 override를
  우선한다. Display 메뉴와 App의 live apply·저장은 후속으로 분리한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\69-platform-shader-defaults.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\69-platform-shader-defaults.worker-result.md.`

- Task68 누적 호출 1/2 완료, Codex 최종 검토 통과. 게임별 store preset을 Session 한 경계에서 gfx
  effect로 변환하고 game texture draw만 effect API로 라우팅한다. start/start_named는 기존 settings
  read를 공유하며 runtime setter는 frame·core·texture·audio·카드 bytes를 바꾸지 않는다. explicit
  Off와 key 부재는 현재 plain draw지만 카드 의미는 유지된다. 작업자 session 37 passed / 0 failed,
  lib 33 passed / 0 failed, core skip 0이며 downstream check와 clippy 종료 0이다. 최종 판정은
  `tasks/68-session-shader-routing.result.md`. 다음은 플랫폼 기본값 계약인 Task69이며 지시서가 작성됐다.

- 다음 작업 지시서 `tasks/68-session-shader-routing.md` 작성 완료. Session이 launch 때 이미 읽은
  게임별 shader setting을 gfx effect로 한 곳에서 변환하고 game texture draw만 effect API로 보낸다.
  key 부재와 explicit Off는 플랫폼 기본값이 아직 없는 현재 둘 다 plain draw지만 저장 의미는 합치지
  않는다. runtime accessor/setter와 start/start_named의 동일 경로, geometry·UV 보존을 검증한다.
  플랫폼 기본값과 Display/App 저장 배선은 후속으로 분리한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\68-session-shader-routing.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\68-session-shader-routing.worker-result.md.`

- Task67 누적 호출 1/2 완료, Codex 최종 검토 통과. `slot2-gfx`에 SharpBilinear, Lcd3x, ZfastCrt,
  Scanline 네 단일 패스 effect와 개별 image draw API가 추가됐다. effect는 game quad 뒤 기본 program으로
  복귀해 UI/present에 새지 않고, 개별 compile/link 실패는 driver log를 보존하며 base draw로
  fallback한다. 기본 gfx 테스트 25 passed / 0 failed, device feature/downstream check와 clippy 종료
  0이며, 별도 Intel GLES2 GL 실행도 1 passed다. Mali G31 실기 compile·화질은 미확인이다. 최종 판정은
  `tasks/67-gfx-shader-effects.result.md`. 다음은 Session shader routing인 Task68이며 지시서가 작성됐다.

- 다음 작업 지시서 `tasks/67-gfx-shader-effects.md` 작성 완료. `slot2-gfx`의 개별 image draw에
  SharpBilinear, Lcd3x, ZfastCrt, Scanline 단일 패스 효과를 추가한다. effect는 game texture 한 장에만
  적용되고 HUD·메뉴·text와 최종 present는 기본 program을 유지한다. effect별 compile 실패는 해당
  draw만 base로 fallback하며 availability/error와 GL resource 수명을 검증한다. Session/store 변환,
  플랫폼 기본값과 UI 배선은 후속으로 분리한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\67-gfx-shader-effects.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\67-gfx-shader-effects.worker-result.md.`

- Task66 누적 호출 1/2 완료, Codex 최종 검토 통과. 게임별 ini의 `shader`에 다섯 선택을 canonical
  표기로 저장하며, key 부재의 플랫폼 기본값 상속과 `shader = none`의 명시적 Off를 구분한다.
  invalid 값은 상속으로 떨어지고 safe write가 기존 setting·unknown key·unreadable 원본을 보존한다.
  작업자 store 테스트 79 passed / 0 failed, 신규 13 passed, downstream `slot2 --tests` check와
  fmt/clippy 종료 0이다. 검토 중 현재 gfx에 없는 동명 타입을 이미 존재한다고 한 주석만 사실에 맞게
  정정했다. 최종 판정은 `tasks/66-game-shader-settings-store.result.md`. 다음은 gfx 내장 shader
  effect 기반인 Task67이며 지시서가 작성됐다.

- 다음 작업 지시서 `tasks/66-game-shader-settings-store.md` 작성 완료. 게임별 ini에 shader preset을
  안전하게 저장하는 store 계약을 추가한다. key 부재의 플랫폼 기본값 상속과 `shader = none`의 명시적
  Off를 구분하고, D-10의 네 preset canonical 표기·invalid fallback·safe write·unknown key 보존을
  검증한다. 플랫폼 기본값, 실제 GPU shader와 Display/App 배선은 후속으로 분리한다. 다음 정확한 전달
  문구:
  `Read and execute C:\SLOT2\tasks\66-game-shader-settings-store.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\66-game-shader-settings-store.worker-result.md.`

- Task65 누적 호출 1/2 완료, Codex 최종 검토 통과. App이 전역 volume level을 시작 시 load하고 실제
  level 변경만 마지막 입력 뒤 750ms debounce로 저장한다. 연속 변경·저장값 복귀·clamp·mute 규칙이
  불필요한 SD write를 막고, save 실패는 runtime 상태를 유지한 채 frame별 retry를 멈춘다. physical
  Power와 Power menu Restart/PowerOff는 종료 전 즉시 flush하며 실패해도 Exit을 계속한다. 작업자
  집중 테스트 29 passed, lib 33 passed, 추가 `slot2` 전체 219 passed / 0 failed, fmt/clippy 종료 0이다.
  최종 판정은 `tasks/65-global-volume-app-persistence.result.md`. 다음은 게임별 shader 설정 store인
  Task66이며 지시서가 작성됐다.

- 다음 작업 지시서 `tasks/65-global-volume-app-persistence.md` 작성 완료. App 시작 때 Task64의 전역
  volume level을 읽고, level 변경은 마지막 입력 뒤 750ms debounce로 합쳐 저장한다. 저장값으로 복귀,
  clamp 입력과 mute는 write를 만들지 않으며 physical Power와 Power menu의 Restart/PowerOff는 종료 전
  즉시 flush한다. 실패는 runtime volume·종료를 막지 않고 frame별 retry도 하지 않는다. 다음 정확한
  전달 문구:
  `Read and execute C:\SLOT2\tasks\65-global-volume-app-persistence.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\65-global-volume-app-persistence.worker-result.md.`

- Task64 누적 호출 1/2 완료, Codex 최종 검토 통과. `GlobalSettings`와 Card 전역 설정 API가
  `System/slot2.ini`의 volume level만 소유한다. 기본 70, 범위 0..=100이며 invalid read는 default,
  invalid API write는 error다. safe write가 unreadable 원본과 unknown/future key를 보존하고 기본 복귀
  때 owned key와 빈 파일만 제거한다. mute는 runtime으로 남았다. 작업자 store 테스트 66 passed /
  0 failed, 신규 11 passed, fmt/clippy 종료 0이다. 최종 판정은
  `tasks/64-global-volume-settings-store.result.md`. 다음은 App load와 저장 시점 배선인 Task65이며
  지시서가 작성됐다.

- 다음 작업 지시서 `tasks/64-global-volume-settings-store.md` 작성 완료. `System/slot2.ini`에 전역
  volume level만 안전하게 읽고 쓰는 `GlobalSettings`와 Card API를 `slot2-store`에 추가한다. 기본값은
  70이며 invalid 값은 default로 떨어지고, mutation은 unreadable 원본과 unknown/future key를 보존한다.
  mute는 부팅 시 뜻밖의 무음을 피하기 위해 runtime으로 남기며 App 배선·저장 시점은 Task65로 분리한다.
  다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\64-global-volume-settings-store.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\64-global-volume-settings-store.worker-result.md.`

- Task63 누적 호출 1/2 완료, Codex 최종 검토 통과. Device 행이 현재 runtime `App.volume`에 연결됐고
  Left/Right/A와 물리 VolUp/VolDown 모두 같은 level/mute 상태를 변경한 뒤 UI snapshot을 다시 읽는다.
  Device overlay는 game/audio를 pause하고 frame 위에 그려지며 session/sink/store를 건드리지 않는다.
  brightness/blue-light는 backend 확정 전까지 `None`/Unavailable이다. 작업자 `slot2`+`slot2-ui`
  테스트 398 passed / 0 failed, 신규 13 passed, core-dependent skip 0, fmt/clippy 종료 0이다. 최종
  판정은 `tasks/63-device-menu-app-wiring.result.md`. 다음은 Task64 전역 volume 설정 store 기반이며
  지시서가 작성됐다.

- 다음 작업 지시서 `tasks/63-device-menu-app-wiring.md` 작성 완료. Device 행을 App에 연결해 기존
  runtime `Volume`의 level/mute를 메뉴 조작과 물리 볼륨 버튼 양쪽에서 단일 상태로 유지한다.
  brightness/blue-light는 검증된 backend가 생길 때까지 `Unavailable`로 두며 platform 경로나 설정
  저장을 추측하지 않는다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\63-device-menu-app-wiring.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\63-device-menu-app-wiring.worker-result.md.`

- Task62 누적 호출 2/2 완료, 사용자 수락으로 Codex 최종 통과. 공개 DeviceMenu가 Volume,
  Brightness, BlueLight snapshot과 capability-aware 탐색, muted/unavailable/bar 표시를 제공한다. 세
  geometry×영·한 layout과 작업자 UI+i18n 223 passed / 0 failed, fmt/clippy 종료 0이다. 완성 문자열
  단위 FaceCache에 맞춰 동적 `Muted`/percentage는 최초 표시 1회 upload, 이후 반복은 0회로 계약을
  정정했다. 최종 판정은 `tasks/62-device-menu-ui.result.md`. 다음은 App의 Device 행에 Volume
  level/mute를 연결하고 brightness/blue-light는 backend 확정 전까지 Unavailable로 두는 Task63이며
  지시서가 작성됐다.

- Task62 시도 1/2는 대화형 요청이 약 2.13MB·622 messages까지 누적돼 provider 400
  `read body failed`로 중단됐다. 요청 role에는 developer가 없고, 대화형 프로세스도 models.yml 갱신
  뒤 시작된 정상 상태라 설정 문제가 아니다. `device_menu.rs`, UI 테스트, lib export, 영·한 FTL과 i18n
  테스트 변경은 working tree에 보존돼 있다. 새 컨텍스트에서 마지막 시도
  `tasks/62-device-menu-ui-recovery.md`로 이어간다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\62-device-menu-ui-recovery.md exactly. Continue the current working tree without reverting or reimplementing attempt 1. This is cumulative attempt 2/2. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\62-device-menu-ui.worker-result.md.`

- 다음 작업 지시서 `tasks/62-device-menu-ui.md` 작성 완료. D-23 Device 메뉴의 Volume, Brightness,
  Blue light를 pure UI snapshot으로 구현한다. Volume은 항상 available이고 brightness/blue-light는
  capability `Option`으로 받아 미지원 상태를 정직하게 표시하며, 실제 App/Volume/platform backend
  배선은 후속으로 분리한다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\62-device-menu-ui.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\62-device-menu-ui.worker-result.md.`

- Task61 누적 호출 2/2 완료, Codex 최종 검토 통과. CorePicker가 인게임 Core 행에 연결됐고 old Session
  checkpoint 뒤 settings를 바꾸지 않은 explicit target start, 성공 후 setting commit, target 자기
  Resume load 순서로 mGBA↔gpSP를 전환한다. checkpoint 두 단계는 독립 실행되며 target/setting/recovery
  실패 때 old core와 Resume을 복구한다. 유효 target Resume bytes, 131072-byte mGBA save RAM 재기록,
  기본 core `None` 저장과 미지 ini key 보존을 실제 core로 검증했다. 작업자 테스트 217 passed / 0 failed,
  core-picker 14 passed·skip 0, fmt/clippy 종료 0이다. 최종 판정은
  `tasks/61-core-picker-app-wiring.result.md`. 다음은 아직 동작이 없는 인게임 Device 행이며 Task62는
  platform backend와 분리한 UI 태스크로 작성한다.

- 다음 작업 지시서 `tasks/61-core-picker-app-wiring.md` 작성 완료. 인게임 Core 행과 Task60 picker를
  연결하고, 기존 Session checkpoint 뒤 settings를 바꾸지 않은 상태에서 지정 core를 먼저 시작한 후
  설정을 commit한다. checkpoint·target 시작·설정 쓰기 실패 때 변경 전 resolver와 Resume으로 기존
  core를 복구하며, 성공 시 default core는 `None`으로 저장하고 새 consumer 하나로 sink를 교체한다.
  다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\61-core-picker-app-wiring.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\61-core-picker-app-wiring.worker-result.md.`

- Task60 누적 호출 1/2 완료, Codex 최종 검토 통과. `CorePicker`가 registry 순서를 기준으로 설치된
  공식 core만 표시하고 highlight와 실제 current badge를 분리한다. 외부 core는 current를 거짓
  표시하지 않으며 empty·single·two-row 탐색, 영·한 이름/재시작 안내, 세 기기 safe area와 warm
  draw가 검증됐다. 작업자 UI+i18n 테스트 211 passed / 0 failed, fmt/clippy 종료 0이다. 최종 판정은
  `tasks/60-core-picker-ui.result.md`. 다음은 App의 Core 행·설정 저장·안전한 Session 재시작 배선인
  Task61이며 지시서가 작성됐다.

- 다음 작업 지시서 `tasks/60-core-picker-ui.md` 작성 완료. registry 후보 순서와 App이 넘길 설치
  core 목록의 교집합을 보여주는 CorePicker를 구현하고, highlight와 실제 current 표시를 분리한다.
  영·한 이름, empty, 재시작 안내와 세 기기 safe-area/warm-draw 검증을 포함한다. App 배선·설정
  저장·Session 재시작은 Task61로 분리했다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\60-core-picker-ui.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\60-core-picker-ui.worker-result.md.`

- Task59 누적 호출 2/2 완료, Codex 최종 검토 통과. 시도 1의 provider 400 뒤 새 대화형 세션에서
  working tree를 이어 완성했다. Session/App이 하나의 core resolver와 실제 core namespace를 사용하고,
  평면 legacy state는 플랫폼 기본 core로 이전된다. Resume/Fresh·퀵세이브·스위처·삭제/undo가 다른
  core state를 보거나 덮지 않는다. 작업자 `slot2` 테스트 176 passed / 0 failed, mGBA/gpSP 실코어
  skip 없음, fmt/clippy 종료 0이다. 최종 판정은 `tasks/59-core-scoped-state-routing.result.md`.
  다음은 CorePicker UI이며 지시서가 작성됐다.

- 다음 작업 지시서 `tasks/59-core-scoped-state-routing.md` 작성 완료. core 선택 resolver를 Session과
  App rescan이 공유하고, 실제 core namespace로 Resume·퀵세이브·스위처·삭제/undo를 전환한다.
  평면 legacy state는 플랫폼 기본 core로 한 번 이전하며 외부 core도 결정적 namespace를 갖는다.
  CorePicker UI와 설정 저장/재시작은 후속으로 분리했다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\59-core-scoped-state-routing.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\59-core-scoped-state-routing.worker-result.md.`

- Task58 누적 호출 2/2 완료, Codex 최종 검토 통과. 검증된 `StateNamespace`와 코어별 state
  CRUD·번호·undo API, 평면 legacy state의 충돌 없는 이전 primitive를 추가했다. 1차 검토에서 찾은
  목적 orphan PNG 결합 가능성과 디렉터리 읽기 오류 은닉은 2차에서 수정됐고 직접 회귀 테스트가
  추가됐다. 작업자 store test는 core_states 9개 포함 전체 0 failed, fmt/clippy 종료 0이며 최종
  판정은 `tasks/58-core-scoped-state-store.result.md`. 다음은 Session/App의 Resume·퀵세이브·
  스위처·삭제 취소를 실제 선택 코어 namespace로 전환하고 legacy state를 플랫폼 기본 코어로 한 번
  이전하는 Task59이며 지시서가 작성됐다.

- 다음 작업 지시서 `tasks/58-core-scoped-state-store.md` 작성 완료. 코어 전환 전에
  `States/<PLAT>/<stem>/<core-base-name>/` namespace, 평면 legacy state의 충돌 없는 이전,
  scoped 목록·읽기·쓰기·삭제·undo 기반을 `slot2-store`에 추가한다. Session/App 배선과 CorePicker는
  후속으로 분리했다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\58-core-scoped-state-store.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\58-core-scoped-state-store.worker-result.md.`

- Task57 누적 호출 1/2 완료, Codex 최종 검토 통과. gpSP·Gambatte의 pinned aarch64/host core,
  6-core registry, 플랫폼별 후보 목록, core-aware option과 unsupported 공식 조합 fallback을
  추가했다. gpSP는 MIT GBA ROM, Gambatte는 gitignore 로컬 GB/GBC ROM으로 실제 실행됐고 작업자
  완료 기준 11개가 모두 종료 0이다. Codex 검토 중 남아 있던 문서의 5-core 표기와 Gambatte 보류
  항목을 6-core 결정에 맞게 정리했다. 최종 판정은
  `tasks/57-alternative-core-foundation.result.md`. 다음은 Core 메뉴를 노출하기 전에 Resume과
  numbered state를 실제 코어별로 격리하는 Task58이며 지시서가 작성됐다.

- 사용자 결정: 게임별 대체 코어를 GB/GBC는 mGBA↔Gambatte, GBA는 mGBA↔gpSP로 제공한다.
  다음 작업 지시서 `tasks/57-alternative-core-foundation.md` 작성 완료. gpSP·Gambatte의 pinned
  aarch64/host build, 6-core registry, 플랫폼별 후보 API, 실제 선택 core별 option과 공식
  unsupported pair fallback까지 기반을 만든다. Core 메뉴 UI와 게임별 저장·재시작·state 호환 정책은
  후속으로 분리했다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\57-alternative-core-foundation.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, access hardware, or change shared configuration. Use the network only for the exact pinned GitHub repositories and libretro buildbot commands authorized by the task. Before stopping, write C:\SLOT2\tasks\57-alternative-core-foundation.worker-result.md.`

- Task56 누적 호출 1/2 완료, Codex 최종 검토 통과. pinned FCEUmm의 NES raw, compare raw,
  Game Genie, PAR, 여섯 separator와 1023-byte 경계를 검증하고 FCEUmm도 mGBA처럼 enabled entry만
  전달하도록 수정했다. 작업자 quirks 21 test/Session 27 test/clippy가 모두 종료 0이고 Session은
  mGBA 실코어로 skip 없이 실행됐다. 최종 판정은 `tasks/56-fceumm-cheat-validator.result.md`.
  8자리 Game Genie/PAR alphabet은 실제로 겹칠 수 있는데 “모호하지 않다”고 적힌 주석은 다음
  quirks 수정에서 Game Genie 우선 판정으로 바로잡는다. 당시 다음 후보였던 gpSP validator 계획은
  이후 사용자의 대체 코어 결정으로 대체됐으며, Task57에서 먼저 gpSP·Gambatte 기반을 추가한다.

- 다음 작업 지시서 `tasks/56-fceumm-cheat-validator.md` 작성 완료. pinned FCEUmm parser의 raw,
  compare raw, Game Genie, PAR 및 여섯 separator와 1023-byte 경계를 순수 validator에 추가한다.
  조사 과정에서 FCEUmm도 `retro_cheat_set`의 `enabled`와 `index`를 읽지 않는 사실을 확인했으므로,
  Task55의 잘못된 FCEUmm 가정을 바로잡아 mGBA와 함께 enabled entry만 전달하도록 한다. Session은
  이미 공통 `cheat_delivery` 경로를 사용하므로 수정 범위에서 제외했다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\56-fceumm-cheat-validator.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, use the network, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\56-fceumm-cheat-validator.worker-result.md.`

- Task55 누적 호출 1/2 완료, Codex 최종 검토 통과. mGBA만 enabled entry를 보내는 delivery policy를
  `slot2-retro::quirks`에 두고 Session이 실제 선택 dylib의 CoreId를 보존한다. 초기 적용·토글·rollback
  모두 같은 apply 경로를 쓰며 원래 file index와 Session 전체 목록을 유지한다. MIT GBA ROM과 mGBA
  실코어 serialize 비교에서 disabled=baseline, enabled≠baseline, 첫 프레임 전 양방향 토글까지
  확인했다. 작업자 quirks 17 test/Session 27 test/clippy가 모두 종료 0이고 skip은 없다. 최종 판정은
  `tasks/55-mgba-disabled-cheat-delivery.result.md`. `Session::cheats()`의 “core가 보유한 목록” 주석은
  mGBA subset 정책과 맞지 않아 다음 Session 수정에서 desired/file-order 목록으로 바로잡아야 한다.

- Task54 누적 호출 1/2 완료, Codex 최종 검토 통과. validator가 platform-aware API와 정확한 5-core
  지원 matrix를 갖고, pinned mGBA의 GBA 3형식 및 GB/GBC 5형식·ASCII/separator 경계를 검증한다.
  SNES9x 회귀는 유지되고 gpSP/FCEUmm/GPGX는 `Unchecked`다. 작업자 fmt/registry 15 test/quirks
  15 test/clippy가 모두 종료 0이다. 최종 판정은 `tasks/54-mgba-cheat-validator.result.md`.
  **다음 최우선 작업은 mGBA disabled-entry 적용 수정**이다. 실제 선택 dylib의 CoreId를 Session에
  보존하고 mGBA reset/reapply에서는 enabled entry만 전달해야 하며 rollback도 같은 정책을 써야 한다.

- Task53 누적 호출 1/2 완료, Codex 최종 검토 통과. 실제 library path에서 다섯 canonical CoreId를
  식별하는 API와 pinned SNES9x의 255-byte 경계·다섯 separator·PAR/address-value/Game Genie
  문법을 검사하는 순수 validator가 추가됐다. 공통 empty/NUL은 거부하고 다른 네 코어는 문법을
  추측하지 않아 `Unchecked`다. 작업자 fmt/registry 14 test/quirks 9 test/clippy가 모두 종료
  0이다. 최종 판정은 `tasks/53-snes9x-cheat-validation-foundation.result.md`. 다음은 나머지 pinned
  코어의 실제 parser 계약을 확인해 validator를 추가하는 작업이며 Session/App 연결은 그 뒤다.

- Task52 누적 호출 1/2 완료, Codex 최종 검토 통과. 인게임 Cheats 행이 Session 길이의
  `CheatMenu`를 열고 A로 현재 항목을 즉시 전체 재적용한다. B/MENU 복귀·pause·게임 frame 위 draw,
  범위 오류 토스트, `.cht` 불변과 손상 파일 전용 launch 토스트를 mGBA 실코어로 확인했다. 작업자
  fmt/App 11 test/i18n 25 test/clippy가 모두 종료 0이다. 최종 판정은
  `tasks/52-cheat-menu-app-wiring.result.md`. D-21의 코어별 치트 형식 사전 검증은 아직 남아 있으며
  다음 지시서는 작성하지 않았다.

- Task51 누적 호출 2/2 완료, Codex 최종 검토 통과. `CheatMenu`가 Session 목록을 복제하지 않고
  빈 상태, 선택·순환, 6행 visible window, 숨은 방향, 영·한 on/off와 힌트를 표시한다. 2회차에서
  임의 길이 description 말줄임을 문자 경계 기반 실제 폭 이분 탐색으로 바꾸고 위쪽 표시를 제목과
  첫 행 사이로 옮겼다. 작업자 fmt/UI 16 test/i18n 24 test/clippy가 모두 종료 0이며 세 geometry와
  warm redraw도 통과했다. 최종 판정은 `tasks/51-cheat-menu-ui.result.md`. 다음은 App 화면 상태와
  `Session::set_cheat_enabled` 연결이며 지시서는 아직 작성하지 않았다.

- Task50 누적 호출 1/2 완료, Codex 최종 검토 통과. Session이 카드의 ordered 치트 목록을 한 번
  읽어 첫 프레임 전에 reset 후 enabled/disabled 전부를 적용한다. 읽기·파싱·적용 오류는 부분
  Session을 남기지 않고 실패하며, accessor와 세션 범위 on/off 전체 재적용 API가 추가됐다.
  `.cht` 원본과 새 Session 기본값은 바뀌지 않는다. 작업자 fmt/session 실코어 test/clippy가 모두
  종료 0이고 23 passed / 0 failed다. 최종 판정은
  `tasks/50-session-cheat-application.result.md`. rollback 실패 방어 분기는 현재 공개 API로 유도할
  수 없어 구조로만 확인했다. 다음은 인게임 치트 UI와 App 연결 또는 D-21 코어별 형식 검증이며
  지시서는 아직 작성하지 않았다.

- Task49 누적 호출 1/2 완료, Codex 최종 검토 통과. `slot2-retro::Core`가 저장소와 독립적인
  `reset_cheats`/`set_cheat` API로 정확한 libretro 심볼을 호출한다. 코드 byte와 enabled 값을
  그대로 전달하고 내부 NUL만 호출 전에 거부하며, reset 수명주기는 호출자가 소유한다. 작업자
  fmt/retro test/clippy가 모두 종료 0이고, mGBA 실코어 테스트도 skip 없이 통과했다. 최종 판정은
  `tasks/49-libretro-cheat-bridge.result.md`. 다음은 코어별 코드 검증과 Session 적용 경계를 정할
  태스크이며 지시서는 아직 작성하지 않았다.

- Task48 누적 호출 2/2 완료, Codex 최종 검토 통과. `.cht` 로더는 공식 필드와 문자열·순서를
  보존하고 손상 파일을 전체 실패시킨다. 2회차에서 선언 count 비례 할당·순회를 제거하고 실제
  owned entry만 검증하도록 고쳤으며, 미지원 indexed metadata가 구조 판정에 미치는 영향도
  제거했다. 작업자 fmt/store test/clippy가 모두 종료 0이고 `cheats` 테스트 15개가 통과했다.
  최종 판정은 `tasks/48-retroarch-cheat-file-loader.result.md`.

- Task47 누적 호출 1/2 완료, Codex 검토 통과. 탐색·실행용 `read_settings`의 관대한 폴백은
  유지하면서, `write_settings`는 기존 ini를 읽지 못하면 원본을 건드리지 않고 오류를 반환한다.
  invalid UTF-8과 디렉터리 경로, 정상 저장·삭제, 알 수 없는 키 보존 및 실제 Display 메뉴의
  실패 토스트·Session/원본 불변을 회귀 테스트로 확인했다. 작업자 fmt/store test/App test/clippy와
  관련 diff check는 모두 종료 0이다. 판정은 `tasks/47-safe-game-settings-write.result.md`.
  `remove_file` 실패 분기를 휴대용 테스트로 직접 유발하지 못한 점은 비차단 위험으로 남겼다.
  다음 기능 태스크 지시서는 아직 작성하지 않았다.

- Task46 누적 호출 2/2 완료, Codex 최종 검토 통과. 인게임 Display 행이 게임별 scale override를
  열고, 저장 성공 뒤에만 실행 중 Session에 즉시 적용한다. 플랫폼 기본값은 다른 알려진 설정과
  알 수 없는 ini 키를 보존하면서 scale만 지우고, scale-only 파일은 App 경로에서 제거된다.
  원자적 쓰기 실패는 카드와 Session을 그대로 두고 오류를 알린다. 2회차에서 잘못된 테스트
  기대값과 검증 사각지대를 고쳤으며, `Card::write_settings`가 빈 설정 파일 삭제 오류를 성공으로
  삼키던 결함도 수정했다. 집중 8개 및 지정 fmt/test/clippy가 모두 종료 0이다. 최종 판정은
  `tasks/46-display-scale-app-wiring.result.md`. `read_settings` 읽기 실패의 빈 설정 폴백은 기존
  정책이자 별도 후속 위험이다. M4 인게임 메뉴 전체는 남은 하위 기능 때문에 미완료로 유지하며,
  다음 태스크 지시서는 아직 작성하지 않았다.

- Task45 누적 호출 1/2 완료, Codex 검토 통과. `slot2-ui::DisplayMenu`가 플랫폼 기본값,
  정수 배율, 화면 비율 맞춤, 화면 채우기를 기존 `Option<ScaleMode>`로 표현하고 현재 값 선택,
  위·아래 순환, 안전 영역 overlay, 영·한 직접 번역과 warm redraw 무업로드를 제공한다.
  작업자의 fmt/test/clippy가 모두 통과했고 검토한 diff check도 종료 0이다. 판정은
  `tasks/45-display-scale-menu-ui.result.md`에 기록했다. 다음은 Task46 App 연결·즉시 적용·게임별
  ini 저장이며 지시서는 아직 작성하지 않았다. M4 인게임 메뉴 항목은 미완료로 유지한다.

- Task44 누적 호출 1/2 완료, Codex 검토 통과. `App::time_control()`이
  `L2 되감기 > R2 또는 빨리감기 latch > 일반 속도`를 단일 판정으로 제공하고,
  `run_frame()`과 `draw()`가 이를 함께 사용한다. 시간 제어 배지는 게임 프레임 뒤이면서
  메뉴·스위처·토스트 앞에 합성되고, 비게임 화면과 세션이 없는 Playing에서는 숨는다.
  R2 즉시 표시·해제, latch, L2 우선 표시와 복귀, draw 무부작용 및 warm redraw 무업로드를
  포함한 작업자 검증이 모두 통과했다. 판정은
  `tasks/44-time-control-hud-app-wiring.result.md`에 기록했고 M4 HUD 배지 항목을 완료 처리했다.
  다음 태스크 지시서는 아직 작성하지 않았다.

- Task43 호출 1/2 완료 및 Codex 최종 판정 통과. `slot2-ui::Hud`에 App/input과 무관한
  `TimeControl::{Rewind, FastForward { speed }}`와 optional badge draw API가 추가됐다. 배지는 panel
  top-centre에 기존 plate/ink로 그려지고 en/ko 직접 정의, speed 4/8 전달, 3기기×2언어 배치,
  기존 corner HUD 불변, 반복 draw 무업로드가 직접 테스트됐다. 작업자 fmt/test/clippy와 Codex
  diff check는 종료 0이며 검증은 중복 실행하지 않았다. 상세
  `tasks/43-time-control-hud-ui.result.md`. M4 HUD 배지는 App 연결 전까지 미완료로 유지하며
  Task44 지시서를 작성했다.

- Task42 누적 호출 2/2 완료 및 Codex 최종 판정 통과. R2 물리 hold는 순간 4배속, R2 더블탭은
  세션 범위 4배속 latch로 연결됐고 두 번째 더블탭으로 해제된다. L2 rewind가 우선하며 메뉴·
  스위처 왕복은 latch를 유지하고, 모든 `stop_session` 경로와 다음 세션은 정상 속도로 초기화된다.
  게임 밖 더블탭은 다음 세션을 무장하지 않는다. 작업자 시도 2의 fmt/test/clippy와 Codex diff
  check는 종료 0이며 검증은 중복 실행하지 않았다. 상세 `tasks/42-fast-forward-latch.result.md`.
  M4 제스처 정리 항목은 완료 처리했다. 다음 HUD 작업은 Task43/44로 분리했다.

- Task41 시도 2/2 완료 및 Codex 최종 판정 통과. 선반 Tap(A)은 선택 카트의 Resume을 Playing과
  첫 프레임 전에 복원하고, Hold(A)는 세션 시작 성공 뒤 기존 Resume 상태·썸네일을 지우고 새로
  시작한다. 누락 파일은 조용히 fresh로, 손상·읽기 불가 상태는 로그와 영·한 토스트 후 usable
  fresh로 폴백한다. 제스처 시점 카트·의도 고정, hold A 입력 억제·release 해제, rescan 불린 캐시,
  draw 무-I/O와 세 기기 안전영역 힌트를 직접 테스트로 확인했다. 작업자 fmt/test/clippy와 Codex
  diff check는 종료 0이며 검증은 중복 실행하지 않았다. 상세
  `tasks/41-resume-or-fresh-launch.result.md`. M4의 resume 항목은 완료 처리했고 다음 기능 태스크는
  아직 작성하지 않았다.

- Task40 Pi 위임 스모크는 기술적으로 성공. 가재코드가 `slot2-store` aarch64 테스트 바이너리
  3개를 Pi 3B+에 복사·실행해 27 passed / 0 failed, `==> all green on aarch64`, 종료 0을
  보고했다. 따라서 Codex 비접속 → 가재코드 → Pi 검증 경로를 사용할 수 있다. 다만 경로 오류와
  300초 타임아웃 중단 뒤 세 번째 호출에서 성공해 실제 3회이며 최대 2회 규칙을 위반했다.
  상세 `tasks/40-pi-delegation-smoke.result.md`. 이후 상대경로/슬래시와 충분한 최초 타임아웃을
  사용하고 모든 실패·중단 호출을 시도로 센다.

- Task39 완료 및 Codex 최종 기능 판정 통과. 스위처 X 삭제, 단일 pending 백업, Y 복원,
  `Instant` 기준 30초 경계, 실패 재시도·기존 undo 보존, 세션 수명, 삭제 후 선택 이동, 썸네일
  캐시 무효화·지연 텍스처 반납, 영·한 힌트가 연결됐다. 작업자 fmt/test/clippy와 Codex diff
  check는 종료 0이며 검증은 중복 실행하지 않았다. 상세
  `tasks/39-state-switcher-delete-undo.result.md`. 기능은 통과했지만 실행 약 62분으로 명세의
  45분 상한을 초과한 운영 편차를 기록했다. M4 세이브 스테이트 스위처 항목은 완료 처리했다.
  Task40 Pi 위임 스모크는 사용자가 가재코드에서 별도로 실행 중이다.

- Task40 운영 스모크 지시서 `tasks/40-pi-delegation-smoke.md` 작성. 사용자의 명시적 요청에
  따라 Codex가 Pi에 직접 접근하지 않고 가재코드에 `build/pi-test.ps1 -Crates slot2-store`를
  맡겨 위임 경로를 실제 ARM 테스트로 확인한다. 소스 수정·RG SP 접근은 금지하며 원격 초기화
  부재일 때만 `-Setup` 1회와 본 테스트 재시도를 허용한다.

- 사용자 운영 결정: Raspberry Pi 3B+ 검증은 Codex가 직접 실행하지 않는다. ARM 호환성·성능
  검증이 필요하면 Codex가 대상과 성공 기준을 태스크에 명시하고, 사용자가 가재코드 대화형
  도구에 전달해 `build/pi-test.ps1` 또는 명시된 Pi 명령을 실행시킨다. RG SP·adb·Samba·SD 카드
  실기 접근 금지는 그대로다. `AGENTS.md`와 `docs/WORKFLOW.md`에 반영했다.

- 다음 작업 지시서 `tasks/39-state-switcher-delete-undo.md` 작성 완료. X로 선택 번호 상태를
  가역 삭제하고 Y로 단일 pending 백업을 30초 안에 복원하며, 절대 `Instant` 만료·재시도·
  세션 수명·선택 이동·썸네일 캐시 무효화·영/한 힌트를 App/UI에 연결하는 범위다. 저장소 원시는
  Task38 결과를 그대로 사용하며 `slot2-store` 수정은 금지했다. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\39-state-switcher-delete-undo.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\39-state-switcher-delete-undo.worker-result.md.`

- Task38 완료 및 Codex 최종 판정 통과. `slot2-store`에 번호 상태와 선택적 PNG 원본 바이트를
  메모리로 가져오는 불투명 `StateBackup`, Resume 보호 `take_state`, 카드/경로 충돌을 거부하는
  원자적 `restore_state`가 추가됐다. 정상·무썸네일·손상 PNG, 다른 카드, state/png 충돌,
  복원 실패 정리와 재시도를 신규 테스트 11개로 확인했다. 작업자 fmt/test/clippy와 Codex diff
  check 모두 종료 0이며 검증은 중복 실행하지 않았다. 상세
  `tasks/38-reversible-state-delete-store.result.md`. 다음은 삭제 버튼, 단일 pending undo, 30초
  만료와 UI 피드백의 App 배선이다. Codex 사용량은 기준선 5시간 58%/주간 56%에서 65%/57%로,
  상한 +7%p/+1%p.

- 다음 작업 지시서 `tasks/38-reversible-state-delete-store.md` 작성 완료. 번호 스테이트와 선택적
  PNG 원본 바이트를 메모리에 보관하며 삭제하고, 충돌 없이 원래 카드·카트 경로에 원자적으로
  복원하는 `slot2-store` 기반이 범위다. 앱 버튼·30초 타이머·UI는 다음 태스크로 분리했다.
  가재코드 전달 직전 Codex 사용률 기준선은 5시간 58%, 주간 56% 사용. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\38-reversible-state-delete-store.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\38-reversible-state-delete-store.worker-result.md.`

- Task37 완료 및 Codex 최종 판정 통과. 인게임 Save State 행에서 재사용 스위처로 진입하고,
  번호 상태 refresh·좌우 탐색·같은 메뉴 행 복귀·선택 상태 실제 로드가 연결됐다. 스위처 동안
  코어/오디오는 정지하고 게임 프레임 뒤에 스위처만 그린다. 성공·빈 목록·손상 상태와 재진입
  갱신을 실제 코어 테스트로 확인했다. 작업자 fmt/test/clippy와 Codex diff check 모두 종료 0이며
  검증은 중복 실행하지 않았다. 상세 `tasks/37-state-switcher-app-wiring.result.md`. 다음 기능은
  스테이트 삭제와 30초 undo다. Codex 사용량은 기준선 5시간 45%/주간 54%에서 56%/55%로,
  상한 +11%p/+1%p.

- 운영 규칙 보완: 수정 시도 보고서는 최초 내용을 반복하지 않고 변경점·새 회귀 테스트·최종
  검증만 20~30줄 delta로 남긴다. Codex 재검토도 관련 delta/diff부터 본다.
- 다음 작업 지시서 `tasks/37-state-switcher-app-wiring.md` 작성 완료. 인게임 메뉴 Save State에서
  스위처 진입, 슬롯 갱신, 좌우 탐색, 복귀, 선택 상태 실제 로드, 성공·빈 목록·실패 처리와
  pause/draw 계약이 범위다. 삭제·30초 undo는 제외했다. 가재코드 전달 직전 Codex 사용률
  기준선은 5시간 45%, 주간 54% 사용. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\37-state-switcher-app-wiring.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\37-state-switcher-app-wiring.worker-result.md.`

- Task36 시도 2/2 완료 및 Codex 최종 판정 통과. 첫 검토에서 발견한 동일색 프레임·라벨 결함을
  별도 `LABEL_INK`로 수정했고, 세 기기·영/한에서 tint 차이와 4.5:1 이상 대비를 검사하는 회귀
  테스트를 추가했다. 번호 정렬·순환 탐색·안전영역·썸네일 비율/실패 캐시·LRU/texture 반납도
  계약과 일치한다. 작업자 최종 fmt/test/clippy와 Codex diff check 모두 종료 0이며 검증은 중복
  실행하지 않았다. 상세 `tasks/36-state-switcher-ui.result.md`. 다음은 App 화면 상태와 인게임 메뉴
  진입·실제 로드 배선이며 삭제와 30초 undo는 별도 태스크로 남는다. Task36 전체 Codex 사용량은
  기준선 5시간 32%/주간 52%에서 39%/53%로, 상한 +7%p/+1%p.

- 다음 작업 지시서 `tasks/36-state-switcher-ui.md` 작성 완료. 번호 스테이트만 다루는 폴라로이드
  UI, 최신 번호 기본 선택, 좌우 순환, 썸네일 캐시·대체 화면, 빈 상태와 영·한 번역이 범위다.
  앱 배선, 실제 로드·삭제, 30초 undo는 제외했다. 가재코드 전달 직전 Codex 사용률 기준선은
  5시간 32%, 주간 52% 사용. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\36-state-switcher-ui.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\36-state-switcher-ui.worker-result.md.`

- Task35 완료 및 Codex 최종 판정 통과. Playing의 SELECT+R1은 새 번호 스테이트를 저장하고,
  SELECT+L1은 `Resume`·mtime을 배제한 최대 번호 스테이트를 실제 코어에 불러온다. 성공·빈 목록·
  저장 실패·로드 실패 피드백과 영·한 번역, 로드 뒤 resampler/rewind 초기화, InGame 화면 차단이
  구현됐다. 작업자 최종 fmt/test/clippy는 모두 종료 0이며 Codex `git diff --check`도 종료 0이다.
  충분한 증거와 diff 일치를 확인해 검증은 중복 실행하지 않았다. 상세
  `tasks/35-quick-save-load.result.md`. 스테이트 스위처 UI, 30초 undo, 인게임 메뉴 Save State 배선은
  남아 있으며 다음 기능 태스크는 아직 작성하지 않았다. Codex 사용량은 기준선 5시간 25%/주간
  50%에서 판정 시작 27%/51%로, 상한 +2%p/+1%p.

- 다음 작업 지시서 `tasks/35-quick-save-load.md` 작성 완료. SELECT+R1 새 번호 스테이트 저장,
  SELECT+L1 최신 번호 스테이트 불러오기, 성공·실패 피드백, 로드 뒤 rewind/audio 상태 정리가
  범위다. 스위처 UI와 30초 undo는 제외했다. 가재코드 전달 직전 Codex 사용률 기준선은 5시간
  25%, 주간 50% 사용. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\35-quick-save-load.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\35-quick-save-load.worker-result.md.`

- Task34 완료 및 Codex 최종 판정 통과. `Screen::InGame` 앱 배선, MENU 탭 열기와 기존 hold
  eject, 코어 정지, 마지막 프레임 위 메뉴, host/device 오디오 pause, 탐색·닫기·Continue·Eject
  동작이 구현됐다. 작업자 검증은 fmt/test/clippy 모두 종료 0이며 단위 테스트 6개와 실제 코어
  통합 테스트 2개를 추가했다. Codex는 충분한 증거와 diff 일치를 확인해 검증을 중복 실행하지
  않았다. 상세 `tasks/34-ingame-menu-app-wiring.result.md`. 다음 기능 태스크는 아직 작성하지
  않았다. Codex 사용량은 기준선 5시간 6%/주간 48%에서 판정 시작 13%/49%로, 상한 +7%p/+1%p.

- 다음 작업 지시서 `tasks/34-ingame-menu-app-wiring.md` 작성 완료. 범위는 MENU 탭으로 메뉴 열기,
  정지된 마지막 프레임 위에 오버레이, 양쪽 런타임 오디오 pause/resume, 탐색·닫기·기존 eject
  배선까지다. 세이브·치트·화면·코어·기기 하위 기능은 제외했다. 가재코드에 전달하기 직전 Codex
  사용률 기준선은 5시간 6%, 주간 48% 사용. 다음 정확한 전달 문구:
  `Read and execute C:\SLOT2\tasks\34-ingame-menu-app-wiring.md exactly. Treat that file as the complete contract. Work directly without delegation. Do not commit, push, access hardware, or change shared configuration. Before stopping, write C:\SLOT2\tasks\34-ingame-menu-app-wiring.worker-result.md.`

- Task33 완료 및 Codex 재검토 통과. 가재코드 대화형 도구가 Task32의 부분 구현을 보존하면서
  인게임 메뉴 번역 8개와 전용 테스트 4개를 추가하고 모듈 순서를 수정했다. Codex가 다시 실행한
  `cargo fmt --all -- --check`, `cargo test -p slot2-ui -p slot2-i18n`,
  `cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings` 모두 종료 코드 0.
  상세는 `tasks/33-ingame-menu-ui-recovery.result.md`. 다음 M4 작업은 아직 앱에 연결되지 않은
  `InGameMenu`의 화면 상태·입력 배선이며, 새 작업 지시서를 먼저 작성한다.

- 사용자 결정: 코딩 에이전트는 사용자가 가재코드 대화형 도구에서 직접 실행하고, Codex는 작업
  지시서와 최종 판정을 담당한다. Task32 부분 코드 회수는
  `tasks/33-ingame-menu-ui-recovery.md`로 완료했다. 자동 OpenCodex 재시도 없음.

- Task32 OpenCodex 구현 2회 실패로 자동 처리 중단. 두 시도 모두 후속 요청에서 502 provider connect timeout. 두 번째 시도가 `in_game_menu.rs`와 `lib.rs`를 부분 수정했으나 번역·전용 테스트·fmt·clippy·결과 보고서가 없어 최종 판정 실패. 부분 코드는 보존. 상세 `tasks/32-ingame-menu-ui.result.md`. 다음은 사용자 선택: 메인 Codex 직접 완성 / OpenCodex 연결 안정화 진단 / 한도 리셋까지 보류.

- 개발 재개: Task27 남은 clippy 종료0, dist 종료0/`==> done`으로 자동 검증 완료. Task32 `tasks/32-ingame-menu-ui.md` 작성. 다음은 사용자가 별도 OpenCodex 작업 세션에서 Task32를 실행하고 결과 파일 경로를 이 Codex 대화에 전달하는 것. Task32 시작 전 Codex 계정 스냅샷은 5시간79%, 주간43%이며 다른 작업/반올림이 섞여 단독 비용으로 단정하지 않는다.

- Task31: 전체 운영 지침 `docs/WORKFLOW.md` 작성 완료. 기본은 Codex 명세 → 사용자 OpenCodex 실행 → 결과 회수·판정. AGENTS에도 반영했다. 다음 개발 요청에서는 작은 태스크 명세와 전달 문구를 작성하고 응답을 끝낸다. 자동 워커 재시도는 하지 않는다.

- 최신 요청 Task30: OpenCodex를 작업 도구로, 수동 중계와 Claude식 에이전트 절감 가능성을 조사 완료. 권장안은 Codex 명세 → 사용자 OpenCodex 실행 → 완료 후 Codex 판정. 상세 `tasks/30-opencodex-workflow-options.result.md`. 유료 실험/설정 변경/Task29 재시도 없음.

- 사용자 요청으로 기능 개발을 멈추고 작업 방식 개선 검증(Task 29)을 수행했다. 규칙 축약 비교는 성공, 자동 스크립트 구현은 2회 실패하여 중단했다.
- main / a8cb4af, 미커밋 변경 있음. 커밋·푸시는 요청받지 않았다.
- Task 27 시계 구현: fmt 종료 0, workspace 372 passed / 0 failed, 결과 줄 54개, 종료 0. clippy와 dist는 아직 미완이다.
- B.AI 실행 경로: OpenCodex localhost:10126 → bai/glm-5.3-flash. 메인 Codex 및 공용 YAML 변경 없음. 프록시 자동 시작은 미설정.
- Task 29 동일 읽기 비교: 기존 input 41,199, 플러그인/스킬 제외 41,111, 규칙 축약 추가 32,606. 모두 정확. 사용 요금/구독 한도 절감률은 미측정.
- Task 29 build/run-bai-worker.ps1 및 build/verify-workspace.ps1은 미검증 초안, 사용 금지. B.AI 구현 2회 모두 502 connect timeout 종료1, 스킬 설정 스키마 오류도 남아 있다. 세 번째 자동 수정 금지, 사용자 지시 필요.
- 규칙은 AGENTS.md, 예전 장문 설명은 docs/AGENT-OPERATIONS-LEGACY.md. 최신 실행 방침은 AGENTS.md가 우선한다.
- 상세 기록: tasks/24~29 결과 파일. 워커 최초 성공 명령은 tasks/26-opencodex-worker.result.md 최상단.
- 다음 확인: `Get-Content tasks/29-lean-worker-validation.result.md`. 최종 워커 보고서는 생성되지 않았다.
- 원래 개발 재개 시 남은 명령: `cargo clippy --workspace --all-targets -- -D warnings`, `powershell -File build/dist-device.ps1`.
- M4 착수. Task33에서 인게임 메뉴 UI 구성요소까지 완료했으며 앱 상태·입력 배선은 미완료다.
  아래의 'green' 등 워크스페이스 전체 수치는 명시된 옛 커밋 기준이다.

---
## 1. 지금 어디인가

**M3(선반 UI와 스킨 시스템) 코드 작업이 끝났고 M4를 진행 중이다.** Task35에서 빠른 저장과
빠른 불러오기를 완료했다. 다음 기능 태스크는 스테이트 스위처 UI, 30초 undo, 인게임 메뉴의
Save State 배선 가운데 계약과 의존성을 확인해 작은 단위로 작성한다.

워크스페이스는 green 이다 — `bbd423d` 기준 52개 바이너리 **369 passed / 0 failed**,
clippy 종료 코드 0.

---

## 2. 최근 커밋 (새 것부터)

| 커밋 | 내용 |
|---|---|
| `bbd423d` | 조사: 시각이 화면 밖으로 나가는 지점 — 지금은 없다 (Task 23) |
| `9a03d06` | M3: 카드 이미지 재빌드, 시간대 설정은 M5 로 (Task 22) |
| `610a815` | chore: 핸드오프 파일(`.claude/resume.md`)을 저장소에 넣는다 |
| `4c09a0a` | M3: 배터리·시계 HUD 완료 표시 |
| `4ce6d3f` | M3: task 21 검증 결과 — 워크스페이스 green |
| `2f4ccd2` | M3: HUD 드로잉 + 앱 배선 (Task 21) |
| `d564a1d` | M3: `slot2-platform` 게이지·시계 (Task 20) |

---

## 3. M3 에 남은 것 — 하나뿐이고 막혀 있다

**코어 선택 화면.** 플랫폼당 코어가 1개뿐이라 고를 게 없다. **gpSP 빌드가 선행**이고,
그건 별도 작업이다. `docs/MILESTONES.md` 에 기록돼 있다.

---

## 4. 시간대 — 결정 확정, 후속 작업 남음

### 확정된 것

**BaseOS(기기 시스템 시각)는 UTC 로 둔다. 시간대는 SLOT2 가 표시할 때만 적용한다.**
사용자 결정(2026-09-24). `crates/slot2-platform/src/clock.rs` 가 이미 이 구조다 —
`utc_now()` 가 순수 UTC 이고 `now_local()` 이 그 위에 오프셋을 얹는다.

파생 계약 (2026-09-25 Task 27에서 `docs/DECISIONS.md` D-25로 등재):

> **저장되는 시각은 전부 UTC 고정, 변환은 표시 직전에만 한다.**
> - 시각이 파일명에 들어가는 기능이 **생긴다면** 이름은 UTC 로 짓고 `Z` 로 드러낸다.
>   (2026-09-24 현재 그런 기능은 없다.)
> - mtime 은 SLOT2 가 쓰는 값이 아니라 커널이 찍는 값이다. SLOT2 는 mtime 을 **UTC 로 읽고**,
>   보여줄 때만 오프셋을 더한다. **mtime 을 로컬로 보정해 파일에 되쓰지 않는다.**

### 왜 런처에 KST 를 못 박았나 (Task 22, 실패로 종결)

**주입할 런처가 존재하지 않는다.** BaseOS 는 `cd $SD; exec /lib/ld-linux-aarch64.so.1
System/frontend` 로 동적 로더에 ELF 를 직접 먹이고 **환경변수를 일절 주지 않는다**
(`docs/DESIGN.md:37` 이 이걸 계약으로 못박고 있다). 로더에 인자로 넘기는 구조라
`System/frontend` 를 셸 스크립트로 바꿔치기하는 우회도 **동작하지 않는다** — 로더는 셔뱅을
해석하지 않는다.

→ **따라서 설정 파일·화면 배선 전에는 기기의 기본 UTC 오프셋이 0이다.** 실기에서 시계가 KST 대비 9시간 이르게
보이는 것은 **버그가 아니다.** M5 설정 화면이 정규 해법이다.

### 조사 결과 요약 (Task 23, 전문은 `tasks/23-time-surface-survey.result.md`)

- 저장소에서 **벽시계를 읽는 곳은 두 곳뿐**이다 — `clock.rs:30` `utc_now()`(HUD 전용),
  `card.rs:282-288` mtime 되읽기(소비자 0). 나머지 `Instant::now` 는 전부 단조 시계라 무관.
- **화면 밖으로 나가는 시각은 하나도 구현돼 있지 않다.** 스크린샷 기능 없음(계획서에도 없음),
  세이브스테이트 헤더 없음, 로그 타임스탬프 없음(로깅이 `eprintln!` 뿐), 플레이 기록 없음.
- 그래서 **지금 고칠 기존 코드는 0건**이고, 위 계약은 "앞으로 생길 것"에 대한 것이다.
  **소비자가 0인 지금이 계약을 박기에 가장 싼 시점이다.**

### PC 에서 시각이 어긋나 보이는 진짜 원인 (파일명이 아니다)

`/mnt/sdcard` 가 **vfat** 이고 마운트 옵션에 `tz=`/`time_offset=` 이 없다
(`docs/device/rgsp-diag.txt:24`). 그래서 디스크의 FAT 타임스탬프는 UTC 인데 **Windows 는
FAT 타임스탬프를 로컬로 해석한다.** 15:00 KST 에 저장한 파일이 탐색기에서 06:00 으로 보인다.

파일명을 로컬 시각으로 바꾸면 이름은 15:00, mtime 은 06:00 이 되어 **한 화면 안에서 두 시각이
9시간 어긋나는 더 나쁜 상태**가 된다. UTC 고정이 mtime 과 일치하므로 정합적이다.

---

## 5. 시계 선행 작업과 다음 단계

2026-09-25 Task 27: (1)~(5) 구현·문서 반영 완료, 최종 검증 결과는 이 문서 상단과
`tasks/27-clock-runtime.result.md` 참조. 아래 설명은 원래 문제와 변경 근거다.
남은 실제 개발은 (6) M4 태스크 분할 및 M5 설정 파일·화면 배선이다.

### (1) 완료 — `clock.rs` 런타임 오프셋

기존 `OnceLock<i32>` 고정값을 `AtomicI32`로 교체하고 범위를 검증하는
`set_utc_offset_min()`을 추가했다. 초기 환경값은 compare_exchange로 게시해 이미 설정한
런타임 값을 덮어쓰지 않는다. 설정 화면에서 호출하는 배선은 아직 없다.

주석은 "환경은 실행 중 바뀌지 않는다"를 근거로 드는데, 이 근거는 **오프셋의 원천이
환경변수일 때만** 참이다. M5 가 원천을 바꾸는 순간 무효가 된다.

### (2) 완료 — HUD의 `is_set`을 UTC 값에 적용

기존 `clock::is_set(local)`은 UTC로 정의된 `SET_AFTER`와 비교하면서 경계가 밀렸다.
이제 단일 UTC 샘플을 `hud_local()`에 전달해 유효성을 먼저 판정한 뒤 표시 오프셋을 적용한다.

### (3) 완료 — 부팅 배너 시각 정보

`main.rs` 부팅 배너에 `utc_offset_min=<n>`과 `utc_now=<epoch>`를 추가했다.
새 의존성 없음. 부팅 이후 개별 로그마다 타임스탬프를 붙이는 기능은 미구현이다.

### (4) 완료 — M5 계획에 `System/slot2.ini` 읽기·쓰기·즉시 반영 명시

기기에 env 를 넣을 자리가 없으므로(4항), 오프셋은 카드의 설정 파일에서 와야 한다.
`System/slot2.ini` 는 **경로만 존재하고 읽는 코드가 없다**(`card.rs:141-143`, 참조는 테스트
1건뿐). 그 배선도 M5 몫이다.

### (5) 완료 — D-25 UTC 저장 계약 등재

4항의 초안을 D-25로 올리고 DESIGN의 카드 시각 계약에도 반영했다.

### (6) M4 착수

`docs/MILESTONES.md` 의 M4 를 읽고 태스크로 자른다. **크레이트 경계로 자르고 한 태스크에
`todo!()` 15개 안쪽**(`AGENTS.md` 3항).

---

## 6. 알려진 이상한 점 — M4 에서 의식적으로 결정해야 할 것들

1. **`atomic_write` 가 동일 바이트를 건너뛴다** (`crates/.../atomic.rs:19-24`, `Ok(false)`).
   카드 마모를 막는 올바른 동작이지만 **부수효과로 mtime 도 갱신되지 않는다.** 내용이 같은
   세이브스테이트를 다시 저장하면 `StateSlot.modified` 는 옛 시각을 유지한다. M4 폴라로이드
   스위처("언제 저장했는지")와 M4 `manifest.json`(mtime 으로 동기화 충돌 판정,
   `DESIGN.md:349-352`)이 **둘 다 영향을 받는다.** 버그라고 단정하지는 않는다 — 내용이 같으면
   같은 상태라는 해석도 성립한다. **M4 에서 의식적으로 결정해라.**

2. **`diag` 리포트에 현재 시각이 없다** (`diag.rs`). 첫 부팅 하드웨어 조사인데 언제 찍혔는지
   파일 안에서 알 수 없다. 카드에 `System/slot2-diag.txt` 로 남으므로 여러 번 부팅하면 어느
   것이 최신인지 내용으로 구분이 안 된다.

3. **`RETRO_MEMORY_RTC` 가 매핑만 되고 안 쓰인다** (`ffi.rs:64`, `host.rs:311,328`).
   RTC 를 가진 게임(포켓몬 루비/사파이어, 보크타이 등)은 코어가 자체적으로 libc `time()` 을
   부르므로 **게임 안의 시각은 SLOT2 의 오프셋과 무관하게 UTC 로 흐른다.** 사용자에게는
   "게임 안 시계가 9시간 어긋난다"로 보인다. **미확인 — 실기에서 RTC 게임을 돌려 봐야 확정된다.**

4. **`System/slot2.log`** 는 `docs/DESIGN.md:38` 이 회전 기록을 약속했는데 **0줄도 구현돼 있지
   않다.** 지금은 stderr → `/tmp/frontend.log` 뿐이고 그건 휘발이라 사용자가 PC 에서 볼 수 없다.

---

## 7. 실기 테스트 — 미실시, **사용자만 한다**

카드 이미지는 **`610a815` 기준으로 재빌드 완료**다 (`C:\SLOT2\dist-device\`,
`System/VERSION.txt` = `SLOT2 0.1.0 (610a815)`). Task 16~21 이 들어가 있다. 재빌드가 필요하면:

```
powershell -File build/dist-device.ps1
```

확인할 것:

- **시계가 UTC 로 보이는 건 정상이다** (4항). 버그로 오진하지 말 것
- 삽입 애니메이션 / 안착한 카트 / 월페이퍼 잔상
- **삽입·배출 효과음이 그림과 맞는지** (리드 0.097s)
- GBA 피치·속도 / MD B 버튼 / L2 되감기(`one per N frames`, 6이면 정상) / GBA 색보정 세기
- **배터리 HUD 가 실기에서 실제로 퍼센트를 읽는지**, 저전력 시 잉크 색 변화, 충전 중 볼트 표시
- HUD 가 패널 모서리에 고정되는지
- `System/frontend` ELF 교체 후 respawn 정상 여부, `/tmp/frontend.log` 에 패닉 없는지
- 카드를 PC 에 꽂아 `Saves/`·`States/` 파일의 탐색기 표시 시각이 실제보다 9시간 이른지 (4항 검증)
- RTC 를 쓰는 GBA 게임에서 게임 내 시각이 UTC 로 흐르는지 (6-3)
- TF2 카드를 쓴다면 그 파티션이 vfat 인지 exfat 인지

배터리 경로는 V-2 에서 확인됐다: `/sys/class/power_supply/axp2202-{battery,usb}`.
코드는 경로를 박지 않고 `type=Battery` + `capacity` 로 걸러 이름순 첫 항목을 고른다.

**미해결**: 밝기 경로(`/sys/class/backlight` 없음, `pwmchip0` 뿐) · HDMI 감지(커널 4.9 에
`/sys/class/drm` 없음, D-22)

---

## 8. 환경 메모

- `BAI_API_KEY` 는 프로세스·유저 스코프 모두 설정돼 있다 (35자). **값을 출력하지 마라.**
- `GJC_CONFIG_DIR` 은 `~/.gjc-bai` 로 해석된다. 상대경로 우려는 기우였다.
- **`fallbackChains` 는 없다.** config 가 v2 스키마로 바뀌며 `modelRoles` 배열로 대체됐다.
- **워커 출력은 버퍼링돼 종료 시에 떨어진다.** 도는 동안 0바이트인 것은 정상이고, 죽이면
  날아간다. 그걸 근거로 멀쩡한 워커를 세 번 죽여 93분을 버렸다. 진행 신호는 `git status --short`.
- 서브에이전트가 **세션 레이트리밋(429)** 으로 죽은 적이 여러 번 있다. Task 23 도 보고서를
  쓴 직후 429 로 종료됐다. **그때는 기다린다.**
- 직전 담당(Claude Code)의 핸드오프 파일은 `.claude/resume.md` 다. 훅이 그 경로로 고정돼 있어
  파일명을 바꿀 수 없다. **Codex 는 이 파일(`docs/HANDOFF-CODEX.md`)을 갱신해라.**
