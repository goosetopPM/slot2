# Task 77 시도 2/2 — 오버레이 저장 의미 설명 수정

현재 checkout에서 직접 작업한다. Task77 구현과 테스트는 유지하고, runtime 계약과 모순되는 설명 두 곳만
바로잡는다. 이것이 **마지막 호출 2/2**다. 실패해도 세 번째 시도를 하지 않는다.

## 먼저 읽을 범위

- `C:\SLOT2\AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `C:\SLOT2\tasks\77-overlay-menu-ui-attempt2.md`
- `C:\SLOT2\crates\slot2-ui\src\overlay_menu.rs`의 파일 상단 module 문서만
- `C:\SLOT2\tasks\77-overlay-menu-ui.worker-result.md`의 계약 의문/남은 위험 부분만
- 의미 확인이 필요하면 `C:\SLOT2\crates\slot2\src\overlay.rs`의 `overlay_enabled` 함수만

다른 production/test 파일, 태스크·로그와 저장소 이력은 읽지 않는다.

## 수정 계약

- 현재 production 계약은 정확히 다음과 같다.
  - `None`: 플랫폼 기본값 상속. 현재 기본값은 off이므로 오버레이를 그리지 않는다.
  - `Some(true)`: 명시적 on. 현재 플랫폼 기본값과 관계없이 해당 geometry의 유효한 카드/내장 PNG가
    있으면 오버레이를 그린다. asset이 없거나 깨졌으면 안전하게 그리지 않는다.
  - `Some(false)`: 명시적 off. 오버레이를 그리지 않는다.
- `overlay_menu.rs` 상단의 “`None` and `Some(true)` both draw nothing today” 설명을 위 계약에 맞게
  고친다. 세 값을 별도 행으로 유지하는 이유도 정확히 남긴다.
- 작업자 보고서의 “`None`과 `Some(true)`가 현재 플랫폼에서는 화면상 같은 결과”라는 남은 위험도 같은
  계약에 맞게 고친다. 후속 App이 `selected()`의 `Option<bool>`을 그대로 저장해야 한다는 결론은 유지한다.
- runtime 코드, UI 동작, 번역, 테스트와 다른 주석은 변경하지 않는다. 잘못된 설명을 고치기 위해 동작을
  설명에 맞춰 바꾸지 않는다.

## 수정 허용 범위

- `C:\SLOT2\crates\slot2-ui\src\overlay_menu.rs`의 상단 module 문서 주석만
- `C:\SLOT2\tasks\77-overlay-menu-ui.worker-result.md`

다른 파일은 수정하지 않는다. 관련 없는 미커밋 변경을 정리·복원·재포맷하지 않는다.

## 범위 밖 및 금지

- UI 구조, rows, navigation, draw와 public API 변경
- App/Display/runtime overlay/store/Session 변경
- 번역과 테스트 변경
- 실제 PNG나 built-in entry 추가
- 전체 workspace 테스트, GL 창, device 배포, 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- 공용 GJC/BAI/OpenCodex/Codex 설정 변경
- 위임, 재귀 태스크 생성, 커밋, 푸시

## 완료 기준

마지막 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-ui --test overlay_menu
cargo clippy -p slot2-ui --all-targets -- -D warnings
```

모두 종료 0이어야 한다. `rg`로 `overlay_menu.rs`와 작업자 보고서에 잘못된 두 문구가 더는 남지 않고,
`Some(true)`의 asset 조건부 enabled 의미가 정확히 적혔는지도 확인한다. 검증 뒤 파일을 바꾸면 영향받는
명령부터 다시 실행한다.

## 결과 보고서

기존 `C:\SLOT2\tasks\77-overlay-menu-ui.worker-result.md`를 **누적 2/2 보고서**로 갱신한다.

- 최종 성공/실패와 누적 호출 2/2
- 고친 두 설명과 최종 세 상태의 정확한 의미
- UI 코드·번역·테스트 동작 무변경 확인
- 각 완료 기준 명령, 종료 코드와 마지막 결과 줄
- 수정 파일 및 최종 검증 뒤 변경 여부
- 남은 위험 또는 계약이 틀려 보이는 부분 1줄
- 이번 호출 및 누적 소요 시간

코드·로그 전문이나 명세를 반복하지 않는다. 첫 응답은 최대 5분, 전체는 최대 45분 기다린다. 실패해도
보고서를 갱신하고 더 시도하지 않는다.
