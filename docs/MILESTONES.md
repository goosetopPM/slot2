# SLOT2 마일스톤

각 마일스톤은 **완료 기준(Acceptance)** 을 실기 또는 호스트에서 확인 가능한 형태로 둔다. 순서는 의존성 순이며, M3~M5는 호스트 창 빌드에서 대부분 진행할 수 있어 M1~M2와 병행 가능하다.

역할: 설계·진단·패치 작성은 Claude, 빌드·배포·실기 확인은 사용자.

---

## M0 — 기반: 부팅되는 빈 껍데기

**목표**: BaseOS가 `System/frontend`로 SLOT2를 띄우고, 화면에 한글/영문 스플래시가 뜨며, 전원 메뉴로 끌 수 있다.

작업
- [ ] `git init`, 워크스페이스 뼈대(모든 크레이트 빈 lib), `rust-toolchain.toml`, `taskfile.yml`, `.gitignore`
- [ ] `build/cross.Dockerfile` + `task build:device` (x86_64 → aarch64, glibc 2.31)
- [ ] `slot2-platform`: `BASEOS_TARGET` 읽기, 프로필 테이블, `baseos-poweroff/reboot` 호출
- [ ] `slot2-gfx`: fbdev EGL 서피스(원본 `fbdev.rs` 이식) + host glutin 서피스, 단색 클리어 + 쿼드 1개
- [ ] `slot2-text` + `slot2-i18n`: fontdue 폴백 체인, Fluent 번들, `en.ftl`/`ko.ftl` 각 3문자열, `JOSA` 스텁
- [ ] `slot2-input`: evdev 열기, 버튼 1개(MENU) 인식 / host 키보드
- [ ] 스플래시 화면: "SLOT2" 워드마크 + 현재 언어의 인사 문장(한글 렌더 확인) + 기기 프로필 표시
- [ ] MENU 홀드 → 전원 메뉴 → 종료
- [ ] CI: fmt + clippy(host/device) + test

**Acceptance**
- RG SP에서 SD 삽입 → 부팅 → 스플래시(한글 포함) → 전원 끄기 동작. `/tmp/frontend.log`에 프로필 라인 기록.
- PC 창(`cargo run -p slot2`)에서 `SLOT2_GEOMETRY` 3종 모두 같은 스플래시가 안전 영역 중앙에 뜸.
- 검증 항목 V-1 ~ V-6 닫힘.

---

## M1 — 코어 호스트: GBA 하나가 돈다

**목표**: 코어 무관 libretro 호스트로 mGBA를 로드해 GBA 게임이 실기에서 60fps로 돌고, 세이브·스테이트가 남는다.

작업
- [ ] `cores/mgba` 빌드 스크립트(원본 핀·패치 이식) → `vendor/mgba_libretro.so`
- [ ] `slot2-retro/host`: FFI, env 콜백 세트(DESIGN §6), 프레임 루프 결합
- [ ] `slot2-audio`: ALSA dlopen(원본 이식) + 링버퍼 + 리샘플 / host cpal
- [ ] `slot2-store`: 카드 스캔(`Games/GBA/*.gba`), `Saves/`, `States/` 원자적 쓰기
- [ ] `quirks/mgba`: BIOS 감지 → 옵션, 입력 맵
- [ ] 임시 UI: 텍스트 목록에서 게임 선택 → 실행 → MENU 홀드로 복귀
- [ ] 스테이트 저장/불러오기(SELECT+R1 / SELECT+L1), SRAM flush
- [ ] `core_bench` 이식(헤드리스 프레임 타이밍)

**Acceptance**
- RG SP에서 GBA 게임 1종 이상 풀스피드, 소리 정상, 전원 껐다 켜도 SRAM 유지, 스테이트 왕복.
- 호스트 창에서도 동일 동작(테스트용 홈브루 ROM).
- V-7, V-8(mGBA) 닫힘.

---

## M2 — 플랫폼 레지스트리와 5개 코어

**목표**: 7개 플랫폼이 전부 실행되고, 플랫폼별 스케일·입력·지오메트리 변경이 처리된다.

작업
- [x] `cores/{fceumm,snes9x,genesis_plus_gx}` 빌드 스크립트 + 라이선스·소스 동봉 규칙 (gpSP는 GBA 대안 코어라 후순위)
- [x] `registry`: PlatformDef 7종 + 원본 크기·표시 비율·오버스캔. 코어 선택은 게임별 ini로 (대안 코어 테이블은 gpSP와 함께)
- [x] 옵션 프리셋 · 입력 맵(SNES·MD 6버튼) · 색보정 — 전부 registry에서. 별도 `quirks/` 모듈은 코어별로 갈라질 일이 생길 때까지 만들지 않는다
- [x] `SET_GEOMETRY`/`SET_SYSTEM_AV_INFO` 처리(MD 256↔320, SNES 하이레즈)
- [x] `ScalePolicy` 3모드 + 플랫폼×지오메트리 기본값, NES 오버스캔 크롭 옵션
- [ ] 되감기 링버퍼 플랫폼별 예산, 빨리감기
- [x] BIOS 감지(gb/gbc/gba/md/sms) + 부팅 로고 토글
- [x] 게임별 설정 파일(`System/games/<PLAT>/<stem>.ini`): 코어 선택·스케일·오버스캔

**Acceptance**
- RG SP에서 플랫폼당 1종 이상 풀스피드. MD 폭 전환·SNES 하이레즈 게임에서 화면 깨짐 없음.
- SNES/MD 6버튼 게임 조작 가능.
- V-8 전 코어 닫힘.

---

## M3 — 선반 UI와 스킨 시스템

**목표**: 원본의 선반·카트리지·삽입/꺼내기 경험을 SLOT2 레이아웃 체계로 재구축하고, 플랫폼별 스킨 테이블이 동작한다. (호스트 창 중심)

작업
- [x] `slot2-ui` 화면 상태기계, 안전 영역 레이아웃, faces 캐시
- [x] `PlatformSkin` 테이블 + GBA/GB/GBC 스킨(원본 SVG·쉘 팔레트 이식), 나머지는 GBA 폴백
- [x] 선반(카루셀), L1/R1 플랫폼 전환 시 스킨 교체, 카트 라벨(`Labels/` + 대체 인쇄) — 월페이퍼는 남음
- [x] 삽입/꺼내기 애니메이션, 슬롯 입구 그래픽 교체 — 효과음(원본 이식)은 남음
- [x] 배터리·시계 HUD(화면 모서리 고정), 토스트, 거부 화면 — M5 시계 화면이 `SLOT2_UTC_OFFSET_MIN`을 세우기 전까지 실기 시계는 UTC로 보인다
- [ ] 코어 선택 화면(기판/칩) — 코어 2개 이상인 플랫폼만. **현재 막힘**: 코어 4개가 플랫폼당 하나라 대상이 없다. gpSP 빌드 후 재개.
- [x] "ROM 1개면 즉시 실행" 전용기 모드 — 두지 않는다(D-24)

**Acceptance**
- 호스트 창 3개 지오메트리에서 선반이 깨지지 않음(안전 영역 + 연장 영역).
- RG SP에서 선반 → 삽입 → 플레이 → 꺼내기 → 선반 순환. GB 선반에서 GB 카트·GB 스킨으로 바뀜.

---

## M4 — 게임 내 기능

**목표**: 원본의 플레이 중 기능을 전 플랫폼에 제공한다.

작업
- [ ] **인게임 메뉴**(D-23): MENU 탭 → 코어 정지·뮤트·프레임 딤 → 패널. 항목: 계속하기·세이브 스테이트·치트·화면·코어·기기·꺼내기. 설정 계층(전역→플랫폼→게임별) + 게임별 ini 쓰기
- [ ] 선반 메뉴: 언어·플랫폼 기본 화면 설정·부팅 로고·연동 모드·시계·About
- [ ] 치트: `.cht` 로더 → `retro_cheat_set`, 인게임 메뉴 "치트"(항목별 on/off, 즉시 재적용), quirks별 형식 검증(D-21)
- [ ] 세이브 스테이트 스위처(폴라로이드, 실행취소 30초) — 인게임 메뉴에서 진입
- [ ] 제스처 정리: MENU 탭/홀드, SELECT 코드, L2/R2. 더블탭 MENU 폐기
- [ ] HUD 배지(빨리감기·되감기 상태)
- [ ] 뚜껑: 닫으면 스테이트 저장 + 화면 끔, 3분 후 전원 끔, 열면 복귀(뚜껑 있는 프로필만)
- [ ] 종료 시 resume 스테이트, A 탭=재개 / A 홀드=새로 시작
- [ ] 전원 메뉴, 시계 설정, About(스티커 — 빌드 정보·라이선스 안내)
- [ ] **연동 모드(기기 쪽)**: `platform::net`(wpa_supplicant/udhcpc 기동·종료), `System/wifi.toml` 읽기, `manifest.json` 작성, `sync.lock` 감시, SyncScreen(hostname/IP/상태), 완료 후 재스캔. 데스크탑 앱 없이 PC의 `sftp`로 검증

**Acceptance**
- RG SP에서 뚜껑 닫기/열기 왕복 후 게임 이어짐. 스테이트 스위처에서 로드·삭제·실행취소 동작.
- 모든 UI 문자열이 `.ftl` 경유(코드에 하드코딩 문자열 0개 — grep으로 확인).
- 연동 모드 진입 → PC에서 `sftp root@rgsp.local`로 `Games/GBA/`에 ROM 추가 + `sync.lock` 생성/삭제 → 모드 종료 후 선반에 새 카트 표시. 모드 밖에서는 `wlan0`에 IP 없음(V-12).

---

## M5 — 다국어 완성과 표시 품질

**목표**: 한국어 UI가 자연스럽고, 셰이더·오버레이가 플랫폼별 기본값으로 적용된다.

작업
- [ ] `ko.ftl` 전수 번역, `JOSA`·`BTN` 실사용, 문장형 문구 최소화 검토
- [ ] Noto Sans KR 서브셋 생성 스크립트(`tools/subset-font.sh`) + 지연 로딩, 대체 라벨 한글·일본어 제목 인쇄 확인
- [ ] 한·영 혼합 정렬 정책, 파일명 특수문자 처리
- [ ] 셰이더 프리셋 4종 + 플랫폼 기본값, 오버레이 로더 + GB·CubeXX용 샘플 베젤
- [ ] 언어팩 덮어쓰기(`System/Lang/`) + 폰트 지정 동작 확인
- [ ] 시간대 설정 — 설정 메뉴에서 UTC 오프셋을 고르고, 그 값이 `SLOT2_UTC_OFFSET_MIN`을 대체한다. BaseOS는 환경변수 없이 `System/frontend`를 exec 하므로(DESIGN §2) 이 화면이 생기기 전까지 실기 시계는 UTC다
- [ ] 번역 기여 가이드(`docs/TRANSLATING.md`)

**Acceptance**
- 한국어로 전 화면 순회 시 깨진 글자·잘림·부자연스러운 조사 없음.
- 카드에 `System/Lang/ja.ftl`(테스트용 3문자열)만 넣어도 언어 목록에 나타남.

---

## M6 — 스킨 완성과 기기 검증

**목표**: 7개 플랫폼 스킨이 전부 고유하고, RG SP 외 지오메트리 실기 검증.

작업
- [ ] NES/SNES/MD/SMS 카트·포트 SVG, 삽입 곡선·효과음
- [ ] 640×480(RG35XX 계열)·720×720(CubeXX) 실기 테스트(보유 또는 커뮤니티)
- [ ] 스틱 기기(H/V/Cube) 입력 프로필 확인, 미확인 타깃 폴백 검증
- [ ] 성능 프로파일: 프레임 예산 내 UI 렌더(원본 기준 2.2% 개선 사례 참고)
- [ ] HDMI: 미러링 동작 확인(V-13). 전용 720p 출력은 `W1280H720` 지오메트리 시제품으로 프레임 시간 실측 후 채택 여부 결정(D-22)

**Acceptance**
- 각 플랫폼 전환 시 하단 포트·카트·소리가 전부 다름.
- 최소 2개 지오메트리 실기 확인, V-9 닫힘.

---

## M7 — 공개 배포

**목표**: GitHub 공개, 릴리즈 zip 한 개로 설치 완료.

작업
- [ ] `release.yml`: 태그 → `dist-device` zip(라이선스·코어 소스·패치 동봉)
- [ ] `README.md`(en/ko): 설치(BaseOS + SLOT2 1장/2장 구성), 카드 레이아웃, 버튼표, FAQ
- [ ] LICENSE(MIT, 원작자 고지 유지), 코어 라이선스 요약, AI 사용 고지
- [ ] 이슈 템플릿, 번역 기여 안내
- [ ] 원본 slot 카드에서 마이그레이션 가이드(`System/` 교체)

**Acceptance**
- 새 SD카드 + BaseOS 이미지 + 릴리즈 zip만으로 처음 사용자가 README대로 설치해 게임 실행.

---

## M8 — 데스크탑 매니지먼트 앱 (별도 저장소)

**목표**: PC의 ROM 라이브러리에서 롬셋을 골라 기기로 보내고, 세이브가 양방향으로 따라다닌다. SD 리더·USB·Wi-Fi 어느 경로로도 같은 결과.

작업
- [ ] 스택 결정(1순위 Tauri + Rust — `slot2-store`의 레이아웃·manifest 코드를 공용 크레이트 `slot2-card`로 분리해 공유)
- [ ] `CardRoot` 백엔드: `LocalPath`(SD 리더·USB 저장소 모드), `Sftp`(dropbear, `<hostname>.local`, 비밀번호는 OS 키체인 저장)
- [ ] 라이브러리: 폴더 스캔, 플랫폼 판정(폴더/확장자 규칙 공유), 중복·해시, 메타데이터
- [ ] 롬셋 편집 → diff 계산 → 적용(추가/삭제, 라벨·게임 ini 동반), `sync.lock` 규약, `sync.log` 작성
- [ ] 세이브 동기화: `<PLAT>/<stem>` 키, mtime+해시 충돌 판정, 세이브 이력 보관·복원
- [ ] 라벨 생성기(원본 Cart Studio 기능 흡수), `wifi.toml` 편집, 게임별 설정 편집
- [ ] 치트 편집기: libretro-database `.cht` 가져오기, 게임별 편집 → `System/cheats/` 동기화(D-21)
- [ ] 기기 발견: mDNS 브라우징(`_ssh._tcp`), 수동 IP 입력

**Acceptance**
- 새 ROM 3개 선택 → Wi-Fi로 전송 → 기기 선반에 카트 3장 추가. 기기에서 플레이 후 세이브 → PC에서 동기화 → 라이브러리에 세이브 이력 1건 추가. 같은 시나리오를 SD 리더로 반복해도 결과 동일.

---

## 이후 (Deferred, DECISIONS 참조)
링크 케이블 · 아날로그 핫키 · 압축 ROM · 사용자 GLSL/`.glslp` · 대체 코어(Gambatte 등) · RG28XX
