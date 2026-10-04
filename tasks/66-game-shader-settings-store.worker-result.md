# Task 66 작업자 결과 — 게임별 셰이더 설정 저장 계약

## 판정

**성공.** 누적 호출 1/2 (이번이 첫 호출). 완료 기준 4개 명령 모두 종료 0이며, 최종 검증 뒤
코드 변경 없음.

## 계약 구현 결과

- **`slot2_store::ShaderPreset`** 공개. `Clone, Copy, Debug, PartialEq, Eq` 파생, variant는
  `Off, SharpBilinear, Lcd3x, ZfastCrt, Scanline` 정확히 다섯 개.
- **canonical 표기** `as_str()`: `none`, `sharp-bilinear`, `lcd3x`, `zfast-crt`, `scanline`.
  write는 항상 이 표기만 쓴다(수동 편집 표기를 읽어도 저장 시 canonical로 되돌린다).
- **`parse()`**는 앞뒤 공백과 ASCII 대소문자를 허용하고, 호환 표기 `off`, `sharp_bilinear`,
  `sharpbilinear`, `lcd-3x`, `zfast_crt`, `zfastcrt`, `scanlines`도 받는다. 빈 값과 그 밖의
  미인식 값은 모두 `None` — typo가 게임 실행을 막지도, 명시적 Off로 조용히 바뀌지도 않는다.
- **상속과 명시적 Off 구분**: `GameSettings::shader: Option<ShaderPreset>` 추가,
  `KEY_SHADER = "shader"`. key 부재/빈 값/invalid는 전부 `None`(플랫폼 기본 상속),
  `shader = none`만 `Some(ShaderPreset::Off)`이다. `default()`와 `is_default()`는 shader가
  `None`일 때만 기본으로 본다.
- `apply_to`는 `Some(p)`이면 canonical 값을 쓰고 `None`이면 `shader` key만 제거한다.
  core/scale/overscan/rewind의 공개 의미와 디스크 표기는 그대로다.
- **범위 준수**: shader 전용 Card API, generic settings framework, 새 파일 형식, `PlatformDef`
  변경, gfx/ui/slot2 production 변경은 만들지 않았다. 기존 `Card::read_settings` /
  `write_settings` 경로를 그대로 쓴다.

## safe write / 보존 결과

- shader 변경이 core·scale·overscan·rewind와 unknown key(`future_filter`)를 보존한다.
- core 변경과 scale 변경이 이미 저장된 shader와 unknown key를 각각 보존한다.
- `None` write는 shader key만 제거한다. 다른 key가 남으면 파일을 유지하고, shader가 마지막
  key였다면 기존 계약대로 파일도 제거한다. 파일이 애초에 없으면 no-op 성공이다.
- 기존 파일이 invalid UTF-8이면 shader 설정·해제 모두 `Error::Io(settings path, _)`이고 exact
  bytes가 보존되며 temp 파일도 남지 않는다. read는 관대하게 `None`을 돌려준다.
- settings path가 directory일 때도 설정·해제 모두 `Error::Io`이고 directory는 제거되지 않는다.
- `card.rs`의 full `GameSettings` literal에 `shader: None`을 추가했고, unknown-key 테스트
  `a_key_this_version_does_not_know_survives_a_save`는 fixture를 `future_filter`로 좁게 바꿔
  같은 회귀 계약을 유지했다. 두 테스트 모두 통과한다.

## 완료 기준 명령 (코드 변경 후 순서대로 1회 실행)

| 명령 | 종료 코드 | 마지막 결과 줄 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | (출력 없음) |
| `cargo test -p slot2-store` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`(doc-tests) — 8개 바이너리 합계 **79 passed / 0 failed**, 그중 `shader_settings` **13 passed / 0 failed** |
| `cargo check -p slot2 --tests` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 17.67s`` |
| `cargo clippy -p slot2-store --all-targets -- -D warnings` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.74s`` |

`cargo fmt --all` 실행으로 Rust 파일이 바뀐 것은 이번에 수정한 4개 파일뿐임을
최근 mtime 확인으로 검증했다(다른 파일 재포맷 없음). workspace test와 device 배포는 실행하지
않았다.

## 생성·수정 파일

- 수정: `crates/slot2-store/src/settings.rs` (`ShaderPreset`, `GameSettings::shader`, `KEY_SHADER`)
- 수정: `crates/slot2-store/src/lib.rs` (`ShaderPreset` export)
- 신규: `crates/slot2-store/tests/shader_settings.rs` (13 tests)
- 수정: `crates/slot2-store/tests/card.rs` (literal `shader: None`, unknown-key fixture 2줄)
- 최종 검증 뒤 코드 변경 없음. 커밋·푸시·네트워크·실기 접근 없음.

## 계약 의문 / 남은 위험 (1줄)

`ShaderPreset`은 `slot2-gfx`의 동명 타입과 값이 겹치므로, 후속 배선 태스크에서 둘을 잇는
변환 지점을 한 곳으로 정하지 않으면 spelling drift 위험이 남는다(이번 범위 밖).

## 소요 시간

약 20분.
