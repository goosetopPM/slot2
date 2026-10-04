# Task 91 — preferred font 마무리 (누적 2/2)

현재 checkout의 1차 구현을 이어서 고친다. 처음부터 다시 만들지 않는다. 1차에서 발견된 두 회귀만
수정하고 전체 완료 기준을 다시 만족시킨다.

이 호출은 **Task91 누적 2/2 마지막 호출**이다. 실패해도 세 번째 호출을 전제로 하지 말고 기존
`tasks/91-language-pack-preferred-font.worker-result.md`를 최종 상태로 갱신한다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\91-language-pack-preferred-font.md`
- `C:\SLOT2\tasks\91-language-pack-preferred-font.worker-result.md`
- `C:\SLOT2\crates\slot2-text\src\imp.rs`의 `Slot`, `ensure_loaded`, `line_metrics`, `measure`,
  `rasterize`만
- `C:\SLOT2\crates\slot2-text\tests\text.rs`의 lazy failure와 line metrics 테스트만
- `C:\SLOT2\crates\slot2-ui\src\cheat_menu.rs`의 layout 상수, `BOX_H`, `row_y`, `title_bottom`,
  `more_above_y`만
- `C:\SLOT2\crates\slot2-ui\tests\cheat_menu.rs`의
  `the_upper_bar_sits_between_the_title_line_and_the_first_row`만
- `C:\SLOT2\crates\slot2-ui\tests\language_font.rs`의 corrupt preferred 테스트만

그 밖의 코드·테스트·로그·이력은 읽지 않는다. 기존 Task91 구현을 정리하거나 다른 설계로 다시 쓰지
않는다.

## 수정 1 — 실패한 첫 font 뒤의 line metrics fallback

현재 `line_metrics`는 `FontId(0)`만 `ensure_loaded`하고 실패하면 `(0, 0, 0)`을 반환한다. 그래서 첫
preferred 파일이 corrupt/missing이면 glyph `resolve`는 뒤 font를 고르지만 bitmap 높이가 0이 되어 실제
텍스트가 사라진다.

- slot을 기존 우선순서대로 순회한다.
- 각 slot을 `ensure_loaded`하고, 정상 load되며 `horizontal_line_metrics(px)`를 제공하는 **첫 font**의
  ascent/descent/line_height를 사용한다.
- failed slot은 기존 `Slot::Failed` 상태를 유지하므로 다시 읽거나 다시 로그하지 않는다.
- 첫 정상 font가 preferred면 계속 preferred metrics를 쓴다. preferred가 실패한 경우에만 OpenSans,
  그마저 실패하면 다음 정상 font로 넘어간다.
- 정상 font가 하나도 없을 때만 기존 `(0.0, 0.0, 0)`을 반환한다.
- glyph resolve 순서, width, glyph cache, raster placement는 바꾸지 않는다.
- `imp.rs` 상단 구현 계약의 “FIRST loaded font” 설명을 **순서상 처음 정상 load된 font**라는 실제 의미로
  고친다.

`crates/slot2-text/tests/text.rs`에 첫 slot이 missing/corrupt lazy font이고 그 뒤에 OpenSans가 있는
체인을 검증한다.

- `measure`의 ascent/descent/line_height가 0이 아니며 뒤 OpenSans 단독 체인과 같다.
- `rasterize` bitmap의 height/data/ink가 0이 아니다.
- 반복 measure/rasterize에서도 실패 slot을 복구하려 들지 않고 정상 결과가 유지된다.
- 기존 `line_height_comes_from_the_first_font_even_for_hangul` 의미는 “첫 **정상** font”로 유지한다.
  정상 첫 font가 있는 기존 체인의 결과를 약화하지 않는다.

`crates/slot2-ui/tests/language_font.rs`의 corrupt preferred 테스트도 width/resolve만 보지 말고
`metrics.line_height > 0`, bitmap height > 0, `ink() > 0`을 확인한다. 이 테스트를 통과시키기 위해
`slot2-ui`에서 corrupt 파일을 eager validate하거나 preferred slot을 숨기지 않는다.

## 수정 2 — Noto line box에 맞는 Cheat 메뉴 제목 영역

한국어 preferred Noto의 `PX_BODY=16` line height는 24px이다. 기존 28px 제목 영역은 4px upper bar가
들어갈 공간과 양쪽 1px clearance를 만들지 못한다.

- `crates/slot2-ui/src/cheat_menu.rs`에 의미가 드러나는 private 상수(예: `TITLE_AREA_H`)를 두고 값을
  **30.0px**로 한다: Noto 24 + bar 4 + 위/아래 1px.
- `BOX_H`, `row_y`, 파일 상단 layout 설명에서 중복된 `28.0`을 이 상수/새 의미로 통일한다.
- row 수, row 높이, bar 크기, padding, hint 위치와 draw 순서는 바꾸지 않는다.
- `more_above_y`의 양쪽 strict clearance와 기존 테스트 단언을 약화하거나 `>=`로 바꾸지 않는다.
- 세 geometry에서 box safe-area와 아래 bar/hint clearance가 계속 성립해야 한다.

임의 font가 30px보다 큰 metrics를 가질 때의 전체 adaptive menu layout은 이번 회복 범위 밖이다. 이번
수정은 저장소가 내장·배포하는 OpenSans/Noto 두 body font의 확정 metrics를 수용하는 최소 회귀 수정이다.

## 수정 허용 파일

- `C:\SLOT2\crates\slot2-text\src\imp.rs`
- `C:\SLOT2\crates\slot2-text\tests\text.rs`
- `C:\SLOT2\crates\slot2-ui\src\cheat_menu.rs`
- `C:\SLOT2\crates\slot2-ui\tests\language_font.rs`의 corrupt preferred 단언만
- `C:\SLOT2\tasks\91-language-pack-preferred-font.worker-result.md`

1차에서 이미 수정한 다른 Task91 파일은 읽거나 다시 손대지 않는다. 직접 깨지는 rustfmt만 허용 파일
안에서 처리한다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 금지

- preferred font 탐색/보안 경계, `UiCtx` chain 순서, i18n/App/runtime 구현 재작성
- `FontChain` public API 변경, 새 dependency·font asset
- Cheat 메뉴 테스트 삭제·완화 또는 bar/row/hint 기능 제거
- 전체 workspace 테스트, 실제 GL 창, device 배포
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 네트워크와 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 코드 변경 뒤 아래를 순서대로 실행한다.

```text
cargo fmt --all -- --check
cargo test -p slot2-text
cargo test -p slot2-ui --test language_font --test cheat_menu
cargo test -p slot2-ui
cargo test -p slot2-i18n
cargo test -p slot2 --test language_startup --test language_picker_app
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2-text -p slot2-ui -p slot2 --all-targets -- -D warnings
```

- 전부 종료 코드 0이어야 한다.
- focused/전체 test result의 passed/failed/ignored 수를 각각 센다.
- core 의존 skip이 있으면 수와 이유를 쓴다.
- 완료 기준 뒤 코드 변경 금지. 변경했으면 영향받는 명령부터 다시 실행한다.

## 결과 보고서 갱신

기존 `C:\SLOT2\tasks\91-language-pack-preferred-font.worker-result.md`를 **누적 호출 2/2** 보고서로
갱신한다.

- 성공/실패와 누적 2/2
- 1차 두 차단 원인의 수정 결과
- failed first slot에서 정상 line metrics/bitmap ink fallback 결과
- Cheat 메뉴 en/ko 및 세 geometry title/bar/row clearance 결과
- Task91 원래 preferred 탐색·lazy/dedup/runtime 계약이 유지된 결과
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored와 skip 수
- 생성·수정 파일 및 최종 검증 뒤 코드 변경 여부
- 남은 범위와 계약이 틀려 보이는 부분
- 이번 호출 및 누적 소요 시간

코드·로그 전문이나 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다. 실패해도
보고서를 남기고 세 번째 호출을 요청하지 않는다.
