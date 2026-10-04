# Task 99 — 플랫폼별 삽입·배출 효과음 프로필 (worker-result)

**상태: 성공. 누적 호출 1/2.**

완료 기준 9개 명령 전부 종료 코드 0, 모든 test binary failed 0 / ignored 0, clippy warning 0,
배포 마지막 줄 `==> done` + `dist-device/System/frontend` 생성 확인. 커밋·push·네트워크·실기·ADB·
Samba·SD 카드·공용 설정(`~/.gjc-bai`) 접근 없음. 위임 없이 직접 수행.

---

## 1. SoundProfile과 7×2 table

`slot2-ui::skin::SoundProfile` (copy, private field, const 생성자 + const accessor):

```rust
pub struct SoundProfile { speed: f32, gain: f32 }
impl SoundProfile {
    pub const fn new(speed: f32, gain: f32) -> SoundProfile { .. }  // build 시 bounds 검증
    pub const fn speed(self) -> f32
    pub const fn gain(self) -> f32
}
```

`PlatformSkin`에 `pub sfx_in: SoundProfile`, `pub sfx_out: SoundProfile` 추가. table 값(계약 그대로):

| Platform | insert `(speed, gain)` | eject `(speed, gain)` |
|---|---|---|
| GB | (0.86, 0.92) | (0.89, 0.88) |
| GBC | (0.93, 0.88) | (0.96, 0.84) |
| GBA | (1.10, 0.86) | (1.13, 0.82) |
| NES | (0.82, 1.00) | (0.85, 0.96) |
| SNES | (1.00, 0.96) | (1.03, 0.92) |
| MD | (1.16, 0.90) | (1.19, 0.86) |
| SMS | (0.96, 0.82) | (0.99, 0.78) |

생성자 bounds는 `0.80<=speed<=1.20`, `0.70<=gain<=1.00`이고 row가 `static` initializer라 const 평가에서
즉시 실패한다(table 오타 = build 실패). NaN은 두 비교가 모두 false, ±inf는 한쪽이 false라 거부된다.
14개 profile은 insert끼리/eject끼리 pairwise distinct이고 같은 플랫폼의 두 방향도 다르다(속도 값부터
다름). 모든 gain이 1.0 이하이므로 어떤 프로필도 녹음보다 큰 진폭을 만들지 않는다(clipping 없음).

## 2. audio styled render, 유효 속도, gain/overflow 방어

`slot2-audio::sfx`:

- `render_styled(rate, speed, gain)`, `lead_at_speed(speed)`, `seconds_at_speed(speed)`,
  `tail_at_speed(speed)` 추가. `render(rate)`는 `render_styled(rate, 1.0, 1.0)`로 위임.
- 속도 구현: 새 파형 합성·clip 연결·envelope·시간 padding 없이 **정수 source rate**만 바꾼다.
  `source = round(ASSET_HZ * speed)`, `Resampler::new(source, rate)` — 기존 resampler 경로 그대로.
- **유효 속도**: `effective_speed = source / ASSET_HZ`. render와 lead/seconds/tail이 같은 값을 쓴다.
  table의 14개 speed는 `48000*speed`가 정확히 정수 Hz로 반올림되고 되돌린 값이 원래 f32와 **정확히
  같은지** 확인했다(예: 1.16 → 55680 → 1.16, 0.82 → 39360 → 0.82). 따라서 cue와 sample 길이가
  반올림 때문에 어긋나지 않는다.
- gain: stereo 확장 직전에 mono sample마다 `((s as f32) * gain).round().clamp(i16::MIN as f32,
  i16::MAX as f32) as i16`. f32 clamp 후 cast라 debug/release 차이도 float→i16 overflow도 없다.
  gain<=1이라 peak는 같거나 작다(테스트로 peak 비교).
- 방어: `rate==0` 또는 `rate>384_000`(RATE_MAX), speed NaN/±inf/`0.80..=1.20` 밖, gain
  NaN/±inf/`0.0..=1.0` 밖이면 render는 빈 vector, timing 3종은 `0.0`. panic·과대 allocation·
  divide-by-zero 없음. `rate==1`은 기존처럼 유한한 정상 반환(1 frame).
- `render(rate)`는 정상 rate에서 **이전과 바이트 단위로 같은 결과**다(source rate가 48000, gain 1.0의
  곱이 항등). `render_styled(rate,1,1)==render(rate)`를 22.05/44.1/48/96/192 kHz에서 단언.
  0과 `u32::MAX`는 이제 empty다 — 예전 `u32::MAX`는 보간 10억 회를 도는 경로였다(계약이 요구한 봉인).
- 변형은 `play()` 한 곳에서 이벤트당 1회만 수행하고 frame마다 다시 render하지 않는다.

## 3. App 배선과 동기

- `advance_insert()`가 `slot2_ui::skin::skin(self.platform())`을 **한 번** 조회해 Inserting이면
  `sfx_in`, Ejecting이면 `sfx_out`을 골라 `play(clip, profile)`에 넘긴다. App/audio에 두 번째
  platform match table 없음.
- `play(clip, profile)`이 `clip.render_styled(self.sink_rate, speed, gain)`으로 만든 clip을 기존대로
  ring에 **전부** 쓰고 `SinkRequest::Open`을 요청한다.
- insert trigger는 `SEATED_AT - Sfx::Insert.lead_at_speed(sfx_in.speed())`, eject는 기존처럼 Ejecting
  첫 frame에 한 번. `sfx_fired` one-shot, idle/empty/row 정책 그대로.
- 동기 근거(48 kHz, 가장 느린 NES가 최악):
  - insert 최장 tail = NES(0.82): `0.097/0.82=0.11829` lead, `0.240/0.82=0.29268` 전체,
    tail `0.17439` → clip 종료 `SEATED_AT+0.17439 = 0.6244 < INSERT_S 0.73`(seated hold 안).
  - insert 최속 = MD(1.16): lead `0.08362`, 종료 `0.45-0.0836+0.20690 = 0.5733`.
  - eject 최장 = NES(0.85): 전체 `0.315/0.85=0.37059 < EJECT_S 0.45` → List 전환이 tail을 자르지 않음.
  - eject 최단 = MD(1.19): `0.26471`. 어떤 profile에서도 접점은 `SEATED_AT`에 맞는다.
- sink rate가 48 kHz가 아니어도 같은 공식이라 cue와 실제 seconds가 함께 움직인다(테스트는 48 kHz App,
  audio 테스트는 22.05/48/96 kHz).

## 4. base PCM/API 불변과 provenance

- `assets/sfx/insert.pcm`, `eject.pcm`: **바이트·파일명·형식 변경 없음**(mtime도 그대로, 손대지 않음).
- `Sfx::render/lead/seconds/tail`의 정상 rate 의미와 결과 불변(0.240/0.097, 0.315/0.021 포함).
  stereo 양쪽 동일 sample, sink가 보고한 rate로 resample 계약 유지.
- `resample.rs`는 **수정하지 않았다**(public API 그대로 사용).
- `assets/sfx/PROVENANCE.md`: 두 파일은 계속 **unmodified MIT source**로 명시. runtime 파생 재생
  (platform profile speed/gain으로 읽음, contact lead와 duration을 같은 유효 속도로 계산)을 추가
  서술. 7×2 수치는 코드가 단일 source of truth라 문서에 복제하지 않음. lead 표의 수치는 "recorded
  lead"로 표기해 styled lead와 구분.
- `sfx.rs` 모듈 doc: "played as recorded"를 base `render()`에만 해당한다고 한정하고 styled path를
  정확히 설명.

## 5. DESIGN / MILESTONES 변경

- `docs/DESIGN.md` §7: `PlatformSkin` sketch의 `sfx_in/sfx_out`을 `Sfx`→`SoundProfile`로, "플랫폼별
  효과음은 아직 없다" 삭제, 새 문단으로 profile 의미·단일 source of truth·App의 변형 lead cue·
  원본 PCM 보존·base render 불변을 현재 구현대로 기록.
- `docs/MILESTONES.md` M6: "NES/SNES/MD/SMS 카트·포트 SVG, 삽입 곡선·효과음" 복합 항목을 `[x]`로
  바꾸고 Task95~99로 cart/port/curve/sfx 완료를 적고, 실기 청감 확인은 남은 검증으로 명시.

## 6. 완료 기준 명령 결과 (최종 검증, 16:39:56–16:52:08 KST)

모든 소스·테스트·문서 mtime은 16:39:26 이하이고 검증은 16:39:56에 시작했다(검증 중 code/test 변경 없음).

| # | 명령 | 종료 코드 | 마지막 결과 줄 | passed/failed/ignored |
|---|---|---|---|---|
| 1 | `cargo fmt --all -- --check` | 0 | (출력 없음) | — |
| 2 | `cargo test -p slot2-audio --test sfx` | 0 | `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s` | 14/0/0 |
| 3 | `cargo test -p slot2-ui --test skin` | 0 | `test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s` | 26/0/0 |
| 4 | `cargo test -p slot2 --test sfx_app` | 0 | `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s` | 11/0/0 |
| 5 | `cargo test -p slot2 --test insert_app` | 0 | `test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.67s` | 16/0/0 |
| 6 | `cargo test --workspace` | 0 | 결과 줄 100개. 첫 줄 `test result: ok. 33 passed; ... in 0.47s`, 마지막 줄(마지막 doc-test) `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | **971/0/0** |
| 7 | `cargo check -p slot2 --no-default-features --features device` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.79s`` | — |
| 8 | `cargo clippy --workspace --all-targets -- -D warnings` | 0 | ``Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.19s`` (warning 0줄) | — |
| 9 | `powershell -File build/dist-device.ps1` | 0 | `==> done` (스크립트 마지막 `Step "done"`; cross 빌드 `Finished \`device\` profile ... in 1m 43s`, `dist-device/System/frontend` 3,061,968 bytes 생성 16:52:06, `System/VERSION.txt` = `SLOT2 0.1.0 (a8cb4af)`) | — |

- 각 테스트 binary의 `test result` 줄 수는 1개(workspace는 100개)이고 워크스페이스 전체 합계는 14/26/11/16/971 모두 failed 0, ignored 0.
- `cargo test --workspace` 총 passed 971은 이번 추가분 14개를 포함한다(이전 checkout 957 → 971, 감소 없음).
  삭제·skip된 테스트 없음. audio sfx 7→14, ui skin 23→26, slot2 sfx_app 7→11, 나머지 suite 수 불변.
- 완료 기준 마지막 명령(배포)까지 끝낸 뒤 code/test를 다시 수정하지 않았다.
- 문서와 달리 `docs/HANDOFF-CODEX.md`는 **수정하지 않았다** — 이 태스크의 수정 허용 파일 목록에 없다.

### 계약 항목 ↔ 테스트 대응

- 항목 1~5 → `crates/slot2-audio/tests/sfx.rs`: `the_hard_numbers_of_the_recording_did_not_move`,
  `the_unstyled_render_is_a_styled_one_that_changes_nothing`, `a_styled_clip_lasts_what_its_speed_says`
  (0.80/0.82/1.00/1.19/1.20 × 22.05/48/96 kHz, `lead+tail==seconds`), `faster_is_shorter_and_slower_is_longer`,
  `a_styled_clip_is_no_louder_than_the_recording_and_the_same_in_both_ears`(peak ≤ gain 1, 좌우 동일,
  비무음), `a_speed_or_gain_off_the_table_is_refused`, `a_rate_no_sink_could_report_is_empty_not_a_hang`
  (0/400 kHz/u32::MAX empty, rate 1 정상).
- 항목 6~7 → `crates/slot2-ui/tests/skin.rs`: `every_shelf_plays_the_recordings_its_row_declares`
  (14개 값·bounds), `every_shelf_has_its_own_sound_profiles`, `a_profile_out_of_range_does_not_build`
  (NaN 포함 거부).
- 항목 8: curve table/label/port/shell/borrowed 기존 테스트를 그대로 통과(파일에서 삭제·완화 없음).
- 항목 9 → `a_shelf_plays_the_clip_its_own_profile_renders`(NES·MD, ring에서 drain한 sample을
  `render_styled`와 비교; profile이 neutral이 아님도 확인).
- 항목 10 → `the_contacts_are_cued_off_the_shelfs_own_lead`(heard=at+lead_at_speed가 SEATED_AT과
  0.05 s 이내 + 실제 시작이 공용 lead 예측보다 자기 lead 예측에 가깝다는 단언 — 공용 lead를 썼으면
  실패).
- 항목 11 → `the_seven_shelves_do_not_sound_alike`(7개 플랫폼 insert/eject를 실제 sample로 pairwise
  비교, 각각 그 shelf의 `render_styled`와 일치).
- 항목 12 → `a_refused_cart_makes_one_noise_each_way_and_all_of_each`(ring open 정확히 2회, 삽입/배출
  각각 변형 clip 전체, 최종 `Screen::List`, `insert_seat()==None`).
- 항목 13 → 기존 `a_shelf_that_refuses_a_press_stays_quiet`/`a_still_shelf_asks_for_no_sink`/
  `walking_the_row_is_not_a_noise`, `insert_app`의 `the_clock_is_the_same_on_every_shelf`/
  `the_core_is_not_asked_for_until_the_cart_seats`/`a_cart_that_loads_...` 그대로 통과.
- 테스트는 private platform index나 test-only 분기를 쓰지 않는다. shelf 선택은 R1 입력으로만 한다.

## 7. 생성·수정 파일

수정(허용 목록 안, 10개):
- `crates/slot2-audio/src/sfx.rs` — 모듈 doc, RATE_MAX/bounds, `render_styled`, `lead_at_speed`,
  `seconds_at_speed`, `tail_at_speed`, `effective_source_rate`/`effective_speed`, `render` 위임.
- `crates/slot2-audio/tests/sfx.rs` — 신규 7개 테스트(기존 7개 유지).
- `crates/slot2-ui/src/skin.rs` — `SoundProfile`(const new/accessor), `sfx_in`/`sfx_out` field, 7행 값,
  모듈·field 문서.
- `crates/slot2-ui/tests/skin.rs` — 신규 3개 테스트(기존 23개 유지).
- `crates/slot2/src/app.rs` — `play(clip, profile)` + `render_styled`, `advance_insert`의 단일 skin
  조회/변형 lead trigger.
- `crates/slot2/tests/sfx_app.rs` — 신규 4개 테스트 + helper(`app_on`, `until_clip`, `two_clips`,
  `expected`, `assert_clip`, `drain`), 기존 duration 단언 2곳을 그 shelf의 변형 길이로 갱신.
- `crates/slot2/tests/insert_app.rs` — quiet-window를 `Sfx::Insert.lead_at_speed(skin.sfx_in.speed())`
  기준으로 갱신(허용 범위 그대로).
- `assets/sfx/PROVENANCE.md`, `docs/DESIGN.md`(§7), `docs/MILESTONES.md`(M6).
- `tasks/99-platform-insert-eject-sfx.worker-result.md`(이 보고서).

생성한 다른 파일 없음. `Cargo.toml` dependency 추가 없음. `dist-device/`는 배포 스크립트가 재생성한
산출물이며 git 추적 대상이 아니다(`git ls-files dist-device` = 0).

참고: 검증 중 `cargo test --workspace`가 내장 오버레이 생성기 테스트로 `assets/overlays/GB/720x720.png`를
다시 썬다(16:42:46). 이 폴더도 git 추적 대상이 아니고 생성기 테스트가 byte 동일성을 스스로 단언하는
기존 동작이다(내가 추가한 변경 아님). 이 외에 검증 시작(16:39:56) 뒤 바뀐 파일은 이 보고서뿐이다.

**최종 검증 뒤 code/test 변경 없음**: 마지막 소스·테스트·문서 수정은 16:39:26 KST(sfx.rs: 검증 중 오탈자
확인을 위해 doc 주석을 한 번 고쳤다 되돌렸고 내용은 byte 동일), 완료 기준 실행은 16:39:56에 시작해
16:52:08(배포)에 끝났다. `assets/sfx/*.pcm`은 01:30:49 그대로 손대지 않았다. 검증 후 남긴 임시 파일 없음.

## 8. 실기에서 사용자가 확인할 것

- 7개 플랫폼에서 삽입/배출 소리가 서로 다르게 들리는지: NES가 가장 낮고 느리게, MD가 가장 빠르고
  짧게, GB/GBC/SMS/SNES/GBA가 그 사이에서 구분되는지.
- **접점 동기**: NES(느린 clip)와 MD(빠른 clip)에서 카트가 슬롯에 닿는 순간과 소리의 접점이 함께
  오는지. 느린 프로필일수록 clip이 일찍 시작해 화면 접촉 전에 이미 소리가 나기 시작한다.
- 배출 시 화면이 List로 돌아갈 때 소리 tail이 잘리지 않는지, 그리고 ring 교체 때문에 생기는 이음
  (pop/click)이 없는지.
- 삽입 중 코어 로드(약 1초 stall) 동안 소리가 끊기지 않는지(기존 one-shot ring 계약 유지).
- 청감상 gain 하한(SMS eject 0.78, MD eject 0.82)이 지나치게 작지 않은지.

## 9. 계약이 틀려 보이는 부분 / 계약과 다르게 구현한 곳

1. **`render`의 상한 도입**: 계약이 `rate 0`과 `u32::MAX`를 empty로 "봉인"하라고 했으므로
   `render_styled`에 `rate > 384_000 → empty`를 넣고 `render`를 그 위임으로 바꿨다. 결과적으로
   384 kHz 초과 rate의 `render`는 이전과 달리 empty다(예전엔 사실상 무한 루프). "정상 sink rate에서
   기존 결과 유지"는 지켰지만, 상한 자체는 계약이 명시하지 않은 새 경계다.
2. **`RangeInclusive::contains`는 const가 아니다**: `SoundProfile::new`는 `static` table에서 const
   평가돼야 하는데 `(0.80..=1.20).contains(&speed)`는 E0658로 컴파일 실패한다(clippy는 반대로
   `manual_range_contains`를 요구한다). 그래서 생성자만 `assert!(0.80 <= speed)` /
   `assert!(speed <= 1.20)` 두 줄로 나눴다(각각 단일 비교라 clippy도 통과). `#[allow]`는 쓰지 않았다.
   audio 쪽은 const가 아니므로 `.contains()`를 쓴다.
3. **항목 10의 0.05 s 허용 오차만으로는 변형 lead와 원본 lead를 구분할 수 없다**: 60 Hz frame
   양자화에서 MD insert는 `0.45-0.0836=0.36638`과 `0.45-0.097=0.353`이 모두 22번째 frame(0.36667)에
   걸린다. 그래서 "실제 시작 시각이 자기 변형 lead 예측과 공용 lead 예측 중 어디에 더 가까운가"라는
   단언을 추가로 넣었다(공용 lead를 쓰면 그 단언이 실패한다). 계약 문구가 요구한 "원본 lead를 썼다면
   실패하는 단언"은 이 형태로만 성립한다.
4. **기존 sfx_app 테스트 2개의 기대값 갱신**: `the_whole_clip_is_handed_over_at_once`와
   `coming_back_out_has_its_own_noise`는 "clip 길이 = 원본 seconds×48000"을 단언하고 있었다. 프로필
   재생이 들어오면 GBA(1.10/1.13)에서 그 값은 실제 길이가 아니므로, 그 shelf의 `render_styled` 길이로
   바꿨다. 삭제·약화가 아니라 새 계약에 맞춘 정정이다(ring이 slot 하나를 비워 두므로 ±1 frame 허용).
5. 참고(계약 위반 아님): 이 checkout은 `core.autocrlf=true` 환경이라 56개 파일이 worktree/CRLF
   차이로 `git status`에 modified로 뜨고, 그중 상당수(`crates/slot2/src/session.rs`,
   `docs/HANDOFF-CODEX.md`, Task95~98 소산 등)는 **다른 작업의 미커밋 변경**이다. 나는 허용 목록의
   10개 파일만 손댔고 그 외 파일은 건드리지 않았다.
