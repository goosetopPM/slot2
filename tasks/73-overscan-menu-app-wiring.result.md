# Task 73 — Codex 최종 판정

- 판정: **통과**
- 누적 호출: **1/2**
- 작업자 보고서: `tasks/73-overscan-menu-app-wiring.worker-result.md`

## 검토 결과

- Display 메뉴는 registry의 overscan이 `NONE`이 아닌 플랫폼에서만 Overscan 행을 제공한다.
  비-crop 플랫폼은 기존 5행/280 높이, crop 플랫폼은 6행/316 높이를 사용하며 그 한 행 목록을
  선택·이동·draw가 함께 참조한다.
- App은 부모 InGame/Display 메뉴를 보존한 별도 Overscan screen을 사용한다. 저장할 전체
  `GameSettings`에서 overscan만 바꾸고, 카드 쓰기가 성공한 뒤에만 Session crop을 적용한다.
  실패 시 카드와 runtime crop을 보존하고 `overscan-save-failed` toast를 표시한다.
- Session의 단일 `overscan_for` 경계가 `None`과 `Some(true)`를 플랫폼 기본 crop으로,
  `Some(false)`를 full image로 해석한다. 시작 경로와 runtime 변경 경로가 같은 해석을 쓴다.
- 합성 NES/FCEUmm 검증은 256x240 원본에서 default/explicit crop의 UV
  `[0, 8/240, 1, 232/240]`와 256x224 visible size, full image의 `[0, 0, 1, 1]`와
  256x240 visible size를 직접 단언한다. GBA의 기존 5행, scale/Shader 흐름, 저장 실패 rollback도
  함께 고정돼 있다.
- 코드 검토 후 `git diff --check` 종료 코드 0을 확인했다. 줄바꿈 변환 경고만 있었고 whitespace
  오류는 없었다.

## 작업자 검증 증거

- `cargo fmt --all -- --check`: 종료 0
- `cargo test -p slot2-ui --test display_menu`: 7 passed / 0 failed
- `cargo test -p slot2 --test session --test display_menu_app`: 71 passed / 0 failed
- `cargo test -p slot2 -p slot2-ui -p slot2-i18n`: 492 passed / 0 failed / 0 ignored
- `cargo clippy -p slot2 -p slot2-ui -p slot2-i18n --all-targets -- -D warnings`: 종료 0
- core-dependent skip: 0

## 남은 확인

- 실제 RG SP/Mali G31에서 crop 경계와 화질은 실기 단계에서 사용자가 확인해야 한다.
- Overlay 선택은 아직 runtime/store 계약이 없어 후속 범위다.
