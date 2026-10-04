# Task 43 최종 판정 — 통과

2026-09-26 Codex 검토 결과, 작업자 보고서와 실제 변경이 일치하고
`tasks/43-time-control-hud-ui.md`의 UI 컴포넌트 계약을 충족했다. 누적 호출은 1/2다.

## 구현 확인

- `slot2_ui::hud::TimeControl`은 `Rewind`와 `FastForward { speed: u32 }`만 표현하며 App/input
  타입에 의존하지 않는다.
- `Hud::draw_time_control`은 optional 값을 받고 `None`에서 아무것도 그리지 않는다. 기존 시계·
  배터리 `Hud::draw` API와 출력은 변경하지 않았다.
- 활성 값은 기존 dark plate와 light ink로 배지 하나만 그린다. clear, origin 변경, 시계·배터리·
  toast·게임 프레임을 그리지 않아 App이 화면과 합성 순서를 결정할 수 있다.
- 문구를 먼저 측정해 패널 폭 기준으로 가로 중앙에 놓고, `HUD_MARGIN`과 `HUD_H`를 재사용한다.
  `safe.x/y`를 사용하지 않아 720x720에서도 물리 패널 상단에 유지된다.
- FastForward 속도는 enum 값이 Fluent `$speed`로 전달된다. 영어는 `4×`/`8×`, 한국어는
  `4배속`처럼 렌더링되며 drawing 코드에 4가 고정되지 않았다.
- 기존 face cache를 재사용한다. 동일 배지의 warm draw는 새 업로드가 없고 다른 문구로 전환한
  뒤에도 한 번 정착하면 추가 업로드가 없다.
- 영어·한국어 팩이 rewind와 fast-forward 키를 직접 정의하며 fallback에 의존하지 않는다.

## 테스트와 검증

- HUD 신규 테스트 4개가 None 무출력, 배지 하나와 정확한 문구·대칭 padding·clear 없음,
  speed 4/8, 3기기×2언어 panel 배치, square panel의 safe-y 불추종, 반복/전환 캐시를 확인한다.
- 기존 HUD 테스트 15개가 그대로 통과해 clock/battery corner 배치와 동작이 유지됨을 확인했다.
- i18n 신규 테스트 1개가 두 키의 en/ko 직접 정의와 speed-4 결과를 확인한다.

작업자가 마지막 코드 변경 뒤 다음 명령을 순서대로 실행했고 이후 코드 변경이 없다고 기록했다.

- `cargo fmt --all -- --check`: 종료 0
- `cargo test -p slot2-ui -p slot2-i18n`: 종료 0, 실패 0; HUD 19개, i18n 22개 포함
- `cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings`: 종료 0
- Codex 관련 파일 `git diff --check`: 종료 0

증거가 충분하므로 Codex에서 동일 테스트를 중복 실행하지 않았다. App 배선은 계약대로 없으며
Task44에 남는다. 따라서 M4의 HUD 배지 마일스톤은 아직 완료 처리하지 않는다. 커밋·푸시·실기·
Pi·배포 빌드는 수행하지 않았다.
