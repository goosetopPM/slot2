# SLOT2 — 재개 포인터

갱신: 2026-10-04.

- 필수 규칙: AGENTS.md
- 전체 운영 지침: docs/WORKFLOW.md
- 현재 상태: docs/HANDOFF-CODEX.md 최상단

Task109는 누적 2/2로 최종 통과했다. 전체 commit 후보 위생, fmt, 973 workspace tests, host/device
clippy, six-core/full-dist/release package gate를 오프라인으로 통과했고 빈 root `$env`를 안전하게
제거했다. local/hosted `VERSION.txt`도 BOM 없는 strict UTF-8·LF 3줄로 통일하고 엄격한 회귀 검사를
추가했다. 최종 판정은 `tasks/109-offline-release-candidate-audit.result.md`다. 다음은 사용자가 public
commit에 task/operations history를 포함할지 결정하고 commit을 명시 승인해야 한다. clean committed
rebuild, remote/push/tag와 hosted/fresh-card/hardware acceptance는 그 뒤에 남는다. 다음 작업 지시서는
아직 작성하지 않았다.

Task107 1차는 누적 1/2 수정 필요다. 번역 issue form의 bare `docs/TRANSLATING*.md` 링크는 issue URL
문맥에서 깨진다. 마지막 수정 지시서는 `tasks/107-issue-forms-translation-contribution-attempt2.md`이며
링크만 `../blob/main/docs/...`로 바꾸고 worker result를 누적 2/2로 갱신해야 한다.

다음 태스크는 `tasks/107-issue-forms-translation-contribution.md`다. 버그·실기 호환성·번역 조율용
GitHub issue form과 영문 canonical/한글 동등 번역 기여 가이드를 만들고 한국어 README 링크를 한글
가이드로 연결한다. 공개 릴리스나 공식 support를 가정하지 않으며 코드·언어팩·workflow는 바꾸지 않는다.

Task106은 누적 2/2로 최종 통과했다. 영문/한글 공개 README의 신규 설치, 지원 범위, 조작, FAQ,
번역·라이선스 안내가 완료됐고 전체 쓰기 원자성 과장도 제거했다. 최종 판정은
`tasks/106-public-readme-en-ko.result.md`다. M7에는 hosted release acceptance, issue template/번역
기여 진입점, 기존 카드 migration guide가 남아 있으며 다음 지시서는 아직 작성하지 않았다.

Task106 1차는 누적 1/2 수정 필요다. 두 README가 모든 SLOT2 쓰기를 원자적이라고 단정하지만 진단과
probe 보고서는 직접 쓰기다. 마지막 수정 지시서는 `tasks/106-public-readme-en-ko-attempt2.md`이며 해당
영문/한글 문단만 좁히고 worker result를 누적 2/2로 갱신해야 한다.

다음 태스크는 `tasks/106-public-readme-en-ko.md`다. 영문 `README.md`와 동등한 한글
`README.ko.md`에 구현된 기능, 신규 1장/2장 설치, 카드 구조, 조작, PC preview, FAQ, 번역·라이선스
경계를 기록한다. 공개 릴리스/hosted CI/광범위 실기와 기존 카드 마이그레이션은 완료로 주장하지
않으며 코드와 packaging/workflow는 변경하지 않는다.

Task105는 누적 2/2로 최종 통과했다. reusable device artifact, offline release pair packager와 guarded
draft tag release 구현은 완료됐고 실제 hosted CI/tag/draft 확인과 M7 문서는 남아 있다. 다음 작업
지시서는 아직 작성하지 않았다.

Task104는 누적 1/2로 최종 통과했다. CI host Cargo 준비와 device core checkout cache-hit 경로의 로컬
검증은 완료됐고, 실제 GitHub Actions의 cache miss/hit 및 artifact 확인은 사용자 통제 acceptance로
남아 있다. 다음 작업 지시서는 아직 작성하지 않았다.

기본 방식은 Codex 명세 작성 → 사용자 OpenCodex 실행 → Codex 최종 판정이다.
과거 CLAUDE.md의 역할 분리·자동 위임 절차를 그대로 적용하지 않는다.
현재 태스크와 미완 검증, 미커밋 상태는 핸드오프에서 확인한다.
Task103은 누적 2/2로 최종 실패했고 자동 재시도를 중단했다. 로컬 Rust notice bundle·dist·zip은
성공했지만 CI device job의 host Cargo cache를 채우지 않아 offline packager가 fresh runner에서
실패한다. 기존 device core cache도 vendor만 보존해 cache hit 시 source checkout이 없다. 별도 후속
태스크와 실제 CI run이 필요하며 세 번째 Task103 호출은 금지한다.
Task103 1차는 누적 1/2 실패다. device runtime crate 66개 중 세 개가 license 선언만 하고 원문 파일을
crate package에 넣지 않아 원 계약대로 중단했다. source identity가 name+version 단계에서 합쳐질 수
있는 문제도 남았다. 마지막 수정 지시서는 `tasks/103-rust-runtime-license-sbom-attempt2.md`이며 세
crate를 투명한 declared-only로 포함하고 source-aware identity를 고친 뒤 dist·zip·CI까지 연결해야 한다.
다음 태스크는 `tasks/103-rust-runtime-license-sbom.md`다. device feature의 normal runtime dependency
closure만 대상으로 원본 crate license/notice, 결정적 project-local SBOM과 hash manifest를 만들고
`System/licenses/rust/` local dist·zip·CI에 연결한다. dev/build/host-only dependency와 tag release는
후속이다.
Task102는 누적 2/2로 최종 통과했다. rollback 실패 시 이전 backup을 보존하고, recursive copy 중간
실패의 부분 candidate도 정리하거나 잠금 시 정확한 경로를 경고한다. failure harness 8종과 affected
packager/dist 검증이 성공했고 Task101 bundle과 28/28 파일 byte 동일하다. 최종 판정은
`tasks/102-safe-source-bundle-rollback.result.md`다. 다음은 M7 Rust dependency notice/SBOM 또는 tag
release이며 지시서는 아직 작성하지 않았다.
Task102 1차는 누적 1/2 수정 필요다. rollback과 backup 보존은 정상이나 candidate 복사가 중간 실패하면
ownership flag가 아직 false라 부분 candidate를 정리·경고하지 않는다. 마지막 수정 지시서는
`tasks/102-safe-source-bundle-rollback-attempt2.md`이며 기존 보고서를 누적 2/2로 갱신해야 한다.
다음 태스크는 `tasks/102-safe-source-bundle-rollback.md`다. Task101의 세 번째 호출이 아니라 별도
후속이며, rollback 실패 시 이전 정상 backup을 절대 삭제하지 않고 Windows 잠금으로 정리할 수 없는
안전한 잔여물은 정확한 복구 경로와 함께 허용한다. Rust 검증은 반복하지 않고 affected PowerShell
packaging/dist 경로와 failure harness만 검증한다.
Task101은 누적 2/2로 최종 실패했고 자동 재시도를 중단했다. 일반 bundle·dist·973 tests는 성공했지만,
rollback rename 실패 시 `finally`가 이전 정상 bundle의 backup까지 삭제할 수 있다. 잠긴 candidate가
남는 현상도 2차의 잔여물 없음 계약과 충돌한다. `tasks/101-core-license-source-bundle.result.md`를 보고
사용자가 계약 조정 또는 별도 후속 태스크를 지시해야 한다. 세 번째 Task101 호출은 금지한다.
Task101 1차는 누적 1/2 수정 필요다. license/source bundle과 workspace 973개 검증은 통과했지만
packager가 기존 output을 먼저 삭제한 뒤 staging을 복사해 교체 실패 시 정상 bundle 소실·부분 output이
가능하다. 마지막 수정 지시서는 `tasks/101-core-license-source-bundle-attempt2.md`이며 같은 parent
candidate + backup/rollback rename으로 교체만 수정하고 worker result를 누적 2/2로 갱신해야 한다.
다음 태스크는 `tasks/101-core-license-source-bundle.md`다. root/upstream/core license 원문과 pinned
pristine source archive, build recipe/patch를 deterministic하게 만들어 local dist·zip·CI artifact에
동봉한다. Rust dependency notice와 tag release는 후속이다.
Task100은 누적 1/2로 최종 통과했다. 여섯 core의 단일 manifest와 local dist·zip·ADB·CI exact-six
gate를 연결했고 gpSP native/device build 인자를 분리했다. workspace 973 passed, device check·fmt·
clippy·full dist·zip·negative staging이 성공했다. 다음은 M7 license/source archive와 tag release
기반이며 지시서는 아직 작성하지 않았다.
Task99는 누적 1/2로 최종 통과했다. 기존 MIT PCM을 보존하고 7개 플랫폼의 insert/eject speed·gain
profile, styled render와 transformed lead cue를 App에 연결했다. workspace 971 passed, device
check·fmt·clippy·배포가 모두 성공했다. M6의 cart/port/curve/sfx 구현은 완료됐고 실기 청감과 나머지
M6 실기 Acceptance가 남았다. 다음 작업 지시서는 아직 작성하지 않았다.
Task98은 누적 1/2로 최종 통과했다. 공용 animation/load/SFX clock을 유지하면서 7개 플랫폼별
insert/eject curve를 `PlatformSkin`과 ShelfView/App에 연결했다. 플랫폼별 sfx와 실기 확인이 남았고
다음 태스크 지시서는 Task99로 작성됐다.
Task97은 누적 1/2로 최종 통과했다. 7개 플랫폼별 port trim이 `PlatformSkin`과 `ShelfView`의
front/cache 경로에 연결됐다. 중앙 opening과 기존 occlusion은 유지되며 curve/sfx와 실기 확인이
남았다. 다음 태스크 지시서는 아직 작성하지 않았다.
Task96은 누적 1/2로 최종 통과했다. MD·SMS 독립 카트리지 shell/detail이 추가돼 7개 플랫폼 모두
고유 카트를 가지며 fallback은 없다. 플랫폼별 port, 삽입 곡선·효과음과 실기 확인이 남았다.
다음 태스크 지시서는 아직 작성하지 않았다.
Task95는 누적 1/2로 최종 통과했다. NES·SNES 독립 카트리지 shell/detail과 계약 검증이 추가됐고
MD·SMS fallback, 플랫폼별 port, 삽입 곡선·효과음은 남아 있다. 다음 태스크는 아직 작성하지 않았다.
Task61은 누적 2/2, Task62는 누적 2/2로 최종 통과했다. Task63도 누적 1/2로 최종 검토 통과해
Device 행의 runtime Volume level/mute App 배선이 완성됐다. 메뉴 조작과 물리 volume 키가 같은
`App.volume`을 사용하며 brightness/blue-light는 backend가 확정되기 전까지 Unavailable이다.
작업자 검증은 398 passed / 0 failed, 신규 Device App test 13 passed, fmt/clippy 종료 0이다.
Task64도 누적 1/2로 최종 통과했다. `System/slot2.ini`의 volume level safe store가 완성됐고
작업자 store 검증은 66 passed / 0 failed, 신규 11 passed, fmt/clippy 종료 0이다. mute는 runtime으로
유지한다. Task65도 누적 1/2로 최종 통과했다. 시작 시 level load, 750ms debounce 저장과 세 power
exit의 즉시 flush가 완성됐고 작업자 `slot2` 전체 검증은 219 passed / 0 failed, fmt/clippy 종료 0이다.
다음 태스크는 `tasks/66-game-shader-settings-store.md`다. 게임별 shader preset의 카드 표기와
safe-write를 추가하되 key 부재의 플랫폼 기본값 상속과 `shader = none`의 명시적 Off를 구분한다.
Task66은 누적 1/2로 최종 통과했다. 작업자 store 테스트 79 passed / 0 failed, 신규 13 passed,
downstream check와 fmt/clippy 종료 0이다. 다음 태스크는 `tasks/67-gfx-shader-effects.md`다.
`slot2-gfx`의 개별 game image draw에 네 내장 단일 패스 effect를 추가하고 UI/present와 격리한다.
Task67은 누적 1/2로 최종 통과했다. 기본 gfx 테스트 25 passed / 0 failed, device/downstream check와
clippy 종료 0이고 Intel GLES2 실제 GL test도 1 passed다. Mali G31 실기 화질은 미확인이다. 다음
태스크는 `tasks/68-session-shader-routing.md`다. Session의 store preset → gfx effect 변환을 한 곳에
두고 game frame draw만 effect API로 보낸다. Task68은 누적 1/2로 최종 통과했다. 작업자 session
37 passed / 0 failed, lib 33 passed / 0 failed, core skip 0이며 downstream check와 clippy 종료 0이다.
다음 태스크는 `tasks/69-platform-shader-defaults.md`다. retro registry에 독립적인 플랫폼 shader 의미
타입을 두고 GB/GBC/GBA는 Lcd3x, NES/SNES/MD/SMS는 ZfastCrt를 기본값으로 정한다. Session은 game
setting key 부재만 이를 상속하고 explicit Off와 네 preset override를 우선한다. UI/App 저장 연결은
그 뒤로 둔다. 1차 production 집중 검증은 통과했지만 `slot2 --tests`에서 기존 App 통합 테스트 8개가
새 GBA `ImageEffect(Lcd3x)`를 찾지 못해 실패했다. 최종 판정은 누적 1/2 미통과이며 다음 실행은
`tasks/69-platform-shader-defaults-attempt2.md`다. production을 바꾸지 않고 여덟 game-frame 단언만
정확한 Lcd3x effect 계약으로 갱신한다. Task69는 2차에서 이를 완료해 누적 2/2 최종 통과했다. 전체
`slot2 --tests`는 231 passed / 0 failed / 0 ignored, core skip 0이고 check/clippy도 종료 0이다. 다음
태스크는 `tasks/70-shader-menu-ui.md`다. Platform default, explicit Off와 네 effect를 구분하는 독립
ShaderMenu UI와 영문/한글·safe-area 계약을 추가한다. Display 진입과 App live apply/persistence는
Task71로 분리한다. Task70은 누적 1/2 최종 통과했다. 작업자 `slot2-ui`+`slot2-i18n` 전체는
231 passed / 0 failed / 0 ignored이고 clippy 종료 0이다. 다음 태스크는
`tasks/71-shader-menu-app-wiring.md`다. Display에 Shader 진입 행을 추가하고 저장 성공 뒤 Session에
즉시 적용하며 failure rollback과 기존 scale 흐름을 함께 검증한다. Task71은 누적 1/2 최종 통과했다.
작업자 세 crate 전체는 472 passed / 0 failed / 0 ignored, core skip 0이고 clippy 종료 0이다. 다음
태스크는 `tasks/72-overscan-menu-ui.md`다. Platform default, explicit crop와 full image를 구분하는
독립 OverscanMenu UI와 번역·safe-area 계약을 추가한다. NES 조건부 App 진입과 저장/live crop은
Task73으로 분리한다. Task72는 누적 1/2 최종 통과했다. 작업자 `slot2-ui`+`slot2-i18n` 전체는
239 passed / 0 failed / 0 ignored이고 clippy 종료 0이다. 다음 태스크는
`tasks/73-overscan-menu-app-wiring.md`다. registry crop이 있는 platform에만 Display 진입 행을 추가하고
저장 성공 뒤 Session crop을 즉시 적용하며 합성 NES/FCEUmm UV와 rollback을 검증한다.
Task73은 누적 1/2로 최종 통과했다. 작업자 세 crate 전체는 492 passed / 0 failed / 0 ignored,
core skip 0이고 clippy 종료 0이다. registry crop이 있는 플랫폼만 Display Overscan 행을 제공하고
저장 성공 뒤 Session에 즉시 적용한다. 다음 태스크는 아직 작성하지 않았으며 Overlay 선택은
runtime/store 계약부터 후속으로 설계해야 한다.
다음 태스크는 `tasks/74-game-overlay-settings-store.md`다. 게임별 ini에 overlay의 플랫폼 기본값
상속/사용/명시적 Off 세 상태와 safe-write를 추가한다. PNG 탐색·decode·renderer·UI/App 배선은
후속 태스크로 분리한다.
Task74는 누적 1/2로 최종 통과했다. 작업자 store 전체는 89 passed / 0 failed / 0 ignored,
downstream test check와 clippy 종료 0이다. overlay의 세 저장 상태와 safe-write가 완성됐다. 다음은
geometry별 PNG resolver와 runtime renderer 기반이며 지시서는 아직 작성하지 않았다.
다음 태스크는 `tasks/75-overlay-asset-layer.md`다. 카드 geometry별 PNG와 compile-time 내장 PNG의
우선순위·fallback, 엄격한 RGBA decode와 texture 수명을 독립 layer로 구현한다. 실제 sample과
Session/App/UI 배선은 후속으로 분리한다.
Task75는 누적 1/2로 최종 통과했다. 집중 테스트 19 passed / 0 failed / 0 ignored, host tests check,
device feature check와 clippy 종료 0이다. geometry별 resolver·PNG decode·texture 수명 기반이 완성됐다.
다음은 Task74 setting 해석과 Session/App의 game→overlay→UI draw·시작/eject 수명 배선이며 지시서는
아직 작성하지 않았다.
다음 태스크는 `tasks/76-overlay-app-runtime-wiring.md`다. 최초 launch에서 overlay setting/source를
해석하고 game frame 뒤·HUD/UI 앞에 draw하며 core switch/recovery와 stop/eject의 texture 수명을
배선한다. 메뉴·저장과 실제 내장 sample은 후속으로 분리한다.
Task76은 누적 1/2로 최종 통과했다. 작업자 `slot2` 전체 287 passed / 0 failed / 0 ignored,
core skip 0이고 device check와 clippy 종료 0이다. game→overlay→HUD/UI draw와 launch/core
switch/recovery/stop 수명이 완성됐다.
다음 태스크는 `tasks/77-overlay-menu-ui.md`다. overlay의 세 저장 의미를 보존하는 독립 선택 UI와
영문·한글 번역만 구현한다. Display/App 저장·즉시 preview와 실제 PNG는 후속으로 분리한다.
Task77은 작업자 호출 1/2로 최종 통과했다. 설명 오류 두 곳은 Codex가 동작 변경 없이 정정했다.
다음 태스크는 `tasks/78-overlay-menu-app-wiring.md`다. Display Overlay 진입, 세 값의 저장 우선과 runtime
source 즉시 반영, write 실패 rollback을 연결한다. 실제 내장 PNG는 후속으로 남긴다.
Task78은 누적 1/2로 최종 통과했다. 관련 suites 552 passed / 0 failed / 0 ignored, core skip 0이며
device check와 clippy 종료 0이다. Display Overlay 진입·저장·runtime 즉시 반영까지 완성됐다. 다음은
production 내장 overlay asset 추가다.
다음 태스크는 `tasks/79-gb-cubexx-built-in-overlay.md`다. GB×720×720 sample 하나를 deterministic하게
생성·등록하고 default-off, card 우선, corrupt fallback과 배포 빌드를 검증한다.
Task79는 누적 1/2로 최종 통과했다. overlay 집중 검증 60 passed, `slot2` 전체 312 passed / 0 failed /
0 ignored, Python skip 0이며 device check·clippy·배포 빌드도 통과했다. GB×720×720 Integer sample의
완전 투명 aperture, production 단일 등록, 기본 off와 카드 우선/fallback을 확인했고 Codex 시각 검토도
마쳤다. 최종 판정은 `tasks/79-gb-cubexx-built-in-overlay.result.md`다. 실기 화질·정렬은 사용자 확인
항목이며 다음 작업 지시서는 아직 작성하지 않았다.
다음 태스크는 `tasks/80-global-timezone-settings-store.md`다. `System/slot2.ini`의
`utc_offset_minutes`를 기본 0·범위 -720..=840으로 독립 read/safe-write하며 volume과 unknown key를
양방향 보존한다. App의 clock 적용과 설정 UI는 후속 태스크다.
Task80은 누적 1/2로 최종 통과했다. 집중 테스트 23 passed, `slot2-store` 전체 101 passed / 0 failed /
0 ignored이고 fmt/clippy 종료 0이다. 시간대와 volume의 독립 safe-write, invalid 원본 보존과 D-25
경계를 확인했다. 최종 판정은 `tasks/80-global-timezone-settings-store.result.md`다. 다음은 App 부팅 시
card offset 적용과 store/platform 범위 일치 검증이며 지시서는 아직 작성하지 않았다.
다음 태스크는 `tasks/81-timezone-startup-app-wiring.md`다. `App::with_card`가 카드 offset을 runtime
clock에 한 번 적용하고 store/platform 범위를 compile-time으로 봉인한다. process-global clock 테스트는
단일 순차 integration test로 격리하며 UI와 저장은 후속이다.
Task81은 누적 1/2로 최종 통과했다. `slot2` 전체 313 passed / 0 failed / 0 ignored, core skip 0이며
device check·fmt·clippy 종료 0이다. 카드 offset의 시작 적용, production 범위 봉인과 read-only 보존을
확인했다. 최종 판정은 `tasks/81-timezone-startup-app-wiring.result.md`다. 다음은 시간대 선택 UI와
runtime 즉시 적용·저장·rollback이며, App 전 boot 진단 로그의 초기값 표기도 후속 마무리 대상이다.
다음 태스크는 `tasks/82-timezone-menu-ui.md`다. UTC±HH:MM 표시, 15분/60분 clamp 조정과
original/selected를 보존하는 독립 `TimezoneMenu`를 추가한다. Shelf/App preview·적용·취소·저장 실패
rollback은 후속 태스크다.
Task82는 누적 1/2로 최종 통과했다. 집중 테스트 11 passed, `slot2-ui`+`slot2-i18n` 전체 260 passed /
0 failed / 0 ignored이고 fmt/clippy 종료 0이다. format, navigation, 세 geometry safe-area, 번역과 warm
redraw를 확인했다. 최종 판정은 `tasks/82-timezone-menu-ui.result.md`다. 다음은 Shelf/App 진입과 runtime
preview·apply/cancel·safe-write 실패 rollback이다.
다음 태스크는 `tasks/83-shelf-menu-ui.md`다. M4 최종 순서의 선반 설정 메뉴를 추가하고, 미구현 행은
비활성으로 표시하며 시간대만 활성화할 수 있는 가용성 계약을 둔다. 실제 App 진입과 시간대
preview·apply/cancel·저장 실패 rollback은 Task84로 분리한다.
Task83은 누적 1/2로 최종 통과했다. 집중 테스트 9 passed, `slot2-ui`+`slot2-i18n` 전체 270 passed /
0 failed / 0 ignored이고 fmt/clippy 종료 0이다. 최종 판정은 `tasks/83-shelf-menu-ui.result.md`다.
다음은 App에서 ShelfMenu와 TimezoneMenu를 연결하고 preview·apply/cancel·safe-write 실패 rollback을
구현하는 `tasks/84-shelf-timezone-app-wiring.md`다. List의 Menu 탭과 Menu hold를 분리하고,
process-global clock 검증은 한 integration test에서 순차 실행한다.
Task84는 누적 1/2로 최종 통과했다. 집중 테스트 1 passed, `slot2` 전체 314 passed / 0 failed /
0 ignored, i18n 33 passed이고 device check·fmt·clippy 종료 0이다. 최종 판정은
`tasks/84-shelf-timezone-app-wiring.result.md`다. 다음은 `tasks/85-about-sticker-ui.md`로 frontend
version·target·MIT·`System/licenses` 안내를 표시하는 독립 About UI를 추가한다. App 배선은
Task86으로 분리한다.
Task85는 누적 1/2로 최종 통과했다. 집중 테스트 8 passed, `slot2-ui`+`slot2-i18n` 전체 279 passed /
0 failed / 0 ignored이고 fmt/clippy 종료 0이다. 최종 판정은 `tasks/85-about-sticker-ui.result.md`다.
다음은 `tasks/86-about-sticker-app-wiring.md`로 Shelf의 TimeZone+About 두 행을 활성화하고
`Screen::About`에 frontend package version과 현재 profile target을 전달한다.
Task86은 누적 1/2로 최종 통과했다. About/Timezone 집중 테스트 각각 1 passed, `slot2` 전체
315 passed / 0 failed / 0 ignored이고 device check·fmt·clippy 종료 0이다. Codex는 현재 동작과
어긋난 주석 두 곳만 정정했다. 최종 판정은 `tasks/86-about-sticker-app-wiring.result.md`다.
다음은 `tasks/87-global-language-settings-store.md`로 `System/slot2.ini`의 독립 language key를
기본 `en`으로 안전하게 저장하고 volume·시간대·unknown key를 보존한다. startup/picker/runtime
적용은 후속으로 분리한다.
Task87은 누적 1/2로 최종 통과했다. 집중 테스트 11 passed, `slot2-store` 전체 112 passed / 0 failed /
0 ignored이고 host/device check·fmt·clippy 종료 0이다. 최종 판정은
`tasks/87-global-language-settings-store.result.md`다. 다음은 저장 언어와 `SLOT2_LANG`의 시작
우선순위 및 pack load 실패 fallback을 App/i18n 경계에 연결하는 태스크이며 지시서는 아직 작성하지
않았다.
다음 태스크는 `tasks/88-language-startup-app-wiring.md`다. 명시적인 `SLOT2_LANG` override, 카드 저장
언어, `en` 순으로 요청 언어를 정해 host/device `UiCtx` 시작에 적용한다. unknown·malformed pack은
원본을 보존한 채 내장 영어로 부팅하며 picker/runtime 교체·저장은 후속으로 분리한다.
Task88 1차 구현의 기능과 325개 테스트는 통과했지만 Codex 검토에서 `announce_language`의 effective
code만 raw 출력되는 계약 위반을 찾았다. 마지막 수정 지시서는
`tasks/88-language-startup-app-wiring-attempt2.md`이며 기존 worker-result를 누적 2/2로 갱신해야 한다.
아직 최종 통과 판정하지 않았다.
Task88은 누적 2/2로 최종 통과했다. 2차에서 requested/effective 로그를 모두 debug escaping하도록
수정했고, 집중 테스트 10 passed, `slot2` 전체 325 passed / 0 failed / 0 ignored, core skip 0이며
device check·fmt·clippy 종료 0이다. 최종 판정은 `tasks/88-language-startup-app-wiring.result.md`다.
다음은 Shelf Language picker UI이며 지시서는 아직 작성하지 않았다.
다음 태스크는 `tasks/89-language-picker-ui.md`다. load 성공 후보의 code와 self-name을 받는 독립
`LanguagePicker`를 구현하고 current/highlight, wrap, 긴 목록 window, 말줄임과 세 geometry safe-area를
검증한다. pack discovery, Shelf/App 배선, runtime `UiCtx` 교체와 저장은 후속이다.
Task89는 누적 1/2로 최종 통과했다. 집중 테스트 11 passed, `slot2-ui` 256 passed,
`slot2-i18n` 34 passed로 모두 실패 0이고 device check·fmt·clippy 종료 0이다. 최종 판정은
`tasks/89-language-picker-ui.result.md`다. 다음은 실제 pack discovery, Shelf/App 입력, runtime
`UiCtx` 교체와 저장 실패 rollback을 연결하는 작업이며 지시서는 아직 작성하지 않았다.
다음 태스크는 `tasks/90-language-picker-app-wiring.md`다. load 성공 pack discovery, Shelf Language
입력, one-shot request와 load→save→`UiCtx` swap을 host/device 공용 경계에 연결한다. load/save 실패는
기존 context·카드·current를 유지하며 toast하고, `lang-font` 적용은 후속으로 분리한다.
Task90 1차 호출은 구현과 622줄 집중 테스트 생성 뒤 대화형 요청이 869 messages / 약 2.32MB로 커져
`400001 read body failed`로 중단됐다. 기존 인스턴스를 완전히 종료하고 새 인스턴스에서
`tasks/90-language-picker-app-wiring-recovery.md`를 누적 2/2 마지막 호출로 수행해야 한다. 결과 보고서는
아직 없다.
Task90은 누적 2/2로 최종 통과했다. 2차에서 lint/format을 정리하고 pending 입력 검증을 보강했다.
집중 테스트 9 passed, `slot2` 전체 334 passed / 0 failed / 0 ignored, `slot2-i18n` 34 passed /
0 failed / 0 ignored이고 device check·fmt·clippy 종료 0이다. 최종 판정은
`tasks/90-language-picker-app-wiring.result.md`다. 다음은 언어팩 `lang-font` preferred font 적용이며
지시서는 아직 작성하지 않았다.
다음 태스크는 `tasks/91-language-pack-preferred-font.md`다. effective pack의 `lang-font`를 안전한
단일 파일명으로 제한하고 card-first preferred → embedded OpenSans → CJK lazy chain에 연결한다.
missing/unsafe/corrupt preference는 언어 전환을 막지 않고 fallback하며, 한국어 metadata의 `.ttf`를
실제 배포 `.otf` 이름으로 바로잡는다.
Task91 1차는 부분 성공(누적 1/2)이다. `slot2-ui` 전체에서 Noto line height로 Cheat 메뉴 upper bar
clearance 1건이 실패했고, corrupt preferred 첫 slot은 glyph가 fallback해도 line metrics 0으로 bitmap이
사라진다. 마지막 수정 지시서는 `tasks/91-language-pack-preferred-font-attempt2.md`이며 기존 보고서를
누적 2/2로 갱신해야 한다.
Task91은 누적 2/2로 최종 통과했다. failed first font 뒤의 line metrics/bitmap fallback과 Noto line
box에 맞춘 Cheat 메뉴 여백을 고쳤고, preferred 탐색·lazy/dedup/runtime 계약을 유지했다. 작업자 검증은
`slot2-text` 12, `slot2-ui` 263, `slot2-i18n` 34, language startup/picker 20 passed로 모두 실패 0이며
device check·fmt·clippy 종료 0이다. 최종 판정은 `tasks/91-language-pack-preferred-font.result.md`다.
다음은 Noto Sans KR subset 생성 스크립트와 host 검증이며 지시서는 아직 작성하지 않았다.
다음 태스크는 `tasks/92-translation-contract-and-guide.md`다. en/ko key·variable·BTN·JOSA 계약을
자동 검사하고 한국어 125개 문구 전수 감사와 `docs/TRANSLATING.md`를 완성한다. Noto subset은 host의
일반 PATH와 Codex 번들 Python 모두 fontTools/`pyftsubset`이 없어 실행 환경 준비 뒤로 미뤘다.
Task92는 누적 1/2로 최종 통과했다. en/ko 각 125개 key, 변수·BTN·JOSA·literal cap/function 계약을
test-only scanner로 봉인하고 한국어 전수 감사와 번역 가이드를 완성했다. 작업자 검증은
`pack_contract` 8, `slot2-i18n` 42, `slot2-ui` 263, `slot2` 335 passed로 모두 실패 0이며 device
check·fmt·clippy 종료 0이다. 최종 판정은 `tasks/92-translation-contract-and-guide.result.md`다.
Noto subset은 fontTools 실행 환경이 없어 계속 후속이다.
다음 태스크는 `tasks/93-cart-filename-and-ordering-contract.md`다. Unicode scalar 순서의 한·영·일 혼합
정렬과 portable 특수문자 stem의 exact 보존, 다섯 부속 경로, 숨김/지원 확장자, 동일 stem 복수 확장자
공유 계약을 테스트와 설계 문서로 봉인한다.
Task93은 누적 1/2로 최종 통과했다. production 변경 없이 신규 6개 계약 테스트를 추가했고 작업자
`slot2-store` 전체는 118 passed / 0 failed / 0 ignored, host/device check·fmt·clippy 종료 0이다.
최종 판정은 `tasks/93-cart-filename-and-ordering-contract.result.md`다. 다음 작업 지시서는 아직 없다.
다음 태스크는 `tasks/94-m5-host-contract-closure.md`다. 카드 `ja.ftl` 3문자열 Acceptance를 App 통합
테스트로 봉인하고, Task80~84 시간대 완료와 Task93 파일명 정책을 MILESTONES/DESIGN/TRANSLATING의
오래된 문구에 반영한다. Noto subset과 실기 Acceptance는 계속 남긴다.
Task94는 누적 1/2로 최종 통과했다. 신규 App test가 카드 `ja.ftl` 3 message의 표시·선택·저장·영어
fallback을 검증했고, Task80~84 시간대 항목과 문서를 현재 동작에 맞췄다. 관련 테스트 67개는 모두
실패·ignored 0이고 device check·fmt·clippy 종료 0이다. 최종 판정은
`tasks/94-m5-host-contract-closure.result.md`다. Noto subset·V-10·한국어 전 화면 실기는 남아 있다.
다음 태스크는 `tasks/95-nes-snes-cartridge-skins.md`다. NES·SNES 독립 카트 shell/detail SVG 네 개와
skin table·raster 계약을 추가하고 MD/SMS fallback은 유지한다. 플랫폼 port 구조와 효과음은 후속이다.
lid는 백라이트 경로 미확정으로 추측 구현하지 않는다.
커밋·푸시는 사용자가 요청할 때만 한다.
# Current Codex pointer (2026-10-04)

Task 111 passed attempt 1/1. The 71 staged whitespace findings are closed; 43 task-history files received
exact one-byte cleanup, and two upstream licences remain byte-identical under narrow path-specific
`-whitespace` attributes. `C:\SLOT2\tasks\111-staged-whitespace-gate-closure.manifest.txt` supersedes Task
110 and now matches all 454 live paths including the Codex verdict. Nothing is staged or committed. Renewed
explicit user authorization is required before staging, cached-diff checking, and committing all 454 paths.
Push, tag, publication, hosted acceptance, and hardware checks remain separate. Do not run old OpenCodex
automation or Task 29 scripts.
