# Task 99 — 플랫폼별 삽입·배출 효과음 프로필

현재 checkout에서 직접 작업한다. Task98까지 7개 플랫폼의 카트·포트·삽입/배출 곡선은 모두 독립됐지만
소리는 아직 공용 `Sfx::{Insert,Eject}` 녹음 하나씩을 같은 속도와 음량으로 재생한다. 기존 MIT 원본 PCM은
바꾸거나 복제하지 않고, `PlatformSkin`에 방향별 재생 프로필을 넣어 각 플랫폼의 카트 무게와 포트 저항에
맞는 소리로 변형한다. 변형 뒤의 lead를 사용해 접점 소리와 화면 접촉 시점이 계속 맞아야 한다.

누적 호출은 최대 2회다. 이번이 첫 호출이면 결과 보고서에 `누적 호출 1/2`를 기록한다.

## 먼저 읽을 범위

다음 파일과 필요한 선언 주변만 읽는다. 저장소 전체 탐색이나 로그 전문 읽기는 하지 않는다.

- `C:\SLOT2\AGENTS.md`
- `C:\SLOT2\tasks\99-platform-insert-eject-sfx.md`
- `C:\SLOT2\tasks\98-platform-insert-eject-curves.worker-result.md`
- `C:\SLOT2\assets\sfx\PROVENANCE.md`
- `C:\SLOT2\crates\slot2-audio\src\sfx.rs`
- `C:\SLOT2\crates\slot2-audio\src\resample.rs`의 public API 주변
- `C:\SLOT2\crates\slot2-audio\tests\sfx.rs`
- `C:\SLOT2\crates\slot2-ui\src\skin.rs`
- `C:\SLOT2\crates\slot2-ui\tests\skin.rs`
- `C:\SLOT2\crates\slot2\src\app.rs`의 `play`, `advance_insert`, platform 선택 주변
- `C:\SLOT2\crates\slot2\tests\sfx_app.rs`
- `C:\SLOT2\crates\slot2\tests\insert_app.rs`의 SFX quiet-window 단언 주변
- `C:\SLOT2\docs\DESIGN.md` §7 스킨/animation 문단만
- `C:\SLOT2\docs\MILESTONES.md` M6만

`CLAUDE.md`와 `AGENTS.md`의 오케스트레이터 역할 분리·bai-gjc 재위임 조항은 무시하고 직접 실행한다.
나머지 안전·품질 규칙은 유지한다.

## 보존할 계약

- `assets/sfx/insert.pcm`, `eject.pcm`의 바이트와 파일명, 48 kHz mono s16le 형식
- 정상 sink rate에서 base `Sfx::render(rate)`, `lead()`, `seconds()`, `tail()`의 현재 public 의미와 결과
- insert 0.240 s / lead 0.097 s, eject 0.315 s / lead 0.021 s의 원본 계약
- stereo 양쪽에 같은 sample을 쓰는 계약과 sink가 보고한 rate로 resample하는 계약
- `INSERT_S = 0.73`, `INSERT_HOLD_S = 0.28`, `SEATED_AT = 0.45`, `EJECT_S = 0.45`
- core load/refusal/Playing/List 전이, one-shot `sfx_fired`, ring 전체 clip 선기록, idle sink 정책
- Task98의 14개 travel curve와 cart/port/label/shell/borrowed 값

원본 `render()`는 과거 호출자와 테스트를 위해 계속 “녹음 그대로”여야 한다. 플랫폼 변형은 별도 명시적
API로만 적용한다.

## 의미 타입과 변형 API

`slot2-ui::skin`에 작고 copy 가능한 `SoundProfile` 의미 타입을 둔다. UI 크레이트가 audio 크레이트에
의존하지 않도록 속도와 음량만 담는다.

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SoundProfile {
    speed: f32,
    gain: f32,
}
```

- const 생성자와 stable accessor를 제공한다. field는 private여도 된다.
- `speed`는 원본 재생 속도다. 1보다 크면 더 짧고 높은 소리, 1보다 작으면 더 길고 낮은 소리다.
- `gain`은 amplitude 배율이다. 이번 table은 `0 < gain <= 1`이므로 변형 때문에 clipping을 만들지 않는다.
- 생성 시 finite이고 `0.80 <= speed <= 1.20`, `0.70 <= gain <= 1.00`임을 table 오타가 build에서
  드러나는 방식으로 검증한다. NaN도 거부해야 한다.

`slot2-audio::Sfx`에는 이름이 명확한 변형 API를 추가한다. 이름은 Rust 스타일에 맞게 조정해도 되지만
다음 의미를 유지한다.

```rust
pub fn render_styled(self, rate: u32, speed: f32, gain: f32) -> Vec<i16>;
pub fn lead_at_speed(self, speed: f32) -> f32;
pub fn seconds_at_speed(self, speed: f32) -> f32;
pub fn tail_at_speed(self, speed: f32) -> f32;
```

- 속도는 원본 source rate를 바꾸어 기존 resampler 경로로 구현한다. 임의 waveform 합성, clip 연결,
  envelope, 시간 padding은 하지 않는다.
- 실제 정수 source rate로 반올림한다면 render와 lead/seconds/tail 계산이 **같은 유효 속도**를 사용해
  화면 cue와 sample 길이가 어긋나지 않아야 한다.
- gain은 stereo 변환 과정이나 그 뒤에 saturating/checked 방식으로 적용한다. float→i16 overflow나
  debug/release 차이가 없어야 한다.
- `speed`가 finite `0.80..=1.20` 밖이거나 `gain`이 finite `0.0..=1.0` 밖이면 styled render는 빈
  vector, styled timing은 `0.0`을 반환한다. rate 0 또는 현실적인 audio sink 상한 384 kHz를 넘는 rate도
  빈 vector를 반환한다. 이 방어는 panic, 거대한 allocation, divide-by-zero를 막으며 production table은
  생성자에서도 잘못된 값을 막는다.
- `render(rate)`는 `render_styled(rate, 1.0, 1.0)`과 같은 결과여야 하며 기존 exact duration 계약을
  유지한다.
- 변형은 이벤트당 한 번만 수행하고 frame마다 다시 render하지 않는다.

## 플랫폼 프로필 table

`PlatformSkin`에 다음 의미의 두 field를 추가한다.

```rust
pub sfx_in: SoundProfile,
pub sfx_out: SoundProfile,
```

아래 값을 정확히 사용한다.

| Platform | insert `(speed, gain)` | eject `(speed, gain)` | 의도 |
|---|---|---|---|
| GB | `(0.86, 0.92)` | `(0.89, 0.88)` | 가벼운 pak이지만 긴 레일 마찰 |
| GBC | `(0.93, 0.88)` | `(0.96, 0.84)` | GB보다 조금 빠른 투명 shell |
| GBA | `(1.10, 0.86)` | `(1.13, 0.82)` | 낮고 짧은 cart의 빠른 click |
| NES | `(0.82, 1.00)` | `(0.85, 0.96)` | 가장 큰 cart의 낮고 단단한 충격 |
| SNES | `(1.00, 0.96)` | `(1.03, 0.92)` | 넓은 shell의 중립적인 push |
| MD | `(1.16, 0.90)` | `(1.19, 0.86)` | 빠른 connector bite와 release |
| SMS | `(0.96, 0.82)` | `(0.99, 0.78)` | 세로 cart의 가벼운 중간 저항 |

- 7개 insert profile은 pairwise distinct, 7개 eject profile도 pairwise distinct다.
- 같은 플랫폼의 insert와 eject도 서로 다르다.
- table 차이는 테스트 padding이 아니라 실제 render 속도와 amplitude에 모두 반영한다.
- profile은 base PCM을 가리키는 자산 ID가 아니다. 방향은 계속 `Sfx::{Insert,Eject}`가 정하고,
  profile은 그 녹음의 재생 방법만 정한다.

## App 배선과 동기

- `App`은 현재 `platform()`의 skin을 한 번 조회해 Inserting이면 `sfx_in`, Ejecting이면 `sfx_out`을
  선택한다. audio나 App에 두 번째 platform별 match table을 만들지 않는다.
- `play`는 선택한 profile로 clip을 render해 기존처럼 ring에 전부 쓴 다음 sink open을 요청한다.
- insert trigger는 `SEATED_AT - Sfx::Insert.lead_at_speed(sfx_in.speed())`를 사용한다. 속도가 바뀌었는데
  원본 `lead()`를 쓰면 실패다.
- eject는 현재처럼 Ejecting 첫 frame에 한 번 시작한다. 변형된 eject 전체 길이는 `EJECT_S` 안에
  끝나야 하며 List 전환이 tail을 자르지 않아야 한다.
- 빠른/느린 모든 profile에서 insert contact가 `SEATED_AT`과 맞고 tail은 seated hold 안에서 끝나야 한다.
- core load 실패로 eject가 이어져도 insert와 eject가 각각 한 번뿐이어야 한다. idle, row 이동, empty
  shelf는 계속 조용하다.
- sink rate가 48 kHz가 아니어도 속도에 따른 실제 seconds와 cue가 유지돼야 한다.

## 자동 계약 테스트

기존 테스트를 삭제하거나 약화하지 않고 다음을 검증한다.

### `slot2-audio/tests/sfx.rs`

1. base `render`, lead, seconds, tail의 기존 정상 rate 결과가 모두 유지된다. 기존의 rate 0과
   `u32::MAX` 비panic 계약은 empty output으로 명확히 봉인한다.
2. `render_styled(rate, 1, 1) == render(rate)`가 여러 sink rate에서 성립한다.
3. production table의 최저·최고 speed를 포함해 rendered frame duration이 `seconds_at_speed`와 허용
   오차 안에서 맞고 `lead + tail == seconds`다.
4. faster profile은 짧고 slower profile은 길며, gain 1 이하 변형은 같은 speed의 gain 1 출력보다 peak가
   커지지 않는다. 결과는 stereo 좌우 동일하고 silence가 아니다.
5. invalid speed/gain은 empty/0.0, rate 0과 `u32::MAX`는 empty output이고 panic·odd stereo sample·
   비정상 대량 allocation을 만들지 않는다. rate 1은 기존처럼 유한한 정상 반환이어야 한다.

### `slot2-ui/tests/skin.rs`

6. 14개 profile 값이 table과 일치하고 bounds 안이다.
7. insert끼리, eject끼리 pairwise distinct이며 각 플랫폼의 두 방향도 다르다.
8. 기존 14개 curve와 cart/port/label/shell/borrowed table은 바뀌지 않았다.

### `slot2/tests/sfx_app.rs`와 필요한 기존 테스트

9. 최소한 가장 느린 NES와 가장 빠른 MD insert를 실제 App으로 시작해 ring의 frame 수와 sample을
   drain하고 각 skin profile의 `render_styled` 결과와 일치함을 확인한다. 단순히 profile struct만
   비교해서 App 배선을 건너뛰면 안 된다.
10. 위 두 플랫폼의 clip 시작 시각이 각자의 transformed lead를 써서 contact frame과 0.05 s 이내로
    맞고, 원본 lead를 공용으로 썼다면 실패하는 단언이어야 한다.
11. 7개 플랫폼 insert 출력과 7개 eject 출력이 각각 pairwise different임을 frame 길이 또는 실제
    sample 내용으로 확인한다. profile field 차이만 검사하지 않는다.
12. 한 insert와 이어진 실패 eject가 각각 정확히 한 번 발생하고 방향별 expected transformed clip을
    ring에 전부 넘긴다.
13. 기존 empty shelf, idle, row navigation quiet 계약과 animation/core 전이 frame 수를 유지한다.

테스트는 private platform index나 test-only production 분기에 의존하지 않는다. 공개 입력 경로로 shelf를
선택하거나 기존 helper를 확장한다. 고정 texture ID나 draw op 수는 이 태스크와 무관하다.

## 출처와 문서

- `assets/sfx/PROVENANCE.md`에서 두 PCM 파일 자체는 원본 저장소에서 가져온 **unmodified MIT source**로
  계속 명시한다. 파일을 변형했다고 쓰지 않는다.
- 같은 문서에 SLOT2가 runtime에 platform profile의 speed/gain으로 재생하고, contact lead와 duration도
  동일 유효 속도로 계산한다는 파생 재생 동작을 추가한다. 7×2 table은 코드가 단일 source of truth이므로
  문서에 수치를 복제하지 않아도 된다.
- `sfx.rs`의 “played as recorded/no stretching” 모듈 설명은 base `render()`에만 해당한다는 점과 platform
  styled path를 정확히 설명하도록 고친다.
- `docs/DESIGN.md` §7에 `PlatformSkin`의 `sfx_in`/`sfx_out` profile, App의 transformed lead cue,
  원본 PCM 보존을 현재 구현대로 적는다.
- `docs/MILESTONES.md` M6의 복합 항목은 이 작업과 전체 검증이 성공하면 `[x]`로 바꾸고 Task95~99로
  cart/port/curve/sfx가 완료됐다고 적는다. 실기 청감 확인은 별도 남은 검증으로 명시한다.

## 수정 허용 파일

- `C:\SLOT2\crates\slot2-audio\src\sfx.rs`
- `C:\SLOT2\crates\slot2-audio\tests\sfx.rs`
- `C:\SLOT2\crates\slot2-ui\src\skin.rs`
- `C:\SLOT2\crates\slot2-ui\tests\skin.rs`
- `C:\SLOT2\crates\slot2\src\app.rs`
- `C:\SLOT2\crates\slot2\tests\sfx_app.rs`
- `C:\SLOT2\crates\slot2\tests\insert_app.rs` — transformed lead에 맞춘 기존 quiet-window 계약만
- `C:\SLOT2\assets\sfx\PROVENANCE.md`
- `C:\SLOT2\docs\DESIGN.md` — §7 관련 문단만
- `C:\SLOT2\docs\MILESTONES.md` — M6 관련 항목만
- `C:\SLOT2\tasks\99-platform-insert-eject-sfx.worker-result.md`

다른 production, test, asset, 문서, 설정 파일은 수정하지 않는다. `Cargo.toml` dependency 추가는 이
설계에 필요하지 않다. 계약 자체가 틀렸거나 허용 파일 밖 수정이 필요하면 추측 구현하지 말고 실패
보고서에 정확한 이유를 쓴다.

## 범위 밖 및 금지

- PCM 파일 생성·복제·재인코딩·교체, 외부 음원 다운로드, 네트워크 사용
- waveform 합성, EQ/reverb, envelope, clip splice, 방향 녹음 뒤집기
- animation duration/curve, cart/port SVG, layout, core/session/store/menu/i18n 변경
- 새 audio dependency 또는 UI→audio dependency 추가
- 접점 동기를 맞추기 위한 animation frame padding이나 `SEATED_AT` 변경
- 테스트만 통과시키는 silence padding, sample append, platform별 hardcoded App match
- 위임, 커밋, push, 실기·ADB·Samba·SD 카드 접근
- `~/.gjc-bai/agent/*.yml` 등 공용 설정 변경

## 완료 기준

마지막 code/test 변경 뒤 아래 원문을 모두 실행한다. 테스트는 작업자인 가재코드가 수행한다. Codex가
다시 실행할 필요가 없도록 종료 코드와 마지막 결과 줄을 보고서에 남긴다.

```text
cargo fmt --all -- --check
cargo test -p slot2-audio --test sfx
cargo test -p slot2-ui --test skin
cargo test -p slot2 --test sfx_app
cargo test -p slot2 --test insert_app
cargo test --workspace
cargo check -p slot2 --no-default-features --features device
cargo clippy --workspace --all-targets -- -D warnings
powershell -File build/dist-device.ps1
```

- 모든 명령 종료 코드 0.
- 모든 실행된 test binary에서 failed 0, 예상 밖 ignored 0.
- `cargo test --workspace` 총 passed 수는 현재 기능 추가로 기존 Task98 시점보다 줄지 않아야 한다.
- clippy warning 0.
- 배포 마지막 줄 `==> done`이고 `dist-device/System/frontend`가 생성된다.
- 완료 기준 뒤 code/test 변경 금지. 변경했으면 영향받는 명령부터 다시 실행한다.

첫 응답은 최대 5분, 전체는 최대 45분 기다린다. 진행 확인이 필요하면 로그 크기가 아니라
`git status --short`를 사용한다.

## 결과 보고서

`C:\SLOT2\tasks\99-platform-insert-eject-sfx.worker-result.md`에 다음을 기록한다.

- 성공/실패와 누적 호출 수
- `SoundProfile`과 7×insert/eject table
- audio styled render, 유효 속도, gain/overflow 방어 요약
- App profile 선택과 transformed lead/contact/tail 동기 근거
- base PCM/API 불변과 provenance 변경 요약
- DESIGN/MILESTONES 변경 요약
- 각 완료 기준 명령, 종료 코드, 마지막 결과 줄, passed/failed/ignored 수
- 생성·수정 파일과 최종 검증 뒤 code/test 변경 여부
- 실기 청감·접점 동기 확인 항목
- 계약이 틀려 보인 부분이 있으면 해당 줄과 이유

실패해도 보고서를 남긴다. 두 번째 호출까지 실패하면 더 시도하지 않는다. 커밋하지 않는다.
