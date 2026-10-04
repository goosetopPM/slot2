# Task 90 — 작업자 결과 (누적 호출 2/2)

## 결과

**성공. 누적 호출 2/2.** 1회차는 구현 중 provider `400001 read body failed`로 중단됐고, 이번
2회차는 기존 구현을 이어서 검증·수정만 했다. 처음부터 다시 만들지 않았고 세 번째 호출을 전제로 한
남은 작업도 없다.

## 이번 호출에서 실제로 고친 것

1회차 결과물에는 컴파일 오류가 없었고 집중 테스트도 통과했다. 남아 있던 문제는 lint와 format이었다.

- `cargo clippy -p slot2 --all-targets -- -D warnings` → 종료 101.
  `tests/language_picker_app.rs`에서 `unused_mut` 1건, `unused_assignments` 9건. 수정 뒤 재컴파일에서
  같은 종류가 2건 더 나와 합계 12건(unused_mut 2, unused_assignments 10)을 모두 제거했다.
- `cargo fmt --all -- --check` → 종료 1. `src/app.rs` 1곳, `tests/language_picker_app.rs` 여러 곳.

수정 내용:

- 사용하지 않는 `now = tap(...)`/`now = open_language(...)` 대입 10곳을 호출문만 남김. 시간 전진은
  그대로이고 뒤따르는 입력 시각은 이전 반환값을 쓰므로 단언 의미는 바뀌지 않는다.
- `let mut now` → `let now` 2곳(재대입이 사라진 곳).
- rustfmt가 앞 줄의 trailing comment에 맞춰 과도하게 들여쓴 주석 1곳을 빈 줄로 분리.
- `cargo fmt -p slot2` 적용. app.rs는 fmt가 지적한 `Screen::List | … | Screen::Language(_)` match
  arm 한 곳만 재포맷됐고, 다른 미커밋 변경은 건드리지 않았다.

계약 누락 보강 1건:

- `a_pack_that_vanishes_before_the_change_keeps_the_running_language`에 "pending request 동안 Language
  화면 입력 무시"를 검증하는 단언을 추가했다. request를 queue한 뒤 B와 Up을 눌러도 화면이 Shelf로
  닫히지 않고 highlight와 request가 그대로임을 확인한다(1회차에는 이 경로가 구현만 되고 검증되지
  않았다).

그 밖의 1회차 구현(app.rs Language 화면·picker 발견·commit, lib.rs service helper, host/device 배선,
ftl 문구)은 계약과 일치해 손대지 않았다.

## 계약 대조 결과

- discovery/filtering: Language 화면을 열 때만 `I18n::available(Some(card/System/Lang))`을 1회 호출하고
  각 code를 같은 dir로 `I18n::load`한다. load 성공 code만 `LanguageOption(code, pack.name())`으로
  들어가고 caller 순서가 유지된다. unknown 확장자(`notes.txt`)와 malformed pack은 제외된다.
- embedded en 복구: card의 `en.ftl`이 malformed여도 `I18n::embedded(FALLBACK)`으로 복구해 후보에 남고
  이름은 "English"다. 실패 code 진단은 `{code:?}`로 escape한 한 줄이다.
- current effective source: `set_current_language(ctx.i18n.code())`가 startup에 정확히 1회 호출되며,
  badge는 stored requested(`language = ko`)나 `SLOT2_LANG`이 아니라 이 값이다. 테스트가 stored
  requested와 다른 current를 확인한다.
- Shelf input: availability는 Language/TimeZone/About true, 첫 선택은 Language. Language에서 Up/Down은
  highlight만, B/Menu는 write·request 없이 parent Shelf, unchanged A는 write·context rebuild 없이 Shelf,
  changed A는 request만 queue하고 `Screen::Language(parent)`에 남는다.
- pending 동안 입력: `(Screen::Language(_), Action::Tap(_)) if language_request.is_some()`이 먼저
  매칭돼 press를 버린다. 이번 호출에서 테스트로 고정했다.
- 순서: `service_language_request`는 `UiCtx::new`를 정확히 1회(profile, `font_dirs.clone()`, requested,
  card lang dir) 만들고, effective != requested면 `language_load_failed()` 후 candidate 폐기·false,
  같으면 `commit_language`(= `Card::write_language` 먼저) 성공 뒤에만 `*ctx = candidate`와 true.
  `I18n::load`를 따로 호출해 pack을 두 번 parse하지 않는다.
- 성공: profile/font_dirs 보존, `ctx.faces` empty(이전 언어 face cache 미재사용), App current 갱신,
  Shelf Language 행으로 복귀.
- 실패: load mismatch는 설정 bytes/mtime·context/current 불변, Language 화면 유지, badge 유지,
  `language-load-failed` toast. save 실패(비 UTF-8 파일, settings path가 directory, 공백 포함 unsafe
  code `x y`)는 기존 bytes/dir·context/current 불변, Language 화면 유지, `language-save-failed` toast,
  panic 없음.
- 저장 경계: 성공 시 Task87 `write_language`만 사용. `ko` 선택은 volume/timezone/unknown key를 보존하고,
  `en` 선택은 `language` key만 제거한다(volume/unknown 보존).
- backend: host_app/device_app 모두 startup에 `set_current_language(ctx.i18n.code())` 1회,
  `run_frame` 뒤 draw 전에 `service_language_request` 1회. 두 backend가 적용 규칙을 복제하지 않는다.
- draw: Language 화면은 shelf 위에 picker panel 하나만 그리고 parent Shelf panel을 겹쳐 그리지 않는다.
  hold bar/toast는 그 뒤, HUD는 마지막에 그려져 기존 z-order가 유지되고, warm redraw는 upload 0이다.
- ftl: en/ko에 `language-load-failed`, `language-save-failed` 완성 문장이 계약 문구 그대로 들어가 있다.

## 완료 기준 명령 (마지막 코드 변경 뒤 순서대로 실행)

| 명령 | 종료 코드 | 마지막 결과 줄 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | (출력 없음) |
| `cargo test -p slot2 --test language_picker_app` | 0 | `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.58s` |
| `cargo test -p slot2` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |
| `cargo test -p slot2-i18n` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |
| `cargo check -p slot2 --no-default-features --features device` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.87s`` |
| `cargo clippy -p slot2 --all-targets -- -D warnings` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.31s`` |

test 수:

- `language_picker_app`: 1 binary, **9 passed / 0 failed / 0 ignored**.
- `slot2`: 31 result 줄 합계 **334 passed / 0 failed / 0 ignored**.
- `slot2-i18n`: 3 result 줄 합계 **34 passed / 0 failed / 0 ignored**.
- core 의존 skip: `-p slot2`의 모든 result 줄이 `0 ignored`이고 런타임 skip 보고도 없다. core library가
  필요한 slot2-retro 테스트는 이번 명령 범위가 아니다.
- clippy/check 출력에 warning 0줄(수정 전 10 warning → 0).

전체 workspace 테스트, device 배포, 실기(Pi/RG SP/adb/Samba/SD) 접근은 실행하지 않았다.

## 파일

이번 호출에서 수정:

- `crates/slot2/tests/language_picker_app.rs` — lint 12건(unused_mut 2, unused_assignments 10)/format 수정
  + pending 입력 단언 1건 추가.
- `crates/slot2/src/app.rs` — rustfmt가 지적한 match arm 들여쓰기 1곳.

이번 호출에서 변경하지 않음(1회차 산출물 그대로): `crates/slot2/src/lib.rs`,
`crates/slot2/src/host_app.rs`, `crates/slot2/src/device_app.rs`, `assets/lang/en.ftl`,
`assets/lang/ko.ftl`, `crates/slot2/src/app.rs`의 Language 구현부,
`tests/timezone_menu_app.rs`/`tests/about_sticker_app.rs`(1회차에 손댈 필요가 없었다).

**최종 검증 뒤 코드 변경 없음.** 검증 시작 시점 이후 소스 파일 mtime이 모든 명령 로그보다 앞선다.
명령 출력 수집용으로 만든 임시 로그 폴더는 삭제했다. 위임·커밋·푸시·네트워크·공용 설정 접근은 하지
않았다.

## 남은 범위

- 언어팩 `lang-font` preferred font 적용(이번 범위 밖). context 전체를 다시 만들기 때문에 현재는 새
  언어 기준 기본 UI/CJK font chain과 face cache가 새로 생성된다.

## 계약이 틀려 보이는 부분

없음. 다만 계약의 테스트 목록 중 "빈 picker 경계는 가능하면 직접 모델 setup으로 A no-op" 항목은
통합 테스트에서 도달할 수 없어 확인하지 못했다. App의 picker는 private field이고 discovery는 항상
embedded `en`을 후보로 넣으므로 빈 목록 상태를 만들 공개 경로가 없다(`choose_language`의 빈 목록
guard는 코드로 존재하고 slot2-ui 쪽 빈 목록 처리는 Task89에서 검증됐다). 이 항목 때문에 App에
테스트 전용 공개 경계를 추가하지는 않았다.

## 소요 시간

약 17분(00:39–00:56 KST). build/test 대기가 대부분이었다.
