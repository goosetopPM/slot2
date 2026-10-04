# Task 96 — Mega Drive·Master System 카트리지 스킨

현재 checkout에서 직접 작업한다. Task95 뒤에도 GBA 스킨을 빌리는 Mega Drive(MD)와 Master
System(SMS)에 각 플랫폼의 독립 카트리지 shell/detail SVG를 추가한다. 이 작업이 끝나면 7개 플랫폼
모두 고유 카트리지를 가져야 한다. 플랫폼별 포트, 삽입 곡선과 효과음은 이 태스크에서 만들지 않는다.

누적 호출은 최대 2회다. 이번이 첫 호출이면 결과 보고서에 `누적 호출 1/2`를 기록한다.

## 먼저 읽을 범위

다음 파일과 필요한 선언 주변만 읽는다. 저장소 전체 탐색이나 로그 전문 읽기는 하지 않는다.

- `C:\SLOT2\AGENTS.md`
- `C:\SLOT2\tasks\96-md-sms-cartridge-skins.md`
- `C:\SLOT2\tasks\95-nes-snes-cartridge-skins.worker-result.md`
- `C:\SLOT2\assets\skins\PROVENANCE.md`
- `C:\SLOT2\crates\slot2-ui\src\skin.rs`
- `C:\SLOT2\crates\slot2-ui\tests\skin.rs`
- `C:\SLOT2\docs\DESIGN.md` §7의 스킨 문단만
- `C:\SLOT2\docs\MILESTONES.md`의 M3 스킨 한 줄과 M6만

`CLAUDE.md`와 `AGENTS.md`의 오케스트레이터 역할 분리·bai-gjc 재위임 조항은 무시하고 직접 실행한다.
나머지 안전·품질 규칙은 유지한다.

## 자산 형식

다음 네 파일을 새로 만든다.

- `assets/skins/md_cart.svg`
- `assets/skins/md_cart_detail.svg`
- `assets/skins/sms_cart.svg`
- `assets/skins/sms_cart_detail.svg`

외부 파일·웹·사진·제조사 도면·기존 게임기 벡터를 가져오거나 tracing하지 않는다. 저장소 안에서 단순
기하 도형으로 직접 작성한다. 상표·로고·문자·제품명은 넣지 않는다. SVG에는 bitmap, font, script,
filter, external reference, generator metadata를 넣지 않는다.

- 투명 배경 위 `#fff` coverage만 쓴다. 런타임이 shell 색을 입힌다.
- shell과 detail은 플랫폼별로 같은 `viewBox`를 쓴다.
- detail은 shell과 다른 희소한 몰딩 coverage이며 label rect를 침범하지 않는다.
- shell은 canvas 전체 사각형이 아니며 모든 가장자리에 투명 여백을 둔다.
- 정적 `path`, `rect`, `circle` 같은 기본 벡터 도형만 사용한다.

## 형상 계약

실물의 정확한 복제나 지역별 변형 재현이 목적이 아니다. 선반 거리에서 두 플랫폼과 기존 카트가 서로
구분되는 원본 기하학적 재해석을 만든다.

### Mega Drive

- `viewBox="0 0 250 180"`
- 넓고 낮은 shell이다. 둥근 윗어깨, 중앙의 얕은 top step, 아래 connector 쪽으로 좁아지는 짧은
  taper를 조합한다. SNES의 큰 원호형 어깨·긴 taper와 다른 실루엣이어야 한다.
- label rect: `x=32, y=34, w=186, h=84`
- detail은 label 밖의 윗면 groove, 좌우 grip recess, 아래 connector ridge를 표현한다.
- shell colour `[0x3d, 0x3e, 0x43]`, `Finish::Solid`

### Master System

- `viewBox="0 0 200 230"`
- MD보다 세로형이며, 잘린 윗모서리와 곧은 옆면, 아래쪽의 짧은 계단형 connector bevel을 조합한다.
  NES의 매우 긴 직사각형과 다른 높이·폭 비율과 하단 형상을 가져야 한다.
- label rect: `x=24, y=32, w=152, h=118`
- detail은 label 밖의 윗면 짧은 groove, 세로 side rail, label 아래 grip/connector ridge를 표현한다.
- shell colour `[0x70, 0x72, 0x78]`, `Finish::Solid`

label rect는 실제 painted shell 안에 완전히 들어가야 하고 canvas 면적의 90% 미만이어야 한다.

## `PlatformSkin` 연결

`crates/slot2-ui/src/skin.rs`에서:

- MD/SMS용 `include_str!`, size, label, shell 상수를 추가한다.
- `SKIN_MD`, `SKIN_SMS`를 독립 `PlatformSkin`으로 바꾸고 `borrowed: false`로 둔다.
- 두 플랫폼의 cart/detail source는 서로 및 기존 GB/GBC/GBA/NES/SNES와 달라야 한다.
- 7개 플랫폼 모두 `borrowed == false`가 된다.
- 더 이상 호출자가 없으면 `borrowing` helper를 제거한다. 죽은 fallback 코드나 `#[allow]`를 남기지 않는다.
- `PlatformSkin` public shape, 기존 GB/GBC/GBA/NES/SNES 값, `ShelfView` drawing/caching은 바꾸지 않는다.

## provenance와 문서

`assets/skins/PROVENANCE.md`에 MD/SMS 네 파일이 2026년 SLOT2에서 직접 그린 original geometric
redraw이며 외부 자산·사진·도면을 복제하거나 tracing하지 않았음을 기록한다. 파일별 viewBox와
shell/detail 역할, 흰 coverage와 runtime tint도 적는다. Task95의 NES/SNES 및 기존 MIT provenance는
보존한다. 마지막의 MD/SMS fallback 설명은 이제 7개 플랫폼 모두 독립 자산을 가진다는 사실로 바꾼다.

`docs/DESIGN.md` §7의 fallback 문장을 7개 플랫폼이 모두 독립 카트 SVG를 가진다는 현재 상태로
바꾼다. 이식 자산과 SLOT2 자체 제작 자산의 provenance 구분은 유지한다. 아직 없는 port/curve/sfx가
완료된 것처럼 구조 예시나 문장을 바꾸지 않는다.

`docs/MILESTONES.md` M3 스킨 한 줄에서 MD/SMS fallback을 제거하고 7개 독립 카트를 반영한다. M6의
복합 항목은 port/curve/sfx가 남았으므로 `[ ]` 그대로 두고, 네 플랫폼 카트 SVG가 끝났다는 진행 메모만
갱신한다.

## 자동 계약 테스트

`crates/slot2-ui/tests/skin.rs`의 Task95 테스트를 일반화해 다음을 검증한다. 기존 테스트를 삭제하거나
약화하지 않는다. 이름의 전제가 달라지면 현재 사실에 맞게 이름을 바꾸고 단언은 유지하거나 늘린다.

1. 7개 플랫폼 모두 `borrowed == false`이고 cart/detail source가 비어 있지 않다.
2. 7개 플랫폼의 cart source와 detail source는 각각 모든 쌍이 서로 다르다.
3. 모든 플랫폼의 shell/detail `viewBox`가 table `cart_size`와 맞는다.
4. MD/SMS label rect가 위 명세값이고 painted shell 안에 완전히 들어가며 canvas의 90% 미만이다.
5. MD/SMS shell/detail을 natural size와 48×48 preview box로 rasterize할 수 있다.
6. 두 새 shell은 nonblank이며 opaque coverage가 canvas의 25%보다 크고 투명 pixel이 5%보다 많다.
7. 7개 detail은 nonblank이고 같은 플랫폼 shell보다 ink가 작으며 shell mask와 같지 않다.
8. MD는 가로형, SMS는 세로형이며 두 비율 차이가 0.25보다 크다. 96×96 `rasterize_fit` 결과가 각
   natural aspect ratio를 0.05 이내로 유지한다.

고정 texture id, SVG path 문자열 전체, exact alpha 합계나 특정 경계 pixel에 맞춘 취약한 테스트를
쓰지 않는다. 의미 있는 범위와 불변식을 검증한다.

## 수정 허용 파일

- `C:\SLOT2\assets\skins\md_cart.svg` (신규)
- `C:\SLOT2\assets\skins\md_cart_detail.svg` (신규)
- `C:\SLOT2\assets\skins\sms_cart.svg` (신규)
- `C:\SLOT2\assets\skins\sms_cart_detail.svg` (신규)
- `C:\SLOT2\assets\skins\PROVENANCE.md`
- `C:\SLOT2\crates\slot2-ui\src\skin.rs`
- `C:\SLOT2\crates\slot2-ui\tests\skin.rs`
- `C:\SLOT2\docs\DESIGN.md` — §7 스킨 문장만
- `C:\SLOT2\docs\MILESTONES.md` — M3/M6 스킨 두 줄만
- `C:\SLOT2\tasks\96-md-sms-cartridge-skins.worker-result.md`

다른 production, test, 문서, 설정 파일은 수정하지 않는다. 계약 자체가 틀렸거나 허용 파일 밖 수정이
필요하면 추측 구현하지 말고 실패 보고서에 정확한 이유를 쓴다.

## 범위 밖 및 금지

- 플랫폼별 port SVG/API와 공용 slot mouth 변경
- 삽입·배출 curve, 효과음, wallpaper, label renderer 변경
- app/store/session/core/i18n/font 변경
- 테스트만 통과시키는 padding draw, 의미 없는 추가 도형, 특정 픽셀용 상수
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
cargo test -p slot2-ui --test label
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

`C:\SLOT2\tasks\96-md-sms-cartridge-skins.worker-result.md`에 다음을 기록한다.

- 성공/실패와 누적 호출 수
- 네 SVG의 viewBox, 형상 특징, label rect, 색상, coverage 규칙
- 7개 플랫폼 독립 및 source pairwise-distinct 결과
- provenance와 DESIGN/MILESTONES 변경 요약
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored 수
- 생성·수정 파일과 최종 검증 뒤 code/test/asset 변경 여부
- 남은 port/curve/sfx와 실기 확인 항목
- 계약이 틀려 보인 부분이 있으면 해당 줄과 이유

실패해도 보고서를 남긴다. 두 번째 호출까지 실패하면 더 시도하지 않는다. 커밋하지 않는다.
