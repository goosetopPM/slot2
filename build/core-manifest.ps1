<#
.SYNOPSIS
  The one list of the cores a SLOT2 card ships, and the two checks that go with it.

.DESCRIPTION
  cores/required.txt is the build and distribution manifest: which cores the device card and
  the release zip contain, and in what order they are assembled. The runtime's own list is
  `Core::ALL` in slot2-retro, and a test in that crate holds the two together — so neither
  build/cores.ps1 nor build/dist-device.ps1 nor the CI workflow carries the names again.

  Every function takes the repository root explicitly rather than reading the caller's
  directory, so build/dist-device.ps1 can point the same checks at a staging tree when it
  wants to see a failure path. Dot-source this file; it defines functions and does nothing
  else, and its functions only read.
#>

# Reading the pinned licenses and the archives they live in needs ZipFile, which Windows
# PowerShell and PowerShell 7 keep in differently named assemblies. One of these loads it and
# the other is a no-op; if neither is there, the first use of the type says so.
foreach ($assembly in @('System.IO.Compression.FileSystem', 'System.IO.Compression')) {
    try { Add-Type -AssemblyName $assembly -ErrorAction Stop } catch { }
}

function Get-CoreManifestPath {
    param([Parameter(Mandatory)][string]$Root)
    Join-Path $Root 'cores/required.txt'
}

function Get-CoreManifest {
  <#
  .SYNOPSIS
    The manifest's core names, in order, validated.
  .DESCRIPTION
    A name that is not `[a-z0-9_]+`, a blank line, a duplicate, or a row whose core has no
    build.sh or commit beside it is an error here — before anything is built or fetched. A
    list that a build could not act on is not a list to skip a core from.

    Every comparison here is case-sensitive on purpose. Windows would happily open
    `cores/Mgba/build.sh` and hand back `vendor/Mgba_libretro.so` beside the real
    `mgba_libretro.so`, and a card assembled that way has a core the frontend cannot name.
  #>
    param([Parameter(Mandatory)][string]$Root)

    $path = Get-CoreManifestPath -Root $Root
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "no core manifest at $path" }
    $names = @(Get-Content -LiteralPath $path | ForEach-Object { $_.Trim() })
    if ($names.Count -eq 0) { throw "$path is empty" }

    $seen = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
    foreach ($name in $names) {
        if ($name -cnotmatch '^[a-z0-9_]+$') { throw "$path has '$name', which is not a core name" }
        if (-not $seen.Add($name)) { throw "$path lists '$name' twice" }
        foreach ($required in @('build.sh', 'commit')) {
            $file = Join-Path $Root "cores/$name/$required"
            if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "$path names $name, which has no $file" }
        }
    }
    return $names
}

function Get-CoreDeviceFile {
  <#
  .SYNOPSIS
    Where one core's device library and its stamp live, as absolute paths.
  #>
    param([Parameter(Mandatory)][string]$Root, [Parameter(Mandatory)][string]$Name)

    $so = Join-Path $Root "vendor/${Name}_libretro.so"
    [pscustomobject]@{ Name = $Name; So = $so; Meta = "$so.meta" }
}

function Assert-CoreDeviceFiles {
  <#
  .SYNOPSIS
    Every manifest core has a device library and a stamp, both real and non-empty.
  .DESCRIPTION
    What the vendor directory is checked against before anything is assembled from it. The
    error names every file that is missing or empty, because "one core is missing" is not
    actionable and "vendor/gpsp_libretro.so is missing" is.
  #>
    param([Parameter(Mandatory)][string]$Root, [Parameter(Mandatory)][string[]]$Names)

    $files = @()
    $bad = @()
    foreach ($name in $Names) {
        $f = Get-CoreDeviceFile -Root $Root -Name $name
        foreach ($p in @($f.So, $f.Meta)) {
            if (-not (Test-Path -LiteralPath $p -PathType Leaf)) { $bad += "$p (missing)" }
            elseif ((Get-Item -LiteralPath $p).Length -eq 0) { $bad += "$p (empty)" }
        }
        $files += $f
    }
    if ($bad.Count -gt 0) {
        throw "the device cores the manifest asks for are not in vendor/:`n  $($bad -join "`n  ")`nrun build/cores.ps1 -DeviceOnly first"
    }
    return $files
}

function Assert-CoreTree {
  <#
  .SYNOPSIS
    An assembled tree holds exactly the manifest's cores and exactly their stamps.
  .DESCRIPTION
    Both halves of an assembly that has been assembled by hand are the same mistake: a core
    the manifest did not ask for, and a manifest core that did not make it. The licenses
    directory also carries the font licences, so only `*_libretro.so.meta` is counted there.
  #>
    param(
        [Parameter(Mandatory)][string]$CoresDir,
        [Parameter(Mandatory)][string]$LicensesDir,
        [Parameter(Mandatory)][string[]]$Names
    )

    $wantSo = @($Names | ForEach-Object { "${_}_libretro.so" })
    $wantMeta = @($Names | ForEach-Object { "${_}_libretro.so.meta" })

    $haveSo = @(Get-ChildItem -LiteralPath $CoresDir -File | Select-Object -ExpandProperty Name)
    $haveMeta = @(
        Get-ChildItem -LiteralPath $LicensesDir -File |
            Select-Object -ExpandProperty Name |
            Where-Object { $_ -like '*_libretro.so.meta' }
    )

    $problems = @()
    foreach ($pair in @(@{ Dir = $CoresDir; Want = $wantSo; Have = $haveSo }, @{ Dir = $LicensesDir; Want = $wantMeta; Have = $haveMeta })) {
        $extra = @($pair.Have | Where-Object { $_ -cnotin $pair.Want })
        $missing = @($pair.Want | Where-Object { $_ -cnotin $pair.Have })
        foreach ($name in $extra) { $problems += "$($pair.Dir)\$name is not in the manifest" }
        foreach ($name in $missing) { $problems += "$($pair.Dir)\$name is missing" }
        foreach ($name in @($pair.Want | Where-Object { $_ -cin $pair.Have })) {
            $file = Join-Path $pair.Dir $name
            if ((Get-Item -LiteralPath $file).Length -eq 0) { $problems += "$file is empty" }
        }
    }
    if ($problems.Count -gt 0) {
        throw "the assembled card is not the manifest's six cores:`n  $($problems -join "`n  ")"
    }
}

# --- the license and source bundle (task 101) -------------------------------------------

function Get-CoreLicenseName {
  <#
  .SYNOPSIS
    The file each upstream repository keeps its license in, at the top level of its tree.
  .DESCRIPTION
    The one thing about a core's license that cannot be discovered by looking: upstream may
    call the file COPYING, Copying, LICENSE or LICENSE.txt, and these six do not agree.
    Everything else about the bundle — the checkout directory, the pin, the repository URL — is
    read from the checkout and the build script rather than written down here.
  #>
    param([Parameter(Mandatory)][string]$Name)

    $names = [System.Collections.Generic.Dictionary[string, string]]::new([System.StringComparer]::Ordinal)
    $names['mgba'] = 'LICENSE'
    $names['gambatte'] = 'COPYING'
    $names['gpsp'] = 'COPYING'
    $names['fceumm'] = 'Copying'
    $names['snes9x'] = 'LICENSE'
    $names['genesis_plus_gx'] = 'LICENSE.txt'
    if (-not $names.ContainsKey($Name)) { throw "no upstream license file is recorded for '$Name'" }
    return $names[$Name]
}

function Get-CorePin {
  <#
  .SYNOPSIS
    The commit a core is pinned to, as the build scripts read it.
  #>
    param([Parameter(Mandatory)][string]$Root, [Parameter(Mandatory)][string]$Name)

    $file = Join-Path $Root "cores/$Name/commit"
    if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "no pin at $file" }
    $pin = (Get-Content -LiteralPath $file -Raw).Trim()
    if ($pin -notmatch '^[0-9a-f]{40}$') { throw "$file has '$pin', which is not a 40-character commit" }
    return $pin
}

function Get-FileSha256 {
    param([Parameter(Mandatory)][string]$Path)
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { throw "no file at $Path" }
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Assert-BytesAreTheSame {
    param([Parameter(Mandatory)][string]$What, [Parameter(Mandatory)][byte[]]$A, [Parameter(Mandatory)][byte[]]$B)
    if ($A.Length -ne $B.Length) { throw "$What differs: $($A.Length) bytes against $($B.Length)" }
    for ($i = 0; $i -lt $A.Length; $i++) {
        if ($A[$i] -ne $B[$i]) { throw "$What differs at byte $i" }
    }
}

function Get-PinnedFileBytes {
  <#
  .SYNOPSIS
    One file out of a pinned commit, byte for byte, without asking the working tree.
  .DESCRIPTION
    The working tree of a core checkout is dirty on purpose — the patches are applied to it and
    a Windows checkout may have had every line ending rewritten — so the only copy of a license
    that can be trusted is the git object. `cat-file` writes the object out untouched and it is
    taken off the process's raw standard output, because PowerShell's pipeline would decode it
    as text and rewrite it.
  #>
    param([Parameter(Mandatory)][string]$Checkout, [Parameter(Mandatory)][string]$Pin,
          [Parameter(Mandatory)][string]$Path)

    $start = New-Object System.Diagnostics.ProcessStartInfo
    $start.FileName = 'git'
    $start.Arguments = "cat-file blob `"$Pin`:$Path`""
    $start.WorkingDirectory = $Checkout
    $start.UseShellExecute = $false
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $process = [System.Diagnostics.Process]::Start($start)
    $memory = New-Object System.IO.MemoryStream
    $process.StandardOutput.BaseStream.CopyTo($memory)
    $errors = $process.StandardError.ReadToEnd()
    $process.WaitForExit()
    # The exit code is read before the handle is given back: a disposed Process does not have
    # one, and asking anyway is how a successful read turns into a failure.
    $exit = $process.ExitCode
    $process.Dispose()
    if ($exit -ne 0) { throw "git cat-file failed for $Checkout $Pin`:$Path`: $errors" }
    return $memory.ToArray()
}

function Export-GitArchive {
  <#
  .SYNOPSIS
    Write `git archive --format=zip` of a pinned tree (or of one path in it) to a file.
  .DESCRIPTION
    Through a process and its raw standard output rather than through the pipeline: PowerShell's
    pipeline decodes a native command's stdout as text, which would rewrite both line endings
    and non-ASCII bytes, and the whole point of this archive is that it is the committed bytes.
    The output path travels as a .NET property and the checkout as the process's working
    directory, so neither has to survive command-line quoting.

    `core.autocrlf` is forced off: git converts a working tree on checkout, and on a machine
    with it on it would convert the archive too, which would ship a source tarball that is not
    the commit it claims to be.
  #>
    param([Parameter(Mandatory)][string]$Checkout, [Parameter(Mandatory)][string]$Pin,
          [Parameter(Mandatory)][string]$Prefix, [Parameter(Mandatory)][string]$Destination,
          [string]$Pathspec)

    $arguments = "-c core.autocrlf=false -c core.eol=lf archive --format=zip --prefix=$Prefix $Pin"
    if ($Pathspec) { $arguments += " -- $Pathspec" }
    $start = New-Object System.Diagnostics.ProcessStartInfo
    $start.FileName = 'git'
    $start.Arguments = $arguments
    $start.WorkingDirectory = $Checkout
    $start.UseShellExecute = $false
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $process = [System.Diagnostics.Process]::Start($start)
    $file = [System.IO.File]::Create($Destination)
    try { $process.StandardOutput.BaseStream.CopyTo($file) } finally { $file.Dispose() }
    $errors = $process.StandardError.ReadToEnd()
    $process.WaitForExit()
    $exit = $process.ExitCode
    $process.Dispose()
    if ($exit -ne 0) { throw "git archive failed for $Checkout at $Pin`: $errors" }
}

function Get-CoreCheckout {
  <#
  .SYNOPSIS
    The one git checkout a core was built from, wherever the build script put it.
  .DESCRIPTION
    mGBA keeps its tree in `mgba/` and the other five in `src/`, which is exactly the sort of
    detail that must not be copied into a table here: a checkout is a directory with a .git in
    it, and there must be one.
  #>
    param([Parameter(Mandatory)][string]$Root, [Parameter(Mandatory)][string]$Name)

    $base = Join-Path $Root "target-device/cores/$Name"
    if (-not (Test-Path -LiteralPath $base)) {
        throw "no checkout for $Name under $base — run build/cores.ps1 -DeviceOnly first"
    }
    $found = @(Find-GitCheckout -Start $base -Depth 3)
    if ($found.Count -ne 1) {
        throw "$Name has $($found.Count) git checkouts under $base; exactly one is expected — run build/cores.ps1 -DeviceOnly"
    }
    return $found[0]
}

function Find-GitCheckout {
    param([Parameter(Mandatory)][string]$Start, [int]$Depth = 3)

    if ($Depth -lt 0) { return @() }
    if (Test-Path -LiteralPath (Join-Path $Start '.git')) { return @($Start) }
    $found = @()
    $children = @(Get-ChildItem -LiteralPath $Start -Directory -Force -ErrorAction SilentlyContinue |
        Where-Object { $_.Name -ne '.git' })
    foreach ($child in $children) {
        $found += @(Find-GitCheckout -Start $child.FullName -Depth ($Depth - 1))
    }
    return $found
}

function Get-CoreRepoUrl {
  <#
  .SYNOPSIS
    The repository a core's source comes from, from the script that fetches it.
  #>
    param([Parameter(Mandatory)][string]$Root, [Parameter(Mandatory)][string]$Name)

    $script = Join-Path $Root "cores/$Name/build.sh"
    if (-not (Test-Path -LiteralPath $script -PathType Leaf)) { throw "no build script at $script" }
    $match = [regex]::Match((Get-Content -LiteralPath $script -Raw), 'https?://[^\s")]+')
    if (-not $match.Success) { throw "$script names no repository URL" }
    # A URL with a variable still in it is a template in a log line, not where the source is
    # fetched from; the first plain one is the repository.
    $url = @([regex]::Matches((Get-Content -LiteralPath $script -Raw), 'https?://[^\s")]+') |
        Where-Object { -not $_.Value.Contains('$') })[0]
    if (-not $url) { throw "$script names no repository URL" }
    return $url.Value
}

function Read-SourceManifest {
  <#
  .SYNOPSIS
    Parse SOURCE-MANIFEST.txt into one block per core.
  .DESCRIPTION
    The format is the one the packager writes: `key=value` lines, a blank line between blocks,
    and repeated `patch=` lines inside a block. Anything else here is a corrupted manifest, and
    a manifest that cannot be read is not a hash to check against.
  #>
    param([Parameter(Mandatory)][string]$Path)

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { throw "no source manifest at $Path" }
    $text = [System.IO.File]::ReadAllText($Path)
    if ($text.Length -eq 0) { throw "$Path is empty" }
    if ($text.Contains("`r")) { throw "$Path is not LF-only" }
    if ($text[0] -eq [char]0xFEFF) { throw "$Path starts with a byte-order mark" }

    $cores = @{}
    $block = $null
    foreach ($line in $text.Split("`n")) {
        if ($line -eq '') { $block = $null; continue }
        $at = $line.IndexOf('=')
        if ($at -lt 1) { throw "$Path has a line that is not key=value: '$line'" }
        $key = $line.Substring(0, $at)
        $value = $line.Substring($at + 1)
        if ($key -eq 'core') {
            if ($cores.ContainsKey($value)) { throw "$Path names $value twice" }
            $block = @{ core = $value; patches = @() }
            $cores[$value] = $block
            continue
        }
        if ($null -eq $block) { throw "$Path has $key before any core=" }
        if ($key -eq 'patch') { $block.patches += , $value; continue }
        if ($block.ContainsKey($key)) { throw "$Path repeats $key for $($block.core)" }
        $block[$key] = $value
    }
    if ($cores.Count -eq 0) { throw "$Path names no cores" }
    return $cores
}

function Assert-SourceHashes {
  <#
  .SYNOPSIS
    Every hash in a source manifest matches the file it names, inside one bundle.
  .DESCRIPTION
    Used on the bundle the packager just wrote and again on the copy inside the assembled card,
    so a truncated archive or a manifest that describes a different build is caught before
    anything is zipped. `Dir` is the directory holding SOURCE-MANIFEST.txt, archives/ and
    recipes/ — the bundle root in both cases.
  #>
    param([Parameter(Mandatory)][string]$Dir, [Parameter(Mandatory)][string[]]$Names)

    $manifest = Read-SourceManifest -Path (Join-Path $Dir 'SOURCE-MANIFEST.txt')
    foreach ($name in $Names) {
        if (-not $manifest.ContainsKey($name)) { throw "SOURCE-MANIFEST.txt has no block for $name" }
        $block = $manifest[$name]
        foreach ($key in @('repo', 'pin', 'archive', 'archive_sha256', 'license', 'recipe')) {
            if (-not $block.ContainsKey($key)) { throw "the $name block has no $key" }
        }
        foreach ($key in @('archive', 'license', 'recipe')) {
            if ($block[$key].Contains('\') -or $block[$key] -match '^([A-Za-z]:|/)') {
                throw "the $name $key is not a relative path: $($block[$key])"
            }
        }

        $archive = Join-Path $Dir $block.archive
        if (-not (Test-Path -LiteralPath $archive -PathType Leaf)) { throw "no archive at $archive" }
        if ((Get-Item -LiteralPath $archive).Length -eq 0) { throw "$archive is empty" }
        $actual = Get-FileSha256 -Path $archive
        if ($actual -cne $block.archive_sha256) {
            throw "$archive hashes to $actual, not to the $($block.archive_sha256) the manifest records"
        }

        # The pinned source must carry the license the card also ships beside it — the same
        # bytes, not merely a file of that name: a line-ending conversion in the archive would
        # make it something other than the commit it claims to be.
        $licenseFile = Split-Path -Leaf $block.license
        $inside = "$name-$($block.pin)/$licenseFile"
        $pinnedLicense = Join-Path $Dir $block.license
        if (-not (Test-Path -LiteralPath $pinnedLicense -PathType Leaf)) { throw "no license at $pinnedLicense" }
        $zip = [System.IO.Compression.ZipFile]::OpenRead($archive)
        try {
            $entry = $zip.Entries | Where-Object { $_.FullName -ceq $inside } | Select-Object -First 1
            if (-not $entry) { throw "$archive has no $inside" }
            $stream = $entry.Open()
            try {
                $memory = New-Object System.IO.MemoryStream
                $stream.CopyTo($memory)
                Assert-BytesAreTheSame -What "$inside in $archive against $($block.license)" `
                    -A $memory.ToArray() -B ([System.IO.File]::ReadAllBytes($pinnedLicense))
            } finally { $stream.Dispose() }
        } finally { $zip.Dispose() }

        foreach ($patch in $block.patches) {
            $parts = $patch.Split(' ')
            if ($parts.Count -ne 2 -or -not $parts[1].StartsWith('sha256:')) {
                throw "the $name patch line is not 'file sha256:<hex>': $patch"
            }
            $file = Join-Path (Join-Path $Dir $block.recipe) $parts[0]
            if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "no recipe patch at $file" }
            $actualPatch = Get-FileSha256 -Path $file
            if ($actualPatch -cne $parts[1].Substring(7)) {
                throw "$file hashes to $actualPatch, not to the $($parts[1]) the manifest records"
            }
        }
    }
    return $manifest
}

function Get-SubDirectoryNames {
    param([Parameter(Mandatory)][string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) { return @() }
    return @(Get-ChildItem -LiteralPath $Path -Directory -Force | Select-Object -ExpandProperty Name)
}

function Get-ChildFileNames {
    param([Parameter(Mandatory)][string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) { return @() }
    return @(Get-ChildItem -LiteralPath $Path -File -Force | Select-Object -ExpandProperty Name)
}

function Assert-SameNameSet {
    param([Parameter(Mandatory)][string]$What, [Parameter(Mandatory)][string[]]$Want, [Parameter(Mandatory)][string[]]$Have)

    $extra = @($Have | Where-Object { $_ -cnotin $Want })
    $missing = @($Want | Where-Object { $_ -cnotin $Have })
    if ($extra.Count -gt 0) { throw "$What holds what the manifest does not: $($extra -join ', ')" }
    if ($missing.Count -gt 0) { throw "$What is missing: $($missing -join ', ')" }
}

function Assert-SourceBundle {
  <#
  .SYNOPSIS
    A source bundle is the pinned source, its licenses and the recipe, and it is complete.
  .DESCRIPTION
    The check before anything is assembled from the bundle: the repository's own notices and
    tracked licenses are there, every checkout is at its pin, the tracked license is the pinned
    blob byte for byte, the bundle holds exactly the manifest's six archives/licenses/recipes,
    every hash in its manifest matches its files, and the recipe is the repository's recipe.
  #>
    param([Parameter(Mandatory)][string]$Root, [Parameter(Mandatory)][string]$Bundle,
          [Parameter(Mandatory)][string[]]$Names)

    foreach ($rel in @('LICENSE', 'CORE-NOTICES.md', 'licenses/upstream-slot/LICENSE')) {
        $file = Join-Path $Root $rel
        if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "no $rel in the repository" }
        if ((Get-Item -LiteralPath $file).Length -eq 0) { throw "$rel is empty" }
    }

    $manifest = Assert-SourceHashes -Dir $Bundle -Names $Names
    Assert-SameNameSet -What "$Bundle/archives" -Want @($Names | ForEach-Object { "${_}-$(Get-CorePin -Root $Root -Name $_).zip" }) `
        -Have (Get-ChildFileNames -Path (Join-Path $Bundle 'archives'))
    Assert-SameNameSet -What "$Bundle/licenses" -Want $Names `
        -Have (Get-SubDirectoryNames -Path (Join-Path $Bundle 'licenses'))
    Assert-SameNameSet -What "$Bundle/recipes" -Want (@($Names) + @('common.sh')) `
        -Have (@(Get-ChildFileNames -Path (Join-Path $Bundle 'recipes')) + @(Get-SubDirectoryNames -Path (Join-Path $Bundle 'recipes')))

    foreach ($name in $Names) {
        $pin = Get-CorePin -Root $Root -Name $name
        $license = Get-CoreLicenseName -Name $name
        $checkout = Get-CoreCheckout -Root $Root -Name $name
        $head = (& git -C $checkout rev-parse HEAD).Trim()
        if ($LASTEXITCODE -ne 0) { throw "git rev-parse failed in $checkout" }
        if ($head -cne $pin) { throw "$name is checked out at $head, not at its pin $pin" }
        & git -C $checkout cat-file -e "$pin^{commit}"
        if ($LASTEXITCODE -ne 0) { throw "$name's pin $pin is not in $checkout" }

        $pinned = Get-PinnedFileBytes -Checkout $checkout -Pin $pin -Path $license
        $tracked = Join-Path $Root "licenses/cores/$name/$license"
        if (-not (Test-Path -LiteralPath $tracked -PathType Leaf)) { throw "no tracked license at $tracked" }
        Assert-BytesAreTheSame -What "$tracked against $pin`:$license" -A ([System.IO.File]::ReadAllBytes($tracked)) -B $pinned
        $generated = Join-Path $Bundle "licenses/$name/$license"
        Assert-SameNameSet -What "$Bundle/licenses/$name" -Want @($license) -Have (Get-ChildFileNames -Path (Join-Path $Bundle "licenses/$name"))
        Assert-BytesAreTheSame -What "$generated against the pinned $license" -A ([System.IO.File]::ReadAllBytes($generated)) -B $pinned

        if ($manifest[$name].repo -cne (Get-CoreRepoUrl -Root $Root -Name $name)) {
            throw "$name's manifest repository and its build script disagree"
        }
        if ($manifest[$name].pin -cne $pin) { throw "$name's manifest pin is not the repository's" }
        if ($manifest[$name].license -cne "licenses/$name/$license") { throw "$name's manifest license path is not the one the bundle holds" }

        foreach ($file in @('commit', 'build.sh')) {
            $recipe = Join-Path $Bundle "recipes/$name/$file"
            $source = Join-Path $Root "cores/$name/$file"
            if (-not (Test-Path -LiteralPath $recipe -PathType Leaf)) { throw "no recipe file at $recipe" }
            Assert-BytesAreTheSame -What "$recipe against the repository's" `
                -A ([System.IO.File]::ReadAllBytes($recipe)) -B ([System.IO.File]::ReadAllBytes($source))
        }
        $patches = @(Get-ChildItem -LiteralPath (Join-Path $Root "cores/$name") -Filter '*.patch' -File | Select-Object -ExpandProperty Name)
        Assert-SameNameSet -What "$Bundle/recipes/$name" -Want (@('commit', 'build.sh') + $patches) `
            -Have (Get-ChildFileNames -Path (Join-Path $Bundle "recipes/$name"))
        if ($manifest[$name].patches.Count -ne $patches.Count) {
            throw "$name's manifest lists $($manifest[$name].patches.Count) patches and the repository has $($patches.Count)"
        }
    }
    $common = Join-Path $Bundle 'recipes/common.sh'
    if (-not (Test-Path -LiteralPath $common -PathType Leaf)) { throw "no recipe common.sh at $common" }
    Assert-BytesAreTheSame -What "$common against the repository's" `
        -A ([System.IO.File]::ReadAllBytes($common)) -B ([System.IO.File]::ReadAllBytes((Join-Path $Root 'cores/common.sh')))

    # Nothing in a manifest may tie the bundle to the machine that made it. The drive pattern
    # is written so a URL's `https://` is not mistaken for `s:/`, and a backslash is refused
    # outright: every path in the bundle is relative and forward-slashed.
    $text = [System.IO.File]::ReadAllText((Join-Path $Bundle 'SOURCE-MANIFEST.txt'))
    if ($text.Contains($Root)) { throw "SOURCE-MANIFEST.txt names the absolute repository root" }
    if ($text.Contains('\')) { throw "SOURCE-MANIFEST.txt is not forward-slashed" }
    if ($text -match '[A-Za-z]:[\\/](?!/)') { throw "SOURCE-MANIFEST.txt names a drive path" }
}

function Assert-LicensesTree {
  <#
  .SYNOPSIS
    An assembled System/licenses holds the notices, the licenses and the source bundle.
  .DESCRIPTION
    The postflight for the card tree: the same preflight checks are made from the tree itself,
    so a copy that dropped a directory, emptied a license or truncated an archive fails before
    anything is zipped. The tracked licences and the generated ones are the same pinned blobs,
    and that is checked here rather than assumed from the copy.
  #>
    param([Parameter(Mandatory)][string]$LicensesDir, [Parameter(Mandatory)][string[]]$Names)

    foreach ($rel in @('SLOT2-LICENSE', 'CORE-NOTICES.md', 'upstream-slot/LICENSE')) {
        $file = Join-Path $LicensesDir $rel
        if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { throw "no $rel in $LicensesDir" }
        if ((Get-Item -LiteralPath $file).Length -eq 0) { throw "$file is empty" }
    }

    Assert-SameNameSet -What "$LicensesDir/cores" -Want $Names -Have (Get-SubDirectoryNames -Path (Join-Path $LicensesDir 'cores'))
    $sources = Join-Path $LicensesDir 'sources'
    Assert-SameNameSet -What "$sources/licenses" -Want $Names -Have (Get-SubDirectoryNames -Path (Join-Path $sources 'licenses'))
    Assert-SameNameSet -What "$sources/recipes" -Want (@($Names) + @('common.sh')) `
        -Have (@(Get-ChildFileNames -Path (Join-Path $sources 'recipes')) + @(Get-SubDirectoryNames -Path (Join-Path $sources 'recipes')))
    $manifest = Assert-SourceHashes -Dir $sources -Names $Names
    Assert-SameNameSet -What "$sources/archives" -Want @($Names | ForEach-Object { "${_}-$($manifest[$_].pin).zip" }) `
        -Have (Get-ChildFileNames -Path (Join-Path $sources 'archives'))
    foreach ($name in $Names) {
        $tracked = Join-Path $LicensesDir "cores/$name"
        $generated = Join-Path $sources "licenses/$name"
        $trackedFiles = Get-ChildFileNames -Path $tracked
        $generatedFiles = Get-ChildFileNames -Path $generated
        Assert-SameNameSet -What $tracked -Want $generatedFiles -Have $trackedFiles
        foreach ($file in $trackedFiles) {
            Assert-BytesAreTheSame -What "$tracked/$file against $generated/$file" `
                -A ([System.IO.File]::ReadAllBytes((Join-Path $tracked $file))) `
                -B ([System.IO.File]::ReadAllBytes((Join-Path $generated $file)))
        }
    }
}
