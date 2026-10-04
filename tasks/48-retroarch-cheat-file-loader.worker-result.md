# Task 48 워커 결과 (누적 2/2) — 카드의 RetroArch `.cht` 로더

**최종: 성공. 누적 호출 2/2.** 2회차에서 지정 검증 3종이 모두 종료 0이고 `.cht` 테스트 15개가
통과했다. 1회차에서 인정된 로더 동작(경로·레이아웃·인용·순서·전체 실패·읽기 전용)은 그대로다.

## 2회차 delta

- **선언 count에 묶인 작업 제거(`cheats.rs`)**: `Vec::with_capacity(count as usize)`와 `0..count`
  루프를 없앴다. 검증은 파일이 실제로 쓴 owned entry의 정렬 맵을 순회하며 (1) owned 인덱스는 0부터
  연속, (2) 모든 owned 인덱스가 선언 count 미만, (3) 완전한 owned entry 수가 선언 count와 같음을
  확인한다. 결과 벡터는 검증을 통과한 실제 entry 수로만 할당하고, 선언 count는 비교에만 쓴다.
  `cheats = 4294967295` 두 줄 파일은 첫 검사에서 바로 거부되며 어떤 할당·루프도 count에 비례하지
  않는다(명세 지시에 따라 옛 코드를 실행해 OOM을 재현하는 음성 테스트는 만들지 않았고, 코드 구조와
  새 경계 테스트로 확인).
- **미지 metadata가 구조에 영향 주지 않음(`cheats.rs`)**: 이제 `desc`/`code`/`enable` 세 필드만
  `Entry`를 만들거나 갱신한다. `cheat999_handler` 같은 미지 indexed 필드는 어떤 인덱스든 gap/
  범위 판정에 관여하지 않는다. 오류 메시지는 인덱스/카운트 맥락을 유지한다(중간 누락은
  `cheatN_desc and cheatN_code are missing (cheats = M)`, 개별 누락은 `cheatN_... is missing`).
- **추가 테스트 4개(`tests/cheats.rs`)**: ① `cheats = 4294967295` + owned 없음 → `Error::Invalid`
  (count를 메시지에 포함, panic·거대 할당 없음), ② 같은 count + 완전한 인덱스 1개 → 누락 인덱스와
  count를 함께 지목, ③ `cheats = 4294967296`(u32 초과) → 여전히 잘못된 count, ④ 미지 indexed
  metadata: `cheats = 0`에서 `cheat4294967295_handler` 무시·빈 결과, `cheats = 1`에서 유효 항목
  앞뒤의 `cheat8750_handler`/`cheat999_mem_search`가 gap이나 레코드 변경을 만들지 않음, 같은 파일에서
  `cheat1_desc`(범위 밖 owned)는 여전히 실패하고 metadata가 선언 인덱스를 대신 채우지 못함.
- **역방향 확인**: metadata가 다시 phantom `Entry`를 만들도록 1줄 되돌리면
  `unknown_indexed_metadata_is_not_an_entry`가 실패했고, 복원 후 sha1이 동일함을 확인했다(이후
  mtime 갱신으로 낡은 아티팩트 재사용을 배제). 기존 11개 테스트는 수정 없이 그대로 통과한다.

## 최종 검증 (명세 순서, 최종 코드 상태)

- `cargo fmt --all -- --check` → 종료 **0**.
- `cargo test -p slot2-store` → 종료 **0**: lib 1 / card 19 / **cheats 15** / state_undo 11, 0 failed.
- `cargo clippy -p slot2-store --all-targets -- -D warnings` → 종료 **0**.
- 최종 검증 후 코드 변경 없음. 워크스페이스 전체 테스트·dist·Pi·실기 접근은 하지 않았다.

## 누적 동작 (1회차 유지)

- `System/cheats/<PLAT>/<stem>.cht`. `ensure_layout`에 `System/cheats` + 7개 플랫폼 하위(부모 먼저),
  기존 폴더·count 동작 유지. `Card::read_cheats`가 `Error::Io(경로)` / `Error::Invalid(줄+키)`,
  없는 파일은 빈 목록. 공식 인용·이스케이프·유니코드·`+` 복수 파트·문장부호 보존, enable
  `true/false/1/0`(대소문자 무시, 누락 false), 전체 줄 주석·미지 키 무시, 읽기 전용.

## 남은 우려

- count 선언이 없는 파일(0바이트 포함)은 계약 5에 따라 오류다. 데스크탑 작성기가 항상
  `cheats = 0`을 쓰지 않으면 빈 `.cht`가 로드 실패가 된다.
- `desc`/`code` 없이 `mem_search*` 같은 검색 필드만 있는 항목은 여전히 거부된다(계약대로).
- 2회차 소요 약 5분(상한 45분 내).
