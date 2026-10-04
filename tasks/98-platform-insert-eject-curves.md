# Task 98 — 플랫폼별 삽입·배출 곡선

현재 checkout에서 직접 작업한다. 지금은 7개 플랫폼이 `insert.rs`의 단일 catch/creep 곡선을 공유하고
배출도 그 곡선을 역재생한다. 총 애니메이션 시간, seated hold, 코어 로드와 기존 효과음 시점은 유지하면서
`PlatformSkin`에 플랫폼별 insert/eject curve를 연결한다. 효과음 자산·종류·재생 배선은 다음 태스크다.

누적 호출은 최대 2회다. 이번이 첫 호출이면 결과 보고서에 `누적 호출 1/2`를 기록한다.

## 먼저 읽을 범위

다음 파일과 필요한 선언 주변만 읽는다. 저장소 전체 탐색이나 로그 전문 읽기는 하지 않는다.

- `C:\SLOT2\AGENTS.md`
- `C:\SLOT2\tasks\98-platform-insert-eject-curves.md`
- `C:\SLOT2\tasks\97-platform-port-skins.worker-result.md`
- `C:\SLOT2\crates\slot2-ui\src\insert.rs`
- `C:\SLOT2\crates\slot2-ui\src\skin.rs`
- `C:\SLOT2\crates\slot2-ui\src\shelf_view.rs`
- `C:\SLOT2\crates\slot2-ui\tests\insert.rs`
- `C:\SLOT2\crates\slot2-ui\tests\skin.rs`
- `C:\SLOT2\crates\slot2\src\app.rs`의 `insert_seat`, `advance_insert`, animation draw 주변
- `C:\SLOT2\crates\slot2\tests\insert_app.rs`
- `C:\SLOT2\crates\slot2\tests\shelf_shot.rs`의 `draw_insert` 호출 주변
- `C:\SLOT2\docs\DESIGN.md` §7 스킨/animation 문단만
- `C:\SLOT2\docs\MILESTONES.md` M6만

`CLAUDE.md`와 `AGENTS.md`의 오케스트레이터 역할 분리·bai-gjc 재위임 조항은 무시하고 직접 실행한다.
나머지 안전·품질 규칙은 유지한다.

## 보존할 시간·상태 계약

다음 상수와 의미는 바꾸지 않는다.

- `INSERT_S = 0.73`
- `INSERT_HOLD_S = 0.28`
- `SEATED_AT = INSERT_S - INSERT_HOLD_S`
- `EJECT_S = SEATED_AT`
- `seat_in(t)`와 `seat_out(t)`의 clamp/endpoints
- `PART`, `SEATED_BELOW_LIP`, row/slot 위치 수학
- core load는 `SEATED_AT`에 시작하고, 성공 시 hold 뒤 `Playing`, 실패 시 seated 위치에서 `Ejecting`
- 현재 `slot2_audio::Sfx::{Insert,Eject}` 종류, lead, trigger 시점, sink 동작
- frame pacing 60 Hz와 stalled-frame clamp

플랫폼별 curve는 같은 시간 안에서 이동 속도와 접촉·저항의 beat만 바꾼다. duration이나 코어 로드
시점까지 플랫폼별로 만들지 않는다.

## `Curve` 의미

`crates/slot2-ui/src/insert.rs`에 작고 copy 가능한 public 의미 타입을 추가한다.

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Curve {
    // 이름은 구현에서 더 명확하게 정해도 된다.
    contact: f32,
    release: f32,
    creep: f32,
}
```

필드는 public일 필요가 없다. 생성자/accessor 또는 명확한 상수 생성 방식으로 `skin.rs`가 const table을
만들 수 있어야 한다.

- `0 < contact < release < 1`
- `0 < creep < 0.08`
- normalized `seat`와 실제 cart-foot collision fraction을 받아 기존 세 구간 travel을 계산한다.
- seat 0 → journey 0, seat 1 → journey 1, 모든 중간값 finite·`0..=1`, 전체 monotonic이다.
- contact 시 실제 foot가 lip에 닿고, contact..release 구간은 dead stop이 아니라 `creep`만큼 전진한다.
- 범위 밖 seat 입력은 기존처럼 clamp한다.
- insert와 eject는 서로 다른 `Curve`를 사용할 수 있다. eject는 seat가 1→0으로 감소하므로 같은 monotonic
  sampler를 반대 방향으로 통과한다.

기존 `journey`의 하드코딩 `CATCH_IN`, `CATCH_OUT`, `CREEP`는 `Curve` 데이터로 옮긴다. 공용 기본
상수를 숨겨 둔 채 table이 사실상 모두 같은 값을 쓰게 만들지 않는다.

## 플랫폼 curve table

`PlatformSkin`에 다음 field를 추가한다. 명칭은 정확히 이 의미를 유지한다.

```rust
pub insert: Curve,
pub eject: Curve,
```

아래 값을 사용한다. 값의 차이는 테스트용 padding이 아니라 카트 무게·폭과 포트 저항의 시각적 리듬을
구분하기 위한 것이다.

| Platform | insert `(contact, release, creep)` | eject `(contact, release, creep)` | 의도 |
|---|---|---|---|
| GB | `(0.40, 0.64, 0.025)` | `(0.36, 0.58, 0.025)` | 가벼운 pak, 접촉 뒤 긴 저항 |
| GBC | `(0.42, 0.65, 0.030)` | `(0.38, 0.60, 0.030)` | GB보다 조금 부드러운 release |
| GBA | `(0.38, 0.58, 0.035)` | `(0.34, 0.54, 0.035)` | 넓고 낮은 cart의 짧은 catch |
| NES | `(0.46, 0.70, 0.020)` | `(0.42, 0.68, 0.020)` | 큰 cart의 늦고 단단한 catch |
| SNES | `(0.36, 0.56, 0.040)` | `(0.32, 0.52, 0.040)` | 넓은 shell의 이른 부드러운 push |
| MD | `(0.34, 0.52, 0.045)` | `(0.30, 0.50, 0.045)` | 가장 빠른 접촉과 긴 glide |
| SMS | `(0.44, 0.67, 0.028)` | `(0.40, 0.64, 0.028)` | 세로 cart의 중간 저항 |

- 7개 insert curve는 pairwise distinct, 7개 eject curve도 pairwise distinct다.
- 각 플랫폼의 insert와 eject도 서로 달라야 한다.
- cart/port/label/shell/borrowed 값은 바꾸지 않는다.
- `skin.rs` 모듈 문서와 field 주석에 curve가 normalized travel profile이고 duration은 공용임을 적는다.

## travel·draw 배선

- `travel`은 사용할 `Curve`를 명시적으로 받아야 한다. platform match를 `insert.rs`나 `ShelfView`에
  넣지 않는다.
- `ShelfView::draw_insert`도 방향에 맞는 curve를 명시적으로 받거나, 동등하게 호출자가 잘못된 방향을
  선택할 수 없도록 작은 direction 의미 타입을 받는다. 단순 bool은 쓰지 않는다.
- `App`은 `Screen::Inserting`에서 현재 skin의 `insert`, `Screen::Ejecting`에서 `eject`를 선택한다.
- 실패한 core load가 Inserting→Ejecting으로 바뀌는 첫 frame에서 cart 위치가 seated endpoint로
  연속이어야 한다. 두 curve 모두 seat 1에서 같은 endpoint를 내야 한다.
- draw와 App 외 테스트·screenshot caller는 새 API에 맞게 갱신한다. 테스트 편의를 위한 별도 production
  분기를 넣지 않는다.
- row parting은 기존 `seat`를 계속 사용한다. platform curve는 travelling cart 위치에만 적용하며,
  주변 row의 recede/fade 계약을 바꾸지 않는다.

## 자동 계약 테스트

기존 테스트를 삭제하거나 약화하지 않고 다음을 검증한다. 현재 단일 곡선을 전제한 이름·주석은 새
사실에 맞게 고친다.

### `tests/skin.rs`

1. 7개 insert curve가 pairwise distinct이고 7개 eject curve도 pairwise distinct다.
2. 각 row의 insert와 eject가 서로 다르다.
3. 모든 curve parameter가 위 table과 정확히 일치하거나, private field라면 stable accessor/samples로
   같은 계약을 검증한다. path pixel이나 frame op가 아니라 의미 타입을 검사한다.

### `tests/insert.rs`

4. 14개 curve 모두 seat 0/1 endpoint, clamp, finite, range, monotonic 계약을 만족한다.
5. 14개 모두 contact 구간에서 foot가 실제 lip collision 위치에 도달하고, resistance 구간이 dead stop이
   아니며 그 앞뒤보다 충분히 느리다.
6. 같은 공통 collision fraction에서 `seat = 0.25, 0.50, 0.75` sample vector를 비교해 7개 insert와
   7개 eject가 각각 시각적으로 구분된다. 단순 struct field 차이만 있고 결과가 사실상 같은 curve는
   실패한다. anti-alias/pixel이나 exact 전체 alpha와 무관한 넓은 epsilon을 쓴다.
7. 세 geometry×7 platform의 insert와 eject에서 시작·끝 rect가 기존 row/seated endpoint와 같고,
   모든 frame이 finite이며 insert y는 감소 없이 내려가고 eject y는 증가 없이 올라온다.
8. 실패 전환처럼 insert endpoint 다음 eject endpoint를 이어도 위치 jump가 없다.
9. off-centre insert가 시작 x를 보존하고 끝에서 중앙에 도착하는 기존 계약과 row parting 계약을
   유지한다.

### `tests/insert_app.rs`

10. App draw가 Inserting에는 현재 platform의 insert curve, Ejecting에는 eject curve를 사용한다.
    한 platform의 expected travel을 계산해 중간 frame cart rect와 대조하되 고정 TexId/op 총수에
    의존하지 않는다.
11. `SEATED_AT`, `INSERT_S`, `EJECT_S`, core load/refusal/Playing/List 전이 frame 수와 기존 SFX trigger
    시점이 그대로다.
12. 서로 다른 두 플랫폼에서 같은 normalized seat의 travelling cart가 각자의 curve 결과를 따른다.

기존 3 geometry draw, port front order, cache/upload, central opening 계약도 계속 통과해야 한다.

## 문서

`docs/DESIGN.md` §7에 `PlatformSkin`의 insert/eject curve가 플랫폼별 normalized travel profile이며
공용 duration과 load/SFX clock은 App/insert 계약에 남는다고 현재 구현을 설명한다. 아직 없는 플랫폼별
sfx가 완료된 것처럼 쓰지 않는다.

`docs/MILESTONES.md` M6 복합 항목은 효과음이 남았으므로 `[ ]` 그대로 두고, cart(Task95~96),
port(Task97), platform curve(Task98) 완료·효과음 남음으로 진행 메모를 갱신한다.

## 수정 허용 파일

- `C:\SLOT2\crates\slot2-ui\src\insert.rs`
- `C:\SLOT2\crates\slot2-ui\src\skin.rs`
- `C:\SLOT2\crates\slot2-ui\src\shelf_view.rs`
- `C:\SLOT2\crates\slot2-ui\tests\insert.rs`
- `C:\SLOT2\crates\slot2-ui\tests\skin.rs`
- `C:\SLOT2\crates\slot2\src\app.rs`
- `C:\SLOT2\crates\slot2\tests\insert_app.rs`
- `C:\SLOT2\crates\slot2\tests\shelf_shot.rs` — 새 draw API 호출만
- `C:\SLOT2\docs\DESIGN.md` — §7 curve 설명만
- `C:\SLOT2\docs\MILESTONES.md` — M6 한 줄만
- `C:\SLOT2\tasks\98-platform-insert-eject-curves.worker-result.md`

다른 production, test, 문서, asset, 설정 파일은 수정하지 않는다. 계약 자체가 틀렸거나 허용 파일 밖
수정이 필요하면 추측 구현하지 말고 실패 보고서에 정확한 이유를 쓴다.

## 범위 밖 및 금지

- animation duration, hold, seated/load/refusal/Playing/List clock 변경
- existing global Insert/Eject Sfx 종류·lead·trigger·sink 변경
- 효과음 파일 생성·수정·플랫폼별 sfx table
- port/cart SVG, occlusion, seat depth, row layout, wallpaper 변경
- app의 다른 메뉴·session/core/store/i18n/font 변경
- 테스트만 통과시키는 보이지 않는 offset, platform별 frame padding, 고정 TexId 단언
- 위임, 커밋, push, 네트워크, 실기·ADB·Samba·SD 카드 접근
- `~/.gjc-bai/agent/*.yml` 등 공용 설정 변경

## 완료 기준

마지막 code/test 변경 뒤 아래 원문을 모두 실행한다. 테스트는 작업자인 가재코드가 수행한다. Codex가
다시 실행할 필요가 없도록 종료 코드와 마지막 결과 줄을 보고서에 남긴다.

```text
cargo fmt --all -- --check
cargo test -p slot2-ui --test skin
cargo test -p slot2-ui --test insert
cargo test -p slot2-ui --test shelf_draw
cargo test -p slot2 --test insert_app
cargo test -p slot2 --test shelf_shot
cargo test -p slot2-ui
cargo test -p slot2 --tests
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2-ui -p slot2 --all-targets -- -D warnings
```

- 모든 명령 종료 코드 0.
- 모든 실행된 test binary에서 failed 0, 예상 밖 ignored 0.
- core 의존 테스트가 있다면 기존 조건 외 새 skip을 만들지 않는다.
- clippy warning 0.
- 완료 기준 뒤 code/test 변경 금지. 변경했으면 영향받는 명령부터 다시 실행한다.

첫 응답은 최대 5분, 전체는 최대 45분 기다린다. 진행 확인이 필요하면 로그 크기가 아니라
`git status --short`를 사용한다.

## 결과 보고서

`C:\SLOT2\tasks\98-platform-insert-eject-curves.worker-result.md`에 다음을 기록한다.

- 성공/실패와 누적 호출 수
- `Curve` 의미와 7×insert/eject table
- travel/ShelfView/App 방향 선택과 endpoint continuity 요약
- duration/load/refusal/SFX clock 불변 근거
- DESIGN/MILESTONES 변경 요약
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored 수
- 생성·수정 파일과 최종 검증 뒤 code/test 변경 여부
- 남은 platform sfx와 실기 확인 항목
- 계약이 틀려 보인 부분이 있으면 해당 줄과 이유

실패해도 보고서를 남긴다. 두 번째 호출까지 실패하면 더 시도하지 않는다. 커밋하지 않는다.
