<#
.SYNOPSIS
  Package a downloaded card tree into the release zip and its SHA-256 sidecar.

.DESCRIPTION
  The release gate: it takes the card tree a tag run downloaded (the artifact the reusable
  device workflow uploaded, which is the contents of dist-device) and writes

    <OutputDir>/slot2-<Tag>.zip          the card root: System/... and nothing else
    <OutputDir>/slot2-<Tag>.zip.sha256   the digest, two spaces and the zip's leaf name, LF

  Nothing about the tree is taken on trust. The release identity is checked first: the tag must
  be `v` + the workspace version in Cargo.toml, the commit must be 40 hex characters, and
  System/VERSION.txt must stamp the same version and a short hash that is a prefix of that
  commit. Then the tree is put through the same repository contracts the assembly used —
  build/core-manifest.ps1 and build/rust-notices.ps1 — as the tree that is about to be shipped
  rather than as the bundle it came from: exactly the manifest's six cores and their stamps,
  the licenses and the source bundle with every archive, recipe and patch hash, the exact Rust
  notice closure with every copied byte, the frontend, the fonts and the version stamp.

  The zip is built and validated in a staging directory beside the output path, with the sidecar
  beside it: the zip is reopened and checked entry by entry - names, root, file set, uncompressed
  lengths and SHA-256 against the tree it was cut from, with absolute paths, backslashes, relative
  segments, duplicate or case-colliding entries, links and files the tree does not hold all refused
  - and the sidecar is read back as bytes. The validated pair is then published with one same-volume
  directory rename onto the output path, so the zip and its sidecar appear together and there is no
  moment at which the output holds one without the other. OutputDir therefore has to be absent: an
  existing directory, file or link is refused before anything is staged and is left byte for byte as
  it was. No failure path copies, merges, replaces, backs up, retries or falls back. Cleanup removes
  only a staging directory this run created and did not promote; a published release directory is
  never deleted here, and a failure after the rename names it for manual inspection.

.PARAMETER CardRoot
  The downloaded card tree: a directory whose only entry is System. Relative paths are taken
  from the caller's directory.

.PARAMETER Tag
  The release tag, exactly `v` + the workspace version in Cargo.toml (a Cargo prerelease is
  allowed; build metadata is not).

.PARAMETER Commit
  The full 40-hexadecimal commit of the tagged revision the tree was built from.

.PARAMETER OutputDir
  Where the zip and its sidecar appear. It must not exist yet: the pair is published by renaming its
  staging directory onto this path, so an existing directory, file or link is refused before staging
  instead of being replaced or merged into. Relative paths are taken from the caller's directory.

.EXAMPLE
  pwsh -NoProfile -File build/package-release.ps1 -CardRoot card-tree -Tag v0.1.0 \
    -Commit 1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d -OutputDir /tmp/slot2-release

.NOTES
  Cargo is read offline, for the Rust notice closure, so the crate sources have to be in the
  local Cargo cache: run `cargo fetch --locked --target aarch64-unknown-linux-gnu` first.
#>
[CmdletBinding()]
param(
    [string]$CardRoot,
    [string]$Tag,
    [string]$Commit,
    [string]$OutputDir
)

$ErrorActionPreference = 'Stop'

# Read at the top because dot-sourcing this file is how build/test-package-release.ps1 calls the
# validators below: it defines the functions and does not package anything, so the negative tests
# exercise the very checks a release run makes instead of a copy of them.
$dotSourced = $MyInvocation.InvocationName -eq '.'

. (Join-Path $PSScriptRoot 'core-manifest.ps1')
. (Join-Path $PSScriptRoot 'rust-notices.ps1')

function Step($msg) { Write-Host "==> $msg" -ForegroundColor Cyan }

# Deleting a directory is not atomic: an open handle inside it can refuse the whole delete.
# Losing an already published release file would be worse than leaving a dotted scratch
# directory, so removal here is best effort and names the exact path it could not clear.
function Remove-TreeBestEffort([string]$Path, [string]$What) {
    if (-not (Test-Path -LiteralPath $Path)) { return }
    try {
        Remove-Item -LiteralPath $Path -Recurse -Force -ErrorAction Stop
    } catch {
        Write-Warning "$What could not be removed and is still at $Path ($($_.Exception.Message))"
    }
}

function Get-WorkspaceVersion {
  <#
  .SYNOPSIS
    The workspace package version, read from the [workspace.package] section of Cargo.toml.
  .DESCRIPTION
    Section-scoped rather than "the first version = in the file": a member crate's version or a
    dependency's must never be mistaken for the one the tag has to name.
  #>
    param([Parameter(Mandatory)][string]$Root)

    $path = Join-Path $Root 'Cargo.toml'
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "no Cargo.toml at $path" }
    # Read as LF: the repository stores LF and a Windows checkout hands back CRLF, and the two
    # have to parse to the same version.
    $text = ([System.IO.File]::ReadAllText($path)) -replace "`r`n", "`n"
    $section = [regex]::Match($text, '(?ms)^\[workspace\.package\][ \t]*$(?<body>.*?)(^\[|\z)')
    if (-not $section.Success) { throw "$path has no [workspace.package] section" }
    $match = [regex]::Match($section.Groups['body'].Value, '(?m)^version[ \t]*=[ \t]*"(?<version>[^"]+)"[ \t]*$')
    if (-not $match.Success) { throw "$path's [workspace.package] has no version" }
    return $match.Groups['version'].Value
}

function Resolve-PathArgument {
  <#
  .SYNOPSIS
    A caller-supplied path as an absolute path, with the characters a path may not carry refused.
  .DESCRIPTION
    Relative paths come from the caller's directory, as the rest of build/ does. A wildcard, a
    quote or a control character is refused rather than passed to the file system, because every
    path in this script is used literally and a path that needs escaping is a path this script
    would have to mis-read to accept.
  #>
    param([Parameter(Mandatory)][string]$Path, [Parameter(Mandatory)][string]$What)

    if ([string]::IsNullOrWhiteSpace($Path)) { throw "no $What was given" }
    foreach ($char in [System.IO.Path]::GetInvalidFileNameChars()) {
        if ($char -eq ':' -or $char -eq '\' -or $char -eq '/') { continue }
        if ($Path.IndexOf($char) -ge 0) { throw "the $What '$Path' contains '$char'" }
    }
    if (-not [System.IO.Path]::IsPathRooted($Path)) { $Path = Join-Path (Get-Location).Path $Path }
    return [System.IO.Path]::GetFullPath($Path)
}

function Assert-NormalDirectory {
  <#
  .SYNOPSIS
    The path exists and is a directory this script may use as one.
  .DESCRIPTION
    A file, or a link of any kind, standing under a name that has to be a directory is refused.
    A reparse point is not a directory to write beside: writing through it would put the release
    somewhere other than the path the caller named.
  #>
    param([Parameter(Mandatory)][string]$Path, [Parameter(Mandatory)][string]$What)

    $attributes = $null
    try { $attributes = [System.IO.File]::GetAttributes($Path) } catch [System.IO.IOException] { $attributes = $null }
    if ($null -eq $attributes) { throw "no $What at $Path" }
    if (-not ($attributes -band [System.IO.FileAttributes]::Directory)) { throw "$Path is not a directory, so it is not a $What" }
    if ($attributes -band [System.IO.FileAttributes]::ReparsePoint) { throw "$Path is a link, not a $What" }
}

function Assert-ExistingAncestorIsDirectory {
  <#
  .SYNOPSIS
    The nearest existing ancestor of a path that does not exist yet is a real directory.
  .DESCRIPTION
    The output directory may be created, so it may not exist; what must not exist is a file or a
    link in the way of creating it. The nearest existing ancestor is where that shows.
  #>
    param([Parameter(Mandatory)][string]$Path, [Parameter(Mandatory)][string]$What)

    $probe = $Path
    while ($true) {
        $parent = Split-Path -Parent $probe
        if (-not $parent -or $parent -eq $probe) { return }
        if (Test-Path -LiteralPath $parent) { Assert-NormalDirectory -Path $parent -What $What; return }
        $probe = $parent
    }
}

function Get-CardTreeFile {
  <#
  .SYNOPSIS
    Every file under the card root, as a relative forward-slashed path, sorted by that path.
  .DESCRIPTION
    The list the zip is cut from and checked against. The walk is explicit rather than
    Get-ChildItem -Recurse because a reparse point must be refused, not followed: a link is not a
    file the card holds, and following one would either duplicate a tree or leave the card root.
  #>
    param([Parameter(Mandatory)][string]$CardRoot)

    $files = New-Object System.Collections.ArrayList
    $pending = New-Object System.Collections.Stack
    $pending.Push([pscustomobject]@{ Dir = $CardRoot; Rel = '' })
    while ($pending.Count -gt 0) {
        $frame = $pending.Pop()
        foreach ($item in @(Get-ChildItem -LiteralPath $frame.Dir -Force)) {
            if ($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) {
                throw "$($item.FullName) is a link; a card tree is files, not links"
            }
            $rel = if ($frame.Rel) { "$($frame.Rel)/$($item.Name)" } else { $item.Name }
            if ($item.PSIsContainer) {
                $pending.Push([pscustomobject]@{ Dir = $item.FullName; Rel = $rel })
            } else {
                [void]$files.Add([pscustomobject]@{ Rel = $rel; Full = $item.FullName; Length = $item.Length })
            }
        }
    }
    return @($files | Sort-Object -Property Rel)
}

function Assert-VersionStamp {
  <#
  .SYNOPSIS
    System/VERSION.txt is the stamp of the version and the commit being released.
  .DESCRIPTION
    One byte contract, the one the reusable workflow writes and checks: UTF-8 with no
    byte-order mark, no carriage return anywhere, and exactly three LF-terminated lines
    including the final LF. build/dist-device.ps1 writes those bytes explicitly
    (UTF8Encoding($false) plus one LF per line), so the local card tree and the hosted
    artifact are the same file down to the byte, and a stamp that carries a mark, a CR,
    invalid UTF-8 or no final LF is refused here rather than shipped. The short hash has to
    be a prefix of the full commit with at least seven characters, which is where git's own
    abbreviation starts, so the stamp cannot name a different revision.
  #>
    param([Parameter(Mandatory)][string]$SystemDir, [Parameter(Mandatory)][string]$Version,
          [Parameter(Mandatory)][string]$Commit)

    $file = Join-Path $SystemDir 'VERSION.txt'
    if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "the card tree has no System/VERSION.txt" }
    $bytes = [System.IO.File]::ReadAllBytes($file)
    if ($bytes.Length -eq 0) { throw "System/VERSION.txt is empty" }

    # Byte level first: a mark or a CR is a producer difference, not a text difference, so it is
    # named as one instead of being normalized away.
    if ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) {
        throw "System/VERSION.txt starts with a UTF-8 byte-order mark; the stamp is UTF-8 with no mark"
    }
    $crOffsets = @(for ($i = 0; $i -lt $bytes.Length; $i++) { if ($bytes[$i] -eq 0x0D) { $i } })
    if ($crOffsets.Count -gt 0) {
        $shown = @($crOffsets | Select-Object -First 5) -join ', '
        throw "System/VERSION.txt holds $($crOffsets.Count) carriage return byte(s), first at offset(s) $shown; every line ends with one LF byte and nothing else"
    }
    if ($bytes[$bytes.Length - 1] -ne 0x0A) {
        throw "System/VERSION.txt does not end with a line feed byte; the stamp is three LF-terminated lines including the last one"
    }

    try {
        $text = (New-Object System.Text.UTF8Encoding($false, $true)).GetString($bytes)
    } catch {
        throw "System/VERSION.txt is not valid UTF-8: $($_.Exception.Message)"
    }
    $split = @($text -split "`n")
    $lines = @($split[0..($split.Count - 2)])
    if ($lines.Count -ne 3) {
        throw "System/VERSION.txt is not the three-line stamp build/dist-device.ps1 writes (it has $($lines.Count) line(s))"
    }

    $match = [regex]::Match($lines[0], '^SLOT2 (?<version>\S+) \((?<hash>[0-9a-fA-F]{7,40})\)$')
    if (-not $match.Success) { throw "System/VERSION.txt's first line is not 'SLOT2 <version> (<short commit>)': '$($lines[0])'" }
    $stamped = $match.Groups['version'].Value
    if ($stamped -cne $Version) {
        throw "System/VERSION.txt stamps version $stamped, not the $Version the tag names"
    }
    $hash = $match.Groups['hash'].Value
    if (-not $Commit.ToLowerInvariant().StartsWith($hash.ToLowerInvariant())) {
        throw "System/VERSION.txt stamps commit $hash, which is not a prefix of the released commit $Commit"
    }

    $fields = @(
        @{ Line = 1; Text = 'Copy the contents of this folder to the root of the card BaseOS boots a frontend from.'; What = 'the card-copy instruction' },
        @{ Line = 2; Text = 'BaseOS runs System/frontend. Logs: /tmp/frontend.log on the device, System/slot2-diag.txt on the card.'; What = 'the frontend and diagnostic paths' }
    )
    foreach ($field in $fields) {
        if ($lines[$field.Line] -cne $field.Text) {
            throw "System/VERSION.txt does not carry $($field.What): '$($lines[$field.Line])'"
        }
    }
}

function Assert-CardTree {
  <#
  .SYNOPSIS
    The downloaded card tree is a complete, contracted card and nothing else.
  .DESCRIPTION
    The repository's own contracts do the work — exactly the manifest's cores and stamps
    (Assert-CoreTree), the licenses and the source bundle with every hash (Assert-LicensesTree)
    and the exact Rust notice closure (Assert-RustNoticeBundle) — and this wraps them with the
    parts that are about the tree as a card: one top-level System directory, the frontend, the
    fonts the repository ships, and the version stamp.

    The Rust closure is re-derived from Cargo's own resolution, so this reads Cargo offline; a
    tree whose bundle no longer matches what the tagged revision resolves is refused here.
  #>
    param([Parameter(Mandatory)][string]$Root, [Parameter(Mandatory)][string]$CardRoot,
          [Parameter(Mandatory)][string]$Version, [Parameter(Mandatory)][string]$Commit,
          [Parameter(Mandatory)][string[]]$Names)

    $top = @(Get-ChildItem -LiteralPath $CardRoot -Force)
    if ($top.Count -ne 1 -or $top[0].Name -cne 'System' -or -not $top[0].PSIsContainer) {
        $found = @($top | ForEach-Object { $_.Name }) -join ', '
        throw "a card tree's root is exactly one System directory; $CardRoot holds $($top.Count) item(s): $found"
    }
    if ($top[0].Attributes -band [System.IO.FileAttributes]::ReparsePoint) { throw "$CardRoot/System is a link" }
    $system = Join-Path $CardRoot 'System'

    $frontend = Join-Path $system 'frontend'
    if (-not (Test-Path -LiteralPath $frontend -PathType Leaf)) { throw "the card tree has no System/frontend" }
    if ((Get-Item -LiteralPath $frontend).Length -eq 0) { throw "System/frontend is empty" }

    Assert-VersionStamp -SystemDir $system -Version $Version -Commit $Commit

    # The fonts the repository ships are the ones a card carries: every asset font, with its OFL
    # text, and no other name standing in for one of them.
    $assets = @(Get-ChildItem -LiteralPath (Join-Path $Root 'assets/fonts') -File)
    $wantFonts = @($assets | Where-Object { $_.Extension -in @('.otf', '.ttf') } | ForEach-Object { $_.Name } | Sort-Object)
    $wantTexts = @($assets | Where-Object { $_.Extension -eq '.txt' } | ForEach-Object { $_.Name } | Sort-Object)
    if ($wantFonts.Count -eq 0 -or $wantTexts.Count -eq 0) { throw 'assets/fonts holds no fonts or no license texts' }
    Assert-SameNameSet -What 'System/Fonts' -Want $wantFonts -Have (Get-ChildFileNames -Path (Join-Path $system 'Fonts'))
    Assert-SameNameSet -What 'System/licenses/fonts' -Want $wantTexts -Have (Get-ChildFileNames -Path (Join-Path $system 'licenses/fonts'))
    foreach ($name in $wantFonts) {
        if ((Get-Item -LiteralPath (Join-Path $system "Fonts/$name")).Length -eq 0) { throw "System/Fonts/$name is empty" }
    }
    foreach ($name in $wantTexts) {
        if ((Get-Item -LiteralPath (Join-Path $system "licenses/fonts/$name")).Length -eq 0) { throw "System/licenses/fonts/$name is empty" }
    }

    Assert-CoreTree -CoresDir (Join-Path $system 'cores') -LicensesDir (Join-Path $system 'licenses') -Names $Names
    Assert-LicensesTree -LicensesDir (Join-Path $system 'licenses') -Names $Names
    Assert-RustNoticeBundle -Root $Root -Bundle (Join-Path $system 'licenses/rust')
}

function New-ReleaseArchive {
  <#
  .SYNOPSIS
    Write the card tree into a zip whose entry names are the tree's own relative paths.
  .DESCRIPTION
    Through ZipArchive rather than Compress-Archive so the entry names are the caller's, exactly:
    Compress-Archive decides the root of the archive itself, and the root here has to be System.
    CreateNew means a name that already exists is refused rather than replaced.
  #>
    param([Parameter(Mandatory)][string]$ZipPath, [Parameter(Mandatory)][object[]]$Files)

    $stream = [System.IO.File]::Open($ZipPath, [System.IO.FileMode]::CreateNew, [System.IO.FileAccess]::Write)
    try {
        $archive = New-Object System.IO.Compression.ZipArchive($stream, [System.IO.Compression.ZipArchiveMode]::Create)
        try {
            foreach ($file in $Files) {
                [void][System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile(
                    $archive, $file.Full, $file.Rel, [System.IO.Compression.CompressionLevel]::Optimal)
            }
        } finally { $archive.Dispose() }
    } finally { $stream.Dispose() }
}

function Assert-ReleaseArchive {
  <#
  .SYNOPSIS
    The zip just written is the card tree, entry for entry and byte for byte.
  .DESCRIPTION
    Opened again from disk and read as a consumer would: every entry name has to be a relative,
    forward-slashed path under System/, no two entries may collide even by case, nothing may be
    a link, every entry has to be a file the tree holds at exactly that uncompressed length and
    hash, and every file the tree holds has to be in the archive exactly once. This is the check
    a mutated archive fails, which is why it reads the file rather than the object that wrote it.
  #>
    param([Parameter(Mandatory)][string]$ZipPath, [Parameter(Mandatory)][object[]]$Files)

    $expected = @{}
    foreach ($file in $Files) { $expected[$file.Rel] = $file }
    $seen = @{}
    $zip = [System.IO.Compression.ZipFile]::OpenRead($ZipPath)
    try {
        foreach ($entry in $zip.Entries) {
            $name = $entry.FullName
            if ([string]::IsNullOrEmpty($name)) { throw "the zip holds an entry with no name" }
            if ($name.Contains('\')) { throw "the zip entry '$name' is not forward-slashed" }
            if ($name.StartsWith('/') -or $name -match '^[A-Za-z]:') { throw "the zip entry '$name' is an absolute path" }
            $parts = $name.Split('/')
            foreach ($part in $parts) {
                if ($part -eq '' -or $part -eq '.' -or $part -eq '..') { throw "the zip entry '$name' has an empty or relative path segment" }
            }
            if ($parts[0] -cne 'System') { throw "the zip entry '$name' is not under System/" }
            if ((($entry.ExternalAttributes -shr 16) -band 0xF000) -eq 0xA000) { throw "the zip entry '$name' is a symbolic link" }
            $key = $name.ToLowerInvariant()
            if ($seen.ContainsKey($key)) { throw "the zip holds '$name' twice, or twice over when case is ignored" }
            $seen[$key] = $true
            if (-not $expected.ContainsKey($name)) { throw "the zip holds '$name', which the card tree does not" }
            $source = $expected[$name]
            if ($entry.Length -ne $source.Length) {
                throw "the zip's '$name' is $($entry.Length) bytes and the tree's is $($source.Length)"
            }
            $stream = $entry.Open()
            try {
                $sha = [System.Security.Cryptography.SHA256]::Create()
                try { $actual = ([System.BitConverter]::ToString($sha.ComputeHash($stream)) -replace '-', '').ToLowerInvariant() }
                finally { $sha.Dispose() }
            } finally { $stream.Dispose() }
            $want = Get-FileSha256 -Path $source.Full
            if ($actual -cne $want) { throw "the zip's '$name' is not the bytes the tree holds ($actual against $want)" }
        }
    } finally { $zip.Dispose() }

    $missing = @($expected.Keys | Where-Object { -not $seen.ContainsKey($_.ToLowerInvariant()) })
    if ($missing.Count -gt 0) { throw "the zip is missing what the tree holds: $(($missing | Sort-Object) -join ', ')" }
}

function Assert-ReleaseSidecar {
  <#
  .SYNOPSIS
    The sidecar is exactly "<sha256>  <zip leaf name>" and one LF, in UTF-8 without a mark.
  .DESCRIPTION
    Compared as bytes against the text it has to be, so a byte-order mark, a carriage return, an
    uppercase digest, a single space or a wrong file name are all the same kind of failure: the
    file is not what `sha256sum -c` would accept.
  .OUTPUTS
    The zip's SHA-256 as read from disk, which is the digest the sidecar has to carry.
  #>
    param([Parameter(Mandatory)][string]$SidecarPath, [Parameter(Mandatory)][string]$ZipPath)

    if (-not (Test-Path -LiteralPath $SidecarPath -PathType Leaf)) { throw "no sidecar at $SidecarPath" }
    $digest = Get-FileSha256 -Path $ZipPath
    $want = $digest + '  ' + [System.IO.Path]::GetFileName($ZipPath) + "`n"
    $wantBytes = (New-Object System.Text.UTF8Encoding($false)).GetBytes($want)
    $bytes = [System.IO.File]::ReadAllBytes($SidecarPath)
    try {
        Assert-BytesAreTheSame -What "the sidecar against '$($want.TrimEnd())'" -A $bytes -B $wantBytes
    } catch {
        throw "$SidecarPath is not '<sha256>  $([System.IO.Path]::GetFileName($ZipPath))' and one LF: $($_.Exception.Message)"
    }
    return $digest
}

function Test-PathTaken {
  <#
  .SYNOPSIS
    The path name is taken by something - a directory, a file, or a link.
  .DESCRIPTION
    Test-Path follows a link, so a link whose target is gone reads as absent while its name is still
    taken; GetAttributes reports a reparse point without dereferencing it, which is what closes that
    gap on Windows. The output path has to be absent as a name, not just as a target.
  #>
    param([Parameter(Mandatory)][string]$Path)

    if (Test-Path -LiteralPath $Path) { return $true }
    try { [void][System.IO.File]::GetAttributes($Path); return $true } catch [System.IO.IOException] { return $false }
}

function Assert-ReleasePairDirectory {
  <#
  .SYNOPSIS
    A directory holds exactly the release zip and its sidecar, and nothing else.
  .DESCRIPTION
    Checked on the staging directory before the rename, so nothing else can be published by accident,
    and again on the output directory after it, so the published directory is exactly the pair. The
    names are compared case-sensitively: `slot2-<tag>.zip` and `slot2-<tag>.zip.sha256` are the pair,
    and a case variant is another item, not that name.
  #>
    param([Parameter(Mandatory)][string]$Path, [Parameter(Mandatory)][string]$ZipLeaf,
          [Parameter(Mandatory)][string]$What)

    if (-not [System.IO.Directory]::Exists($Path)) { throw "no $What at $Path" }
    $names = @(Get-ChildItem -LiteralPath $Path -Force | ForEach-Object { $_.Name })
    $want = @($ZipLeaf, "$ZipLeaf.sha256")
    $extra = @($names | Where-Object { $want -cnotcontains $_ })
    $missing = @($want | Where-Object { $names -cnotcontains $_ })
    if ($extra.Count -gt 0 -or $missing.Count -gt 0) {
        throw "$What must hold exactly '$ZipLeaf' and '$ZipLeaf.sha256'; it holds $($names.Count) item(s): $($names -join ', ')"
    }
}

function Publish-ReleasePair {
  <#
  .SYNOPSIS
    Publish the validated zip and sidecar as one unit: one same-volume rename of their directory.
  .DESCRIPTION
    The pair is published by renaming the directory that holds it, so there is no moment at which the
    output holds the zip without its sidecar - the partial release a two-file promotion could leave
    behind. This is the only publication step: the destination has to be absent, and nothing here
    copies, merges, replaces, backs up or retries.

    The caller owns the staging directory until this returns, and owns the output directory after it.
    On any failure this function removes the staging directory it was handed - which is still
    unpromoted by definition - and reports the destination path it refused to touch. The absence of
    the destination is read again immediately before the rename, because POSIX rename replaces an
    empty destination directory while MoveFile refuses any existing one; the re-read is what makes
    the refusal hold on both platforms, and a non-empty destination is refused by the rename itself.
  .OUTPUTS
    The output directory path, so the caller can record that its staging directory was promoted.
  #>
    param([Parameter(Mandatory)][string]$Staging, [Parameter(Mandatory)][string]$OutputDir)

    if (Test-PathTaken -Path $OutputDir) {
        Remove-TreeBestEffort -Path $Staging -What 'the staged release pair'
        throw "refusing to publish: $OutputDir already exists as a directory, file or link; it was left untouched and the staged pair at $Staging was removed"
    }
    try {
        [System.IO.Directory]::Move($Staging, $OutputDir)
    } catch {
        Remove-TreeBestEffort -Path $Staging -What 'the staged release pair'
        throw "could not publish the validated release pair to $OutputDir ($($_.Exception.Message)); $OutputDir was left untouched"
    }
    return $OutputDir
}

function Invoke-ReleasePackaging {
  <#
  .SYNOPSIS
    The whole run: identity, tree, staging, validation, publication.
  #>
    param([string]$CardRoot, [string]$Tag, [string]$Commit, [string]$OutputDir)

    $root = Split-Path -Parent $PSScriptRoot
    if (-not (Test-Path -LiteralPath (Join-Path $root 'Cargo.toml') -PathType Leaf)) {
        throw "no Cargo.toml under $root; this script belongs in the repository's build/ directory"
    }
    $separator = [System.IO.Path]::DirectorySeparatorChar
    $ordinal = [System.StringComparison]::OrdinalIgnoreCase

    $card = Resolve-PathArgument -Path $CardRoot -What 'card root'
    $out = Resolve-PathArgument -Path $OutputDir -What 'output directory'

    # --- the release identity, before anything is read ---------------------------------------
    $version = Get-WorkspaceVersion -Root $root
    if ($version -notmatch '^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?$') {
        throw "Cargo.toml's workspace version '$version' is not a version a release tag may name"
    }
    if ($Tag -cne "v$version") {
        throw "the tag '$Tag' is not 'v$version' (v plus the workspace version in Cargo.toml); a release tag names the version it ships"
    }
    if ($Commit -cnotmatch '^[0-9a-fA-F]{40}$') { throw "the commit '$Commit' is not 40 hexadecimal characters" }

    # --- the paths this run may not read from or write to ------------------------------------
    foreach ($pair in @(@{ Path = $card; What = 'card root' }, @{ Path = $out; What = 'output directory' })) {
        if ($pair.Path -eq [System.IO.Path]::GetPathRoot($pair.Path)) { throw "refusing the filesystem root as the $($pair.What) ($($pair.Path))" }
    }
    if ($card -eq $out) { throw "the card tree and the output directory are the same path ($card)" }
    if (($root + $separator).StartsWith($out + $separator, $ordinal)) { throw "refusing $out, which contains the repository" }
    if (($card + $separator).StartsWith($out + $separator, $ordinal)) { throw "the output directory $out contains the card tree $card" }
    if (($out + $separator).StartsWith($card + $separator, $ordinal)) { throw "the output directory $out is inside the card tree $card" }

    Assert-NormalDirectory -Path $card -What 'card root'

    # --- the output path, which has to be absent before anything is written ------------------
    # The zip and its sidecar are one release unit and are published by renaming the directory that
    # holds them, so a path that already exists is refused here, before any staging begins, and is
    # left exactly as it was: nothing in this script replaces, merges into or cleans up an existing
    # path, and no failure may leave half a release pair behind. The release workflow passes a fresh
    # runner-temporary path, so this stricter rule costs the release nothing.
    if (Test-PathTaken -Path $out) {
        throw "refusing to publish: $out already exists as a directory, file or link; a release pair is published only into an absent directory, and that path was left untouched"
    }
    Assert-ExistingAncestorIsDirectory -Path $out -What 'output directory ancestor'

    $names = Get-CoreManifest -Root $root
    Step "validating the card tree at $card"
    Assert-CardTree -Root $root -CardRoot $card -Version $version -Commit $Commit -Names $names
    $files = @(Get-CardTreeFile -CardRoot $card)
    if ($files.Count -eq 0) { throw "$card holds no files" }

    # --- stage the pair beside the output, then publish it with one directory rename --------
    $leaf = "slot2-$Tag.zip"
    $parent = Split-Path -Parent $out
    New-Item -ItemType Directory -Force $parent | Out-Null
    $staging = Join-Path $parent ('.slot2-release-' + [guid]::NewGuid().ToString('n'))
    New-Item -ItemType Directory $staging | Out-Null
    $stagedZip = Join-Path $staging $leaf
    $stagedSidecar = Join-Path $staging "$leaf.sha256"
    $finalZip = Join-Path $out $leaf
    $finalSidecar = Join-Path $out "$leaf.sha256"
    $hash = $null
    $promoted = $false
    try {
        Step "writing $leaf from $($files.Count) files"
        New-ReleaseArchive -ZipPath $stagedZip -Files $files
        Assert-ReleaseArchive -ZipPath $stagedZip -Files $files
        $hash = Get-FileSha256 -Path $stagedZip
        [System.IO.File]::WriteAllText($stagedSidecar, "$hash  $leaf`n", (New-Object System.Text.UTF8Encoding($false)))
        [void](Assert-ReleaseSidecar -SidecarPath $stagedSidecar -ZipPath $stagedZip)
        Assert-ReleasePairDirectory -Path $staging -ZipLeaf $leaf -What 'the staging directory'

        Step "publishing $out"
        $promoted = Publish-ReleasePair -Staging $staging -OutputDir $out
        try {
            Assert-ReleasePairDirectory -Path $out -ZipLeaf $leaf -What 'the published release directory'
            $hash = Assert-ReleaseSidecar -SidecarPath $finalSidecar -ZipPath $finalZip
        } catch {
            throw "the published release pair at $out did not validate after the rename: $($_.Exception.Message) - it was left in place for manual inspection"
        }
    } finally {
        # Ownership: only a staging directory this run created and never promoted may be removed.
        # After the rename the staging path IS the output directory, so $promoted keeps this away from
        # it, and a failure inside Publish-ReleasePair has already removed the directory it owned.
        if (-not $promoted) { Remove-TreeBestEffort -Path $staging -What 'the staging directory' }
    }

    Step "released: $finalZip ($((Get-Item -LiteralPath $finalZip).Length) bytes, sha256 $hash)"
    Step "sidecar: $finalSidecar"
    Step 'done'
}

# Dot-sourcing defines the validators for the offline tests; only a real invocation packages.
if (-not $dotSourced) {
    Invoke-ReleasePackaging -CardRoot $CardRoot -Tag $Tag -Commit $Commit -OutputDir $OutputDir
}
