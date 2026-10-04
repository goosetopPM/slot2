<#
.SYNOPSIS
  Write the third-party Rust notices of the device runtime as one deterministic, offline bundle.

.DESCRIPTION
  The bundle is the corresponding notices for the Rust packages a card's binary links, and it is
  what System/licenses/rust holds:

    THIRD-PARTY-RUST.md   the inventory, package by package
    RUST-SBOM.json        slot2-rust-sbom-v1, this project's own deterministic inventory
    RUST-MANIFEST.txt     SHA-256 and byte length of every other file in the bundle
    packages/<key>/       PACKAGE.txt and the package's original license/notice files

  The package set comes from Cargo — the normal dependency closure of `slot2` for
  aarch64-unknown-linux-gnu with default features off and feature `device` on — and never from a
  list of crate names kept in step by hand. Everything is read from the local Cargo cache:
  `--offline` is on, and a package that is not cached fails the run by name instead of being
  fetched.

  The bundle is built and checked in a staging directory beside the output and only then renamed
  into place, so a run that cannot finish leaves the previous bundle exactly where it was.

.PARAMETER OutputDir
  Where the bundle goes. Relative paths are taken from the caller's directory.

.EXAMPLE
  .\build\package-rust-notices.ps1 -OutputDir target/rust-notices
#>
param(
    [Parameter(Mandatory)][string]$OutputDir
)

$ErrorActionPreference = 'Stop'
$caller = (Get-Location).Path
if (-not [System.IO.Path]::IsPathRooted($OutputDir)) { $OutputDir = Join-Path $caller $OutputDir }
$OutputDir = [System.IO.Path]::GetFullPath($OutputDir)

$root = Split-Path -Parent $PSScriptRoot
Set-Location $root
. (Join-Path $PSScriptRoot 'rust-notices.ps1')

function Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }

function Remove-TreeBestEffort([string]$Path, [string]$What) {
    if (-not (Test-Path -LiteralPath $Path)) { return }
    try { Remove-Item -LiteralPath $Path -Recurse -Force -ErrorAction Stop }
    catch { Write-Warning "$What could not be removed and is still at $Path ($($_.Exception.Message))" }
}

# Replacing a directory is not something to be talked into by an argument that names the
# repository or its parent.
if ($OutputDir -eq $root) { throw "refusing to replace the repository itself ($OutputDir)" }
if ($OutputDir -eq [System.IO.Path]::GetPathRoot($OutputDir)) { throw "refusing to replace a filesystem root ($OutputDir)" }
if (($root + [System.IO.Path]::DirectorySeparatorChar).StartsWith($OutputDir + [System.IO.Path]::DirectorySeparatorChar)) {
    throw "refusing to replace $OutputDir, which contains the repository"
}
try {
    $outputAttributes = [System.IO.File]::GetAttributes($OutputDir)
} catch [System.IO.IOException] {
    $outputAttributes = $null
}
if ($null -ne $outputAttributes -and
    (-not ($outputAttributes -band [System.IO.FileAttributes]::Directory) -or
        ($outputAttributes -band [System.IO.FileAttributes]::ReparsePoint))) {
    throw "refusing to replace $OutputDir, which exists but is not a normal directory"
}

# --- discovery: nothing is written until Cargo has named every package and every notice ----
Step "reading the device dependency closure from Cargo (offline)"
$scope = Get-RustNoticeScope -Root $root
Step ("{0} third-party packages, Cargo.lock {1}" -f $scope.Packages.Count, $scope.LockSha256.Substring(0, 12))

# --- staging: the bundle is written beside the output, so the final rename stays in one
#     filesystem, and it is checked where it stands before it is allowed to replace anything --
$parent = Split-Path -Parent $OutputDir
New-Item -ItemType Directory -Force $parent | Out-Null
$staging = Join-Path $parent ("." + (Split-Path -Leaf $OutputDir) + ".staging-" + [guid]::NewGuid().ToString('n'))
$backup = Join-Path $parent ("." + (Split-Path -Leaf $OutputDir) + ".backup-" + [guid]::NewGuid().ToString('n'))

# The staging directory's GUID path belongs to this run from the moment the write can create it,
# and a copy that fails half way through leaves part of one behind, so ownership is read from the
# file system at cleanup time rather than from a flag a failed write would not have set.
$backupHolds = $false      # the previous bundle is standing aside under $backup
$promoted = $false         # the staging directory took the output's name
$settled = $false          # the promoted bundle passed its own check, so the backup may go
$preserve = $false         # the rollback failed; backup and the rejected output must stay

try {
    New-Item -ItemType Directory -Force $staging | Out-Null
    Write-RustNoticeBundle -Scope $scope -Bundle $staging
    Assert-RustNoticeBundle -Root $root -Bundle $staging
    Step "bundle checked ($($scope.Packages.Count) packages)"

    $replaced = Test-Path -LiteralPath $OutputDir
    if ($replaced) {
        [System.IO.Directory]::Move($OutputDir, $backup)
        $backupHolds = $true
    }
    try {
        [System.IO.Directory]::Move($staging, $OutputDir)
        $promoted = $true
        # The rename moved the bytes; it did not make them a bundle, so the promoted directory is
        # checked before the replacement is called complete.
        Assert-RustNoticeBundle -Root $root -Bundle $OutputDir
    } catch {
        $promotionError = $_
        if (-not $backupHolds) { throw $promotionError }
        try {
            if ($promoted) {
                [System.IO.Directory]::Move($OutputDir, $staging)
                $promoted = $false
            }
            [System.IO.Directory]::Move($backup, $OutputDir)
            $backupHolds = $false
        } catch {
            $rollbackError = $_
            $preserve = $true
            $kept = @()
            if (Test-Path -LiteralPath $staging) { $kept += "the rejected bundle at $staging" }
            if (Test-Path -LiteralPath $OutputDir) { $kept += "the rejected output at $OutputDir" }
            $keptText = if ($kept.Count -eq 0) { 'no directory needed to be kept' } else { 'kept: ' + ($kept -join '; ') }
            throw ("replacing $OutputDir failed: $($promotionError.Exception.Message)" + [Environment]::NewLine +
                   "restoring the previous bundle failed: $($rollbackError.Exception.Message)" + [Environment]::NewLine +
                   "the last known-good bundle is kept at $backup" + [Environment]::NewLine +
                   $keptText)
        }
        throw $promotionError
    }

    $settled = $true
    $files = @(Get-ChildItem -LiteralPath $OutputDir -Recurse -File)
    $bytes = ($files | Measure-Object -Property Length -Sum).Sum
    Step ("wrote {0} files ({1} bytes) to {2}" -f $files.Count, $bytes, $OutputDir)
} finally {
    if (-not $preserve) {
        Remove-TreeBestEffort -Path $staging -What 'the rejected staging bundle'
        if ($settled -and $backupHolds) { Remove-TreeBestEffort -Path $backup -What 'the previous bundle' }
    }
}

# An explicit status, not an inherited one: the callers (build/dist-device.ps1 and CI) invoke this
# script with `&`, where `$LASTEXITCODE` would otherwise still hold whatever native command ran
# last in the caller. Reaching this line means the bundle was built, checked and put in place.
exit 0
