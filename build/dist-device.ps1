<#
.SYNOPSIS
  Build the SLOT2 device binary in the cross container and assemble the card tree.

.DESCRIPTION
  Produces dist-device\ with exactly what goes onto the card root (BaseOS TF1 "BASEOS"
  volume, or a FAT32/exFAT TF2 card):

    System\frontend            aarch64 binary, launched by BaseOS's frontend-session
    System\cores\*.so          the libretro cores, one per shelf
    System\Fonts\*.otf|*.ttf   UI + CJK fonts (the CJK one is loaded lazily on the device)
    System\licenses\           the SLOT2 and upstream notices, every core's license, the
                             corresponding source: pristine archives plus SLOT2's recipe, and
                             the Rust runtime's third-party notices with their SBOM

  Copy the *contents* of dist-device\ to the card root.

  What has to be in it is cores/required.txt, the same manifest build/cores.ps1 builds and CI
  checks. A card without one of those cores boots to an empty shelf and looks like a bug, so
  every core's .so and .meta are checked — before the previous tree is deleted when they are
  missing, and again in the assembled tree before anything is zipped or pushed. The same goes
  for the licensing: the notices and the pinned source are built by
  build/package-core-sources.ps1 first, verified, and only then copied in; the third-party Rust
  notices and their SBOM are built by build/package-rust-notices.ps1 and verified the same way.

.PARAMETER Adb
  After assembling, push System\ onto a connected device at /mnt/sdcard and reboot it.

.PARAMETER Zip
  Also write dist\slot2-<version>-<git>.zip for a release.

.PARAMETER NoBuild
  Skip the cargo build and the core build, and (re)assemble from the last build. The core
  checks still run: this assembles the manifest's cores or it fails.
#>
param(
    [switch]$Adb,
    [switch]$Zip,
    [switch]$NoBuild
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root
. (Join-Path $PSScriptRoot 'core-manifest.ps1')
. (Join-Path $PSScriptRoot 'rust-notices.ps1')

$image = 'slot2-cross'
$bin = Join-Path $root 'target-device/aarch64-unknown-linux-gnu/device/slot2'
$out = Join-Path $root 'dist-device'

function Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }

# The manifest is the list of cores a card must have, read and validated first: a list that
# cannot be acted on is an error before anything is built.
$cores = Get-CoreManifest -Root $root

if (-not $NoBuild) {
    docker image inspect $image *> $null
    if ($LASTEXITCODE -ne 0) {
        Step "building cross image $image (first time only)"
        docker build -t $image -f build/cross.Dockerfile build
        if ($LASTEXITCODE -ne 0) { throw "docker build failed" }
    }

    Step "cross-compiling slot2 for aarch64 (profile: device)"
    docker run --rm -v "${root}:/src" -v slot2-cargo:/cargo -v slot2-rustup:/usr/local/rustup -w /src $image `
        sh -c "cargo build --profile device --target aarch64-unknown-linux-gnu --target-dir target-device -p slot2 --no-default-features --features device && file target-device/aarch64-unknown-linux-gnu/device/slot2"
    if ($LASTEXITCODE -ne 0) { throw "device build failed" }

    Step "building the device cores (cores/required.txt)"
    & (Join-Path $PSScriptRoot 'cores.ps1') -DeviceOnly
    if ($LASTEXITCODE -ne 0) { throw "core build failed" }
}

if (-not (Test-Path $bin)) { throw "missing $bin (build first)" }

# Every check that can fail happens before the existing tree is touched: a run that cannot
# assemble the manifest's cores must leave the last good dist-device\ where it was.
$coreFiles = Assert-CoreDeviceFiles -Root $root -Names $cores

# The Rust runtime's third-party notices are part of the tree too, so they are built and checked
# here, still ahead of the delete below. Like the core source bundle this is a build product, not
# repository content: it lives under target/. It is read from the local Cargo cache only.
$rustBundle = Join-Path $root 'target/rust-notices'
Step "packaging Rust runtime notices ($($cores.Count) cores, device closure)"
& (Join-Path $PSScriptRoot 'package-rust-notices.ps1') -OutputDir $rustBundle
if ($LASTEXITCODE -ne 0) { throw "rust notice packaging failed" }
Assert-RustNoticeBundle -Root $root -Bundle $rustBundle

# The notices, each core's license and the corresponding source are part of the tree too, so
# they are built and verified here, still ahead of the delete below. The bundle is a build
# product, not repository content: it lives under target/ like everything else this builds.
$bundle = Join-Path $root 'target/core-sources'
Step "packaging core sources (cores/required.txt)"
& (Join-Path $PSScriptRoot 'package-core-sources.ps1') -OutputDir $bundle
if ($LASTEXITCODE -ne 0) { throw "core source packaging failed" }
Assert-SourceBundle -Root $root -Bundle $bundle -Names $cores

Step "assembling $out"
if (Test-Path $out) { Remove-Item -Recurse -Force $out }
New-Item -ItemType Directory -Force "$out\System\Fonts" | Out-Null
New-Item -ItemType Directory -Force "$out\System\cores" | Out-Null
New-Item -ItemType Directory -Force "$out\System\licenses\fonts" | Out-Null
New-Item -ItemType Directory -Force "$out\System\licenses\upstream-slot" | Out-Null
New-Item -ItemType Directory -Force "$out\System\licenses\cores" | Out-Null
New-Item -ItemType Directory -Force "$out\System\licenses\sources" | Out-Null
New-Item -ItemType Directory -Force "$out\System\licenses\rust" | Out-Null
Copy-Item $bin "$out\System\frontend"
Copy-Item assets\fonts\*.otf, assets\fonts\*.ttf "$out\System\Fonts\"
Copy-Item assets\fonts\*.txt "$out\System\licenses\fonts\"
Copy-Item (Join-Path $root 'LICENSE') "$out\System\licenses\SLOT2-LICENSE"
Copy-Item (Join-Path $root 'CORE-NOTICES.md') "$out\System\licenses\CORE-NOTICES.md"
Copy-Item (Join-Path $root 'licenses/upstream-slot/LICENSE') "$out\System\licenses\upstream-slot\LICENSE"
Get-ChildItem -LiteralPath $bundle -Force | Copy-Item -Destination "$out\System\licenses\sources" -Recurse -Force
Get-ChildItem -LiteralPath $rustBundle -Force | Copy-Item -Destination "$out\System\licenses\rust" -Recurse -Force
# The manifest's cores, in its order — never a glob: a stray Windows .dll or a core this
# build did not ask for is not something to hand a player.
foreach ($core in $coreFiles) {
    Copy-Item $core.So "$out\System\cores\"
    Copy-Item $core.Meta "$out\System\licenses\"
    $license = Get-CoreLicenseName -Name $core.Name
    New-Item -ItemType Directory -Force "$out\System\licenses\cores\$($core.Name)" | Out-Null
    Copy-Item (Join-Path $root "licenses/cores/$($core.Name)/$license") "$out\System\licenses\cores\$($core.Name)\$license"
    Step ("core: " + $core.Name)
}
Assert-CoreTree -CoresDir "$out\System\cores" -LicensesDir "$out\System\licenses" -Names $cores
Assert-LicensesTree -LicensesDir "$out\System\licenses" -Names $cores
# The copied Rust notices are checked as the tree that is about to be zipped, not as the bundle
# they came from: the exact package set, every SBOM field, every copied byte and every manifest
# hash is re-read from System/licenses/rust before anything leaves this machine.
Assert-RustNoticeBundle -Root $root -Bundle "$out\System\licenses\rust"

# The stamp is the one file a user reads before copying anything, and the workflow writes the same
# three lines with printf, so it is built byte for byte here: UTF-8 with no byte-order mark, and one
# LF after each line. The shell's text writers would add a mark and end the file with CRLF on Windows
# PowerShell, which the hosted version-stamp gate rejects, so the bytes are written explicitly instead.
$version = (Select-String -Path Cargo.toml -Pattern '^version = "(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
$sha = (git rev-parse --short HEAD 2>$null)
if (-not $sha) { $sha = 'nogit' }
$stampLines = @(
    "SLOT2 $version ($sha)"
    'Copy the contents of this folder to the root of the card BaseOS boots a frontend from.'
    'BaseOS runs System/frontend. Logs: /tmp/frontend.log on the device, System/slot2-diag.txt on the card.'
)
$stampPath = Join-Path $out 'System\VERSION.txt'
[System.IO.File]::WriteAllText($stampPath, ($stampLines -join "`n") + "`n", (New-Object System.Text.UTF8Encoding($false)))

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
    adb shell 'mkdir -p /mnt/sdcard/System/Fonts /mnt/sdcard/System/licenses /mnt/sdcard/System/cores'
    adb push "$out\System\frontend" /mnt/sdcard/System/frontend
    adb push "$out\System\Fonts\." /mnt/sdcard/System/Fonts/
    adb push "$out\System\licenses\." /mnt/sdcard/System/licenses/
    adb push "$out\System\cores\." /mnt/sdcard/System/cores/
    adb push "$out\System\VERSION.txt" /mnt/sdcard/System/VERSION.txt
    adb shell 'sync'
    Step "rebooting device"
    adb reboot
}

Step "done"
