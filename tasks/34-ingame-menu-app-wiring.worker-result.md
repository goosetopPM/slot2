# Task 34 결과 — 인게임 메뉴를 앱에 배선

2026-09-25 19:07~19:25 (로컬, 약 18분). 판정: **성공.**

## 변경 파일

- `crates/slot2/src/app.rs` — `Screen::InGame(InGameMenu)` 추가, MENU 탭/길게 누르기 분기,
  InGame 입력 처리, 사운드 일시정지 정책 메서드 `App::audio_paused()`, InGame 그리기 경로,
  상태 머신 모듈 주석 갱신, 그리고 새 앱 테스트 6개.
  - 벽지는 `Screen::Playing`과 `Screen::InGame` 모두에서 그리지 않는다.
  - InGame 그리기는 세션의 마지막 프레임(`upload_video` + `draw`) 후 메뉴 오버레이만 그린다.
    HUD 조건에는 InGame을 넣지 않았다(게임이 화면의 주인이라는 기존 규칙 유지).
  - `run_frame`의 `screen != Playing` 가드는 그대로 두었다. 가짜 pause API를 만들지 않았다.
- `crates/slot2/src/host_app.rs`, `crates/slot2/src/device_app.rs` — 싱크 pause 판단을
  `matches!(app.screen, Screen::Power(_))`에서 `app.audio_paused()`로 바꿨다. 화면 정책이
  앱 한 곳에만 있고 두 루프가 같은 질문을 한다.
- `crates/slot2/tests/ingame_menu_app.rs` — 신규 통합 테스트 2개(실제 코어 사용).
- `tasks/34-ingame-menu-app-wiring.worker-result.md` — 이 보고서.

`slot2-ui`, `slot2-input`, `slot2-audio`, `Session`, 언어 자산, 매니페스트, 문서는 건드리지
않았다. 기존 테스트를 약화·수정하지 않았고, 기존 테스트 파일의 헬퍼 변경도 필요하지 않았다.

## 최종 화면 전환과 사운드 규칙

```text
Screen::Playing
  Tap(Menu)   → Screen::InGame(InGameMenu::default())   매번 새 기본값(첫 행 Continue)
  Hold(Menu)  → 세션 정지 + Screen::Ejecting             기존 이젝트 그대로
Screen::InGame(menu)
  Tap(Up/Down)→ menu.up()/down(), 양방향 wrap
  Tap(A)      → Continue : Screen::Playing (세션 유지)
                Eject    → stop_session() + Screen::Ejecting (anim=0, sfx_fired=false)
                나머지 5행 → 메뉴 유지(아무 일도 없음)
  Tap(B|Menu) → Screen::Playing (세션 유지)
```

- 탭과 길게 누르기는 충돌하지 않는다. `Gestures`는 릴리스에서 탭을, 600ms에서 홀드를 한 번
  내보내고 홀드 뒤 릴리스는 아무것도 내보내지 않는다. 그래서 승리 패턴을 먼저 두었다.
- Power/볼륨은 `match` 앞쪽의 전역 분기라 InGame에서도 기존 동작을 유지한다.
- 사운드 규칙: `App::audio_paused()`가 `Screen::Power`와 `Screen::InGame`에서 true, 그 외
  false. 두 루프가 이 한 메서드만 본다. 게임이 멈춰 있는 동안은 코어가 소리를 만들지 않고
  싱크도 소비하지 않으므로 링에 쌓이는 것은 없다.
- `frame_time()`은 메뉴가 열려 있는 동안 1/60을 유지한다(코어가 전진하지 않으므로 메뉴는
  UI 속도로 그린다).
- 이젝트 순서: `stop_session()`이 먼저 `SinkRequest::Close`를 만들고, 다음 tick에서 이젝트
  클립이 자기 링을 열어 `SinkRequest::Open`을 만든다. MENU 홀드 경로와 동일하다.

## 검증

마지막 코드 변경 뒤 순서대로 실행했다.

1. `cargo fmt --all -- --check` — 종료 **0**, diff 없음.
2. `cargo test -p slot2 -p slot2-ui` — 종료 **0**, 실패 0.
   - `slot2` lib 13개 통과(기존 7 + 신규 6: `a_menu_tap_over_a_game_opens_the_menu_and_a_hold_still_ejects`,
     `up_and_down_walk_the_menu_and_wrap`,
     `closing_the_menu_returns_to_the_game_and_leaves_the_session_alone`,
     `the_rows_with_nothing_behind_them_stay_in_the_menu`,
     `menus_pause_the_audio_and_a_running_game_does_not`,
     `drawing_the_in_game_menu_neither_clears_nor_paints_the_shelf_under_it`).
   - `tests/ingame_menu_app.rs` 2개 통과: 실제 mgba 코어로 arm.gba를 넣어 Playing까지 간 뒤,
     메뉴가 열린 30프레임 동안 `frames_run()`이 증가하지 않음, 게임 프레임 이미지 → 메뉴 dim
     순서와 `Op::Clear` 없음, B로 복귀 후 세션 유지·싱크 요청 없음·프레임 재개, 그리고 Eject가
     `Ejecting` + 세션 없음 + `SinkRequest::Close`를 만들고 `EJECT_S` 뒤 `List`로 돌아옴을 확인.
   - 기존 `slot2`/`slot2-ui` 통합 테스트 전부 통과(회귀 없음).
3. `cargo clippy -p slot2 -p slot2-ui --all-targets -- -D warnings` — 종료 **0**, 경고 0.

최종 검증 이후 코드 변경 없음(마지막 변경은 4번째 테스트 항목 추가였고 그 뒤 fmt→test→clippy를
다시 돌렸다). 워크스페이스 전체 테스트와 `build/dist-device.ps1`은 실행하지 않았다.

## 남은 항목과 계약 의견

- Save State/Cheats/Display/Core/Device는 계약대로 아무 화면도 열지 않는다. 이 5행은 지금
  "반응 없음"이 정답이며, 다음 태스크가 각각의 실제 화면을 붙일 자리다.
- 메뉴 안에서 MENU를 길게 눌러도 아무 일도 일어나지 않는다. 계약은 탭만 닫기로 정했고,
  홀드로 이젝트까지 하면 실수로 게임이 꺼질 수 있어 그대로 두었다. 필요하면 별도 결정이 필요하다.
- `Inserting`/`Ejecting` 중에는 메뉴가 열리지 않는다(기존 규칙 유지, 계약도 요구하지 않음).
- 게임 위 메뉴는 벽지·HUD를 그리지 않지만, 메뉴가 열린 동안 하단에 홀드 진행 바는 여전히
  그려질 수 있다(그 위를 메뉴가 덮는다). 동작 문제는 없으나 화면 설계상 남은 사소한 항목이다.
- 실기 검증은 하지 않았다(계약 범위 밖, 사용자 몫).
- 실행 시간 약 18분. 모델 토큰·비용 정보는 이 도구가 노출하지 않아 `unavailable`.
