# Task 46 최종 검토 — 게임별 화면 배율 앱 연결

## 판정

**성공** — 누적 호출 2/2.

## 최종 계약 확인

- 인게임 Display 행은 활성 세션 카트의 정확한 `Option<ScaleMode>`로 `DisplayMenu`를 연다.
- 위·아래 이동은 설정을 바꾸지 않고, B/MENU는 같은 Display 행으로 돌아간다.
- A는 전체 게임 설정에서 scale만 교체해 저장하며, 성공한 뒤에만 실행 중 Session에 같은 정책을
  즉시 적용한다. 메뉴와 세션, 오디오 sink는 유지되고 코어 프레임은 진행하지 않는다.
- Platform default는 다른 알려진 설정과 알 수 없는 ini 키를 보존하면서 scale override만 지운다.
  scale-only 파일은 App 커밋 직후 제거되고 Session은 시작 시점과 같은 기본 정책을 쓴다.
- 원자적 쓰기 실패는 기존 카드 설정과 실행 중 배율을 모두 유지하고, 시도한 행에 머물면서
  영·한 오류 토스트를 표시한다.
- 마지막 게임 프레임을 현재 배율로 먼저 그리고 Display overlay를 얹으며, 벽지·HUD·부모 메뉴·
  스위처는 끼어들지 않는다.
- `Card::write_settings`는 빈 설정 파일 삭제 실패를 더 이상 성공으로 보고하지 않는다.

## 2/2 보완 확인

- 행 순서와 양끝 순환 테스트의 잘못된 기대값을 바로잡았다.
- Platform default 테스트가 저장소 직접 정리로 결과를 가리지 않고 App 동작 직후 파일 제거를
  검사한다.
- 쓰기 실패 테스트는 읽을 수 있는 원본 ini를 유지한 채 `<game.ini>.tmp`만 디렉터리로 막아
  atomic write를 실패시키고, 카드 설정과 Session 배율 불변을 함께 검사한다.
- 설정 파일 경로가 디렉터리여서 제거할 수 없을 때 `write_settings`가 `Err`를 반환하는 저장소
  회귀 테스트를 추가했다. 결함 상태에서 이 테스트가 실패하는 음성 대조도 확인됐다.

## 작업자 검증 증거

- `cargo test -p slot2 --test display_menu_app`: 종료 코드 0, 8 passed / 0 failed
- `cargo fmt --all -- --check`: 종료 코드 0
- `cargo test -p slot2-store -p slot2 -p slot2-ui -p slot2-i18n`: 종료 코드 0, 0 failed
- `cargo clippy -p slot2-store -p slot2 -p slot2-ui -p slot2-i18n --all-targets -- -D warnings`:
  종료 코드 0
- 최종 검증 뒤 코드 변경 없음

충분한 작업자 증거가 있어 동일 테스트는 다시 실행하지 않았다. 관련 파일의
`git diff --check`는 종료 코드 0이다.

## 남은 위험과 후속

- `read_settings`가 읽기 실패를 빈 설정으로 처리하는 기존 정책은 유지된다. 카드 탐색을 막지
  않으려는 정책이지만 설정 편집 시 읽기 실패와 실제 기본값을 구분하지 못하므로 별도 저장소
  계약으로 다룬다.
- M4 인게임 메뉴에는 Cheats, Core, Device와 향후 shader/overlay 기능이 남아 있어 전체 항목은
  아직 완료 처리하지 않는다.
- 다음 태스크 지시서는 아직 작성하지 않았다.
