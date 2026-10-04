<#
.SYNOPSIS
  Offline proof for build/package-release.ps1: the release pair, its refusals, and its publication.

.DESCRIPTION
  The release packager is the gate between the artifact CI uploaded and a draft release, so the
  checks a release run makes are exercised here as checks, not as a comment:

    * two runs into two fresh, absent output paths, each output directory holding exactly the zip and
      the sidecar, with the archive entry names and the extracted file hashes compared between them;
    * the first run extracted with ZipFile.ExtractToDirectory and compared with the card tree —
      root, file set, sizes and SHA-256 — plus a byte-level parse of the sidecar that does not use
      the packager's own reader;
    * every refusal: the release identity, the tree, and the pre-existing output path, each against a
      copy of the tree and with a full fingerprint of the directory it must not change taken before
      and after (an existing directory with a marker, an otherwise empty directory with a marker, a
      file, and a link);
    * the publication step itself: a staged two-file pair and a destination that appears after the
      staging directory was prepared, so the rename fails with a real collision and the owned staging
      directory is cleaned by production code;
    * a structural proof that production publication has exactly one directory move, no file move,
      and that the move comes after the validations;
    * the version stamp as bytes: the input tree has to carry the normalized stamp (UTF-8 with no
      mark, no CR, exactly three LF-terminated lines), the production validator is proved to refuse a
      mark, CRLF, a missing final LF and invalid UTF-8 with a focused diagnostic, and the local
      producer, the reusable workflow and the validator are shown to carry the same three lines;
    * a hand-built archive for each archive-validation rule — absolute, backslashed, relative,
      duplicated, case-colliding, unexpected, linked, wrong-length, wrong-bytes, missing entry and a
      wrong root — against a control archive the production writer produced, which has to pass.

  Nothing here touches the network, Docker, a device, or the card tree it is given: the tree is copied
  once and every mutation happens in the copy. Cargo is read offline by the Rust notice check, so the
  crate sources have to be in the local Cargo cache (cargo fetch --locked ...). build/package-release.ps1
  is dot-sourced, so the validators under test are the ones a release run uses rather than a second
  implementation of them, and the focused promotion test calls the production helper directly instead
  of a copy of it.

.PARAMETER CardTree
  The card tree to package. Relative paths are taken from the caller's directory. It is read only; the
  copy the tests mutate lives under -ScratchRoot.

.PARAMETER ScratchRoot
  Where the copies, outputs and bad archives go. It is created if it does not exist and it must not be
  inside the card tree. Defaults to a new directory under the temporary directory.

.EXAMPLE
  powershell -NoProfile -ExecutionPolicy Bypass -File build/test-package-release.ps1
#>
[CmdletBinding()]
# Named differently from the packager's own parameters on purpose: the packager is dot-sourced
# below, and a dot-sourced param block writes its parameter names into this scope, so a -CardRoot,
# -Tag, -Commit or -OutputDir here would be re-bound to the packager's own defaults.
param(
    [string]$CardTree = 'dist-device',
    [string]$ScratchRoot
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'package-release.ps1')

function Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }

$failures = New-Object System.Collections.ArrayList
$skipped = New-Object System.Collections.ArrayList

function Get-DirectoryFingerprint {
  <#
  .SYNOPSIS
    What a directory holds, as text: every entry's relative path, each file's size and SHA-256, and
    each link recorded as a link rather than followed.
  .DESCRIPTION
    The fingerprint a preservation check compares before and after a failed run. Links are not
    followed, so a link is one line whether or not its target exists, and nothing outside the
    directory can make the fingerprint move.
  #>
    param([Parameter(Mandatory)][string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) { return '<absent>' }
    $lines = New-Object System.Collections.ArrayList
    $pending = New-Object System.Collections.Stack
    $pending.Push([pscustomobject]@{ Dir = $Path; Rel = '' })
    while ($pending.Count -gt 0) {
        $frame = $pending.Pop()
        foreach ($item in @(Get-ChildItem -LiteralPath $frame.Dir -Force | Sort-Object -Property Name)) {
            $rel = if ($frame.Rel) { "$($frame.Rel)/$($item.Name)" } else { $item.Name }
            if ($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) {
                [void]$lines.Add("link $rel")
            } elseif ($item.PSIsContainer) {
                [void]$lines.Add("dir  $rel")
                $pending.Push([pscustomobject]@{ Dir = $item.FullName; Rel = $rel })
            } else {
                [void]$lines.Add("file $rel $($item.Length) $(Get-FileSha256 -Path $item.FullName)")
            }
        }
    }
    if ($lines.Count -eq 0) { return '<empty>' }
    return (($lines | Sort-Object) -join "`n")
}

function Get-StagingLeftover {
  <#
  .SYNOPSIS
    Every path under -Path whose own name is one of this packager's staging directories.
  .DESCRIPTION
    The staging directory is the only thing a failed run may create and remove, so a leftover is both
    a cleanup bug and evidence about which path a run got to. Links are not followed.
  #>
    param([Parameter(Mandatory)][string]$Path)

    $found = New-Object System.Collections.ArrayList
    $pending = New-Object System.Collections.Stack
    $pending.Push($Path)
    while ($pending.Count -gt 0) {
        $dir = $pending.Pop()
        foreach ($item in @(Get-ChildItem -LiteralPath $dir -Force)) {
            if ($item.Name -like '.slot2-release-*') { [void]$found.Add($item.FullName) }
            if ($item.PSIsContainer -and -not ($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint)) {
                $pending.Push($item.FullName)
            }
        }
    }
    return @($found | Sort-Object)
}

function Get-EntryNameList {
    param([Parameter(Mandatory)][string]$ZipPath)
    $zip = [System.IO.Compression.ZipFile]::OpenRead($ZipPath)
    try { return @($zip.Entries | ForEach-Object { $_.FullName }) } finally { $zip.Dispose() }
}

function Assert-SidecarBytes {
  <#
  .SYNOPSIS
    The sidecar as bytes: no mark, LF only, lowercase digest, two spaces, the zip's leaf name.
  .DESCRIPTION
    Parsed here rather than with the packager's reader, so the sidecar is proved by a second reading
    of the format and not by the code that wrote it.
  #>
    param([Parameter(Mandatory)][string]$SidecarPath, [Parameter(Mandatory)][string]$ZipPath)

    $bytes = [System.IO.File]::ReadAllBytes($SidecarPath)
    if ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) { throw 'the sidecar starts with a UTF-8 mark' }
    $text = [System.Text.Encoding]::ASCII.GetString($bytes)
    if ($text.Contains("`r")) { throw 'the sidecar is not LF-only' }
    if (-not $text.EndsWith("`n") -or $text.EndsWith("`n`n")) { throw 'the sidecar does not end in exactly one LF' }
    $leaf = [System.IO.Path]::GetFileName($ZipPath)
    $expected = "$(Get-FileSha256 -Path $ZipPath)  $leaf"
    if ($text -cne "$expected`n") { throw "the sidecar is not '$expected' and one LF" }
}

function Assert-NormalizedStampBytes {
  <#
  .SYNOPSIS
    The input card tree's System/VERSION.txt is the one normalized stamp, read as bytes here.
  .DESCRIPTION
    The byte contract the local producer and the reusable workflow both write: strict UTF-8 with no
    byte-order mark, no carriage return anywhere, and exactly three LF bytes with the last one closing
    the file, carrying the three semantic lines. Read with explicit byte checks rather than through
    the packager's validator, so the normal input tree is proved normalized by a second reader, and
    returned so the mutations below can start from the very bytes the producer wrote.
  #>
    param([Parameter(Mandatory)][string]$Path, [Parameter(Mandatory)][string]$Version)

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { throw "the card tree has no System/VERSION.txt at $Path" }
    [byte[]]$bytes = [System.IO.File]::ReadAllBytes($Path)
    if ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) { throw "$Path starts with a UTF-8 byte-order mark" }
    $cr = @($bytes | Where-Object { $_ -eq 0x0D }).Count
    if ($cr -ne 0) { throw "$Path holds $cr carriage return byte(s)" }
    $lf = @($bytes | Where-Object { $_ -eq 0x0A }).Count
    if ($lf -ne 3) { throw "$Path holds $lf LF byte(s), not three" }
    if ($bytes[$bytes.Length - 1] -ne 0x0A) { throw "$Path does not end with a LF byte" }
    $text = (New-Object System.Text.UTF8Encoding($false, $true)).GetString($bytes)
    $split = @($text -split "`n")
    $lines = @($split[0..($split.Count - 2)])
    if ($lines.Count -ne 3) { throw "$Path is $($lines.Count) line(s), not the three-line stamp" }
    if ($lines[0] -notmatch ('^SLOT2 ' + [regex]::Escape($Version) + ' \([0-9a-fA-F]{7,40}\)$')) {
        throw "$Path's first line is not 'SLOT2 $Version (<short commit>)': '$($lines[0])'"
    }
    $expected = @(
        'Copy the contents of this folder to the root of the card BaseOS boots a frontend from.',
        'BaseOS runs System/frontend. Logs: /tmp/frontend.log on the device, System/slot2-diag.txt on the card.'
    )
    for ($i = 0; $i -lt $expected.Count; $i++) {
        if ($lines[$i + 1] -cne $expected[$i]) { throw "$Path line $($i + 2) is not '$($expected[$i])': '$($lines[$i + 1])'" }
    }
    return ,$bytes
}

function Assert-OutputDirectoryHolds {
  <#
  .SYNOPSIS
    A successful run's output directory holds exactly the zip and the sidecar, and no staging left.
  #>
    param([Parameter(Mandatory)][string]$OutputDir, [Parameter(Mandatory)][string]$ZipLeaf)

    if (-not [System.IO.Directory]::Exists($OutputDir)) { throw "no output directory at $OutputDir" }
    $names = @(Get-ChildItem -LiteralPath $OutputDir -Force | ForEach-Object { $_.Name })
    $want = @($ZipLeaf, "$ZipLeaf.sha256")
    $extra = @($names | Where-Object { $want -cnotcontains $_ })
    $missing = @($want | Where-Object { $names -cnotcontains $_ })
    if ($extra.Count -gt 0 -or $missing.Count -gt 0) {
        throw "$OutputDir holds $($names.Count) item(s) ($($names -join ', ')); expected exactly $ZipLeaf and $ZipLeaf.sha256"
    }
}

function Test-Scenario {
  <#
  .SYNOPSIS
    One refusal: it has to fail, for the named reason, and it may not change the directory it names.
  #>
    param([Parameter(Mandatory)][string]$Name, [Parameter(Mandatory)][scriptblock]$Action,
          [Parameter(Mandatory)][string]$Expect, [string]$Preserve)

    $before = if ($Preserve) { Get-DirectoryFingerprint -Path $Preserve } else { $null }
    $observed = '<no failure>'
    try { & $Action } catch { $observed = $_.Exception.Message }

    $ok = ($observed -ne '<no failure>') -and ($observed -like "*$Expect*")
    $preserved = $true
    if ($Preserve -and $ok) { $preserved = ((Get-DirectoryFingerprint -Path $Preserve) -ceq $before) }
    if (-not $ok) {
        [void]$failures.Add("$Name`: expected '$Expect', observed: $observed")
    } elseif (-not $preserved) {
        [void]$failures.Add("$Name`: it changed $Preserve")
    }
    Write-Host ("[{0}] {1}{2}" -f $(if ($ok -and $preserved) { 'OK  ' } else { 'FAIL' }), $Name, $(if ($ok) { '' } else { "  <- $observed" }))
}

function Assert-PreservedDirectory {
    param([Parameter(Mandatory)][string]$Name, [Parameter(Mandatory)][string]$Path, [Parameter(Mandatory)][string]$Before)
    $after = Get-DirectoryFingerprint -Path $Path
    if ($after -cne $Before) {
        [void]$failures.Add("$Name`: $Path changed")
        Write-Host "[FAIL] $Name  <- $Path changed"
    } else {
        Write-Host ("[OK  ] {0}  -- {1} fingerprints equal ({2} line(s))" -f $Name, (Split-Path -Leaf $Path), $Before.Split("`n").Count)
    }
}

function Write-ZipEntry {
    param([Parameter(Mandatory)]$Archive, [Parameter(Mandatory)][string]$Name,
          [Parameter(Mandatory)][byte[]]$Bytes, [string]$UnixMode)

    $entry = $Archive.CreateEntry($Name, [System.IO.Compression.CompressionLevel]::NoCompression)
    if ($UnixMode) { $entry.ExternalAttributes = [int]([Convert]::ToInt32($UnixMode, 16) -shl 16) }
    $stream = $entry.Open()
    try { $stream.Write($Bytes, 0, $Bytes.Length) } finally { $stream.Dispose() }
}

function New-TestArchive {
    param([Parameter(Mandatory)][string]$Path, [Parameter(Mandatory)][scriptblock]$Build)

    $stream = [System.IO.File]::Open($Path, [System.IO.FileMode]::CreateNew, [System.IO.FileAccess]::Write)
    try {
        $archive = New-Object System.IO.Compression.ZipArchive($stream, [System.IO.Compression.ZipArchiveMode]::Create)
        try { & $Build $archive } finally { $archive.Dispose() }
    } finally { $stream.Dispose() }
}

# --- the scratch root ---------------------------------------------------------------------
$cardTree = if ([System.IO.Path]::IsPathRooted($CardTree)) { $CardTree } else { Join-Path (Get-Location).Path $CardTree }
$cardTree = [System.IO.Path]::GetFullPath($cardTree)
if (-not (Test-Path -LiteralPath $cardTree -PathType Container)) { throw "no card tree at $cardTree" }
$scratch = if ($ScratchRoot) { $ScratchRoot } else { Join-Path ([System.IO.Path]::GetTempPath()) ("slot2-release-test-" + [guid]::NewGuid().ToString('n')) }
if (-not [System.IO.Path]::IsPathRooted($scratch)) { $scratch = Join-Path (Get-Location).Path $scratch }
$scratch = [System.IO.Path]::GetFullPath($scratch)
$separator = [System.IO.Path]::DirectorySeparatorChar
if (($scratch + $separator).StartsWith($cardTree + $separator, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "the scratch directory $scratch is inside the card tree $cardTree"
}
New-Item -ItemType Directory -Force $scratch | Out-Null
Step "scratch: $scratch"

$version = Get-WorkspaceVersion -Root $root
$releaseCommit = (& git -C $root rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $releaseCommit -notmatch '^[0-9a-fA-F]{40}$') { throw "git did not name the commit of $root" }
$releaseTag = "v$version"
$leaf = "slot2-$releaseTag.zip"
Step "identity: tag $releaseTag, commit $releaseCommit, pair $leaf + $leaf.sha256"

# The tree under test is a copy, and every mutation below happens in a copy of that copy: the card
# tree the caller named is only ever read.
$tree = Join-Path $scratch 'tree'
Step "copying $cardTree to $tree"
Copy-Item -LiteralPath $cardTree -Destination $tree -Recurse -Force

# The input tree has to be the normalized artifact before anything is packaged from it: the packager
# now refuses a mark, a CR, invalid UTF-8 or a missing final LF, so a tree built by an older producer
# must fail here with the reason rather than halfway through the run. The bytes are kept for the
# negative cases below, which mutate exactly what the producer wrote.
$stampInTree = Join-Path $tree 'System\VERSION.txt'
[byte[]]$canonical = Assert-NormalizedStampBytes -Path $stampInTree -Version $version
Write-Host ("[OK  ] the input tree's System/VERSION.txt is the normalized stamp ({0} bytes, no mark, no CR, exactly three LF bytes, final byte LF)" -f $canonical.Length)

# --- two runs into absent output paths ----------------------------------------------------
$outA = Join-Path $scratch 'out-a'
$outB = Join-Path $scratch 'out-b'
foreach ($pair in @(@{ Out = $outA; Name = 'A' }, @{ Out = $outB; Name = 'B' })) {
    if (Test-Path -LiteralPath $pair.Out) { throw "the test expected $($pair.Out) to be absent before run $($pair.Name)" }
    Step "run $($pair.Name): packaging $tree into the absent $($pair.Out)"
    Invoke-ReleasePackaging -CardRoot $tree -Tag $releaseTag -Commit $releaseCommit -OutputDir $pair.Out
}
$zipA = Join-Path $outA $leaf
$zipB = Join-Path $outB $leaf
$sideA = "$zipA.sha256"
$sideB = "$zipB.sha256"

foreach ($pair in @(@{ Out = $outA; Name = 'A' }, @{ Out = $outB; Name = 'B' })) {
    try {
        Assert-OutputDirectoryHolds -OutputDir $pair.Out -ZipLeaf $leaf
        Write-Host "[OK  ] output $($pair.Name) holds exactly $leaf and $leaf.sha256"
    } catch {
        [void]$failures.Add("output $($pair.Name): $($_.Exception.Message)")
        Write-Host "[FAIL] output $($pair.Name)  <- $($_.Exception.Message)"
    }
}

# Owned staging directories are removed by the run that created them, promoted or not.
$leftovers = @(Get-StagingLeftover -Path $scratch)
if ($leftovers.Count -gt 0) {
    [void]$failures.Add("staging directories left behind: $($leftovers -join ', ')")
    Write-Host "[FAIL] no staging directory left behind  <- $($leftovers -join ', ')"
} else {
    Write-Host '[OK  ] no staging directory left behind after the runs'
}

Assert-SidecarBytes -SidecarPath $sideA -ZipPath $zipA
Assert-SidecarBytes -SidecarPath $sideB -ZipPath $zipB
Step "sidecars: '<sha256>  $leaf' and one LF, no mark"

# The zip as a consumer reads it: extracted, then compared with the tree file by file.
$extractA = Join-Path $scratch 'extract-a'
[System.IO.Compression.ZipFile]::ExtractToDirectory($zipA, $extractA)
$roots = @(Get-ChildItem -LiteralPath $extractA -Force | Select-Object -ExpandProperty Name)
if ($roots.Count -ne 1 -or $roots[0] -cne 'System') { throw "the extracted zip's only root is not System: $($roots -join ', ')" }
$extracted = Get-DirectoryFingerprint -Path $extractA
if ($extracted -cne (Get-DirectoryFingerprint -Path $tree)) { throw 'the extracted zip is not the card tree, file set, sizes and hashes' }
Step "extracted: root System, $(@(Get-ChildItem -LiteralPath $extractA -Recurse -File).Count) files, every size and hash equal to the tree"

$entriesA = @(Get-EntryNameList -ZipPath $zipA)
$entriesB = @(Get-EntryNameList -ZipPath $zipB)
if (($entriesA -join "`n") -cne ($entriesB -join "`n")) { throw 'the two runs do not hold the same archive entries' }
$extractB = Join-Path $scratch 'extract-b'
[System.IO.Compression.ZipFile]::ExtractToDirectory($zipB, $extractB)
if ((Get-DirectoryFingerprint -Path $extractB) -cne $extracted) { throw 'the two runs do not extract to the same files' }
$sameContainer = (Get-FileSha256 -Path $zipA) -eq (Get-FileSha256 -Path $zipB)
Step "two runs: $($entriesA.Count) archive entries each, same names, same extracted hashes (container bytes identical: $sameContainer)"

# --- the release identity, refused before the output path is even looked at ----------------
$prev = $outA
$marker = Join-Path $prev 'marker.txt'
Set-Content -LiteralPath $marker -Value 'previous output marker' -NoNewline
$prevFingerprint = Get-DirectoryFingerprint -Path $prev
Step "refusals: identity refusals point at $prev, which must survive; tree refusals get their own absent output path"

Test-Scenario -Name 'tag/version mismatch' -Preserve $prev -Expect "is not 'v$version'" -Action {
    Invoke-ReleasePackaging -CardRoot $tree -Tag 'v9.9.9' -Commit $releaseCommit -OutputDir $prev
}
Test-Scenario -Name 'malformed/traversal tag' -Preserve $prev -Expect "is not 'v$version'" -Action {
    Invoke-ReleasePackaging -CardRoot $tree -Tag 'v0.1.0/../../etc' -Commit $releaseCommit -OutputDir $prev
}
Test-Scenario -Name 'non-40-hex commit' -Preserve $prev -Expect '40 hexadecimal characters' -Action {
    Invoke-ReleasePackaging -CardRoot $tree -Tag $releaseTag -Commit 'abc123' -OutputDir $prev
}

# The tree refusals have to reach the tree checks, so their output path is absent - and it stays
# absent, together with its parent, which is where a staging directory would have shown up.
function New-AbsentCase {
    param([Parameter(Mandatory)][string]$Leaf)
    $case = Join-Path $scratch "case-$Leaf"
    if (Test-Path -LiteralPath $case) { Remove-Item -LiteralPath $case -Recurse -Force }
    New-Item -ItemType Directory -Force $case | Out-Null
    return $case
}

function New-MutatedTree {
    param([Parameter(Mandatory)][string]$Leaf, [Parameter(Mandatory)][scriptblock]$Mutate)
    $copy = Join-Path $scratch "neg-$Leaf"
    Copy-Item -LiteralPath $tree -Destination $copy -Recurse -Force
    & $Mutate $copy
    return $copy
}

$cases = @(
    @{ Name = 'VERSION hash mismatch'; Expect = 'not a prefix of the released commit'; Tree = (New-MutatedTree -Leaf 'version' -Mutate {
            param($copy)
            $text = "SLOT2 $version (deadbee)`n" +
                    "Copy the contents of this folder to the root of the card BaseOS boots a frontend from.`n" +
                    "BaseOS runs System/frontend. Logs: /tmp/frontend.log on the device, System/slot2-diag.txt on the card.`n"
            [System.IO.File]::WriteAllText((Join-Path $copy 'System/VERSION.txt'), $text, (New-Object System.Text.UTF8Encoding($false)))
        }) },
    @{ Name = 'missing core'; Expect = 'is missing'; Tree = (New-MutatedTree -Leaf 'nocore' -Mutate { param($copy) Remove-Item -LiteralPath (Join-Path $copy 'System/cores/mgba_libretro.so') -Force }) },
    @{ Name = 'extra top-level item'; Expect = 'exactly one System directory'; Tree = (New-MutatedTree -Leaf 'extra' -Mutate { param($copy) Set-Content -LiteralPath (Join-Path $copy 'README.txt') -Value 'not part of a card' }) },
    @{ Name = 'tampered Rust notice'; Expect = 'RUST-MANIFEST.txt'; Tree = (New-MutatedTree -Leaf 'rust' -Mutate { param($copy) [System.IO.File]::AppendAllText((Join-Path $copy 'System/licenses/rust/THIRD-PARTY-RUST.md'), "tampered`n") }) },
    # The stamp's byte contract, refused by the production validator. Every case starts from the bytes
    # the producer wrote and is mutated explicitly, so no PowerShell encoding default can construct it.
    @{ Name = 'VERSION stamp with a UTF-8 byte-order mark'; Expect = 'byte-order mark'; Tree = (New-MutatedTree -Leaf 'version-bom' -Mutate {
            param($copy)
            [byte[]]$bytes = [byte[]](0xEF, 0xBB, 0xBF) + $canonical
            [System.IO.File]::WriteAllBytes((Join-Path $copy 'System/VERSION.txt'), $bytes)
        }) },
    @{ Name = 'VERSION stamp with CRLF line endings'; Expect = 'carriage return'; Tree = (New-MutatedTree -Leaf 'version-crlf' -Mutate {
            param($copy)
            $crlf = New-Object System.Collections.Generic.List[byte]
            foreach ($b in $canonical) {
                if ($b -eq 0x0A) { $crlf.Add([byte]0x0D) }
                $crlf.Add($b)
            }
            [System.IO.File]::WriteAllBytes((Join-Path $copy 'System/VERSION.txt'), $crlf.ToArray())
        }) },
    @{ Name = 'VERSION stamp without the final LF'; Expect = 'does not end with a line feed'; Tree = (New-MutatedTree -Leaf 'version-nolf' -Mutate {
            param($copy)
            [byte[]]$bytes = $canonical[0..($canonical.Length - 2)]
            [System.IO.File]::WriteAllBytes((Join-Path $copy 'System/VERSION.txt'), $bytes)
        }) },
    @{ Name = 'VERSION stamp with invalid UTF-8 bytes'; Expect = 'is not valid UTF-8'; Tree = (New-MutatedTree -Leaf 'version-badutf8' -Mutate {
            param($copy)
            [byte[]]$bytes = $canonical[0..($canonical.Length - 2)] + [byte[]](0xFF, 0xFE, 0x0A)
            [System.IO.File]::WriteAllBytes((Join-Path $copy 'System/VERSION.txt'), $bytes)
        }) }
)
foreach ($case in $cases) {
    $parent = New-AbsentCase -Leaf ($case.Name -replace '[^A-Za-z0-9]', '-')
    $caseDir = Join-Path $parent 'out'
    $before = Get-DirectoryFingerprint -Path $parent
    $observed = '<no failure>'
    try { Invoke-ReleasePackaging -CardRoot $case.Tree -Tag $releaseTag -Commit $releaseCommit -OutputDir $caseDir } catch { $observed = $_.Exception.Message }
    $ok = ($observed -ne '<no failure>') -and ($observed -like "*$($case.Expect)*")
    if (-not $ok) {
        [void]$failures.Add("$($case.Name): expected '$($case.Expect)', observed: $observed")
        Write-Host ("[FAIL] {0}  <- {1}" -f $case.Name, $observed)
    } else {
        Write-Host ("[OK  ] {0}  -- {1}" -f $case.Name, $observed)
        Assert-PreservedDirectory -Name "  $($case.Name) left its parent untouched" -Path $parent -Before $before
    }
}

# --- the output path has to be absent, and must be left alone if it is not -----------------
# Each case gets its own parent so the fingerprint covers the destination and everything beside it,
# which is where a staging directory would appear before publication.
$occupied = @(
    @{ Name = 'pre-existing output directory with a marker'; Prepare = {
            param($out)
            New-Item -ItemType Directory -Force $out | Out-Null
            Set-Content -LiteralPath (Join-Path $out 'marker.txt') -Value 'previous output marker' -NoNewline
            Set-Content -LiteralPath (Join-Path $out 'stray.txt') -Value 'not a release file' -NoNewline
        } },
    @{ Name = 'pre-existing output directory, otherwise empty plus marker'; Prepare = {
            param($out)
            New-Item -ItemType Directory -Force $out | Out-Null
            Set-Content -LiteralPath (Join-Path $out 'marker.txt') -Value 'only the marker' -NoNewline
        } },
    @{ Name = 'pre-existing output path that is a file'; Prepare = {
            param($out)
            Set-Content -LiteralPath $out -Value 'a file, not a directory' -NoNewline
        } },
    @{ Name = 'pre-existing output path that is a link'; Prepare = $null }
)
foreach ($case in $occupied) {
    $parent = New-AbsentCase -Leaf ($case.Name -replace '[^A-Za-z0-9]', '-')
    $caseDir = Join-Path $parent 'out'
    if ($case.Name -like '* link') {
        $target = Join-Path $parent 'link-target'
        New-Item -ItemType Directory -Force $target | Out-Null
        $created = cmd /c mklink /J "$caseDir" "$target" 2>&1
        if (-not (Test-Path -LiteralPath $caseDir)) {
            [void]$skipped.Add("$($case.Name): mklink said '$($created -join ' ')'")
            Write-Host "[SKIP] $($case.Name)  <- this host cannot create a directory link: $($created -join ' ')"
            continue
        }
    } else {
        & $case.Prepare $caseDir
    }

    $before = Get-DirectoryFingerprint -Path $parent
    $observed = '<no failure>'
    try { Invoke-ReleasePackaging -CardRoot $tree -Tag $releaseTag -Commit $releaseCommit -OutputDir $caseDir } catch { $observed = $_.Exception.Message }
    $ok = ($observed -ne '<no failure>') -and ($observed -like '*already exists*')
    if (-not $ok) {
        [void]$failures.Add("$($case.Name): expected 'already exists', observed: $observed")
        Write-Host ("[FAIL] {0}  <- {1}" -f $case.Name, $observed)
    } else {
        Write-Host ("[OK  ] {0}  -- {1}" -f $case.Name, $observed)
        Assert-PreservedDirectory -Name "  $($case.Name) untouched" -Path $parent -Before $before
        Write-Host ("       fingerprint: {0}" -f ($before -replace "`n", ' | '))
    }
}

# --- the publication step itself, with a collision that appears after staging -------------
# The production helper is called directly, with a staged two-file pair and a destination that already
# holds a marker. The rename has to fail, the destination (marker included) has to stay as it was, the
# owned staging directory has to be gone, and no zip or sidecar may appear at the destination.
$tiny = Join-Path $scratch 'tiny'
New-Item -ItemType Directory -Force (Join-Path $tiny 'System') | Out-Null
$alpha = [System.Text.Encoding]::UTF8.GetBytes("alpha`n")
$beta = [byte[]](0..9)
[System.IO.File]::WriteAllBytes((Join-Path $tiny 'System/a.txt'), $alpha)
[System.IO.File]::WriteAllBytes((Join-Path $tiny 'System/b.bin'), $beta)
$tinyFiles = @(Get-CardTreeFile -CardRoot $tiny)

$staging = Join-Path $scratch 'owned-staging'
$lateParent = New-AbsentCase -Leaf 'late-destination'
$late = Join-Path $lateParent 'out'
New-Item -ItemType Directory -Force $staging | Out-Null
New-ReleaseArchive -ZipPath (Join-Path $staging $leaf) -Files $tinyFiles
$stagedDigest = Get-FileSha256 -Path (Join-Path $staging $leaf)
[System.IO.File]::WriteAllText((Join-Path $staging "$leaf.sha256"), "$stagedDigest  $leaf`n", (New-Object System.Text.UTF8Encoding($false)))
[void](Assert-ReleaseSidecar -SidecarPath (Join-Path $staging "$leaf.sha256") -ZipPath (Join-Path $staging $leaf))
New-Item -ItemType Directory -Force $late | Out-Null
Set-Content -LiteralPath (Join-Path $late 'marker.txt') -Value 'late marker' -NoNewline
$stagedBefore = @(Get-ChildItem -LiteralPath $staging -Force).Name
$lateBefore = Get-DirectoryFingerprint -Path $lateParent
$observed = '<no failure>'
try { [void](Publish-ReleasePair -Staging $staging -OutputDir $late) } catch { $observed = $_.Exception.Message }
$lateAfter = Get-DirectoryFingerprint -Path $lateParent
$stagingGone = -not (Test-Path -LiteralPath $staging)
$pairAtDestination = @(
    (Test-Path -LiteralPath (Join-Path $late $leaf)),
    (Test-Path -LiteralPath (Join-Path $late "$leaf.sha256")))
if ($observed -eq '<no failure>') {
    [void]$failures.Add('promotion collision: the rename was accepted over an existing directory')
    Write-Host '[FAIL] promotion collision  <- the rename was accepted over an existing directory'
} elseif (($lateAfter -cne $lateBefore)) {
    [void]$failures.Add('promotion collision: the destination changed')
    Write-Host '[FAIL] promotion collision  <- the destination changed'
} elseif (-not $stagingGone) {
    [void]$failures.Add('promotion collision: the owned staging directory was not cleaned')
    Write-Host '[FAIL] promotion collision  <- the owned staging directory was not cleaned'
} elseif (($pairAtDestination -contains $true)) {
    [void]$failures.Add('promotion collision: the pair appeared at the destination')
    Write-Host '[FAIL] promotion collision  <- the pair appeared at the destination'
} else {
    Write-Host ("[OK  ] promotion collision  -- staged pair [{0}] refused: {1}" -f ($stagedBefore -join ', '), $observed)
    Assert-PreservedDirectory -Name '  promotion collision left the destination untouched' -Path $lateParent -Before $lateBefore
    Write-Host ("       fingerprint: {0}" -f ($lateBefore -replace "`n", ' | '))
    Write-Host '       owned staging directory cleaned: True; zip or sidecar at the destination: 0'
}

# The previous output of run A, after every failing run above: all three files unchanged.
if (-not (Test-Path -LiteralPath $marker)) { [void]$failures.Add('the marker file is gone') }
if ((Get-Content -LiteralPath $marker -Raw) -cne 'previous output marker') { [void]$failures.Add('the marker file changed') }
Assert-SidecarBytes -SidecarPath $sideA -ZipPath $zipA
Step 'previous output intact: marker, zip and sidecar unchanged'

# --- production publication is one directory move and no file move ------------------------
$source = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'package-release.ps1') -Raw
$lines = $source -split '\r?\n'
$moveLines = @(0..($lines.Count - 1) | Where-Object { $lines[$_] -match '\[System\.IO\.Directory\]::Move' })
$helperStart = @(0..($lines.Count - 1) | Where-Object { $lines[$_] -match '^function Publish-ReleasePair' })[0]
$callerStart = @(0..($lines.Count - 1) | Where-Object { $lines[$_] -match '^function Invoke-ReleasePackaging' })[0]
$cleanupLines = @($lines | Where-Object { $_ -match 'Remove-TreeBestEffort -Path' })
$checks = @(
    @{ Name = 'exactly one directory move in production code'; Ok = ($moveLines.Count -eq 1) },
    @{ Name = 'no [System.IO.File]::Move anywhere'; Ok = (-not ($source -match '\[System\.IO\.File\]::Move')) },
    @{ Name = 'no Move-Item, Copy-Item or Rename-Item anywhere'; Ok = (-not ($source -cmatch '\bMove-Item|\bCopy-Item|\bRename-Item')) },
    @{ Name = 'the move lives in the publication helper, not in the caller'; Ok = ($moveLines[0] -gt $helperStart -and $moveLines[0] -lt $callerStart) },
    @{ Name = 'the helper is called once, from the packaging run'; Ok = (($lines | Where-Object { $_ -match '^\s*\$promoted = Publish-ReleasePair ' }).Count -eq 1) },
    @{ Name = 'cleanup of the staging directory is guarded by the promotion flag'; Ok = (($lines | Where-Object { $_ -match 'if \(-not \$promoted\) \{ Remove-TreeBestEffort -Path \$staging' }).Count -eq 1) },
    @{ Name = 'every cleanup call names a staging path and nothing else'; Ok = ($cleanupLines.Count -eq 3 -and @($cleanupLines | Where-Object { $_ -match '\$staging' -or $_ -match '\$Staging' }).Count -eq 3) },
    @{ Name = 'no cleanup call can name the output path or a released file'; Ok = (-not ($source -cmatch 'Remove-TreeBestEffort[^\r\n]*(\$out\b|\$OutputDir|\$finalZip|\$finalSidecar)')) }
)
foreach ($check in $checks) {
    if ($check.Ok) { Write-Host "[OK  ] $($check.Name)" } else { [void]$failures.Add($check.Name); Write-Host "[FAIL] $($check.Name)" }
}
$validationLines = @(0..($lines.Count - 1) | Where-Object { $lines[$_] -match 'Assert-ReleaseArchive -ZipPath \$stagedZip|Assert-ReleasePairDirectory -Path \$staging' })
$callLine = @(0..($lines.Count - 1) | Where-Object { $lines[$_] -match '^\s*\$promoted = Publish-ReleasePair ' })[0]
if ($validationLines.Count -eq 2 -and ($validationLines | ForEach-Object { $_ -lt $callLine }) -notcontains $false) {
    Write-Host "[OK  ] the publication call comes after the zip, sidecar and staging-directory checks (lines $($validationLines -join ', ') < $callLine)"
} else {
    [void]$failures.Add('the publication call does not come after the staging checks')
    Write-Host '[FAIL] the publication call does not come after the staging checks'
}

# --- one stamp byte contract across both producers and the validator ----------------------
# The local build and the tag workflow have to write the same bytes, and the release gate has to name
# that one contract: a mark, a CR, invalid UTF-8 or a missing final LF is a producer difference that
# must not be read away at release time. The workflow is read here and not modified.
$producer = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'dist-device.ps1') -Raw
$workflow = Get-Content -LiteralPath (Join-Path $root '.github/workflows/device-artifact.yml') -Raw
$stampLiterals = @(
    'Copy the contents of this folder to the root of the card BaseOS boots a frontend from.',
    'BaseOS runs System/frontend. Logs: /tmp/frontend.log on the device, System/slot2-diag.txt on the card.'
)
$crProbe = '$' + "'" + '\' + 'r' + "'"
$stampChecks = @(
    @{ Name = 'the local producer writes VERSION.txt with explicit BOM-free UTF-8'; Ok = ($producer.Contains('[System.IO.File]::WriteAllText($stampPath') -and $producer.Contains('New-Object System.Text.UTF8Encoding($false)')) },
    @{ Name = 'the local producer joins its stamp lines with LF and ends the file with one LF'; Ok = ($producer.Contains('$stampLines -join "`n"') -and $producer.Contains('+ "`n"')) },
    @{ Name = 'the local producer writes VERSION.txt through no Out-File or Set-Content command'; Ok = (-not ($producer -cmatch 'Out-File|Set-Content')) },
    @{ Name = 'the local producer carries the same three stamp lines'; Ok = (($stampLiterals | ForEach-Object { $producer.Contains($_) }) -notcontains $false) },
    @{ Name = 'the reusable workflow writes the same three stamp lines with printf'; Ok = ($workflow.Contains('SLOT2 %s (%s)') -and (($stampLiterals | ForEach-Object { $workflow.Contains($_) }) -notcontains $false)) },
    @{ Name = 'the reusable workflow gate checks the mark, the CR, the three lines and their text'; Ok = ($workflow.Contains('efbbbf') -and $workflow.Contains($crProbe) -and $workflow.Contains('is not three lines') -and $workflow.Contains('does not carry the card-copy instruction') -and $workflow.Contains('does not name the frontend and the diagnostic paths')) },
    @{ Name = 'the validator names the same byte contract both producers write'; Ok = ($source.Contains('byte-order mark') -and $source.Contains('carriage return byte') -and $source.Contains('does not end with a line feed') -and $source.Contains('is not valid UTF-8') -and (($stampLiterals | ForEach-Object { $source.Contains($_) }) -notcontains $false)) }
)
foreach ($check in $stampChecks) {
    if ($check.Ok) { Write-Host "[OK  ] $($check.Name)" } else { [void]$failures.Add($check.Name); Write-Host "[FAIL] $($check.Name)" }
}

# --- the archive validator, on archives it did not write ----------------------------------
$badRoot = Join-Path $scratch 'bad'
New-Item -ItemType Directory -Force $badRoot | Out-Null
$control = Join-Path $badRoot 'control.zip'
New-ReleaseArchive -ZipPath $control -Files $tinyFiles
try {
    Assert-ReleaseArchive -ZipPath $control -Files $tinyFiles
    Write-Host '[OK  ] archive control: the writer and the validator agree on a good archive'
} catch {
    [void]$failures.Add("archive control: the validator refused the production writer's archive: $($_.Exception.Message)")
    Write-Host "[FAIL] archive control  <- $($_.Exception.Message)"
}

$mutations = @(
    @{ File = 'absolute';   Name = 'archive: absolute entry';  Expect = 'absolute path';                Build = { param($a) Write-ZipEntry -Archive $a -Name '/System/a.txt' -Bytes $alpha; Write-ZipEntry -Archive $a -Name 'System/b.bin' -Bytes $beta } },
    @{ File = 'backslash';  Name = 'archive: backslash entry'; Expect = 'forward-slashed';              Build = { param($a) Write-ZipEntry -Archive $a -Name 'System\a.txt' -Bytes $alpha; Write-ZipEntry -Archive $a -Name 'System/b.bin' -Bytes $beta } },
    @{ File = 'dotdot';     Name = 'archive: .. segment';      Expect = 'relative path segment';        Build = { param($a) Write-ZipEntry -Archive $a -Name 'System/../a.txt' -Bytes $alpha; Write-ZipEntry -Archive $a -Name 'System/b.bin' -Bytes $beta } },
    @{ File = 'dot';        Name = 'archive: . segment';       Expect = 'relative path segment';        Build = { param($a) Write-ZipEntry -Archive $a -Name 'System/./a.txt' -Bytes $alpha; Write-ZipEntry -Archive $a -Name 'System/b.bin' -Bytes $beta } },
    @{ File = 'duplicate';  Name = 'archive: duplicate entry'; Expect = 'twice';                        Build = { param($a) Write-ZipEntry -Archive $a -Name 'System/a.txt' -Bytes $alpha; Write-ZipEntry -Archive $a -Name 'System/a.txt' -Bytes $alpha; Write-ZipEntry -Archive $a -Name 'System/b.bin' -Bytes $beta } },
    @{ File = 'case';       Name = 'archive: case collision';  Expect = 'twice';                        Build = { param($a) Write-ZipEntry -Archive $a -Name 'System/a.txt' -Bytes $alpha; Write-ZipEntry -Archive $a -Name 'System/A.txt' -Bytes $alpha; Write-ZipEntry -Archive $a -Name 'System/b.bin' -Bytes $beta } },
    @{ File = 'root';       Name = 'archive: wrong root';      Expect = 'not under System/';            Build = { param($a) Write-ZipEntry -Archive $a -Name 'Other/a.txt' -Bytes $alpha; Write-ZipEntry -Archive $a -Name 'System/b.bin' -Bytes $beta } },
    @{ File = 'unexpected'; Name = 'archive: unexpected file'; Expect = 'which the card tree does not'; Build = { param($a) Write-ZipEntry -Archive $a -Name 'System/a.txt' -Bytes $alpha; Write-ZipEntry -Archive $a -Name 'System/b.bin' -Bytes $beta; Write-ZipEntry -Archive $a -Name 'System/c.txt' -Bytes $alpha } },
    @{ File = 'link';       Name = 'archive: symbolic link';   Expect = 'symbolic link';                Build = { param($a) Write-ZipEntry -Archive $a -Name 'System/a.txt' -Bytes $alpha -UnixMode 'A1FF'; Write-ZipEntry -Archive $a -Name 'System/b.bin' -Bytes $beta } },
    @{ File = 'length';     Name = 'archive: wrong length';    Expect = 'bytes and the tree';           Build = { param($a) Write-ZipEntry -Archive $a -Name 'System/a.txt' -Bytes ([System.Text.Encoding]::UTF8.GetBytes("alpha`n`n")); Write-ZipEntry -Archive $a -Name 'System/b.bin' -Bytes $beta } },
    @{ File = 'bytes';      Name = 'archive: wrong bytes';     Expect = 'not the bytes';                Build = { param($a) Write-ZipEntry -Archive $a -Name 'System/a.txt' -Bytes ([System.Text.Encoding]::UTF8.GetBytes("alpHa`n")); Write-ZipEntry -Archive $a -Name 'System/b.bin' -Bytes $beta } },
    @{ File = 'missing';    Name = 'archive: missing entry';   Expect = 'missing what the tree holds';  Build = { param($a) Write-ZipEntry -Archive $a -Name 'System/b.bin' -Bytes $beta } }
)
foreach ($mutation in $mutations) {
    $zipPath = Join-Path $badRoot "$($mutation.File).zip"
    $build = $mutation.Build
    Test-Scenario -Name $mutation.Name -Expect $mutation.Expect -Action {
        New-TestArchive -Path $zipPath -Build $build
        Assert-ReleaseArchive -ZipPath $zipPath -Files $tinyFiles
    }
}

# --- the whole scratch, after everything ---------------------------------------------------
$leftovers = @(Get-StagingLeftover -Path $scratch)
if ($leftovers.Count -gt 0) {
    [void]$failures.Add("staging directories left behind at the end: $($leftovers -join ', ')")
    Write-Host "[FAIL] no staging directory left at the end  <- $($leftovers -join ', ')"
} else {
    Write-Host '[OK  ] no staging directory left behind anywhere in the scratch'
}
Assert-PreservedDirectory -Name 'the previous output survived every failing run' -Path $prev -Before $prevFingerprint
Assert-OutputDirectoryHolds -OutputDir $outB -ZipLeaf $leaf
Write-Host '[OK  ] the untouched output directory still holds exactly the zip and the sidecar'

Step "scratch kept for inspection: $scratch"
foreach ($note in $skipped) { Write-Host "SKIPPED: $note" -ForegroundColor Yellow }
if ($failures.Count -gt 0) {
    Write-Host ''
    foreach ($failure in $failures) { Write-Host "FAIL: $failure" -ForegroundColor Red }
    throw "$($failures.Count) check(s) failed"
}
Step 'all checks passed'
