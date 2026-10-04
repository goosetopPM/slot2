# Task 102 worker result — 이전 정상 source bundle을 지키는 교체 상태 기계

## 1. 판정

**성공 (누적 시도 2/2, 최종).** 1차에서 `build/package-core-sources.ps1`의 마지막 교체 구간을 상태
기반으로 다시 썼고, 안전 실패 6종을 실제 저장소 사본 staging에서 주입 검증했다. 2차에서 Codex가 지적한
**부분 복사된 candidate의 소유권** 결함을 고쳐(§9), 실패 2종을 추가 검증하고 최종 4개 명령을 다시
실행했다. 모든 명령이 종료 코드 0으로 끝났고 검증 뒤 코드·내용 변경은 없다.

- 1차 호출(§2~§8): 교체 상태 기계, 안전 실패 6종, 최종 명령 4개 통과.
- 2차 호출(§9): 부분 복사 실패 2종(잠금 없음/잠금 있음) 통과, 최종 명령 4개 재통과.

## 2. 수정한 파일과 구현 (1차 호출)

바뀐 code/content는 `C:\SLOT2\build\package-core-sources.ps1` 하나뿐이다(테스트 전용 switch·환경
변수 없음, source/license/pin/archive/recipe/hash/deterministic/path 검증 약화 없음, repo root·
filesystem root 보호 유지).

### 2.1 교체 상태 (line 204-275)

`$parent`·`$candidate`·`$backup`·`[System.IO.Directory]::Move` 구조는 그대로 두고, 무조건 정리를
상태로 바꿨다.

| 상태 | 뜻 |
|---|---|
| `$backupHolds` | 이전 bundle이 `$backup` 이름으로 옆에 서 있다 |
| `$promoted` | candidate rename이 성공해 `$OutputDir`이 새 bundle을 들고 있다 |
| `$settled` | 승격과 **승격 후 재검증**까지 통과해 backup을 지워도 된다 |
| `$preserve` | rollback이 실패했다 — backup과 rejected output을 절대 건드리지 않는다 |

candidate는 상태 flag가 아니라 **파일 시스템 존재**(`Test-Path $candidate`)로 소유권을 판단한다. 이
GUID 경로는 이 실행만 만들 수 있고, 재귀 복사가 중간에 실패하면 flag를 세우지 못한 채 부분 디렉터리가
남기 때문이다(2차 수정, §9).

### 2.2 순서와 정리 규칙

1. candidate를 복사하고 **그 자리에서** `Assert-SourceBundle`을 다시 통과시킨다. 이 전에는
   `$OutputDir`을 건드리지 않는다 (line 218-226).
2. `Move($OutputDir, $backup)`이 실패하면 이전 output은 이름을 떠나지 않았다. 원래 예외가 그대로
   올라가고 candidate는 scratch로만 취급된다 (line 228-234).
3. 승격 rename 성공 뒤 `Assert-SourceBundle -Bundle $OutputDir`을 한 번 더 돌린다. 이 검사까지
   통과한 뒤에만 `$settled = $true`가 되고, 그때만 backup이 삭제 대상이 된다 (line 236-241, 269-271).
4. 승격 또는 승격 후 검증이 실패하면 `$backupHolds`일 때 이전 bundle을 되돌린다. 승격돼 있던
   rejected output은 먼저 **rename으로** candidate의 빈 이름으로 물러나므로 어느 tree도 삭제하지
   않는다 (line 242-258).
5. rollback이 실패하면 `$preserve = $true`로 두고, 원래 승격/검증 실패 + rollback 실패 + **정확한
   backup 경로** + 남아 있는 candidate/rejected-output 경로를 담은 오류를 던진다 (line 259-266).
6. `finally`는 `$preserve`면 아무것도 지우지 않는다. 그 외에도 candidate는 **존재할 때만**(부분 복사
   포함), 새 output은 이전 bundle이 돌아왔거나 애초에 없을 때만, **backup은 `$settled`일 때만** 지운다
   (line 273-284).
7. 삭제는 `Remove-TreeBestEffort`(line 46-54) 하나로 모았다. 열린 handle 때문에 실패하면
   `WARNING: … could not be removed and is still at <정확한 경로> (이유)`만 남기고 원래 오류를 가리지
   않는다.

### 2.3 normal directory guard (line 53-75)

`$OutputDir` 자리에 이미 무언가 있으면 `[System.IO.File]::GetAttributes`로 검사해 **일반 디렉터리가
아니면**(regular file, reparse point/junction 포함) staging·candidate를 만들기 전에 거부한다. Task 101
§12.6이 남긴 "깨진 junction이면 `Test-Path`가 false" 위험도 이 검사로 닫힌다.

## 3. 안전 실패 6종 (실제 저장소 사본 staging, 실제 checkout/junction 없음)

staging: `%TEMP%\slot2-task102-gate\repo` (필요 파일 복사 + `git clone --shared --no-checkout`로 만든
6개 checkout, HEAD는 pin). **reparse point 0개**라 통째로 지워도 실제 checkout을 따라가지 않는다.
하네스는 lock과 파일시스템 race만 외부에서 조정했고 스크립트에는 아무 hook도 넣지 않았다.

보존 hash는 "정렬한 `<sha256>  <bundle 기준 상대경로>` 줄들의 SHA-256"이다. 주입 전후가 같은
marker tree digest `c1d4fbcbdef2ce8b5e66fa036be37e3cd1f3e5cd09d922e2e312f0ff2ea8f593`(marker.txt +
keep.txt, 2 files)이다.

| # | 시나리오 | 주입 | 종료 | 보존/잔여물 |
|---|---|---|---|---|
| 1 | 정상 교체 (옛 marker tree 위) | 없음 | **0** | 28 files / 40,524,361 B, 옛 marker 사라짐, candidate·backup **없음**, 마지막 줄 `==> wrote 28 files …`, `Assert-SourceBundle` 재통과. bundle digest `6a45fe720b4033cbd57e20091e278a4e31f66cf3290e66dd4445f8f012a2e1a3` |
| 2 | move-aside 실패 | 옛 output의 `marker.txt`를 `FileShare.Read`로 붙잡음 | **1** | 이전 tree digest 동일, 잔여물 0, 정리 경고 0. 오류: `Access to the path '…\c2-out' is denied.` |
| 3 | 승격 실패 → rollback 성공 | candidate의 `SOURCE-MANIFEST.txt`를 외부 프로세스가 `FileShare.Read`로 보유 | **1** | `package-core-sources.ps1:232`(`Move($candidate, $OutputDir)`)에서 실패, 이전 tree digest 동일(2/2 files), candidate는 **주입한 handle 때문에** 남고 `WARNING … still at …\.c3-out.candidate-2110a8b2953745458d29040ecc378d90`로 정확한 경로 출력. handle 해제 뒤 잔여물 삭제 성공 |
| 4 | 승격 실패 → **rollback 실패** | output 이름에 외부 race 장애물(`.c4-out` 파일) 생성 | **1** | `restoring the previous bundle failed`, backup `.c4-out.backup-f7131252e80d4dc7aa19486827dd9dbb`가 **남아 있고 byte 동일**(2/2 files, digest `c1d4fbcb…`), 오류가 정확한 backup 경로와 `kept: the candidate at …\.c4-out.candidate-cde173f5cd8c4595ad08274dec017f02; the rejected output at …\c4-out`를 출력, 정리 단계가 아무것도 지우지 않음 |
| 5 | 잠긴 잔여물 정리 실패 | 옛 output + candidate 둘 다 잠금 | **1** | 이전 tree digest 동일, candidate `…\.c5-out.candidate-7c2cd40c30b14817acbc2213bc129d79`만 남고 **정확한 경로** 경고, 원래 오류 `Access to the path '…\c5-out' is denied.`도 보존. handle 해제 뒤 잔여물 삭제 성공 |
| 6 | output 자리가 regular file | 미리 `c6-out` 파일 생성 | **1** | `refusing to replace …\c6-out, which exists but is not a normal directory`. 파일 SHA-256 불변, candidate·backup 0, staging 디렉터리(`repo\target`)도 새로 생기지 않음(=candidate 생성 전 거부) |

case 3·5에서 "승격 실패 + rollback 성공"과 "정리 실패 경고"가, case 4에서 "rollback 실패 시 backup
보존 + 정확한 복구 경로"가 각각 관측됐다. case 4의 장애물은 "다른 프로세스가 output 이름을 차지한"
상황의 대역이다(계약이 허용한 외부 race).

## 4. 최종 검증 (1차 호출: 2026-10-01 10:14–10:29, 실제 저장소)

| # | 명령 | 종료 | 마지막 결과 줄 |
|---|---|---|---|
| 1 | `powershell -NoProfile -ExecutionPolicy Bypass -File build/package-core-sources.ps1 -OutputDir target/task102-core-sources-a` | 0 | `==> wrote 28 files (40524361 bytes) to C:\SLOT2\target\task102-core-sources-a` |
| 2 | `… -OutputDir target/task102-core-sources-b` | 0 | `==> wrote 28 files (40524361 bytes) to C:\SLOT2\target\task102-core-sources-b` |
| 3 | `powershell -NoProfile -ExecutionPolicy Bypass -File build/dist-device.ps1 -NoBuild -Zip` | 0 | `==> zipping dist\slot2-0.1.0-a8cb4af.zip` 뒤 `==> done` |
| 4 | `git diff --check` | 0 | stdout 진단 없음(stderr에 기존 CRLF 경고만) |

Rust 코드는 바뀌지 않아 Task 101이 통과한 workspace test·clippy는 반복하지 않았다.

## 5. 결정성·hash·dist

- 두 bundle: 각 28 files / 40,524,361 bytes. 정렬한 상대 경로 + 파일 SHA-256이 **서로 완전히 동일**.
- `SOURCE-MANIFEST.txt` SHA-256 `622b9a70f3b52e846b54e270b11c020898ce4b812c8172bf8e7765fbba42133d` —
  **Task 101과 같은 값**, 두 bundle 동일.
- `target/task101-core-sources-a`·`-b`(Task 101 산출물, 아직 존재)와 **28/28 파일 byte 동일**
  (상대 경로·SHA-256 차이 없음). Task 101 보고서가 공개한 파일별 hash 13개도 새 bundle에서 모두 일치.
- 정렬한 `path sha256` 목록(상대 경로 forward slash, LF, 끝 newline)의 SHA-256:
  `3bb9d6423a761e7f55452bb8f917150959776dd4f32ee853ac0c00eedbad52db` — task101-a/b, task102-a/b,
  배포본 `target/core-sources`, `dist-device\System\licenses\sources` **모두 같은 값**.
- `dist-device`: 55 files / 76,544,488 bytes, `System/licenses` 45 (archives 6, recipes 15, `.so.meta` 6,
  core license 6 등), `System/cores` 정확히 6, `System/VERSION.txt` = `SLOT2 0.1.0 (a8cb4af)`.
  `System/licenses/sources`는 28 files, manifest hash `622b9a70…`.
- zip `dist/slot2-0.1.0-a8cb4af.zip` 47,121,707 bytes, entries 57(files 55 + dir 2): cores 6, archives 6,
  source tree 28, `.so.meta` 6, patches 2 — 여섯 core/license/source tree가 그대로 유지됐다.

## 6. 변경 파일과 검증 뒤 변경 여부

- 바뀐 code/content: `build/package-core-sources.ps1`(마지막 write 10:05:36)와 이 보고서뿐이다.
- 최종 검증(10:14~10:29) **이후** code/content write 없음. 6시간 내 write된 저장소 파일은
  `build/package-core-sources.ps1`(검증 전)과 `target/**`·`dist/**`·`dist-device/**`(검증 산출물),
  그리고 내가 만들지 않은 `docs/HANDOFF-CODEX.md`(09:51)·에이전트 세션 파일뿐이다.
- 스크립트 위생: LF만(CR 0), BOM 없음, trailing whitespace 0, tab 0, 최대 줄 길이 126.

## 7. 계약 관련 의견

1. **Task 101의 `bundle digest` `c6e0cf89…`는 재현하지 못했다.** Task 101 보고서는 그 값을
   "정렬한 `path sha256` 목록의 SHA-256"이라고만 적고 경로 표기·구분자·인코딩을 적지 않았다.
   Task 101 자신의 산출물(`target/task101-core-sources-a`)이 아직 디스크에 있어 그 tree로 다시
   계산해 봤지만, 상대/절대·backslash/slash·1/2 space·tab·LF/CRLF·BOM·UTF-16 등 가능한 표기를
   모두 시도해도 같은 값이 나오지 않았다. 대신 더 강한 증거로 내용 불변을 확인했다: **Task 101
   bundle과 28/28 파일 byte 동일**, 공개된 파일별 hash 13개 일치, 위 §5의 (표기를 명시한) 목록
   digest가 다섯 tree에서 동일. 이 이상의 차이는 계약이 요구한 "concrete contract reason"에 해당한다.
2. **rollback 실패는 외부 흉내 없이는 만들 수 없다.** 같은 volume의 directory rename은 대상 이름이
   비어 있으면 실패할 이유가 없어서, case 4는 계약이 허용한 외부 장애물로 output 이름을 선점해
   만들었다. 실제 사고도 같은 모양(다른 프로세스가 이름/권한을 쥐고 있음)이라 판단한다.
3. **`git diff --check`는 이번 변경을 실제로 검사하지 않는다.** `build/package-core-sources.ps1`은
   이 작업 트리에서 untracked(??) 상태라 diff 대상이 아니다. 종료 0은 사실상 빈 검사이므로, 위생은
   §6처럼 직접 확인했다.
4. 잔여물 계약은 "성공·실패 모두 쓰레기 0"이 아니라 "정리가 실제로 실패한 경우에만, 정확한 경로
   경고와 함께 허용"으로 구현했다. case 2·6은 잔여물 0, case 3·5는 주입한 lock 때문에 잔여물 +
   경고, case 4는 backup·candidate 보존이 **의도된 결과**다.

## 8. 하네스 위치와 정리

- `%TEMP%\slot2-task102-gate\`(스크립트 `setup2.ps1`·`runner.ps1`·`holder.ps1`, 로그 `log-c*.txt`,
  `summary.txt`, staging `repo\`, case 상태). reparse point 0개. (§9에서 같은 gate를 재사용했다.)
- case 3·5의 테스트 handle은 프로세스를 종료해 해제했고, 잠겨 있던 candidate 잔여물도 삭제 성공했다.
  남아 있는 프로세스 0개(확인). case 4의 backup·candidate와 case 4·6의 대역 파일은 "정리가 지우면
  안 되는 것"의 증거로 그 자리에 남겨 두었다. 실제 `target/core-sources`, `target-device`, `dist*`,
  추적 license, core checkout은 rename·삭제·잠금·수정되지 않았다.

## 9. 2차 호출 (누적 2/2) — 부분 복사된 candidate

### 9.1 Codex가 지적한 결함

`build/package-core-sources.ps1:216-217`(1차 기준)에서 `$candidateHolds = $true`는 재귀
`Copy-Item`이 **반환한 뒤**에만 설정됐다. 복사가 candidate 디렉터리를 만들고 일부를 복사한 뒤
실패하면 `finally`는 `$candidateHolds`가 false인 채로 돌아, 그 부분 candidate를 지우지도 경고하지도
않았다(Task 102 요구 8 위반).

### 9.2 수정 (script만)

- `$candidateHolds`를 **삭제**했다. candidate 정리는 이제 파일 시스템 존재로 판단한다:
  `finally`에서 `Remove-TreeBestEffort -Path $candidate -What 'the rejected candidate'`를
  조건 없이 부르고(함수 내부가 `Test-Path`로 먼저 검사한다), 그 GUID 경로를 "이 실행이 만들 수 있는
  순간부터 이 실행 소유"로 취급한다.
- 승격된 output이 candidate로 오인될 여지는 없다: 승격 rename이 일어나면 candidate 이름은 사라지고,
  rollback이 rejected output을 candidate 이름으로 물릴 때만 **다시** 이 실행 소유가 된다.
- `$preserve`(rollback 실패) 규칙과 backup 보존 규칙은 그대로다. 제품용 테스트 switch·환경변수는
  여전히 없다.

### 9.3 주입 방법 (스크립트 무수정, 외부 lock만)

staging bundle의 `recipes\snes9x\commit`를 외부 프로세스가 `FileShare.None`으로 잡는다. 이 항목은
재귀 복사가 `SOURCE-MANIFEST.txt` → `archives`(40MB) → `licenses` → `recipes` 순으로 도달하는 **늦은**
항목이라, 복사가 candidate 디렉터리와 27개 항목을 만든 뒤 실패한다(probe로 순서 확인).
- 7a: stage 항목만 잠금 → 복사 실패 → 정리는 lock 없음.
- 7b: 위와 동시에, candidate가 나타나는 즉시 그 안의 파일(관측: `SOURCE-MANIFEST.txt`)에
  `FileShare.Read` handle을 건다 → 정리 실패.

### 9.4 결과 (실패 시나리오 2종)

| 시나리오 | 종료 | 이전 output | 부분 candidate | 정리/경고 |
|---|---|---|---|---|
| **7a** 부분 복사, lock 없음 | **1** | digest 동일(2/2 files) | 관측 최대 27 files, 종료 시 잔여물 **0** | 경고 없음(정리 성공), 복사 오류 보존 |
| **7b** 부분 복사, candidate 잠금 | **1** | digest 동일(2/2 files) | 남은 잔여물 1개(정리가 일부 삭제 후 잠긴 파일에서 실패), digest `8fbb6a45c1e30f66cdefc4227470408d04ef5566cea4d90004f30f87e23fe91b` | 경고가 `.c7b-out.candidate-ee66e60401bd407e8242dbc418610364`를 **정확한 경로**로 출력 |

- 복사 오류는 두 경우 모두 보인다: `Copy-Item : The process cannot access the file '<stage>\recipes\snes9x\commit' because it is being used by another process.`
  (at `build/package-core-sources.ps1:218`), 그리고 정리 경고가 이 오류를 가리지 않는다.
- 7b의 잔여물은 잠금 handle을 해제한 뒤 삭제 성공했다. 잠긴 stage 항목 때문에 스크립트 자신의 stage
  정리(`Remove-Item $stage`, 원래 `-ErrorAction SilentlyContinue`)도 실패했는데, 이는 주입의 부작용이며
  handle 해제 뒤 harness가 정리했다.
- 1차의 실패 6종도 같은 실행에서 재검증해 모두 1차와 동일한 결과를 얻었다(정상 교체 0, move-aside
  1·잔여물 0, 승격 실패 rollback 성공 1·이전 tree 동일, rollback 실패 1·backup byte 동일·정확한 경로,
  잠긴 잔여물 경고 1, regular file 1).

### 9.5 최종 명령 재실행 (2차, 2026-10-01 10:46–10:49)

| # | 명령 | 종료 | 마지막 결과 줄 |
|---|---|---|---|
| 1 | `… package-core-sources.ps1 -OutputDir target/task102-core-sources-a` | 0 | `==> wrote 28 files (40524361 bytes) to C:\SLOT2\target\task102-core-sources-a` |
| 2 | `… package-core-sources.ps1 -OutputDir target/task102-core-sources-b` | 0 | `==> wrote 28 files (40524361 bytes) to C:\SLOT2\target\task102-core-sources-b` |
| 3 | `… build/dist-device.ps1 -NoBuild -Zip` | 0 | `==> zipping dist\slot2-0.1.0-a8cb4af.zip` 뒤 `==> done` |
| 4 | `git diff --check` | 0 | stdout 진단 없음(stderr에 기존 CRLF 경고만) |

- 두 bundle은 28 files / 40,524,361 bytes로 서로 **완전히 동일**하고, `target/task101-core-sources-a`·`-b`와도
  28/28 파일 byte 동일하다. manifest sha256 `622b9a70f3b52e846b54e270b11c020898ce4b812c8172bf8e7765fbba42133d`,
  명시한 목록 digest `3bb9d6423a761e7f55452bb8f917150959776dd4f32ee853ac0c00eedbad52db` — 둘 다 1차와 같다.
  Task 101 공개 hash 13개도 다시 일치.
- `dist-device` 55 files / 76,544,488 bytes, `System/licenses` 45, `System/cores` 6, `System/licenses/sources`
  28 files(같은 manifest·digest). zip 47,121,707 bytes / entries 57(files 55 + dir 2), cores 6, archives 6,
  source tree 28, `.so.meta` 6, patches 2 — 1차와 동일.
- Rust 코드는 바뀌지 않아 cargo test·clippy는 다시 돌리지 않았다.

### 9.6 변경 파일과 검증 뒤 변경 여부 (2차)

- 바뀐 code/content는 `build/package-core-sources.ps1`(마지막 write 10:45:15, sha256
  `72173e5eb28145bcd2437366ddd890d6fca7a4e247c30c6758ae70d55dfa4e82`)와 이 보고서뿐이다. 1차의 변경은
  여기에 누적돼 있다.
- 4개 명령은 10:46–10:49에 실행됐고, 그 뒤 code/content write 없음(6시간 내 write는 위 script, 검증
  산출물 `target/**`·`dist/**`·`dist-device/**`, 내가 만들지 않은 `docs/HANDOFF-CODEX.md`·에이전트 세션
  파일뿐). 스크립트는 LF만(CR 0), BOM 없음, trailing whitespace 0, tab 0, 최대 줄 126자.

### 9.7 남은 계약 의견 (2차)

1. stage 정리는 이번 계약의 대상이 아니지만, 주입한 stage lock 때문에 스크립트 자신의 stage 삭제가
   조용히 실패할 수 있다는 점을 관측했다(원래도 `-ErrorAction SilentlyContinue`). 실제 사고에서는
   `target/` 아래에 `package-core-sources-<guid>` 쓰레기가 남을 수 있다는 뜻이며, 다음에 다룰 값이
   있다면 그 구간도 같은 정책(시도 후 실패하면 정확한 경로 경고)으로 맞추는 편이 일관된다.
2. 7b의 잔여물은 잠긴 파일 1개만 남는다. `Remove-Item -Recurse -Force`는 잠긴 항목을 만날 때까지
   지우고 실패하므로 "잔여물이 있다"는 사실과 경고의 정확한 경로가 계약이 요구한 증거다.
