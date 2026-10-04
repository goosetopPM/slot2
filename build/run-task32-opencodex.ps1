param()

$ErrorActionPreference = 'Stop'
if (Test-Path variable:PSNativeCommandUseErrorActionPreference) {
    $PSNativeCommandUseErrorActionPreference = $false
}

$codexExe = 'C:\Users\gyuha\AppData\Local\OpenAI\Codex\bin\13995fba801849b0\codex.exe'
$taskFile = 'C:\SLOT2\tasks\32-ingame-menu-ui.opencodex.md'
$resultFile = 'C:\SLOT2\tasks\32-ingame-menu-ui.worker-result.md'
$jsonPath = 'C:\SLOT2\.gjc-logs\32-opencodex-attempt2.jsonl'
$stderrPath = 'C:\SLOT2\.gjc-logs\32-opencodex-attempt2.stderr.log'
$answerPath = 'C:\SLOT2\.gjc-logs\32-opencodex-attempt2-answer.txt'
$summaryPath = 'C:\SLOT2\.gjc-logs\32-opencodex-attempt2-summary.json'

foreach ($required in @($codexExe, $taskFile)) {
    if (-not (Test-Path -LiteralPath $required -PathType Leaf)) {
        throw "Required file not found: $required"
    }
}

foreach ($output in @($jsonPath, $stderrPath, $answerPath, $summaryPath, $resultFile)) {
    if (Test-Path -LiteralPath $output) {
        throw "Refusing to overwrite an existing Task 32 output: $output"
    }
}

$health = Invoke-RestMethod -Uri 'http://127.0.0.1:10126/healthz' -TimeoutSec 5
if ($health.status -ne 'ok') {
    throw "OpenCodex health check failed: $($health.status)"
}

$taskPrompt = @'
Execute C:\SLOT2\tasks\32-ingame-menu-ui.opencodex.md directly.
That ASCII task file is the complete contract. Do not read project handoff, roadmap, design, decisions, CLAUDE, legacy operations notes, or unrelated task reports.
Read only the allowed source files and directly imported UI code required to compile.
This is cumulative implementation attempt 2/2 and the final automatic attempt.
Do not delegate, commit, push, access hardware, or edit global/shared configuration.
Write the required result to C:\SLOT2\tasks\32-ingame-menu-ui.worker-result.md.
'@

$workerArgs = @(
    'exec'
    '--ephemeral'
    '--json'
    '--sandbox', 'workspace-write'
    '-C', 'C:\SLOT2'
    '-m', 'bai/glm-5.3-flash'
    '-c', "model_provider='slot2_bai'"
    '-c', "model_providers.slot2_bai.name='SLOT2 OpenCodex worker'"
    '-c', "model_providers.slot2_bai.base_url='http://127.0.0.1:10126/v1'"
    '-c', "model_providers.slot2_bai.wire_api='responses'"
    '-c', 'model_providers.slot2_bai.requires_openai_auth=false'
    '-c', 'model_providers.slot2_bai.request_max_retries=0'
    '-c', 'model_providers.slot2_bai.stream_max_retries=0'
    '-c', "web_search='disabled'"
    '-c', "model_reasoning_effort='low'"
    '-c', 'features.multi_agent=false'
    '-o', $answerPath
    '-'
)

Write-Host 'Task 32 is running through OpenCodex. Wait for this command to finish.'
$timer = [System.Diagnostics.Stopwatch]::StartNew()
$savedErrorActionPreference = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
$taskPrompt | & $codexExe @workerArgs 1> $jsonPath 2> $stderrPath
$workerExit = $LASTEXITCODE
$ErrorActionPreference = $savedErrorActionPreference
$timer.Stop()

$usage = $null
Get-Content -LiteralPath $jsonPath | ForEach-Object {
    try {
        $event = $_ | ConvertFrom-Json
        if ($event.type -eq 'turn.completed' -and $event.usage) {
            $usage = $event.usage
        }
    } catch {
        # Preserve malformed diagnostic lines in the JSONL; they are not usage records.
    }
}

$summary = [ordered]@{
    task = 32
    implementation_attempt = 2
    exit_code = $workerExit
    elapsed_seconds = [math]::Round($timer.Elapsed.TotalSeconds, 1)
    result_file_exists = Test-Path -LiteralPath $resultFile
    usage = $usage
    json_log = $jsonPath
    stderr_log = $stderrPath
}
$summary | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $summaryPath -Encoding UTF8

Write-Host "OpenCodex exit code: $workerExit"
Write-Host "Result exists: $($summary.result_file_exists)"
Write-Host "Summary: $summaryPath"
if ($workerExit -ne 0) {
    exit $workerExit
}
