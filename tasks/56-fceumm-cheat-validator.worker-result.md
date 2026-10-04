# Task 56 워커 결과 — FCEUmm NES 치트 validator와 disabled 전달 수정

**성공. 누적 호출 1/2.** 지정 검증 4종 모두 종료 0. Session 테스트는 mGBA 실코어로 실행됐다
(27 passed, skip 없음).

pinned FCEUmm commit은 `cores/fceumm/commit`의 `236ccdfc911e84c60fea6b9d0699c2d440a8de14`로 고정
근거와 일치했다.

## FCEUmm이 인정·거부하는 형식

- **인정(네 token 형식)**: raw `AAAA:VV`(4 hex + colon + 2 hex, `1234:AB`, `00ff:01`),
  raw-with-compare `AAAA?CC:VV`(`1234?56:78`), NES Game Genie 6자(`SXIOPO`)·8자(`GZUXNGEI`)
  (alphabet `APZLGITYEOXUKSVN`, 대소문자 무관), Pro Action Replay 8 hex(`12345678`, `abcdef01`).
- **separator**: 정확히 `+`, `,`, `;`, `.`, `_`, space 여섯 문자. `strtok` 의미대로 선행·후행·연속
  separator는 건너뛰되 유효 token이 하나 이상이어야 한다(separator만 있는 입력은 거부).
- **거부**: raw/compare의 잘못된 길이·구두점 위치·non-hex, 5/7/9자 Game Genie와 alphabet 밖 문자,
  7/9자리 PAR, tab/LF/CR(separator가 아니라 token 안으로 들어간다), 한글·전각 숫자 등 non-ASCII,
  유효 token과 무효 token이 섞인 입력(전체 실패 — core는 그 token만 버리고 나머지를 적용하지만
  프론트엔드는 부분 적용을 만들지 않는다).
- **1023-byte 경계**: raw token 128개를 `+`로 이어 정확히 1023 bytes인 입력은 `Validated`, 같은
  fixture에 1 byte를 더한 1024 bytes는 길이 오류(reason에 1024 명시). fixture 길이를 assert해 경계
  테스트가 다른 길이를 재지 않게 했다. 8자리 token은 hex(PAR)와 Game Genie alphabet이 서로 부분집합이
  아니어서 모호하지 않다.
- 입력은 복사·변환·정규화하지 않고, 오류 reason은 token 번호와 이유만 담고 code를 보관하지 않는다.

## FCEUmm disabled 전달 정책과 Session 공통 경로

- `cheat_delivery(CoreId::Fceumm)`을 `EnabledEntriesOnly`로 바꿨다. 이제 mGBA와 FCEUmm 두 adapter가
  `index`/`enabled`를 무시하고 모든 token을 활성 cheat로 넣는다는 사실로 주석을 고쳤고(“mGBA만”,
  “다른 모든 core는 enabled를 읽는다”는 틀린 설명 제거), gpSP·SNES9x·Genesis Plus GX는
  `PassAllEntries`를 유지한다.
- Session 코드는 한 줄도 수정하지 않았다(`session.rs` mtime이 Task 55 시점 그대로). 초기 적용·토글·
  rollback이 모두 `apply_cheats(core, self.core_id, …)` 한 경로를 쓰므로 정책 변경이 자동으로
  적용된다. Session 회귀는 실코어 27개 전부 통과로 확인했다(Session 쪽 FCEUmm fixture는 이 저장소에
  NES 테스트 ROM·세션 픽스처가 없어 만들지 않았고, 계약도 공통 경로 회귀만 요구한다).

## 기존 코어 회귀

- SNES9x + SNES(세 형식·255/256-byte·600-byte 한글)와 mGBA GB/GBC/GBA(GBA 13/17-byte unit, GB 5형식,
  non-ASCII 거부) 테스트는 그대로 통과한다.
- 지원 matrix에서 FCEUmm + NES가 `Validated`로 바뀌었고, `Unchecked`는 gpSP + GBA와
  Genesis Plus GX + MD/SMS 두 코어만 남았다(해당 테스트를 그 목록으로 정정).
- 공통 empty/interior-NUL 선행 오류, 지원하지 않는 조합 거부, 공개 signature·registry matrix는
  변경 없음.

## 검증 (최종 코드 상태, 명세 순서)

- `cargo fmt --all -- --check` → 종료 **0**.
- `cargo test -p slot2-retro --test cheat_quirks` → 종료 **0**,
  `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s`.
- `cargo test -p slot2 --test session` → 종료 **0**,
  `test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 14.06s`
  (실코어 실행, skip 없음).
- `cargo clippy -p slot2-retro -p slot2 --all-targets -- -D warnings` → 종료 **0**(경고·오류 없음).
- 최종 검증 후 코드 변경 없음.

## 생성·수정 파일

- `crates/slot2-retro/src/quirks.rs`: FCEUmm 전달 정책, `validate_fceumm`과 token 판정, dispatch arm,
  `CheatDelivery` 문서 정정.
- `crates/slot2-retro/tests/cheat_quirks.rs`: FCEUmm 테스트 4개 추가(형식·separator·거부·1023 경계),
  지원 matrix와 `Unchecked` 목록·전달 정책 테스트 정정, 오류 형태 테스트 확장.
- 그 밖의 파일은 수정하지 않았다(Session·App·UI·i18n·`.cht` 경로 포함).

## 남은 upstream 위험

pinned adapter의 raw branch는 `strtoul`의 변환 성공을 확인하지 않고 PAR helper는 길이 8만 맞으면
성공을 반환해 non-hex에서 초기화되지 않은 값을 읽을 수 있으며, multi-token에서 앞 token의 `type`이
다음 token으로 전파되므로 이 validator가 token별로 독립 해석해 거부한 코드를 core는 다른 type으로
받아들일 수 있다 — validator는 의도된 문법만 요구하고 그 느슨함은 재현하지 않았다(계약대로).

## 계약이 틀려 보이는 부분

고정 근거가 “인식되지 않는 token은 core가 로그로 버리고 나머지를 계속 적용한다”고 적으면서 구현
계약은 “모든 token이 유효할 때만 Validated”를 요구하는데, 이는 core 동작의 재현이 아니라 프론트엔드의
의도적으로 더 엄격한 정책이다(부분 적용 방지) — 그대로 구현했고 주석에 그 이유를 남겼다.
소요 약 5분(상한 45분 내).
