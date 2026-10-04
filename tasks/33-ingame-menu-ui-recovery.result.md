# Task 33 결과 — M4 인게임 메뉴 UI 완성

2026-09-25 18:32~18:39 (로컬, 약 7분). 판정: **성공.**

## 변경 파일

- `crates/slot2-ui/src/in_game_menu.rs` — 변경 없음. 워커가 남긴 부분 구현을 그대로 보존했다.
  선택 인덱스 private, 7개 항목 고정 순서, 전용 Fluent 키, 위/아래 wrap, 안전 영역 중앙
  박스, dim → 박스 → 제목 → 7행 → 강조 → 힌트 순서의 overlay 그리기가 계약과 일치했고
  클리어 호출도 없었다. 다시 쓰거나 대체하지 않고 필요한 나머지만 채웠다.
- `crates/slot2-ui/src/lib.rs` — `pub mod in_game_menu;` 위치를 `image` 뒤로 옮겼다.
  이 파일의 rustfmt 모듈 정렬 위반이 `cargo fmt --check` 실패의 유일한 원인이었다.
- `assets/lang/en.ftl` — 8개 키 추가(빈 줄 1개 포함 9줄).
- `assets/lang/ko.ftl` — 같은 8개 키의 한국어 문구 추가. UTF-8 그대로 유지.
- `crates/slot2-ui/tests/in_game_menu.rs` — 신규 전용 테스트 4개.
- `tasks/33-ingame-menu-ui-recovery.result.md` — 이 보고서.

추가 키: `ingame-menu-title`, `ingame-continue`, `ingame-save-state`, `ingame-cheats`,
`ingame-display`, `ingame-core`, `ingame-device`, `ingame-eject`. 힌트는 기존
`hint-select`/`hint-back`를 재사용했다.

`git status --short`에서 위 허용 목록 밖의 소스 변경은 없다. 기존 사용자 미커밋 변경은
건드리지 않았고 커밋·푸시도 하지 않았다.

## 검증

명세 순서대로 실행했다.

1. `cargo fmt --all -- --check` — 처음 종료 1, `crates/slot2-ui/src/lib.rs:15` 모듈 순서 1건.
   해당 줄만 고친 뒤 재실행 종료 **0**, diff 없음.
2. `cargo test -p slot2-ui -p slot2-i18n` — 종료 **0**, 실패 0. 신규 파일
   `tests/in_game_menu.rs` 포함 4개 테스트 통과(`rows_keep_their_order_and_dedicated_keys`,
   `default_is_continue_and_navigation_wraps`,
   `every_dedicated_key_has_english_and_korean_text`,
   `overlays_the_frame_and_keeps_every_row_inside_the_safe_area`). 신규 테스트만 따로 재실행해도
   4 passed / 0 failed.
3. `cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings` — 종료 **0**, 경고 0.

세 명령은 위 순서로 돌렸고, 그 사이 코드 수정은 없었다. 두 번째 명령 뒤에 신규 테스트
바이너리를 한 번 더 돌렸을 뿐이며 파일은 바뀌지 않았다. 최종 검증 이후 코드 변경 없음.

테스트는 3개 기기(`rgsp`, `rg35xxsp`, `rgcubexx`) × 2개 언어(en/ko)로 확인한다. dim이
패널 전체를 덮는지, `Op::Clear`가 하나도 없는지, 선택 행에만 강조 사각형이 있고 나머지
행에는 없는지, 7개 행 사각형과 모든 텍스트 이미지가 안전 영역 안인지, 각 행 라벨이 자기
행 안에 중앙 정렬되는지를 검사한다. 그리기 명령 총 개수에는 결합하지 않았다.

## 남은 항목과 계약 의견

- 이 태스크 범위대로 `crates/slot2/src/app.rs`와 `Screen`에 배선하지 않았으므로, 이 메뉴는
  아직 실행 바이너리에서 호출되지 않는다. 상태 전환·입력 배선·세션 정지/재개는 다음 태스크의
  실제 작업량이다.
- 워커가 `selected_index()` 공개 접근자를 추가했다(`PowerMenu`는 `selected`를 pub 필드로 둔다).
  인덱스 불변식을 깨지 않고 테스트가 확인할 수 있는 형태라 유지했다. 계약 위반은 아니지만
  두 메뉴의 공개 방식이 다르다는 점은 기록해 둔다.
- `BOX_H = 344`는 제목·7행·힌트가 640x480 안전 영역 안에 들어가도록 잡힌 값이며, 테스트가
  대상 기기 3종에서 이를 강제한다. 화면이 골든 파일 검증은 범위 밖이라 하지 않았다.
- 실행 시간 약 7분. 모델 토큰·비용 정보는 이 대화형 도구가 노출하지 않아 `unavailable`.
