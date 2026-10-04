# Task 86 — Codex 최종 판정

## 판정

**통과 (누적 호출 1/2).** 두 번째 작업자 호출은 필요하지 않다.

## 확인 내용

- 선반 가용성은 private `App::shelf_availability()` 한 곳에서 TimeZone과 About만 활성화한다.
- `Screen::About(ShelfMenu, AboutSticker)`가 parent의 About 선택을 보존하며 `Screen`의 `Copy` 계약을
  유지한다.
- Shelf A가 TimeZone과 About을 명시적으로 분기하고 나머지 비활성 choice는 no-op이다.
- About에서 B/Menu만 parent Shelf로 돌아가고 A·방향 입력은 screen, clock, volume, 파일, toast를
  바꾸지 않는다.
- About는 List와 같은 wallpaper+shelf 위에 단독으로 그려지며 parent Shelf/Timezone panel이
  중복되지 않는다.
- draw 시 `slot2` package의 `env!("CARGO_PKG_VERSION")`과 `ctx.profile.target`을 `AboutInfo`로
  직접 전달한다. App/Screen에 별도 문자열 복사본이 없다.
- HUD는 유지되고 About는 game audio pause 집합에 포함되지 않는다.
- 기존 시간대 통합 테스트는 availability 관련 첫 구역만 두 활성 행 계약으로 갱신됐고,
  preview/save/cancel/failure rollback 시나리오는 유지됐다.

## Codex 수정

production 동작은 변경하지 않았다. `app.rs`에서 “시간대만 열 수 있다”고 남아 있던 주석 두 곳을
현재 동작인 “시간대와 About을 열 수 있다”로 정정했다.

## 작업자 검증 증거

- `cargo fmt --all -- --check`: 종료 0
- `cargo test -p slot2 --test about_sticker_app`: **1 passed / 0 failed / 0 ignored**
- `cargo test -p slot2 --test timezone_menu_app`: **1 passed / 0 failed / 0 ignored**
- `cargo test -p slot2`: **315 passed / 0 failed / 0 ignored**, core-dependent skip 0
- `cargo check -p slot2 --no-default-features --features device`: 종료 0
- `cargo clippy -p slot2 --all-targets -- -D warnings`: 종료 0

Task 84 기준 314개에 신규 About App test 1개가 추가된 315개로 테스트 수가 일치한다. Codex는
프로젝트 규칙에 따라 테스트를 다시 실행하지 않고 보고서와 production·test 코드를 교차 검토했다.
최종 검증 뒤 Codex 변경은 주석 두 곳뿐이다.

## 남은 사항

- Language, DisplayDefaults, BootLogo, Sync 네 행은 비활성이다.
- 실제 기기에서 About 글자 크기와 `System/licenses` 경로 가독성 확인이 필요하다.
- M7의 라이선스 전문·코어 라이선스 요약은 별도 배포 작업으로 남아 있다.
