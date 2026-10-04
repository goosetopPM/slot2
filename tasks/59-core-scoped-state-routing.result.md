# Task 59 — Codex 최종 판정

## 판정

**통과. 누적 호출 2/2.** 시도 1은 provider 400으로 중단됐고, 새 대화형 세션의 마지막 허용
호출에서 현재 working tree를 이어 완성했다.

## 확인한 내용

- `resolve_core`가 설정 이름, 파일 존재, 공식 코어의 플랫폼 지원, 기본 fallback과 최종
  `StateNamespace`를 한 곳에서 결정한다. Session launch와 App rescan이 같은 resolver를 사용한다.
- 설정 core는 core 디렉터리 안의 파일 이름으로만 처리된다. 경로 성분이 있는 값은 기본 core로
  fallback하며 core 디렉터리 밖 library를 선택하지 않는다.
- 공식 core는 canonical base name, 외부 core는 유효한 lowercase stem 또는 고정 FNV-1a 기반
  `external_...` namespace를 사용한다. 영속 경로에 process-seeded hasher를 쓰지 않는다.
- Session은 실제로 연 core의 namespace를 소유하고 numbered state와 Resume을 모두 scoped API로
  읽고 쓴다. save RAM 공유 계약은 바뀌지 않았다.
- App rescan은 평면 legacy state를 플랫폼 기본 core namespace로 먼저 이전한 뒤, 실제 선택 core
  namespace의 Resume만 캐시한다. 충돌 시 원본·목적지를 보존하고 둘을 합치지 않는다.
- Resume/Fresh, 퀵세이브·로드, 스위처 목록·로드, 삭제와 undo가 현재 Session namespace만 사용한다.
  Fresh는 다른 core의 Resume과 thumbnail을 보존한다.
- 제품 실행 경로에는 migration 입력 외 평면 state API 호출이 남지 않았다. 남은 평면 호출은 legacy
  이전 통합 테스트와 `#[cfg(test)]`의 평면 `StateBackup` 수명 fixture다.

## 격리 증거

- 같은 GBA cart에서 mGBA와 gpSP가 같은 slot 번호와 Resume에 서로 다른 bytes를 저장한다.
- gpSP는 mGBA Resume을 보거나 load하지 않으며, 각 core로 돌아가면 자기 state가 다시 보인다.
- legacy state는 gpSP가 선택돼 있어도 mGBA namespace로 이전되고 gpSP shelf hint에 나타나지 않는다.
- quick state와 switcher의 load/delete/undo가 다른 core의 state와 PNG를 보존한다.
- mGBA와 gpSP host core를 사용한 통합 테스트는 skip 없이 실행됐다.

## 작업자 검증 증거

- `cargo fmt --all -- --check` — 종료 0
- `cargo test -p slot2` — 176 passed / 0 failed / 0 ignored, 실코어 skip 없음
- `cargo clippy -p slot2 --all-targets -- -D warnings` — 종료 0
- 최종 검증 뒤 코드 변경 없음

Codex는 사용자 운영 규칙에 따라 테스트를 다시 실행하지 않고 보고서와 resolver·제품 배선·통합
테스트를 대조했다.

## 남은 제한

외부 core stem은 lowercase canonical로 접으므로 대소문자만 다른 두 외부 library는 대소문자 구분
파일시스템에서 같은 namespace를 쓴다. 설정과 `CoreId::from_library_path`가 공식 core를 이미
대소문자 비구분으로 해석하는 현재 계약과 일치하며, 지원 core 전환에는 영향이 없다.

## 다음 방향

state 안전 경계가 완성됐으므로 다음은 GB/GBC와 GBA에만 나타나는 CorePicker UI다. UI는
`supported_cores(platform)`과 실제 설치 파일을 입력으로 선택·취소·현재값 표시만 구현하고, 설정
저장과 Session 재시작은 별도 App 배선 태스크로 분리한다.
