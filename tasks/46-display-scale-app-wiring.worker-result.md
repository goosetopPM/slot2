# Task 46 워커 결과 (누적 2/2) — 게임별 화면 배율 앱 연결

**최종: 성공. 누적 호출 2/2.** 2회차에서 지정 검증 3종이 모두 종료 0이고 집중 통합 테스트 8개가
통과했다. App/세션 구현은 1회차 그대로이며, 2회차에는 명세가 요구한 저장소 결함 1건과 테스트
기대값 3건만 고쳤다.

## 2회차 delta

- `crates/slot2/tests/display_menu_app.rs`
  - `navigation_wraps_and_closing_returns_to_the_same_row`: 실제 행 순서(Platform default,
    Integer, Aspect fit, Fill)로 수정. Integer에서 Up → Platform default, Up 한 번 더 → Fill
    wrap, Fill에서 Down → Platform default wrap, 다시 Integer로 내려온 뒤 B/MENU 복귀 확인.
    항해만으로는 카드 설정과 라이브 배율이 변하지 않음을 추가 단언. 복귀 후 재진입은 인게임
    메뉴에서 MENU 재탭(Playing으로 닫힘) 대신 Display 행에서 바로 A로 들어가도록 정리.
  - `platform_default_removes_only_the_scale_override`: Fill → Platform default를 Down으로 수정.
    App 커밋 직후 scale 부재, overscan/rewind 보존, 미지의 ini 키 보존, 라이브
    `Session::policy_for(None)`을 직접 단언. scale-only 파일 제거도 App의 A 커밋으로 검증하고
    결과를 가리던 저장소 직접 호출(`write_settings(default)`)을 삭제.
  - `a_write_that_fails_leaves_the_game_alone`: 카드 설정 폴더를 파일로 파괴하던 준비를 제거.
    Integer를 저장하고 ini가 읽히는 상태에서 메뉴를 열어 Integer로 시작한 뒤, `<arm.ini>.tmp`를
    디렉터리로 만들어 atomic write만 실패시킴. 실패 토스트, 시도 행(Aspect fit), 라이브 Integer
    정책, 카드 원본 Integer 유지, ini 파일 존재, Display 화면·세션 유지, 싱크 요청 없음을 단언하고
    종료 시 차단 디렉터리를 제거.
- `crates/slot2-store/src/settings.rs`: 완전히 빈 설정으로 돌아갈 때 `Card::write_settings`가
  `remove_file` 오류를 `Error::Io`로 전파한다(이전에는 `let _ =`로 삼키고 `Ok(())`). 파일이 없으면
  계속 성공 no-op이며, 미지 키 보존과 일반 원자적 쓰기는 그대로다.
- `crates/slot2-store/tests/card.rs`: 설정 파일 자리에 디렉터리를 두고
  `write_settings(..., GameSettings::default())`가 `Err`를 내도록 요구하는 휴대용 회귀 테스트를
  추가(기존 성공 제거 테스트는 유지). 차단 디렉터리를 치운 뒤에는 다시 성공 no-op임도 확인한다.
- 이 저장소 테스트가 실제 결함을 잡는지 확인하려고 `write_settings`를 이전 동작으로 되돌려 1회
  실행했고 `Err` 단언이 실패(종료 101)했다. 복원 후 mtime을 갱신해 낡은 아티팩트 재사용을 없앤
  뒤 통과를 확인했고, 최종 검증은 아래 최종 상태에서 다시 돌린 결과다.

## 검증 (최종 코드 상태, 명세 순서)

- 집중: `cargo test -p slot2 --test display_menu_app` → 종료 **0**, 8 passed / 0 failed. 수정된
  테스트가 구현 결함을 드러내지 않아 `app.rs`·`session.rs`는 2회차에 손대지 않았다.
- `cargo fmt --all -- --check` → 종료 **0**(신규 테스트 2곳의 포맷만 정리).
- `cargo test -p slot2-store -p slot2 -p slot2-ui -p slot2-i18n` → 종료 **0**, 41개 결과 줄 전부
  0 failed(`display_menu_app` 8, `card` 16 포함).
- `cargo clippy -p slot2-store -p slot2 -p slot2-ui -p slot2-i18n --all-targets -- -D warnings`
  → 종료 **0**.
- 최종 검증 후 코드 변경 없음(설정 파일 sha1로 확인). 워크스페이스 전체 테스트·dist 빌드·Pi·실기
  접근은 실행하지 않았다.

## 남은 우려

- `read_settings`는 읽기 실패를 계속 빈 설정으로 뭉갠다(카드를 못 읽어도 선반이 떠야 함). 설정
  파일 자리에 디렉터리가 있으면 메뉴가 Platform default로 열리는데 이는 계약 범위 밖이다.
- 커밋·푸시 없음, 다른 태스크의 미커밋 변경은 그대로 두었다.

## 누적 동작 (1회차 구현 유지)

- 인게임 메뉴 Display 행 A → 세션 카트의 `read_settings().scale`로 `DisplayMenu`를 만들어
  `Screen::Display(parent, menu)` 진입. 세션 없으면 무동작·무 I/O.
- Up/Down wrap, B/MENU는 같은 Display 행으로 복귀. A는 전체 `GameSettings`를 읽어 `scale`만
  교체해 저장하고, 성공한 뒤에만 `Session::policy_for(choice)`를 적용하며 메뉴는 열린 채 유지.
- Platform default는 scale만 지우고(마지막 값이면 ini 파일 제거) 실행 시점과 같은 정책을 적용.
- 실패 시 세션·카드 불변, 시도 행 유지, 오류 1줄 로그, `display-save-failed` 토스트.
- 서브메뉴 동안 코어 정지·오디오 일시정지·싱크 요청 없음. 그리기 순서 게임 프레임 → dim/메뉴 →
  홀드 바·토스트, 벽지·HUD·부모 메뉴·스위처는 그리지 않음.
- 변경 파일(누적): `crates/slot2/src/app.rs`, `crates/slot2/src/session.rs`,
  `crates/slot2/tests/display_menu_app.rs`, `assets/lang/{en,ko}.ftl`,
  `crates/slot2-i18n/tests/i18n.rs` + 2회차 `crates/slot2-store/src/settings.rs`,
  `crates/slot2-store/tests/card.rs`.
- 2회차 소요 약 14분(상한 45분 내).
