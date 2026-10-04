<#
.SYNOPSIS
  Build the device cores in the cross container and fetch matching host cores for Windows.

.DESCRIPTION
  Device (.so, aarch64): cores/<name>/build.sh runs inside slot2-cross; the result lands in
  vendor/<name>_libretro.so with a .meta beside it recording pin, patches and flags. A core
  whose .meta matches `build.sh stamp` is up to date and skipped.

  Host (.dll, x86_64 Windows): fetched from the libretro buildbot into vendor/, for
  `cargo run` on this PC. Not what ships; the device core is built from source.

  Which cores, when -Core is not given, is cores/required.txt — the same manifest
  build/dist-device.ps1 distributes and CI builds. There is no second list here to forget.

.PARAMETER Core
  Which cores to handle. Default: every core in cores/required.txt, in its order.

.PARAMETER Force
  Rebuild / refetch even if up to date.

.PARAMETER HostOnly / DeviceOnly
  Do just one side.
#>
param(
    [string[]]$Core,
    [switch]$Force,
    [switch]$HostOnly,
    [switch]$DeviceOnly
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root
. (Join-Path $PSScriptRoot 'core-manifest.ps1')
New-Item -ItemType Directory -Force vendor | Out-Null

function Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }

$image = 'slot2-cross'

# Read and validate the manifest before the loop, so a bad list is an error and never a
# partially built set of cores. An explicit -Core is the caller's own list and is checked row
# by row below, the way it always was.
if (-not $Core) { $Core = Get-CoreManifest -Root $root }

foreach ($name in $Core) {
    if (-not (Test-Path (Join-Path $root "cores\$name\build.sh"))) { throw "no cores\$name\build.sh" }

    if (-not $HostOnly) {
        # The container sees the repository at /src, so what is handed to build.sh is the path
        # inside it; the host paths are for the checks either side.
        $soRel = "vendor/${name}_libretro.so"
        $so = Join-Path $root "vendor/${name}_libretro.so"
        $meta = "$so.meta"
        $stamp = docker run --rm -v "${root}:/src" -w /src $image sh "cores/$name/build.sh" stamp
        if ($LASTEXITCODE -ne 0) { throw "stamp failed for $name (is the $image image built?)" }
        $current = if (Test-Path $meta) { (Get-Content $meta -Raw).Trim() } else { '' }
        if (-not $Force -and (Test-Path $so) -and ($current -eq (($stamp -join "`n").Trim()))) {
            Step "$name device core is current ($soRel)"
        } else {
            Step "building $name device core (aarch64, LTO) — a few minutes"
            docker run --rm -v "${root}:/src" -w /src $image sh "cores/$name/build.sh" build "target-device/cores/$name" $soRel
            if ($LASTEXITCODE -ne 0) { throw "device core build failed for $name" }
        }
    }

    if (-not $DeviceOnly) {
        $dllRel = "vendor\${name}_libretro.dll"
        $dll = Join-Path $root $dllRel
        if (-not $Force -and (Test-Path $dll)) {
            Step "$name host core present ($dllRel)"
        } else {
            Step "fetching $name host core from the libretro buildbot"
            $url = "https://buildbot.libretro.com/nightly/windows/x86_64/latest/${name}_libretro.dll.zip"
            $zip = Join-Path $root "vendor\${name}_host.zip"
            $ProgressPreference = 'SilentlyContinue'
            Invoke-WebRequest -Uri $url -OutFile $zip
            Expand-Archive -Path $zip -DestinationPath (Join-Path $root vendor) -Force
            Remove-Item $zip
            if (-not (Test-Path $dll)) { throw "buildbot zip did not contain ${name}_libretro.dll" }
        }
    }
}

Get-ChildItem (Join-Path $root vendor) | Select-Object Name, Length
