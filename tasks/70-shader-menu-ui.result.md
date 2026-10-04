# Task 70 — Codex 최종 판정

## 판정

**통과. 누적 호출 1/2.** 추가 작업자 호출은 필요하지 않다. Codex는 사용자 운영 규칙에 따라 검증
명령을 다시 실행하지 않고 작업자 보고서와 UI·번역·집중 테스트를 대조했다.

## 통과한 부분

- 새 enum 없이 `Option<ShaderPreset>`을 그대로 사용하며 Platform default, explicit Off와 네 effect를
  정확한 여섯 행으로 분리한다.
- constructor/query와 Up/Down wrap이 public 고정 배열의 같은 순서를 사용하고 key mapping은 exhaustive다.
- overlay는 clear하지 않고 dim, safe-area panel, title, rows, hints 순으로 그리며 정확히 한 행만
  highlight한다.
- 세 device geometry와 영문/한글에서 380×316 panel과 모든 UI가 640×480 safe area 안에 있다.
- 신규 문구는 두 language pack에 직접 정의됐고 warm redraw 및 highlight 이동은 glyph texture를 다시
  upload하지 않는다.
- App, Session, store, gfx, registry와 기존 scale-only DisplayMenu는 수정하지 않았다.

## 검증 근거

- 작업자 shader menu 집중 테스트: **7 passed / 0 failed / 0 ignored**.
- 작업자 `slot2-ui` + `slot2-i18n` 전체: **231 passed / 0 failed / 0 ignored**.
- 작업자 fmt와 두 crate all-target clippy: 종료 0.
- 최종 검증 뒤 production 변경이 없고 Codex의 관련 파일 `git diff --check`도 오류가 없다.
- 검토 중 NES 기본 effect를 일반어 `scanlines`로 부른 주석 두 곳만 정확한 `ZfastCrt` 계약으로 정정했다.
  실행 코드와 테스트 단언은 바뀌지 않았다.

## 다음 방향

Task71에서 기존 scale Display 화면에 Shader 진입 행을 추가하고 별도 Shader screen을 App에 연결한다.
선택은 카드에 먼저 안전하게 저장한 뒤 Session에 즉시 적용하며, 실패 시 runtime과 저장값을 모두
유지해야 한다.
