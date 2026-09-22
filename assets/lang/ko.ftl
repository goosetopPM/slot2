# SLOT2 — 한국어
#
# 규칙 (docs/DESIGN.md §8):
# - 문장은 통째로. 코드가 조각을 이어 붙이지 않는다.
# - 조사는 JOSA($변수, "을/를") 처럼 후보 쌍을 적는다. 앞 단어의 받침에 따라 고른다.
#   지원 쌍: 을/를, 이/가, 은/는, 과/와, 으로/로, 아/야, 이여/여
# - 버튼은 { BTN("menu") } 자리표시자. 문장 어디에든 둘 수 있다.

lang-name = 한국어
lang-font = NotoSansKR-Regular.ttf

splash-hello = SLOT2에 오신 것을 환영합니다
splash-device = { $target } ({ $panel })에서 실행 중

hint-insert = { BTN("a") } 눌러 꽂기
hint-eject = { BTN("menu") } 길게 눌러 꺼내기
hint-menu = { BTN("menu") } 메뉴

cart-inserted = { JOSA($title, "을/를") } 꽂았습니다
cart-ejected = { JOSA($title, "을/를") } 꺼냈습니다
platform-switched = { JOSA($platform, "으로/로") } 전환했습니다

state-saved = { $n }번 슬롯에 { JOSA($title, "을/를") } 저장했습니다
states-count = 세이브 스테이트 { $n }개

power-off = 전원 끄기
power-restart = 다시 시작
resume = 계속하기
