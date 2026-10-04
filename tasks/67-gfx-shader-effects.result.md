# Task 67 — Codex 최종 판정

## 판정

**통과. 누적 호출 1/2.** 추가 작업자 호출은 필요하지 않다. Codex는 사용자의 운영 규칙에 따라
검증 명령을 다시 실행하지 않고 작업자 보고서, shader source, Canvas 계약과 GL program 수명 코드를
대조했다.

## 통과한 부분

- gfx 전용 공개 `ShaderEffect`가 SharpBilinear, Lcd3x, ZfastCrt, Scanline 네 variant와 고정 순서를
  제공한다. store 의존이나 renderer의 가짜 Off program은 추가하지 않았다.
- Canvas effect draw가 texture, destination, UV crop, tint와 effect를 한 호출로 전달한다. 지원하지 않는
  Canvas는 동일한 일반 image draw로 fallback하고 RecordingCanvas는 별도 op로 계약 전체를 기록한다.
- effect는 일반 batch를 먼저 flush한 뒤 quad 하나에만 적용되고 즉시 기본 program으로 돌아간다.
  HUD·메뉴·글자·rect와 최종 present에는 shader state가 전파되지 않는다.
- 네 source는 GLES2 단일 패스 범위에서 서로 다른 실제 연산을 수행한다. source texture 크기,
  destination 크기와 crop UV를 사용하며 neighbor sample은 crop 경계를 넘지 않는다.
- 기본 program compile/link 실패는 canvas 생성 실패로 유지한다. 선택 effect 하나의 실패는 log를
  보존하고 해당 draw만 일반 image로 fallback한다.
- 공통 compile/link helper가 vertex·fragment·program object를 성공과 각 실패 경로에서 정리한다.
  Drop도 실제로 생성된 effect program만 한 번씩 삭제한다.
- Task66의 store enum과 gfx enum은 서로 의존하지 않으며 변환은 후속 Session 경계로 남았다.

## 검증 근거

- 작업자 `cargo fmt --all -- --check`: 종료 0.
- 작업자 `cargo test -p slot2-gfx`: 기본 실행 **25 passed / 0 failed**. 환경변수가 없어 screenshot의
  GL body는 이 완료 기준 실행에서는 skip됐다.
- 작업자가 별도로 `SLOT2_GFX_TEST=1`을 사용해 Windows Intel OpenGL ES 2.0 context에서 GL test
  **1 passed**를 확인했다. 네 effect가 모두 compile/link됐고 base 및 effect 상호 간 영상 차이,
  후속 일반 draw 불변과 UV crop sentinel 격리가 통과했다.
- 작업자가 임시 probe로 effect의 compile 실패와 link 실패를 각각 유발해 canvas 생존, 나머지
  effect 정상 동작, 실패 effect의 base와 byte-identical fallback을 확인한 뒤 probe를 제거했다.
- 작업자 device feature check, downstream `cargo check -p slot2 --tests`, gfx clippy가 모두 종료 0이다.
- 최종 검증 뒤 코드 변경이 없고 Codex의 범위 파일 `git diff --check`도 오류가 없다.

## 남은 한계

Mali G31에서의 compile과 실제 패널 화질·성능은 아직 확인되지 않았다. 이번 태스크는 x86 Windows의
Intel GLES2까지 검증했으며, 실기 검증은 사용자 단계로 남긴다. 후속 Session 배선은 shader에 전체
texture 크기와 현재 crop UV를 그대로 넘겨야 neighbor 위치가 한 texel 밀리지 않는다.

## 다음 방향

저장 형식과 gfx 실행 기반이 준비됐다. 다음은 Session에 store preset → gfx effect 변환을 한 곳에 두고,
게임 frame draw만 effect API로 전환하는 배선이다. 이 단계에서는 플랫폼 기본값을 아직 정하지 않고
명시적 게임 override와 Off를 먼저 실행 중에 안전하게 적용하는 것이 적절하다.
