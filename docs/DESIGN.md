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
│  ├─ slot2-retro/            libretro FFI 호스트(코어 무관), quirks/{mgba,gambatte,gpsp,fceumm,snes9x,gpgx}, Registry
│  └─ slot2-store/            카드 레이아웃, 스캔, 세이브/스테이트, 설정 ini, 원자적 쓰기, 마이그레이션
├─ cores/                     코어별 build.sh + 핀 커밋 + *.patch (mgba, gambatte, gpsp, fceumm, snes9x, genesis_plus_gx)
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
| NES | 256×240 (오버스캔 상하 8px 크롭, 기본 켬) | 8:7 | 2× | 2× | 2× |
| SNES | 256×224 / 512×448 | 8:7 | 2× | 2× | 2× |
| MD | 320×224 / 256×224 | 4:3 맞춤 | 2× | 2× | 2× |
| SMS | 256×192 | 8:7 | 2× | 2× | 2× |

위 배율표는 어디에도 저장하지 않는다. 플랫폼의 원본 크기와 패널 크기로부터 `ScalePolicy::Integer`가 계산해 내는 값이고, 표 자체는 `crates/slot2/tests/scaling.rs`가 실행 가능한 형태로 들고 있다. 설계와 코드가 서로 모르게 어긋나는 것을 막는 장치다.

PAR은 플랫폼 상수가 아니다. MD는 256·320 두 폭을 오가면서 둘 다 같은 4:3 화면을 채웠으므로, 픽셀 비율이 아니라 **표시 비율**로 모델링한다(`Aspect::Display`). 덕분에 폭이 바뀌어도 화면이 튀지 않는다 — 이것이 M2 수용 기준 "MD 폭 전환에서 화면 깨짐 없음"의 기하학적 알맹이다. NES·SNES·SMS는 8:7 픽셀(`Aspect::Pixel`), GB 계열과 GBA는 정사각(`Aspect::Square`).

**오버스캔**은 코어가 아니라 표시 계층이 자른다. FCEUmm은 기본으로 상하 8줄을 잘라 256×224만 넘겨주는데, 그 옵션을 꺼서 240줄을 전부 받고 우리가 UV로 자른다. 코어마다 제각각인 옵션에 정책을 흩어놓는 대신 한 곳에 모으기 위해서다 — 크롭 기준이 모든 코어에 동일하고, 코어 옵션을 왕복하지 않으니 즉시 토글되며, 그런 옵션이 아예 없는 코어에도 같은 답을 줄 수 있다. 대가는 텍스처 16줄이다.

**게임별 설정**은 `System/games/<PLAT>/<stem>.ini`에 **바꾼 것만** 적는다. 아무도 건드리지 않은 게임에는 파일 자체가 없고, 설정을 되돌리면 파일을 지운다 — 설정 폴더가 "설정 가능한 모든 것의 덤프"가 아니라 "누군가 내린 결정의 목록"으로 읽히도록. 저장할 때 우리가 모르는 키는 건드리지 않는다(상위 버전이 쓴 키를 하위 버전이 날리면 안 된다). 오타는 기본값으로 떨어진다 — 설정은 틀릴 수 있어도 게임을 못 켜게 만들 수는 없고, 없는 코어를 지정해도 플랫폼 기본 코어로 폴백하며 로그만 남긴다.

**BIOS**는 카드에 파일이 있으면 곧 "진짜 부팅 화면을 보고 싶다"는 뜻으로 읽는다. 여기 코어들은 전부 BIOS 없이도 게임을 돌리므로 파일의 유무가 좌우하는 것은 구동 가능 여부가 아니라 부팅 연출이다. `gb_bios.bin`을 굳이 넣은 사람은 로고가 내려오는 걸 보고 싶은 것이고, 넣지 않은 사람에게 코어의 흉내를 보여줄 이유는 없다. 그래서 **파일의 존재 자체가 설정**이고(`registry::options_for`), 사용자가 여기에 반대하기 전까지 설정할 것이 없다. mGBA는 `mgba_skip_bios`, GPGX는 `genesis_plus_gx_bios`로 표현된다.

**지오메트리 변경**: `SET_GEOMETRY` / `SET_SYSTEM_AV_INFO` 콜백 시 텍스처 재할당·배치 재계산 (MD 폭 전환, SNES 하이레즈). 배치는 매 프레임 코어가 보고한 크기에서 다시 계산하므로 별도 처리가 필요 없고, 텍스처는 크기가 달라진 프레임에서 재할당된다.

**코어 옵션은 세 곳에서 온다** — 콘솔이 어디서 돌든 필요한 것(`PlatformDef`), 이 기기가 감당할 수 있는 것(`Tuning`), 카드에 BIOS가 있는지. 게임별 `.ini`가 그 위를 덮는다.

값 자체는 저자가 RG SP에서 spruceOS를 쓰며 정착시킨 설정에서 출발했다 — 아무도 고르지 않은 기본값보다 나은 근거다. 다만 **그대로 옮기지는 않는다**: spruce는 기기 하나를 설정하지만 SLOT2는 세 가지 패널에 걸친 열 대를 상대한다. 그 설정 중 실은 "720×480 화면"이나 "H700의 여유 사이클"에 대한 판단이었던 것은 `Tuning`으로 표현하고, 모든 콘솔에 보편적으로 참인 양 적어두지 않는다.

`Tuning { geometry, fast_cpu }`가 가르는 예:

| 옵션 | 무엇에 달렸나 | 결과 |
|---|---|---|
| `mgba_sgb_borders` | 패널 크기 | 세 기기 전부 OFF |
| `snes9x_gfx_hires` | CPU | 호스트만 ON |
| `fceumm_sndquality` | CPU | 기기 Low, 호스트 High |
| `mgba_idle_optimization` | 무관 | 항상 `Remove Known` |

SGB 테두리는 화면을 감싸는 게 아니라 코어 출력을 160×144에서 **256×224로 교체**한다. 정수 배율이 전체에 적용되므로 게임 자체가 한 단계 작아진다 — 640×480에서 3×→2×, 720×720에서 4×→2×. 우리 세 패널 전부 손해라 전부 끈다. 기준은 "테두리를 껴도 3× 이상 남는가"이고, 그건 TV 크기부터 참이다. 반대로 SNES 하이레즈(512×448)는 세 패널 모두 1×로 들어가므로 화면이 아니라 사이클의 문제다.

**코어의 aspect·overscan 옵션은 하나도 설정하지 않는다.** 둘 다 프론트엔드가 `PlatformDef`에서 결정하므로, 코어에게도 시키면 보정이 두 번 걸린다. FCEUmm의 기본 8줄 크롭이 실제로 그럴 뻔했고, 테스트가 막는다.

**색보정은 코어가 아니라 콘솔에 붙는다.** mGBA 하나가 게임보이 셋을 전부 돌리지만 정답은 셋이 다르므로, 코어의 `Auto`(구동 중인 시스템에서 추론)를 쓰지 않고 여기서 정한다 — 어느 선반에서 나온 게임인지는 프론트엔드가 이미 알고, 여기서 정해야 나중에 반박할 수 있다.

| 플랫폼 | `mgba_color_correction` | 비고 |
|---|---|---|
| GBA | `GBA` | 패널이 과채도였고 아티스트가 그걸 감안해 그렸다 |
| GBC | `GBC` | 위와 같은 이유 |
| GB | `OFF` + `mgba_gb_colors = "DMG Green"` | 보정할 색이 없다. 있는 건 팔레트이고, 원본 LCD의 초록이 게임보이 선반의 이유다 |

mGBA의 보정에는 **강도 조절이 없다**(켜거나 끄거나). 다만 무거운 필터가 아니라 패널 모델이라 원래 온건하다. 이보다 더 약하게 주고 싶으면 우리 쪽 그레이딩 패스가 있어야 한다 — DESIGN의 `CoreQuirks::color_grade`가 그 자리다. GB 팔레트 기본값은 mGBA가 `Grayscale`인데 그건 누구의 기억도 아니라서 `DMG Green`으로 바꿨다(spruce의 Gambatte 설정이 `GB-DMG`였던 것과 같은 선택).

**셰이더 프리셋**: `None`, `SharpBilinear`, `Lcd3x`(휴대기 기본), `ZfastCrt`(거치기 기본), `Scanline`. GLSL ES 단일 패스. 표준 uniform: `TextureSize`, `InputSize`, `OutputSize`, `FrameCount` — 후일 `.glslp` 호환의 기반.

**오버레이**: `assets/overlays/<platform>/<geometry>.png` 및 카드 `System/Overlays/` 덮어쓰기. 게임 레이어 위, UI 레이어 아래.

**선반 행의 세로 위치는 패널 중심이 아니라 슬롯 기준이다.** 선반이 보여주는 것은 "자기가 들어갈 구멍 위에 서 있는 카트리지"이고, 패널 중심에서 재면 패널이 세로로 길어지는 순간 그 관계가 끊어진다 — 720×720에서 슬롯은 밑단으로 내려가는데 행은 제자리에 남아, 서로 무관한 두 물체 사이에 배경만 넓게 깔렸다. 행의 중심선은 슬롯 띠 위 `CENTRE_ABOVE_SLOT`만큼에 놓는다. 이 값은 고른 게 아니라 **잰 것**이다: 480 높이 패널에서 중앙 배치하면 중심이 240, 슬롯 띠가 422였으므로 그 간격을 그대로 유지한다. 덕분에 480 패널 둘은 한 픽셀도 움직이지 않고 720×720만 내려온다.

한 선반 안에서 플랫폼끼리는 여전히 **바닥이 아니라 중심을 공유한다**(게임보이 팩 253단위 대 GBA 카트 135단위).

**UI 안전 영역**: 640×480을 중앙 배치. 오프셋 `((W-640)/2, (H-480)/2)`. 확장 영역은 배경·선반 연장. HUD(배터리·시계)는 실제 화면 모서리에 고정.

---

## 6. 에뮬 호스트 (`slot2-retro`)

**host/** — libretro API 구현. 코어 특정 지식 없음.
- 로드: `libloading`으로 `.so` 열기, `retro_set_environment` → 콜백 등록.
- env 콜백 지원 목록(초기): `SET_PIXEL_FORMAT`, `GET_SYSTEM_DIRECTORY`, `GET_SAVE_DIRECTORY`, `GET_VARIABLE`/`SET_VARIABLES`/`GET_VARIABLE_UPDATE`, `GET_CORE_OPTIONS_VERSION`, `SET_CORE_OPTIONS(_V2)`, `SET_GEOMETRY`, `SET_SYSTEM_AV_INFO`, `GET_LOG_INTERFACE`, `SET_INPUT_DESCRIPTORS`, `SET_CONTROLLER_INFO`, `GET_RUMBLE_INTERFACE`, `GET_INPUT_BITMASKS`, `SET_SUPPORT_NO_GAME`(거부), `GET_LANGUAGE`, `SET_FRAME_TIME_CALLBACK`, `GET_AUDIO_VIDEO_ENABLE`. 미지원은 `false` 반환 + 로그.
- 세이브: `retro_get_memory(SAVE_RAM)` 주기적 flush + 종료 시. 스테이트: `retro_serialize`.
- 되감기: 직렬화 링버퍼, 플랫폼별 `RewindBudget{interval_frames, ring_bytes}`. 상태를 통째로 쌓는 건 불가능하다 — MD 상태가 1MB이고 6프레임마다 하나면 초당 10MB다. 그런데 0.1초 동안 상태는 거의 변하지 않는다(실측: GBA 528KB 중 61바이트, MD 1MB 중 29바이트, SNES가 가장 심할 때 823KB 중 4KB). 그래서 **온전한 상태 하나 + 그 뒤로 XOR 델타 사슬**로 간다. XOR은 자기 역함수라 한 인코딩이 양방향을 겸하고, 안 바뀐 구간은 varint 두 개로 끝난다.
  - **캡처 간격은 기기가 정한다.** 캡처 비용(직렬화 + 델타)은 대부분 메모리 트래픽이라 기계와 콘솔에 달렸고, 여기 적어둘 수 있는 값이 아니다. A53(파이 3B+, 기기와 같은 코어)에서 NES는 프레임의 0.5%, MD는 **22%**다. 한 기계의 측정치로 플랫폼별 숫자를 박는 대신, 세션이 **도는 기계에서 직접 재서** 프레임 예산(3%)을 넘으면 간격을 두 배씩 넓힌다(최대 60프레임). 느린 기기에서 되감기가 거칠어질 뿐, 0.1초마다 끊기지는 않는다.
  - 되감기는 링을 **소비한다**. 1초 되감고 다시 진행하면 되감아 지나온 1초는 남지 않고, 그 자리에 새로 지나가며 찍은 상태가 들어간다.
- 빨리감기: 한 표시 프레임 안에서 코어를 여러 번 돌린다. 오디오는 버린다 — 4배속으로 재생하면 음정이 4배가 되고 링이 넘친다.
- 시간 조작 핫키는 **L2(되감기) / R2(빨리감기)**. 여기 플랫폼 중 트리거를 쓰는 기종이 없어 비어 있는 버튼이다.
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

**코어 빌드** (`cores/<name>/build.sh`): 핀 커밋 + 패치, `.meta` 스탬프로 재빌드 판정, 산출물 `vendor/<name>_libretro.so`. 스탬프에는 repo·핀·target·make 디렉터리와 makefile·추가 make 인자·triple·패치 해시가 모두 들어가므로, 인자를 바꾸면 같은 핀이라도 다시 빌드된다. 배포하는 여섯 코어(mGBA·Gambatte·gpSP·FCEUmm·snes9x·GPGX)가 모두 이 경로로 빌드되고, Windows `.dll`은 개발 테스트용으로 libretro buildbot에서 받는다.

**배포 코어 목록은 `cores/required.txt` 한 곳이다.** 여섯 이름을 순서대로 적은 이 manifest가 `build/cores.ps1`의 기본 목록이자 `build/dist-device.ps1`이 카드와 zip에 넣는 목록이고 CI의 host·device 빌드가 순서대로 읽는 목록이다 — PowerShell에도 workflow에도 이름 배열을 따로 두지 않는다. runtime의 단일 목록은 계속 `Core::ALL`(`slot2-retro`)이고, `build/core-manifest.ps1`의 검증(빈 값·중복·안전하지 않은 이름·누락된 build.sh/commit 거부)과 `slot2-retro`의 filesystem contract 테스트가 manifest와 `Core::ALL`을 맞물린다. **여섯 `.so`와 여섯 `.so.meta`가 vendor에 없으면 배포는 실패한다**: 하나라도 없거나 비어 있으면 기존 `dist-device`를 지우기 전에 이름과 경로를 찍고 멈추고, 조립 뒤에도 `System/cores`·`System/licenses`가 manifest의 정확한 여섯인지 다시 확인한 다음에만 zip·ADB로 간다. core가 하나 없는 카드는 선반 하나가 비어 있는 채로 부팅해 버그처럼 보이므로 경고로 넘기지 않는다.

**라이선스와 대응 소스도 같은 gate를 탄다.** `build/package-core-sources.ps1`이 각 core의 pinned checkout에서 `git archive`로 **pristine 소스**(commit 그대로, build output·`.git`·적용된 패치 없음)를 뜨고, `cores/common.sh`·`commit`·`build.sh`·`*.patch`를 **recipe**로 함께 복사하며, core별 top-level 라이선스를 `git cat-file`로 꺼내 `licenses/cores/<name>/`의 추적 복사본과 byte 비교한다. 산출물은 `SOURCE-MANIFEST.txt`(repo·pin·archive SHA-256·patch SHA-256, 상대경로만)와 함께 카드의 `System/licenses/sources/`에 들어가고, 배포 스크립트는 기존 `dist-device`를 지우기 전에 bundle을 만들고 검증한 뒤에야 조립을 시작한다. archive가 해당 pin의 라이선스를 같은 byte로 포함하지 않거나, recipe가 repository와 다르거나, manifest hash가 파일과 맞지 않으면 zip·ADB 이전에 실패한다. 즉 binay에 대응하는 소스는 **archive + recipe**이고, `System/licenses/<name>_libretro.so.meta`가 어느 pin·인자로 빌드됐는지를 봉인한다.

**코어가 신고한 값은 그대로 받는다.** 코어의 `sample_rate`·`fps`는 호스트가 "상식적인 범위"로 다듬을 대상이 아니다. mGBA는 GBA 오디오를 **65536 Hz**(`GBA_OUTPUT_RATE`, GBA 사운드 FIFO의 32768 Hz의 두 배)로 내보내고 그렇게 신고한다. 한때 호스트에 "50 kHz 넘으면 32768로 간주" 클램프가 있었고, 그 탓에 모든 GBA 게임이 한 옥타브 낮게·두 배 느리게 재생됐다. 게다가 초과분이 오디오 링을 넘쳐 잘려 나가면서 템포만 얼추 맞아 보여, **명백한 고장이 막연한 잡음처럼 위장**됐다. 검증은 범위가 아니라 **자기일관성**으로 한다 — 신고한 레이트와 실제로 넘겨준 샘플 수가 일치하는지(`the_declared_sample_rate_is_what_the_core_actually_produces`).

**프레임 페이싱과 동적 레이트 제어.** 루프는 1/60이 아니라 **코어의 `fps`**로 재운다(GBA 59.7275). 60 Hz로 돌리면 코어가 0.46 % 많은 오디오를 만들고, 링이 찬 뒤로는 매 프레임 꼬리가 버려져 초당 60번 이어붙는 소리가 난다. 남는 표류(호스트 시계 ↔ 코덱 48 kHz 크리스털)는 `drc_trim`이 링 점유율을 보고 출력 레이트를 ±0.5 % 안에서 미세 조정해 흡수한다. 0.5 %는 음정 변화로 들리지 않고, 대안(샘플을 버리거나 반복하기)은 곧 클릭이다.

---

## 7. UI (`slot2-ui`)

**화면 상태기계**: `Shelf` → `Inserting` → `Playing` ↔ `InGameMenu`(→ `StateSwitcher` / `CheatMenu` / `DisplayMenu` / `CorePicker`) → `Ejecting` → `Shelf`. `CorePicker`는 후보가 2개 이상인 플랫폼에서 설치 core를 보여주고 선택 시 checkpoint 뒤 Session을 다시 시작한다. 선반 부속: `ShelfMenu`(언어·플랫폼 기본 화면 설정·부팅 로고·연동 모드·시계·About), `PowerMenu`, `SyncScreen`, `Toast`, `Refusal`.

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
    insert: Curve, eject: Curve, sfx_in: SoundProfile, sfx_out: SoundProfile,
}
```
GBA·GB·GBC는 원본 SVG 이식, NES·SNES·MD·SMS는 SLOT2 자체 카트 SVG(2026, 원본 tracing 없이 기하학적 재해석). 7개 플랫폼 모두 독립 cart shell/detail과 독립 port trim을 가지며 폴백은 없다. port trim은 front 단계에서 band 위에 합성되고 중앙 opening이 비어 있어 slot에 꽂힌 카트리지가 계속 보인다. `PlatformSkin`의 `insert`/`eject`는 플랫폼별 **normalized travel profile**(contact·release·creep, 애니메이션 전체에 대한 비율)이고, 애니메이션 길이·seated hold·코어 로드 시점은 `insert`/App 계약에 남는 공용 clock이다.

`sfx_in`/`sfx_out`은 같은 두 녹음(`assets/sfx/*.pcm`, MIT 원본 그대로)을 플랫폼별 재생 프로필 `SoundProfile{speed, gain}`로 읽는 것이다. 속도·음량만 담은 copy 가능한 의미 타입이고 `slot2-ui`는 audio에 의존하지 않는다. 방향은 계속 `Sfx::{Insert,Eject}`가 정하고, 프로필은 그 녹음을 어떻게 재생할지만 정한다. App은 `platform()`의 skin을 한 번 조회해 삽입/배출에 맞는 프로필을 고르고 clip을 이벤트당 한 번만 render해 ring에 전부 쓴 뒤 sink를 연다. 표·생성자 bounds는 코드가 단일 source of truth다. 접점 cue는 원본 lead가 아니라 **변형 속도의 lead**(`Sfx::lead_at_speed`)로 `SEATED_AT`에서 역산하므로 어떤 프로필에서도 접점 소리와 화면 접촉 시점이 `SEATED_AT`에 함께 온다. base `Sfx::render`는 녹음 그대로이고 과거 호출자 계약은 그대로 유지된다.

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
slot2-text: 폰트 폴백 체인 [언어팩 lang-font= 선호(지연 로딩) → UI 라틴 → 한글·CJK(Noto Sans KR 서브셋, 지연 로딩) → 아이콘] · 폭 측정 · 글리프 캐시
        │
        ▼
face 래스터 → 텍스처
```

- **JOSA 규칙**: 마지막 글자가 한글 음절이면 `(cp-0xAC00)%28`로 받침 판정, `으로/로`는 ㄹ받침(8) 예외. 숫자는 읽는 소리로 판정. 그 외(영문·기호)는 병기형 `을(를)`.
- **어순**: 코드에서 문자열 결합 금지. 모든 문장은 완성 템플릿 + 이름 붙은 자리표시자.
- **언어 선택**: 설정 항목. 기본 `en`. 언어팩은 `lang-font = "..."`로 UI 본문의 선호 폰트 파일명 하나를 지정한다(`System/Fonts/` 우선, 없으면 기본 체인).
- **한글·CJK 폰트**: Noto Sans KR(OFL) 서브셋 1개를 항상 내장 — 게임 제목의 일본어·한자를 언어 설정과 무관하게 렌더해야 하기 때문. `pyftsubset`으로 라틴+한글 음절+가나+KS X 1001 한자+기호로 제한(≈17k 글리프, 5~6MB). fontdue는 로드 시 전 글리프를 파싱하므로 부팅 경로에서 빼고 지연 로딩(첫 미스 또는 백그라운드). 대체 라벨 인쇄(`Labels/`에 이미지 없을 때)도 같은 체인.
- **정렬**: Unicode scalar value 순서(Rust `str`의 UTF-8 바이트 비교가 이 순서를 보존한다). ASCII 대문자 → 소문자 → 가나·한자 → 한글 음절 순으로 드러나며(로케일 collation·자연수 정렬·대소문자 folding 없음), 같은 stem의 복수 허용 확장자는 확장자 순으로 결정적이다. NFC/NFD 정규화를 하지 않으므로 파일시스템이 다르게 준 두 이름은 다른 카트다.

---

## 9. 카드 레이아웃 (`slot2-store`)

```
<card root>/
├─ System/
│  ├─ frontend             SLOT2 바이너리 (BaseOS 계약)
│  ├─ cores/*.so           cores/required.txt의 여섯: mgba, gambatte, gpsp, fceumm, snes9x, genesis_plus_gx
│  ├─ licenses/            고지·코어 라이선스·대응 소스(§6 코어 빌드)
│  │  ├─ SLOT2-LICENSE          SLOT2 자체 MIT(root LICENSE)
│  │  ├─ CORE-NOTICES.md        코어/원작/폰트/스킨·효과음 고지와 대응 소스 설명
│  │  ├─ upstream-slot/LICENSE  원작 brandonkowalski/slot의 MIT 원문
│  │  ├─ fonts/                 Open Sans·Noto Sans KR OFL
│  │  ├─ cores/<name>/<license> pinned commit의 top-level 라이선스 원문
│  │  ├─ sources/          SOURCE-MANIFEST.txt + archives/<name>-<pin>.zip(git archive) + licenses/ + recipes/
│  │  ├─ rust/             Rust 런타임 third-party 고지: THIRD-PARTY-RUST.md + RUST-SBOM.json + RUST-MANIFEST.txt + packages/<key>/(PACKAGE.txt + crate 원문 라이선스)
│  │  └─ <name>_libretro.so.meta
│  ├─ Lang/*.ftl           선택: 언어팩 덮어쓰기
│  ├─ Fonts/               선택
│  ├─ Overlays/            선택
│  ├─ slot2.ini            전역 설정
│  ├─ games/<PLAT>/<stem>.ini   게임별 설정(코어 선택·스케일·오버스캔·셰이더)
│  ├─ cheats/<PLAT>/<stem>.cht  치트(RetroArch .cht 포맷, 데스크탑이 씀, D-21)
│  ├─ manifest.json        연동 프로토콜: SLOT2가 씀 (§12.3)
│  ├─ sync.lock            연동 프로토콜: 데스크탑이 전송 중 생성
│  ├─ wifi.toml            연동 프로토콜: 데스크탑이 씀, SSID/PSK 목록
│  └─ slot2.log
├─ BIOS/                   gb_bios.bin gbc_bios.bin gba_bios.bin bios_{U,E,J}.sms  (전부 선택)
├─ Games/{GB,GBC,GBA,NES,SNES,MD,SMS}/
├─ Labels/{...}/<stem>.png
├─ Saves/{...}/<stem>.sav
├─ States/{...}/<stem>/<core>/{resume.state, N.state, N.png}
└─ Wallpapers/*.png
```

원본 slot 카드(`Games/`, `Labels/`, `Saves/`, `States/`)와 **호환**을 유지한다 — 기존 slot 사용자가 `System/`만 바꾸면 그대로 쓸 수 있게. 확장자 매핑: GB `.gb`, GBC `.gbc`, GBA `.gba`, NES `.nes`, SNES `.sfc .smc`, MD `.md .gen .bin`, SMS `.sms`. 폴더+확장자로만 플랫폼 판정(헤더 스니핑 없음). 쓰기는 전부 원자적(temp + rename).

게임 하나의 키는 **스캔한 파일명에서 마지막 허용 확장자 하나만 뗀 stem**이다. 확장자 판정만 대소문자를 무시하고, 공백·여러 점·괄호·대괄호·`'`/`+`/`&`/`!`/`#`/`%`/`@`/`_`/`-`와 한글·가나·한자는 그대로 보존한다(정규화·sanitize·rename 없음). 그 stem이 위 `Labels/`·`Saves/`·`States/`·`games/<PLAT>/<stem>.ini`·`cheats/<PLAT>/<stem>.cht`에 그대로 쓰이고, 플랫폼이 다르면 폴더로 갈라진다. 같은 플랫폼에서 같은 stem의 허용 확장자가 둘(SNES `.sfc`/`.smc`)이면 **카트 두 개로 표시되고 위 경로를 공유한다** — 레이아웃이 stem당 세이브 하나만 두기 때문이다.

**스테이트는 코어별로 한 단계 더 내려간다.** libretro 스테이트는 코어가 자기 메모리를 직렬화한 것이라 다른 코어는 읽을 수 없고, 한 게임에 코어 둘(mGBA·gpSP)이 붙는 순간 같은 번호가 서로 다른 바이트를 뜻하게 되기 때문이다. `<core>`는 공식 코어면 `CoreId::base_name()`(예: `mgba_libretro`), 이 프론트엔드가 모르는 라이브러리면 stem에서 결정적으로 만든 이름이다(그대로 쓸 수 없는 문자는 `external_<hex>_<hex>`로 접는다). 어느 코어를 열지와 그 코어의 namespace는 `session::resolve_core` 한 곳이 정하고, 실행과 선반의 Resume 조회가 같은 함수를 쓴다 — 게임별 ini의 코어 값은 코어 디렉터리 안의 **파일 이름**으로만 취급해 `/`, `\`, `..` 같은 값이 카드 밖 라이브러리를 열지 못하게 한다.

코어를 고를 수 없던 시절의 **평면 스테이트**(`<stem>/{resume.state,N.state}`)는 스캔 때 플랫폼 **기본** 코어의 namespace로 한 번 옮긴다 — 그 시절의 파일은 정의상 기본 코어의 것이고, 지금 선택된 대체 코어의 것이 아니다. 목적지에 이미 파일이 있으면 옮기지 않고 평면 원본을 그대로 둔다: 어느 쪽이 최신인지 프론트엔드가 알 방법이 없고, 둘을 합치는 것보다 둘 다 남기는 쪽이 안전하다. 세이브 RAM은 이 태스크의 대상이 아니며 여전히 코어 간 공유다.

---

시각 계약(D-25): 시스템 시각과 저장 시각은 UTC로 유지하며, 시간대 오프셋은 표시 직전에만 적용한다. mtime을 로컬 시각으로 보정해 되쓰지 않는다. 향후 시각을 파일명에 넣는 기능은 UTC와 `Z` 표기를 사용한다. 표시 offset의 영속 값은 `System/slot2.ini`의 `utc_offset_minutes`(기본 0) 하나이며, 시작 시 한 번 적용되고 설정 메뉴에서 방향 입력으로 즉시 preview·A 적용·B 취소되고 저장 실패 시 이전 값으로 되돌아간다.

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
- CI: `ci.yml`(fmt, clippy host+device, test). 카드 트리 조립은 `device-artifact.yml` 하나만 한다 —
  `workflow_call` 전용, `contents: read`, artifact 이름은 호출자가 준다. `ci.yml`의 `device` job
  (`needs: check`)과 태그 릴리스가 이 파일을 같이 호출하므로 릴리스가 싣는 트리는 CI가 조립한 트리와 같다.
- 릴리스: `release.yml`은 `v*` 태그 push에서만 돈다(`contents: read`, 태그별 concurrency·취소 없음).
  `build` job이 `device-artifact.yml`을 릴리스용 artifact 이름으로 호출하고, `publish` job
  (`contents: write` 하나뿐)이 그 artifact를 이름으로 내려받아 태그 커밋(`git rev-parse HEAD`)을
  checkout(`ref`=태그, `fetch-depth: 0`)한 뒤 `build/package-release.ps1 -CardRoot <트리> -Tag v<버전>
  -Commit <40hex> -OutputDir <dir>`로 `slot2-<태그>.zip`과 `.zip.sha256` 하나만 만든다. 스크립트는
  태그=작업공간 버전, 커밋 40hex, `System/VERSION.txt`의 버전·짧은 커밋, 여섯 core·라이선스·소스 번들·
  Rust 고지·폰트·frontend를 검사하고, zip을 다시 열어 엔트리 이름·루트(`System` 하나)·길이·해시를 트리와
  대조하며, 기존 출력 파일은 덮어쓰지 않고 실패하면 이전 출력을 그대로 둔다. 게시 단계는 러너의 `gh`로
  **draft** 릴리스(`--draft --verify-tag --generate-notes`, 기존 릴리스가 있으면 실패)를 만드는 것뿐이고
  공개는 사용자가 한다. **호스팅 CI 미실행** — cache miss/hit artifact parity, 태그 실행, draft 검사,
  수동 공개는 미검증이다.

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
| V-8 | 6개 코어의 H700 실속도 | 코어별 벤치(`core_bench` 이식) | M2 | ⬜ |
| V-9 | 640×480·720×720 기기 실기 확인 (보유 시) | 커뮤니티 테스트 | M6 | ⬜ |
| V-10 | Noto Sans KR 서브셋의 RG SP 로딩 시간·메모리 | 부팅 로그에 `font: loaded N glyphs in X ms` | M0 | ⚠️ A53에서 전체 폰트 파싱 52초 → 지연 로딩으로 부팅은 막지 않으나 서브셋이 M5보다 앞당겨질 수 있음 |
| V-11 | BaseOS 루트FS에 `wpa_supplicant`, `udhcpc`, `rfkill` 존재 여부와 경로 | `adb shell "which wpa_supplicant udhcpc rfkill; ls /data"` | M0 | ✅ `/usr/sbin`에 `wpa_supplicant`·`wpa_cli`·`udhcpc`·`rfkill`·`dropbear`·`avahi-daemon` 모두 존재. `iw`는 없음 |
| V-12 | SLOT2가 띄운 wpa_supplicant + udhcpc로 `rgsp.local` SFTP 접속 성공 | 연동 모드 진입 후 PC에서 `sftp root@rgsp.local` | M4 | ⬜ |
| V-13 | HDMI 연결 시 커널이 패널을 미러링하는지, `/dev/fb0` 해상도가 바뀌는지, 핫플러그 시 EGL 서피스가 살아남는지 | 케이블 연결 전후 `adb shell "cat /sys/class/graphics/fb0/virtual_size; ls /sys/class/drm"` + 부팅 로그 | M0 (관찰) / M6 (결정) | ⚠️ **`/sys/class/drm` 없음**(커널 4.9 벤더 fbdev) — DRM 커넥터로는 핫플러그를 알 수 없음. `fb0/virtual_size` 폴링 또는 `ahubhdmi` 상태가 대안 후보. D-22 재검토 필요 |

M1 실기 확인(2026-09): 방향키 이동 · A 실행 · MENU 길게 → 전원 메뉴 · L1/R1 플랫폼 전환 · 전원 끄기(`baseos-poweroff`) · 게임 구동 모두 정상.
