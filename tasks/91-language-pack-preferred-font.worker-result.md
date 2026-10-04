# Task 91 — 작업자 결과 (누적 호출 2/2)

## 결과

**성공. 누적 2/2(마지막).** 1차의 두 차단 원인을 고쳤고 완료 기준 8개가 모두 종료 0이다. 1차에서
구현한 preferred 탐색·lazy/dedup·runtime 전환 계약은 손대지 않았고 그 테스트도 그대로 통과한다.

## 1차 두 차단 원인의 수정

1. **실패한 첫 slot이 line box를 지우던 문제** — `crates/slot2-text/src/imp.rs`의 `line_metrics`가
   `FontId(0)`만 보고 실패 시 `(0, 0, 0)`을 돌려주던 것을, slot을 순서대로 `ensure_loaded`하며
   `horizontal_line_metrics(px)`를 주는 **첫 정상 font**의 ascent/descent/line_height로 바꿨다. 실패한
   slot은 기존 `Slot::Failed`로 남아 다시 읽거나 다시 로그하지 않는다. glyph `resolve` 순서, width,
   glyph cache, raster 배치는 그대로다. 상단 구현 계약의 “FIRST loaded font” 설명도 “순서상 처음 정상
   load된 font”로 고쳤다.
   - 새 테스트 `a_failed_first_slot_keeps_the_line_box_of_the_fonts_behind_it`: corrupt lazy slot 뒤에
     OpenSans를 둔 체인에서 ascent/descent/line_height/width가 **OpenSans 단독 체인과 동일**하고
     `rasterize`의 height·data·ink가 0이 아니며, 반복 measure/rasterize에서도 실패 slot을 재시도하지
     않고 같은 결과가 나온다.
   - `crates/slot2-ui/tests/language_font.rs`의 corrupt preferred 테스트를 `metrics.line_height > 0`,
     `rasterize` height > 0, `ink() > 0`까지 확인하도록 강화했다. corrupt 파일을 eager validate하거나
     preferred slot을 숨기지 않았고, 간접 검증(resolve)만 하던 것을 실제 line metrics·픽셀까지 본다.
2. **Cheat 메뉴 제목 영역** — `crates/slot2-ui/src/cheat_menu.rs`에 `TITLE_AREA_H = 30.0`(제목 line box
   + upper bar 4px + 위/아래 1px)을 두고 `BOX_H`, `row_y`, 파일 상단 layout 설명의 중복된 `28.0`을 이
   상수로 통일했다. row 수·ROW_H·bar 크기·PADDING·hint 위치·draw 순서는 그대로이고, `more_above_y`의
   양쪽 strict clearance 단언도 약화하지 않았다(상수만 키워 밴드를 확보).

## 검증 결과

- failed first slot fallback: 위 새 테스트가 line metrics·bitmap ink fallback을 확인하고, UI 쪽 corrupt
  preferred 테스트도 같은 사실을 확인한다. corrupt slot은 계속 lazy 등록 상태이며 `resolve('A')`는
  slot1(OpenSans), `resolve('가')`는 slot2(CJK)로 fallback한다.
- Cheat 메뉴: `the_upper_bar_sits_between_the_title_line_and_the_first_row`가 **3 geometry
  (rgsp/rg35xxsp/rgcubexx) × en/ko 6개 조합 전부 통과**한다. ko에서 밴드는 30−24=6px(> MORE_H 4),
  en은 30−22=8px이고 `title_bottom < above`, `above + MORE_H < first_row`, lower bar clearances,
  safe-area 검사가 모두 성립한다. cheat_menu 테스트 16개 전부 통과.
- Task91 원래 계약 유지: `language_font` 7개 테스트가 그대로 통과한다 — effective `I18n::font()`가
  source, 안전한 단일 파일명만 `font_dirs` 순서로 첫 regular file 채택, preferred(지연) → OpenSans
  (eager) → CJK(지연), preferred==CJK면 한 slot만 앞에 등록, missing/unsafe/malformed는 기본 체인,
  context 생성 시 파일을 읽지 않음(`is_loaded` false). runtime 전환 테스트도 preferred 적용·face cache
  교체·missing preference에서도 전환 성공을 유지한다.

## 완료 기준 명령 (마지막 코드 변경 뒤 순서대로 실행)

| # | 명령 | 종료 | 마지막 결과 줄 | passed/failed/ignored |
| --- | --- | --- | --- | --- |
| 1 | `cargo fmt --all -- --check` | 0 | (출력 없음) | — |
| 2 | `cargo test -p slot2-text` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; … finished in 0.00s` (doc-tests) | 12 / 0 / 0 (3줄) |
| 3 | `cargo test -p slot2-ui --test language_font --test cheat_menu` | 0 | `test result: ok. 7 passed; 0 failed; 0 ignored; … finished in 30.39s` | 23 / 0 / 0 (cheat_menu 16 + language_font 7) |
| 4 | `cargo test -p slot2-ui` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; … finished in 0.00s` | 263 / 0 / 0 (27줄) |
| 5 | `cargo test -p slot2-i18n` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; … finished in 0.00s` | 34 / 0 / 0 (3줄) |
| 6 | `cargo test -p slot2 --test language_startup --test language_picker_app` | 0 | `test result: ok. 10 passed; 0 failed; 0 ignored; … finished in 1.31s` | 20 / 0 / 0 (2줄) |
| 7 | `cargo check -p slot2 --no-default-features --features device` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.90s`` | — |
| 8 | `cargo clippy -p slot2-text -p slot2-ui -p slot2 --all-targets -- -D warnings` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 23.22s`` | warning 0줄 |

- core 의존 skip: 모든 result 줄이 `0 ignored`이고 런타임 skip 보고가 없다. core library가 필요한
  slot2-retro 테스트는 이 명령 범위가 아니다.
- 4번은 1차에서 262 / **1** / 0(cheat_menu ko)이었고 지금은 263 / 0 / 0이다.
- 전체 workspace 테스트, 실제 GL 창, device 배포, 실기 접근은 실행하지 않았다.

## 생성·수정 파일 (이번 호출)

- `crates/slot2-text/src/imp.rs` — `line_metrics` fallback + 상단 계약 문구.
- `crates/slot2-text/tests/text.rs` — failed-first-slot 회귀 테스트 1개.
- `crates/slot2-ui/src/cheat_menu.rs` — `TITLE_AREA_H = 30.0`, `BOX_H`/`row_y`/헤더 주석.
- `crates/slot2-ui/tests/language_font.rs` — corrupt preferred 단언 강화.
- `tasks/91-language-pack-preferred-font.worker-result.md` — 이 보고서.
- 1차 산출물(`slot2-ui/src/lib.rs`, `slot2-ui/tests/splash.rs`, `slot2/tests/language_picker_app.rs`,
  `assets/lang/ko.ftl`, `slot2-i18n/tests/i18n.rs`, `docs/DESIGN.md` §8, `docs/DECISIONS.md` D-14)은
  이번 호출에서 손대지 않았다.
- **최종 검증 뒤 코드 변경 없음**: 완료 기준 실행 시작(07:36:22 KST)보다 모든 소스 mtime이 앞선다.
  출력 수집용 임시 폴더는 삭제했다. 위임·커밋·푸시·네트워크·실기·공용 설정 접근 없음.

## 남은 범위

- Noto Sans KR 서브셋 생성과 실기(V-10) 지연 로딩 시간·메모리 측정.
- 한국어·혼합 script 정렬 검토(정렬 정책은 이번 범위 밖).
- `TITLE_AREA_H`는 저장소가 내장·배포하는 OpenSans(22px)/Noto(24px) body line box를 수용하는
  값이다. 그보다 큰 line box를 가진 서드파티 카드 폰트가 preferred가 되면 Cheat 메뉴 제목 밴드가
  다시 좁아진다(이번 회복 범위 밖으로 명시된 항목). 필요하면 title 영역을 측정된 line box에서
  유도하는 후속 작업이 맞다.

## 계약이 틀려 보이는 부분

1차 보고서에 적었던 두 가지는 이번 수정으로 해소됐다: (a) corrupt preferred가 line box를 0으로
만들어 텍스트를 지우던 문제는 `slot2-text`의 “첫 정상 font” metrics 규칙으로, (b) ko에서 Cheat 메뉴
제목 밴드가 4px로 줄어 엄격 clearance를 못 맞추던 문제는 `TITLE_AREA_H = 30.0`으로 해결됐다. 새로
틀린 것으로 보이는 계약은 없다. 다만 위 “남은 범위” 3번(임의 폰트 metrics에 대한 adaptive layout)은
attempt2 명세가 범위 밖으로 둔 제약이며, 계약 위반이 아니라 다음 작업 후보로 남긴다.

## 소요 시간

- 이번 호출(2/2): 약 16분(07:32–07:48 KST).
- 누적: 약 66분(1차 01:17–02:07 약 50분 + 2차 16분). 대부분 `cargo test -p slot2-ui` 반복이었다.
