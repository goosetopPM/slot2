# Task 54 — mGBA 플랫폼별 치트 validator

## 목적

Task 53의 순수 치트 검증 API를 플랫폼 인식형으로 바꾸고, 저장소가 pin한 mGBA libretro adapter가
GB/GBC와 GBA에서 실제로 전달하는 형식만 검증한다. mGBA를 `Unchecked`에서 `Validated`로 올리되,
검증되지 않은 gpSP·FCEUmm·Genesis Plus GX는 그대로 `Unchecked`로 둔다.

이번 태스크는 `slot2-retro`의 순수 validator만 다룬다. Session 적용과 UI 오류 표시는 후속이다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 모두 횟수에 포함한다.

## 먼저 읽을 범위

- `AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `docs/DECISIONS.md`의 D-03, D-05, D-21만
- `tasks/53-snes9x-cheat-validation-foundation.result.md`
- `crates/slot2-retro/src/quirks.rs`
- `crates/slot2-retro/src/registry.rs`의 `Platform`, `Core`, `PLATFORMS` 주변만
- `crates/slot2-retro/tests/cheat_quirks.rs`
- `cores/mgba/commit`

## 고정된 upstream 근거

네트워크를 사용하지 말고 아래 사실을 이 태스크의 입력 계약으로 사용한다.

저장소의 pinned mGBA commit은 `e31759b24e7a4e3899285ff720d7b573ac328ae7`이다. 해당
`src/platform/libretro/libretro.c`의 `retro_cheat_set`은 실행 중 플랫폼에 따라 다르게 동작한다.

### GBA content

- 입력의 `+` 또는 C locale의 ASCII whitespace(`space`, `tab`, `LF`, `VT`, `FF`, `CR`) 한 바이트를
  operand separator로 바꾼다.
- adapter가 `mCheatAddLine`에 넘기는 canonical unit은 다음 셋이다.
  1. CodeBreaker: `AAAAAAAA+VVVV` — 8 hex + separator + 4 hex
  2. GameShark/PAR: `AAAAAAAA+VVVVVVVV` — 8 hex + separator + 8 hex
  3. VBA 32-bit: `AAAAAAAA:VVVVVVVV` — 8 hex + colon + 8 hex
- 위 예시의 `+` 자리는 위 ASCII whitespace 한 바이트여도 같다.
- 여러 unit은 `+` 또는 ASCII whitespace **한 바이트**로 이어진다. 연속 separator는 adapter의
  고정 위치 재구성을 어긋나게 하므로 유효 형식으로 인정하지 않는다.
- underlying parser가 더 짧은 VBA 값을 아는 것과 별개로, 이 libretro adapter는 17-byte unit에
  도달해야 호출하므로 `AAAAAAAA:VV`와 `AAAAAAAA:VVVV`는 전달되지 않는다.

### GB/GBC content

- `+` 또는 위 ASCII whitespace가 code unit 경계다.
- canonical unit은 다음 다섯 형식이다.
  1. GameShark: `XXXXXXXX` — 8 hex
  2. Game Genie: `XXX-XXX` — 3 hex, hyphen, 3 hex
  3. Game Genie with compare: `XXX-XXX-XXX`
  4. CodeBreaker: `XXXXXX-XX` — 6 hex, hyphen, 2 hex
  5. VBA: `XXXX:XX` — 4 hex, colon, 2 hex
- 각 unit은 최대 11 bytes이며 여러 unit은 separator 한 바이트로 잇는다. 빈 unit은 인정하지 않는다.

### 공통 주의

- adapter가 byte별로 C `isspace`를 호출하므로 mGBA code는 ASCII만 받아야 한다. non-ASCII UTF-8을
  core까지 보내지 않는다.
- ASCII hex는 `0-9`, `a-f`, `A-F`다.
- adapter는 `retro_cheat_set`의 `index`와 `enabled`를 사용하지 않는다. 따라서 현재 Session이
  disabled entry까지 전달하는 방식은 mGBA에서 실제 비활성화를 보장하지 않는다. **이번 태스크에서
  Session을 수정하지 말고 결과 보고서의 남은 위험에 반드시 기록한다.** 후속 태스크가 실제 선택
  CoreId에 따라 disabled entry를 생략하도록 연결한다.

근거 URL은 보고서에 길게 인용하지 말고 source provenance로만 기록한다.

- pinned adapter:
  `https://github.com/libretro/mgba/blob/e31759b24e7a4e3899285ff720d7b573ac328ae7/src/platform/libretro/libretro.c#L2037-L2097`
- pinned GBA parser:
  `https://github.com/libretro/mgba/blob/e31759b24e7a4e3899285ff720d7b573ac328ae7/src/gba/cheats.c#L145-L235`
- 공식 libretro-database CodeBreaker 예:
  `https://github.com/libretro/libretro-database/blob/master/cht/Nintendo%20-%20Game%20Boy%20Advance/Mega%20Man%20Zero%20%28USA%2C%20Europe%29%20%28Code%20Breaker%29.cht`

## 구현 계약

### 1. 플랫폼 인식형 API

기존 공개 API를 다음 의미로 변경한다.

```rust
pub fn validate_cheat(
    core: CoreId,
    platform: Platform,
    code: &str,
) -> Result<CheatValidation, CheatValidationError>;
```

- Task 53의 공통 empty/interior-NUL 검사와 오류 타입은 유지한다.
- core와 platform의 지원 관계가 맞지 않으면 문법 검사 전에 오류를 반환한다.
  - mGBA: GB, GBC, GBA
  - gpSP: GBA
  - FCEUmm: NES
  - SNES9x: SNES
  - Genesis Plus GX: MD, SMS
- 지원 관계는 한 곳에서만 정의한다. `CoreId::supports_platform(platform)` 같은 작은 registry API로
  두어도 좋고 quirks 내부의 단일 함수로 두어도 좋다. 기존 `PLATFORMS` 기본 core만 조회해서 gpSP를
  누락하면 안 된다.
- mismatch 오류에는 core와 platform을 식별할 수 있는 reason을 남기되 cheat code 전체는 넣지 않는다.
- SNES9x는 `Platform::Snes`에서 Task 53과 완전히 같은 결과를 내야 한다.

### 2. mGBA GBA validator

- 입력 전체가 ASCII인지 먼저 검사한다.
- 위 GBA 세 unit 형식만 인정한다.
- `+`와 여섯 ASCII whitespace를 separator로 인정하되 separator는 정확히 한 바이트여야 한다.
- CodeBreaker의 4-hex operand와 GameShark/PAR의 8-hex operand를 모두 인정한다.
- 여러 unit을 순서대로 끝까지 소비해야 `Validated`다. 선행·후행·연속 separator, 남는 문자, 부분
  unit이 하나라도 있으면 전체 오류다.
- lowercase/mixed-case hex를 인정한다.
- 입력을 변환하거나 정규화하지 않는다.

### 3. mGBA GB/GBC validator

- GB와 GBC는 같은 mGBA parser branch와 같은 다섯 unit 형식을 사용한다.
- 입력 전체가 ASCII인지 먼저 검사한다.
- `+`와 여섯 ASCII whitespace를 unit separator로 인정하되 빈 unit은 오류다.
- 모든 unit이 다섯 형식 중 하나이고 입력 전체가 소비되어야 `Validated`다.
- lowercase/mixed-case hex를 인정하고 입력을 변경하지 않는다.

### 4. 기존 코어 상태

- SNES9x + SNES는 기존 `Validated`/오류 계약을 유지한다.
- gpSP + GBA, FCEUmm + NES, Genesis Plus GX + MD/SMS는 non-empty·NUL-free 입력에 대해 계속
  `Unchecked`다.
- 해당 세 코어의 문법을 추측하지 않는다.

## 테스트 계약

`crates/slot2-retro/tests/cheat_quirks.rs`를 확장해 최소 다음을 직접 확인한다.

- 5개 core의 전체 지원 platform matrix와 모든 mismatch 거부
- 기존 SNES9x 유효·무효·255/256-byte 테스트가 `Platform::Snes`에서 그대로 유지됨
- GBA의 8+4, 8+8, 8:8 형식과 lower/mixed case
- GBA 실제 DB 형태 `3202B634+0080` 및 다중 unit
  `000052DC+000A+10000F4E+0007`
- GBA unit 내부와 unit 사이에서 `+`, space, tab, LF, VT, FF, CR 한 바이트 처리
- GBA의 짧은 VBA 값, 7/9자리 주소, 3/5/6/7자리 operand, non-hex, 선행·후행·연속 separator,
  잘린 마지막 unit 거부
- GB와 GBC 각각 8-hex, `XXX-XXX`, `XXX-XXX-XXX`, `XXXXXX-XX`, `XXXX:XX` 성공
- 실제 DB 형태 `151-91A-7FC+151-93A-F7E+151-9BA-5DB` 성공
- GB/GBC의 잘못된 hyphen/colon 위치, 길이, non-hex, 12-byte unit, 빈 unit 거부
- mGBA 세 플랫폼에서 한글·전각 숫자 등 non-ASCII 거부
- gpSP/FCEUmm/Genesis Plus GX의 올바른 플랫폼은 계속 `Unchecked`
- 오류 reason이 전체 code를 복사하지 않고 입력 문자열도 바뀌지 않음

테스트 통과만을 위한 제품 분기, 실제 core 호출, 제품용 mock hook는 금지한다.

## 수정 허용 범위

- `crates/slot2-retro/src/quirks.rs`
- 지원 matrix API가 꼭 필요할 때만 `crates/slot2-retro/src/registry.rs`
- `crates/slot2-retro/tests/cheat_quirks.rs`
- registry API를 추가한 경우에만 `crates/slot2-retro/tests/registry.rs`
- `crates/slot2-retro/src/lib.rs`는 re-export 변경이 꼭 필요할 때만
- `tasks/54-mgba-cheat-validator.worker-result.md`

그 밖의 파일은 수정하지 않는다.

## 금지·범위 밖

- 커밋·푸시
- 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- `~/.gjc-bai/agent/*.yml` 및 공용 설정 수정
- Session/App 연결, disabled entry filtering, launch/toast 변경
- `.cht` 읽기·쓰기 변경
- gpSP·FCEUmm·Genesis Plus GX 문법 validator
- upstream parser 코드를 그대로 복사하거나 C FFI 추가
- 워크스페이스 전체 테스트와 배포 빌드

## 완료 기준

최종 코드 상태에서 아래를 원문 그대로 순서대로 실행한다.

```text
cargo fmt --all -- --check
cargo test -p slot2-retro --test registry
cargo test -p slot2-retro --test cheat_quirks
cargo clippy -p slot2-retro --all-targets -- -D warnings
```

각 명령의 종료 코드와 마지막 결과 줄을 기록한다. 검증 뒤 소스를 고치면 네 명령을 다시 실행한다.
첫 응답까지 최대 5분, 전체 최대 45분 기다린다. 진행 신호는 로그 크기가 아니라
`git status --short`다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\54-mgba-cheat-validator.worker-result.md`를 작성한다.

- 성공/실패와 누적 호출 횟수
- 플랫폼 인식형 API와 지원 matrix
- mGBA GBA 및 GB/GBC에서 인정·거부하는 형식
- SNES9x 회귀와 나머지 코어 `Unchecked` 유지 결과
- 네 검증 명령의 종료 코드와 마지막 결과 줄
- 생성·수정 파일 목록
- **mGBA가 `enabled`를 무시하는 현재 Session 위험**과 후속 필요 사항
- 계약이 틀려 보이는 부분 또는 추가 남은 위험 1줄

실패해도 반드시 원인과 마지막 상태를 쓴다. 두 번째 호출도 실패하면 더 시도하지 않는다.
