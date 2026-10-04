<#
.SYNOPSIS
  Pack the pinned core sources, their licenses and SLOT2's build recipe into one directory.

.DESCRIPTION
  The bundle this writes is the corresponding source for the cores a card ships, and it is what
  System/licenses/sources holds:

    SOURCE-MANIFEST.txt          pins, repository URLs, archive hashes, patch hashes
    archives/<name>-<pin>.zip    `git archive` of the pinned commit, pristine
    licenses/<name>/<file>       the pinned top-level license, from the git object
    recipes/common.sh            SLOT2's shared build script, byte for byte
    recipes/<name>/              its commit, build.sh and *.patch

  Nothing is fetched: the checkouts build/cores.ps1 already made are used as they are, and every
  byte that ends up here comes out of a git object rather than out of a working tree, which for
  a patched core is dirty on purpose.

  Everything is built in a staging directory and the output directory is replaced only once the
  staged bundle has passed its own checks, so a failure leaves the previous bundle where it was.

.PARAMETER OutputDir
  Where the bundle goes. Relative paths are taken from the caller's directory.

.EXAMPLE
  .\build\package-core-sources.ps1 -OutputDir target/core-sources
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
. (Join-Path $PSScriptRoot 'core-manifest.ps1')

function Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }

# Deleting a directory is not atomic: an open handle inside it can refuse the whole delete.
# Losing the last known-good bundle would be worse than leaving a dotted scratch directory, so
# every removal here is best effort and names the exact path it could not clear.
function Remove-TreeBestEffort([string]$Path, [string]$What) {
    if (-not (Test-Path -LiteralPath $Path)) { return }
    try {
        Remove-Item -LiteralPath $Path -Recurse -Force -ErrorAction Stop
    } catch {
        Write-Warning "$What could not be removed and is still at $Path ($($_.Exception.Message))"
    }
}

# Replacing a directory is not something to be talked into by an argument that names the
# repository or its parent.
if ($OutputDir -eq $root) { throw "refusing to replace the repository itself ($OutputDir)" }
if ($OutputDir -eq [System.IO.Path]::GetPathRoot($OutputDir)) { throw "refusing to replace a filesystem root ($OutputDir)" }
if (($root + [System.IO.Path]::DirectorySeparatorChar).StartsWith($OutputDir + [System.IO.Path]::DirectorySeparatorChar)) {
    throw "refusing to replace $OutputDir, which contains the repository"
}

# The output has to be a directory this script may replace. A file, or a link of any kind,
# standing under that name is refused here, before anything is staged: it is not a bundle to
# move aside and the rename that would replace it is not a rename the script may perform.
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

$names = Get-CoreManifest -Root $root

# --- preflight: every input, before anything is written ---------------------------------
$plan = @()
foreach ($name in $names) {
    $pin = Get-CorePin -Root $root -Name $name
    $checkout = Get-CoreCheckout -Root $root -Name $name
    & git -C $checkout cat-file -e "$pin^{commit}"
    if ($LASTEXITCODE -ne 0) { throw "$name's pin $pin is not in $checkout — run build/cores.ps1 -DeviceOnly" }
    $head = (& git -C $checkout rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0) { throw "git rev-parse failed in $checkout" }
    if ($head -cne $pin) { throw "$name is checked out at $head, not at its pin $pin" }

    $license = Get-CoreLicenseName -Name $name
    $licensePath = Join-Path $root "licenses/cores/$name/$license"
    if (-not (Test-Path -LiteralPath $licensePath -PathType Leaf)) { throw "no tracked license at $licensePath" }
    if ((Get-Item -LiteralPath $licensePath).Length -eq 0) { throw "$licensePath is empty" }

    $repo = Get-CoreRepoUrl -Root $root -Name $name
    $meta = Join-Path $root "vendor/${name}_libretro.so.meta"
    if (-not (Test-Path -LiteralPath $meta -PathType Leaf)) { throw "no stamp at $meta — run build/cores.ps1 -DeviceOnly" }
    $stamp = Get-Content -LiteralPath $meta -Raw
    if (-not ($stamp.Contains("repo=$repo") -or $stamp.Contains("source=$repo/tree/$pin"))) {
        throw "$meta does not name the repository $repo the build script uses"
    }
    if (-not $stamp.Contains("commit=$pin")) { throw "$meta does not name the pin $pin" }

    $recipe = Join-Path $root "cores/$name"
    foreach ($file in @('commit', 'build.sh')) {
        $path = Join-Path $recipe $file
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "no $file at $path" }
        if ((Get-Item -LiteralPath $path).Length -eq 0) { throw "$path is empty" }
    }
    $patches = @(Get-ChildItem -LiteralPath $recipe -Filter '*.patch' -File | Sort-Object Name)
    foreach ($patch in $patches) {
        if ($patch.Length -eq 0) { throw "$($patch.FullName) is empty" }
    }

    $plan += [pscustomobject]@{
        Name = $name; Pin = $pin; Checkout = $checkout; Repo = $repo
        License = $license; LicensePath = $licensePath; Patches = $patches
    }
}

# --- staging: the bundle is built somewhere else and moved in when it is complete ---------
$target = Join-Path $root 'target'
New-Item -ItemType Directory -Force $target | Out-Null
$stage = Join-Path $target ("package-core-sources-" + [guid]::NewGuid().ToString('n'))
New-Item -ItemType Directory -Force $stage | Out-Null
New-Item -ItemType Directory -Force (Join-Path $stage 'archives') | Out-Null
New-Item -ItemType Directory -Force (Join-Path $stage 'licenses') | Out-Null
New-Item -ItemType Directory -Force (Join-Path $stage 'recipes') | Out-Null

try {
    $manifestLines = @()
    foreach ($core in $plan) {
        $name = $core.Name
        $archiveRel = "archives/$($name)-$($core.Pin).zip"
        $archive = Join-Path $stage $archiveRel
        Step "$name archive ($archiveRel)"
        Export-GitArchive -Checkout $core.Checkout -Pin $core.Pin `
            -Prefix "$($name)-$($core.Pin)/" -Destination $archive
        $archiveHash = Get-FileSha256 -Path $archive

        # The license comes out of the pinned object here too: the working tree of a patched
        # core is not what the archive holds, and a checked-out copy on Windows may have had
        # every line ending rewritten.
        $pinned = Get-PinnedFileBytes -Checkout $core.Checkout -Pin $core.Pin -Path $core.License
        Assert-BytesAreTheSame -What "$($core.LicensePath) against $($core.Pin):$($core.License)" `
            -A ([System.IO.File]::ReadAllBytes($core.LicensePath)) -B $pinned
        $licenseDir = Join-Path $stage "licenses/$name"
        New-Item -ItemType Directory -Force $licenseDir | Out-Null
        [System.IO.File]::WriteAllBytes((Join-Path $licenseDir $core.License), $pinned)

        $recipeDir = Join-Path $stage "recipes/$name"
        New-Item -ItemType Directory -Force $recipeDir | Out-Null
        foreach ($file in @('commit', 'build.sh')) {
            Copy-Item -LiteralPath (Join-Path $root "cores/$name/$file") -Destination $recipeDir
        }
        foreach ($patch in $core.Patches) {
            Copy-Item -LiteralPath $patch.FullName -Destination $recipeDir
        }

        $manifestLines += "core=$name"
        $manifestLines += "repo=$($core.Repo)"
        $manifestLines += "pin=$($core.Pin)"
        $manifestLines += "archive=$archiveRel"
        $manifestLines += "archive_sha256=$archiveHash"
        $manifestLines += "license=licenses/$name/$($core.License)"
        $manifestLines += "recipe=recipes/$name"
        foreach ($patch in $core.Patches) {
            $hash = Get-FileSha256 -Path (Join-Path $recipeDir $patch.Name)
            $manifestLines += "patch=$($patch.Name) sha256:$hash"
        }
        $manifestLines += ''
        Step ("  {0} bytes, license {1} ({2} bytes), {3} patch(es)" -f `
            (Get-Item -LiteralPath $archive).Length, $core.License, $pinned.Length, $core.Patches.Count)
    }

    Copy-Item -LiteralPath (Join-Path $root 'cores/common.sh') -Destination (Join-Path $stage 'recipes')

    # No timestamps, no machine names, no absolute paths: two runs from the same checkout have
    # to produce the same manifest, so the card's copy and a release's copy can be compared.
    $manifestText = ($manifestLines -join "`n") + "`n"
    [System.IO.File]::WriteAllText((Join-Path $stage 'SOURCE-MANIFEST.txt'), $manifestText,
        (New-Object System.Text.UTF8Encoding($false)))

    # --- the staged bundle checks itself before it is allowed to replace anything ----------
    Assert-SourceBundle -Root $root -Bundle $stage -Names $names
    Step "bundle checked ($($plan.Count) cores)"

    # The replacement is a rename, never a copy over the old bundle: the candidate is built
    # beside the output (same parent, so the rename stays inside one filesystem even when the
    # output is somewhere else entirely), it is checked again where it stands, the old bundle is
    # moved aside, and only then does the candidate take its name.
    #
    # Every step below is tracked as state rather than as a cleanup list, because deleting a
    # directory is not a rename: someone else's open handle inside it can refuse the delete part
    # way through. The states encode one priority — the last known-good bundle outranks a tidy
    # directory. The backup goes only once `$settled` says the output holds a validated bundle,
    # and when a rollback fails `$preserve` leaves the backup and the rejected output untouched.
    #
    # The moves are `[System.IO.Directory]::Move`, which is one rename at the operating system's
    # level. `Move-Item` is not: with a file held open inside the directory it sets about moving
    # the directory child by child and stops half way, so a run that failed while standing the old
    # bundle aside would have left the bundle scattered between two names — the failure this
    # whole arrangement is here to refuse.
    $parent = Split-Path -Parent $OutputDir
    New-Item -ItemType Directory -Force $parent | Out-Null
    $candidate = Join-Path $parent ("." + (Split-Path -Leaf $OutputDir) + ".candidate-" + [guid]::NewGuid().ToString('n'))
    $backup = Join-Path $parent ("." + (Split-Path -Leaf $OutputDir) + ".backup-" + [guid]::NewGuid().ToString('n'))

    # The candidate's GUID names a directory only this run can own, and the recursive copy below can
    # create it and then fail half way through it, so ownership is read from the file system at
    # cleanup time rather than from a flag that a copy which never returned would not have set.
    $backupHolds = $false      # the old bundle is standing aside under $backup
    $promoted = $false         # the candidate rename succeeded and $OutputDir holds the new bundle
    $settled = $false          # the new bundle is in place and validated, so the backup may go
    $preserve = $false         # the rollback failed; the backup and the rejected output must stay

    try {
        Copy-Item -LiteralPath $stage -Destination $candidate -Recurse -Force
        Assert-SourceBundle -Root $root -Bundle $candidate -Names $names
        $files = @(Get-ChildItem -LiteralPath $candidate -Recurse -File)
        $bytes = ($files | Measure-Object -Property Length -Sum).Sum

        $replaced = Test-Path -LiteralPath $OutputDir
        if ($replaced) {
            # The old bundle is not touched until the candidate has been copied and checked. If
            # even this rename fails the output never left its name, so the original error stands
            # and the candidate is only scratch.
            [System.IO.Directory]::Move($OutputDir, $backup)
            $backupHolds = $true
        }

        try {
            [System.IO.Directory]::Move($candidate, $OutputDir)
            $promoted = $true
            # The rename proves the bytes moved, not that they are a bundle, so the promoted
            # directory is checked as well before the replacement is called complete.
            Assert-SourceBundle -Root $root -Bundle $OutputDir -Names $names
        } catch {
            $promotionError = $_
            if (-not $backupHolds) {
                # No earlier bundle was standing aside, so there is nothing to restore; the
                # pre-run state, no output at all, is what the cleanup below returns to.
                throw $promotionError
            }
            # Put the old bundle back. A rejected output still under the output's name is renamed
            # to the candidate's free name first, so neither tree has to be deleted.
            try {
                if ($promoted) {
                    [System.IO.Directory]::Move($OutputDir, $candidate)
                    $promoted = $false
                }
                [System.IO.Directory]::Move($backup, $OutputDir)
                $backupHolds = $false
            } catch {
                $rollbackError = $_
                $preserve = $true
                $kept = @()
                if (Test-Path -LiteralPath $candidate) { $kept += "the candidate at $candidate" }
                if (Test-Path -LiteralPath $OutputDir) { $kept += "the rejected output at $OutputDir" }
                $keptText = if ($kept.Count -eq 0) { 'no directory needed to be kept' } else { 'kept: ' + ($kept -join '; ') }
                throw ("replacing $OutputDir failed: $($promotionError.Exception.Message)" + [Environment]::NewLine +
                       "restoring the previous bundle failed: $($rollbackError.Exception.Message)" + [Environment]::NewLine +
                       "the last known-good bundle is kept at $backup" + [Environment]::NewLine +
                       $keptText)
            }
            throw $promotionError
        }

        # Promotion and its re-check both passed; only now is the old bundle disposable.
        $settled = $true
        Step ("wrote {0} files ({1} bytes) to {2}" -f $files.Count, $bytes, $OutputDir)
    } finally {
        # Cleanup follows the states above. $preserve means the rollback could not put the old
        # bundle back, so the backup and the rejected output are the only copies left and are
        # deliberately untouched. The candidate is the run's own directory whenever it exists —
        # including a partial one left by a copy that failed — so it is removed on existence and
        # not on a state flag. Every removal is best effort: a directory someone else holds open
        # cannot be deleted, and its exact path is reported instead of being hidden.
        if (-not $preserve) {
            Remove-TreeBestEffort -Path $candidate -What 'the rejected candidate'
            if (-not $settled -and $promoted -and -not $backupHolds) {
                Remove-TreeBestEffort -Path $OutputDir -What 'the rejected output'
            }
            if ($settled -and $backupHolds) { Remove-TreeBestEffort -Path $backup -What 'the previous bundle' }
        }
    }
} finally {
    Remove-Item -LiteralPath $stage -Recurse -Force -ErrorAction SilentlyContinue
}
