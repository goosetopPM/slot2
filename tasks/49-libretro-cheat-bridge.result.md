# Task 49 최종 검토 — libretro Core 치트 브리지

## 판정

**통과** — 누적 호출 1/2.

`ffi::Api`가 `retro_cheat_reset`과 `retro_cheat_set`을 필수 심볼로 정확한 libretro ABI에 맞춰
로드한다. `Core::reset_cheats`와 `Core::set_cheat`는 기존 active-slot guard 안에서 호출되며,
저장소 타입이나 Session/App 정책에 의존하지 않는다.

`set_cheat`는 index와 enabled 값을 그대로 전달하고 코드 문자열을 분해·정규화하지 않는다.
disabled 항목도 코어에 전달하며 reset은 호출자가 명시적으로 수행한다. C 문자열로 표현할 수 없는
내부 NUL만 코어 호출 전에 `Error::Game`으로 거부한다.

## 검증 근거

- `cargo fmt --all -- --check` — 종료 0.
- `cargo test -p slot2-retro` — 종료 0. lib 12, cores 5, mGBA 12, registry 12,
  rewind_cost 1, 모두 0 failed.
- `cargo clippy -p slot2-retro --all-targets -- -D warnings` — 종료 0.
- 코어 독립 단위 테스트가 문자열 byte 보존과 내부 NUL 오류를 검증한다.
- 이번 환경에는 mGBA 코어가 있어 실코어 테스트가 skip되지 않았다. MIT 테스트 ROM을 로드한 뒤
  reset, enabled/disabled set, 프레임 실행, 재-reset 및 NUL 거부 뒤 프레임 실행을 확인했다.
- Codex가 관련 세 파일의 diff와 테스트 helper를 검토했고 `git diff --check`상 새 공백 오류가 없다.

## 남은 비차단 사항

- libretro 치트 API는 반환값이 `void`이므로 코어가 특정 코드 형식을 받아들였는지는 이 계층에서
  알 수 없다. 코어별 형식 검증과 Session 적용 정책은 후속 태스크가 담당해야 한다.
