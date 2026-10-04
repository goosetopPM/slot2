# Task 85 — 작업자 결과

- **누적 호출 횟수:** 1/2
- **결과:** 성공 (완료 기준 4개 명령 모두 종료 0)
- **소요 시간:** 약 19분 (13:44 → 14:03 KST, 최종 검증 순차 실행 포함)

## 구현 요약

- 신규 `crates/slot2-ui/src/about_sticker.rs`: `AboutInfo<'a>`(version/target 참조 2개),
  `AboutSticker`(상태 없는 unit struct, `Clone + Copy + Debug + Default + PartialEq + Eq`),
  `AboutSticker::new()`(const), `AboutSticker::box_origin(ctx)`, `draw(canvas, ctx, info)`.
  layout 상수 `BOX_W = 440`, `BOX_H = 260`, `PAD = 16`, 워드마크 크기 `WORDMARK_PX = 32`(본문의 2배),
  줄별 y 상수와 `HINT_Y = BOX_H - PAD - PX_HINT`.
- draw: `Canvas::clear` 없음 → physical panel 전체 `BLACK alpha 0.6` dim → safe area 중앙 panel
  (`BACKDROP`) → `about-title`(PX_BODY INK_DIM) → `about-wordmark`(WORDMARK_PX INK) →
  `about-version`(version arg) → `about-target`(target arg) → `about-license` → `about-notices` →
  `hint-back`(PX_HINT INK_DIM). 모든 줄은 실제 측정 폭으로 가로 중앙 정렬하며 clip/ellipsis/생략 없음.
- 상태·입력·filesystem·clock·환경변수 접근 없음. `CARGO_PKG_VERSION`/host OS/`env!`/git SHA를 읽지
  않는다(값은 호출자가 argument로 넘긴다). 새 dependency·build.rs·asset·색 시스템 없음.
- `lib.rs`의 module/re-export 두 곳만 수정. en/ko pack에 여섯 key를 표의 문구 그대로 추가했고
  `shelf-about`(행 이름)은 그대로 뒀다. `crates/slot2-i18n/tests/i18n.rs`에 신규 key 직접 정의 검증
  1개 추가(총 i18n test 33 → 34).

## 검증 항목별 결과

- **공개 API / state-free / Copy:** `AboutSticker`는 필드가 없는 unit struct이고 `Copy + Eq`다.
  draw가 받는 유일한 동적 값은 `AboutInfo`의 두 `&str`이다. `pub const fn new()`와 re-export 확인.
- **version/target argument:** draw된 quad 폭이 `about-version`/`about-target`을 그 argument로 렌더한
  폭과 정확히 일치(영·한 각각). 다른 값("9999.9999.9999", "rg-other")의 폭과는 일치하지 않아, 값이
  실제로 전달된 argument임을 확인. Rust에서 문자열을 이어 붙이지 않는다.
- **고정 라이선스/고지 문구:** `about-license`는 `SLOT2 · MIT`, `about-notices`는
  `Core and font notices: System/licenses` / `코어 및 글꼴 고지: System/licenses`로 두 pack에 직접
  존재하며 그대로 그려진다. 위 문구가 FTL 안에 있고 코드 조합이 없음을 i18n 테스트가 봉인한다.
- **세 geometry safe area:** `rg35xxsp`(640×480), `rgsp`(720×480), `rgcubexx`(720×720) × en/ko에서
  panel과 모든 glyph/button span이 safe area 안. 720×720에서는 safe area 중심을 사용한다.
  실제 세 패널의 safe area는 모두 중앙이라 panel 중심 규칙과 수치가 같으므로, safe area를 임의로
  (0,0)에 둔 합성 ctx로 두 규칙을 구분해 검증했다(box는 safe 기준 (100,110), panel 중심 규칙이면
  (140,230)).
- **긴 입력:** `version = "1234567890.1234567890"`, `target = "unknown-long-target"`에서 세 geometry ×
  두 언어 모두 모든 quad가 safe area 안이고 각 줄 폭이 box 내부(408px) 이하. 글꼴 크기를 낮추거나
  자르지 않았다.
- **빈 입력:** `version = ""`, `target = ""`에서 panic 없이 두 줄이 그대로 그려지고 나머지 문구도
  정상 표시된다.
- **줄 겹침:** 각 줄의 잉크가 자기 행(slot) 안에만 있고 다음 줄 행을 침범하지 않음을 실제 quad
  경계로 확인(워드마크 32px가 260px box 안에서 아래 줄과 겹치지 않도록 y 상수를 정했다).
- **warm redraw:** 같은 언어/정보로 8회 재드로 시 upload 0, 다음 프레임의 mark 목록도 첫 프레임과 동일.
- **문구 직접 정의:** 여섯 key가 en/ko pack에 직접 존재(`[key]` marker 아님)하고, 표의 정확한 문자열로
  해석됨. `about-wordmark`/`about-license`는 두 언어에서 같은 고정 제품 정보이고, 나머지 네 key는
  번역되어 서로 다름. `hint-back`은 기존 키 재사용.

## 완료 기준 명령 (마지막 코드 변경 뒤 순서대로)

| 명령 | 종료 코드 | 마지막 결과 줄 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | (출력 없음) |
| `cargo test -p slot2-ui --test about_sticker` | 0 | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.01s` |
| `cargo test -p slot2-ui -p slot2-i18n` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (Doc-tests slot2_ui, 최종 줄) |
| `cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.65s` |

- `about_sticker`: **8 passed / 0 failed / 0 ignored**.
- `cargo test -p slot2-ui -p slot2-i18n`: **279 passed / 0 failed / 0 ignored**
  (slot2-ui lib 1 + integration 244, slot2-i18n i18n.rs 34, doctests 0).
- 중간에 clippy 2건(`clippy::too_many_arguments` — production helper 인자 8개,
  `clippy::useless_format` — test 코드)을 allow 없이 구조 수정으로 해결했다: `line` helper가 box x를
  인자로 받지 않고 `box_origin(ctx)`에서 읽도록 바꿔 7개 인자로 줄였다.
- workspace test, 실제 GL test, device 배포는 실행하지 않았다.

## 변경 파일

- 신규: `crates/slot2-ui/src/about_sticker.rs`, `crates/slot2-ui/tests/about_sticker.rs`
- 수정: `crates/slot2-ui/src/lib.rs` (module/re-export 두 줄),
  `crates/slot2-i18n/tests/i18n.rs` (신규 key 검증 1개), `assets/lang/en.ftl`, `assets/lang/ko.ftl`
- 신규: `tasks/85-about-sticker-ui.worker-result.md`

App, Screen/input, `ShelfAvailability`, store/platform, dist script와 license 파일은 수정하지 않았다.
최종 검증 4개 명령 실행 뒤 코드 변경 없음(fmt가 재포맷한 파일은 이번에 건드린 Rust 파일뿐).

## 후속 / 남은 위험

- About 행 활성화(App `Screen` variant, `ShelfAvailability`의 `about` 필드 on, 입력·draw 연결)과
  version/target을 넘기는 호출자 배선이 다음 태스크 범위다. 이 화면은 아직 어디에서도 열리지 않는다.
- 아주 긴 값(예: 100자 이상)은 이 화면이 자르지 않으므로 box 폭을 넘어 safe area까지 벗어날 수 있다.
  명세가 요구한 두 방어 입력은 여유 있게 들어가지만, 호출자가 무제한 문자열을 넘길 수 있다면 자르지
  않고 줄이는 규칙(글꼴 축소 등)을 후속에서 정해야 한다.
- `System/licenses` 안내는 실제 배포 스크립트(`build/dist-device.ps1`의 `System\licenses\fonts`와
  코어 `.meta` 복사)와 일치한다. 라이선스 전문 뷰어·스크롤·URL은 만들지 않았다.
- 계약이 틀려 보이는 부분은 없었다. 다만 "큰 워드마크"의 크기는 명세에 수치가 없어 `BOX_H = 260`
  안에서 아래 줄과 겹치지 않는 32px로 정했다(본문의 2배, splash의 `PX_WORDMARK` 64는 box를 넘침).
