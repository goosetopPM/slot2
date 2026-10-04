# Task 49 워커 결과 — libretro Core 치트 브리지

**성공. 누적 호출 1/2.** 지정 검증 3종 모두 종료 0. 실코어 테스트는 mGBA가 있는 환경에서 실제로
실행됐다(skip 아님).

## FFI 심볼과 공개 API (최종 형태)

- `ffi::Api`에 `cheat_reset: unsafe extern "C" fn()`와
  `cheat_set: unsafe extern "C" fn(c_uint, bool, *const c_char)`를 추가하고 `retro_cheat_reset` /
  `retro_cheat_set`을 기존 `get!` 매크로로 로드한다. 심볼이 없으면 기존 계약대로 `Error::Load`로
  코어 로드가 실패하며 panic하지 않는다.
- `Core::reset_cheats(&mut self)`와 `Core::set_cheat(&mut self, index: u32, enabled: bool,
  code: &str) -> Result<(), Error>` (권장 형태 그대로). 두 호출 모두 `ActiveSlot::bind` guard
  안에서 실행된다.

## 문자열·NUL·disabled·reset 처리

- 코드는 `CString::new`로만 준비해 `code.as_ptr()`로 그대로 전달한다. 구두점, `+`로 이어진 복수
  파트, 공백, UTF-8을 재해석·분해·정규화하지 않으며 빈 문자열도 그대로 넘긴다.
- 내부 NUL만 코어를 호출하기 전에 `Error::Game("the cheat code contains a NUL byte")`로 반환한다
  (panic·절단 없음). 인자 오류에 `Error::Game`을 쓰는 것은 `write_memory`의 "no region"/"wrong
  size"와 같은 이 크레이트의 기존 관례다.
- `enabled == false`도 실제 `retro_cheat_set`에 전달한다. disabled 항목을 거르는 정책은 이 계층에
  없다.
- `set_cheat`이 reset을 자동 호출하지 않는다: `reset → 인덱스 순서대로 재적용` 수명주기는 호출자가
  소유한다.

## 테스트

- `src/host.rs` 단위 테스트 2개(코어 불필요): C 문자열 준비가 바이트를 그대로 유지하는지(평범한
  코드·`+` 복수 파트·공백·구두점 포함 코드·UTF-8·빈 문자열), 내부 NUL이 오류이며 panic하지 않는지.
- `tests/mgba.rs`에 실코어 테스트 1개 추가: 기존 명시적 skip 규약(`core_path()`가 없으면 return)을
  유지하며, 코어가 있으면 두 새 심볼이 로드된 코어에서 `reset → set(true) → set(false) → 프레임
  5회 → reset → 프레임`이 계속되고 NUL 거부 뒤에도 정상 실행됨을 확인한다. 상태는 코어를 drop하며
  다음 테스트로 넘기지 않는다. 테스트에 쓰는 코드 문자열은 카드가 담는 GameShark v1 두 토막
  `+` 형태(`12345678+9ABCDEF0`)이며, 호스트가 이를 변형하지 않는다는 점이 검증 대상이다.
- 실코어가 없는 환경에서도 위 단위 테스트 2개는 항상 실행되므로 순수 로직 검증이 skip으로
  가려지지 않는다. `#[allow(...)]`나 테스트 전용 우회 경로는 넣지 않았다.

## 검증 (최종 코드 상태, 명세 순서)

- `cargo fmt --all -- --check` → 종료 **0**.
- `cargo test -p slot2-retro` → 종료 **0**: lib **12 passed**(신규 2 포함) / cores 5 / mgba **12**
  / registry 12 / rewind_cost 1 / doc 0, 모든 결과 줄 0 failed.
- `cargo clippy -p slot2-retro --all-targets -- -D warnings` → 종료 **0**(경고·오류 없음).
- 최종 검증 후 코드 변경 없음. 워크스페이스 전체 테스트·배포 빌드는 실행하지 않았다.

## 생성·수정 파일

- `crates/slot2-retro/src/ffi.rs` (+4): 두 심볼 필드와 `get!` 로드.
- `crates/slot2-retro/src/host.rs` (+67): `cheat_code_c` 헬퍼, `reset_cheats`/`set_cheat`, 단위 테스트 2개.
- `crates/slot2-retro/tests/mgba.rs` (+42): 실코어 치트 브리지 테스트 1개.
- 신규 파일 없음. `lib.rs`와 다른 크레이트는 손대지 않았다(공개 재노출도 기존 `Core` 메서드뿐이라 불필요).

## 계약 우려 / 남은 위험

`retro_cheat_set`은 void라 코어가 코드를 거부했는지 알 수 없다 — mGBA는 CodeBreaker 형식을 "not
supported"로 무시하고(로그는 이 호스트가 GET_LOG_INTERFACE를 거부해 보이지 않음), 형식 적합성은
후속 코어별 quirks 검증 몫이다. 소요 약 8분(상한 45분 내).
