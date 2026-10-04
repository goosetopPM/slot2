<#
.SYNOPSIS
  The third-party Rust notices of the device runtime: the package set Cargo selects, the bundle
  layout, and the checks that hold the bundle together.

.DESCRIPTION
  Everything here reads Cargo's own data — `cargo tree` and `cargo metadata` in offline mode —
  and never a list of crate names written down beside the code. The set is the transitive normal
  dependency closure of package `slot2` for the device target with default features off and
  feature `device` on, minus every workspace/path package, because SLOT2's own MIT license
  already covers those.

  The bundle this describes:

    THIRD-PARTY-RUST.md   the inventory, package by package
    RUST-SBOM.json        slot2-rust-sbom-v1, a project-local deterministic inventory
    RUST-MANIFEST.txt     SHA-256 and byte length of every other file here
    packages/<key>/       PACKAGE.txt and, where the crate package has them, its original
                          license/notice files

  Every package carries a `notice_status`:

    packaged-text  the crate package contains at least one nonempty license/notice file, and
                   those files are copied here byte for byte
    declared-only  the crate package declares a license expression and supplied no license or
                   notice text at all; only the generated PACKAGE.txt is written, and the
                   declaration is repeated exactly as Cargo reports it

  Nothing is ever invented, downloaded, normalized, borrowed from another crate, or reduced to
  one side of an `OR`. The copied original text governs; this inventory is not legal advice.
  A package with neither `license` nor `license-file` metadata fails, and a declared
  `license-file` that is missing or empty fails.
#>

. (Join-Path $PSScriptRoot 'core-manifest.ps1')

function Get-RustNoticeSelection {
  <#
  .SYNOPSIS
    The build the notices must cover, and the Cargo commands that describe it.
  .DESCRIPTION
    `cargo tree --edges normal` is the normal closure Cargo resolves for that target, and
    `cargo metadata` is the same resolution read as a graph plus each package's declared
    metadata. Both are offline: a package that is not in the local Cargo cache is a failure that
    names it, never something fetched here.

    `cargo metadata` resolves the whole workspace and unifies features across its members, so it
    is a superset of this closure; the closure itself therefore comes from `cargo tree -p slot2`,
    which is the only interface that answers for this exact selection.
  #>
    $target = 'aarch64-unknown-linux-gnu'
    return [pscustomobject]@{
        Target = $target
        Package = 'slot2'
        Feature = 'device'
        TreeArguments = @(
            'tree', '-p', 'slot2', '--edges', 'normal', '--target', $target,
            '--no-default-features', '--features', 'device', '--prefix', 'none', '--offline')
        TreeMetadataArguments = @(
            'tree', '-p', 'slot2', '--edges', 'normal', '--target', $target,
            '--no-default-features', '--features', 'device', '--prefix', 'none',
            '--format', '{p}|{l}|{r}', '--offline')
        MetadataArguments = @(
            'metadata', '--format-version', '1', '--offline', '--no-default-features',
            '--features', 'slot2/device', '--filter-platform', $target)
    }
}

function Invoke-CargoCapture {
  <#
  .SYNOPSIS
    Run cargo and take its raw stdout, so UTF-8 metadata is not re-decoded through the pipeline.
  #>
    param([Parameter(Mandatory)][string]$Root, [Parameter(Mandatory)][string[]]$Arguments)

    $start = New-Object System.Diagnostics.ProcessStartInfo
    $start.FileName = 'cargo'
    $start.Arguments = (($Arguments | ForEach-Object { if ($_ -match '[\s"]') { '"' + $_ + '"' } else { $_ } }) -join ' ')
    $start.WorkingDirectory = $Root
    $start.UseShellExecute = $false
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.StandardOutputEncoding = [System.Text.Encoding]::UTF8
    $start.StandardErrorEncoding = [System.Text.Encoding]::UTF8
    $process = [System.Diagnostics.Process]::Start($start)
    $out = $process.StandardOutput.ReadToEnd()
    $err = $process.StandardError.ReadToEnd()
    $process.WaitForExit()
    $exit = $process.ExitCode
    $process.Dispose()
    return [pscustomobject]@{ Exit = $exit; Stdout = $out; Stderr = $err; Command = ('cargo ' + $start.Arguments) }
}

function New-CargoFailure {
  <#
  .SYNOPSIS
    The message a Cargo read fails with: what ran, what it said, and how to populate the cache
    without this script ever fetching anything.
  #>
    param([Parameter(Mandatory)]$Run, [Parameter(Mandatory)][string]$Root)

    return ("$($Run.Command) failed in $Root`: $([Environment]::NewLine)$($Run.Stderr.Trim())`n" +
        'The packages this build needs are read from the local Cargo cache only. With network access, populate it once with ' +
        "`cargo fetch --locked --target $(Get-RustNoticeSelection | Select-Object -ExpandProperty Target)` and run this again; this script never fetches.")
}

function ConvertFrom-RustTreeLine {
  <#
  .SYNOPSIS
    One `cargo tree` line as a package identity, or $null when the line is not a package.
  .DESCRIPTION
    A line is `name v version`, with two kinds of trailing text: cargo's own markers `(*)`
    (already shown above) and `(proc-macro)`, and a source — a filesystem path for a workspace
    package, a URL or `registry+…` for anything else. A workspace package is dropped here: it is
    not third party. A package with no source text is the default registry, which cargo does not
    print.

    Source identity is kept, not collapsed into name and version: the identity of a package is
    its name, its version and this token.
  #>
    param([string]$Line)

    $line = "$Line".Trim()
    # A `(*)` means "already shown above" and is cargo's note about the tree, not part of the
    # package; it is removed before the trailing text is read, or the greedy paren match would
    # swallow both.
    $line = $line -replace ' \(\*\)$', ''
    if (-not $line) { return $null }
    if ($line -notmatch '^(?<name>\S+) v(?<version>\S+?)(?<tail>\s+\((?<inner>.*)\))?$') { return $null }
    $token = ''
    if ($Matches['tail']) {
        $inner = $Matches['inner']
        if ($inner -eq '*' -or $inner -eq 'proc-macro') { }
        elseif ($inner -match '^[A-Za-z]:' -or $inner.StartsWith('/') -or $inner.StartsWith('\') -or $inner.Contains('\')) { return $null }
        else { $token = $inner }
    }
    return [pscustomobject]@{ Name = $Matches['name']; Version = $Matches['version']; SourceToken = $token }
}

function Get-RustNoticeIdentity {
    param($Entry)
    return "$($Entry.Name) $($Entry.Version) $($Entry.SourceToken)"
}

function Get-RustNoticeTreeFacts {
  <#
  .SYNOPSIS
    The device closure as `cargo tree` prints it, read twice and normalized twice.
  .DESCRIPTION
    The plain emission names packages; the fully formatted one adds each package's declared
    license expression and repository. Each is parsed by its own path and the two sets of
    source-aware identities must agree — a package that shows up in one and not the other, or
    with a different source, is the failure this comparison exists to catch.
  #>
    param([Parameter(Mandatory)][string]$Root)

    $selection = Get-RustNoticeSelection
    $plain = Invoke-CargoCapture -Root $Root -Arguments $selection.TreeArguments
    if ($plain.Exit -ne 0) { throw (New-CargoFailure -Run $plain -Root $Root) }
    $formatted = Invoke-CargoCapture -Root $Root -Arguments $selection.TreeMetadataArguments
    if ($formatted.Exit -ne 0) { throw (New-CargoFailure -Run $formatted -Root $Root) }

    $order = New-Object System.Collections.ArrayList
    $entries = @{}
    foreach ($line in ($plain.Stdout -split "\r?\n")) {
        $entry = ConvertFrom-RustTreeLine -Line $line
        if (-not $entry) { continue }
        $identity = Get-RustNoticeIdentity -Entry $entry
        if ($entries.ContainsKey($identity)) { continue }
        $entries[$identity] = $entry
        [void]$order.Add($identity)
    }
    if ($order.Count -eq 0) { throw "cargo tree reported no packages for $($selection.Target)" }

    $licenses = @{}
    $repositories = @{}
    $formattedIdentities = @{}
    foreach ($line in ($formatted.Stdout -split "\r?\n")) {
        if (-not $line) { continue }
        $parts = $line.Split('|')
        if ($parts.Count -lt 3) { continue }
        $entry = ConvertFrom-RustTreeLine -Line $parts[0]
        if (-not $entry) { continue }
        $identity = Get-RustNoticeIdentity -Entry $entry
        $formattedIdentities[$identity] = $true
        $licenses[$identity] = $parts[1].Trim()
        $repositories[$identity] = $parts[2].Trim()
    }

    $missing = @($order | Where-Object { -not $formattedIdentities.ContainsKey($_) } | Sort-Object)
    $extra = @($formattedIdentities.Keys | Where-Object { -not $entries.ContainsKey($_) } | Sort-Object)
    if ($missing.Count -gt 0 -or $extra.Count -gt 0) {
        $lines = @()
        foreach ($n in $missing) { $lines += "  in the plain tree, not in the formatted one: $n" }
        foreach ($n in $extra) { $lines += "  in the formatted tree, not in the plain one: $n" }
        throw "the device dependency closure disagrees between two cargo tree emissions:$([Environment]::NewLine)$($lines -join [Environment]::NewLine)"
    }

    return [pscustomobject]@{
        Entries = @($order | ForEach-Object { $entries[$_] })
        Licenses = $licenses
        Repositories = $repositories
        Target = $selection.Target
    }
}

function Get-RustNoticeMetadataPackages {
  <#
  .SYNOPSIS
    Every package `cargo metadata` knows, with its declared metadata, as an unfiltered list.
  .DESCRIPTION
    Returning the whole list rather than a dictionary keyed by name and version is the point:
    two packages can share a name and version and differ only in source, and a dictionary would
    have silently kept one of them. Callers match on source identity and fail on ambiguity.
  #>
    param([Parameter(Mandatory)][string]$Root)

    $selection = Get-RustNoticeSelection
    $run = Invoke-CargoCapture -Root $Root -Arguments $selection.MetadataArguments
    if ($run.Exit -ne 0) { throw (New-CargoFailure -Run $run -Root $Root) }
    $metadata = $run.Stdout | ConvertFrom-Json

    $rootPackage = @($metadata.packages | Where-Object { $_.name -eq $selection.Package -and -not $_.source })
    if ($rootPackage.Count -ne 1) { throw "cargo metadata names $($rootPackage.Count) packages called $($selection.Package)" }

    $features = @()
    foreach ($node in $metadata.resolve.nodes) {
        if ($node.id -eq $rootPackage[0].id) { $features = @($node.features | Where-Object { $_ -ne 'default' } | Sort-Object) }
    }
    return [pscustomobject]@{
        Packages = @($metadata.packages | Where-Object { $_.source })
        RootVersion = "$($rootPackage[0].version)"
        RootFeatures = $features
    }
}

function Test-RustNoticeSourceMatches {
  <#
  .SYNOPSIS
    Whether a `cargo tree` source token and a `cargo metadata` source are the same origin.
  .DESCRIPTION
    cargo prints the default registry with no source at all, and a git dependency as
    `<url>#<rev>` where metadata records `git+<url>?<query>#<rev>`. Anything that cannot be
    matched is left unmatched on purpose: the caller turns a package with several candidates
    that cannot be told apart into a clear failure rather than a silent choice.
  #>
    param([string]$Token, [string]$Source)

    if (-not $Token) { return $Source.StartsWith('registry+') }
    if ($Source -ceq $Token) { return $true }
    $metadataCore = $Source -replace '^git\+', '' -replace '^registry\+', ''
    $tokenCore = $Token -replace '^git\+', '' -replace '^registry\+', ''
    $metadataOrigin = (($metadataCore -split '#')[0] -split '\?')[0]
    $tokenOrigin = (($tokenCore -split '#')[0] -split '\?')[0]
    if ($metadataOrigin -cne $tokenOrigin) { return $false }
    # Same origin. A revision has to match only when cargo printed one, because metadata carries
    # the query cargo omits from its output.
    $tokenRev = ''
    if ($tokenCore.Contains('#')) { $tokenRev = ($tokenCore -split '#')[-1] }
    if ($tokenRev) { return (((($metadataCore -split '#')[-1]) -split '\?')[0] -ceq $tokenRev) }
    if ($metadataCore.Contains('#')) { return ($metadataCore.EndsWith($tokenCore)) }
    return $true
}

function Resolve-RustNoticePackages {
  <#
  .SYNOPSIS
    Match each closure identity to exactly one `cargo metadata` package.
  .DESCRIPTION
    A name and version alone is not an identity. When several metadata packages share them, the
    source has to single one out; when it cannot, this fails and says so, because dropping one
    of them would put a crate in the device binary whose notice is not in the bundle.
  #>
    param([Parameter(Mandatory)][AllowEmptyCollection()]$Entries, [Parameter(Mandatory)][AllowEmptyCollection()]$Packages)

    $resolved = @()
    $usedIds = @{}
    foreach ($entry in $Entries) {
        $candidates = @($Packages | Where-Object { $_.name -ceq $entry.Name -and $_.version -ceq $entry.Version })
        if ($candidates.Count -eq 0) {
            throw "cargo tree names $($entry.Name) $($entry.Version) for the device build and cargo metadata does not know it"
        }
        $matches = @($candidates | Where-Object { Test-RustNoticeSourceMatches -Token $entry.SourceToken -Source "$($_.source)" })
        if ($matches.Count -ne 1) {
            $described = @($candidates | ForEach-Object { "    $($_.name) $($_.version) from $($_.source)" })
            throw ("$($entry.Name) $($entry.Version) is ambiguous: cargo tree reports source '$($entry.SourceToken)' and cargo metadata has $($candidates.Count) package(s) under that name and version, $($matches.Count) of which match:" +
                [Environment]::NewLine + ($described -join [Environment]::NewLine) +
                [Environment]::NewLine + '  A package this build links cannot be told apart, so the closure is not safe to document.')
        }
        $package = $matches[0]
        if ($usedIds.ContainsKey($package.id) -and $usedIds[$package.id] -ne (Get-RustNoticeIdentity -Entry $entry)) {
            throw "two closure identities resolved to $($package.name) $($package.version) from $($package.source)"
        }
        $usedIds[$package.id] = (Get-RustNoticeIdentity -Entry $entry)
        $resolved += [pscustomobject]@{
            Identity = Get-RustNoticeIdentity -Entry $entry
            Name = $entry.Name
            Version = $entry.Version
            SourceToken = $entry.SourceToken
            Source = "$($package.source)"
            License = "$($package.license)"
            LicenseFile = "$($package.license_file)"
            Repository = "$($package.repository)"
            Homepage = "$($package.homepage)"
            ManifestPath = "$($package.manifest_path)"
        }
    }
    return $resolved
}

function Get-RustNoticeKey {
  <#
  .SYNOPSIS
    The stable, path-safe key a package's directory is named after.
  .DESCRIPTION
    `name-version` with everything outside `[A-Za-z0-9._-]` replaced, which is what keeps
    `proc-macro-hack 0.5.20+deprecated` a directory Windows can make. `-Disambiguate` adds the
    source's own hash, and callers pass it only where two packages would otherwise want the
    same directory name.
  #>
    param([Parameter(Mandatory)][string]$Name, [Parameter(Mandatory)][string]$Version, [string]$Source,
          [switch]$Disambiguate)

    $base = [regex]::Replace("$Name-$Version", '[^A-Za-z0-9._-]', '_')
    if (-not $Disambiguate -or -not $Source) { return $base }
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try { $digest = [System.BitConverter]::ToString($sha.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($Source))).Replace('-', '').ToLowerInvariant() }
    finally { $sha.Dispose() }
    return "$base-s$($digest.Substring(0, 8))"
}

function Get-RustNoticeSourceKind {
    param([string]$Source)
    if (-not $Source) { return 'path' }
    if ($Source.StartsWith('registry+')) { return 'registry' }
    if ($Source.StartsWith('git+')) { return 'git' }
    return 'other'
}

function Get-RustNoticeDestName {
  <#
  .SYNOPSIS
    The name a package's license file gets inside its directory.
  .DESCRIPTION
    A top-level file keeps its name; a declared `license-file` under a subdirectory keeps its
    path as a prefix, so `LICENSE` and `licenses/LICENSE` in one package cannot collide. The
    result is restricted to characters a path can carry on every platform this runs on.
  #>
    param([Parameter(Mandatory)][string]$Relative)

    $flat = $Relative.Replace('\', '/').Replace('/', '_')
    return [regex]::Replace($flat, '[^A-Za-z0-9._-]', '_')
}

function Get-RustLockChecksums {
  <#
  .SYNOPSIS
    The `checksum` Cargo.lock records for each package, keyed by `name version source`.
  .DESCRIPTION
    Read from the lock file rather than asked of the network: the checksum is what ties the
    extracted source on this machine to the crate the lock names. The source is part of the key,
    so two packages that share a name and version keep their own checksums.
  #>
    param([Parameter(Mandatory)][string]$Path)

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { throw "no Cargo.lock at $Path" }
    $text = [System.IO.File]::ReadAllText($Path)
    $checksums = @{}
    foreach ($chunk in ($text -split '(?m)^\[\[package\]\]\r?\n')) {
        $name = [regex]::Match($chunk, '(?m)^name = "([^"]+)"').Groups[1].Value
        if (-not $name) { continue }
        $version = [regex]::Match($chunk, '(?m)^version = "([^"]+)"').Groups[1].Value
        $source = [regex]::Match($chunk, '(?m)^source = "([^"]+)"').Groups[1].Value
        $checksum = [regex]::Match($chunk, '(?m)^checksum = "([^"]+)"').Groups[1].Value
        $checksums["$name $version $source".Trim()] = $checksum
    }
    return $checksums
}

function Get-RustNoticeKeys {
  <#
  .SYNOPSIS
    The stable directory key for each resolved package, disambiguated only where it has to be.
  .DESCRIPTION
    A key is `name-version` until two packages want the same one, which happens exactly when the
    same name and version arrive from two different sources. Those two get the source's own hash
    appended and stay two packages; everything else keeps the short name.
  #>
    param([Parameter(Mandatory)][AllowEmptyCollection()]$Packages)

    $sourcesByBase = @{}
    foreach ($package in $Packages) {
        $base = Get-RustNoticeKey -Name $package.Name -Version $package.Version
        if (-not $sourcesByBase.ContainsKey($base)) { $sourcesByBase[$base] = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal) }
        [void]$sourcesByBase[$base].Add($package.Source)
    }

    $result = @()
    foreach ($package in $Packages) {
        $base = Get-RustNoticeKey -Name $package.Name -Version $package.Version
        $result += [pscustomobject]@{
            Package = $package
            Base = $base
            Key = Get-RustNoticeKey -Name $package.Name -Version $package.Version -Source $package.Source `
                -Disambiguate:($sourcesByBase[$base].Count -gt 1)
        }
    }
    return $result
}

function Get-RustNoticeScope {
  <#
  .SYNOPSIS
    The third-party packages the device runtime is built from, each with its declared metadata,
    its notice status, and the license files that travel with it.
  #>
    param([Parameter(Mandatory)][string]$Root)

    $tree = Get-RustNoticeTreeFacts -Root $Root
    $metadata = Get-RustNoticeMetadataPackages -Root $Root
    $resolved = Resolve-RustNoticePackages -Entries $tree.Entries -Packages $metadata.Packages
    $checksums = Get-RustLockChecksums -Path (Join-Path $Root 'Cargo.lock')

    $entries = @()
    foreach ($assignment in @(Get-RustNoticeKeys -Packages $resolved)) {
        $package = $assignment.Package
        $key = $assignment.Key
        $packageDir = Split-Path -Parent $package.ManifestPath
        if (-not (Test-Path -LiteralPath $packageDir -PathType Container)) {
            throw "no cached source for $($package.Name) $($package.Version) at $packageDir — populate the Cargo cache with an online `cargo fetch` and run this again"
        }
        if (-not $package.License -and -not $package.LicenseFile) {
            throw "$($package.Name) $($package.Version) declares neither license nor license-file metadata"
        }

        $candidates = New-Object System.Collections.ArrayList
        if ($package.LicenseFile) {
            $declared = $package.LicenseFile.Replace('\', '/')
            $path = Join-Path $packageDir ($declared -replace '/', '\')
            if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
                throw "$($package.Name) $($package.Version) declares license-file '$declared', which is not in the package at $packageDir"
            }
            if ((Get-Item -LiteralPath $path).Length -eq 0) {
                throw "$($package.Name) $($package.Version) declares license-file '$declared', which is empty"
            }
            [void]$candidates.Add([pscustomobject]@{ Relative = $declared; FullName = $path })
        }
        foreach ($file in @(Get-ChildItem -LiteralPath $packageDir -File |
                Where-Object { $_.Name -match '^(?i)(license|licence|copying|copyright|notice)' } | Sort-Object Name)) {
            if ($file.Length -eq 0) { continue }
            if (@($candidates | Where-Object { $_.FullName -ieq $file.FullName }).Count -gt 0) { continue }
            [void]$candidates.Add([pscustomobject]@{ Relative = $file.Name; FullName = $file.FullName })
        }

        # The status is read from what the crate package actually contains, never recorded
        # beside it: a crate that starts shipping a text becomes packaged-text on its own.
        $status = if ($candidates.Count -gt 0) { 'packaged-text' } else { 'declared-only' }
        if ($status -eq 'declared-only' -and -not $package.License) {
            throw "$($package.Name) $($package.Version) has no packaged license text and declares no license expression either"
        }

        $notices = @()
        $destNames = @{}
        foreach ($candidate in @($candidates | Sort-Object Relative)) {
            $dest = Get-RustNoticeDestName -Relative $candidate.Relative
            if ($destNames.ContainsKey($dest)) {
                throw "$($package.Name) $($package.Version) would write two notice files to '$dest'"
            }
            $destNames[$dest] = $true
            $bytes = [System.IO.File]::ReadAllBytes($candidate.FullName)
            $sha = [System.Security.Cryptography.SHA256]::Create()
            try { $digest = [System.BitConverter]::ToString($sha.ComputeHash($bytes)).Replace('-', '').ToLowerInvariant() }
            finally { $sha.Dispose() }
            $notices += [pscustomobject]@{
                Source = $candidate.FullName
                Relative = "$key/$dest"
                Path = "packages/$key/$dest"
                Bytes = $bytes.Length
                Sha256 = $digest
            }
        }

        $checksumKey = "$($package.Name) $($package.Version) $($package.Source)"
        $entries += [pscustomobject]@{
            Key = $key
            Name = $package.Name
            Version = $package.Version
            Source = $package.Source
            Identity = $package.Identity
            SourceKind = Get-RustNoticeSourceKind -Source $package.Source
            Checksum = if ($checksums.ContainsKey($checksumKey)) { $checksums[$checksumKey] } else { '' }
            License = $package.License
            LicenseFile = $package.LicenseFile
            NoticeStatus = $status
            Repository = $package.Repository
            Homepage = $package.Homepage
            PackageDir = $packageDir
            Notices = $notices
        }
    }

    $entries = @($entries | Sort-Object Key)
    $seenKeys = @{}
    foreach ($entry in $entries) {
        if ($seenKeys.ContainsKey($entry.Key)) { throw "two packages share the stable key $($entry.Key)" }
        $seenKeys[$entry.Key] = $true
    }

    return [pscustomobject]@{
        Root = $Root
        Target = $tree.Target
        RootPackage = (Get-RustNoticeSelection).Package
        RootVersion = $metadata.RootVersion
        RootFeatures = @($metadata.RootFeatures)
        LockSha256 = (Get-FileSha256 -Path (Join-Path $Root 'Cargo.lock'))
        Packages = $entries
    }
}

function Get-RustDeclaredOnlyNotice {
  <#
  .SYNOPSIS
    The sentence a package with a declaration and no shipped text carries, verbatim.
  #>
    return ('The crate package declares this license expression but supplied no license/notice text.' + [Environment]::NewLine +
        'Consult the recorded upstream repository. This inventory does not replace the license and is not legal advice.')
}

function Format-RustNoticeJson {
  <#
  .SYNOPSIS
    The SBOM text: UTF-8 without BOM, LF only, no timestamps and no path from this machine.
  #>
    param([Parameter(Mandatory)]$Scope)

    $packages = @()
    foreach ($entry in $Scope.Packages) {
        $packages += [ordered]@{
            key = $entry.Key
            name = $entry.Name
            version = $entry.Version
            source = $entry.Source
            source_kind = $entry.SourceKind
            checksum = $entry.Checksum
            license = $entry.License
            license_file = $entry.LicenseFile
            notice_status = $entry.NoticeStatus
            repository = $entry.Repository
            homepage = $entry.Homepage
            notices = @($entry.Notices | ForEach-Object { $_.Path } | Sort-Object)
        }
    }
    $document = [ordered]@{
        format = 'slot2-rust-sbom-v1'
        target = $Scope.Target
        no_default_features = $true
        root = [ordered]@{
            name = $Scope.RootPackage
            version = $Scope.RootVersion
            features = @($Scope.RootFeatures | Sort-Object)
        }
        cargo_lock = [ordered]@{ path = 'Cargo.lock'; sha256 = $Scope.LockSha256 }
        packages = $packages
    }
    $json = $document | ConvertTo-Json -Depth 8
    return (($json -replace "`r`n", "`n") + "`n")
}

function Get-RustNoticeInventoryText {
    param([Parameter(Mandatory)]$Scope)

    $packagedText = @($Scope.Packages | Where-Object { $_.NoticeStatus -eq 'packaged-text' })
    $declaredOnly = @($Scope.Packages | Where-Object { $_.NoticeStatus -eq 'declared-only' })

    $lines = @()
    $lines += '# Third-party Rust notices'
    $lines += ''
    $lines += 'These are the third-party Rust packages the device runtime links, as Cargo resolves them for'
    $lines += "target ``$($Scope.Target)`` with default features off and feature ``$($Scope.RootFeatures -join ',')`` on:"
    $lines += 'the transitive normal dependency closure of package `slot2`. SLOT2''s own workspace crates are'
    $lines += 'not listed here; the repository''s MIT license covers them.'
    $lines += ''
    $lines += 'Every runtime package is inventoried. Where the crate package itself contains license or notice'
    $lines += 'files, they are copied byte for byte into `packages/` and the entry is marked `packaged-text`.'
    $lines += 'A **`declared-only`** entry is one whose crate package declares a license expression and supplied'
    $lines += 'no license or notice text at all: nothing was invented, downloaded or borrowed for it, and the'
    $lines += 'declaration below is repeated exactly as Cargo reports it.'
    $lines += ''
    $lines += 'Where a package declares an SPDX expression, that declaration is repeated here and no side of it'
    $lines += 'is chosen: **the copied original text governs, and this inventory is not legal advice.**'
    $lines += ''
    $lines += "Cargo.lock SHA-256: ``$($Scope.LockSha256)``"
    $lines += "Packages: $($Scope.Packages.Count) ($($packagedText.Count) packaged-text, $($declaredOnly.Count) declared-only)"
    $lines += ''
    foreach ($entry in $Scope.Packages) {
        $lines += "## $($entry.Key)"
        $lines += ''
        $lines += "| field | value |"
        $lines += "|---|---|"
        $lines += "| package | $($entry.Name) $($entry.Version) |"
        $lines += "| source | $($entry.Source) |"
        if ($entry.Checksum) { $lines += "| checksum | $($entry.Checksum) |" }
        $lines += "| declared license | $(if ($entry.License) { $entry.License } else { '(none declared)' }) |"
        if ($entry.LicenseFile) { $lines += "| declared license-file | $($entry.LicenseFile) |" }
        $lines += "| notice status | $($entry.NoticeStatus) |"
        if ($entry.Repository) { $lines += "| repository | $($entry.Repository) |" }
        if ($entry.Homepage) { $lines += "| homepage | $($entry.Homepage) |" }
        foreach ($notice in $entry.Notices) { $lines += "| notice | $($notice.Path) |" }
        $lines += ''
        if ($entry.NoticeStatus -eq 'declared-only') {
            $lines += (Get-RustDeclaredOnlyNotice)
            $lines += ''
        }
    }
    return (($lines -join "`n") + "`n")
}

function Get-RustNoticePackageText {
    param([Parameter(Mandatory)]$Entry)

    $lines = @()
    $lines += "key=$($Entry.Key)"
    $lines += "name=$($Entry.Name)"
    $lines += "version=$($Entry.Version)"
    $lines += "source=$($Entry.Source)"
    $lines += "source_kind=$($Entry.SourceKind)"
    if ($Entry.Checksum) { $lines += "checksum=$($Entry.Checksum)" }
    $lines += "license=$(if ($Entry.License) { $Entry.License } else { '' })"
    $lines += "license_file=$($Entry.LicenseFile)"
    $lines += "notice_status=$($Entry.NoticeStatus)"
    $lines += "repository=$($Entry.Repository)"
    $lines += "homepage=$($Entry.Homepage)"
    foreach ($notice in $Entry.Notices) { $lines += "notice=$($notice.Path)" }
    $lines += ''
    if ($Entry.NoticeStatus -eq 'declared-only') {
        $lines += (Get-RustDeclaredOnlyNotice)
    } else {
        $lines += 'The copied original text in this directory governs; this file is an inventory, not legal advice.'
    }
    return (($lines -join "`n") + "`n")
}

function Write-RustNoticeBundle {
  <#
  .SYNOPSIS
    Write the whole bundle into a directory that is still a staging directory.
  #>
    param([Parameter(Mandatory)]$Scope, [Parameter(Mandatory)][string]$Bundle)

    $packagesDir = Join-Path $Bundle 'packages'
    New-Item -ItemType Directory -Force $packagesDir | Out-Null
    $utf8 = New-Object System.Text.UTF8Encoding($false)

    foreach ($entry in $Scope.Packages) {
        $dir = Join-Path $packagesDir $entry.Key
        New-Item -ItemType Directory -Force $dir | Out-Null
        foreach ($notice in $entry.Notices) {
            [System.IO.File]::WriteAllBytes((Join-Path $dir ([System.IO.Path]::GetFileName($notice.Path))), [System.IO.File]::ReadAllBytes($notice.Source))
        }
        [System.IO.File]::WriteAllText((Join-Path $dir 'PACKAGE.txt'), (Get-RustNoticePackageText -Entry $entry), $utf8)
    }

    [System.IO.File]::WriteAllText((Join-Path $Bundle 'THIRD-PARTY-RUST.md'), (Get-RustNoticeInventoryText -Scope $Scope), $utf8)
    [System.IO.File]::WriteAllText((Join-Path $Bundle 'RUST-SBOM.json'), (Format-RustNoticeJson -Scope $Scope), $utf8)

    # RUST-MANIFEST.txt is written last and does not hash itself.
    $manifestPath = Join-Path $Bundle 'RUST-MANIFEST.txt'
    $lines = @()
    foreach ($file in @(Get-ChildItem -LiteralPath $Bundle -Recurse -File | Sort-Object FullName)) {
        if ($file.FullName -ieq $manifestPath) { continue }
        $rel = $file.FullName.Substring($Bundle.Length + 1).Replace('\', '/')
        $lines += "$(Get-FileSha256 -Path $file.FullName) $($file.Length) $rel"
    }
    [System.IO.File]::WriteAllText($manifestPath, (($lines -join "`n") + "`n"), $utf8)
}

function Test-RustNoticeRelativePath {
    param([string]$Path)
    if (-not $Path) { return $false }
    if ($Path.Contains('\')) { return $false }
    if ($Path.StartsWith('/') -or $Path -match '^[A-Za-z]:') { return $false }
    foreach ($part in $Path.Split('/')) {
        if ($part -eq '' -or $part -eq '.' -or $part -eq '..') { return $false }
    }
    return $true
}

function Assert-RustNoticeBundle {
  <#
  .SYNOPSIS
    The postflight: the bundle that is about to be shipped is the closure Cargo resolves now.
  .DESCRIPTION
    The package set, the notice status of every package and the bytes of every copied file are
    rediscovered rather than trusted, so a bundle that lost a package, kept one the device build
    does not link, drifted from the cached crate, or was edited after it was written is refused
    here. A declared-only package is checked for the very thing that makes it declared-only: it
    must have no notice file anywhere, and it must still be present in the directory set, the
    SBOM, the inventory and the hash manifest.
  #>
    param([Parameter(Mandatory)][string]$Root, [Parameter(Mandatory)][string]$Bundle)

    if (-not (Test-Path -LiteralPath $Bundle -PathType Container)) { throw "no rust notice bundle at $Bundle" }
    $scope = Get-RustNoticeScope -Root $Root
    $expected = @($scope.Packages | ForEach-Object { $_.Key })

    foreach ($rel in @('THIRD-PARTY-RUST.md', 'RUST-SBOM.json', 'RUST-MANIFEST.txt')) {
        $file = Join-Path $Bundle $rel
        if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "no $rel in $Bundle" }
        if ((Get-Item -LiteralPath $file).Length -eq 0) { throw "$file is empty" }
    }
    $inventory = [System.IO.File]::ReadAllText((Join-Path $Bundle 'THIRD-PARTY-RUST.md'))

    $have = @(Get-ChildItem -LiteralPath (Join-Path $Bundle 'packages') -Directory | Select-Object -ExpandProperty Name | Sort-Object)
    Assert-SameNameSet -What "$Bundle/packages" -Want @($expected | Sort-Object) -Have $have

    foreach ($entry in $scope.Packages) {
        $dir = Join-Path $Bundle "packages/$($entry.Key)"
        $packageFile = Join-Path $dir 'PACKAGE.txt'
        if (-not (Test-Path -LiteralPath $packageFile -PathType Leaf)) { throw "no PACKAGE.txt in $dir" }
        $recorded = [System.IO.File]::ReadAllText($packageFile)
        foreach ($field in @("key=$($entry.Key)", "name=$($entry.Name)", "version=$($entry.Version)",
                "source=$($entry.Source)", "notice_status=$($entry.NoticeStatus)")) {
            if (-not $recorded.Contains($field)) { throw "$packageFile does not record '$field'" }
        }
        if (-not $inventory.Contains("## $($entry.Key)")) { throw "$($entry.Key) is missing from THIRD-PARTY-RUST.md" }

        if ($entry.NoticeStatus -eq 'declared-only') {
            if ($entry.Notices.Count -ne 0) { throw "$($entry.Key) is declared-only and the discovery gave it notice files" }
            if (-not $recorded.Contains('supplied no license/notice text')) { throw "$packageFile does not state that the crate package supplied no license text" }
            Assert-SameNameSet -What $dir -Want @('PACKAGE.txt') -Have (Get-ChildFileNames -Path $dir)
        } else {
            if ($entry.Notices.Count -eq 0) { throw "$($entry.Key) is packaged-text with no notice file" }
            Assert-SameNameSet -What $dir -Want (@($entry.Notices | ForEach-Object { [System.IO.Path]::GetFileName($_.Path) }) + @('PACKAGE.txt')) `
                -Have (Get-ChildFileNames -Path $dir)
            foreach ($notice in $entry.Notices) {
                $file = Join-Path $Bundle ($notice.Path -replace '/', '\')
                if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "no $($notice.Path) in $Bundle" }
                if ((Get-Item -LiteralPath $file).Length -ne $notice.Bytes) { throw "$($notice.Path) is not the length the package's own text has" }
                if ((Get-FileSha256 -Path $file) -cne $notice.Sha256) { throw "$($notice.Path) is not the bytes $($entry.Name) $($entry.Version) ships" }
            }
        }
    }

    # The SBOM: parsed, then compared with the same discovery that wrote it.
    $sbom = [System.IO.File]::ReadAllText((Join-Path $Bundle 'RUST-SBOM.json')) | ConvertFrom-Json
    if ($sbom.format -cne 'slot2-rust-sbom-v1') { throw "RUST-SBOM.json declares format '$($sbom.format)'" }
    if ($sbom.target -cne $scope.Target) { throw "RUST-SBOM.json names target $($sbom.target), not $($scope.Target)" }
    if ($sbom.root.name -cne $scope.RootPackage) { throw "RUST-SBOM.json names root package $($sbom.root.name)" }
    if ($sbom.root.version -cne $scope.RootVersion) { throw "RUST-SBOM.json names root version $($sbom.root.version)" }
    if ($sbom.cargo_lock.sha256 -cne $scope.LockSha256) { throw 'RUST-SBOM.json does not name this Cargo.lock' }
    $sbomKeys = @($sbom.packages | ForEach-Object { $_.key })
    Assert-SameNameSet -What 'RUST-SBOM.json packages' -Want $expected -Have $sbomKeys
    $sorted = @($sbomKeys | Sort-Object)
    if (($sbomKeys -join "`n") -cne ($sorted -join "`n")) { throw 'RUST-SBOM.json packages are not sorted by key' }
    $unique = @($sbomKeys | Sort-Object -Unique)
    if ($unique.Count -ne $sbomKeys.Count) { throw 'RUST-SBOM.json repeats a package key' }
    for ($i = 0; $i -lt $scope.Packages.Count; $i++) {
        $entry = $scope.Packages[$i]
        $record = @($sbom.packages | Where-Object { $_.key -ceq $entry.Key })[0]
        # The SBOM spells its fields with underscores; the entry objects are PowerShell
        # properties. They are paired explicitly rather than relied on to match.
        $fields = @(
            @{ Sbom = 'name'; Entry = 'Name' }, @{ Sbom = 'version'; Entry = 'Version' },
            @{ Sbom = 'source'; Entry = 'Source' }, @{ Sbom = 'checksum'; Entry = 'Checksum' },
            @{ Sbom = 'license'; Entry = 'License' }, @{ Sbom = 'license_file'; Entry = 'LicenseFile' },
            @{ Sbom = 'notice_status'; Entry = 'NoticeStatus' },
            @{ Sbom = 'repository'; Entry = 'Repository' }, @{ Sbom = 'homepage'; Entry = 'Homepage' }
        )
        foreach ($field in $fields) {
            $want = "$($entry.($field.Entry))"
            $got = "$($record.($field.Sbom))"
            if ($got -cne $want) { throw "RUST-SBOM.json records $($field.Sbom) '$got' for $($entry.Key), not '$want'" }
        }
        $statuses = @('packaged-text', 'declared-only')
        if ($record.notice_status -cnotin $statuses) {
            throw "RUST-SBOM.json records notice_status '$($record.notice_status)' for $($entry.Key)"
        }
        $paths = @($record.notices)
        $wantPaths = @($entry.Notices | ForEach-Object { $_.Path } | Sort-Object)
        # A path inside the SBOM is checked for being a path at all before it is compared with the
        # crate's, so a malformed document is refused as malformed.
        foreach ($path in $paths) {
            if (-not (Test-RustNoticeRelativePath -Path $path)) { throw "RUST-SBOM.json has an unsafe notice path '$path' for $($entry.Key)" }
            if (-not (Test-Path -LiteralPath (Join-Path $Bundle ($path -replace '/', '\')) -PathType Leaf)) {
                throw "RUST-SBOM.json names $path, which is not in $Bundle"
            }
        }
        if ($paths.Count -ne $wantPaths.Count) {
            throw "RUST-SBOM.json lists $($paths.Count) notice path(s) for $($entry.Key) and the crate has $($wantPaths.Count)"
        }
        if ($wantPaths.Count -gt 0) {
            Assert-SameNameSet -What "RUST-SBOM.json notices for $($entry.Key)" -Want $wantPaths -Have $paths
        }
    }

    # RUST-MANIFEST.txt: every entry checked, and every other file present in it exactly once.
    $manifestPath = Join-Path $Bundle 'RUST-MANIFEST.txt'
    $listed = @{}
    foreach ($line in ([System.IO.File]::ReadAllText($manifestPath) -split "\r?\n")) {
        if (-not $line) { continue }
        if ($line -notmatch '^(?<sha>[0-9a-f]{64}) (?<bytes>[0-9]+) (?<path>.+)$') { throw "RUST-MANIFEST.txt has an unreadable line: $line" }
        $rel = $Matches['path']
        if ($rel -ceq 'RUST-MANIFEST.txt') { throw 'RUST-MANIFEST.txt lists itself' }
        if (-not (Test-RustNoticeRelativePath -Path $rel)) { throw "RUST-MANIFEST.txt has an unsafe path '$rel'" }
        if ($listed.ContainsKey($rel)) { throw "RUST-MANIFEST.txt lists $rel twice" }
        $file = Join-Path $Bundle ($rel -replace '/', '\')
        if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "RUST-MANIFEST.txt names $rel, which is not in $Bundle" }
        if ([int64]$Matches['bytes'] -ne (Get-Item -LiteralPath $file).Length) { throw "$rel is not the length RUST-MANIFEST.txt records" }
        if ((Get-FileSha256 -Path $file) -cne $Matches['sha']) { throw "$rel is not the hash RUST-MANIFEST.txt records" }
        $listed[$rel] = $true
    }
    $actual = @()
    foreach ($file in @(Get-ChildItem -LiteralPath $Bundle -Recurse -File)) {
        if ($file.FullName -ieq $manifestPath) { continue }
        $actual += $file.FullName.Substring($Bundle.Length + 1).Replace('\', '/')
    }
    Assert-SameNameSet -What 'RUST-MANIFEST.txt' -Want @($actual | Sort-Object) -Have @($listed.Keys | Sort-Object)
    foreach ($entry in $scope.Packages) {
        if (-not $listed.ContainsKey("packages/$($entry.Key)/PACKAGE.txt")) {
            throw "RUST-MANIFEST.txt does not cover $($entry.Key)"
        }
    }

    # Nothing is returned on purpose: this is a check, and a returned object would be echoed to
    # the caller's output stream as noise.
}
