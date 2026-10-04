# Task 51 최종 보완 — 말줄임 비용과 위쪽 표시 간격

## 상태

Task 51 누적 호출 **2/2, 마지막 허용 호출**이다. 1회차의 선택·스크롤·빈 상태·영한 번역·safe
area·warm redraw 동작과 공개 API는 승인한다. 아래 두 결함만 고치고 다른 범위로 확장하지 않는다.

## 결함 1 — 긴 description의 제곱 시간 측정

현재 `fit`은 전체 문자열을 한 번 잰 뒤, 잘라낼 위치를 찾으면서 모든 Unicode scalar prefix를
처음부터 다시 측정한다. 길이가 N이면 shaping할 문자량이 O(N²)이고 이 함수는 visible 행마다 매
draw 호출된다. `.cht` description 길이에 상한이 없으므로 작은 기기에서 긴 한 줄이 메뉴를 멈추게
할 수 있다.

- UTF-8 문자 경계 목록을 한 번 만들고 폭에 맞는 가장 긴 prefix를 이분 탐색하는 방식처럼,
  전체 측정 횟수를 O(log N) 수준으로 줄인다. 단순히 고정 글자 수로 자르지 말고 실제 font measure
  결과로 `max_w`를 지켜야 한다.
- 짧은 문자열 불변, `…` suffix, UTF-8 안전성은 유지한다.
- `max_w`가 0이거나 ellipsis 자체보다 좁은 경우에도 반환 문자열의 측정 폭이 `max_w`를 넘지 않게
  한다. 이때 빈 문자열 반환을 허용한다.
- 400자 기존 테스트를 유지하고, 수만 글자의 ASCII와 한글 입력에서도 올바른 prefix/ellipsis를
  내며 합리적인 시간에 끝나는 경계 테스트를 추가한다. 불안정한 wall-clock 상한 assertion은 넣지
  말고 알고리즘 구조와 큰 입력 결과로 검증한다.

## 결함 2 — 위쪽 hidden-row 표시가 제목과 겹침

현재 위 막대 y는 첫 행 top보다 12px 위다. 제목은 `by + PAD`에서 PX_BODY line height만큼 그려져
해당 막대와 세로 영역이 겹칠 수 있다.

- 위 막대를 `제목 line bottom < 막대 top < 막대 bottom < 첫 행 top`인 실제 빈 공간에 둔다.
- 아래 막대와 행·힌트의 기존 안전영역은 유지한다.
- en/ko와 세 geometry에서, 위 막대가 실제로 보이는 스크롤 상태를 그려 제목·첫 행 어느 쪽과도
  겹치지 않는 좌표 assertion을 추가한다.

## 수정 허용 범위

- `crates/slot2-ui/src/cheat_menu.rs`
- `crates/slot2-ui/tests/cheat_menu.rs`
- `tasks/51-cheat-menu-ui.worker-result.md`

번역, lib.rs와 다른 크레이트는 수정하지 않는다.

## 금지

- 승인된 UI/API 재설계
- App/Session 연결, `.cht` 쓰기, 코어별 검증
- 테스트 전용 제품 분기와 lint allow
- 커밋·푸시·네트워크·실기·공용 설정 접근
- 세 번째 호출

## 최종 검증

최종 코드 상태에서 아래를 원문 그대로 순서대로 실행한다.

```text
cargo fmt --all -- --check
cargo test -p slot2-ui --test cheat_menu
cargo test -p slot2-i18n --test i18n
cargo clippy -p slot2-ui --all-targets -- -D warnings
```

검증 뒤 소스를 고치면 네 명령을 다시 실행한다. 전체 45분을 넘기지 않는다.

## 보고

기존 `C:\SLOT2\tasks\51-cheat-menu-ui.worker-result.md`를 **누적 2/2 최종 보고서**로 갱신한다.

- 1회차 승인 동작 유지 여부
- 새 `fit` 탐색 방식과 큰 ASCII/한글·극소 폭 결과
- 위 막대와 제목/첫 행의 실제 간격 검증
- 네 검증 명령의 종료 코드와 마지막 결과 줄
- 최종 수정 파일
- 실패 시 원인과 남은 위험

이번 호출이 실패해도 세 번째 시도는 하지 않는다.
