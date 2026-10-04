# Task 45 결과 — 게임별 화면 배율 메뉴 UI

판정: **성공.** 누적 호출 1/2. 약 28분(17:12~17:40).

## 변경 파일

- `crates/slot2-ui/src/display_menu.rs` (신규) — `DisplayMenu`와 `ROWS`. 순서는 계약대로
  `[None, Some(Integer), Some(AspectFit), Some(Fill)]`이며 `slot2_store::ScaleMode`를 그대로 쓰고
  새 열거형을 만들지 않았다. `new(Option<ScaleMode>)`는 그 값의 행을 초기 선택하고, 공개 표면은
  읽기 전용 `selected() -> Option<ScaleMode>`와 `up()/down()`뿐이다(둘 다 양끝 wrap).
  그리기는 in-game/power 메뉴와 같은 언어: 패널 전체 dim(BLACK 0.6) → 안전 영역 중앙 박스
  (BACKDROP, 340x244) → 제목(PX_BODY, INK_DIM) → 4행(PX_TITLE, INK, 선택 행만 INK 0.15 강조) →
  기존 `hint-select`/`hint-back`. `clear` 없음. 모듈 주석은 배율 전용 서브메뉴임을 밝히고 셰이더·
  오버레이 컨트롤이 있다고 주장하지 않는다.
- `crates/slot2-ui/src/lib.rs` — `pub mod display_menu;`와 `pub use display_menu::DisplayMenu;`.
- `crates/slot2-ui/tests/display_menu.rs` (신규) — 계약 테스트 6개.
- `assets/lang/en.ftl`, `ko.ftl` — `display-platform-default`(Platform default/플랫폼 기본값),
  `display-integer`(Integer scale/정수 배율), `display-aspect-fit`(Fit aspect ratio/화면 비율 맞춤),
  `display-fill`(Fill screen/화면 채우기). 제목은 이 메뉴를 여는 in-game 행과 같은 단어라
  `ingame-display`("Display"/"화면")를 재사용했다.
- `crates/slot2-i18n/tests/i18n.rs` — 위 4키와 재사용 제목의 en/ko 직접 정의 검증 1개.
- `tasks/45-display-scale-menu-ui.worker-result.md` — 이 보고서.

App·Session·store·gfx·런타임 루프·문서는 손대지 않았다. 설정 writer나 파일 접근도 없다.

## 검증 (마지막 코드 변경 뒤, 명세 순서)

1. `cargo fmt --all -- --check` — 종료 **0**.
2. `cargo test -p slot2-ui -p slot2-i18n` — 종료 **0**, 실패 0. `tests/display_menu.rs` 6개(네 생성
   값이 각자 행을 선택, 이동 순서·양끝 wrap·한 바퀴, dim이 첫 마크이고 강조가 정확히 한 행이며
   3기기×2언어에서 dim 외 모든 마크가 안전 영역 안, 행별 라벨이 자기 키의 폭·자기 행에, warm
   재도색 업로드 0(한국어 CJK 포함, 강조 이동 포함), 행 정의 순서·키 유일성), i18n 23개(신규 1),
   기존 slot2-ui 스위트 전부 통과.
3. `cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings` — 종료 **0**.

최종 검증 이후 코드 변경 없음. 커밋·푸시·실기·Pi·dist 빌드 없음.

## 남은 항목과 계약 의견

- App 화면 변형·Display 행 배선·세션 배율 적용·설정 저장은 계약대로 Task 46 몫이라 손대지 않았다.
  `DisplayMenu`는 `Option<ScaleMode>`만 알고 App/입력 타입에 의존하지 않으며 Copy라 Screen에 넣을 수 있다.
- 계약 의견: 제목 키는 새로 만들지 않고 `ingame-display`를 재사용했다(같은 단어 "Display"/"화면"을
  두 키로 중복 정의하지 않기 위해서다). 새 키는 행 4개뿐이다.
