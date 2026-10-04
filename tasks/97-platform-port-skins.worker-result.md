# Task 97 — 작업자 결과 (누적 호출 1/2)

## 결과

**성공 (누적 호출 1/2).** 완료 기준 7개 모두 종료 0. 7개 플랫폼별 port trim SVG를 새로 만들고
`PlatformSkin`에 `port`/`port_size`를 추가해 `ShelfView`의 front 단계에서 합성했다. procedural
back/slit/band/lip과 `MOUTH_*`·`LIP_H`·`SLIT_H` 의미, 삽입 occlusion/travel 수학은 그대로다.

## 7개 port SVG

| platform | file | viewBox | 특징 (trim은 x<20, x>W-20 두 구역) | 중앙 opening |
|---|---|---|---:|---|
| GB | `gb_port.svg` | 294×58 | 각진 side block + 작은 원형 fastener(원 stroke) | 20..274 |
| GBC | `gbc_port.svg` | 294×58 | 둥근 side cap + 짧은 세로 rib 두 개 | 20..274 |
| GBA | `gba_port.svg` | 294×58 | 낮은 wing형 side trim + 얇은 top rail(y 0..3) | 20..274 |
| NES | `nes_port.svg` | 264×58 | 두 단 계단형 side block + 긴 직선 top rail(y 3..6) | 20..244 |
| SNES | `snes_port.svg` | 294×58 | 둥근 shoulder(Q 곡선) + 분리된 bottom corner rail(사이에 gap) | 20..274 |
| MD | `md_port.svg` | 304×58 | 넓은 chamfer side wedge + 중앙이 끊긴 top rail(132..172 gap) | 20..284 |
| SMS | `sms_port.svg` | 254×58 | 좁은 stepped side trim + 짧은 bottom rail 두 개(중앙 회피) | 20..234 |

- viewBox 폭 = `cart_size.0 + MOUTH_EXTRA(14) + 40`, 높이 = `MOUTH_H = 58.0` (계약 표와 일치).
- coverage 규칙: 투명 배경 위 `#fff`만(외부 자산·tracing 없음), 정적 path/rect/circle(+원 stroke)만,
  text·logo·bitmap·font·script·filter·external reference·generator metadata 없음.
- 중앙 opening은 전부 완전히 비어 있다(trim 도형이 x 20..W-20 안에 하나도 없다). top/bottom rail은
  trim 구역과 opening의 위/아래 가장자리에만 걸치고 opening의 세로 중앙(y 37..49)은 비워 둔다.
- 결과: 7개 모두 nonblank이고 canvas 전체를 채우지 않으며(계약 테스트는 opaque coverage < 1/3을 보장; 실제로는 trim 구역만 칠해 그보다 훨씬 적다), 모든 가장자리에 여백이 있다.

## `PlatformSkin`·`ShelfView`

- `PlatformSkin`에 계약대로 `pub port: &'static str`, `pub port_size: (f32, f32)`를 추가했고, 7개 static
  모두 자기 source를 직접 가진다(fallback·공유 없음). cart/detail/label/shell/borrowed 값은 불변.
- 크기는 `const fn port_size(cart_w)` = `(cart_w + MOUTH_EXTRA + 40.0, MOUTH_H)`로 계산해 각 static이
  `port_size(SIZE_X.0)`을 쓴다(숫자 중복 없음). module 문서에 “port trim이 생겼고, slot/bay/band는
  여전히 `shelf_view`가 rect로 만들며 `socket.svg`는 core picker의 IC socket”이라는 구분을 적었다.
- `ShelfView`: `draw_port` helper 하나가 `ArtCache::mask`로 natural `port_size`(정수 round)에 raster/upload
  하고, `x = round((panel_w - port_size.0)/2)`, `y = band_y`, `w = port_size.0`, `h = MOUTH_H`로
  `LIP` 색 tint해 그린다. `draw`와 `draw_insert`가 같은 helper를 쓴다.
- draw order: back(bay/slit) → hint → carts(+travelling cart) → `draw_front`(band/lip) → **port**.
  port를 `draw_front` 뒤에 둔 이유: band가 mouth 바깥을 덮으므로 그 전에 그리면 20px trim이 전부
  가려진다. cart 뒤·front 최상단이라는 계약(port가 카트를 덮고, 중앙은 비어 seated cart가 보임)은
  그대로 지킨다. `MOUTH_H`/`MOUTH_EXTRA`/`LIP_H`/`SLIT_H`와 occlusion/seat/travel 수학은 무변경.

## 계약 테스트 결과 (기존 44개 유지 + 신규 11개)

- `skin` 16 → **21**: 7개 port source nonempty·pairwise distinct, viewBox == `port_size`,
  `port_size.0 == cart_size.0 + 14 + 40`·`port_size.1 == MOUTH_H`, natural/64×58 rasterize + nonblank +
  canvas 전체 채움 거부, 7개 **mask** pairwise distinct(source만 다른 동일 그림 거부), 중앙 ±8px ×
  y 0.65..0.85 구역 coverage 0 + opening 바깥 좌우 trim 각각 coverage 존재.
- `shelf_draw` 10 → **13**: 빈 선반에서도 port가 band 위치·panel 중앙 정렬·panel 안에 그려짐(3 geometry
  × 7 platform), 같은 canvas에서 두 플랫폼 빈 선반의 port texture가 서로 다름(cart/title 없는 상태),
  같은 platform의 반복 redraw·40 frame scroll·빈 행 전환에서 port가 재업로드되지 않음(scroll 중 새로
  선택된 카트의 title face 1회 업로드만 허용, port 크기 upload는 0).
- `insert` 18 → **21**: `seat == 1.0`에서 port op가 travelling cart op보다 뒤(front), port 중앙
  opening 계약(mask에서 그 지점 alpha 0) + draw order를 조합해 seated cart가 중앙에서 가려지지 않음을
  확인(가짜 pixel 합성기 없음), 3 geometry × 7 platform × 대표 insert frame에서 port rect가 finite·
  positive·panel 폭 안.
- 고정 TexId 숫자·전체 op 수·path 문자열·exact alpha 합계·개별 경계 pixel에 맞춘 단언 없음. 기존 테스트
  삭제·약화 없음(이름·주석 전제가 바뀐 곳 없음).
- UI 전체: 269 → **280 passed** (신규 11).

## 문서

- `assets/skins/PROVENANCE.md`: port 7개 구역을 추가 — 2026년 SLOT2에서 직접 그린 original geometric
  redraw, 외부 자산 복제/tracing 없음, logo·문자 없음, 파일별 viewBox와 특징, 흰 coverage + runtime
  `LIP` tint, trim 폭이 mouth + 14px + 좌우 20px이고 높이가 mouth와 같다는 점, **중앙이 비어 있는 것이
  장식이 아니라 기능**이라는 점. 기존 cart/MIT provenance는 보존.
- `docs/DESIGN.md` §7 스킨 문장: 7개 플랫폼 모두 독립 cart shell/detail + 독립 port trim, 폴백 없음,
  port는 front 단계에서 band 위에 합성되고 중앙 opening이 비어 꽂힌 카트리지가 보인다는 사실 추가.
  insert/eject curve·sfx가 아직 없다는 서술은 그대로 두었다.
- `docs/MILESTONES.md`: M3 스킨 줄에 “7개 플랫폼 독립 port trim(Task97)” 추가. M6 복합 항목은 `[ ]`
  그대로 두고 “카트 SVG(Task95~96)와 port trim(Task97) 완료. 삽입 곡선·효과음은 남음”으로 갱신.

## 완료 기준 명령 (마지막 code/test/asset 변경 뒤 순서대로 실행)

| # | 명령 | 종료 | 마지막 result 줄 | passed/failed/ignored |
| --- | --- | --- | --- | --- |
| 1 | `cargo fmt --all -- --check` | 0 | (출력 없음) | — |
| 2 | `cargo test -p slot2-ui --test skin` | 0 | `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s` | 21 / 0 / 0 |
| 3 | `cargo test -p slot2-ui --test shelf_draw` | 0 | `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.80s` | 13 / 0 / 0 |
| 4 | `cargo test -p slot2-ui --test insert` | 0 | `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 28.67s` | 21 / 0 / 0 |
| 5 | `cargo test -p slot2-ui` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (doc-tests) | 280 / 0 / 0 (27줄) |
| 6 | `cargo check -p slot2 --no-default-features --features device` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.04s`` | — |
| 7 | `cargo clippy -p slot2-ui -p slot2 --all-targets -- -D warnings` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 18.66s`` | warning 0줄 |

모든 test binary에서 failed 0, ignored 0.

## 생성·수정 파일

- `assets/skins/{gb,gbc,gba,nes,snes,md,sms}_port.svg` (신규 7개).
- `assets/skins/PROVENANCE.md`, `crates/slot2-ui/src/skin.rs`, `crates/slot2-ui/src/shelf_view.rs`,
  `crates/slot2-ui/tests/{skin,shelf_draw,insert}.rs`, `docs/DESIGN.md`, `docs/MILESTONES.md`.
- `tasks/97-platform-port-skins.worker-result.md` — 이 보고서.
- **최종 검증 뒤 code/test/asset 변경 없음**: 완료 기준 실행 시작(13:16:36 KST)보다 모든 파일 mtime이
  앞선다(문서 포함). 출력 수집 폴더 삭제, 백업 파일 없음. 커밋·푸시·네트워크·실기·공용 설정 접근 없음.

## 남은 것

- 삽입·배출 곡선(작동은 procedural travel이 담당)과 효과음(`assets/sfx/{insert,eject}.pcm`는 아직 어느
  platform에도 연결되지 않음), wallpaper. `PlatformSkin`의 port API는 이번에 닫혔고 mouth_rect/seat_depth
  같은 추가 port 필드는 없다.
- 실기에서 7개 port trim의 색·정렬(카트 폭 대비 trim 위치)이 어떻게 보이는지는 사용자 확인 항목이다.

## 계약이 틀려 보이는 부분

1. 계약은 “port는 front 단계에서 그린다”고만 했고 “front” 안에서의 순서를 지정하지 않았다. procedural
   band/lip이 mouth 바깥 trim을 덮으므로 port는 `draw_front` **뒤**여야 한다(그 전에 두면 trim이 전부
   가려짐). 그래서 back → hint → carts → band/lip → port 순서로 두었고, cart 뒤라는 항목 10 조건과
   중앙 opening 조건은 그대로 만족한다. 이 해석을 코드 주석과 보고서에 남긴다.
2. `docs/DESIGN.md` §7의 구조 예시(`port: Svg, mouth_rect: Rect, seat_depth, insert/eject Curve,
   sfx_*`)는 실제 필드(`port`, `port_size`)와 다르다. 이 스케치는 이번 태스크 이전부터 있던 설계 수준
   서술이고, 계약이 “§7 스킨 문장만” 바꾸라고 했으므로 손대지 않았다. mouth_rect/seat_depth를 구현한
   것으로 읽히지 않도록 문장에는 언급하지 않았다.
3. Task 96 계약의 “`PlatformSkin` public shape을 바꾸지 않는다”와 이번 계약의 “field 두 개를 추가한다”가
   표면적으로 충돌한다. 앞은 그 태스크 시점의 제약이고 이번은 명시적 추가 요구이므로 그대로 추가했다.
   기존 필드 값은 하나도 바뀌지 않았다.
