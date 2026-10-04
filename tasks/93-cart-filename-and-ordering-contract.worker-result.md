# Task 93 — 작업자 결과 (누적 호출 1/2)

## 결과

**성공 (1/2).** 완료 기준 7개 모두 종료 0. **production 로직은 한 줄도 바꾸지 않았다** — 새 계약
테스트 6개가 현재 구현에서 그대로 통과했으므로 `Card::scan`과 경로 함수를 다시 쓰지 않았고, 바꾼 것은
module 문서·DESIGN·M5 checkbox와 새 테스트 파일뿐이다.

## 최종 정렬 정책과 실제 순서

정책: `Cart::title`(=`stem`)의 Rust `str` 순서 = Unicode scalar 값 순서. locale collation·자연수 정렬·
대소문자 folding·NFC/NFD 정규화 없음. 동일 title은 확장자 byte 순서로 결정적.

테스트가 실제로 단언하는 순서:

- `Zed < alpha < あいう < 漢字 < 가방 < 각도 < 나무` — ASCII 대문자가 소문자보다 먼저(사전순이면
  `apple < Zed`가 되어야 하므로 이 순서가 scalar 순서의 증거), 그 다음 가나, 한자, 한글 음절.
  `가방 < 각도 < 나무`로 가나다순을 직접 단언.
- 생성 순서 독립: 같은 파일들을 다른 순서로 만든 두 폴더가 같은 목록을 낸다.
- `Alpha < alpha`는 확장자만 다른 한 stem 두 파일(`Alpha.sfc`, `alpha.smc`)로 단언한다(아래 계약
  항목 1 참조).
- 같은 stem 두 확장자는 `Twin.sfc` → `Twin.smc`(확장자 byte 순서).

## 파일명·부속 경로 결과

보존한 이름(스캔 결과 그대로):

| 원본 파일명 | stem / title | ROM path |
| --- | --- | --- |
| `10-in-1 [한글] + 日本語 (Rev A)!.gba` | `10-in-1 [한글] + 日本語 (Rev A)!` | 그대로 |
| `Alpha.beta's & more #50%@home.GBA` | `Alpha.beta's & more #50%@home` | 그대로 |

마지막 허용 확장자 하나만 제거되고 그 앞의 공백·점·괄호·대괄호·`'`·`+`·`&`·`!`·`#`·`%`·`@`와
한글·가나·한자는 그대로 남는다. `stem == title`이고 `rom`은 원본 경로와 정확히 같다. 그 stem이 다섯
부속 경로에 그대로 쓰이며 전부 카드 root/platform 폴더 안에 있고 `..` 성분이 없다:

- `Labels/GBA/<stem>.png` — 실제 파일을 만들어 `label()`이 `Some(path)`를 돌려주는 것까지 확인
- `Saves/GBA/<stem>.sav` — 작은 byte round-trip 확인
- `States/GBA/<stem>` — 그 디렉터리에 state를 쓰고 다시 읽어 확인
- `System/games/GBA/<stem>.ini`
- `System/cheats/GBA/<stem>.cht`

skip: `.hidden.gba`, `._Zelda.gba`, directory `subdir.gba`, `name.gba.bak`(마지막 확장자가 미지원),
`packed.zip`, 확장자 없는 `readme`는 카트가 아니다. extension 대소문자는 무시하므로 `Case.GBA`,
`Case2.GbA`는 나타나고 stem은 `Case`, `Case2`로 원래 표기를 유지한다. UTF-8로 읽을 수 없는 이름은
기존처럼 건너뛴다(Windows에서 만들 수 없는 이름을 흉내 내는 테스트는 넣지 않았다).

동일 stem 복수 확장자: `Twin.sfc`+`Twin.smc` → 카트 둘 모두 표시되고 ROM path는 서로 다르지만
label/save/states/settings/cheat 다섯 경로가 두 카트에서 완전히 같다(레거시 레이아웃의 stem당 세이브
하나 제약). 같은 stem의 GB/GBA 카트는 플랫폼 폴더 때문에 다섯 경로가 모두 다르다.

## production 변경

없음. 계약 불일치는 발견되지 않았고, `card.rs`에는 module 문서만 손댔다(스캔/정렬 문구를 정확히
다듬고, 같은 platform 동일 stem 복수 확장자가 부속 경로를 공유한다는 제약을 기록).

## 문서·마일스톤

- `crates/slot2-store/src/card.rs` module doc: 마지막 확장자 하나만 제거·나머지 문자 보존, `title` =
  `stem`, scalar 순서 정렬(대문자 → 소문자 → 가나·한자 → 한글 음절), 동일 title은 확장자 byte 순서,
  정규화·collation·folding 없음, stem이 다섯 부속 경로의 key이고 플랫폼 폴더로 갈라지며 같은 플랫폼의
  복수 확장자는 경로를 공유한다는 제한.
- `docs/DESIGN.md` §8 정렬 한 줄을 scalar/확장자 tie-break/정규화 없음으로 교체, §9에 stem 규칙과
  부속 경로 공유 제약 문단 추가.
- `docs/MILESTONES.md` M5 `한·영 혼합 정렬 정책, 파일명 특수문자 처리` 한 항목만 `[x]`.

## 완료 기준 명령 (마지막 코드 변경 뒤 순서대로 실행)

| # | 명령 | 종료 | 마지막 result 줄 | passed/failed/ignored |
| --- | --- | --- | --- | --- |
| 1 | `cargo fmt --all -- --check` | 0 | (출력 없음) | — |
| 2 | `cargo test -p slot2-store --test card_filename_contract` | 0 | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.53s` | 6 / 0 / 0 |
| 3 | `cargo test -p slot2-store --test card` | 0 | `test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s` | 19 / 0 / 0 |
| 4 | `cargo test -p slot2-store` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` (doc-tests) | 118 / 0 / 0 (12줄) |
| 5 | `cargo check -p slot2` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.14s`` | — |
| 6 | `cargo check -p slot2 --no-default-features --features device` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.98s`` | — |
| 7 | `cargo clippy -p slot2-store -p slot2 --all-targets -- -D warnings` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 14.61s`` | warning 0줄 |

core 의존 skip 없음(모든 result 줄 `0 ignored`, 런타임 skip 보고 없음). 전체 workspace 테스트, 실제 GL
창, device 배포, 실기 접근은 하지 않았다.

## 생성·수정 파일

- `crates/slot2-store/tests/card_filename_contract.rs` (신규, 6 tests) — 의존성 없이 `std`와
  `slot2_store`만 사용. test마다 process id + test name이 들어간 고유 temp 폴더를 쓰고 `Drop`에서
  best-effort 정리한다.
- `crates/slot2-store/src/card.rs` — module 문서만.
- `docs/DESIGN.md` — §8 정렬 한 줄, §9 카드 레이아웃 문단.
- `docs/MILESTONES.md` — M5 checkbox 한 개.
- `tasks/93-cart-filename-and-ordering-contract.worker-result.md` — 이 보고서.
- **최종 검증 뒤 코드 변경 없음**: 완료 기준 실행 시작(10:24:40 KST)보다 모든 파일 mtime이 앞선다.
  변이 검사용 임시 `.bak`/`.orig` 파일은 남기지 않았고 출력 수집 폴더는 삭제했다. 위임·커밋·푸시·
  네트워크·실기·공용 설정 접근 없음.

변이 검사로 새 테스트가 실제로 물어뜯는지 확인했다(임시 수정 후 원복, 최종 상태 재검증 완료): stem을
마지막 확장자 대신 전체 파일명으로 → 6개 전부 실패, extension 대소문자 구분으로 → 2개 실패,
`.`/`._` skip 제거 → 1개 실패.

## 계약이 틀려 보이는 부분

1. **`Alpha < alpha`는 Windows에서 두 파일로 단언할 수 없다.** NTFS는 대소문자를 구분하지 않아 같은
   폴더에 `Alpha.gba`와 `alpha.gba`가 공존하지 못한다(두 번째 쓰기가 첫 파일을 덮어쓴다). 그래서
   계약이 5번에서 허용한 "한 stem의 두 확장자"를 빌려 `Alpha.sfc`/`alpha.smc`로 단언했다 — 파일명은
   다르고 단언하는 것은 title의 순서다. 테스트 module doc에 이유를 적었다.
2. **확장자 tie-break는 이 host에서 판별력이 없다.** 같은 stem이면 파일명 순서와 확장자 byte 순서가
   항상 같아서, NTFS처럼 이름 순으로 `read_dir`을 돌려주는 파일시스템에서는 tie-break를 제거해도 같은
   순서가 나온다(확인함: 제거해도 6개 모두 통과). ext4처럼 hash 순서를 주는 파일시스템에서는 tie-break가
   실제로 결정하므로 테스트는 Linux에서 판별력을 갖는다. 지금 단언은 "쉘프가 보여 주는 답"을 고정한다.
   이 사실을 테스트 주석에 남겼다.
3. 계약은 skip 대상에 "directory"를 들지만 `scan`은 애초에 `path.is_file()`로 거르므로, directory
   이름이 지원 확장자로 끝나도(`subdir.gba`) 카트가 되지 않는다 — 테스트로 고정했고 모순은 없다.

## 소요 시간

약 15분(10:10–10:26 KST). 대부분 `cargo test -p slot2-store`(전체) 1회와 변이 검사 5회의 컴파일이었다.
