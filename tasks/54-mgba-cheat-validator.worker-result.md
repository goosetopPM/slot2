# Task 54 워커 결과 — mGBA 플랫폼별 치트 validator

**성공. 누적 호출 1/2.** 지정 검증 4종 모두 종료 0.

근거는 pin된 upstream commit `e31759b24e7a4e3899285ff720d7b573ac328ae7`(mgba) 하나로만 두었다:
adapter `src/platform/libretro/libretro.c`의 `retro_cheat_set`(GBA/GB 분기와 separator 처리)와
GBA parser `src/gba/cheats.c`. 네트워크는 쓰지 않았다.

## 플랫폼 인식형 API와 지원 matrix

- `validate_cheat(core: CoreId, platform: Platform, code: &str) -> Result<CheatValidation,
  CheatValidationError>`로 확장(권장 시그니처 그대로). Task 53의 공통 empty/interior-NUL 검사와
  오류 타입은 유지하되, 오류에 `platform`과 `platform()` accessor를 추가해 mismatch를 텍스트
  파싱 없이 구분할 수 있게 했다.
- 지원 관계는 `registry::Core::supports_platform(platform)` 한 곳에만 있다: mGBA = GB/GBC/GBA,
  gpSP = GBA, FCEUmm = NES, SNES9x = SNES, Genesis Plus GX = MD/SMS. gpSP는 `PLATFORMS`의 기본
  core가 아니라 테이블 조회로는 누락되므로 손으로 적었고, registry 테스트가 같은 표를 반대 방향으로
  다시 검증한다(모든 `PLATFORMS` 기본 core가 자기 플랫폼을 지원 + gpSP는 기본이 아님).
- 순서: mismatch → 빈 문자열 → 내부 NUL → 문법. mismatch 오류는 core·platform을 보존하고 reason에
  code를 담지 않는다.

## mGBA가 인정·거부하는 형식

- **GBA**(unit이 separator를 포함하는 구조라 fixed-offset 재구성 전체를 파싱):
  - 인정: `AAAAAAAA+VVVV`(8 hex + separator + 4 hex), `AAAAAAAA+VVVVVVVV`(8 + 8),
    `AAAAAAAA:VVVVVVVV`(8 + `:` + 8). separator는 unit 내부이든 unit 사이든 `+` 또는 C isspace
    여섯 바이트(space/tab/LF/VT/FF/CR) **한 바이트**만. 여러 unit은 separator 한 바이트로 잇는다.
    실제 DB 형태 `3202B634+0080`, `000052DC+000A+10000F4E+0007` 통과.
  - 거부: 7/9자리 주소, 3/5/6/7자리 operand, 짧은 VBA(`AAAAAAAA:VV`, `AAAAAAAA:VVVV`), non-hex,
    선행·후행·연속 separator, 잘린 마지막 unit, non-ASCII. 길이 판정은 unit 후보(13/17 bytes)별
    역방향 DP로 O(n)이고, 후보가 겹치는 입력은 소비가 정확히 끝나는 조합만 통과한다.
- **GB/GBC**(한 unit이 곧 한 token이라 split 후 각 token 검사):
  - 인정: `XXXXXXXX`(8 hex), `XXX-XXX`, `XXX-XXX-XXX`, `XXXXXX-XX`(9), `XXXX:XX`. 단일 separator로
    연결, 실제 DB 형태 `151-91A-7FC+151-93A-F7E+151-9BA-5DB` 통과.
  - 거부: hyphen/colon 위치 오류, 6/10/12-byte unit 등 길이 오류, non-hex, 빈 unit(선행·후행·연속
    separator), non-ASCII.
- non-ASCII는 세 플랫폼 모두 문법 검사 전에 거부한다(mGBA가 byte별 `isspace`를 호출하므로 한글·
  전각 숫자를 core로 보내지 않는다). ASCII hex는 0-9/a-f/A-F, 대소문자 모두 인정하고 입력은
  변환·정규화하지 않는다.

## SNES9x 회귀와 나머지 코어

- `CoreId::Snes9x` + `Platform::Snes`는 Task 53과 완전히 같은 결과다: 세 형식·다섯 separator·연속
  separator 허용, 255-byte 통과/256-byte 실패, 600-byte 한글 길이 거부, non-hex·잘못된 위치 거부를
  그대로 유지(테스트도 `Platform::Snes`로 이식).
- gpSP+GBA, FCEUmm+NES, GPX+MD/SMS는 여전히 `Unchecked`이며, mGBA나 SNES9x가 거부하는 코드를
  넣어도 `Unchecked`로 남는다(남의 문법을 빌려 거부하지 않는다). 그 세 코어의 문법은 추측하지 않았다.

## 검증 (최종 코드 상태, 명세 순서)

- `cargo fmt --all -- --check` → 종료 **0**.
- `cargo test -p slot2-retro --test registry` → 종료 **0**,
  `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s`.
- `cargo test -p slot2-retro --test cheat_quirks` → 종료 **0**,
  `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s`.
- `cargo clippy -p slot2-retro --all-targets -- -D warnings` → 종료 **0**(경고·오류 없음).
- 최종 검증 후 코드 변경 없음.

## 생성·수정 파일

- `crates/slot2-retro/src/quirks.rs`: 플랫폼 인식형 `validate_cheat`, mismatch 처리, 오류에 platform,
  mGBA GBA 구조 파서와 GB/GBC 5형식 validator(SNES9x 규칙은 그대로).
- `crates/slot2-retro/src/registry.rs`: `Core::supports_platform`(+55줄).
- `crates/slot2-retro/tests/cheat_quirks.rs`: 15 테스트(지원 matrix, SNES9x 회귀, GBA/GB 형식과
  거부, separator 7종, non-ASCII, `Unchecked` 유지, 오류 형태, 입력 불변).
- `crates/slot2-retro/tests/registry.rs`: 지원 matrix 교차 검증 테스트 1개.
- `lib.rs`는 이번 회차에 수정하지 않았다(re-export가 그대로 필요를 충족).

## mGBA가 `enabled`를 무시하는 현재 Session 위험 (필수 기록)

pin된 mGBA adapter의 `retro_cheat_set`은 `index`와 `enabled`를 **사용하지 않는다**. 따라서 Task 50의
Session이 disabled 항목까지 `set_cheat(index, enabled, code)`로 전달하더라도, mGBA에서는 그 항목이
실제로 꺼지지 않는다(활성화된 치트처럼 적용될 수 있다). 이번 태스크에서는 Session을 건드리지 않았다.
후속 태스크가 **실제 선택된 CoreId**(`Core::from_library_path`로 판별)에 따라 disabled entry를 생략해
적용해야 하며, 그 전까지 mGBA의 off 표시는 UI상의 상태일 뿐 코어 동작을 보장하지 않는다.

## 계약이 틀려 보이는 부분 / 추가 남은 위험

GBA의 8+8 GameShark unit과 8+4 CodeBreaker unit 뒤에 또 다른 unit이 붙는 입력은 두 해석이 모두
가능한데, DP가 "입력 전체를 정확히 소비하는" 조합만 통과시키므로 오탐은 없지만, 이론적으로 두 해석이
모두 성립하는 문자열은 어느 쪽으로 읽히든 같은 unit 집합이 된다. 그 밖에는 SNES9x와 mGBA 외 세
코어가 여전히 `Unchecked`라는 점이 남은 위험이다. 소요 약 14분(상한 45분 내).
