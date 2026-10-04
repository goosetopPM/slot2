# Task 51 워커 결과 (누적 2/2 최종) — 인게임 치트 메뉴 UI

**최종: 성공. 누적 호출 2/2.** 지정 검증 4종 모두 종료 0. 1회차에서 승인된 동작은 그대로이고,
이번 회차는 지적된 두 결함(말줄임 비용, 위쪽 표시 간격)만 고쳤다.

## 1회차 승인 동작 유지

공개 API(`CheatMenu::new/selected_index/up/down/draw`, `fit`), 선택·스크롤 불변식, 빈 상태·행·
상태·힌트·safe-area·warm redraw 표현, 영한 번역 키 4개는 변경 없이 1회차 테스트 13개가 그대로
통과한다(현재 16개 = 기존 13 + 신규 3).

## 결함 1 — `fit` 탐색 비용 (수정)

- 방식: UTF-8 문자 경계 목록(`char_indices` + 끝)을 한 번 만들고, 전체 문자열 1회 측정으로
  "그대로 들어감"을 먼저 판정한 뒤, `prefix + …`가 `max_w`에 들어가는 **가장 긴 경계**를 이분
  탐색한다(폭은 prefix가 길수록 커지므로 단조). 측정 횟수는 1(전체) + 1(ellipsis) + ⌈log₂ N⌉로,
  5만 자 입력에서도 약 18회다. 고정 글자 수로 자르지 않고 실제 font measure로 `max_w`를 지킨다.
- 큰 입력 결과: ASCII 5만 자·한글(치트×2만 = 4만 자) 모두 최대 prefix + `…`를 돌려주고, 폭이
  `max_w` 이하이며, **다음 문자를 하나 더 붙이면 초과**함을 단언해 "그냥 들어가는 prefix"가 아니라
  최대 prefix임을 고정했다. 기존 400자 테스트는 유지했다.
- 극소 폭: `max_w`가 0, 1, ellipsis 폭−1 px이면 빈 문자열을 돌려주고 그 폭(0)이 `max_w`를 넘지
  않는다. ellipsis + 한 글자 폭에서는 정확히 `"I…"`를 돌려준다. 짧은 문자열 불변·`…` suffix·
  UTF-8 안전성(문자 경계 절단, prefix 보존)은 그대로다.

## 결함 2 — 위쪽 hidden-row 표시 간격 (수정)

- `more_above_y`가 "첫 행 12px 위"(제목 line box 안)에서 **제목 line box 아래 ~ 첫 행 위의 실제 빈
  대역**으로 이동했다. `title_bottom`(제목 line height 측정)과 `row_y(0)` 사이에서 막대를 가운데
  두고, 양쪽에서 1px 이내로 붙지 않도록 클램프한다. `more_below_y`와 행·힌트 안전영역은 그대로다.
- en/ko × 세 geometry(rgsp 720×480 / rg35xxsp 640×480 / rgcubexx 720×720)에서 위 막대가 보이는
  스크롤 상태를 그려 `title_bottom < 막대 top`, `막대 top + 4 < 첫 행 top`, 그 좌표에 막대가 실제로
  그려짐, 제목과 첫 행 사이 대역 폭 > 막대 높이, 아래 막대는 마지막 행 아래·힌트 위를 각각 단언한다.

## 검증 (최종 코드 상태, 명세 순서)

- `cargo fmt --all -- --check` → 종료 **0**.
- `cargo test -p slot2-ui --test cheat_menu` → 종료 **0**,
  `test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 26.45s`.
- `cargo test -p slot2-i18n --test i18n` → 종료 **0**,
  `test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s`.
- `cargo clippy -p slot2-ui --all-targets -- -D warnings` → 종료 **0**(경고·오류 없음).
- 최종 검증 후 코드 변경 없음.

## 최종 수정 파일

- 이번 회차: `crates/slot2-ui/src/cheat_menu.rs`, `crates/slot2-ui/tests/cheat_menu.rs`만.
- 누적(1회차 포함): 위 2개 + `crates/slot2-ui/src/lib.rs`(모듈 등록·재노출),
  `assets/lang/en.ftl`·`assets/lang/ko.ftl`(각 4키), `crates/slot2-i18n/tests/i18n.rs`(1테스트).
- 번역·`lib.rs`·다른 크레이트는 이번 회차에 수정하지 않았다.

## 남은 위험

위 막대가 사는 대역은 고정 28px 제목 영역에서 측정된 제목 line height를 뺀 폭(현재 약 6px)이라,
line height가 훨씬 큰 폰트가 들어오면 1px 클램프 때문에 이론상 제목과 겹칠 수 있다 — 실제 배포
폰트 조합에서는 테스트가 여유를 확인한다. 2회차 소요 약 7분(전체 45분 내).
