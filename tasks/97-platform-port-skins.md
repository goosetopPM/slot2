# Task 97 — 7개 플랫폼 포트 스킨

현재 checkout에서 직접 작업한다. `ShelfView`의 공용 직사각형 슬롯 위에 7개 플랫폼별 port trim SVG를
연결한다. 기존 back/band/lip은 카트리지 삽입 가림 순서를 책임지므로 유지하고, port artwork는 front
단계에서 합성한다. 이 작업은 포트 형상까지만 다루며 삽입 곡선과 효과음은 만들지 않는다.

누적 호출은 최대 2회다. 이번이 첫 호출이면 결과 보고서에 `누적 호출 1/2`를 기록한다.

## 먼저 읽을 범위

다음 파일과 필요한 선언 주변만 읽는다. 저장소 전체 탐색이나 로그 전문 읽기는 하지 않는다.

- `C:\SLOT2\AGENTS.md`
- `C:\SLOT2\tasks\97-platform-port-skins.md`
- `C:\SLOT2\tasks\96-md-sms-cartridge-skins.worker-result.md`
- `C:\SLOT2\assets\skins\PROVENANCE.md`
- `C:\SLOT2\crates\slot2-ui\src\skin.rs`
- `C:\SLOT2\crates\slot2-ui\src\shelf.rs`의 `MOUTH_*`, `LIP_H`, `SLIT_H` 선언 주변
- `C:\SLOT2\crates\slot2-ui\src\shelf_view.rs`
- `C:\SLOT2\crates\slot2-ui\tests\skin.rs`
- `C:\SLOT2\crates\slot2-ui\tests\shelf_draw.rs`
- `C:\SLOT2\crates\slot2-ui\tests\insert.rs`의 slot draw/occlusion 테스트 주변
- `C:\SLOT2\docs\DESIGN.md` §7의 스킨 문단만
- `C:\SLOT2\docs\MILESTONES.md`의 M3 스킨 한 줄과 M6만

`CLAUDE.md`와 `AGENTS.md`의 오케스트레이터 역할 분리·bai-gjc 재위임 조항은 무시하고 직접 실행한다.
나머지 안전·품질 규칙은 유지한다.

## 설계 경계

현재 `draw_back`은 bay/slit을 카트보다 먼저 그리고 `draw_front`는 양옆 band와 top lip을 카트보다
나중에 그린다. 이 순서와 `MOUTH_H`, `MOUTH_EXTRA`, `LIP_H`, `SLIT_H` 의미를 유지한다. port SVG는
장식용 front trim이며 다음 조건을 지킨다.

- port texture는 카트보다 나중, 즉 `draw_front` 단계에서 그린다.
- 중앙 opening은 투명해 seated cart가 계속 보인다.
- 기존 procedural back/slit/band/lip을 제거하거나 port bitmap 한 장으로 대체하지 않는다.
- port는 `ArtCache`를 통해 최초 1회 raster/upload하며 매 frame, scroll, insert frame마다 다시 올리지 않는다.
- port 위치는 panel 중앙, `band_y`이며 높이는 정확히 `MOUTH_H`다. 화면 지오메트리별 임의 보정 상수나
  플랫폼 match를 `ShelfView`에 넣지 않는다. 차이는 `PlatformSkin` 데이터가 가진다.

## `PlatformSkin` 계약

`crates/slot2-ui/src/skin.rs`의 `PlatformSkin`에 다음 public field를 추가한다.

```rust
pub port: &'static str,
pub port_size: (f32, f32),
```

- `port`는 투명 배경의 흰 coverage SVG source다.
- `port_size`는 SVG viewBox의 `(width, height)`이며 높이는 모든 플랫폼에서 `58.0 == MOUTH_H`다.
- 모든 7개 static row에 자기 port source와 size를 직접 지정한다. fallback이나 공유 source는 없다.
- 기존 cart/detail/label/shell/borrowed 값은 바꾸지 않는다.
- 모듈 문서의 “port drawing이 없다”는 설명을 현재 계약으로 갱신한다. `socket.svg`는 core picker IC
  socket이며 이 포트들과 무관하다는 구분은 유지한다.

## 자산 형식과 크기

다음 7개 파일을 새로 만든다.

| platform | file | viewBox | 중앙 opening x 범위 |
|---|---|---:|---:|
| GB | `assets/skins/gb_port.svg` | `0 0 294 58` | `20..274` |
| GBC | `assets/skins/gbc_port.svg` | `0 0 294 58` | `20..274` |
| GBA | `assets/skins/gba_port.svg` | `0 0 294 58` | `20..274` |
| NES | `assets/skins/nes_port.svg` | `0 0 264 58` | `20..244` |
| SNES | `assets/skins/snes_port.svg` | `0 0 294 58` | `20..274` |
| MD | `assets/skins/md_port.svg` | `0 0 304 58` | `20..284` |
| SMS | `assets/skins/sms_port.svg` | `0 0 254 58` | `20..234` |

각 width는 `cart_size.0 + MOUTH_EXTRA + 40`이다. 가운데 `cart_size.0 + MOUTH_EXTRA`가 기존 mouth이고,
양옆 20px은 trim이 band와 만나는 영역이다.

외부 파일·웹·사진·제조사 도면·기존 게임기 벡터를 가져오거나 tracing하지 않는다. 저장소 안에서 단순
기하 도형으로 직접 작성한다. 상표·로고·문자·제품명은 넣지 않는다. SVG에는 bitmap, font, script,
filter, external reference, generator metadata를 넣지 않는다.

- 투명 배경 위 `#fff` coverage만 쓴다. `ShelfView`가 기존 `LIP` 색으로 tint한다.
- 정적 `path`, `rect`, `circle` 같은 기본 벡터 도형만 사용한다.
- 모든 port는 nonblank이고 canvas 전체를 채우지 않는다.
- 중앙 `(width / 2, MOUTH_H * 0.75)` 주변은 투명해야 한다. 최소한 중앙 기준 좌우 8px,
  y=`MOUTH_H * 0.65 .. 0.85` 사각 영역 전체에 coverage가 없어야 seated cart가 보인다.
- opening 바깥 양옆 trim에는 coverage가 있어야 한다.
- top/bottom rail을 중앙을 가로질러 그릴 수 있지만 opening의 세로 중앙을 막아서는 안 된다.

7개 source는 모두 달라야 하며 선반 거리에서도 구분되는 간결한 특징을 준다. 예:

- GB: 각진 side block과 작은 원형 fastener
- GBC: 둥근 side cap과 짧은 이중 세로 rib
- GBA: 낮은 wing형 side trim과 얇은 top rail
- NES: 계단형 side block과 긴 직선 top rail
- SNES: 둥근 shoulder trim과 분리된 bottom corner rail
- MD: 넓은 chamfer side trim과 중앙이 끊긴 top rail
- SMS: 좁은 stepped side trim과 짧은 bottom rail

이는 방향일 뿐 제조사 형상을 복제하라는 뜻이 아니다. 도형 padding으로 source만 다르게 만드는 것은
금지하며, coverage mask 자체가 pairwise distinct해야 한다.

## `ShelfView` 연결

- cart shell/detail과 함께 또는 별도 helper에서 port를 natural `port_size`로 cache한다.
- port draw rect는 `x = (panel_w - port_size.0) / 2`, `y = band_y`, `w = port_size.0`,
  `h = MOUTH_H`다. 필요한 경우 x만 기존처럼 pixel round한다.
- `draw`와 `draw_insert`가 동일한 경로로 같은 port를 그린다.
- port image는 procedural front band/lip 및 cart와의 기존 occlusion 의미를 보존하는 순서에 둔다.
  seated cart의 중앙 opening이 가려지지 않아야 한다.
- tint는 기존 `LIP`을 사용한다. port 전용 색 field나 플랫폼별 색 match를 추가하지 않는다.
- 빈 선반도 해당 플랫폼 port를 그린다.
- 한 `ShelfView`에서 플랫폼을 바꾸면 해당 port source로 바뀐다.

## provenance와 문서

`assets/skins/PROVENANCE.md`에 7개 port가 2026년 SLOT2에서 직접 그린 original geometric redraw이며
외부 자산·사진·도면을 복제하거나 tracing하지 않았음을 기록한다. 파일별 viewBox와 특징, 흰 coverage와
runtime tint, 중앙 opening이 투명한 front trim이라는 역할을 적는다. 기존 cart provenance는 보존한다.

`docs/DESIGN.md` §7 스킨 문장에 7개 플랫폼이 독립 cart shell/detail뿐 아니라 독립 port trim을 가진다고
반영한다. insert/eject curve와 sfx가 아직 없다는 구조적 사실은 완료된 것처럼 쓰지 않는다.

`docs/MILESTONES.md` M3 스킨 한 줄에 7개 독립 port를 반영한다. M6 복합 항목은 curve/sfx가 남았으므로
`[ ]` 그대로 두고 port 완료(Task97), curve/sfx 남음으로 진행 메모를 갱신한다.

## 자동 계약 테스트

기존 테스트를 삭제하거나 약화하지 않고 다음을 검증한다. 이름이나 주석의 전제가 바뀌면 현재 사실에
맞게 고친다.

### `tests/skin.rs`

1. 7개 `port` source가 nonempty이고 모든 쌍에서 다르다.
2. 각 port SVG viewBox가 table `port_size`와 일치한다.
3. 각 `port_size.0 == cart_size.0 + MOUTH_EXTRA + 40.0`, `port_size.1 == MOUTH_H`다.
4. 7개 port를 natural size와 64×58 fit box로 rasterize할 수 있으며 nonblank다.
5. 7개 port mask가 모든 쌍에서 다르다. source 문자열만 다른 동일 그림은 실패해야 한다.
6. 각 port의 중앙 투명 사각 영역은 coverage 0이고 opening 바깥 양옆에는 각각 coverage가 있다.

### `tests/shelf_draw.rs`

7. 빈 선반에서도 port `Image`가 band 위치에 panel 중앙 정렬로 그려지고 panel 안에 있다.
8. 같은 canvas와 `ShelfView`에서 두 플랫폼의 빈 선반을 차례로 그리면 서로 다른 port texture를 쓴다.
   cart/title이 없는 상태로 확인해 port 차이만 검증한다.
9. 동일 플랫폼을 여러 frame 다시 그리거나 scroll해도 port upload가 반복되지 않는다. 기존 cache 계약을
   깨지 않는다.

### `tests/insert.rs`

10. `seat == 1.0`에서 port image가 travelling cart 뒤가 아니라 뒤쪽 procedural bay 뒤가 아닌 front
    순서, 즉 cart image보다 나중에 그려진다.
11. port mask 중앙 opening 계약과 기존 rect occlusion 계약을 함께 사용해 seated cart가 중앙에서
    계속 보인다는 기존 의미를 유지한다. RecordingCanvas가 texture pixel을 직접 합성하지 못한다면
    skin mask 테스트와 draw order 테스트를 조합하며 가짜 pixel 합성기를 만들지 않는다.
12. 세 지오메트리와 7개 플랫폼의 `draw`/대표 insert frame에서 port rect가 finite·positive이고 panel
    가로 범위를 벗어나지 않는다.

고정 TexId 숫자, 전체 op 수, SVG path 문자열, exact alpha 합계나 특정 anti-alias 경계 pixel에 맞춘
취약한 단언을 쓰지 않는다.

## 수정 허용 파일

- `C:\SLOT2\assets\skins\gb_port.svg` (신규)
- `C:\SLOT2\assets\skins\gbc_port.svg` (신규)
- `C:\SLOT2\assets\skins\gba_port.svg` (신규)
- `C:\SLOT2\assets\skins\nes_port.svg` (신규)
- `C:\SLOT2\assets\skins\snes_port.svg` (신규)
- `C:\SLOT2\assets\skins\md_port.svg` (신규)
- `C:\SLOT2\assets\skins\sms_port.svg` (신규)
- `C:\SLOT2\assets\skins\PROVENANCE.md`
- `C:\SLOT2\crates\slot2-ui\src\skin.rs`
- `C:\SLOT2\crates\slot2-ui\src\shelf_view.rs`
- `C:\SLOT2\crates\slot2-ui\tests\skin.rs`
- `C:\SLOT2\crates\slot2-ui\tests\shelf_draw.rs`
- `C:\SLOT2\crates\slot2-ui\tests\insert.rs`
- `C:\SLOT2\docs\DESIGN.md` — §7 스킨 문장만
- `C:\SLOT2\docs\MILESTONES.md` — M3/M6 스킨 두 줄만
- `C:\SLOT2\tasks\97-platform-port-skins.worker-result.md`

다른 production, test, 문서, 설정 파일은 수정하지 않는다. 계약 자체가 틀렸거나 허용 파일 밖 수정이
필요하면 추측 구현하지 말고 실패 보고서에 정확한 이유를 쓴다.

## 범위 밖 및 금지

- procedural back/slit/band/lip 제거 또는 occlusion/seat/travel 수학 변경
- insert/eject curve, timing, state machine 변경
- 효과음 자산·재생, wallpaper, cart/label renderer 변경
- app/store/session/core/i18n/font 변경
- 테스트만 통과시키는 padding draw, 의미 없는 차이용 도형, 고정 TexId 단언
- 위임, 커밋, push, 네트워크, 실기·ADB·Samba·SD 카드 접근
- `~/.gjc-bai/agent/*.yml` 등 공용 설정 변경

## 완료 기준

마지막 code/test/asset 변경 뒤 아래 원문을 모두 실행한다. 테스트는 작업자인 가재코드가 수행한다.
Codex가 다시 실행할 필요가 없도록 종료 코드와 마지막 결과 줄을 보고서에 남긴다.

```text
cargo fmt --all -- --check
cargo test -p slot2-ui --test skin
cargo test -p slot2-ui --test shelf_draw
cargo test -p slot2-ui --test insert
cargo test -p slot2-ui
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2-ui -p slot2 --all-targets -- -D warnings
```

- 모든 명령 종료 코드 0.
- 모든 실행된 test binary에서 failed 0, 예상 밖 ignored 0.
- clippy warning 0.
- 완료 기준 뒤 code/test/asset 변경 금지. 변경했으면 영향받는 명령부터 다시 실행한다.

첫 응답은 최대 5분, 전체는 최대 45분 기다린다. 진행 확인이 필요하면 로그 크기가 아니라
`git status --short`를 사용한다.

## 결과 보고서

`C:\SLOT2\tasks\97-platform-port-skins.worker-result.md`에 다음을 기록한다.

- 성공/실패와 누적 호출 수
- 7개 SVG의 viewBox, 특징, opening/coverage 규칙
- `PlatformSkin` port table과 `ShelfView` cache/draw order 요약
- 빈 선반·플랫폼 전환·insert occlusion 계약 결과
- provenance와 DESIGN/MILESTONES 변경 요약
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored 수
- 생성·수정 파일과 최종 검증 뒤 code/test/asset 변경 여부
- 남은 curve/sfx와 실기 확인 항목
- 계약이 틀려 보인 부분이 있으면 해당 줄과 이유

실패해도 보고서를 남긴다. 두 번째 호출까지 실패하면 더 시도하지 않는다. 커밋하지 않는다.
