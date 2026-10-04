# Task 60 — worker result (플랫폼별 CorePicker UI)

## 결과
성공. 누적 호출 **1/2**(이번 호출에서 완료, 중단·실패 호출 없음).

## 후보 filtering·order 계약
- `CorePicker::new(platform, installed: &[CoreId], current: Option<CoreId>)`. UI는 `slot2_retro`의
  `CoreId`·`Platform`을 직접 쓰고 자체 enum·문자열 ID를 만들지 않는다.
- 후보 = `supported_cores(platform)` 순서를 그대로 둔 채 `installed.contains(core)`로만 거른다.
  입력의 순서·중복·타 플랫폼 core는 순서나 후보를 바꾸지 못한다(중복은 `contains`로 자연히 1개,
  지원하지 않는 core는 registry 목록에 없어 탈락). GB/GBC `[mGBA, Gambatte]`, GBA `[mGBA, gpSP]`,
  NES `[FCEUmm]`, SNES `[Snes9x]`, MD/SMS `[Genesis Plus GX]` — 테스트가 7개 플랫폼 전부에서
  registry와 1:1 비교한다.
- 공개 query: `rows() -> &[CoreId]`(읽기 전용), `len()`, `is_empty()`, `highlighted() -> Option<CoreId>`,
  `current() -> Option<CoreId>`. 후보 벡터를 mutable로 노출하지 않고, private `selected`로 불변식을
  유지한다. `key(core)`는 `CoreId` 전체에 대해 exhaustive match라 core가 늘면 빌드가 깨진다.

## highlighted/current/empty/navigation 계약
- 현재 공식 core가 후보에 있으면 그 행이 최초 highlight이고 `current()`도 같은 값이다. 탐색해도
  `current()`는 고정(그리기에서도 badge가 2회 이동 내내 같은 행에 남는 것을 프레임으로 확인).
- `current`가 `None`, 다른 플랫폼 core, 또는 설치되지 않은 core면 highlight는 첫 행이고 `current()`는
  `None`이며 어느 행에도 badge가 그려지지 않는다(3 경우 × 3 profile × 2 언어에서 확인).
- 후보 0개면 selection 개념이 없고 `highlighted()`/`current()`가 `None`, 행 highlight 0개, 첫 행 자리에
  localized empty 문장을 그린다. 1개면 Up/Down 모두 no-op으로 그 행 유지. 2개면 양끝에서 wrap하고
  한 바퀴가 제자리로 돌아온다. 빈 목록에서 Up/Down은 panic 없이 no-op.
- 입력 이벤트·A/B·sound·timer는 넣지 않았다(호출은 Task61).

## 최종 layout·localized message·warm draw
- layout: `BOX_W 360`, `BOX_H 190`, `PAD 16`, `ROW_H 36`, `MAX_ROWS 2`, `NOTICE_GAP 10`,
  `BADGE_INSET 6`. 행은 `by + PAD + 28 + i*ROW_H`, 안내는 `row_y(MAX_ROWS) + NOTICE_GAP`, hints는
  DisplayMenu와 같은 `by + BOX_H - PAD - PX_HINT` 한 줄. `MAX_ROWS`는 registry의 최장 목록과 같음을
  테스트가 단언하고, 안내·hints가 겹치지 않으며 각각 `BOX_W - 2*PAD` 안에 들어감을 폭 측정으로
  확인했다(한국어 안내가 가장 넓은 항목).
- 그리기: 전체 물리 패널 dim(DIM, `Canvas::clear` 없음) → safe area 중앙 panel(BACKDROP) → 제목
  `ingame-core`(PX_BODY, INK_DIM) → 후보 행(highlight 정확히 1개, INK alpha 0.15, 라벨 PX_TITLE) →
  실행 core 행 오른쪽 끝 `core-current` badge(INK_DIM, 행 rect 안) → 재시작 안내(PX_HINT, INK_DIM) →
  `hint-select`+`hint-back`. 3 profile × en/ko에서 첫 op이 full-panel dim이고, dim을 뺀 모든 op이
  640×480 safe area 안에 있으며, title/각 행/badge/안내/hints가 각자 band에 실제로 그려짐을 확인.
- localized: `core-name-mgba = mGBA`, `core-name-gambatte = Gambatte`, `core-name-gpsp = gpSP`,
  `core-name-fceumm = FCEUmm`, `core-name-snes9x = Snes9x`,
  `core-name-genesis-plus-gx = Genesis Plus GX` — 두 pack 표기가 동일한 고유명사이고 `libretro`
  문자열은 pack에도 화면에도 없다(base filename 폭이 프레임에 등장하지 않음을 단언). `core-current`
  = `Current`/`현재`, `core-picker-empty` = `No cores available`/`사용 가능한 코어가 없습니다`,
  `core-picker-restart` = `Changing core restarts the game`/`코어를 바꾸면 게임을 다시 시작합니다`.
- warm draw: 한국어 ctx에서 첫 프레임만 upload가 있고, 같은 상태 8회 반복과 highlight 이동 4회
  (down/up ×2)에서 새 `UploadAlpha8`/`UploadRgba8`가 0. empty picker도 첫 프레임 뒤 4회 반복에서 0.

## 완료 기준 명령 (원문 순서, 마지막 코드 변경 뒤, 이후 코드 변경 없음)
1. `cargo fmt --all -- --check` → exit 0, 출력 없음.
2. `cargo test -p slot2-ui -p slot2-i18n` → exit 0. `test result:` 줄 21개, 합계 **211 passed /
   0 failed / 0 ignored**. 마지막 줄: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured;
   0 filtered out; finished in 0.00s`(slot2_ui doc-tests). 새 suite `core_picker` 9 passed,
   `i18n` 26 passed(신규 core picker message 테스트 포함).
3. `cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings` → exit 0,
   `Finished \`dev\` profile … in 8.58s`.

## 생성·수정 파일
- 생성: `crates/slot2-ui/src/core_picker.rs`, `crates/slot2-ui/tests/core_picker.rs`,
  `tasks/60-core-picker-ui.worker-result.md`.
- 수정: `crates/slot2-ui/Cargo.toml`(`slot2-retro = { workspace = true }`, 주석 포함),
  `crates/slot2-ui/src/lib.rs`(module + re-export 2줄), `assets/lang/en.ftl`·`ko.ftl`(core picker
  block), `crates/slot2-i18n/tests/i18n.rs`(신규 테스트 1개), `Cargo.lock`(+1줄, slot2-ui 의존 목록).
- 기존 DisplayMenu/InGameMenu 테스트·소스는 건드리지 않았고, 검증 뒤 코드 변경 없음.
  커밋·푸시·네트워크·실기·공용 설정 변경 없음.

## 계약 의견 / 남은 위험
- `MAX_ROWS`가 registry 최장 목록에 고정돼 있어, 나중에 한 플랫폼에 core가 3개가 되면 layout 테스트가
  먼저 깨진다(의도한 tripwire지만 그때 BOX_H를 함께 키워야 한다).
- 외부 core(`core_id() == None`)와 세션 없음이 화면상 같은 "badge 없음"으로 보인다. 계약대로이지만
  Task61이 두 경우를 구분해 표시하려면 UI에 다른 입력이 필요하다.

## 소요 시간
약 16분(14:06–14:22 KST). 누적 호출 1/2.
