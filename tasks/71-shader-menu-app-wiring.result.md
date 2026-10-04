# Task 71 — Codex 최종 판정

## 판정

**통과. 누적 호출 1/2.** 추가 작업자 호출은 필요하지 않다. Codex는 사용자 운영 규칙에 따라 검증
명령을 다시 실행하지 않고 작업자 보고서와 Display/Shader UI, App 상태 전이·저장 코드와 집중 테스트를
대조했다.

## 통과한 부분

- `DisplayChoice`가 네 scale 선택과 Shader 진입을 타입으로 구분하고, 기존 scale constructor와 저장
  흐름을 유지한다. 다섯 행의 navigation, label, 280 높이 safe-area도 검증됐다.
- `Screen::Shader`가 parent InGame/Display menu를 보존해 B/MENU가 같은 Shader 행으로 돌아간다.
- Shader A는 전체 settings에서 shader만 바꾸고 카드 write 성공 뒤에만 Session effect를 적용한다.
- runtime 변환은 기존 `Session::shader_effect_for`만 사용한다. App에 store/retro/gfx mapping 사본이 없다.
- GBA의 key 부재는 Lcd3x, explicit Off는 plain, 네 preset은 동명 effect로 정확히 저장·적용된다.
- Platform default 복귀는 shader key만 제거하며 shader-only ini도 제거한다. 다른 known field와 unknown
  key는 set/clear 양쪽에서 보존된다.
- write 실패와 unreadable 원본은 카드 bytes와 prior runtime effect를 유지하고 attempted row에 남아
  localized failure toast를 표시한다.
- Shader 화면은 core/audio를 pause하고 sink와 texture 수명을 바꾸지 않는다. game frame을 현재 effect로
  먼저 그리고 Shader overlay만 올리며 parent menu, wallpaper, HUD와 clear가 끼지 않는다.

## 검증 근거

- 작업자 Display UI 집중 테스트: **6 passed / 0 failed / 0 ignored**.
- 작업자 App 집중 테스트: **19 passed / 0 failed / 0 ignored**, core-dependent skip 0.
- 작업자 `slot2` + `slot2-ui` + `slot2-i18n` 전체: **472 passed / 0 failed / 0 ignored**.
- 작업자 fmt와 세 crate all-target clippy: 종료 0.
- 최종 검증 뒤 코드 변경이 없고 Codex의 관련 파일 `git diff --check`도 오류가 없다.

## 다음 방향

게임별 셰이더 선택·저장·즉시 적용까지 완료됐다. Display 설계에서 남은 항목은 overlay와 overscan
selector지만 overlay는 아직 runtime/store 계약이 없다. 다음은 이미 registry와 Session에 존재하는
overscan을 게임별 UI에 연결할지, M4의 다른 미완 항목으로 이동할지 범위를 정하는 단계다.
