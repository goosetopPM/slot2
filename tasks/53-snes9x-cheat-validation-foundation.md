# Task 53 — SNES9x 치트 검증 기반

## 목적

D-21의 코어별 치트 형식 사전 검증을 작은 단위로 시작한다. 이번 태스크는 실제 선택된 libretro
라이브러리 이름을 `CoreId`로 식별하는 API와, pinned SNES9x 파서가 안전하게 받을 수 있는 코드만
인정하는 순수 validator를 `slot2-retro`에 만든다.

이번 태스크에서는 Session이나 App에 연결하지 않는다. mGBA·gpSP·FCEUmm·Genesis Plus GX는
검증했다고 가장하지 않고 명시적인 `Unchecked` 결과로 남긴다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 모두 횟수에 포함한다.

## 먼저 읽을 범위

- `AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `docs/DECISIONS.md`의 D-03, D-05, D-21만
- `tasks/49-libretro-cheat-bridge.result.md`
- `tasks/52-cheat-menu-app-wiring.result.md`
- `crates/slot2-retro/src/lib.rs`의 module/re-export와 오류 형식 주변만
- `crates/slot2-retro/src/registry.rs`의 `Core` 정의와 구현만
- `crates/slot2-retro/tests/registry.rs`
- `cores/snes9x/commit`

## 고정된 upstream 근거

네트워크를 사용하지 말고 아래 사실을 이 태스크의 입력 계약으로 사용한다.

- 저장소가 pin한 SNES9x libretro commit은 `cores/snes9x/commit`의 값이다.
- 그 adapter의 `retro_cheat_set`은 코드를 `char codeCopy[256]`에 복사하므로 terminating NUL을
  포함해 맞으려면 Rust 입력은 **최대 255 bytes**여야 한다.
- adapter는 `+`, `,`, `.`, `;`, ASCII space를 separator로 사용하며 빈 separator run은 건너뛴다.
- 각 token은 다음 셋 중 하나다.
  1. 8자리 ASCII hex Pro Action Replay 코드
  2. `AAAAAA:VV` 형태: 6자리 ASCII hex 주소, colon, 2자리 ASCII hex 값
  3. `XXXX-XXXX` 형태의 SNES Game Genie 코드. 여덟 문자는 대소문자 구분 없이
     `DF4709156BC8A23E` alphabet에 속한다.
- libretro `retro_cheat_set`은 반환값이 `void`라 호출 뒤 거부 여부를 host가 알아낼 수 없다.

근거를 코드 주석에 남길 때는 **왜 255-byte 경계와 별도 validator가 필요한지**만 적고, 위 계약을
장황하게 복사하지 않는다.

## 구현 계약

### 1. 실제 라이브러리 이름 → CoreId

`registry::Core`에 경로에서 코어를 식별하는 공개 API를 추가한다. 권장 시그니처는 다음과 같다.

```rust
pub fn from_library_path(path: &Path) -> Option<Self>
```

- 파일명 stem이 각 variant의 `base_name()`과 정확히 일치할 때만 해당 variant를 반환한다.
- Windows에서 실제 파일명 대소문자가 달라도 인식되도록 ASCII case-insensitive로 비교한다.
- `.dll`, `.so`, `.dylib`를 모두 host OS와 무관하게 인식한다. 확장자 없는 base name도 인식해도
  되지만, 부분 문자열·접두/접미 쓰레기·다른 확장자를 잘못 인식하면 안 된다.
- 부모 디렉터리 이름은 판정에 쓰지 않는다.
- 기존 `base_name`, `file_name`, platform registry 결과는 바꾸지 않는다.

### 2. 명시적인 검증 상태

새 `quirks` 모듈에 공개 API를 둔다. 이름은 Rust 관례에 맞게 조정할 수 있지만 다음 의미를
보존한다.

```rust
pub enum CheatValidation {
    Validated,
    Unchecked,
}

pub fn validate_cheat(core: CoreId, code: &str)
    -> Result<CheatValidation, CheatValidationError>;
```

- SNES9x에서 계약을 모두 만족하면 `Validated`다.
- 다른 네 CoreId는 이번 태스크에서 문법을 추측하지 않고 `Unchecked`다.
- 모든 코어에서 빈 문자열과 interior NUL은 오류다. 이 공통 검사는 `Unchecked`보다 먼저 한다.
- 오류는 적어도 core와 사람이 이해할 수 있는 reason을 보존하며 `Debug`, `Display`,
  `std::error::Error`를 제공한다. 전체 cheat code를 오류 문자열에 그대로 복사하지 않는다.
- 이 타입과 함수는 crate root에서 re-export한다.
- validator는 순수 함수이며 파일·core·전역 상태를 읽지 않는다.

### 3. SNES9x 규칙

- 길이 판정은 Unicode scalar 수가 아니라 `code.as_bytes().len()`으로 한다. 256 bytes 이상은 core를
  호출하기 전에 오류다.
- separator는 고정 근거의 다섯 종류만 인정한다. 연속·선행·후행 separator는 core처럼 건너뛸 수
  있지만, separator만 있는 문자열은 유효 token이 없으므로 오류다.
- 모든 non-empty token이 세 형식 중 하나여야 한다. 하나라도 틀리면 전체 오류다.
- ASCII hex는 `0-9`, `a-f`, `A-F`만 인정한다. Game Genie alphabet도 대소문자를 모두 인정한다.
- 코드를 변환·정규화·재정렬하지 않는다.
- 255-byte 경계의 유효한 다중 token은 통과하고, 같은 구조의 256-byte 입력은 길이 오류가 나도록
  테스트한다.

### 4. 경계 유지

- `Core::reset_cheats`와 `Core::set_cheat`의 동작·오류는 바꾸지 않는다.
- `slot2`, `slot2-store`, UI, i18n, Session에는 연결하지 않는다.
- 다른 코어의 문법과 플랫폼별 치트 포맷을 추측해서 추가하지 않는다.
- SNES9x upstream parser 코드를 복사하거나 C FFI를 추가하지 않는다.

## 테스트 계약

기존 `crates/slot2-retro/tests/registry.rs`와 새
`crates/slot2-retro/tests/cheat_quirks.rs`에서 최소 다음을 직접 확인한다.

- 5개 canonical base name의 `.dll`, `.so`, `.dylib` 식별과 ASCII 대소문자 처리
- 비슷한 접두/접미 이름, 알 수 없는 core, 다른 확장자, 파일명 없는 경로의 거부
- SNES9x의 8 hex, 6-hex-colon-2-hex, Game Genie 각 유효 예와 대소문자
- 다섯 separator와 여러 token, separator run 처리
- 잘못된 길이·문자·colon/hyphen 위치, tab/newline, 유효 token이 없는 입력의 거부
- 정확히 255 bytes인 유효 다중 token의 성공과 256 bytes의 실패
- 공통 empty/interior-NUL 오류가 모든 CoreId에 적용됨
- SNES9x 성공은 `Validated`, 나머지 네 코어의 non-empty/NUL-free 입력은 `Unchecked`
- 검증 호출이 입력 문자열을 변경하지 않음

테스트 통과만을 위한 예외 분기, 실제 core 호출, 제품용 mock hook는 금지한다.

## 수정 허용 범위

- `crates/slot2-retro/src/registry.rs`
- `crates/slot2-retro/src/lib.rs`
- 새 `crates/slot2-retro/src/quirks.rs` 또는 `crates/slot2-retro/src/quirks/mod.rs`와 그 하위 파일
- `crates/slot2-retro/tests/registry.rs`
- 새 `crates/slot2-retro/tests/cheat_quirks.rs`
- `tasks/53-snes9x-cheat-validation-foundation.worker-result.md`

그 밖의 파일은 수정하지 않는다.

## 금지·범위 밖

- 커밋·푸시
- 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- `~/.gjc-bai/agent/*.yml` 및 공용 설정 수정
- Session/App 연결과 launch/toast 변경
- `.cht` 읽기·쓰기 변경
- mGBA·gpSP·FCEUmm·Genesis Plus GX 문법 validator
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

끝나기 전에 `C:\SLOT2\tasks\53-snes9x-cheat-validation-foundation.worker-result.md`를 작성한다.

- 성공/실패와 누적 호출 횟수
- library path 식별 계약과 테스트 결과
- SNES9x validator가 인정·거부하는 형식과 255-byte 경계
- 다른 네 코어가 `Unchecked`로 남는지
- 네 검증 명령의 종료 코드와 마지막 결과 줄
- 생성·수정 파일 목록
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄

실패해도 반드시 원인과 마지막 상태를 쓴다. 두 번째 호출도 실패하면 더 시도하지 않는다.
