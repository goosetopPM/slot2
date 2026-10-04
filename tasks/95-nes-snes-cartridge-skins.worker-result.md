# Task 95 — 작업자 결과 (누적 호출 1/2)

## 결과

**성공 (1/2).** 완료 기준 8개 모두 종료 0. 새 SVG 네 개를 직접 작성해 `PlatformSkin` 테이블에
연결했고, NES/SNES가 GBA 스킨을 빌리던 상태를 각자 artwork로 바꿨다. MD/SMS만 fallback으로 남겼고
production drawing/caching 로직은 손대지 않았다.

## 네 SVG

| 파일 | viewBox | 형상 특징 |
| --- | --- | --- |
| `nes_cart.svg` | 210×270 | 세로형 몸통(4..206 × 4..266), 위쪽 12px 어깨 chamfer 두 개, 아래 connector 쪽 6×8 bevel 두 개. 단일 path 실루엣 |
| `nes_cart_detail.svg` | 210×270 | label 좌우 바깥 세로 groove 두 개(x 9..15, 195..201), label 아래 grip ridge(x 26..184, y 196..203), grip line 두 개, connector moulding(x 62..148, y 236..243) |
| `snes_cart.svg` | 240×190 | 넓고 낮은 몸통(6..234 × 6..184), 위 모서리 46px 라운드 어깨, y=124부터 아래로 좁아지는 taper(228 → 192폭)와 아래 bevel |
| `snes_cart_detail.svg` | 240×190 | label 위 어깨 groove(x 24..216, y 14..20), label 좌우 바깥 grip line 두 개, 아래 ridge(x 34..206, y 146..153), connector moulding(x 74..166, y 164..171) |

- label rect: NES `(20, 28, 170, 142)`, SNES `(25, 34, 190, 92)` — table에 그대로 선언했고 두 rect 모두
  shell의 *painted box* 안에 들어가며 면적이 각각 42.6% / 38.2%로 90% 미만이다.
- shell colour: NES `[0x68,0x69,0x6e]` solid charcoal, SNES `[0x96,0x94,0x9b]` solid light warm gray.
  둘 다 `Finish::Solid`, `borrowed: false`.
- 공통: 투명 배경 위 흰색 coverage만(`fill="#fff"`), text/logo/trademark/bitmap/font/script/filter/
  external reference 없음, path·rect만 사용, generator metadata 없음. shell과 detail은 platform별로
  같은 viewBox를 쓴다.
- 이 detail 두 파일은 **흰색 단일 채널**(alpha = 형태)이다. `ShelfView`가 같은 coverage mask로
  rasterize해 shell 위에 0.8× 색으로 덮으므로 light/shadow 두 channel을 가정하지 않는다.

## provenance

`assets/skins/PROVENANCE.md`에 새 구역을 만들었다: 2026년 SLOT2를 위해 저장소 안에서 직접 그린
original geometric redraw이고 제조사 vector/도면/사진 tracing이나 외부 asset 복사가 없으며, logo·
wordmark·trademark text가 없다는 사실, 네 파일의 viewBox와 역할(shell/detail), 흰 coverage mask이며
runtime tint를 받는다는 점, 그리고 기존 Game Boy detail과 달리 단일 채널이라는 차이를 적었다.
기존 원본 이식 6개 파일의 MIT provenance 문단과 표는 그대로 두었고, 마지막 문단을 "Mega Drive와
Master System만 아직 artwork이 없어 GBA 스킨을 빌린다"로 고쳤다.

## table 결과

- `borrowed == false`: GB, GBC, GBA, NES, SNES (5개). `borrowed == true`: MD, SMS (2개) — 둘은 여전히
  GBA의 `cart`·`cart_detail`·`cart_size`·`label`을 그대로 쓰고, `borrowing` helper도 그대로다.
- NES/SNES의 `cart`·`cart_detail` source는 비어 있지 않고 다섯 독립 플랫폼 사이에서 서로 모두 다르다
  (전 쌍 비교). 존재만 하는 미사용 asset이 아니다: 네 파일 모두 `include_str!`로 테이블에 들어가고
  계약 테스트가 natural/preview 두 크기로 rasterize한다.
- GBA/GB/GBC 계약과 `PlatformSkin` shape/public API, `ShelfView` drawing·caching은 변경 없음.

## 자동 계약 결과 (`crates/slot2-ui/tests/skin.rs`, 10 → 16 tests)

- 독립 5 / fallback 2 구분, MD·SMS가 GBA cart·detail·size·label을 빌림.
- 다섯 독립 플랫폼의 shell/detail source 전 쌍 서로 다름(복사·재사용 0).
- 모든 독립 shell/detail의 `viewBox` == table `cart_size` (기존 테스트 유지, 기존 GB/GBC도 통과).
- NES/SNES label rect가 명세값과 정확히 같고 shell mask의 painted box 안에 들어감 + 면적 90% 미만.
- NES/SNES shell·detail을 natural size(210×270, 240×190)와 48×48 preview fit으로 rasterize.
- NES/SNES shell mask는 불투명 25% 초과, 투명 5% 초과 — blank도 canvas 전체 rectangle도 아님.
- 모든 독립 플랫폼 detail mask는 nonblank이고 같은 크기 shell보다 ink가 작고 shell mask와 다름.
- NES(0.778)와 SNES(1.263) aspect가 0.2 이상 다르고, NES는 세로·SNES는 가로이며 `rasterize_fit`이
  96 박스에서 그 비율을 0.05 이내로 유지.
- 고정 texture id, path 문자열 전체, exact alpha 합계, 경계 pixel에 의존하지 않는다. 기존 테스트는
  삭제·약화 없이 유지했고, 기존 `the_four_shelves_without_artwork_borrow_and_admit_it`은 전제가
  거짓이 되어 `the_two_shelves_without_artwork_borrow_and_admit_it`으로 이름을 바꾸고 단언을
  MD/SMS detail·label까지 늘렸다.

## 문서

- `docs/DESIGN.md` §7: `GBA·GB·GBC는 원본 SVG 이식. NES/SNES/MD/SMS는 제작 전까지 GBA 스킨 폴백.`
  → `GBA·GB·GBC는 원본 SVG 이식, NES·SNES는 SLOT2 자체 카트 SVG(2026…). MD/SMS만 제작 전까지 GBA
  스킨 폴백.` 구조 예시의 port/curve/sfx 필드는 손대지 않아 완료로 읽히지 않는다.
- `docs/MILESTONES.md` M3 스킨 줄: GBA/GB/GBC 이식 + NES/SNES 자체 카트 SVG, MD/SMS만 폴백으로 갱신.
  M6 `NES/SNES/MD/SMS 카트·포트 SVG, 삽입 곡선·효과음`은 `[ ]` 그대로 두고 `NES/SNES 카트 SVG
  완료(Task95). MD/SMS 자산과 플랫폼별 포트·삽입 곡선·효과음은 남음` 진행 메모만 붙였다.

## 완료 기준 명령 (마지막 code/asset 변경 뒤 순서대로 실행)

| # | 명령 | 종료 | 마지막 result 줄 | passed/failed/ignored |
| --- | --- | --- | --- | --- |
| 1 | `cargo fmt --all -- --check` | 0 | (출력 없음) | — |
| 2 | `cargo test -p slot2-ui --test skin` | 0 | `test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s` | 16 / 0 / 0 |
| 3 | `cargo test -p slot2-ui --test shelf_draw` | 0 | `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.66s` | 10 / 0 / 0 |
| 4 | `cargo test -p slot2-ui --test insert` | 0 | `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.41s` | 18 / 0 / 0 |
| 5 | `cargo test -p slot2-ui --test label` | 0 | `test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.95s` | 20 / 0 / 0 |
| 6 | `cargo test -p slot2-ui` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (doc-tests) | 269 / 0 / 0 (27줄) |
| 7 | `cargo check -p slot2 --no-default-features --features device` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.45s`` | — |
| 8 | `cargo clippy -p slot2-ui -p slot2 --all-targets -- -D warnings` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.68s`` | warning 0줄 |

core 의존 skip 없음(모든 result 줄 `0 ignored`). 전체 workspace 테스트, 실제 GL 창, device 배포, 실기
접근은 하지 않았다. 새 cart 크기(210×270, 240×190)로도 `shelf_draw`/`insert`/`label`과 UI 전체가
그대로 통과한다(선반 배치는 `cart_size`에서 계산되므로 회귀 없음).

## 생성·수정 파일

- `assets/skins/nes_cart.svg`, `nes_cart_detail.svg`, `snes_cart.svg`, `snes_cart_detail.svg` (신규 4개).
- `assets/skins/PROVENANCE.md` — 새 구역 + 마지막 문단.
- `crates/slot2-ui/src/skin.rs` — NES/SNES 상수·static, module 문서.
- `crates/slot2-ui/tests/skin.rs` — 계약 6개 추가, borrow 테스트 갱신.
- `docs/DESIGN.md` §7 한 문장, `docs/MILESTONES.md` M3/M6 두 줄.
- `tasks/95-nes-snes-cartridge-skins.worker-result.md` — 이 보고서.
- **최종 검증 뒤 code/test/asset 변경 없음**: 완료 기준 실행 시작(11:18:58 KST)보다 모든 파일 mtime이
  앞선다(문서도 그 전에 수정 완료). 출력 수집 폴더는 삭제했다. 위임·커밋·푸시·네트워크·실기·공용 설정
  접근 없음.

## 후속으로 남긴 것

- 플랫폼별 port SVG와 `PlatformSkin` port API, 공용 slot mouth(ShelfView 코드가 그대로 그림).
- MD·SMS 카트리지 자산과 그 두 platform의 fallback 해제.
- insert/eject curve, 효과음, wallpaper, label renderer.
- 실기에서 새 카트 비율·색이 어떻게 보이는지는 사용자 확인 항목이다.

## 계약이 틀려 보이는 부분

1. 기존 테스트 이름 `the_four_shelves_without_artwork_borrow_and_admit_it`은 이번 변경으로 전제가
   거짓이 된다. 계약은 "기존 테스트를 삭제하거나 약화하지 않는다"고 했으므로 이름만 현재 사실로 바꾸고
   (두 platform) 단언을 오히려 늘렸다(MD/SMS의 detail·label까지 확인). 삭제·약화는 없다.
2. 항목 6의 "투명 pixel이 충분히 있다"는 새 두 shell에만 5% 경계로 적용했다. 이식된 GBA cart는 path가
   캔버스 가장자리(x=0, x=240)까지 닿도록 그려져 있어 같은 경계가 성립하지 않는다. 대신 이식 자산에는
   nonblank와 detail < shell 규칙을 적용해 완전 blank/전체 채움을 배제했다.
3. 새 detail을 흰색 단일 채널로 그렸지만 기존 `gb_cart_detail.svg`는 검정(그림자)+흰색(빛) 두 채널을
   쓴다. `svg::rasterize`가 alpha를 취하므로 검정 도형도 불투명 mask가 되어 두 방식 모두 동작한다.
   이번 태스크는 새 파일만 규칙을 따르게 했고, 기존 파일은 손대지 않았다(PROVENANCE에 차이를 기록).

## 소요 시간

약 26분(11:00–11:26 KST). SVG 작성·배선은 짧았고 대부분 `cargo test -p slot2-ui` 전체와 insert 테스트
반복이었다.
