# Task 61 — CorePicker App 배선과 실패 안전 코어 전환

현재 checkout에서 직접 작업한다. Task60의 `CorePicker`를 인게임 메뉴의 Core 행에 연결하고, 선택한
공식 core로 Session을 다시 시작한다. 전환 도중 checkpoint, target core 시작, 설정 저장 중 어느 단계가
실패해도 가능한 한 기존 core와 플레이 위치를 복구한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\61-core-picker-app-wiring.md`
- `C:\SLOT2\tasks\59-core-scoped-state-routing.result.md`
- `C:\SLOT2\tasks\60-core-picker-ui.result.md`
- `C:\SLOT2\tasks\46-display-scale-app-wiring.result.md`의 App submenu 배선 방식만
- `C:\SLOT2\tasks\47-safe-game-settings-write.result.md`의 settings 쓰기 실패 계약만
- `C:\SLOT2\crates\slot2\src\app.rs`의 `Screen`, `App`, Session 시작/중단, 인게임 submenu,
  draw/audio pause 부분만
- `C:\SLOT2\crates\slot2\src\session.rs`의 `CoreChoice`, `resolve_core`, `Session::start`,
  core/state/save accessor와 stop 부분만
- `C:\SLOT2\crates\slot2-ui\src\core_picker.rs`
- `C:\SLOT2\crates\slot2-ui\src\in_game_menu.rs`의 Core 행과 choice만
- `C:\SLOT2\crates\slot2-retro\src\registry.rs`의 `CoreId`, `PlatformDef`,
  `supported_cores`만
- `C:\SLOT2\crates\slot2-store\src\settings.rs`의 `GameSettings`와 read/write 계약만
- `C:\SLOT2\crates\slot2\src\host_app.rs`, `device_app.rs`의 `SinkRequest` 처리 부분만
- 관련 App/Session 통합 테스트 helper와 영·한 FTL의 메뉴/toast 부분만

직접 관련된 helper와 타입만 추가로 읽는다. 워커 로그, 저장소 이력, 다른 마일스톤 파일은 읽지 않는다.

## 기존 계약

- Core 행은 인게임 메뉴에 이미 있다. `CorePicker`는 공식 registry 순서와 설치 목록의 교집합을
  표시하고 current badge와 highlight를 분리한다.
- 복수 후보 플랫폼은 GB/GBC의 `[mGBA, Gambatte]`, GBA의 `[mGBA, gpSP]`다. 다른 플랫폼은
  registry 후보가 하나뿐이므로 CorePicker를 열지 않는다.
- `GameSettings.core == None`은 플랫폼 기본 core다. 기본 core를 선택할 때 문자열 override를 남기지
  않는다. 대체 core는 canonical `CoreId::base_name()`을 저장한다.
- save RAM은 게임별 공유이고, Resume/numbered state는 실제 core namespace별로 격리돼 있다.
- libretro state는 core 사이에 호환된다고 가정하지 않는다. target core는 자기 Resume만 읽는다.
- 실행 중인 core가 외부 core이면 `Session::core_id()`는 `None`이다. 공식 picker 행에 current badge를
  거짓 표시하지 않는다.
- `Card::write_settings`는 기존/미지 key를 보존하고, 읽을 수 없는 ini를 빈 설정으로 덮지 않는다.
- `SinkRequest`는 한 칸이다. `Open`과 새 consumer를 내면 host/device loop가 기존 sink를 새 sink로
  교체한다. 같은 action에서 `Close` 뒤 `Open`을 큐처럼 쌓으려 하지 않는다.

## 핵심 안전 계약

선택값을 먼저 settings에 쓴 뒤 target 시작을 시도하는 방식은 금지한다. target 시작이 실패하고 settings
rollback까지 실패하면 기존 core도 다시 찾지 못할 수 있다. 다음 의미의 순서를 지킨다.

1. 선택한 core가 current와 같으면 settings, state, Session, sink를 건드리지 않고 기존 인게임 메뉴로
   돌아간다.
2. 현재 Session을 **살려 둔 채** save RAM flush와 현재 namespace의 Resume 저장을 모두 시도한다.
   어느 하나라도 실패하면 현재 Session을 그대로 유지하고 picker에 머문다.
3. checkpoint가 성공한 뒤에만 현재 Session을 내려놓는다. 이때 settings는 아직 이전 값 그대로다.
4. settings를 임시로 바꾸지 않고도 지정한 공식 `CoreId`로 Session을 시작할 수 있는 좁은 내부 경로를
   만든다. target 시작이 실패하면 변경되지 않은 settings의 일반 `Session::start`로 이전 core를
   다시 시작하고, 방금 저장한 이전 core Resume을 불러온다.
5. target Session이 실제로 시작된 뒤에만 새 `GameSettings.core`를 안전하게 쓴다. 쓰기가 실패하면
   target을 버리고, 여전히 변경되지 않은 settings를 이용해 이전 core와 Resume을 복구한다.
6. target 시작과 settings 저장이 모두 성공하면 target namespace의 Resume이 있으면 그것만 불러오고
   새 Session과 consumer를 설치한다. Resume이 없으면 fresh target으로 계속한다. target Resume이
   깨졌으면 기존 `resume-load-failed` 의미대로 fresh target을 유지하고 사실을 알린다.

이 순서와 같은 안전성을 유지한다면 helper의 정확한 이름과 내부 분리는 자유다. 두 core를 동시에
살려 두거나 hot-swap하지 않는다.

## 구현 계약

### 1. 화면과 CorePicker 수명

- `CorePicker`는 후보 `Vec`을 소유해 `Copy`가 아니다. 기존 `Screen: Copy` 계약을 깨거나 picker를
  Screen variant 안에 넣지 않는다. `StateSwitcher`처럼 App이 picker 한 개를 소유하고,
  `Screen::Core(InGameMenu)`처럼 돌아갈 부모 메뉴만 화면 상태에 둔다.
- App 생성 시 안전한 빈 picker를 만들고 Core 행을 열 때 현재 cart platform, 설치된 공식 core,
  `Session::core_id()`로 refresh/rebuild한다. 파일 검색은 그 순간 한 번만 하며 draw에서 filesystem을
  읽지 않는다.
- 설치 여부는 `core_dir/CoreId::file_name()`이 실제 파일인지로 판정한다. registry 순서나 플랫폼
  지원표를 App에서 다시 적지 않는다.
- `supported_cores(platform).len() < 2`인 플랫폼에서는 picker를 열지 않고 localized
  `core-no-alternatives` toast를 보이며 인게임 메뉴의 Core 행에 머문다.
- 복수 후보 플랫폼은 설치된 공식 core가 0개 또는 1개여도 picker를 연다. Task60의 empty/single UI가
  실제 상황을 정직하게 보여준다.

### 2. 입력·draw·pause

- Core 화면에서 Up/Down은 picker 탐색, B/Menu는 열 때의 같은 Core 행으로 복귀한다.
- A는 highlighted가 없으면 `core-picker-empty` toast를 보이고 화면을 유지한다.
- A가 current 공식 core와 같으면 write/checkpoint/restart/sink 요청 없이 같은 인게임 Core 행으로
  돌아간다.
- 다른 core의 성공 전환은 picker와 인게임 메뉴를 닫고 `Playing`으로 돌아간다.
- Core 화면은 게임의 마지막 frame 위에 `CorePicker::draw`로 그린다. shelf wallpaper나 인게임 메뉴를
  아래에 다시 그리지 않고 canvas를 clear하지 않는다.
- Core 화면 동안 게임 frame과 audio는 멈춘다. App의 `audio_paused`와 frame 진행 조건에 포함한다.
- 성공 또는 복구 Session의 consumer 하나만 `pending_consumer`에 두고 `SinkRequest::Open` 하나를 낸다.
  같은 core 선택, empty, 취소, checkpoint 실패는 sink를 건드리지 않는다.

### 3. 지정 core 시작 경로

- `Session::start`의 core load, BIOS/options, save RAM, cheats, rewind/display/audio 초기화 코드를
  복제하지 않는다. 일반 resolver 경로와 지정 공식 core 경로가 같은 내부 생성 경로를 공유한다.
- 지정 core는 현재 cart platform의 `supported_cores`에 속해야 하고, library path는 반드시
  `core_dir.join(core.file_name())`으로 만든다. 임의 경로나 외부 문자열을 받지 않는다.
- 없는 파일, 지원하지 않는 조합, load 실패는 오류다. 다른 core로 조용히 fallback하지 않는다.
- core options, cheat delivery, state namespace는 **실제로 지정해 연 core**를 기준으로 한다.
  save RAM의 기존 공유 정책은 바꾸지 않는다.

### 4. checkpoint와 전환 성공

- 기존 Session이 살아 있는 동안 `flush_save`와 `save_state(..., StateKind::Resume)`를 호출한다.
  둘 다 성공해야 Session을 버릴 수 있다. 기존 `stop`의 일반 종료 정책은 회귀시키지 않는다.
- checkpoint 실패 시 `core-checkpoint-failed` toast를 보이고 picker에 머문다. Session, screen의 부모
  메뉴, current setting과 sink는 그대로다.
- target Session 시작 뒤 target namespace의 Resume만 시도한다. 다른 core의 Resume을 probe하거나
  삭제하지 않는다.
- 새 설정은 기존 `GameSettings` 전체를 읽어 `core` 필드만 바꿔 `write_settings`에 넘긴다.
  선택값이 플랫폼 default면 `None`, 아니면 canonical base name이다. scale/overscan/rewind와 알 수 없는
  ini key를 보존한다.
- 성공하면 새 Session을 설치하고 undo 및 fast-forward latch처럼 이전 Session에 속한 일시 상태를
  지운다. 새 Session은 기본 속도로 시작한다.

### 5. 실패 복구

- target start 실패: settings는 아직 이전 값이어야 한다. 일반 `Session::start`로 이전 선택을 다시
  열고 이전 namespace의 Resume을 불러온다. 성공하면 picker에 머물고 `core-switch-failed` toast를
  보인다.
- settings 저장 실패: 시작했던 target Session을 버리고 같은 방식으로 이전 core를 복구한다. 성공하면
  picker에 머물고 `core-setting-save-failed` toast를 보인다.
- 이전 core가 외부 library이거나 잘못된 설정의 fallback이었다 해도, 변경되지 않은 settings와 기존
  resolver를 사용하므로 원래 실행 결과로 돌아가야 한다. 이전 core를 official ID로 추측하지 않는다.
- 이전 core 재시작은 됐지만 Resume load가 실패하면 fresh 이전 Session이라도 플레이 가능 상태로
  보존하고 `core-recovery-state-failed`를 보인다. 이 더 심각한 사실을 일반 switch/save 실패 toast로
  덮지 않는다.
- 이전 core 재시작 자체가 실패하면 Session이 없는 상태를 Playing/Core overlay로 남기지 않는다.
  sink를 Close하고 기존 Ejecting→shelf 경로로 빠지며 `core-recovery-failed`를 보인다.
- 실패를 로그만 남기고 성공처럼 settings/current badge를 바꾸지 않는다.

### 6. localized message

영문·한글 pack에 최소한 아래 의미를 직접 정의하고 i18n 직접 정의 테스트에 포함한다.

- `core-no-alternatives`: 이 플랫폼은 선택할 다른 코어가 없음
- `core-checkpoint-failed`: 게임 상태를 안전하게 저장하지 못해 코어를 바꾸지 않음
- `core-switch-failed`: 선택한 코어를 시작하지 못해 이전 코어로 복구함
- `core-setting-save-failed`: 코어 선택을 저장하지 못해 이전 코어로 복구함
- `core-recovery-state-failed`: 이전 코어는 복구했지만 플레이 위치를 이어하지 못함
- `core-recovery-failed`: 코어 전환 실패 뒤 게임을 복구하지 못함

문구는 짧고 플레이어가 다음 상태를 이해할 수 있게 쓴다. 내부 경로·오류 문자열은 화면에 노출하지
않고 로그에 남긴다. 기존 `core-picker-empty`와 `resume-load-failed`는 의미가 맞는 곳에서 재사용한다.

## 테스트 계약

`core_picker_app.rs` 통합 테스트와 필요한 좁은 Session/App 단위 테스트를 추가해 최소한 다음을 직접
검증한다. 기존 MIT GBA ROM과 `vendor/mgba_libretro.*`, `vendor/gpsp_libretro.*`를 재사용하며, 이
환경에 두 core가 있으므로 성공/복구 핵심 테스트를 skip하지 않는다.

- GBA Core 행은 설치된 mGBA/gpSP를 registry 순서로 열고 실제 current를 표시한다. NES 등 단일 후보
  플랫폼은 picker를 열지 않고 toast를 보인다.
- Core overlay는 audio/frame을 pause하고 마지막 game frame 위에 그리며, B/Menu가 같은 Core 행으로
  돌아가고 sink를 건드리지 않는다.
- empty A는 no-op+toast, current A는 settings/state/Session/sink를 건드리지 않는 종료다.
- mGBA→gpSP 성공: mGBA save RAM과 Resume이 먼저 기록되고, 설정은
  `gpsp_libretro`, Session/current/namespace는 gpSP, sink 요청은 새 consumer의 Open 하나이며 화면은
  Playing이다. mGBA numbered/Resume은 보존된다.
- gpSP→플랫폼 기본 mGBA 성공: 설정의 core key는 `None` 의미로 제거되고 다른 알려진 필드와 미지 ini
  key는 보존된다. target mGBA Resume이 있으면 그것만 불러온다.
- checkpoint 쓰기를 의도적으로 실패시키면 old Session과 설정이 그대로이고 picker에 머물며 sink
  요청이 없다.
- 설치된 이름이지만 load할 수 없는 target library로 target-start 실패를 만들면 설정은 바뀌지 않고
  old core와 방금 쓴 old Resume이 복구된다. 화면은 picker, sink는 복구 consumer의 Open 하나다.
- settings path를 directory로 만드는 등 결정적 I/O 실패로 setting-write 실패를 만들면 target을
  성공처럼 유지하지 않고 old core/Resume을 복구한다. settings가 target 값으로 반쯤 바뀌지 않는다.
- target core의 Resume 없음은 fresh target 성공, 깨진 target Resume은 fresh target+
  `resume-load-failed`이며 다른 core의 Resume은 읽거나 삭제하지 않는다.
- failure recovery 중 Resume load만 실패한 경우와 old Session start 자체가 실패한 경우의 화면,
  toast, sink, session 없음/있음 상태를 각각 검증한다.
- 영문과 한글 pack이 모든 새 key를 직접 정의한다.

검증은 observable state, settings/state 파일, `Session::core_id/state_namespace/core_state`, screen,
toast, sink request/consumer를 사용한다. sleep, 실제 오디오 장치, texture id, 상용 ROM에 의존하지 않는다.
테스트를 위해 제품 실패 처리를 약화하거나 특정 테스트 경로 상수를 넣지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2\src\app.rs`
- `C:\SLOT2\crates\slot2\src\session.rs`
- `C:\SLOT2\crates\slot2\tests\core_picker_app.rs` (신규)
- 직접 필요한 경우 기존 `C:\SLOT2\crates\slot2\tests\session.rs`
- `C:\SLOT2\assets\lang\en.ftl`
- `C:\SLOT2\assets\lang\ko.ftl`
- `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`
- `C:\SLOT2\tasks\61-core-picker-app-wiring.worker-result.md`

그 밖의 파일은 수정하지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- CorePicker UI layout/model, registry 후보/순서, core build/pin 변경
- state directory 형식, legacy migration, save RAM 공유 정책 변경
- hot-swap, 동시에 두 libretro core 유지, background 전환
- 다른 인게임 submenu 또는 shelf launch UX 변경
- 전체 workspace 테스트, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2 -p slot2-i18n
cargo clippy -p slot2 -p slot2-i18n --all-targets -- -D warnings
```

모두 종료 0이어야 한다. `cargo test -p slot2`의 mGBA/gpSP core-switch 핵심 테스트가 skip 없이
실행됐는지 별도로 확인한다. 검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다. workspace test나
device 배포는 실행하지 않는다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\61-core-picker-app-wiring.worker-result.md`를 작성한다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- picker open/input/draw/pause와 설치 후보 계산 결과
- checkpoint → explicit target start → settings commit 순서 및 default `None` 저장 결과
- target start/settings/checkpoint 실패별 old core·Resume·sink 복구 증거
- mGBA↔gpSP 실코어 테스트의 skip 여부와 핵심 state/settings 결과
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
