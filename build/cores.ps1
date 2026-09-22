<#
.SYNOPSIS
  Build the device cores in the cross container and fetch matching host cores for Windows.

.DESCRIPTION
  Device (.so, aarch64): cores/<name>/build.sh runs inside slot2-cross; the result lands in
  vendor/<name>_libretro.so with a .meta beside it recording pin, patches and flags. A core
  whose .meta matches `build.sh stamp` is up to date and skipped.

  Host (.dll, x86_64 Windows): fetched from the libretro buildbot into vendor/, for
  `cargo run` on this PC. Not what ships; the device core is built from source.

.PARAMETER Core
  Which cores to handle. Default: all known.

.PARAMETER Force
  Rebuild / refetch even if up to date.

.PARAMETER HostOnly / DeviceOnly
  Do just one side.
#>
param(
    [string[]]$Core = @('mgba'),
    [switch]$Force,
    [switch]$HostOnly,
    [switch]$DeviceOnly
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root
New-Item -ItemType Directory -Force vendor | Out-Null

function Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }

$image = 'slot2-cross'

foreach ($name in $Core) {
    if (-not (Test-Path "cores\$name\build.sh")) { throw "no cores\$name\build.sh" }

    if (-not $HostOnly) {
        $so = "vendor/${name}_libretro.so"
        $meta = "$so.meta"
        $stamp = docker run --rm -v "${root}:/src" -w /src $image sh "cores/$name/build.sh" stamp
        if ($LASTEXITCODE -ne 0) { throw "stamp failed for $name (is the $image image built?)" }
        $current = if (Test-Path $meta) { (Get-Content $meta -Raw).Trim() } else { '' }
        if (-not $Force -and (Test-Path $so) -and ($current -eq (($stamp -join "`n").Trim()))) {
            Step "$name device core is current ($so)"
        } else {
            Step "building $name device core (aarch64, LTO) — a few minutes"
            docker run --rm -v "${root}:/src" -w /src $image sh "cores/$name/build.sh" build "target-device/cores/$name" $so
            if ($LASTEXITCODE -ne 0) { throw "device core build failed for $name" }
        }
    }

    if (-not $DeviceOnly) {
        $dll = "vendor\${name}_libretro.dll"
        if (-not $Force -and (Test-Path $dll)) {
            Step "$name host core present ($dll)"
        } else {
            Step "fetching $name host core from the libretro buildbot"
            $url = "https://buildbot.libretro.com/nightly/windows/x86_64/latest/${name}_libretro.dll.zip"
            $zip = "vendor\${name}_host.zip"
            $ProgressPreference = 'SilentlyContinue'
            Invoke-WebRequest -Uri $url -OutFile $zip
            Expand-Archive -Path $zip -DestinationPath vendor -Force
            Remove-Item $zip
            if (-not (Test-Path $dll)) { throw "buildbot zip did not contain ${name}_libretro.dll" }
        }
    }
}

Get-ChildItem vendor | Select-Object Name, Length
