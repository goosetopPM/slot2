# Task 27 — 시계 런타임 오프셋 기반

사용자가 개발 재개를 요청했다. 인수인계 5항의 시계 선행 작업 묶음이다.
사용자 승인된 OpenCodex B.AI 워커로 실행. CLAUDE.md/AGENTS.md의 역할 분리·재위임 조항은 오케스트레이터 전용이므로 무시한다. 추가 워커 실행·커밋·푸시·실기 접근·공용 설정 수정 금지.

수정 범위: `crates/slot2-platform/src/clock.rs`와 관련 새 테스트, `crates/slot2/src/app.rs` HUD 시각 배선, `crates/slot2/src/main.rs` 부팅 배너만. 문서는 오케스트레이터가 수정한다.

요구사항:
1. 환경변수는 기존과 같이 초기값만 제공하고 기본값 UTC 0 유지. 런타임 setter로 오프셋 즉시 변경 가능. AtomicI32를 이용해 프레임마다 환경 조회나 잠금 없이 읽기. 초기화와 setter 경합에도 설정값을 뒤늦게 환경값으로 덮어쓰지 않을 것. 기존 OnceLock<i32> 고정값 제거.
2. setter의 유효 범위 OFFSET_MIN..=OFFSET_MAX. 범위 밖 입력은 거부하고 기존 값 유지. 호출자가 성공/실패를 알 수 있는 API. UTC 원본/시스템 시계 변경 금지.
3. HUD 표시 여부는 동일한 UTC 샘플로 is_set을 판정한 뒤, 표시 시각에만 오프셋 적용. 테스트 가능한 순수 helper를 clock 모듈에 두어도 됨. UTC 경계 SET_AFTER 전후를 +14/-12 오프셋에서도 일관되게 판정. 과한 추상화 금지.
4. 부팅 배너에 utc_offset_min과 utc_now(epoch)를 추가. 의존성 추가 없음.
5. 테스트는 런타임 변경·범위 밖 거부·UTC 판정 경계를 검증. 전역 상태 테스트끼리 경합하지 않게 독립 통합 테스트 바이너리나 로컬 상태를 활용. 기존 테스트 삭제/약화 금지. 호출자 산술 unwrap 금지, allow lint 금지.

완료 기준 명령 원문:
`cargo test -p slot2-platform`
`cargo fmt --all -- --check`

오케스트레이터 최종 검증:
`cargo test --workspace`
`cargo clippy --workspace --all-targets -- -D warnings`
`powershell -File build/dist-device.ps1`

재시도 최대 2회. 첫 응답 최대 5분, 전체 최대 45분. 로그 버퍼링 정상. 진행 신호 git status --short. 기존 미커밋 문서와 tasks/24~26 보존.
결과는 `tasks/27-clock-runtime.worker-result.md`: 성공/실패, 마지막 테스트 결과 줄, 실패 원인 1줄, 변경 파일 목록, 계약이 틀리면 해당 줄. 테스트 출력 전문 금지.
