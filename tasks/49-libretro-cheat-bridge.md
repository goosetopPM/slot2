# Task 49 — libretro Core 치트 브리지

## 목적

Task 48이 읽은 치트를 나중에 실행 세션이 적용할 수 있도록 `slot2-retro::Core`에 libretro의
`retro_cheat_reset` / `retro_cheat_set` 저수준 API를 노출한다. 저장소 타입이나 App 정책은 이
크레이트에 끌어들이지 않는다.

이 태스크는 **누적 최대 2회**다. 같은 태스크의 실패·중단 호출도 모두 횟수에 포함한다.

## 먼저 읽을 범위

- `AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `crates/slot2-retro/src/ffi.rs`
- `crates/slot2-retro/src/host.rs`
- `crates/slot2-retro/src/lib.rs`에서 공개 API와 오류 타입 주변만
- `crates/slot2-retro/tests/mgba.rs`
- 이 태스크에 필요한 기존 테스트 helper만 제한적으로 읽는다.

## 구현 계약

### 1. FFI 심볼

- 동적 코어를 여는 `Api`에 아래 필수 libretro 심볼을 정확한 ABI로 추가한다.
  - `retro_cheat_reset(void)`
  - `retro_cheat_set(unsigned index, bool enabled, const char *code)`
- 다른 필수 심볼과 같은 방식으로 로드한다. 심볼이 없으면 코어 로드 오류로 반환하며 panic하지
  않는다.
- 새 dependency나 자체 libretro 헤더 복사본을 추가하지 않는다.

### 2. Core 공개 API

- `Core`에 저장소 타입과 무관한 명확한 메서드를 추가한다. 권장 형태는 다음과 같다. 기존 명명과
  오류 관례상 더 자연스러운 동등 API는 허용한다.

```rust
pub fn reset_cheats(&mut self);
pub fn set_cheat(&mut self, index: u32, enabled: bool, code: &str) -> Result<(), Error>;
```

- 두 호출 모두 기존 libretro callback/active-slot 규약이 요구하는 guard를 사용한다.
- `set_cheat`는 전달받은 `index`, `enabled`, `code`를 그대로 C API에 전달한다. 구두점, `+`로
  이어진 복수 파트, 공백, UTF-8을 재해석·분해·정규화하지 않는다.
- C 문자열 내부 NUL만 호출 전에 오류로 반환한다. panic하거나 문자열을 자르지 않는다.
- reset을 `set_cheat` 내부에서 자동 호출하지 않는다. 호출자가 `reset -> 인덱스 순서대로 전체
  재적용` 수명주기를 제어할 수 있어야 한다.
- `enabled == false`도 실제 `retro_cheat_set`에 전달한다. disabled entry를 생략하는 정책을 이
  계층에 넣지 않는다.
- 브리지 계층은 코드 형식이나 빈 문자열을 코어별로 검증하지 않는다. Task 48의 완성된 항목 계약과
  후속 코어별 검증이 담당한다.

### 3. 의존성 경계

- `slot2-retro`에서 `slot2-store`에 의존하지 않는다.
- `slot2_store::Cheat`, 카드 경로, 게임별 설정, UI/App/Session 적용 정책을 추가하지 않는다.
- 코어 이름별 코드 형식 검증·변환도 이번 범위가 아니다.

## 테스트 계약

- 코어 없이 실행 가능한 집중 테스트로 다음을 확인한다.
  - 평범한 코드, 문장부호, `+` 복수 파트, UTF-8이 C 문자열 준비 과정에서 바이트 그대로 유지된다.
  - 내부 NUL은 반환 오류이며 panic하지 않는다.
- 기존 mGBA 실코어 테스트의 명시적 skip 규약을 유지하면서, 코어가 있을 때 새 두 심볼을 실제로
  로드하고 reset/set/reset 호출 뒤 프레임 실행이 계속 가능함을 확인한다. 테스트 코드 자체는
  유효한 mGBA 형식을 사용하고 상태를 다음 테스트로 누출하지 않는다.
- 실코어가 없는 환경의 skip을 성공으로 위장해 브리지의 순수 로직 검증까지 건너뛰지 않는다.
- 테스트만 만족시키는 별도 경로나 `#[allow(...)]`를 넣지 않는다.

## 수정 허용 범위

- `crates/slot2-retro/src/ffi.rs`
- `crates/slot2-retro/src/host.rs`
- 공개 재노출 또는 문서가 꼭 필요할 때만 `crates/slot2-retro/src/lib.rs`
- `crates/slot2-retro/tests/mgba.rs`
- 같은 크레이트의 집중 단위 테스트
- `tasks/49-libretro-cheat-bridge.worker-result.md`

그 밖의 소스는 수정하지 않는다. 특히 `slot2-store`, `slot2-app`, `slot2-ui`, 카드 데이터와 공용
GJC 설정은 건드리지 않는다.

## 금지·범위 밖

- 커밋·푸시
- 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- `~/.gjc-bai/agent/*.yml` 및 공용 설정 수정
- Task 48 파서 재수정
- Session/App 연결, 시작 시 자동 적용, UI, 저장·토글 정책
- 코어별 치트 코드 allowlist/validator
- 워크스페이스 전체 테스트와 배포 빌드

## 완료 기준

최종 코드 상태에서 아래를 원문 그대로 순서대로 실행한다.

```text
cargo fmt --all -- --check
cargo test -p slot2-retro
cargo clippy -p slot2-retro --all-targets -- -D warnings
```

각 명령의 종료 코드와 마지막 결과 줄을 기록한다. 검증 뒤 소스를 고치면 세 명령을 다시 실행한다.
첫 응답까지 최대 5분, 전체 최대 45분 기다린다. 진행 신호는 로그 크기가 아니라
`git status --short`다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\49-libretro-cheat-bridge.worker-result.md`를 작성한다.

- 성공/실패 및 누적 호출 횟수
- FFI 심볼과 공개 API의 실제 최종 형태
- 문자열·NUL·disabled·reset 수명주기 처리
- 추가/수정한 테스트와 실코어 skip 여부
- 세 검증 명령의 종료 코드와 마지막 결과 줄
- 생성·수정 파일 목록
- 계약이 틀려 보이는 부분 또는 남은 위험 1줄

실패해도 반드시 원인과 마지막 상태를 쓴다. 두 번째 호출도 실패하면 더 시도하지 않는다.
