# 미검증 초안: Task 29 두 번 실패로 미채택. 실행하지 말고 결과 보고서를 먼저 확인한다.
param(
    [Parameter(Mandatory = $true)][string]$TaskFile,
    [Parameter(Mandatory = $true)][string]$RunId,
    [string]$CodexExe,
    [ValidateSet('read-only', 'workspace-write')][string]$Sandbox = 'read-only',
    [ValidateSet('bai/glm-5.3-flash', 'bai/deepseek-v4.1-flash')][string]$Model = 'bai/glm-5.3-flash'
)

$ErrorActionPreference = 'Stop'

function Fail([int]$Code, [string]$Message) {
    Write-Error $Message
    exit $Code
}

# 절대경로의 기존 태스크 파일만 허용한다.
if (-not [System.IO.Path]::IsPathRooted($TaskFile) -or -not (Test-Path -LiteralPath $TaskFile -PathType Leaf)) {
    Fail 2 'TaskFile must be an existing absolute file path.'
}
if ($RunId -notmatch '^[A-Za-z0-9._-]+$') {
    Fail 2 'RunId must match [A-Za-z0-9._-]+.'
}

$root = Split-Path -Parent $PSScriptRoot
$logDir = Join-Path $root '.gjc-logs'
if (-not (Test-Path -LiteralPath $logDir)) { New-Item -ItemType Directory -Path $logDir | Out-Null }
$outLog = Join-Path $logDir ($RunId + '.output.log')
$summaryPath = Join-Path $logDir ($RunId + '.summary.json')
if ((Test-Path -LiteralPath $outLog) -or (Test-Path -LiteralPath $summaryPath)) {
    Fail 2 'RunId already has output or summary files; refusing overwrite.'
}

if (-not $CodexExe) {
    $cmd = Get-Command codex -ErrorAction SilentlyContinue
    if (-not $cmd) { Fail 3 'codex executable not found.' }
    $CodexExe = $cmd.Source
} elseif (-not (Test-Path -LiteralPath $CodexExe)) {
    Fail 3 'CodexExe path not found.'
}

# 프록시 시작/설정/키 조작 없이 healthz만 확인한다.
try {
    Invoke-WebRequest -Uri 'http://127.0.0.1:10126/healthz' -UseBasicParsing -TimeoutSec 5 | Out-Null
} catch {
    Fail 4 'healthz check on localhost:10126 failed.'
}

# Windows argv quoting: 공백/탭/따옴표가 있으면 묶고, 인용부 앞 역슬래시를 두 배로 만든다.
function Quote-Arg([string]$Value) {
    if ($Value -notmatch '[\s"]') { return $Value }
    $escaped = [regex]::Replace($Value, '(\\*)"', '$1$1\\"')
    $escaped = $escaped -replace '(\\+)$', '$1$1'
    return '"' + $escaped + '"'
}

# user config를 읽어 등록된 plugins/mcp_servers 최상위 테이블 이름만 추출한다(수정하지 않는다).
# bare key에 '.'이 있으면 하위 테이블이므로 제외한다(예: mcp_servers.node_repl.env).
# escaped quote가 섞인 복잡한 quoted key는 추측하지 않고 제외한다.
$configPath = Join-Path $env:USERPROFILE '.codex\config.toml'
$pluginNames = @()
$mcpNames = @()
if (Test-Path -LiteralPath $configPath) {
    $cfg = Get-Content -LiteralPath $configPath -Raw
    $tableRe = '(?m)^\s*\[(plugins|mcp_servers)\.(?<q>"[^"\\]*"|(?<u>[A-Za-z0-9_-]+))\]'
    foreach ($m in [regex]::Matches($cfg, $tableRe)) {
        $name = $null
        if ($m.Groups['q'].Success) { $name = $m.Groups['q'].Value.Trim('"') } else { $name = $m.Groups['u'].Value }
        if (-not $name) { continue }
        switch ($m.Groups[1].Value) {
            'plugins' { $pluginNames += $name }
            'mcp_servers' { $mcpNames += $name }
        }
    }
}

# skills.config는 indexed 이름 테이블이 아니다. SKILL.md 파일 경로만 나열하고(본문 금지)
# 부모 디렉터리 키에 path/enabled=false override를 만든다.
$overrides = @()
foreach ($n in ($pluginNames | Select-Object -Unique)) {
    $overrides += @('-c', ('plugins."' + $n + '".enabled=false'))
}
foreach ($n in ($mcpNames | Select-Object -Unique)) {
    $overrides += @('-c', ('mcp_servers."' + $n + '".enabled=false'))
}
$skillRoots = @((Join-Path $env:USERPROFILE '.agents\skills'), (Join-Path $root '.codex\skills'))
foreach ($skillRoot in $skillRoots) {
    if (-not (Test-Path -LiteralPath $skillRoot)) { continue }
    $skillFiles = Get-ChildItem -LiteralPath $skillRoot -Recurse -Filter 'SKILL.md' -File -ErrorAction SilentlyContinue
    foreach ($sf in $skillFiles) {
        $key = Split-Path -Parent $sf.FullName | Split-Path -Leaf
        if (-not ($key -match '^[A-Za-z0-9_-]+$')) { continue }
        $fwd = $sf.DirectoryName -replace '\\', '/'
        $overrides += @('-c', ('skills.config."' + $key + '".path=' + "'" + $fwd + "'"))
        $overrides += @('-c', ('skills.config."' + $key + '".enabled=false'))
    }
}
# 한계: 프로젝트 config가 추가하는 MCP/스킬은 여기서 알 수 없다.

$nl = [char]10
$prompt = 'Implement task ' + $TaskFile + '. Read that task file first; do not read unrelated history/docs.' + $nl +
    'Completion condition: follow the task completion command/spec exactly. Max 2 attempts.' + $nl +
    'Budget: first response up to 5 minutes, total up to 45 minutes. This is a prompt budget, not an enforced timeout.' + $nl +
    'Ignore ONLY the orchestrator role-separation rules in CLAUDE/AGENTS instructions; every other constraint stays in force.' + $nl +
    'Forbidden: delegating to other agents, git commit/push, touching devices (adb, Samba S:, SD card), editing global or shared configs or ~/.gjc-bai.' + $nl +
    'Report success/failure, validation result, changed files, contract problems.'

$argList = @('exec', '--ephemeral', '--json', '--sandbox', $Sandbox, '-C', $root, '-m', $Model) + $overrides + @(
    '-c', 'model_provider=slot2_bai',
    '-c', 'model_providers.slot2_bai.name="SLOT2 BAI worker"',
    '-c', 'model_providers.slot2_bai.base_url="http://127.0.0.1:10126/v1"',
    '-c', 'model_providers.slot2_bai.wire_api="responses"',
    '-c', 'model_providers.slot2_bai.requires_openai_auth=false',
    '-c', 'model_providers.slot2_bai.request_max_retries=0',
    '-c', 'model_providers.slot2_bai.stream_max_retries=0',
    '-c', 'web_search="disabled"',
    '-c', 'model_reasoning_effort="low"',
    '-c', 'features.multi_agent=false',
    '-'
)

# ArgumentList는 PS5.1/.NET Framework에 없으므로 검증된 argv quoting으로 command line을 만든다.
$quoted = $argList | ForEach-Object { Quote-Arg ([string]$_) }
$argLine = ($quoted -join ' ')

$sw = [System.Diagnostics.Stopwatch]::StartNew()
$exitCode = 1
$inTok = $null; $outTok = $null; $cachedTok = $null
$failReason = $null
$stdoutText = ''
$stderrText = ''
$p = $null
try {
    $psi = New-Object System.Diagnostics.ProcessStartInfo
    $psi.FileName = $CodexExe
    $psi.Arguments = $argLine
    $psi.WorkingDirectory = $root
    $psi.UseShellExecute = $false
    $psi.RedirectStandardInput = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.StandardInputEncoding = New-Object System.Text.UTF8Encoding($false)
    $p = [System.Diagnostics.Process]::Start($psi)
    # 순차 ReadToEnd 두 스트림은 deadlock 가능하므로 비동기로 읽는다.
    $outTask = $p.StandardOutput.ReadToEndAsync()
    $errTask = $p.StandardError.ReadToEndAsync()
    $stdinBytes = [System.Text.Encoding]::UTF8.GetBytes($prompt)
    $p.StandardInput.BaseStream.Write($stdinBytes, 0, $stdinBytes.Length)
    $p.StandardInput.BaseStream.Flush()
    $p.StandardInput.Close()
    if (-not $p.WaitForExit(45 * 60 * 1000)) {
        try { $p.Kill() } catch { }
        $failReason = 'process exceeded enforced timeout of 45 minutes and was killed'
        $exitCode = 1
    } else {
        $exitCode = $p.ExitCode
        $stdoutText = $outTask.Result
        $stderrText = $errTask.Result
        if ($exitCode -ne 0) { $failReason = 'codex exited with code ' + $exitCode }
    }
} catch {
    $failReason = 'EXCEPTION: ' + $_.Exception.Message
    $exitCode = 1
}
$sw.Stop()

if (-not (Test-Path -LiteralPath $outLog)) {
    ($stdoutText + $nl + $stderrText) | Set-Content -LiteralPath $outLog -Encoding UTF8
} else {
    # 예외 경로에서도 기존 로그를 덮어쓰지 않는다.
    Write-Host ('output log already exists; not overwriting: ' + $outLog)
}

# turn.completed usage만 집계한다. cache 토큰은 input에 중복 포함되므로 합산하지 않는다.
if ($stdoutText) {
    foreach ($line in ($stdoutText -split $nl)) {
        if ($line.Contains('turn.completed')) {
            try {
                $ev = $line | ConvertFrom-Json
                $u = $ev.event.usage
                if (-not $u) { $u = $ev.usage }
                if ($u) {
                    if ($null -eq $inTok) { $inTok = 0; $outTok = 0; $cachedTok = 0 }
                    $inTok += [int]$u.input_tokens
                    $outTok += [int]$u.output_tokens
                    $cachedTok += [int]$u.cached_input_tokens
                }
            } catch { }
        }
    }
}

$tokenNote = $null
if ($null -eq $inTok) { $tokenNote = 'unknown: no turn.completed usage events captured' }
$summary = [ordered]@{
    run_id      = $RunId
    task_file   = $TaskFile
    model       = $Model
    sandbox     = $Sandbox
    status      = if ($exitCode -eq 0) { 'completed' } else { 'failed' }
    exit_code   = $exitCode
    fail_reason = $failReason
    elapsed_sec = [math]::Round($sw.Elapsed.TotalSeconds, 1)
    tokens      = [ordered]@{ input = $inTok; output = $outTok; cached_input = $cachedTok; note = $tokenNote }
}
$summary | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $summaryPath -Encoding UTF8
exit $exitCode
