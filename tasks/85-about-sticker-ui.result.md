# Task 85 — Codex 최종 판정

## 판정

**통과 (누적 호출 1/2).** 두 번째 작업자 호출은 필요하지 않다.

## 확인 내용

- `AboutInfo<'a>`는 호출자가 주는 `version`과 `target`만 담고, `AboutSticker`는 상태 없는 `Copy`
  unit struct다.
- UI 크레이트 version, host OS, 환경변수, git SHA를 읽거나 추측하지 않는다.
- 동적 두 값은 `about-version`, `about-target`의 Fluent argument로 전달되며 Rust 문자열 조합이 없다.
- 화면은 title, SLOT2 wordmark, version, target, `SLOT2 · MIT`, `System/licenses`의 코어·글꼴
  고지 위치, 뒤로 힌트를 표시한다. 아직 완성되지 않은 M7 라이선스 전문 제공을 약속하지 않는다.
- draw는 clear 없이 physical panel dim 뒤 safe-area 중앙 panel을 그리고 실제 측정 폭으로 각 줄을
  배치한다.
- 영문·한글 여섯 key가 각 built-in pack에 직접 정의돼 있고 기존 `shelf-about`은 유지된다.
- 세 실제 geometry, 두 언어, 긴·빈 입력, off-centre 합성 safe area와 warm redraw가 테스트로
  봉인돼 있다.

## 작업자 검증 증거

- `cargo fmt --all -- --check`: 종료 0
- `cargo test -p slot2-ui --test about_sticker`: **8 passed / 0 failed / 0 ignored**
- `cargo test -p slot2-ui -p slot2-i18n`: **279 passed / 0 failed / 0 ignored**
- `cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings`: 종료 0

Codex는 프로젝트 규칙에 따라 테스트를 다시 실행하지 않고 작업자 보고서와 production·test 코드를
교차 검토했다.

## 남은 사항

- About 행을 시간대와 함께 활성화하는 Shelf availability
- `Screen::About`과 A 진입, B/Menu 복귀, draw 연결
- App package version과 현재 device target을 `AboutInfo`로 전달
- 실제 기기에서 글자 크기와 고지 경로 가독성 확인

호출자는 compile-time package version과 등록된 짧은 target id를 전달할 예정이므로 보고서의
100자 이상 임의 입력 위험은 현재 production 경로에 해당하지 않는다. 위 App 배선은 Task 86 범위다.
