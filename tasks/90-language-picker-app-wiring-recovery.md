# Task 90 — 400 중단 복구 2/2

이 호출은 Task90의 **누적 두 번째이자 마지막 호출**이다. 첫 호출은 구현 도중 오래 열린 대화형
세션의 요청이 869 messages / 약 2.32MB까지 커져 provider `400001 read body failed`로 중단됐다.
요청 role과 tool-call 연결은 정상이고, 현재 checkout에는 구현과
`crates/slot2/tests/language_picker_app.rs`가 이미 존재한다. **처음부터 다시 구현하지 않는다.**

## 수행할 일

1. `C:\SLOT2\tasks\90-language-picker-app-wiring.md`를 완전한 계약으로 다시 읽는다.
2. 현재 Task90 변경분만 좁게 검토한다.
   - `crates/slot2/src/app.rs`
   - `crates/slot2/src/lib.rs`
   - `crates/slot2/src/host_app.rs`
   - `crates/slot2/src/device_app.rs`
   - `crates/slot2/tests/language_picker_app.rs`
   - Task90 때문에 수정한 `timezone_menu_app.rs`, `about_sticker_app.rs`
   - `assets/lang/en.ftl`, `ko.ftl`의 신규 실패 문구
3. 남은 compile/test 오류나 원 계약 누락만 수정한다. 관련 없는 기존 변경을 정리·복원하지 않는다.
4. 마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2 --test language_picker_app
cargo test -p slot2
cargo test -p slot2-i18n
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2 --all-targets -- -D warnings
```

5. `C:\SLOT2\tasks\90-language-picker-app-wiring.worker-result.md`를 **누적 호출 2/2**로 작성한다.
   원 Task90이 요구한 보고 항목, 실제 수정 파일, 각 명령 종료 코드·마지막 결과 줄·test 수, 최종 검증
   뒤 코드 변경 여부를 포함한다.

## 금지

- 세 번째 호출을 전제로 작업 남기기
- 위임, 새 태스크 생성, 커밋, 푸시
- 네트워크, 실기, 공용 GJC/BAI/OpenCodex/Codex 설정 접근
- 전체 workspace 테스트와 device 배포
- 첫 호출의 거대한 HTTP 요청/대화 로그 읽기 또는 본문 복사

실패해도 반드시 worker-result를 2/2 실패 보고서로 남긴다.
