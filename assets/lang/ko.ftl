# SLOT2 — 한국어
#
# 규칙 (docs/DESIGN.md §8):
# - 문장은 통째로. 코드가 조각을 이어 붙이지 않는다.
# - 조사는 JOSA($변수, "을/를") 처럼 후보 쌍을 적는다. 앞 단어의 받침에 따라 고른다.
#   지원 쌍: 을/를, 이/가, 은/는, 과/와, 으로/로, 아/야, 이여/여
# - 버튼은 { BTN("menu") } 자리표시자. 문장 어디에든 둘 수 있다.

lang-name = 한국어
lang-font = NotoSansKR-Regular.otf

splash-hello = SLOT2에 오신 것을 환영합니다
splash-device = { $target } ({ $panel })에서 실행 중

hint-insert = { BTN("a") } 눌러 꽂기
hint-eject = { BTN("menu") } 길게 눌러 꺼내기
hint-menu = { BTN("menu") } 메뉴

cart-inserted = { JOSA($title, "을/를") } 꽂았습니다
cart-ejected = { JOSA($title, "을/를") } 꺼냈습니다
platform-switched = { JOSA($platform, "으로/로") } 전환했습니다

state-saved = { $n }번 슬롯에 { JOSA($title, "을/를") } 저장했습니다
state-loaded = { $n }번 슬롯을 불러왔습니다
states-empty = 아직 저장한 세이브 스테이트가 없습니다
state-save-failed = 세이브 스테이트를 저장하지 못했습니다
state-load-failed = 세이브 스테이트를 불러오지 못했습니다
resume-load-failed = 이어할 수 없어 새 게임을 시작합니다
state-deleted = { $n }번 슬롯을 지웠습니다
state-delete-failed = 세이브 스테이트를 지우지 못했습니다
state-restored = { $n }번 슬롯을 되돌렸습니다
state-restore-failed = 세이브 스테이트를 되돌리지 못했습니다
undo-empty = 되돌릴 것이 없습니다

state-switcher-title = 세이브 스테이트
state-slot = { $n }번 슬롯
hint-load = { BTN("a") } 불러오기
hint-delete = { BTN("x") } 지우기
hint-undo = { BTN("y") } 되돌리기
states-count = 세이브 스테이트 { $n }개

power-off = 전원 끄기
power-restart = 다시 시작
resume = 계속하기

power-menu-title = 전원
hint-select = { BTN("a") } 선택
hint-back = { BTN("b") } 뒤로

ingame-menu-title = 메뉴
ingame-continue = 계속하기
ingame-save-state = 세이브 스테이트 저장
ingame-cheats = 치트
ingame-display = 화면
ingame-core = 코어
ingame-device = 기기
ingame-eject = 카드 꺼내기
display-platform-default = 플랫폼 기본값
display-integer = 정수 배율
display-aspect-fit = 화면 비율 맞춤
display-fill = 화면 채우기
display-save-failed = 화면 설정을 저장하지 못했습니다

# 셰이더 하위 메뉴. 이름은 프리셋의 것이고, 첫 행은 선택이 없는 상태라 display-platform-default를
# 그대로 쓴다. zfast CRT는 프리셋 고유 이름이라 번역하지 않는다.
shader-title = 셰이더
shader-off = 끄기
shader-sharp-bilinear = 선명한 이중선형
shader-lcd3x = LCD 3배
shader-zfast-crt = zfast CRT
shader-scanline = 스캔라인
shader-save-failed = 셰이더 설정을 저장하지 못했습니다

# 오버스캔 하위 메뉴. 자를 것이 있는 기종에서만 Display를 거쳐 열린다. 첫 행은 선택이 없는 상태라
# display-platform-default를 그대로 쓴다.
overscan-title = 오버스캔
overscan-crop = 가장자리 자르기
overscan-full = 전체 이미지 표시
overscan-save-failed = 오버스캔 설정을 저장하지 못했습니다

# 오버레이 하위 메뉴. 그림은 기종·geometry마다 하나뿐이라 고를 이름이나 경로가 없다. 첫 행은 선택이
# 없는 상태라 display-platform-default를 그대로 쓴다.
overlay-title = 오버레이
overlay-on = 켜기
overlay-off = 끄기
overlay-save-failed = 오버레이 설정을 저장하지 못했습니다

# 코어 선택 화면. 여섯 이름은 고유명사라 어느 언어에서나 같은 표기다. 플레이어에게 라이브러리
# 파일 이름은 보이지 않는다.
core-name-mgba = mGBA
core-name-gambatte = Gambatte
core-name-gpsp = gpSP
core-name-fceumm = FCEUmm
core-name-snes9x = Snes9x
core-name-genesis-plus-gx = Genesis Plus GX
core-current = 현재
core-picker-empty = 사용 가능한 코어가 없습니다
core-picker-restart = 코어를 바꾸면 게임을 다시 시작합니다

# 기기 메뉴: 볼륨·밝기·블루라이트.
device-volume = 볼륨
device-brightness = 밝기
device-blue-light = 블루라이트
device-percent = { $value }%
device-muted = 음소거
device-unavailable = 사용 불가
hint-adjust = { BTN("left") }{ BTN("right") } 조절
hint-mute = { BTN("a") } 음소거

# 시간대 화면: UTC에서 더할 고정 분 수. 지역명이나 DST 규칙은 없다. 0도 부호를 남겨 두어
# 플레이어가 움직인 값과 손대지 않은 값이 구분된다. 설명 문구는 D-25가 정한 대로 이 값이 화면의
# 시계만 옮긴다는 것을 말한다.
timezone-title = 시간대
timezone-value = UTC{ $offset }
timezone-note = 표시에만 적용하며 시스템 시각은 UTC로 유지합니다
timezone-hint-adjust = { BTN("left") }{ BTN("right") } 15분  { BTN("up") }{ BTN("down") } 1시간
timezone-hint-apply = { BTN("a") } 적용
timezone-hint-cancel = { BTN("b") } 취소
timezone-save-failed = 시간대를 저장하지 못해 이전 값으로 돌아갔습니다

# 선반 설정 메뉴: M4가 확정한 여섯 항목을 그 순서대로 보여 준다. 화면이 아직 없는 항목도 자리를
# 지키며 사용 불가로 표시된다. 그래야 나머지 화면이 들어올 때 메뉴 순서가 흔들리지 않는다.
# 시간대 행은 그 화면의 제목을 그대로 쓴다.
shelf-menu-title = 설정
shelf-language = 언어
shelf-display-defaults = 기본 화면 설정
shelf-boot-logo = 부팅 로고
shelf-sync = 연동 모드
shelf-about = 정보

# 언어 선택 화면. 언어는 스스로를 부르는 이름으로 표시되므로 행의 이름은 그 언어팩의 `lang-name`이고,
# 옆의 code는 설정 파일에 쓰이는 값이다. 지금 쓰는 언어의 행에는 사용 중 표시가 붙는다. 위치 줄은
# 언어가 아니라 행을 센다.
language-code = { $code }
language-code-current = { $code } · 사용 중
language-picker-empty = 사용 가능한 언어가 없습니다
language-picker-position = { $current } / { $total }
language-load-failed = 선택한 언어를 불러오지 못해 이전 언어를 유지합니다
language-save-failed = 언어를 저장하지 못해 이전 언어를 유지합니다

# 정보 스티커. 넘겨받은 버전과 기기 이미지만 보여 준다. 바이너리에 리비전이 심어져 있지 않고,
# UI 크레이트의 패키지 버전은 프론트엔드의 것이 아니다. 라이선스 줄은 고정 제품 정보이고, 고지
# 줄은 배포 스크립트가 글꼴과 코어의 고지를 채워 넣는 폴더를 가리킨다.
about-title = 정보
about-wordmark = SLOT2
about-version = 버전 { $version }
about-target = 기기 { $target }
about-license = SLOT2 · MIT
about-notices = 코어 및 글꼴 고지: System/licenses

# 인게임 메뉴에서 코어를 바꿀 때. 각 실패 문구는 어느 내부 단계가 틀렸는지가 아니라 지금 무엇이
# 실행 중인지를 말한다.
core-no-alternatives = 이 선반에서 고를 수 있는 다른 코어가 없습니다
core-checkpoint-failed = 게임 상태를 저장하지 못해 코어를 바꾸지 않았습니다
core-switch-failed = 선택한 코어를 시작하지 못해 이전 코어로 돌아갔습니다
core-setting-save-failed = 코어 선택을 저장하지 못해 이전 코어로 돌아갔습니다
core-recovery-state-failed = 코어는 되돌렸지만 플레이 위치를 이어하지 못했습니다
core-recovery-failed = 코어 전환 뒤 게임을 되돌리지 못했습니다
cheat-toggle-failed = { $title }의 치트를 바꾸지 못했습니다
cheat-load-failed = { $title }의 치트를 읽지 못했습니다

cheat-empty = 이 게임에는 치트가 없습니다
cheat-enabled = 켜짐
cheat-disabled = 꺼짐
hint-cheat-toggle = { BTN("a") } 켜기/끄기


list-empty = 이 폴더에 게임이 없습니다
hint-play = { BTN("a") } 실행
hint-resume = { BTN("a") } 이어하기
hint-new-game = { BTN("a") } 길게 눌러 새로 시작

time-rewind = 되감기
time-fast-forward = { $speed }배속
hint-switch = { BTN("l1") }{ BTN("r1") } 기종
hint-back-to-list = { BTN("menu") } 길게 눌러 종료


# 선반이 실행하지 못하는 카트리지와 그 이유. 둘 다 플레이어가 직접 고칠 수 있는 것이라, 거절
# 표시(흔들림)만 두지 않고 문장도 여기에 있다.
core-missing = 이 선반의 코어가 없습니다. 카드의 System/cores에 넣어 주세요.
cart-broken = { JOSA($title, "을/를") } 시작하지 못했습니다.
