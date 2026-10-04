# Task 56 — FCEUmm NES 치트 validator와 disabled 전달 수정

## 목적

저장소가 pin한 FCEUmm libretro adapter의 실제 파서 계약을 `slot2-retro::quirks`에 추가한다.
FCEUmm + NES를 `Unchecked`에서 `Validated`로 올리고, adapter가 `retro_cheat_set`의 `enabled`와
`index`를 사용하지 않는 사실에 맞춰 FCEUmm도 enabled entry만 전달하도록 정책을 바로잡는다.

Task 55의 Session은 이미 `cheat_delivery(CoreId)`를 초기 적용·토글·rollback의 공통 경로에서
사용한다. 따라서 이번 태스크는 Session을 다시 구현하지 않고 quirks 정책과 순수 validator만
수정한다.

이 태스크는 **누적 최대 2회**다. 실패·중단 호출도 모두 횟수에 포함한다.

## 먼저 읽을 범위

- `AGENTS.md`의 안전·품질 규칙. 역할 분리 조항만 무시하고 직접 작업한다.
- `docs/DECISIONS.md`의 D-03, D-21만
- `tasks/54-mgba-cheat-validator.result.md`
- `tasks/55-mgba-disabled-cheat-delivery.result.md`
- `crates/slot2-retro/src/quirks.rs`
- `crates/slot2-retro/tests/cheat_quirks.rs`
- `crates/slot2/src/session.rs`의 `apply_cheats`와 `cheat_delivery` 사용 부분만 읽고 수정하지 않는다.
- `cores/fceumm/commit`

## 고정된 upstream 근거

네트워크를 사용하지 말고 아래 사실을 이 태스크의 입력 계약으로 사용한다.

저장소의 pinned FCEUmm commit은 `236ccdfc911e84c60fea6b9d0699c2d440a8de14`다. 해당
`src/drivers/libretro/libretro.c`의 `retro_cheat_set`은 다음처럼 동작한다.

- `code == NULL`만 바로 반환한다.
- 입력을 1024-byte `temp`에 `strlcpy`하므로 NUL terminator를 포함해 최대 1023 input bytes만
  온전히 보존한다. 그보다 길면 조용히 잘린다.
- `+,;._ `의 여섯 ASCII 문자로 `strtok` 분리한다. 선행·후행 separator와 연속 separator는
  빈 token 없이 건너뛴다. tab, LF, CR 등 다른 whitespace는 separator가 아니다.
- 각 non-empty token을 다음 순서로 해석한다.
  1. raw: `AAAA:VV` — 정확히 4 hex, colon, 2 hex
  2. raw with compare: `AAAA?CC:VV` — 4 hex, question mark, 2 hex compare, colon, 2 hex value
  3. NES Game Genie: 정확히 6 또는 8자, alphabet `APZLGITYEOXUKSVN`, 대소문자 무관
  4. Pro Action Replay: 정확히 8 hex, 대소문자 무관
- token 하나라도 위 형식이 아니면 core는 그 token만 로그로 버리고 나머지를 계속 적용한다.
  프론트엔드는 일부만 조용히 적용되는 상태를 만들지 않도록 **모든 token이 유효할 때만** 전체를
  `Validated`로 판정한다.
- separator만 있는 문자열은 token이 하나도 없어 아무것도 적용하지 않으므로 거부한다.
- adapter와 Game Genie helper가 byte별 C `toupper`를 호출하므로 non-ASCII UTF-8을 core까지
  보내지 않는다.
- raw branch의 `strtoul`과 PAR branch의 `sscanf`는 변환 성공을 확인하지 않는다. 특히 pinned PAR
  helper는 길이 8만 맞으면 성공을 반환해 non-hex에서 초기화되지 않은 값을 읽을 수 있다. validator는
  의도된 문법인 ASCII hex를 엄격히 요구한다.
- `retro_cheat_set(unsigned index, bool enabled, ...)` 본문은 `index`와 `enabled`를 읽지 않고 모든
  인식된 token을 활성 cheat로 추가한다. 따라서 FCEUmm의 disabled entry는 core에 보내면 안 된다.

Task 55의 “mGBA 외 네 core는 enabled flag를 읽는다”는 입력 계약 중 FCEUmm 부분은 위 pinned
source 확인으로 폐기한다. 나머지 세 core의 정책은 이번 태스크에서 바꾸지 않는다.

source provenance:

- pinned adapter:
  `https://github.com/libretro/libretro-fceumm/blob/236ccdfc911e84c60fea6b9d0699c2d440a8de14/src/drivers/libretro/libretro.c#L3443-L3537`
- pinned Game Genie/PAR helpers:
  `https://github.com/libretro/libretro-fceumm/blob/236ccdfc911e84c60fea6b9d0699c2d440a8de14/src/cheat.c#L321-L419`

## 구현 계약

### 1. FCEUmm 전달 정책 수정

- `cheat_delivery(CoreId::Fceumm)`을 `CheatDelivery::EnabledEntriesOnly`로 바꾼다.
- `CoreId::Mgba`도 계속 `EnabledEntriesOnly`다.
- gpSP, SNES9x, Genesis Plus GX는 기존 `PassAllEntries`를 유지한다.
- enum과 함수 문서·주석에서 “mGBA만”, “다른 모든 core는 enabled를 읽는다”처럼 틀린 설명을
  모두 고친다. 이유는 두 pinned adapter가 flag를 무시한다는 사실로 적는다.
- Session 코드를 수정하지 않는다. 기존 공통 apply 경로가 정책 변경을 자동으로 사용해야 한다.

### 2. FCEUmm + NES validator

- `validate_cheat(CoreId::Fceumm, Platform::Nes, code)`가 위 parser 계약을 검사한다.
- 입력 전체가 ASCII이고 1023 bytes 이하여야 한다. 1023은 허용하고 1024부터 거부한다.
- separator는 정확히 `+`, `,`, `;`, `.`, `_`, space 여섯 문자다.
- adapter의 `strtok` 의미대로 선행·후행·연속 separator는 건너뛴다. 단 유효 token이 하나 이상
  있어야 한다.
- 모든 non-empty token이 raw, raw-with-compare, Game Genie, PAR 중 하나여야 한다.
- raw와 PAR의 각 숫자 자리는 ASCII hex만 허용한다. `strtoul`/`sscanf`의 느슨함이나 undefined
  behavior를 validator에서 복제하지 않는다.
- Game Genie alphabet은 대소문자를 모두 허용하지만 다른 문자는 거부한다.
- token 하나가 잘못되면 나머지 token이 유효해도 전체 오류다. 부분 적용은 허용하지 않는다.
- 성공 시 `Ok(CheatValidation::Validated)`를 반환한다. 입력을 복사·변환·정규화하지 않는다.
- 오류 reason은 사람이 구분할 수 있게 하되 cheat code 전체를 보관하거나 출력하지 않는다.

### 3. 기존 계약 보존

- SNES9x + SNES와 mGBA + GB/GBC/GBA의 기존 유효·무효 결과를 바꾸지 않는다.
- gpSP + GBA와 Genesis Plus GX + MD/SMS는 계속 `Unchecked`다.
- 지원하지 않는 core/platform 조합, 공통 empty, interior NUL의 기존 선행 오류 순서를 유지한다.
- registry 지원 matrix와 공개 API signature를 바꾸지 않는다.

## 테스트 계약

`crates/slot2-retro/tests/cheat_quirks.rs`를 확장·정정해 최소 다음을 직접 확인한다.

- 지원 matrix의 FCEUmm + NES 예가 이제 `Validated`이고, gpSP/GPGX만 `Unchecked`로 남음
- FCEUmm raw `1234:AB`, zero-page `00ff:01`, raw compare `1234?56:78` 성공
- Game Genie 6자 `SXIOPO`, 8자 `GZUXNGEI` 및 lower/mixed case 성공
- PAR 8 hex `12345678`, `abcdef01`, mixed case 성공
- 네 형식을 `+,;._ ` 각각으로 연결한 다중 token 성공
- 선행·후행·연속 separator가 있어도 유효 token이 하나 이상이면 성공
- separator만 있는 입력은 실패
- raw/compare의 잘못된 길이·구두점 위치·non-hex, 5/7/9자 Game Genie, alphabet 밖 문자,
  7/9자리 PAR 실패
- tab, LF, CR이 token 사이에 들어간 경우 실패
- 한글·전각 숫자 등 non-ASCII 실패
- 유효 token과 무효 token이 섞이면 전체 실패
- 정확히 1023 bytes인 유효한 다중 token 성공, 1024 bytes부터 실패. fixture 길이를 assert해
  경계 테스트가 우연히 다른 길이가 되지 않게 한다.
- mGBA와 FCEUmm은 `EnabledEntriesOnly`, gpSP/SNES9x/GPGX는 `PassAllEntries`
- 기존 SNES9x/mGBA validator 테스트 전부 회귀 유지
- 오류 reason이 전체 code를 복사하지 않고 입력 문자열도 바뀌지 않음

제품용 mock hook, 실제 core 호출, parser의 느슨한 C 변환을 흉내 낸 unsafe 코드는 금지한다.

## 수정 허용 범위

- `crates/slot2-retro/src/quirks.rs`
- `crates/slot2-retro/tests/cheat_quirks.rs`
- `tasks/56-fceumm-cheat-validator.worker-result.md`

그 밖의 파일은 수정하지 않는다.

## 금지·범위 밖

- 커밋·푸시
- 네트워크 사용
- Pi, RG SP, adb, Samba, SD 카드 등 실기 접근
- `~/.gjc-bai/agent/*.yml` 및 공용 설정 수정
- Session/App/UI/i18n/toast 변경
- `.cht` 읽기·쓰기 변경
- validator의 Session 연결
- gpSP·Genesis Plus GX 문법 validator
- FCEUmm upstream parser 수정 또는 C FFI 추가
- FCEUmm의 multi-token `type` 상태 전파 같은 upstream 의미 변경
- 워크스페이스 전체 테스트와 배포 빌드

## 완료 기준

최종 코드 상태에서 아래를 원문 그대로 순서대로 실행한다.

```text
cargo fmt --all -- --check
cargo test -p slot2-retro --test cheat_quirks
cargo test -p slot2 --test session
cargo clippy -p slot2-retro -p slot2 --all-targets -- -D warnings
```

각 명령의 종료 코드와 마지막 결과 줄을 기록한다. 검증 뒤 소스를 고치면 네 명령을 다시 실행한다.
첫 응답까지 최대 5분, 전체 최대 45분 기다린다. 진행 신호는 로그 크기가 아니라
`git status --short`다.

## 결과 보고서

끝나기 전에 `C:\SLOT2\tasks\56-fceumm-cheat-validator.worker-result.md`를 작성한다.

- 성공/실패와 누적 호출 횟수
- FCEUmm에서 인정·거부하는 네 token 형식, separator, 1023-byte 경계
- FCEUmm disabled 전달 정책 수정과 Session 공통 경로 회귀 결과
- SNES9x/mGBA 회귀 및 gpSP/GPGX `Unchecked` 유지 결과
- 네 검증 명령의 종료 코드와 마지막 결과 줄
- 생성·수정 파일 목록
- pinned parser의 느슨한 PAR 변환 및 multi-token `type` 상태 전파를 포함한 남은 upstream 위험 1줄
- 계약이 틀려 보이는 부분 1줄

실패해도 반드시 원인과 마지막 상태를 쓴다. 두 번째 호출도 실패하면 더 시도하지 않는다.
