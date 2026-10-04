# Task 68 — Codex 최종 판정

## 판정

**통과. 누적 호출 1/2.** 추가 작업자 호출은 필요하지 않다. Codex는 사용자의 운영 규칙에 따라
검증 명령을 다시 실행하지 않고 작업자 보고서와 Session·집중 테스트 변경을 대조했다.

## 통과한 부분

- store `ShaderPreset`에서 gfx `ShaderEffect`로 가는 변환이 Session의 exhaustive match 한 곳에만
  존재한다. 문자열·variant index 변환이나 crate 간 새 의존은 없다.
- 네 preset은 동명의 gfx effect로, explicit Off와 key 부재는 현재 플랫폼 기본값 미구현 상태에서
  각각 별도 match arm을 거쳐 plain draw로 해석된다. 카드의 두 저장 의미는 합쳐지거나 다시 쓰이지 않는다.
- `Session::open`이 이미 한 번 읽은 settings로 shader runtime 상태를 초기화한다. `start`와
  `start_named`가 같은 경로를 사용하며 추가 settings I/O가 없다.
- accessor와 setter가 renderer 상태만 다룬다. setter는 frame, core, texture, audio/sink와 카드 bytes를
  변경하지 않는다.
- game draw는 기존 scale/aspect/overscan 계산에서 나온 texture id, geometry, crop UV와 white tint를
  effect 유무와 관계없이 공유한다. effect가 있을 때만 `image_effect_uv`로 라우팅한다.
- frame 또는 texture가 없으면 effect 설정 여부와 무관하게 기존처럼 아무 image도 그리지 않는다.
- App, UI, store, gfx와 platform/retro registry는 수정하지 않았다.

## 검증 근거

- 작업자 `cargo fmt --all -- --check`: 종료 0.
- 작업자 `cargo test -p slot2 --test session`: **37 passed / 0 failed**, core-dependent skip 0.
- 작업자 `cargo test -p slot2 --lib`: **33 passed / 0 failed**.
- 작업자 downstream `cargo check -p slot2 --tests`와
  `cargo clippy -p slot2 --all-targets -- -D warnings`: 종료 0.
- 신규 8개 테스트가 여섯 mapping 입력, None/Off 보존, 네 effect routing, geometry·UV·tint 동일성,
  scale 병행, setter 무부작용, named-core launch와 frame 부재를 검증했다.
- 최종 검증 뒤 코드 변경이 없고 Codex의 범위 파일 `git diff --check`도 오류가 없다.

## 남은 한계

현재 real-core fixture는 overscan 없는 GBA라 cropped UV의 실제 값은 full UV다. production의 두 draw
분기는 같은 계산된 `uv` 변수를 직접 공유하고 Task67이 crop sentinel을 실제 GLES2로 검증했으므로
이번 라우팅의 추가 수정은 필요하지 않다. NES crop과 실기 화질은 후속 통합 검증 대상으로 남는다.

## 다음 방향

게임별 저장, gfx 실행, Session routing이 연결됐다. 다음은 플랫폼별 기본 shader를 registry의 안정적인
계약으로 추가해 key 부재만 기본값을 상속하고 explicit Off는 계속 plain draw가 되게 하는 단계다.
Display 메뉴와 App의 즉시 변경·저장은 그 기본값까지 확정한 뒤 연결하는 것이 적절하다.
