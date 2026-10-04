# Task 62 — 시도 2/2: provider 400 이후 현재 작업 복구

이 작업은 Task62의 **누적 마지막 2/2 호출**이다. 시도 1은 대화형 요청이 약 2.13MB·622 messages까지
누적된 뒤 provider 400 `read body failed`로 중단됐다. 설정 문제나 `developer` role 문제는 아니며,
현재 working tree에는 구현 파일이 남아 있다.

## 작업 방법

1. `C:\SLOT2\AGENTS.md`의 안전·품질 규칙을 따른다. 역할 분리 조항만 무시한다.
2. `C:\SLOT2\tasks\62-device-menu-ui.md`를 완전한 원 계약으로 읽는다.
3. 아래 현재 파일과 그 diff만 확인한다.
   - `C:\SLOT2\crates\slot2-ui\src\device_menu.rs`
   - `C:\SLOT2\crates\slot2-ui\tests\device_menu.rs`
   - `C:\SLOT2\crates\slot2-ui\src\lib.rs`의 device export 부분
   - `C:\SLOT2\assets\lang\en.ftl`, `ko.ftl`의 device message 부분
   - `C:\SLOT2\crates\slot2-i18n\tests\i18n.rs`의 device message 테스트 부분
4. 시도 1의 변경을 reset/revert/재구현하지 말고, 미완 부분·컴파일 오류·테스트 실패만 고쳐 원 계약을
   끝낸다.
5. App, platform, store, audio crate와 관련 없는 기존 변경은 건드리지 않는다.

## 현재 확인된 상태

- 신규 `device_menu.rs`와 `tests/device_menu.rs`가 존재한다.
- `slot2-ui/src/lib.rs`, 영·한 FTL, i18n 테스트에도 변경이 있다.
- 결과 보고서는 아직 없다. 시도 1의 검증 완료 여부도 확인되지 않았으므로 추측하지 않는다.

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2-ui -p slot2-i18n
cargo clippy -p slot2-ui -p slot2-i18n --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 검증 뒤 코드를 바꾸면 영향받는 명령부터 다시 실행한다.

## 최종 보고서

끝나기 전에 `C:\SLOT2\tasks\62-device-menu-ui.worker-result.md`를 작성한다.

- 성공/실패와 누적 호출 **2/2**
- 시도 1 provider 400과 보존한 변경, 이번 delta
- 원 Task62의 availability/clamp/navigation/draw/i18n/warm-cache 계약 결과
- 각 완료 기준 명령, 종료 코드와 마지막 결과 줄
- 생성·수정 파일, 최종 검증 뒤 코드 변경 여부
- 계약 의문 또는 남은 위험 1줄, 소요 시간

네트워크·실기·공용 설정 변경·위임·커밋·푸시는 금지한다. 이번 호출이 실패해도 보고서를 남기고 세 번째
시도를 하지 않는다.
