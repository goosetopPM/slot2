# Task 47 워커 결과 — 읽을 수 없는 게임 설정 덮어쓰기 거부

**성공. 누적 호출 1/2.** 지정 검증 4종 모두 종료 0.

## 프로덕션 변경 (1곳)

`crates/slot2-store/src/settings.rs`의 `Card::write_settings`가 기존 ini를 손실 fallback 없이
읽는다(`Ini::load(&path)?`). 기존 설정 파일을 읽을 수 없으면(무효 UTF-8, 경로에 디렉터리 등)
파일을 건드리기 전에 `Error::Io`를 반환한다 — 임시 파일 생성·target 재작성·부분 적용·미지 키
폐기가 모두 일어나지 않는다. 읽기 정책은 그대로 관대해서 `read_settings`는 누락·무효 UTF-8·
디렉터리 모두 기본값을 돌려주고, 선반 스캔과 실행은 영향을 받지 않는다.

성공 동작은 모두 보존된다: 파일이 없으면 정상 시작점(기본값 쓰기는 no-op 성공), known 필드
갱신, 미지 키 보존, 소유 키 제거, 실제로 빈 ini 삭제, 삭제 오류 전파, 동일 바이트 원자적 쓰기
no-op. `GameSettings`·`ScaleMode`·`Ini`·`Error`·`read_settings`의 공개 형태는 바꾸지 않았다.
Task 46의 App 경로는 수정하지 않았고, 새 오류에서도 실패 토스트·시도 행 유지·세션 배율 불변을
그대로 지킨다(아래 App 회귀 테스트로 확인).

## 추가한 회귀 테스트

- `crates/slot2-store/tests/card.rs`: (1) 무효 UTF-8 ini는 `read_settings`가 기본값, (2) 그
  파일에 비기본 설정을 쓰면 `Err`이고 바이트가 정확히 그대로이며 임시 파일도 남지 않음,
  (3) 기본값 clear도 `Err`에 바이트 불변, (4) 설정 파일 자리에 디렉터리를 두면 비기본 쓰기와
  기본 clear 모두 `Err`이고 디렉터리가 남음, (5) 파일이 없으면 비기본 쓰기 성공 후 기본값으로
  돌아가면 파일이 사라짐. 기존 라운드트립·미지 키·삭제 오류 테스트는 손대지 않았다.
- `crates/slot2/tests/display_menu_app.rs`: 무효 UTF-8 ini는 관대한 읽기 정책으로 Platform
  default 행에서 열리고, 다른 배율에서 A를 누르면 `display-save-failed`가 뜨고 시도 행이 남고
  세션 정책이 실행 시점 그대로이며 파일 바이트가 변하지 않는다.
- 역방향 확인: 손실 fallback을 되돌려 실행하면 위 store 테스트와 App 테스트가 각각 실패했다
  (의미 있는 회귀 테스트임을 확인). 디렉터리 경로는 이전에도 atomic write 단계에서 `Err`였다.

## 최종 검증 (명세 순서, 최종 코드 상태)

- `cargo fmt --all -- --check` → 종료 **0**(신규 테스트 포맷 정리 후).
- `cargo test -p slot2-store` → 종료 **0**: card 19 passed / state_undo 11 / lib 1, 0 failed.
- `cargo test -p slot2 --test display_menu_app` → 종료 **0**: 9 passed.
- `cargo clippy -p slot2-store -p slot2 --all-targets -- -D warnings` → 종료 **0**.
- 최종 검증 후 코드 변경 없음(설정 파일 sha1 1c194f4a… 확인). 워크스페이스 전체 테스트·dist
  빌드·Pi·실기 접근은 하지 않았다.

## 남은 우려

- `a_settings_file_that_cannot_be_removed_is_an_error`(Task 46)는 이제 디렉터리 경로에서 "읽기
  거부"로 `Err`가 나므로 `remove_file` 오류 분기를 직접 검증하지 못한다. 그 분기를 휴대용으로
  실패시키는 방법이 없어(권한 비트는 플랫폼별로 다름) 테스트는 그대로 두었다.
- 소요 약 4분(상한 45분 내).
