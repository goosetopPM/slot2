<#
.SYNOPSIS
  Build the SLOT2 device binary in the cross container and assemble the card tree.

.DESCRIPTION
  Produces dist-device\ with exactly what goes onto the card root (BaseOS TF1 "BASEOS"
  volume, or a FAT32/exFAT TF2 card):

    System\frontend            aarch64 binary, launched by BaseOS's frontend-session
    System\Fonts\*.otf|*.ttf   UI + CJK fonts (the CJK one is loaded lazily on the device)
    System\licenses\           font licenses (cores join here in M1)

  Copy the *contents* of dist-device\ to the card root.

.PARAMETER Adb
  After assembling, push System\ onto a connected device at /mnt/sdcard and reboot it.

.PARAMETER Zip
  Also write dist\slot2-<version>-<git>.zip for a release.

.PARAMETER NoBuild
  Skip the cargo build and just (re)assemble from the last build.

.EXAMPLE
  .\build\dist-device.ps1
  .\build\dist-device.ps1 -Adb
  .\build\dist-device.ps1 -Zip
#>
param(
    [switch]$Adb,
    [switch]$Zip,
    [switch]$NoBuild
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$image = 'slot2-cross'
$bin = 'target-device/aarch64-unknown-linux-gnu/device/slot2'
$out = Join-Path $root 'dist-device'

function Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }

if (-not $NoBuild) {
    docker image inspect $image *> $null
    if ($LASTEXITCODE -ne 0) {
        Step "building cross image $image (first time only)"
        docker build -t $image -f build/cross.Dockerfile build
        if ($LASTEXITCODE -ne 0) { throw "docker build failed" }
    }

    Step "cross-compiling slot2 for aarch64 (profile: device)"
    docker run --rm -v "${root}:/src" -v slot2-cargo:/cargo -v slot2-rustup:/usr/local/rustup -w /src $image `
        sh -c "cargo build --profile device --target aarch64-unknown-linux-gnu --target-dir target-device -p slot2 --no-default-features --features device && file $bin"
    if ($LASTEXITCODE -ne 0) { throw "device build failed" }
}

if (-not (Test-Path $bin)) { throw "missing $bin (build first)" }

Step "assembling $out"
if (Test-Path $out) { Remove-Item -Recurse -Force $out }
New-Item -ItemType Directory -Force "$out\System\Fonts" | Out-Null
New-Item -ItemType Directory -Force "$out\System\licenses\fonts" | Out-Null
Copy-Item $bin "$out\System\frontend"
Copy-Item assets\fonts\*.otf, assets\fonts\*.ttf "$out\System\Fonts\"
Copy-Item assets\fonts\*.txt "$out\System\licenses\fonts\"

$version = (Select-String -Path Cargo.toml -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
$sha = (git rev-parse --short HEAD 2>$null)
if (-not $sha) { $sha = 'nogit' }
@"
SLOT2 $version ($sha)
Copy the contents of this folder to the root of the card BaseOS boots a frontend from.
BaseOS runs System/frontend. Logs: /tmp/frontend.log on the device, System/slot2-diag.txt on the card.
"@ | Out-File -Encoding utf8 "$out\System\VERSION.txt"

Get-ChildItem -Recurse -File $out | ForEach-Object {
    "{0,10}  {1}" -f $_.Length, $_.FullName.Substring($out.Length + 1)
}

if ($Zip) {
    New-Item -ItemType Directory -Force dist | Out-Null
    $zipPath = "dist\slot2-$version-$sha.zip"
    if (Test-Path $zipPath) { Remove-Item $zipPath }
    Step "zipping $zipPath"
    Compress-Archive -Path "$out\*" -DestinationPath $zipPath
}

if ($Adb) {
    Step "deploying over adb to /mnt/sdcard"
    adb wait-for-device
    adb shell 'mkdir -p /mnt/sdcard/System/Fonts /mnt/sdcard/System/licenses'
    adb push "$out\System\frontend" /mnt/sdcard/System/frontend
    adb push "$out\System\Fonts\." /mnt/sdcard/System/Fonts/
    adb push "$out\System\licenses\." /mnt/sdcard/System/licenses/
    adb push "$out\System\VERSION.txt" /mnt/sdcard/System/VERSION.txt
    adb shell 'sync'
    Step "rebooting device"
    adb reboot
}

Step "done"
