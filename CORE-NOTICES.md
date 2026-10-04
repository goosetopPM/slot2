# Core notices and corresponding source

SLOT2 is a libretro frontend for a handheld, and the six emulator cores it ships are other
people's work. This file says what is bundled with them and where the original texts are. It is
**not legal advice and it does not replace any license text**: where this file and an original
license disagree, the original license governs.

On a card, everything described here lives under `System/licenses/`:

```text
System/licenses/
├─ SLOT2-LICENSE                the MIT license below, for SLOT2's own code
├─ CORE-NOTICES.md              this file
├─ upstream-slot/LICENSE        the original project's MIT license, verbatim
├─ fonts/                       Open Sans and Noto Sans KR licenses (OFL)
├─ cores/<name>/<license>       each core's own license, verbatim from its pinned commit
├─ sources/                     pristine pinned source and SLOT2's build recipe
│  ├─ SOURCE-MANIFEST.txt       pins, archive hashes, license paths, patch hashes
│  ├─ archives/<name>-<pin>.zip `git archive` of the pinned commit
│  ├─ licenses/<name>/<license> the same core licences, as packed
│  └─ recipes/                  common.sh, and each core's commit/build.sh/*.patch
└─ <name>_libretro.so.meta      the stamp the binary was built with (repo, pin, flags)
```

## SLOT2's own code

The SLOT2 frontend — everything in `crates/`, `build/`, `assets/` that is not listed as ported
below — is MIT licensed. The full text is `LICENSE` in the repository and
`System/licenses/SLOT2-LICENSE` on the card:

```text
Copyright (c) 2026 SLOT2 contributors
```

## From the original project

Parts of SLOT2 are ported from [brandonkowalski/slot](https://github.com/brandonkowalski/slot)
(MIT), which is the project this frontend reworks. The original license is bundled verbatim as
`licenses/upstream-slot/LICENSE` (and `System/licenses/upstream-slot/LICENSE`); it is not
summarised or edited here. Which files came from it, and under what terms, is recorded per asset
in the repository's provenance documents:

| what was ported | recorded in |
|---|---|
| cartridge and slot artwork (`assets/skins/*.svg`) | `assets/skins/PROVENANCE.md` |
| the slot's own noises (`assets/sfx/*.pcm`) | `assets/sfx/PROVENANCE.md` |
| the emulator host and the frontend's structure | file headers and `docs/DESIGN.md` |

## The six cores

Each row's license is bundled verbatim from the pinned commit — the file named below, as it is
in the upstream tree, with no reformatting. The source archive and the recipe together are the
corresponding source for the binary SLOT2 ships (see the next section).

| core (display name) | repository | pin | bundled license | source archive | recipe |
|---|---|---|---|---|---|
| mGBA | https://github.com/libretro/mgba | `e31759b24e7a4e3899285ff720d7b573ac328ae7` | `cores/mgba/LICENSE` | `sources/archives/mgba-e31759b24e7a4e3899285ff720d7b573ac328ae7.zip` | `sources/recipes/mgba/` |
| Gambatte | https://github.com/libretro/gambatte-libretro | `d9d6cd06382d1ced30de34d56d3609452323dab1` | `cores/gambatte/COPYING` | `sources/archives/gambatte-d9d6cd06382d1ced30de34d56d3609452323dab1.zip` | `sources/recipes/gambatte/` |
| gpSP | https://github.com/libretro/gpsp | `5819380c2ffb0900219d700a382ee68c464ebb99` | `cores/gpsp/COPYING` | `sources/archives/gpsp-5819380c2ffb0900219d700a382ee68c464ebb99.zip` | `sources/recipes/gpsp/` |
| FCEUmm | https://github.com/libretro/libretro-fceumm | `236ccdfc911e84c60fea6b9d0699c2d440a8de14` | `cores/fceumm/Copying` | `sources/archives/fceumm-236ccdfc911e84c60fea6b9d0699c2d440a8de14.zip` | `sources/recipes/fceumm/` |
| Snes9x | https://github.com/libretro/snes9x | `fae2fea08f74180759ef540ee94259213f503480` | `cores/snes9x/LICENSE` | `sources/archives/snes9x-fae2fea08f74180759ef540ee94259213f503480.zip` | `sources/recipes/snes9x/` |
| Genesis Plus GX | https://github.com/libretro/Genesis-Plus-GX | `c2838c7dc4236fc2fe94e5dbd08b41486067918e` | `cores/genesis_plus_gx/LICENSE.txt` | `sources/archives/genesis_plus_gx-c2838c7dc4236fc2fe94e5dbd08b41486067918e.zip` | `sources/recipes/genesis_plus_gx/` |

What each of those files is, for identification only — the bundled text is the authority:

- **mGBA** — Mozilla Public License, version 2.0 (the file begins `Mozilla Public License
  Version 2.0`).
- **Gambatte**, **gpSP**, **FCEUmm** — the GNU General Public License, version 2, in full (each
  file is the complete GPL-2.0 text beginning `GNU GENERAL PUBLIC LICENSE / Version 2, June
  1991`).
- **Snes9x** — Snes9x's own license, not an SPDX-listed one. Its terms include
  `Permission to use, copy, modify and/or distribute Snes9x in both binary and source form, for
  non-commercial purposes, is hereby granted without fee,` and `Snes9x is freeware for PERSONAL
  USE only. Commercial users should seek permission of the copyright holders first.` Some
  copyright holders listed in the file say `(Under no circumstances will commercial rights be
  given)`.
- **Genesis Plus GX** — Genesis Plus GX's own license, not an SPDX-listed one. Its terms
  include `Redistributions may not be sold, nor may they be used in a commercial product or
  activity.` The file also carries the licenses of the third-party code it bundles.

> **Snes9x and Genesis Plus GX: not for sale or commercial use.** The pinned originals state,
> in the terms quoted above, that their redistributions may not be sold and may not be used in a
> commercial product or activity, and that commercial users must ask the copyright holders
> first. SLOT2 ships those two cores on that basis only: this project does not grant you any
> commercial right to them, and it cannot. If you intend to sell a device or a service with
> these cores on it, read `System/licenses/cores/snes9x/LICENSE` and
> `System/licenses/cores/genesis_plus_gx/LICENSE.txt` and obtain permission from the respective
> copyright holders. Read every core's bundled license before you redistribute anything.

## Corresponding source

Each `sources/archives/<name>-<pin>.zip` is `git archive` of exactly the pinned commit: the
upstream tree as committed, including the top-level license file, without build output, without
`.git`, without local test ROMs and without any patch applied. `sources/recipes/` holds SLOT2's
side of the build byte for byte — `cores/common.sh`, and each core's `cores/<name>/commit`,
`cores/<name>/build.sh` and `cores/<name>/*.patch`. The patch beside a core is applied to that
archive to produce what SLOT2 builds; the resulting binary's stamp (repo, pin, make arguments,
toolchain triple, patch hashes) is `System/licenses/<name>_libretro.so.meta`, and
`sources/SOURCE-MANIFEST.txt` records the same pin with the archive's SHA-256 and each patch's
SHA-256. To rebuild a shipped core: check out the pin from the repository in the manifest, apply
the patches in the recipe, and run the recipe's `build.sh` — that is the source the binary
corresponds to.

## Fonts

`System/licenses/fonts/` holds the SIL Open Font License texts for the two fonts the UI draws
with: Open Sans (`assets/fonts/OpenSans-Regular.ttf`, `OpenSans-OFL.txt`) and Noto Sans KR
(`assets/fonts/NotoSansKR-Regular.otf`, `NotoSansKR-OFL.txt`).

## AI-assisted development

Parts of SLOT2 were implemented and reviewed with AI coding tools: Claude Code, OpenAI Codex,
GajaeCode/OpenCodex, and models served through B.AI. They were used as tools by the maintainers;
the project's copyright and licensing are unchanged by that, and the maintainers are responsible
for what the project ships, including the notices in this file.

## Rust runtime dependencies

`System/licenses/rust/` holds the third-party notices for the Rust packages the device binary
links: the transitive **normal** dependency closure of package `slot2` for
target `aarch64-unknown-linux-gnu`, with default features off and feature `device` on. SLOT2's
own workspace crates are not listed there — the root MIT license above covers them. System
libraries and the emulator cores are not in this set either; the cores have their own section.

The tree is generated by `build/package-rust-notices.ps1` from Cargo's own resolution, offline,
and is checked again when `build/dist-device.ps1` assembles the card:

```text
System/licenses/rust/THIRD-PARTY-RUST.md   the inventory, package by package
System/licenses/rust/RUST-SBOM.json        slot2-rust-sbom-v1, the machine-readable inventory
System/licenses/rust/RUST-MANIFEST.txt     SHA-256 and byte length of every other file
System/licenses/rust/packages/<key>/       PACKAGE.txt and the crate's own license files
```

Where a crate package contains license or notice files, they are copied byte for byte and its
entry is marked `packaged-text`. A **`declared-only`** entry is one whose crate package declares a
license expression and supplied no license or notice text at all: nothing was invented,
downloaded or borrowed for it, and the declaration is repeated exactly as Cargo reports it,
together with the upstream repository the crate came from.

**The copied original text governs.** This section, `THIRD-PARTY-RUST.md` and every `PACKAGE.txt`
are inventories, not legal advice, and they do not replace the license of any package. Where a
declaration is an SPDX expression such as `MIT OR Apache-2.0`, both sides are carried as declared
and no side of it is chosen here.

## Still to come

The release workflow that publishes the card tree (`.github/workflows/release.yml`) and the public
README are separate work and are not in this bundle yet.
