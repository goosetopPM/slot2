<#
.SYNOPSIS
  Cross-compile the test suites for aarch64, ship them to the Pi, run them there.

.DESCRIPTION
  The Pi 3B+ is the same Cortex-A53 as the H700, so this answers "does what we build for
  the device actually run on ARM Linux?" without the handheld: the cores we compile
  ourselves (`vendor/*.so`), glibc symbol use, pointer/alignment assumptions and real
  frame timings.

  Test binaries bake in `CARGO_MANIFEST_DIR` (= /src/crates/<name> inside the build
  container), and the tests reach assets through `../../assets`, so the Pi keeps a matching
  `/src/assets` and `/src/vendor`. `-Setup` creates them once.

.PARAMETER Crates
  Which packages to test. Default: the ones that are meaningful on aarch64.

.PARAMETER Setup
  Create /src on the Pi and copy assets + cores. Run once, and again when assets change.

.PARAMETER Pi
  user@host. Default goosetop@192.168.68.166.
#>
param(
    [string[]]$Crates = @('slot2-platform', 'slot2-i18n', 'slot2-text', 'slot2-store', 'slot2-audio', 'slot2-retro'),
    [switch]$Setup,
    [string]$Pi = 'goosetop@192.168.68.166'
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root
$image = 'slot2-cross'
$remote = '/src'

function Step($m) { Write-Host "==> $m" -ForegroundColor Cyan }

if ($Setup) {
    Step "preparing $remote on $Pi"
    # The crate directories must exist even though they stay empty: a test resolves its
    # assets as `<manifest dir>/../../assets`, and the kernel only walks `..` through
    # directories that are really there.
    $crateDirs = (Get-ChildItem crates -Directory | ForEach-Object { "$remote/crates/$($_.Name)" }) -join ' '
    ssh $Pi "sudo mkdir -p $remote/assets $remote/vendor $remote/bin $crateDirs && sudo chown -R `$USER $remote"
    if ($LASTEXITCODE -ne 0) { throw "could not create $remote (sudo password needed?)" }
    # assets/test/local is deliberately left behind: it holds real ROMs, which belong on
    # this machine and nowhere else. The tests that want one skip when it is absent.
    ssh $Pi "mkdir -p $remote/assets/test"
    scp -r assets/fonts assets/lang "${Pi}:$remote/assets/"
    $testFiles = Get-ChildItem assets/test -File
    if ($testFiles) { scp $testFiles "${Pi}:$remote/assets/test/" }
    $so = Get-ChildItem vendor\*_libretro.so -ErrorAction SilentlyContinue
    if ($so) { scp $so "${Pi}:$remote/vendor/" }
    ssh $Pi "ls -R $remote | head -40"
    Step "setup done"
}

$pkgArgs = ($Crates | ForEach-Object { "-p $_" }) -join ' '

Step "cross-compiling test binaries for aarch64"
$json = docker run --rm -v "${root}:/src" -v slot2-cargo:/cargo -v slot2-rustup:/usr/local/rustup -w /src $image `
    sh -c "cargo test --no-run --target aarch64-unknown-linux-gnu --target-dir target-device $pkgArgs --message-format=json 2>/dev/null"
if ($LASTEXITCODE -ne 0) { throw "cross build failed" }

$exes = @()
foreach ($line in $json) {
    if ($line -notmatch '"executable"') { continue }
    try { $o = $line | ConvertFrom-Json } catch { continue }
    if ($o.executable -and $o.profile.test) { $exes += $o.executable }
}
if (-not $exes) { throw "no test executables in cargo's output" }
Step "$($exes.Count) test binaries"

# Paths come back as container paths (/src/...); map them to the host.
$local = $exes | ForEach-Object { $_ -replace '^/src/', '' } | ForEach-Object { Join-Path $root ($_ -replace '/', '\') }
$missing = $local | Where-Object { -not (Test-Path $_) }
if ($missing) { throw "built but not found locally: $($missing -join ', ')" }

Step "copying to $Pi"
scp $local "${Pi}:$remote/bin/"
if ($LASTEXITCODE -ne 0) { throw "scp failed (run with -Setup first?)" }

Step "running on the Pi"
$names = ($local | ForEach-Object { Split-Path -Leaf $_ }) -join ' '
# Written to a file on the Pi rather than passed inline: a remote command goes through two
# shells, and every `$` in a loop would have to survive both.
$runner = "cd $remote`nfail=0`nfor t in $names; do`n  echo `"---- `$t ----`"`n  chmod +x bin/`$t`n  ./bin/`$t --test-threads=1 || fail=1`ndone`nexit `$fail`n"
$tmp = Join-Path $env:TEMP 'slot2-pi-run.sh'
[IO.File]::WriteAllText($tmp, $runner.Replace("`r`n", "`n"), (New-Object Text.UTF8Encoding $false))
scp $tmp "${Pi}:$remote/run.sh" | Out-Null
ssh $Pi "sh $remote/run.sh"
$code = $LASTEXITCODE
Step $(if ($code -eq 0) { "all green on aarch64" } else { "FAILURES on aarch64 (exit $code)" })
exit $code
