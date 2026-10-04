# Task 96 — 작업자 결과 (누적 호출 1/2)

## 결과

**성공 (누적 호출 1/2).** 완료 기준 8개 모두 종료 0. MD/SMS에 독립 shell/detail SVG를 추가해 **7개
플랫폼 전부**가 고유 카트리지를 갖게 됐고, 마지막 fallback helper(`borrowing`)를 제거했다.
`PlatformSkin` public shape와 기존 다섯 플랫폼 값, `ShelfView` drawing/caching은 손대지 않았다.

## 네 SVG

| 파일 | viewBox | 형상 특징 |
| --- | --- | --- |
| `md_cart.svg` | 250×180 | 넓고 낮은 shell(8..242 × 14..172). 위 모서리 12px 라운드, 중앙 x 95..155의 얕은 top step(6px 내려앉음), y=142부터 30px 높이로 18px 좁아지는 짧은 taper. SNES의 큰 원호 어깨·긴 taper와 다른 실루엣 |
| `md_cart_detail.svg` | 250×180 | label 위 top-face groove(x 30..220, y 24..29), 좌우 grip recess(x 14..21 / 229..236, y 60..130), 아래 connector ridge 2단(x 40..210 y 138..145, x 80..170 y 158..164) |
| `sms_cart.svg` | 200×230 | MD보다 세로형(10..190 × 10..222). 잘린 윗모서리(14px 대각 두 개), 곧은 옆면, 아래 y=208에서 8px 계단진 connector foot(182..18폭, y 208..222). NES의 긴 직사각 비율·단일 chamfer와 다른 하단 |
| `sms_cart_detail.svg` | 200×230 | label 위 짧은 top groove(x 34..166, y 22..27), 세로 side rail 두 개(x 14..20 / 180..186, y 60..150), label 아래 grip ridge(x 30..170, y 168..175)와 connector ridge(x 44..156, y 192..198) |

- label rect: MD `(32, 34, 186, 84)` = 34.7%, SMS `(24, 32, 152, 118)` = 39.0% — 명세값 그대로 table에
  선언했고, 둘 다 painted shell 안에 완전히 들어가며 canvas 면적의 90% 미만이다.
- shell colour: MD `[0x3d,0x3e,0x43]` solid dark gray, SMS `[0x70,0x72,0x78]` solid mid gray.
- coverage 규칙: 투명 배경 위 `#fff`만. shell은 흰 실루엣(runtime tint), detail은 alpha가 곧 형태인
  흰색 단일 채널이며 `ShelfView`가 shell 위에 0.8× 색으로 덮는다. bitmap/font/script/filter/external
  reference/generator metadata 없음, 상표·문자 없음. shell과 detail은 platform별로 같은 viewBox.
- shell은 canvas 전체 사각형이 아니다: 네 가장자리 모두 투명 여백(예: MD 좌우 8px, SMS 위 10px).

## table·테스트 결과

- 7개 플랫폼 모두 `borrowed == false`, cart/detail source 비어 있지 않음.
- 7개 cart source와 7개 detail source가 각각 **모든 쌍에서 서로 다름**(21쌍 × 2). MD/SMS는 물론 기존
  GB/GBC/GBA/NES/SNES와도 겹치지 않는다.
- 모든 shell/detail의 `viewBox`가 table `cart_size`와 정확히 일치(이제 모든 플랫폼에 detail이 있으므로
  빈 detail 예외 guard를 제거해 오히려 강화).
- 네 자체 제작 카트의 label rect 정확값 + painted shell 내부 + canvas 90% 미만.
- 네 카트 shell/detail을 natural size와 48×48 preview로 rasterize.
- 네 shell은 nonblank, opaque > 25%, clear > 5% — blank도 전체 사각형도 아님.
- 7개 detail 전부 nonblank이고 같은 platform shell보다 ink가 작고 shell mask와 동일하지 않음.
- 비율: NES 0.778(세로) · SNES 1.263(가로) · MD 1.389(가로) · SMS 0.870(세로). MD↔SMS 차 0.519 > 0.25,
  NES↔SNES 차 0.485 > 0.2(Task95 단언 유지), 96×96 `rasterize_fit`이 네 카트 모두 natural aspect를
  0.05 이내로 유지.
- 테스트 수는 16개 그대로다(기존 16개 중 이름·범위만 현재 사실로 일반화, 약화·삭제 없음). 고정 texture
  id·path 문자열·exact alpha 합계·개별 경계 pixel에 의존하는 단언은 없다.

## provenance와 문서

- `assets/skins/PROVENANCE.md`: MD/SMS 네 파일 구역을 추가해 2026년 SLOT2에서 직접 그린 original
  geometric redraw이고 외부 자산·사진·도면을 복제·tracing하지 않았으며 로고·문자·상표가 없다는 점,
  파일별 viewBox와 shell/detail 역할, 흰 coverage와 runtime tint를 기록했다. Task95의 NES/SNES 구역과
  기존 MIT provenance(도입부·표·`gb-cart-lineart` 설명)는 그대로 두었고, 마지막 문단을 “7개 플랫폼
  모두 자기 shell/detail을 가진다(3개 이식 + 4개 자체 제작)”로 교체했다.
- `docs/DESIGN.md` §7: `MD/SMS만 제작 전까지 GBA 스킨 폴백` → `7개 플랫폼 모두 독립 카트 shell/detail을
  가지며 폴백은 없다`(이식/자체 제작 구분 유지). port/curve/sfx 구조 예시 문장은 손대지 않았다.
- `docs/MILESTONES.md`: M3 스킨 줄에 MD/SMS 자체 카트를 포함시키고 “7개 플랫폼 모두 독립 카트, 폴백
  없음”으로 갱신. M6 복합 항목은 `[ ]` 그대로 두고 “네 플랫폼 카트 SVG 완료(Task95~96), 7개 플랫폼
  모두 독립 카트. 플랫폼별 포트·삽입 곡선·효과음은 남음”으로 진행 메모만 갱신.

## 완료 기준 명령 (마지막 code/test/asset 변경 뒤 순서대로 실행)

| # | 명령 | 종료 | 마지막 result 줄 | passed/failed/ignored |
| --- | --- | --- | --- | --- |
| 1 | `cargo fmt --all -- --check` | 0 | (출력 없음) | — |
| 2 | `cargo test -p slot2-ui --test skin` | 0 | `test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s` | 16 / 0 / 0 |
| 3 | `cargo test -p slot2-ui --test shelf_draw` | 0 | `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s` | 10 / 0 / 0 |
| 4 | `cargo test -p slot2-ui --test insert` | 0 | `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 26.31s` | 18 / 0 / 0 |
| 5 | `cargo test -p slot2-ui --test label` | 0 | `test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.25s` | 20 / 0 / 0 |
| 6 | `cargo test -p slot2-ui` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (doc-tests) | 269 / 0 / 0 (27줄) |
| 7 | `cargo check -p slot2 --no-default-features --features device` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.52s`` | — |
| 8 | `cargo clippy -p slot2-ui -p slot2 --all-targets -- -D warnings` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 17.10s`` | warning 0줄 |

모든 test binary에서 failed 0, ignored 0. 새 cart 비율(MD 250×180, SMS 200×230)로도 `shelf_draw`/
`insert`/`label`과 UI 전체가 그대로 통과한다(선반 배치는 `cart_size`에서 계산된다).

## 생성·수정 파일

- `assets/skins/md_cart.svg`, `md_cart_detail.svg`, `sms_cart.svg`, `sms_cart_detail.svg` (신규 4개).
- `assets/skins/PROVENANCE.md` — MD/SMS 구역 추가 + 마지막 문단 교체.
- `crates/slot2-ui/src/skin.rs` — MD/SMS 상수·static을 독립으로, module 문서 갱신, `borrowing` 제거.
- `crates/slot2-ui/tests/skin.rs` — 16개 테스트를 7개 플랫폼 기준으로 일반화(이름 갱신 포함).
- `docs/DESIGN.md` §7 한 문장, `docs/MILESTONES.md` M3/M6 두 줄.
- `tasks/96-md-sms-cartridge-skins.worker-result.md` — 이 보고서.
- **최종 검증 뒤 code/test/asset 변경 없음**: 완료 기준 실행 시작(11:59:42 KST)보다 모든 파일 mtime이
  앞선다(문서 포함). 출력 수집 폴더 삭제, 임시 `.bak`/`.orig` 없음. 커밋·푸시·네트워크·실기·공용 설정
  접근 없음.

## 남은 것

- 플랫폼별 port SVG와 `PlatformSkin` port 계약, 공용 slot mouth(`ShelfView` 코드가 그대로 그림).
- 삽입·배출 곡선과 효과음(자산 `assets/sfx/*.pcm`는 아직 어느 platform에도 연결되지 않았다).
- 실기에서 7개 카트의 비율·색·라벨 크기가 어떻게 보이는지는 사용자 확인 항목이다.

## 계약이 틀려 보이는 부분

1. 항목 6(“두 새 shell의 opaque > 25%, clear > 5%”)을 이번에는 **자체 제작 네 카트**에 적용했다. 이식된
   GBA cart는 path가 캔버스 가장자리(x=0, x=240)까지 닿도록 그려져 있어 같은 5% 여백 규칙이 성립하지
   않는다(GB/GBC도 거의 가득 찬다). 계약이 요구한 대상은 새 shell이므로 범위를 넓히지 않았고, 이식
   자산에는 nonblank·detail < shell 규칙을 그대로 적용했다.
2. 항목 8은 MD↔SMS 비율 차(>0.25)만 요구한다. 자체 제작 네 카트의 비율은 NES 0.778 / SMS 0.870처럼
   서로 가까운 쌍이 있다 — 비율의 pairwise-distinct는 요구되지 않고(항목 2의 distinct는 SVG source에
   대한 것), 실루엣·몰딩이 달라 선반 거리에서 구분된다. 계약 위반은 아니지만 오해 소지가 있어 적는다.
3. `PlatformSkin::borrowed` 필드는 이제 항상 false다. 계약이 “public shape을 바꾸지 않는다”고 했으므로
   필드는 유지했고, helper만 제거했다. 죽은 코드나 `#[allow]`는 남기지 않았다.
