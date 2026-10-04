# Task 26 — OpenCodex를 통한 B.AI 워커 검증

사용자 선택: 현재 Codex 유지, B.AI 코드 생성 워커만 연결.

- OpenCodex 2.64.0을 `.gjc-logs/ocx-worker/install`에 설치.
- 독립 OPENCODEX_HOME, 루프백 10126 포트, 전역 Codex/Claude/Grok 연동과 자동 shim 수정 비활성화.
- BAI_API_KEY는 사용자 환경에서 프로세스로만 주입. 키를 출력하거나 파일에 저장하지 않음.
- `glm-5.3-flash`에 최소 요청을 보내고 도구 호출 왕복 및 별도 Codex CLI 워커에서 파일 읽기를 확인.
- 현재 Codex의 config.toml과 bai-gjc 공용 YAML의 해시를 전후 비교.
- 원래 저장소 소스 및 공유 설정 수정·커밋·실기 조작 금지.
- 동일 검증 실패 최대 2회. 성공하지 못하면 원인을 기록하고 중단.

실행 결과와 정확한 재사용 명령은 결과 보고서에 남긴다.
