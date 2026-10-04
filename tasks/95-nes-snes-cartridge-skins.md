# Task 95 — NES·SNES 카트리지 스킨

현재 checkout에서 직접 작업한다. GBA 스킨을 빌리던 NES와 SNES 선반에 각 플랫폼의 비율과 몰딩을
가진 독립 카트리지 SVG를 만들고 `PlatformSkin` 테이블과 자동 계약에 연결한다.

플랫폼별 포트는 현재 `PlatformSkin`에 필드가 없고 `ShelfView`가 공용 슬롯 입구를 코드로 그린다.
이번 태스크는 카트리지 두 종만 완결한다. 포트 구조, MD·SMS 자산과 효과음은 후속으로 남긴다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 횟수에 포함한다.

## 먼저 읽을 범위

필요한 부분만 읽는다.

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\94-m5-host-contract-closure.result.md`
- `C:\SLOT2\docs\DECISIONS.md`의 D-07~D-09만
- `C:\SLOT2\docs\DESIGN.md`의 §7 `PlatformSkin` 부분만
- `C:\SLOT2\docs\MILESTONES.md`의 M3 스킨 한 줄과 M6만
- `C:\SLOT2\assets\skins\PROVENANCE.md`
- `C:\SLOT2\assets\skins\cart.svg`, `cart_detail.svg`, `gb_cart.svg`,
  `gb_cart_detail.svg` — silhouette/detail 형식 참고만
- `C:\SLOT2\crates\slot2-ui\src\skin.rs`
- `C:\SLOT2\crates\slot2-ui\src\svg.rs`의 coverage 계약만
- `C:\SLOT2\crates\slot2-ui\tests\skin.rs`
- `C:\SLOT2\crates\slot2-ui\src\shelf_view.rs`의 `prepare_textures`와 `draw_cart`만

App/store/core/i18n/font, 다른 UI 화면, 워커 로그와 저장소 이력은 읽지 않는다. 집중 테스트가 직접
깨질 때만 해당 오류 주변을 최소한으로 추가 확인한다.

## 자산 형식

새 파일 네 개를 직접 작성한다.

- `assets/skins/nes_cart.svg`
- `assets/skins/nes_cart_detail.svg`
- `assets/skins/snes_cart.svg`
- `assets/skins/snes_cart_detail.svg`

공통 규칙:

- shell과 detail은 같은 platform 안에서 정확히 같은 `viewBox`를 쓴다.
- 투명 배경 위 흰색 coverage만 사용한다. 색은 `PlatformSkin::shell` tint가 정한다.
- path/rect/rounded rect 같은 정적 vector만 사용한다. text, logo, 상표, embedded bitmap, font,
  script, filter, external reference를 넣지 않는다.
- shell SVG는 하나의 알아볼 수 있는 외곽 실루엣이다. detail SVG는 몰딩·그립·홈만 담고, shell 전체를
  다시 채우지 않는다.
- detail은 별도 mask로 어둡게 tint되므로 alpha가 곧 형태다. 검정/흰색 명도 차로 light/shadow 두
  channel을 표현한다고 가정하지 않는다.
- 과도한 path point나 generator metadata를 넣지 않는다. 사람이 읽고 수정할 수 있는 간결한 SVG다.
- 인터넷 자산이나 제조사 도면을 복사하지 않는다. 보편적인 물리적 비율을 바탕으로 SLOT2용으로 직접
  만든 기하학적 재해석이며, trademark text와 logo는 없다.

### NES

- `viewBox="0 0 210 270"`.
- 세로형 직사각 shell, 짧은 윗어깨와 아래 connector 쪽의 작은 bevel로 NES 카트임을 구분한다.
- label rect: `x=20, y=28, w=170, h=142`. shell 안쪽이며 아래 connector/grip 영역을 비운다.
- detail은 label rect를 가로지르지 않는 좌우 세로 groove와 아래쪽 grip/connector 몰딩을 사용한다.
- shell: solid neutral charcoal-gray `[0x68, 0x69, 0x6e]`.

### SNES

- `viewBox="0 0 240 190"`.
- NES보다 넓고 낮으며 둥근 윗어깨와 아래 connector 쪽으로 좁아지는 형상으로 구분한다.
- label rect: `x=25, y=34, w=190, h=92`. shell 안쪽이며 아래 몰딩을 비운다.
- detail은 label rect 밖의 어깨 groove, 양쪽 grip line, 아래 connector 몰딩을 사용한다.
- shell: solid light warm-gray `[0x96, 0x94, 0x9b]`.

SVG에서 label 자체나 글자를 그리지 않는다. `ShelfView`가 위 rect에 카드 label/printed title을 별도로
그린다.

## `PlatformSkin` 연결

`crates/slot2-ui/src/skin.rs`에서:

- 새 SVG 네 개를 `include_str!`로 넣고 위 size, label, shell 상수를 선언한다.
- `SKIN_NES`, `SKIN_SNES`를 독립 `PlatformSkin`으로 바꾸고 `borrowed: false`로 둔다.
- NES/SNES의 `cart`와 `cart_detail`은 각자 새 자산이며 서로 또는 GBA/GB/GBC와 source가 같아서는
  안 된다.
- MD와 SMS만 기존 GBA fallback과 `borrowed: true`를 유지한다.
- `borrowing` helper는 두 fallback에 계속 사용한다. GBA/GB/GBC 계약은 바꾸지 않는다.
- `PlatformSkin` shape, public API와 `ShelfView` drawing/caching logic은 바꾸지 않는다.

module 문서를 “다섯 플랫폼은 독립 artwork, MD/SMS 두 플랫폼만 borrowed”라는 현재 사실로 고친다.

## provenance와 설계 문서

`assets/skins/PROVENANCE.md`에서 기존 원본 이식 파일의 MIT provenance는 그대로 유지한다. 새 네 파일은
별도 구역에 다음 사실을 기록한다.

- 2026 SLOT2용으로 저장소 안에서 직접 작성한 original geometric redraw
- 제조사 원본 vector/사진 tracing이나 외부 asset 복사 없음
- logo/text/trademark 없음
- 각 viewBox와 역할(shell/detail)
- 흰 coverage mask이며 runtime tint를 받는다는 점

`docs/DESIGN.md` §7의 fallback 문장을 NES/SNES 독립, MD/SMS fallback으로 바꾼다. 구조 예시의 아직 없는
port/curve/sfx 필드는 이번 구현으로 완성됐다고 쓰지 않는다.

`docs/MILESTONES.md` M3 스킨 한 줄도 같은 현재 상태로 고친다. M6의 NES/SNES/MD/SMS 카트·포트·효과음
항목은 MD/SMS와 port/sfx가 남았으므로 `[ ]` 그대로 두고, NES/SNES 카트만 완료됐다는 짧은 진행 메모를
붙인다.

## 자동 계약 테스트

`crates/slot2-ui/tests/skin.rs`의 기존 계약을 현재 상태에 맞게 강화한다.

1. GB/GBC/GBA/NES/SNES는 `borrowed == false`, MD/SMS만 `true`이고 GBA cart/size를 빌린다.
2. NES/SNES의 cart와 detail source는 비어 있지 않고 서로 및 기존 세 플랫폼 source와 다르다.
3. 모든 독립 shell/detail의 `viewBox`가 table `cart_size`와 정확히 같다.
4. NES/SNES label rect가 위 명세값이고 shell 안에 완전히 들어가며 전체 면적의 90% 미만이다.
5. NES/SNES shell/detail 각각을 natural size와 작은 preview size로 rasterize할 수 있다.
6. 각 shell mask에는 충분한 불투명 pixel과 충분한 투명 pixel이 모두 있다. 완전 blank나 canvas 전체
   rectangle을 거부한다.
7. 각 detail mask는 nonblank지만 같은 크기의 shell보다 coverage가 작고 shell mask와 같지 않다.
8. NES와 SNES의 natural aspect ratio가 서로 다르고, rasterize-fit이 그 비율을 유지한다.

고정 texture id, SVG path 문자열 전체, exact alpha 합계, anti-aliasing 경계 pixel 하나에 의존하지 않는다.
의미 있는 넓은 비율/범위로 검증한다. 기존 테스트를 삭제하거나 약화하지 않는다.

## 수정 허용 파일

- `C:\SLOT2\assets\skins\nes_cart.svg` (신규)
- `C:\SLOT2\assets\skins\nes_cart_detail.svg` (신규)
- `C:\SLOT2\assets\skins\snes_cart.svg` (신규)
- `C:\SLOT2\assets\skins\snes_cart_detail.svg` (신규)
- `C:\SLOT2\assets\skins\PROVENANCE.md`
- `C:\SLOT2\crates\slot2-ui\src\skin.rs`
- `C:\SLOT2\crates\slot2-ui\tests\skin.rs`
- `C:\SLOT2\docs\DESIGN.md` — §7 fallback 한 문장만
- `C:\SLOT2\docs\MILESTONES.md` — M3/M6 skin 두 줄만
- `C:\SLOT2\tasks\95-nes-snes-cartridge-skins.worker-result.md`

그 밖의 production·test·문서 파일은 수정하지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지
않는다.

## 범위 밖 및 금지

- 플랫폼별 port SVG/`PlatformSkin` port API, 공용 slot mouth 변경
- MD·SMS 카트리지 자산
- insert/eject curve, 효과음, wallpaper, label renderer 변경
- App/store/core/i18n/font 변경
- 외부 이미지·도면·SVG 다운로드, network 사용
- 제조사 logo/text/trademark 추가
- 전체 workspace 테스트, 실제 GL 창, device 배포
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 code/asset 변경 뒤 아래를 순서대로 실행한다.

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

- 모두 종료 코드 0이어야 한다.
- 각 test command의 마지막 result 줄에서 passed/failed/ignored 수를 보고한다.
- SVG 네 개가 실제 table에서 참조되고 rasterize되어야 한다. 존재만 하는 미사용 asset은 실패다.
- 완료 기준 뒤 code/test/asset 변경 금지. 변경했으면 영향받는 명령부터 다시 실행한다.
- 문서만 마지막에 바꿨다면 코드 재실행은 필요 없지만 그 사실을 보고한다.

## 결과 보고서

`C:\SLOT2\tasks\95-nes-snes-cartridge-skins.worker-result.md`에 다음만 간결하게 쓴다.

- 성공/실패와 진짜 누적 호출 횟수(최대 2)
- 각 SVG의 viewBox, 형상 특징, label rect와 shell colour
- original authorship/provenance 기록 내용
- NES/SNES 독립 및 MD/SMS fallback table 결과
- raster/coverage/aspect/label 자동 계약 결과
- DESIGN/MILESTONES 진행 표기
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored 수
- 생성·수정 파일과 최종 검증 뒤 code/test/asset 변경 여부
- port/MD/SMS/sfx가 후속이라는 점
- 계약이 틀려 보이는 부분이 있으면 그 내용
- 소요 시간

코드·로그 전문이나 이 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다.
실패해도 보고서를 남기며 두 번째 호출까지 실패하면 더 시도하지 않는다.
