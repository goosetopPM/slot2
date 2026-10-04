# Task 79 작업자 보고서

## 결과

성공. 누적 호출 1/2 (실행 호출 1회).

## Asset

- 경로: `assets/overlays/GB/720x720.png` (신규, repository asset)
- SHA-256: `F370D0270AD2CC1C503D12ABEEF5BD67EE0711B68864985468B502DAFED0DFE2`
- byte size: 4464
- PNG 구조: 720×720, bit depth 8, color type 6(RGBA8, straight alpha), no interlace, filter 0 고정.
  chunk는 IHDR(13)/IDAT(4407)/IEND(0)뿐이며 tEXt·tIME·툴 이름·경로가 없다.
- geometry/aperture: aperture는 x=40..679, y=72..647의 640×576이고 그 안 모든 pixel이 정확히
  `(0,0,0,0)`이다. `slot2_gfx::place(Integer, (160,144), (1,1), (720,720))`이 `(40,72,640,576)`을
  돌려주며, 투명 pixel의 bounding box도 같은 값이다.
- palette/alpha: aperture 밖 149,760 pixel 전부 alpha 255이고 실제로 등장하는 색은 네 가지다 —
  graphite base `(30,31,35)` 124,864 px, olive inner rim `(92,96,62)` 14,736 px, olive hairline
  `(60,64,42)` 6,320 px, plum accent `(76,53,70)` 3,840 px. aperture 경계 바로 바깥 pixel은 base와 다른
  rim 색이다. text·logo·상표·캐릭터·외부 artwork·noise·고채도색 없음.

## Generator

- `build/generate-overlays.py` 신규. `struct`, `zlib`, `pathlib`만 사용한다(Pillow·ImageMagick·font·
  network 없음). `Path(__file__).resolve().parent.parent`로 repository root를 계산하므로 어디서 실행해도
  같은 파일을 만든다.
- 결정성: 연속 2회 실행 SHA-256 동일(`F370D0…0DFE2`), 두 번째 실행 뒤 파일 bytes 불변. 시각·난수·
  환경 정보를 쓰지 않고, scanline filter를 0으로 고정하고 deflate level도 상수(`ZLIB_LEVEL = 9`)다.
  palette/두께는 script 상단 이름 붙은 상수(`GRAPHITE`, `OLIVE`, `OLIVE_HAIRLINE`, `PLUM`, `CLEAR`,
  `RIM_W`, `RIM_GAP_W`, `EDGE_HAIRLINE_W`, `ACCENT_W`, `ACCENT_HAIRLINE_W`, `ACCENT_FROM_EDGE`,
  `ACCENT_GAP`)로 두었고 주석은 디자인 의도만 적었다.
- `assets/overlays/README.md` 신규: generated source, 대상 pair, 640×576 aperture와 origin, 기본 Integer
  전용, 카드 override 경로와 우선순위, 제3자 asset을 쓰지 않았다는 사실, 재생성 명령을 적었다.

## Production 등록

- `BUILT_IN_OVERLAYS`는 정확히 한 entry다: `Platform::Gb` × `Geometry::W720H720`,
  id `gb-cubexx-frame-v1`, `include_bytes!("../../../assets/overlays/GB/720x720.png")`.
  table 길이 1, pair/id 중복 없음, 나머지 20 pair(7 platform × 3 geometry = 21)에는 built-in이 없다.
  GBC는 GB의 그림을 공유하지 않는다.
- 낡은 “비어 있음/후속 artwork” 설명을 모듈 doc·table doc·`resolve_with` doc에서 현재 사실로 고쳤다.
- `overlay_enabled`는 그대로다: `None` false, `Some(false)` false, `Some(true)` true. sample 등록이
  platform default를 켜지 않는다.
- card precedence와 fallback: 유효한 card PNG가 built-in보다 먼저 쓰이고, truncated·wrong-panel·
  not-a-picture card file은 production built-in으로 fallback하며 `OverlaySourceKind::BuiltIn`과 card
  failure를 함께 남긴다. production built-in만 있는 layer는 panel-size texture를 1회 upload하고
  frame마다 full-panel plain image를 그리며 재upload/free하지 않는다.
- decoder·texture/cache/lifecycle·draw order는 변경하지 않았다.

## 테스트 (`crates/slot2/tests/overlay_layer.rs`)

기존 19 passed → 신규 8개 추가로 27 passed. 추가한 검증은 위 항목들(표 길이/pair/id, decode 크기,
aperture 전수와 bounding box, 상단·하단/좌우 rim 색과 alpha, 20 pair 부재, card precedence·fallback,
built-in layer draw/upload 횟수, default-off)과 generator 결정성(SHA-256 두 번 + tracked bytes 불변)이다.
SHA-256은 이 파일에 직접 구현했고(기존 CRC-32와 같은 이유) Python hashlib 값과 일치함을 확인했다.
binary PNG compressed bytes는 테스트 상수로 복제하지 않았고, texture id용 padding upload/draw도 없다.
기존 `nothing_registered_and_nothing_on_the_card_resolves_to_no_sources`는 production table이 더 이상
비어 있지 않아 등록되지 않은 pair(GBA×640×480)를 쓰는 이름/내용으로 바꿨다(단언 약화 없음).

## 완료 기준 명령

| 명령 | 종료 코드 | 마지막 결과 |
|---|---|---|
| `python build/generate-overlays.py` ×2 + SHA-256 비교 | 0 | `F370D0…0DFE2` 두 번 동일, throw 없음 |
| `cargo fmt --all -- --check` | 0 | 출력 없음 |
| `cargo test -p slot2 --test overlay_layer --test overlay_app --test overlay_menu_app` | 0 | 27 / 16 / 17 passed, 합계 60 passed / 0 failed / 0 ignored |
| `cargo test -p slot2` | 0 | 결과 줄 26개, 312 passed / 0 failed / 0 ignored |
| `cargo check -p slot2 --no-default-features --features device` | 0 | `Finished \`dev\` profile ... in 3.65s` |
| `cargo clippy -p slot2 --all-targets -- -D warnings` | 0 | `Finished \`dev\` profile ... in 16.81s` |
| `powershell -File build/dist-device.ps1` | 0 | 마지막 줄 `==> done` (device cross build 7m 01s) |

core-dependent skip: 0. `vendor/`에 여섯 코어가 모두 있어 skip 경로를 타지 않았다.
generator 결정성 테스트의 python skip: 0 (python 3.12.10). workspace test, 실제 GL test, 기기 배포
(`-Adb`)는 실행하지 않았고 `dist-device.ps1`은 인자 없이 로컬 조립만 했다.

검증 중간에 clippy가 새 테스트 helper의 `map_or(false, …)`를 지적해 `is_ok_and(…)`로 고쳤고, 그 뒤
완료 기준 전체(fmt부터 dist-device까지)를 다시 실행했다. 최종 검증 뒤 코드·asset·generator 변경 없음.

## 문서

`docs/MILESTONES.md` M5의 `셰이더 프리셋 4종 + 플랫폼 기본값, 오버레이 로더 + GB·CubeXX용 샘플 베젤`
한 줄만 `[ ]` → `[x]`로 바꿨다. 다른 checkbox와 결정문은 건드리지 않았다(파일의 나머지 diff는 이전
태스크의 미커밋 변경이다).

## 파일

생성:
- `build/generate-overlays.py`
- `assets/overlays/README.md`
- `assets/overlays/GB/720x720.png`

수정:
- `crates/slot2/src/overlay.rs` (production entry 1개, doc 정정)
- `crates/slot2/tests/overlay_layer.rs` (신규 8 tests + 낡은 빈 table 단언 정정)
- `docs/MILESTONES.md` (M5 checkbox 1줄)

다른 production/test/docs 파일은 수정하지 않았다. 임시 `target/t79-*.txt`는 삭제했다.

## 범위 밖 확인

다른 platform/geometry, GB의 640×480·720×480 pair, GBC 공유, platform default On 변경은 건드리지
않았다. Aspect fit/Fill용 별도 aperture와 scale-aware asset 선택은 만들지 않았으므로, 720×720에서
Integer 이외 scale을 고른 경우의 화면과 실기 화질 확인은 범위 밖으로 남는다.

## 계약 의문 / 남은 위험

- 남은 위험 1줄: sample aperture는 기본 Integer 배치(640×576 @ (40,72)) 전용이고 기본값이 여전히
  off이므로, 실기에서 그림을 보려면 사용자가 게임별 Overlay를 On으로 고르고 Integer scale을 유지해야
  하며 Aspect fit/Fill에서는 bezel이 게임과 어긋난다.
