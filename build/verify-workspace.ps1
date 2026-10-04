# 미검증 초안: Task 29 두 번 실패로 미채택. 실행하지 말고 결과 보고서를 먼저 확인한다.
param(
    [Parameter(Mandatory = $true)][string]$RunId,
    [ValidateSet('Fmt', 'Test', 'Clippy', 'Dist')]
    [string[]]$Steps = @('Fmt', 'Test', 'Clippy', 'Dist')
)

$ErrorActionPreference = 'Stop'

if ($RunId -notmatch '^[A-Za-z0-9._-]+$') {
    Write-Error 'RunId must match [A-Za-z0-9._-]+.'
    exit 2
}

$root = Split-Path -Parent $PSScriptRoot
$logDir = Join-Path $root '.gjc-logs'
if (-not (Test-Path -LiteralPath $logDir)) { New-Item -ItemType Directory -Path $logDir | Out-Null }

# 명시 순서와 무관하게 항상 canonical 순서로 남은 검사만 수행한다.
$canonical = @('Fmt', 'Test', 'Clippy', 'Dist')
$selected = @($canonical | Where-Object { $Steps -contains $_ })
$summaryPath = Join-Path $logDir ($RunId + '.verify-summary.json')
if (Test-Path -LiteralPath $summaryPath) {
    Write-Error 'RunId already has a verify summary; refusing overwrite.'
    exit 2
}

# 실행 전에 선택된 모든 단계 로그를 검사해 부분 실행/덮어쓰기를 막는다.
$precheckFail = $null
foreach ($step in $selected) {
    $log = Join-Path $logDir ($RunId + '.' + $step + '.log')
    if (Test-Path -LiteralPath $log) { $precheckFail = $log; break }
}
if ($precheckFail) {
    Write-Error ('step log already exists; refusing partial run: ' + $precheckFail)
    exit 2
}

$lockPath = Join-Path $logDir 'verify.lock'
$lockStream = $null
$swTotal = [System.Diagnostics.Stopwatch]::StartNew()
$results = @()
$failedStep = $null
$failReason = $null

function Invoke-Step([string]$Name, [string[]]$Cmd) {
    $log = Join-Path $logDir ($RunId + '.' + $Name + '.log')
    if (Test-Path -LiteralPath $log) { throw ('log exists for ' + $Name + '; refusing overwrite') }
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    $exit = 1
    $output = @()
    try {
        $output = @(& $Cmd[0] $Cmd[1..($Cmd.Length - 1)] 2>&1 | ForEach-Object { $_.ToString() })
        $exit = $LASTEXITCODE
    } catch {
        $output = @('EXCEPTION: ' + $_.Exception.Message)
        $exit = 1
    }
    $sw.Stop()
    $output | Set-Content -LiteralPath $log -Encoding UTF8

    $passed = 0; $failedCnt = 0; $resultLines = 0
    foreach ($line in $output) {
        # 실제 형식: "test result: ok. 3 passed; 0 failed; ..." / "test result: FAILED. ..."
        if ($line -match 'test result:\s*(?:ok|FAILED)\.\s*(\d+) passed;\s*(\d+) failed') {
            $resultLines++
            $passed += [int]$Matches[1]
            $failedCnt += [int]$Matches[2]
        }
    }

    $ok = ($exit -eq 0)
    if ($ok -and $Name -eq 'Test' -and ($failedCnt -gt 0 -or $passed -lt 372)) {
        $ok = $false
        Set-Variable -Name failReason -Value ('Test baseline violated: passed=' + $passed + ' failed=' + $failedCnt) -Scope 2
    }
    if ($ok -and $Name -eq 'Dist') {
        if (-not ($output | Where-Object { $_ -like '*==> done*' })) {
            $ok = $false
            Set-Variable -Name failReason -Value 'Dist finished without marker: ==> done' -Scope 2
        }
    }
    if (-not $ok -and -not $failReason) {
        Set-Variable -Name failReason -Value ($Name + ' exited with code ' + $exit) -Scope 2
    }

    $label = 'FAIL'; if ($ok) { $label = 'OK' }
    # 진행 문구는 Write-Host로 return 스트림을 오염시키지 않는다.
    Write-Host ($Name + ': ' + $label + ' (exit ' + $exit + ', ' + [math]::Round($sw.Elapsed.TotalSeconds, 1) + 's)')
    return [pscustomobject]@{
        step = $Name; command = ($Cmd -join ' '); exit = $exit
        elapsed_sec = [math]::Round($sw.Elapsed.TotalSeconds, 1); ok = $ok
        passed = $passed; failed = $failedCnt; test_result_lines = $resultLines
    }
}

function New-FailedStep([string]$Name, [string]$Reason) {
    return [pscustomobject]@{ step = $Name; command = ''; exit = 1; elapsed_sec = 0; ok = $false; passed = 0; failed = 0; test_result_lines = 0; reason = $Reason }
}

try {
    # FileShare.None으로 동시 검증 실행을 차단한다(다른 워커와의 상호배제까지는 보장하지 않음).
    $lockStream = [System.IO.File]::Open($lockPath, [System.IO.FileMode]::OpenOrCreate, [System.IO.FileAccess]::ReadWrite, [System.IO.FileShare]::None)

    # lock 안에서 summary 중복 생성 경쟁을 재확인한다.
    if (Test-Path -LiteralPath $summaryPath) {
        Write-Error 'verify summary appeared while waiting for lock; refusing overwrite.'
        exit 2
    }

    # 모든 외부 명령은 프로젝트 루트 기준으로 실행한다.
    Push-Location $root
    try {
        foreach ($step in $selected) {
            $r = $null
            try {
                switch ($step) {
                    'Fmt'    { $r = Invoke-Step 'Fmt' @('cargo', 'fmt', '--all', '--', '--check') }
                    'Test'   { $r = Invoke-Step 'Test' @('cargo', 'test', '--workspace') }
                    'Clippy' { $r = Invoke-Step 'Clippy' @('cargo', 'clippy', '--workspace', '--all-targets', '--', '-D', 'warnings') }
                    'Dist' {
                        # 실제 재귀 삭제 대상인 root\dist-device의 절대경로와 reparse point를 검사한다.
                        $expectedDist = [System.IO.Path]::GetFullPath((Join-Path $root 'dist-device'))
                        $distScript = [System.IO.Path]::GetFullPath((Join-Path $root 'build\dist-device.ps1'))
                        if (-not (Test-Path -LiteralPath $expectedDist -PathType Container)) { throw ('missing dist-device directory: ' + $expectedDist) }
                        $distItem = Get-Item -LiteralPath $expectedDist -Force
                        if ($distItem.Attributes -band [System.IO.FileAttributes]::ReparsePoint) { throw 'dist-device is a reparse point' }
                        if (-not (Test-Path -LiteralPath $distScript -PathType Leaf)) { throw ('missing dist script: ' + $distScript) }
                        $r = Invoke-Step 'Dist' @('powershell', '-NoProfile', '-File', $distScript)
                    }
                }
            } catch {
                $r = New-FailedStep $step $_.Exception.Message
                Write-Host ($step + ': FAIL (' + $_.Exception.Message + ')')
                Set-Variable -Name failReason -Value ($step + ': ' + $_.Exception.Message) -Scope 2
            }
            $results += $r
            if (-not $r.ok) { $failedStep = $step; break }
        }
    } finally {
        Pop-Location
    }
} catch {
    if (-not $failedStep) { $failedStep = 'setup' }
    $failReason = 'top-level: ' + $_.Exception.Message
    Write-Host ('VERIFY ERROR: ' + $failReason)
} finally {
    if ($lockStream) { $lockStream.Close() }
}

$swTotal.Stop()
$summary = [ordered]@{
    run_id      = $RunId
    status      = if ($failedStep) { 'failed' } else { 'completed' }
    failed_step = $failedStep
    fail_reason = $failReason
    elapsed_sec = [math]::Round($swTotal.Elapsed.TotalSeconds, 1)
    steps       = $results
}
$summary | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $summaryPath -Encoding UTF8
Write-Output ('summary: ' + $summaryPath)
if ($failedStep) { exit 1 }
exit 0
