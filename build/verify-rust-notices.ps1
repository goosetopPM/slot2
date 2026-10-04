<#
.SYNOPSIS
  Check an assembled Rust notice tree — `System/licenses/rust` — against what Cargo resolves now.

.DESCRIPTION
  The postflight entry point for the card tree, for the CI assembly step to run where the packager
  itself is a separate process. It is the same check the packager and build/dist-device.ps1 make:
  the exact package set, every package's `notice_status` and evidence files, every copied byte,
  every SBOM field and every RUST-MANIFEST.txt hash. Cargo runs offline against the local cache.

.PARAMETER Bundle
  The assembled tree to check. Relative paths are taken from the caller's directory.

.EXAMPLE
  pwsh -NoProfile -File build/verify-rust-notices.ps1 -Bundle dist-device/System/licenses/rust
#>
param(
    [Parameter(Mandatory)][string]$Bundle
)

$ErrorActionPreference = 'Stop'
if (-not [System.IO.Path]::IsPathRooted($Bundle)) { $Bundle = Join-Path (Get-Location).Path $Bundle }
$Bundle = [System.IO.Path]::GetFullPath($Bundle)

$root = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'rust-notices.ps1')

Assert-RustNoticeBundle -Root $root -Bundle $Bundle
Write-Host "==> rust notice bundle checked: $Bundle"
