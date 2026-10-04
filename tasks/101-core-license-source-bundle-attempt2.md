# Task 101 attempt 2 — source bundle 교체의 트랜잭션 보장

누적 호출 **2/2, 마지막 호출**이다. `tasks/101-core-license-source-bundle.md`의 전체 계약과
`tasks/101-core-license-source-bundle.worker-result.md`의 1차 결과를 먼저 읽고, 아래 결함만 수정한다.

## 발견된 결함

`build/package-core-sources.ps1:162-164`는 검증된 staging을 만든 뒤 기존 `OutputDir`을 먼저 삭제하고,
빈 `OutputDir`을 만든 다음 파일을 복사한다. 이 복사가 디스크 부족·권한·I/O 오류로 중간 실패하면 직전
정상 bundle은 사라지고 부분 output이 남는다. 원 계약의 다음 문장을 만족하지 않는다.

> 실패가 직전 정상 source bundle을 삭제하거나 부분 output을 남기면 안 된다.

1차 보고서의 negative staging은 모두 preflight 실패라서 이 교체 구간을 검증하지 못했다.

## 수정 계약

- `build/package-core-sources.ps1`의 최종 교체를 **같은 parent에 둔 완성 candidate + rollback 가능한
  directory rename** 방식으로 바꾼다.
- 기존 `OutputDir`은 candidate 복사와 candidate 자체 검증이 모두 끝날 때까지 건드리지 않는다.
- 교체 시 기존 output을 임시 backup 이름으로 옮긴 뒤 candidate를 `OutputDir`로 rename/move한다.
  candidate 승격이 실패하면 backup을 원래 이름으로 복구하고 부분 `OutputDir`을 남기지 않는다.
- 성공 뒤 backup을 정리한다. 성공·실패 어느 쪽이든 candidate/staging/backup 쓰레기를 남기지 않는다.
- `OutputDir`이 repo 밖이거나 다른 volume이어도 candidate는 반드시 `OutputDir`의 parent에 만들어
  최종 rename이 같은 filesystem 안에서 일어나게 한다.
- repo root·filesystem root 보호와 deterministic bytes, archive/license/recipe/hash 계약은 유지한다.
- `build/package-core-sources.ps1`과 기존 worker result 외에는 변경하지 않는다. 테스트만을 위한 production
  switch나 환경변수는 추가하지 않는다.

## 검증

수정 뒤 원 Task101 완료 기준 명령 9개를 모두 다시 실행한다. 추가로 다음을 보고서에 근거와 함께 적는다.

1. 기존 marker/tree가 있는 output을 정상 교체하면 새 bundle만 남고 candidate/backup이 없다.
2. 교체 구간 실패를 안전한 임시 staging에서 유발해 기존 marker/tree가 보존되고 부분 `OutputDir`,
   candidate, backup이 남지 않는다. 실제 `target/core-sources`, `dist-device`, checkout, tracked license를
   rename/delete하지 않는다.
3. 두 packager output의 relative paths와 SHA-256이 여전히 exact equality다.
4. workspace는 **973 passed보다 줄지 않고**, failed/ignored 0, clippy warning 0, full dist 마지막 줄
   `==> done`이다.

검증 뒤 code/content를 바꾸지 않는다. 커밋·push·network·ADB·Samba·SD 카드·공용 설정 변경·위임은
금지한다.

`C:\SLOT2\tasks\101-core-license-source-bundle.worker-result.md`를 기존 내용을 보존한 **누적 2/2**
보고서로 갱신한다. 수정 내용, rollback 검증 방법과 결과, 아홉 명령의 종료 코드/마지막 결과 줄,
최종 변경 파일, 검증 뒤 변경 여부를 기록한다. 실패해도 보고서를 남기고 세 번째 호출은 하지 않는다.
