# Task 98 — 작업자 결과 (누적 호출 1/2)

## 결과

**성공 (누적 호출 1/2).** 완료 기준 10개 모두 종료 0. `PlatformSkin`에 플랫폼별 insert/eject curve를
넣고 `travel`·`ShelfView::draw_insert`·`App` 배선을 방향별로 바꿨다. 애니메이션 길이·seated hold·코어
로드·거부·`Playing`/`List` 전이·효과음 종류·lead·trigger 시점은 그대로다.

## `Curve` 의미와 7×2 table

```rust
pub struct Curve { contact: f32, release: f32, creep: f32 }   // private fields
Curve::new(contact, release, creep)   // const fn: 0<contact<release<1, 0<creep<0.08 를
                                      // const 평가 시점에 assert → table 오타는 build 실패
Curve::contact() / release() / creep()      // accessor
Curve::journey(collision, seat) -> f32      // 3구간 sampler
```

`journey`는 정규화된 `seat`와 **실제 foot-lip collision fraction**(`collision_at(safe, h)`)을 받아
세 구간을 계산한다: `seat < contact`는 `collision * ease(seat/contact)`(lip까지 낙하),
`contact..release`는 `collision + creep`까지 선형 전진(접촉 저항, dead stop 아님),
그 뒤는 `ease`로 정착. `collision`은 [0,1]로 clamp하고 `collision+creep`을 1로 상한해 어떤 입력에도
monotonic·finite·`0..=1`이 유지된다(실제 table에서는 collision ≈ 0.15~0.45, creep ≤ 0.045라 원래
산술과 동일).

| Platform | insert (contact, release, creep) | eject (contact, release, creep) |
|---|---|---|
| GB | (0.40, 0.64, 0.025) | (0.36, 0.58, 0.025) |
| GBC | (0.42, 0.65, 0.030) | (0.38, 0.60, 0.030) |
| GBA | (0.38, 0.58, 0.035) | (0.34, 0.54, 0.035) |
| NES | (0.46, 0.70, 0.020) | (0.42, 0.68, 0.020) |
| SNES | (0.36, 0.56, 0.040) | (0.32, 0.52, 0.040) |
| MD | (0.34, 0.52, 0.045) | (0.30, 0.50, 0.045) |
| SMS | (0.44, 0.67, 0.028) | (0.40, 0.64, 0.028) |

14개 curve 전부 pairwise distinct, 각 플랫폼의 insert ≠ eject, `borrowed`와 cart/port/label/shell
값은 불변. `skin.rs` 모듈 문서와 field 주석에 “curve는 normalized travel profile이고 duration·
load·SFX clock은 공용”이라고 적었다.

## travel·draw·App 배선

- `travel(safe, curve, cart, rest_x, seat)`: curve를 **명시적으로** 받는다. platform match는
  `insert.rs`에도 `ShelfView`에도 없다.
- `ShelfView::draw_insert(canvas, ctx, safe, platform, titles, at: Insertion)`:
  `pub struct Insertion { pub seat: f32, pub motion: Motion }` 하나로 받고, `Motion::{Insert,Eject}`로
  `skin.insert`/`skin.eject`를 고른다. bool이 아니고, 호출자가 seat와 방향을 따로 넘겨 잘못 짝지을 수
  없다. row parting/recede는 기존 `seat`를 그대로 쓴다.
- `App`: `Screen::Inserting → Motion::Insert`, `Screen::Ejecting → Motion::Eject`를 그 한 곳에서 정해
  `draw_insert`에 넘긴다.
- endpoint continuity: 두 curve 모두 `journey(_, 1.0) == 1`이라 seat 1에서 같은 rect다. 거부 프레임
  (`Inserting`→`Ejecting`, `anim = 0`)에서 cart가 점프하지 않음을 7플랫폼×3 geometry로 단언한다.
- off-centre insert(시작 x 보존, 끝 중앙 도착)와 row parting 계약은 기존 테스트가 그대로 통과한다.

## 불변 근거 (duration/load/refusal/SFX clock)

- `INSERT_S`, `INSERT_HOLD_S`, `SEATED_AT`, `EJECT_S`, `seat_in`/`seat_out` clamp, `PART`,
  `SEATED_BELOW_LIP`, row/slot 수학: 무변경.
- `advance_insert`의 `SEATED_AT` 로드·거부(`anim = 0`)·`INSERT_S`에서 `Playing`·`EJECT_S`에서 `List`
  로직과 효과음(`Sfx::{Insert,Eject}`, lead, trigger, sink 동작) 무변경 → 기존 테스트
  `the_core_is_not_asked_for_until_the_cart_seats`(SFX lead 창), `a_core_that_will_not_load_sends_the
  _cart_back_out`(eject frame 수 == EJECT_S×60 ±1), `the_cart_goes_in_on_the_clock`,
  `the_animation_runs_at_the_panels_rate`, `a_stalled_frame_does_not_throw_the_cart_through_the_floor`
  가 수정 없이 통과.
- 추가로 `the_clock_is_the_same_on_every_shelf`: curve가 크게 다른 GBA와 NES에서 seat 도달 frame 수와
  eject 전체 frame 수가 동일함을 확인(둘 다 27, 27).

## 자동 계약 테스트 (기존 유지 + 신규 10개)

- `tests/skin.rs` 21 → **23**: 14 curve pairwise distinct(insert 7, eject 7, 같은 플랫폼 insert≠eject),
  table 값과 범위를 accessor로 검사(path pixel·frame op 아님).
- `tests/insert.rs` 21 → **26**: 14 curve × 3 geometry의 seat 0/1 endpoint·clamp·finite·`0..=1`·
  monotonic, contact에서 foot가 정확히 lip에 닿고(`y+h == lip`, 0.5px) 저항 구간이 dead stop이 아니며
  낙하·push 구간보다 3배 이상 느림, 공통 collision 0.4에서 seat 0.25/0.50/0.75 sample vector가
  7개 insert·7개 eject 각각 pairwise 구분(성분 차 > 0.005; 가장 가까운 쌍 GB/GBC는 ~0.02),
  시작 rect == row 배치·끝 rect == seated endpoint(3 geometry × 7 platform × 양방향)와 방향별 단조 y,
  insert/eject가 seat 1에서 같은 rect(거부 프레임 연속).
- `tests/insert_app.rs` 13 → **16**: App이 `Inserting`에 그 플랫폼 insert curve, `Ejecting`에 eject
  curve를 쓰는지 중간 frame cart rect와 계산된 expected travel로 대조(TexId·op 총수 비의존), 같은
  정규화 seat에서 두 플랫폼(GBA/NES) cart가 각자 profile을 따르고 journey가 0.02 이상 다름, shelf가
  달라도 clock이 같음.
- 고정 TexId·전체 op 수·path 문자열·exact alpha·개별 경계 pixel 단언 없음. 기존 테스트 삭제·약화 없음
  (이름·주석 전제가 바뀐 곳은 새 API에 맞게 갱신).

## 문서

- `docs/DESIGN.md` §7: `PlatformSkin`의 `insert`/`eject`가 플랫폼별 normalized travel profile
  (contact·release·creep, 애니메이션 전체에 대한 비율)이고 애니메이션 길이·seated hold·코어 로드
  시점·효과음 cue는 `insert`/App 계약의 공용 clock이라고 추가. 플랫폼별 효과음은 아직 없다는 사실을
  명시했고 port/cart 설명은 그대로다.
- `docs/MILESTONES.md` M6 복합 항목은 `[ ]` 그대로 두고 “카트 SVG(Task95~96), port trim(Task97),
  플랫폼별 insert/eject curve(Task98)까지 완료. 플랫폼별 효과음은 남음”으로 진행 메모 갱신.

## 완료 기준 명령 (마지막 code/test 변경 뒤 순서대로 실행)

| # | 명령 | 종료 | 마지막 result 줄 | passed/failed/ignored |
| --- | --- | --- | --- | --- |
| 1 | `cargo fmt --all -- --check` | 0 | (출력 없음) | — |
| 2 | `cargo test -p slot2-ui --test skin` | 0 | `test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s` | 23 / 0 / 0 |
| 3 | `cargo test -p slot2-ui --test insert` | 0 | `test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 39.95s` | 26 / 0 / 0 |
| 4 | `cargo test -p slot2-ui --test shelf_draw` | 0 | `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.48s` | 13 / 0 / 0 |
| 5 | `cargo test -p slot2 --test insert_app` | 0 | `test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.07s` | 16 / 0 / 0 |
| 6 | `cargo test -p slot2 --test shelf_shot` | 0 | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | 1 / 0 / 0 |
| 7 | `cargo test -p slot2-ui` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (doc-tests) | 287 / 0 / 0 (27줄) |
| 8 | `cargo test -p slot2 --tests` | 0 | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.75s` | 339 / 0 / 0 (30줄) |
| 9 | `cargo check -p slot2 --no-default-features --features device` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.09s`` | — |
| 10 | `cargo clippy -p slot2-ui -p slot2 --all-targets -- -D warnings` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 54.58s`` | warning 0줄 |

모든 test binary에서 failed 0, ignored 0. core 의존 테스트(`insert_app`의 `a_cart_that_loads…`)는 기존
`vendor/mgba_libretro.*` 조건 그대로이며 새 skip 조건을 만들지 않았다. 전체 workspace 테스트, 실제 GL
창, device 배포, 실기 접근은 하지 않았다.

## 생성·수정 파일

- `crates/slot2-ui/src/insert.rs` — `Curve`(+const 생성자/accessor/journey), `Motion`, `Insertion`,
  `collision_at` 공개, `travel`이 curve를 받음. 하드코딩 `CATCH_IN`/`CATCH_OUT`/`CREEP` 제거.
- `crates/slot2-ui/src/skin.rs` — `insert`/`eject` field, 7행 table, module·field 문서.
- `crates/slot2-ui/src/shelf_view.rs` — `draw_insert(..., at: Insertion)`과 방향→curve 선택.
- `crates/slot2-ui/tests/{skin,insert}.rs`, `crates/slot2/src/app.rs`, `crates/slot2/tests/{insert_app,
  shelf_shot}.rs` — 새 API 호출부와 계약 테스트.
- `docs/DESIGN.md` §7 한 문장, `docs/MILESTONES.md` M6 한 줄.
- `tasks/98-platform-insert-eject-curves.worker-result.md` — 이 보고서.
- **최종 검증 뒤 code/test 변경 없음**: 완료 기준 실행 시작(14:28:45 KST)보다 모든 파일 mtime이
  앞선다(문서 포함). 출력 수집 폴더 삭제, 임시 파일 없음. 커밋·푸시·네트워크·실기·공용 설정 접근 없음.

## 남은 것

- 플랫폼별 효과음 파일·종류·table(Task99 예정). 지금은 기존 공용 `Sfx::{Insert,Eject}` 하나뿐이다.
- 실기에서 7개 플랫폼 curve의 리듬 차이가 어떻게 보이는지는 사용자 확인 항목이다(호스트 창/실기).

## 계약이 틀려 보이는 부분

1. `draw_insert`에 `seat`와 `Motion`을 따로 넘기면 인자가 8개가 되어 clippy `too_many_arguments
   (8/7)`로 실패한다. 이 저장소는 “린트를 allow로 덮지 않는다”는 규칙이 있어 `#[allow]` 대신
   `Insertion { seat, motion }` 한 값으로 묶었다 — 계약이 허용한 “작은 direction 의미 타입”과 같은
   취지이고, seat와 방향의 짝을 타입으로 강제한다.
2. 항목 6의 “시각적으로 구분”을 성분 차 > 0.005로 구현했다. 가장 가까운 쌍(GB/GBC insert)이 0.25
   sample에서 약 0.020 차이라 epsilon을 그 이상으로 조이면 계약의 table 자체가 실패한다. 지금 epsilon은
   “사실상 같은 curve”를 걸러내기에 충분하지만, table 값이 조금만 더 가까워지면 이 테스트가 먼저
   깨진다는 점을 기록해 둔다.
3. `catch_at`을 `collision_at`으로 이름을 바꾸고 공개했다. 계약의 sampler가 collision fraction을
   인자로 받으므로 테스트가 그 값을 계산할 공개 경로가 필요했다(허용 파일 안의 public 추가).
4. `Curve::journey`는 `collision`을 [0,1]로 clamp하고 `collision+creep`을 1로 상한한다. 현재 table과
   실제 geometry에서는 원래 산술과 동일하지만, 카트 높이가 비정상적으로 큰 미래의 값에서도
   단조성이 깨지지 않게 하는 방어다(동작 변화 없음).
