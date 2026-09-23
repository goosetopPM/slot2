# SLOT2 설계 문서

BaseOS 위에서 동작하는 Rust 프론트엔드 + 인프로세스 libretro 호스트. 결정 근거는 [DECISIONS.md](DECISIONS.md), 일정은 [MILESTONES.md](MILESTONES.md).

원본 참고: [brandonkowalski/slot](https://github.com/brandonkowalski/slot) (MIT). 로컬 읽기 전용 체크아웃 `C:\Users\gyuha\slot-2`.

---

## 1. 레이어

```
┌──────────────────────────────────────────────────────────┐
│ SLOT2                                                    │
│  ui ─ screens · skins · faces        i18n ─ Fluent 번들   │
│  retro ─ libretro host · quirks · registry               │
│  gfx ─ surface · compositor · shader · overlay · scaling │
│  audio · input · store                                   │
│  platform ─ device profile · BaseOS 계약 · power/sysfs   │
├──────────────────────────────────────────────────────────┤
│ BaseOS (무수정)  fbdev/EGL(libmali) · ALSA · evdev · sysfs │
│                 frontend-session → System/frontend        │
├──────────────────────────────────────────────────────────┤
│ 스톡 Anbernic 부트로더 · 커널 · 드라이버 (H700)           │
└──────────────────────────────────────────────────────────┘
```

원칙: BaseOS 저장소를 포크하거나 수정하지 않는다. BaseOS가 바뀌면 `platform` 크레이트만 따라간다.

---

## 2. BaseOS 계약 (`overlay/usr/sbin/frontend-session` 기준)

| 항목 | BaseOS 동작 | SLOT2 대응 |
|---|---|---|
| 카드 마운트 | TF2 `/dev/mmcblk1p1` 우선, 없으면 TF1 데이터 파티션 `/dev/mmcblk0p7`. 둘 다 `/mnt/sdcard` (`/mnt/SDCARD` 심링크). vfat는 `utf8,shortname=mixed`, exfat 커널 드라이버 | 카드 루트 = **cwd**. cwd가 비정상이면 `/mnt/sdcard` 폴백. 한글 파일명 문제 없음 |
| 실행 우선순위 | `System/frontend` → `System/launch_frontend.sh` → `System/slot` → NextUI → spruce | 바이너리를 **`System/frontend`** 로 배치. 같은 카드의 `System/slot`은 무시됨(우선순위 낮음) |
| 실행 방식 | `cd $SD; exec /lib/ld-linux-aarch64.so.1 System/frontend`, 환경변수 없음, exec 비트 불필요 | `SLOT_ROOT` 같은 env 의존 금지 |
| 로그 | stdout/stderr → `/tmp/frontend.log` (휘발) | stderr에 쓰고, 추가로 카드 `System/slot2.log`에 회전 기록 |
| 종료 | busybox init이 재실행(respawn). 종료 코드 규약 없음 | 정상 종료 = 재시작. 전원 끄기는 `baseos-poweroff`, 재부팅은 `baseos-reboot` 호출 |
| 기기 식별 | `/etc/baseos-release` 의 `BASEOS_TARGET=<id>` | `platform::detect()`가 읽어 프로필 로드 |
| 업데이트 | BaseOS 자체는 `.bosupd` | SLOT2 업데이트는 `System/` 폴더 교체 |

---

## 3. 크레이트 구성

```
SLOT2/
├─ Cargo.toml                 workspace (resolver 2), profile.device = release + fat LTO
├─ crates/
│  ├─ slot2/                  bin. 부팅, 메인 루프, device_app / host_app
│  ├─ slot2-platform/         BASEOS_TARGET → DeviceProfile, 전원·뚜껑·백라이트·배터리 sysfs, poweroff/reboot, net(wpa_supplicant/udhcpc 제어)
│  ├─ slot2-gfx/              Surface(fbdev EGL | host glutin), Compositor, ScalePolicy, ShaderPreset, Overlay
│  ├─ slot2-ui/               화면(Screen) 상태기계, PlatformSkin, faces(래스터 캐시), 스팬 레이아웃
│  ├─ slot2-i18n/             Fluent 번들 로더, JOSA/BTN/PLURAL 함수, 언어팩 탐색
│  ├─ slot2-text/             폰트 폴백 체인(fontdue), 글리프 캐시, 폭 측정
│  ├─ slot2-input/            evdev(device) | gilrs+winit(host) → Event{Button, Axis}, 제스처(hold/double-tap/chord)
│  ├─ slot2-audio/            ALSA dlopen(device) | cpal(host), 링버퍼, 리샘플, SFX
│  ├─ slot2-retro/            libretro FFI 호스트(코어 무관), quirks/{mgba,gpsp,fceumm,snes9x,gpgx}, Registry
│  └─ slot2-store/            카드 레이아웃, 스캔, 세이브/스테이트, 설정 ini, 원자적 쓰기, 마이그레이션
├─ cores/                     코어별 build.sh + 핀 커밋 + *.patch (mgba, gpsp, fceumm, snes9x, genesis_plus_gx)
├─ assets/                    skins/*.svg, fonts/, sfx/, lang/{en,ko}.ftl, overlays/, shaders/
├─ build/cross.Dockerfile     debian:bullseye amd64 + rust + gcc-aarch64-linux-gnu + cmake
├─ taskfile.yml               build / test / core:* / dist:device / deploy:device / log:device
├─ .github/workflows/         ci.yml (check+test), release.yml (dist zip)
└─ docs/
```

원본 크레이트 → SLOT2 매핑: `slot-retro`→`slot2-retro`(호스트 로직 대부분 이식), `slot-store`→`slot2-store`, `slot-power`→`slot2-platform`, `slot-input`→`slot2-input`, `slot-gfx`/`slot-ui`는 구조를 새로 잡고 필요한 조각만 이식. `slot-sfxcut`은 `tools/`로 후순위.

---

## 4. 기기 프로필 (`slot2-platform`)

```rust
struct DeviceProfile {
    target: &'static str,        // rgsp, rg35xxsp, ...
    geometry: Geometry,          // W640x480 | W720x480 | W720x720
    has_lid: bool,               // SP 계열
    has_sticks: bool,            // H / V / CubeXX
    paths: SysfsPaths,           // backlight, battery, rumble, lid switch (probe로 보정)
}
```

| BASEOS_TARGET | 지오메트리 | 뚜껑 | 스틱 |
|---|---|---|---|
| `rgsp` **(개발 기준)** | 720×480 | ✓ | – |
| `rg34xx` | 720×480 | – | – |
| `rg34xxsp` | 720×480 | ✓ | – |
| `rg35xxsp` | 640×480 | ✓ | – |
| `rg35xxplus` `rg35xxh` `rg35xxpro` | 640×480 | – | h: ✓ |
| `rg40xxh` `rg40xxv` | 640×480 | – | ✓ |
| `rgcubexx` | 720×720 | – | ✓ |

미확인 타깃: fbdev 해상도를 probe해 지오메트리를 정하고 뚜껑·스틱은 없음으로 간주. 호스트 빌드는 `SLOT2_GEOMETRY=640x480|720x480|720x720`.

---

## 5. 표시 (`slot2-gfx`)

**파이프라인**: 코어 프레임(RGB565/XRGB8888) → 텍스처 → [셰이더 패스] → 스케일 배치 → [오버레이] → UI 레이어 → swap. 60Hz 타이밍은 프레임 루프가 보장(스왑 인터벌 무시 드라이버 대비).

**스케일 정책** (플랫폼별 기본값, 사용자 토글):

| 모드 | 동작 |
|---|---|
| `Integer` | 정수 배율 최대, PAR 무시 |
| `AspectFit` | 플랫폼 PAR 반영, 비율 유지 최대 |
| `Fill` | 화면 꽉 채움 |

| 플랫폼 | 원본 | PAR | 640×480 | 720×480 | 720×720 |
|---|---|---|---|---|---|
| GBA | 240×160 | 1:1 | 2× | **3×** | 3× |
| GB/GBC | 160×144 | 1:1 | 3× | 3× | **4×** |
| NES | 256×240 (오버스캔 8px 크롭 옵션) | 8:7 | 2× | 2× | 2× |
| SNES | 256×224 / 512×448 | 8:7 | 2× | 2× | 2× |
| MD | 320×224 / 256×224 | 4:3 맞춤 | 2× | 2× | 2× |
| SMS | 256×192 | 8:7 | 2× | 2× | 2× |

위 배율표는 어디에도 저장하지 않는다. 플랫폼의 원본 크기와 패널 크기로부터 `ScalePolicy::Integer`가 계산해 내는 값이고, 표 자체는 `crates/slot2/tests/scaling.rs`가 실행 가능한 형태로 들고 있다. 설계와 코드가 서로 모르게 어긋나는 것을 막는 장치다.

PAR은 플랫폼 상수가 아니다. MD는 256·320 두 폭을 오가면서 둘 다 같은 4:3 화면을 채웠으므로, 픽셀 비율이 아니라 **표시 비율**로 모델링한다(`Aspect::Display`). 덕분에 폭이 바뀌어도 화면이 튀지 않는다 — 이것이 M2 수용 기준 "MD 폭 전환에서 화면 깨짐 없음"의 기하학적 알맹이다. NES·SNES·SMS는 8:7 픽셀(`Aspect::Pixel`), GB 계열과 GBA는 정사각(`Aspect::Square`).

**지오메트리 변경**: `SET_GEOMETRY` / `SET_SYSTEM_AV_INFO` 콜백 시 텍스처 재할당·배치 재계산 (MD 폭 전환, SNES 하이레즈). 배치는 매 프레임 코어가 보고한 크기에서 다시 계산하므로 별도 처리가 필요 없고, 텍스처는 크기가 달라진 프레임에서 재할당된다.

**셰이더 프리셋**: `None`, `SharpBilinear`, `Lcd3x`(휴대기 기본), `ZfastCrt`(거치기 기본), `Scanline`. GLSL ES 단일 패스. 표준 uniform: `TextureSize`, `InputSize`, `OutputSize`, `FrameCount` — 후일 `.glslp` 호환의 기반.

**오버레이**: `assets/overlays/<platform>/<geometry>.png` 및 카드 `System/Overlays/` 덮어쓰기. 게임 레이어 위, UI 레이어 아래.

**UI 안전 영역**: 640×480을 중앙 배치. 오프셋 `((W-640)/2, (H-480)/2)`. 확장 영역은 배경·선반 연장. HUD(배터리·시계)는 실제 화면 모서리에 고정.

---

## 6. 에뮬 호스트 (`slot2-retro`)

**host/** — libretro API 구현. 코어 특정 지식 없음.
- 로드: `libloading`으로 `.so` 열기, `retro_set_environment` → 콜백 등록.
- env 콜백 지원 목록(초기): `SET_PIXEL_FORMAT`, `GET_SYSTEM_DIRECTORY`, `GET_SAVE_DIRECTORY`, `GET_VARIABLE`/`SET_VARIABLES`/`GET_VARIABLE_UPDATE`, `GET_CORE_OPTIONS_VERSION`, `SET_CORE_OPTIONS(_V2)`, `SET_GEOMETRY`, `SET_SYSTEM_AV_INFO`, `GET_LOG_INTERFACE`, `SET_INPUT_DESCRIPTORS`, `SET_CONTROLLER_INFO`, `GET_RUMBLE_INTERFACE`, `GET_INPUT_BITMASKS`, `SET_SUPPORT_NO_GAME`(거부), `GET_LANGUAGE`, `SET_FRAME_TIME_CALLBACK`, `GET_AUDIO_VIDEO_ENABLE`. 미지원은 `false` 반환 + 로그.
- 세이브: `retro_get_memory(SAVE_RAM)` 주기적 flush + 종료 시. 스테이트: `retro_serialize`.
- 되감기: 직렬화 링버퍼, 플랫폼별 `RewindBudget{interval_frames, ring_bytes}`.
- 프레임 콜백: 비디오 → gfx 텍스처, 오디오 → 링버퍼(리샘플), 입력 폴 → input 스냅샷.

**quirks/** — 코어별 모듈. 트레이트 하나:
```rust
trait CoreQuirks {
    fn options(&self, ctx: &LaunchCtx) -> Vec<(String, String)>;   // BIOS 유무·팔레트 등 반영
    fn input_map(&self, platform: Platform) -> InputMap;
    fn post_load(&mut self, host: &mut Host) {}                    // 링크 등 특수 초기화
    fn color_grade(&self) -> Option<Grade> { None }                 // mGBA GB 색보정 등
}
```

**registry** — 플랫폼 테이블(코드 상수, 후일 `System/platforms.toml` 덮어쓰기 가능):
```rust
struct PlatformDef {
    id: Platform, folder: "GBA", exts: &["gba"],
    cores: &[Core::Mgba, Core::Gpsp], default_core: Core::Mgba,
    native: (240,160), par: Par::Square,
    scale_default: [Integer, Integer, Integer],   // 지오메트리별
    shader_default: Lcd3x, skin: SkinId::Gba,
    bios: &["gba_bios.bin"], rewind: RewindBudget{..},
}
```

**코어 빌드** (`cores/<name>/build.sh`): 핀 커밋 + 패치, `.meta` 스탬프로 재빌드 판정, 산출물 `vendor/<name>_libretro.so`. 원본 mGBA·gpSP 패치는 그대로 가져온다.

**코어가 신고한 값은 그대로 받는다.** 코어의 `sample_rate`·`fps`는 호스트가 "상식적인 범위"로 다듬을 대상이 아니다. mGBA는 GBA 오디오를 **65536 Hz**(`GBA_OUTPUT_RATE`, GBA 사운드 FIFO의 32768 Hz의 두 배)로 내보내고 그렇게 신고한다. 한때 호스트에 "50 kHz 넘으면 32768로 간주" 클램프가 있었고, 그 탓에 모든 GBA 게임이 한 옥타브 낮게·두 배 느리게 재생됐다. 게다가 초과분이 오디오 링을 넘쳐 잘려 나가면서 템포만 얼추 맞아 보여, **명백한 고장이 막연한 잡음처럼 위장**됐다. 검증은 범위가 아니라 **자기일관성**으로 한다 — 신고한 레이트와 실제로 넘겨준 샘플 수가 일치하는지(`the_declared_sample_rate_is_what_the_core_actually_produces`).

**프레임 페이싱과 동적 레이트 제어.** 루프는 1/60이 아니라 **코어의 `fps`**로 재운다(GBA 59.7275). 60 Hz로 돌리면 코어가 0.46 % 많은 오디오를 만들고, 링이 찬 뒤로는 매 프레임 꼬리가 버려져 초당 60번 이어붙는 소리가 난다. 남는 표류(호스트 시계 ↔ 코덱 48 kHz 크리스털)는 `drc_trim`이 링 점유율을 보고 출력 레이트를 ±0.5 % 안에서 미세 조정해 흡수한다. 0.5 %는 음정 변화로 들리지 않고, 대안(샘플을 버리거나 반복하기)은 곧 클릭이다.

---

## 7. UI (`slot2-ui`)

**화면 상태기계**: `Shelf` → `Inserting` → `Playing` ↔ `InGameMenu`(→ `StateSwitcher`) → `Ejecting` → `Shelf`. 선반 부속: `ShelfMenu`(언어·플랫폼 기본 화면 설정·부팅 로고·연동 모드·시계·About), `CorePicker`(코어 2개 이상인 플랫폼만), `PowerMenu`, `SyncScreen`, `Toast`, `Refusal`.

**인게임 메뉴 (D-23)**: MENU 탭 → 코어 정지·오디오 뮤트 → 마지막 프레임을 어둡게 깔고 패널. 항목: 계속하기 / 세이브 스테이트 / 치트 / 화면(스케일·셰이더·오버레이, 변경 즉시 뒤 화면에 반영) / 코어 / 기기 / 꺼내기. 닫으면 그 자리에서 재개.

**게임 중 조작**

| 조작 | 동작 |
|---|---|
| MENU 탭 | 인게임 메뉴 |
| MENU 홀드 | 스테이트 저장 후 꺼내기 |
| SELECT+R1 / SELECT+L1 | 즉시 세이브 / 최근 로드 |
| L2 홀드 | 되감기 |
| R2 홀드 / 더블탭 | 빨리감기 / 고정 |
| SELECT+↑↓ / ←→ | 밝기 / 블루라이트 |
| VOL± | 볼륨, 동시 누름 = 뮤트 |

**설정 계층**: 전역 `System/slot2.ini` → 플랫폼 기본값(registry) → 게임별 `System/games/<PLAT>/<stem>.ini`. 인게임 메뉴는 게임별에, 선반 메뉴는 플랫폼/전역에 쓴다. 게임별 값이 있으면 항목 옆에 표시하고 "플랫폼 기본값으로" 항목을 둔다.

**PlatformSkin** (D-08):
```rust
struct PlatformSkin {
    cart: Svg, cart_size: (u32,u32), label_rect: Rect, shell_palette: &[Shell],
    port: Svg, mouth_rect: Rect, seat_depth: f32,
    insert: Curve, eject: Curve, sfx_in: Sfx, sfx_out: Sfx,
}
```
GBA·GB·GBC는 원본 SVG 이식. NES/SNES/MD/SMS는 제작 전까지 GBA 스킨 폴백.

**faces**: 텍스트·복합 요소는 CPU 래스터 → 텍스처, 내용 변경 시에만 재생성. GL은 쿼드 합성만(원본 방식).

**레이아웃 규칙**: 텍스트 폭은 항상 측정. 고정 폭 슬롯 금지. 버튼 힌트 행은 스팬 열을 순서대로 흘림.

---

## 8. 다국어 (`slot2-i18n`, `slot2-text`)

```
assets/lang/{en,ko}.ftl (내장)  +  System/Lang/<code>.ftl (카드, 덮어쓰기)
        │
        ▼
FluentBundle  ── 함수: JOSA(text, "을/를"), BTN("menu"), PLURAL(기본 내장)
        │
        ▼
Vec<Span>  = [Text(..) | Btn(Button) | Icon(Glyph)]
        │
        ▼
slot2-text: 폰트 폴백 체인 [UI 라틴(언어팩 font= 로 교체 가능) → 한글·CJK(Noto Sans KR 서브셋, 지연 로딩) → 아이콘] · 폭 측정 · 글리프 캐시
        │
        ▼
face 래스터 → 텍스처
```

- **JOSA 규칙**: 마지막 글자가 한글 음절이면 `(cp-0xAC00)%28`로 받침 판정, `으로/로`는 ㄹ받침(8) 예외. 숫자는 읽는 소리로 판정. 그 외(영문·기호)는 병기형 `을(를)`.
- **어순**: 코드에서 문자열 결합 금지. 모든 문장은 완성 템플릿 + 이름 붙은 자리표시자.
- **언어 선택**: 설정 항목. 기본 `en`. 언어팩은 `font = "..."`로 자기 폰트 지정(내장 또는 `System/Fonts/`).
- **한글·CJK 폰트**: Noto Sans KR(OFL) 서브셋 1개를 항상 내장 — 게임 제목의 일본어·한자를 언어 설정과 무관하게 렌더해야 하기 때문. `pyftsubset`으로 라틴+한글 음절+가나+KS X 1001 한자+기호로 제한(≈17k 글리프, 5~6MB). fontdue는 로드 시 전 글리프를 파싱하므로 부팅 경로에서 빼고 지연 로딩(첫 미스 또는 백그라운드). 대체 라벨 인쇄(`Labels/`에 이미지 없을 때)도 같은 체인.
- **정렬**: 코드포인트 정렬(한글 음절은 자모순 배열이라 가나다순 성립).

---

## 9. 카드 레이아웃 (`slot2-store`)

```
<card root>/
├─ System/
│  ├─ frontend             SLOT2 바이너리 (BaseOS 계약)
│  ├─ cores/*.so           mgba, gpsp, fceumm, snes9x, genesis_plus_gx
│  ├─ licenses/            코어 라이선스·소스 아카이브·패치
│  ├─ Lang/*.ftl           선택: 언어팩 덮어쓰기
│  ├─ Fonts/               선택
│  ├─ Overlays/            선택
│  ├─ slot2.ini            전역 설정
│  ├─ games/<PLAT>/<stem>.ini   게임별 설정(코어 선택·스케일·셰이더)
│  ├─ cheats/<PLAT>/<stem>.cht  치트(RetroArch .cht 포맷, 데스크탑이 씀, D-21)
│  ├─ manifest.json        연동 프로토콜: SLOT2가 씀 (§12.3)
│  ├─ sync.lock            연동 프로토콜: 데스크탑이 전송 중 생성
│  ├─ wifi.toml            연동 프로토콜: 데스크탑이 씀, SSID/PSK 목록
│  └─ slot2.log
├─ BIOS/                   gb_bios.bin gbc_bios.bin gba_bios.bin bios_{U,E,J}.sms  (전부 선택)
├─ Games/{GB,GBC,GBA,NES,SNES,MD,SMS}/
├─ Labels/{...}/<stem>.png
├─ Saves/{...}/<stem>.sav
├─ States/{...}/<stem>/{resume.state, N.state, N.png}
└─ Wallpapers/*.png
```

원본 slot 카드(`Games/`, `Labels/`, `Saves/`, `States/`)와 **호환**을 유지한다 — 기존 slot 사용자가 `System/`만 바꾸면 그대로 쓸 수 있게. 확장자 매핑: GB `.gb`, GBC `.gbc`, GBA `.gba`, NES `.nes`, SNES `.sfc .smc`, MD `.md .gen .bin`, SMS `.sms`. 폴더+확장자로만 플랫폼 판정(헤더 스니핑 없음). 쓰기는 전부 원자적(temp + rename).

---

## 10. 입력 (`slot2-input`)

- device: evdev 직접 읽기(`/dev/input/event*`), 기기 프로필의 키코드 맵. 축(스틱·L2/R2 아날로그)은 `Axis` 이벤트로 올림(D-15).
- host: winit 키보드 + gilrs 게임패드.
- 제스처 계층: hold(MENU 꺼내기), double-tap(MENU 스테이트 스위처, R2 빨리감기 고정), chord(SELECT+R1 등). 원본 `gesture.rs` 이식.
- 플랫폼별 논리 버튼 맵은 registry/quirks 소유(SNES 6버튼, MD 6버튼 등).
  - MD는 코어에게 평범한 `RETRO_DEVICE_JOYPAD`를 넘긴다. Genesis Plus GX는 그때 카트리지의 I/O 지원 필드를 읽어 **3버튼 게임에는 3버튼 패드를** 준다. 6버튼을 강제하면 6버튼을 모르는 게임이 패드를 아예 못 읽는 사고가 난다. 우리 쪽은 여섯 개를 전부 매핑해 두고, 쓰이지 않는 셋은 비용이 0이다.
  - MD 바닥줄 A/B/C는 libretro의 Y/B/A, 윗줄 X/Y/Z는 L/X/R, Mode는 Select(코어 `libretro.c`). 기기 다이아몬드 배치에 얹으면 A/B/C가 왼쪽·아래·오른쪽(엄지가 훑는 호), X/Y/Z가 L1·위·R1이 된다.

---

## 11. 빌드·배포

- `build/cross.Dockerfile`: `debian:bullseye` amd64, rustup + `aarch64-unknown-linux-gnu` 타깃, `gcc-aarch64-linux-gnu g++-aarch64-linux-gnu cmake make git file`. 링커 env `CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc`. glibc 2.31 하한(BaseOS 2.35 추정보다 낮아 전방 호환).
- `task build:device` → `target-device/device/frontend`. `task dist:device` → `dist-device/`(위 카드 레이아웃 `System/` 완성본 + 라이선스·소스). `task deploy:device` → adb push + reboot.
- 호스트: `cargo run -p slot2`(창 720×480, `SLOT2_ROOT=./sdcard`). Windows 네이티브가 막히면 WSL2+WSLg.
- CI: `ci.yml`(fmt, clippy host+device, test), `release.yml`(태그 → dist zip + 라이선스 동봉).

---

## 12. 데스크탑 연동과 카드 프로토콜 (D-19, D-20)

### 12.1 BaseOS가 제공하는 것 (소스 확인)
- 부팅 시: `8821cs.ko` 로드, `rfkill unblock wifi`, `ip link set wlan0 up`, dropbear sshd(root / `baseos.conf` `ssh_password`), adb(USB), `usb-storage-mode`(MENU 누른 채 케이블 연결).
- DHCP 이벤트 시: `udhcpc/default.script` → `baseos-mdns` → avahi로 `<hostname>.local` 광고. hostname 기본 = `BASEOS_TARGET`(예: `rgsp.local`).
- **하지 않는 것**: wpa_supplicant 실행. AP 접속은 프론트엔드 책임.

### 12.2 SLOT2 연동 모드 (`slot2-platform::net`, `slot2-ui::SyncScreen`)
```
사용자: 설정 → 연동 모드
  1. 꽂힌 카트 꺼냄 → Saves/States flush, 파일 핸들 닫기
  2. Wi-Fi 접속: System/wifi.toml → /tmp/wpa_supplicant.conf 생성 →
     wpa_supplicant -B -i wlan0 -c ... → udhcpc -i wlan0   (mDNS는 BaseOS가 이어받음)
  3. System/manifest.json 갱신
  4. 화면: hostname.local · IP · 상태(연결 대기 / 전송 중 / 완료), 게임 진입 차단
  5. System/sync.lock 감시: 생성 → "전송 중", 삭제 → 재스캔 → 선반 복귀
  6. 모드 종료 시(설정이 '상시 접속'이 아니면) wpa_supplicant/udhcpc 종료
```
연동 모드 밖에서는 네트워크 프로세스가 없다. 프레임 루프에 네트워크 코드 없음.

### 12.3 카드 프로토콜 (두 앱이 공유하는 계약)
| 파일 | 작성자 | 내용 |
|---|---|---|
| `System/manifest.json` | SLOT2 (연동 모드 진입·부팅 시) | `device{target, hostname, slot2_version}`, `platforms[]`, `files[]{path, size, mtime, sha1?}` — Games/Labels/Saves/States/BIOS 대상 |
| `System/sync.lock` | 데스크탑 | 존재 = 전송 중. 내용: 앱 버전, 시작 시각 |
| `System/wifi.toml` | 데스크탑 | `[[network]] ssid, psk` 목록. 기기에서는 선택만 |
| `System/sync.log` | 데스크탑 | 마지막 동기화 요약(추가/삭제/충돌) — SLOT2가 완료 토스트에 표시 |

세이브 키 = `<PLAT>/<stem>`. 데스크탑에서 ROM 이름을 바꾸면 Saves/Labels/States/games ini도 함께 옮기는 것은 데스크탑 책임. 충돌은 mtime+해시로 판정, 데스크탑 라이브러리가 세이브 이력을 보관.

### 12.4 데스크탑 앱 (별도 저장소, M8)
- 핵심 추상화 `CardRoot` 백엔드: `LocalPath`(SD 리더·USB 저장소 모드) | `Sftp`(`<hostname>.local`, dropbear). 동기화 로직은 백엔드 무관.
- 기능: ROM 라이브러리 → 기기 롬셋 선택 → diff 적용, 세이브 양방향 동기화·이력, 라벨 생성(원본 Cart Studio 역할), `wifi.toml`·게임별 ini 편집.
- 스택은 M8에서 결정. Rust(Tauri)면 `slot2-store`의 레이아웃·manifest 코드를 크레이트로 공유 가능.
- v2 후보: SLOT2 내장 에이전트(HTTP + mDNS 서비스) → `CardRoot::Agent` 백엔드 추가. SSH 비밀번호 불필요, 앱 내 진행률.

---

## 13. 검증 대기 항목

실기·환경에서 확인해야 확정되는 것. 각 항목은 해당 마일스톤에서 닫는다.

원본 로그는 `docs/device/rgsp-diag.txt`(진단)와 `docs/device/rgsp-input.txt`(입력 프로브).

| # | 항목 | 확인 방법 | 마일스톤 | 결과 |
|---|---|---|---|---|
| V-1 | RG SP의 실제 버튼 구성(L2/R2 유무, 아날로그 축 유무), evdev 장치명·키코드 | `adb shell "cat /proc/bus/input/devices"` + `evtest` | M0 | ✅ 2026-09 실기. evdev 3개(`axp2202-pek`/`ANBERNIC-keys`/`dierct-keys-polled`), 아날로그 없음, L2/R2 있음. **벤더 드라이버가 리눅스 게임패드 관례를 따르지 않음**: 페이스/숄더가 `BTN_SOUTH..BTN_START`(304–315)에 한 칸씩 밀려 배치되고 D-패드는 키가 아니라 `ABS_HAT0X/Y`(±1). 메뉴키는 312와 354를 동시에 냄. 실측 표 = `DEFAULT_H700_KEYMAP` |
| V-2 | RG SP 뚜껑 스위치·백라이트·배터리 sysfs 경로 | `adb shell "ls /sys/class/backlight /sys/class/power_supply; grep -r . /sys/class/input/*/name"` | M0 | ⚠️ 배터리 `/sys/class/power_supply/axp2202-{battery,usb}` 확인. **`/sys/class/backlight` 없음** — `/sys/class/pwm/pwmchip0`뿐이라 밝기 경로 미정(M4 전 재조사). 뚜껑은 `gpio-keys-polled`(= `dierct-keys-polled`) 후보 |
| V-3 | BaseOS glibc 실제 버전 | `adb shell "/lib/ld-linux-aarch64.so.1 --version"` | M0 | ✅ 2.35. bullseye(2.31) 기준 크로스빌드로 충분 |
| V-4 | `System/frontend` 경로로 부팅되는지, `/tmp/frontend.log` 생성 | 배포 후 `adb shell cat /tmp/frontend.log` | M0 | ✅ `cwd=/mnt/sdcard`, TF2 = `mmcblk0p7` vfat + `utf8` (한글 파일명 OK) |
| V-5 | 호스트 빌드가 Windows 네이티브에서 도는지 | `cargo run -p slot2` | M0 | ✅ |
| V-6 | fbdev EGL 초기화·60Hz 스왑이 RG SP(720×480)에서 정상 | M0 스플래시 | M0 | ✅ `/dev/mali0` + `libEGL.so.1`/`libGLESv2.so.2` 존재, dlopen 성공. fb0 `virtual_size=720,960`(더블버퍼) / `modes=U:720x480p-59` → 패널은 720×480 |
| V-7 | ALSA 장치명(RG SP) | `adb shell "cat /proc/asound/cards"` | M1 | ✅ card0 `audiocodec`(`pcmC0D0p`), card2 `ahubhdmi`. 실기 게임 구동에서 소리 확인 |
| V-8 | 5개 코어의 H700 실속도 | 코어별 벤치(`core_bench` 이식) | M2 | ⬜ |
| V-9 | 640×480·720×720 기기 실기 확인 (보유 시) | 커뮤니티 테스트 | M6 | ⬜ |
| V-10 | Noto Sans KR 서브셋의 RG SP 로딩 시간·메모리 | 부팅 로그에 `font: loaded N glyphs in X ms` | M0 | ⚠️ A53에서 전체 폰트 파싱 52초 → 지연 로딩으로 부팅은 막지 않으나 서브셋이 M5보다 앞당겨질 수 있음 |
| V-11 | BaseOS 루트FS에 `wpa_supplicant`, `udhcpc`, `rfkill` 존재 여부와 경로 | `adb shell "which wpa_supplicant udhcpc rfkill; ls /data"` | M0 | ✅ `/usr/sbin`에 `wpa_supplicant`·`wpa_cli`·`udhcpc`·`rfkill`·`dropbear`·`avahi-daemon` 모두 존재. `iw`는 없음 |
| V-12 | SLOT2가 띄운 wpa_supplicant + udhcpc로 `rgsp.local` SFTP 접속 성공 | 연동 모드 진입 후 PC에서 `sftp root@rgsp.local` | M4 | ⬜ |
| V-13 | HDMI 연결 시 커널이 패널을 미러링하는지, `/dev/fb0` 해상도가 바뀌는지, 핫플러그 시 EGL 서피스가 살아남는지 | 케이블 연결 전후 `adb shell "cat /sys/class/graphics/fb0/virtual_size; ls /sys/class/drm"` + 부팅 로그 | M0 (관찰) / M6 (결정) | ⚠️ **`/sys/class/drm` 없음**(커널 4.9 벤더 fbdev) — DRM 커넥터로는 핫플러그를 알 수 없음. `fb0/virtual_size` 폴링 또는 `ahubhdmi` 상태가 대안 후보. D-22 재검토 필요 |

M1 실기 확인(2026-09): 방향키 이동 · A 실행 · MENU 길게 → 전원 메뉴 · L1/R1 플랫폼 전환 · 전원 끄기(`baseos-poweroff`) · 게임 구동 모두 정상.
