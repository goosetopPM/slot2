# Task 77 작업자 보고서

## 결과

성공. 누적 호출 1/2 (이 보고서를 쓰는 시점까지 실행 호출 1회).

## 구현 내용

- 신규 `crates/slot2-ui/src/overlay_menu.rs`: public `overlay_menu` module + `OverlayMenu`.
  `ROWS: [Option<bool>; 3] = [None, Some(true), Some(false)]` 순서 그대로이고, layout 상수는
  `BOX_W = 360`, `BOX_H = 208`, `PAD = 16`, `ROW_H = 36`.
- `OverlayMenu::new(current)`는 `ROWS.iter().position(...)`으로 정확히 그 행을 선택하고,
  `selected()`는 highlight된 행의 `Option<bool>`을 그대로 돌려준다. `up`/`down`은 modulo로 양 끝에서
  wrap한다. enum·bool 문자열화·row index 의존은 없다.
- `key(choice)`는 `None => "display-platform-default"`, `Some(true) => "overlay-on"`,
  `Some(false) => "overlay-off"`를 세 값 exhaustive match로 돌려준다.
- `draw`는 기존 Overscan/Shader와 동일한 palette·text 크기·safe-area·face cache를 쓴다.
  `Canvas::clear`를 호출하지 않고, 순서는 전 패널 `DIM`(BLACK 0.6) → `BACKDROP` panel → title
  `overlay-title`(PX_BODY, INK_DIM) → 세 rows(PX_TITLE, INK, 선택 행만 INK alpha 0.15 highlight) →
  `hint-select`/`hint-back`(PX_HINT, INK_DIM) 한 줄이다.
- `crates/slot2-ui/src/lib.rs`에 module 선언과 re-export 두 줄만 추가.
- `assets/lang/en.ftl`: `overlay-title = Overlay`, `overlay-on = On`, `overlay-off = Off`.
  `assets/lang/ko.ftl`: `overlay-title = 오버레이`, `overlay-on = 켜기`, `overlay-off = 끄기`.
  기존 key 문구는 바꾸지 않았고 새 generic On/Off key도 만들지 않았다.
- 신규 `crates/slot2-ui/tests/overlay_menu.rs`에 집중 테스트 7개,
  `crates/slot2-i18n/tests/i18n.rs`에 신규 key 직접 정의 테스트 1개
  (`overlay_menu_messages_read_naturally_in_both_packs`)를 추가했다. 기존 menu 테스트는 수정하지 않았다.

## 검증에서 확인한 것

- 세 값 round-trip: `None`/`Some(true)`/`Some(false)` 각각 constructor가 자기 행을 열고 `selected()`가
  같은 값을 돌려준다. 세 값의 selected index가 서로 다르다(특히 `None`과 `Some(false)`가 별도 행).
- 고정 순서 이동과 wrap: `None → Some(true) → Some(false) → None`, 역방향도 동일하며 한 바퀴 후 제자리.
- draw: `Op::Clear` 없음. 첫 mark가 `(0,0,pw,ph)` 크기의 `DIM` rect이고 그 다음이 panel이다.
  highlight rect는 `bx + PAD`, `row_y`, `BOX_W - 2*PAD`, `ROW_H`, `INK.with_alpha(0.15)` 정확히 하나뿐이다.
- `rgsp`/`rg35xxsp`/`rgcubexx` × en/ko에서 box, title, 세 row label, highlight, hint가 모두 640×480
  safe area 안에 있다(전 패널 dim만 예외). 각 행은 자기 localized label 폭·y로 자기 row 안에 그려진다.
- warm 상태: ko로 첫 draw에서 upload가 발생하고, 이후 8회 redraw와 highlight 이동 뒤
  `UploadAlpha8`/`UploadRgba8`가 0개다.
- 문구: en `Overlay`/`On`/`Off`, ko `오버레이`/`켜기`/`끄기`가 fallback 없이 그대로 나온다.

## 완료 기준 명령

| 명령 | 종료 코드 | 마지막 결과 |
|---|---|---|
| `cargo fmt --all -- --check` | 0 | 출력 없음 |
| `cargo test -p slot2-ui --test overlay_menu` | 0 | `test result: ok. 7 passed; 0 failed; 0 ignored; ... finished in 19.37s` |
| `cargo test -p slot2-ui -p slot2-i18n` | 0 | 결과 줄 25개 모두 ok, 합계 248 passed / 0 failed / 0 ignored |
| `cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 28.31s` |

참고: `slot2-i18n`의 `tests/i18n.rs`는 30 → 31 tests가 됐고 `slot2-ui`에는 `overlay_menu` 7개가 늘었다.
기존 수치(239) 대비 감소 없음. workspace test, GL test, device 배포는 실행하지 않았다.

## 파일

생성:
- `crates/slot2-ui/src/overlay_menu.rs`
- `crates/slot2-ui/tests/overlay_menu.rs`

수정:
- `crates/slot2-ui/src/lib.rs` (`pub mod overlay_menu;` / `pub use overlay_menu::OverlayMenu;` 두 줄)
- `assets/lang/en.ftl`, `assets/lang/ko.ftl` (신규 세 key + 주석)
- `crates/slot2-i18n/tests/i18n.rs` (신규 테스트 1개만)

최종 검증 명령 실행 뒤 코드 변경 없음. 임시로 만든 `target/t77-ui-i18n.txt`는 삭제했다. 관련 없는
미커밋 변경은 건드리지 않았다.

## 범위 밖 확인

Display menu 진입 행, App의 settings 저장·rollback·toast·live preview, Session/runtime overlay layer,
store 변경, `BUILT_IN_OVERLAYS` entry와 실제 `assets/overlays/` PNG는 추가하지 않았다. 이 UI는 PNG
존재 여부도 플랫폼 조회도 하지 않는다.

## 계약 의문 / 남은 위험

- 남은 위험 1줄: `None`은 현재 기본값 off를 상속하지만 `Some(true)`는 유효한 카드/내장 asset이 있으면
  명시적으로 그린다. 후속 App 저장 경로는 이 차이를 보존하도록 `selected()`의 `Option<bool>`을 그대로
  써야 한다.

## Codex 검토 정정

- 2026-09-28: 구현·테스트를 바꾸지 않고 위 두 설명만 runtime 계약에 맞게 정정했다. 작업자 검증 뒤
  동작 변경은 없으며 `git diff --check`로 문서 수정의 whitespace 오류가 없음을 확인한다.

## 소요 시간

약 20분 (19:51 ~ 20:11 KST, 빌드·테스트 대기 포함).
