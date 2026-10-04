# Task 88 — 검토 수정 2/2: effective 언어 로그 escaping

Task88의 **누적 두 번째이자 마지막 호출**이다. 기존 구현과
`tasks/88-language-startup-app-wiring.worker-result.md`를 이어서 수정한다.

## 발견된 결함

`crates/slot2/src/main.rs`의 `announce_language`는 `requested`만 `{:?}`로 escape하고 `effective`는
`{effective}`로 raw 출력한다.

```rust
eprintln!("slot2: language requested={requested:?} effective={effective}");
```

`SLOT2_LANG`은 계약상 제어문자를 포함한 문자열도 그대로 요청값으로 전달한다. Linux 카드에는 그런
이름의 FTL 파일도 존재할 수 있으므로 load가 성공하면 `ctx.i18n.code()`에도 제어문자가 남는다. 이때
effective code가 로그 줄을 삽입할 수 있어 원 명세의 “제어문자를 그대로 로그에 흘리지 말고 debug
escaping을 사용한다”를 위반한다.

## 수정 계약

- `announce_language`가 **requested와 effective를 모두 Rust debug escaping**으로 출력하게 한다.
- 정상 예시는 `slot2: language requested="ko" effective="ko"`처럼 두 값 모두 따옴표가 보인다.
- 줄바꿈·탭·따옴표·역슬래시는 실제 제어문자로 출력되지 않고 `\n`, `\t`, `\"`, `\\` 형태로 한
  로그 줄 안에 남아야 한다.
- 언어 precedence, resolver, Card/UiCtx/App 생성 순서, fallback, FTL/store 파일과 테스트 시나리오는
  바꾸지 않는다.
- 변경은 `crates/slot2/src/main.rs`의 해당 진단 함수와 바로 인접한 설명에만 제한한다. 테스트를
  통과시키기 위한 source-text assertion이나 새 로깅 abstraction을 만들지 않는다.

## 완료 기준

마지막 코드 변경 뒤 아래를 원문 그대로 순서대로 실행한다.

```powershell
cargo fmt --all -- --check
cargo test -p slot2 --test language_startup
cargo test -p slot2
cargo check -p slot2 --no-default-features --features device
cargo clippy -p slot2 --all-targets -- -D warnings
```

모두 종료 0이어야 한다. 전체 workspace 테스트와 device 배포는 실행하지 않는다. 실기·네트워크·공용
설정 접근, 위임, 커밋, 푸시는 금지한다.

## 결과 보고서

기존 `C:\SLOT2\tasks\88-language-startup-app-wiring.worker-result.md`를 **누적 호출 2/2** 보고서로
갱신한다.

- 두 로그 필드가 모두 debug-escaped된 최종 형식
- 그 밖의 동작 변경이 없다는 확인
- 다섯 완료 기준 명령의 종료 코드, 마지막 결과 줄과 test 수
- 최종 검증 뒤 코드 변경 여부와 소요 시간

실패해도 보고서를 갱신한다. 이번 호출 뒤에는 성공/실패와 관계없이 세 번째 호출을 하지 않는다.
