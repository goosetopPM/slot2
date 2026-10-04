# Task 101 — Codex 최종 판정

## 판정

**실패 (누적 호출 2/2). 자동 재시도 중단.** 라이선스·source bundle 내용과 일반 성공 경로는
검증됐지만, 마지막 수정의 실패 복구가 계약을 끝까지 만족하지 않는다. 세 번째 작업자 지시서는 만들지
않는다.

## 확인된 정상 범위

- root MIT, upstream 원문, 여섯 core의 pinned license blob과 pristine source archive·recipe·patch가
  요구한 구조와 byte/hash 계약을 만족한다.
- 두 bundle의 28개 파일과 SHA-256이 동일하며 local dist·zip·CI assembly에 연결됐다.
- 작업자 2차 검증은 workspace **973 passed / 0 failed / 0 ignored**, clippy warning 0이고 아홉 완료
  명령이 모두 종료 코드 0이었다.
- 같은 parent의 candidate를 먼저 완성·검증하고 `[System.IO.Directory]::Move`로 교체하는 기본 방향은
  올바르다. 일반 move-aside 실패와 candidate 승격 실패에서는 이전 output이 보존됐다.

## 차단 결함

1. `build/package-core-sources.ps1:190`의 backup 복구 rename이 실패하면 그 예외가 전파되지만,
   `:195-198`의 바깥 `finally`가 candidate와 **backup을 무조건 삭제**한다. 따라서 외부 잠금·권한·I/O
   오류로 rollback까지 실패하면 직전 정상 bundle을 담은 유일한 backup을 삭제할 수 있다. 이는 이번
   수정의 핵심인 “실패가 직전 정상 source bundle을 삭제하면 안 된다”는 계약을 다시 위반한다.
2. 2차 지시서는 성공·실패 모두 candidate/backup 쓰레기가 남지 않아야 한다고 명시했다. 그러나 작업자
   보고서 §12.3의 승격 실패 실험은 열린 handle 때문에 candidate가 남았다고 기록했고, §12.6에서도
   `.candidate-*`/`.backup-*`가 남을 수 있다고 인정한다. 계약을 충족하지 못한 상태를 성공으로 보고했다.

안전한 복구의 우선순위는 이전 정상 bundle 보존이다. rollback이 실패하면 backup을 절대 지우지 않고
복구 가능한 경로와 오류를 남겨야 한다. 외부 프로세스가 파일을 잠근 동안 중간 디렉터리를 반드시
삭제하라는 계약은 Windows에서 보장할 수 없으므로, 이 부분은 계약을 “안전한 잔여물은 허용하되 기존
bundle은 보존”으로 수정할지 사용자 결정이 필요하다.

## 다음 상태

동일 태스크가 누적 2/2에 도달했으므로 자동 수정은 중단한다. 사용자가 계약 조정 또는 별도 후속 태스크를
명시해야 한다.
