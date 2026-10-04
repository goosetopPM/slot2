# Task 99 — Codex 최종 판정

## 판정

**통과 (누적 호출 1/2).** 추가 작업자 호출은 필요하지 않다.

## 확인한 내용

- 기존 MIT `insert.pcm`/`eject.pcm`은 수정하지 않고 `SoundProfile { speed, gain }`을
  `PlatformSkin::sfx_in`/`sfx_out`에 연결했다. 7개 플랫폼의 삽입 프로필과 배출 프로필은 각각
  pairwise distinct이며 같은 플랫폼의 두 방향도 다르다.
- `Sfx::render_styled`는 기존 resampler의 정수 source rate와 gain만 사용한다. 실제 render와
  `lead_at_speed`/`seconds_at_speed`/`tail_at_speed`가 같은 유효 속도를 사용하므로 접점 cue와 sample
  길이가 어긋나지 않는다.
- 정상 sink rate의 base `render`, 원본 lead/duration, stereo center 계약은 유지된다. rate 0 또는
  384 kHz 초과와 잘못된 speed/gain은 빈 출력으로 거부해 과대 allocation과 divide-by-zero를 막는다.
- App은 현재 플랫폼 skin을 한 번 조회해 방향별 profile로 clip을 이벤트당 한 번 render하고 ring에
  전부 기록한다. insert는 변형된 lead로 `SEATED_AT`에서 역산하고 eject는 첫 frame에 시작한다.
- 가장 느린 NES insert tail은 seated hold 안에 끝나고 NES eject도 `EJECT_S` 전에 끝난다. 가장 빠른
  MD를 포함한 App 통합 테스트가 실제 ring sample과 각 profile의 expected render를 대조한다.
- provenance와 DESIGN은 원본 PCM 보존, runtime speed/gain 변형, transformed lead를 현재 구현대로
  설명한다. M6의 cart/port/curve/sfx 복합 항목은 완료됐다.
- Codex 검토 중 M3 항목에 남아 있던 “효과음은 남음” 문구 한 줄을 Task99 완료 상태로 바로잡았다.
  문서만 수정했으며 code/test 동작은 바뀌지 않았다.

## 검증 근거

- `slot2-audio --test sfx`: **14 passed / 0 failed / 0 ignored**
- `slot2-ui --test skin`: **26 passed / 0 failed / 0 ignored**
- `slot2 --test sfx_app`: **11 passed / 0 failed / 0 ignored**
- `slot2 --test insert_app`: **16 passed / 0 failed / 0 ignored**
- workspace 전체: **971 passed / 0 failed / 0 ignored**
- device check, fmt, workspace clippy: 모두 종료 코드 0, clippy warning 0
- device 배포: 종료 코드 0, 마지막 줄 `==> done`, `dist-device/System/frontend` 생성

위 수치는 작업자 보고서의 최종 code/test 변경 후 실행 결과다. Codex는 프로젝트 규칙에 따라 테스트를
다시 실행하지 않고 구현, profile table, App 배선과 계약 테스트를 대조했다. 이후 Codex 변경은 오래된
MILESTONES 문구 한 줄 정정뿐이다.

## 남은 범위

- 실기에서 7개 플랫폼의 삽입·배출 음색 차이와 접점 동기, tail 단절·pop/click 여부 확인
- M6의 640×480/720×720 실기, 스틱 기기 입력 프로필, lid close/open, 고부하 프레임 페이싱 검증
