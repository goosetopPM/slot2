# SLOT2

SLOT2 is a cartridge-style frontend for [BaseOS](https://github.com/pvaibhav/BaseOS) on Anbernic H700
handhelds. It plays GB, GBC, GBA, NES, SNES, MD and SMS games with in-process libretro cores, on a shelf of
cartridges. It began from [brandonkowalski/slot](https://github.com/brandonkowalski/slot) (MIT) and was
rebuilt around it.
Note on the name: "SLOT2" is named as a homage to the Nintendo DS Slot-2 (GBA slot),
not to claim it is better or superior to the original slot.

**한국어 문서: [README.ko.md](README.ko.md)**

## 1. Overview and project status

SLOT2 is a **pre-release** project. No public release has been published, no tag release has been executed,
and no GitHub download page exists yet; a release asset is the file layout described below, which the
project can build and check locally.

- The development baseline, and the only device physically tested so far, is the **RG SP**. The other
  device profiles below are implemented from BaseOS's device data, but they are **not fully
  hardware-accepted** yet.
- The implementation and the automated local validation are extensive: the workspace builds, tests, lints
  and formats in CI, the card tree is validated piece by piece before it is packaged, and the packaging and
  publication steps have offline checks of their own.
- Still pending, and not claimed anywhere below: a hosted tag-release execution, a fresh-card first-user
  walkthrough, and broad physical-device acceptance.

What SLOT2 does today, in short:

- a cartridge shelf per platform, with cart and port artwork;
- per-game core choice where a platform has more than one shipped core (GB, GBC, GBA);
- game launch that resumes from the latest resume or numbered state, or starts fresh on request;
- save data, numbered states with thumbnails, quick save and quick load, and a state switcher that can
  load, delete and undo a delete;
- rewind and fast-forward;
- cheats read from a game's own RetroArch-format cheat file;
- the display scale rows (platform default, integer, aspect-fit, fill), the built-in shader presets,
  overscan controls on the platforms that crop, and an overlay on/off choice;
- English and Korean built in, card language packs, a time zone setting, and volume;
- a PC preview window for development.

Deliberately not there yet, and shown as unavailable in the UI rather than faked:

- **Sync** (Wi-Fi/SFTP transfer to a desktop app): no backend. The Sync row in the settings menu cannot be
  opened.
- **Brightness** and **blue-light control**: no backend. Their rows in the device menu appear as
  unavailable and cannot be selected.
- **Custom shader files**: only the built-in presets; user GLSL files are not supported.
- **Compressed ROMs** (`.zip`, `.7z`, `.gz` and the like): not supported.
- **RG28XX**: out of scope for this project.
- The **Display defaults** and **Boot logo** rows in the settings menu have no screen behind them yet.

## 2. Supported devices, platforms, and cores

BaseOS names the device it booted in `/etc/baseos-release` as `BASEOS_TARGET=<id>`; SLOT2 reads that id and
picks a profile. These are the profiles that exist:

| `BASEOS_TARGET` | Device | Panel | Lid | Sticks |
| --- | --- | --- | --- | --- |
| `rgsp` | RG SP (development baseline, physically tested) | 720x480 | yes | no |
| `rg34xx` | RG34XX | 720x480 | no | no |
| `rg34xxsp` | RG34XXSP | 720x480 | yes | no |
| `rg35xxsp` | RG35XXSP | 640x480 | yes | no |
| `rg35xxplus` | RG35XX Plus | 640x480 | no | no |
| `rg35xxh` | RG35XXH | 640x480 | no | yes |
| `rg35xxpro` | RG35XX Pro | 640x480 | no | no |
| `rg40xxh` | RG40XXH | 640x480 | no | yes |
| `rg40xxv` | RG40XXV | 640x480 | no | yes |
| `rgcubexx` | RG CubeXX | 720x720 | no | yes |

An unrecognized `BASEOS_TARGET` falls back to a 640x480 panel with no lid and no sticks, and is reported as
unknown in the boot log. That fallback is a safe default, **not** a promise that every H700 handheld works:
the table above is the supported set, and RG28XX is out of scope.

Platforms and their cores. Every core runs inside the frontend process; the alternate core is a per-game
choice in that game's menu.

| Platform | ROM folder | Extensions | Default core | Alternate core |
| --- | --- | --- | --- | --- |
| GB | `Games/GB` | `.gb` | mGBA | Gambatte |
| GBC | `Games/GBC` | `.gbc` | mGBA | Gambatte |
| GBA | `Games/GBA` | `.gba` | mGBA | gpSP |
| NES | `Games/NES` | `.nes` | FCEUmm | - |
| SNES | `Games/SNES` | `.sfc`, `.smc` | Snes9x | - |
| MD | `Games/MD` | `.md`, `.gen`, `.bin` | Genesis Plus GX | - |
| SMS | `Games/SMS` | `.sms` | Genesis Plus GX | - |

The shipped core set is exactly those six and is listed in [cores/required.txt](cores/required.txt). The
frontend looks for them in `System/cores/`; if one of them is missing from the card, the launch fails
rather than starting a game it cannot run.

ROM discovery is deliberately plain:

- the folder and the file extension decide the platform; file contents are never inspected;
- only **regular files directly inside** a platform folder are seen - a nested folder is not searched;
- only the **last** extension is matched, case-insensitively, and the rest of the name - spaces, brackets,
  dots and the script itself - is kept as it is;
- compressed ROMs are not opened or unpacked.

**No ROMs and no BIOS files are included** with SLOT2. You supply your own. The current cores work without
a BIOS; if you have one from a lawful source you may put it directly in `BIOS/`, which is where the
frontend looks for one. Do not download BIOS files from anywhere you would not download a game from.

## 3. Fresh installation

> **This is a fresh installation guide only.** Replacing `System/` on a card that already has an original
> slot installation is a **different procedure**, written down in [docs/MIGRATION.md](docs/MIGRATION.md).
> Do not use these steps as an upgrade guide for a card you already use. **Back up your card first**: SLOT2
> creates and writes save data, states and settings, and an installation that goes wrong over an existing
> card can lose what was on it.

SLOT2 does not image a card and does not touch BaseOS. Install BaseOS on TF1 by following the
[BaseOS documentation](https://github.com/pvaibhav/BaseOS), then put SLOT2 on top of it as below.

A SLOT2 release asset is one zip whose **root is `System/`**, plus a `.sha256` sidecar next to it that
holds the zip's SHA-256. No public release has been published yet, so there is no download page to link to;
if you have a zip, this is what it should look like, and the sidecar is how you check it arrived intact.

Check the zip before extracting it. `<zip>` stands for the real file name (for example `slot2-<tag>.zip`);
substitute it, and open the sidecar in a text editor to read the expected digest:

```powershell
# Windows PowerShell: compare the printed hash with the digest inside <zip>.sha256
(Get-FileHash <zip> -Algorithm SHA256).Hash
```

```sh
# Linux: the sidecar is in sha256sum format, so this checks it for you
sha256sum -c <zip>.sha256

# macOS: print the hash and compare it with the digest inside <zip>.sha256
shasum -a 256 <zip>
```

One card (BaseOS and SLOT2 on the same TF card):

1. Install BaseOS on TF1 following its documentation.
2. Mount the BaseOS data/card-root volume on your PC.
3. Extract the **contents** of the SLOT2 zip at that root, so that the executable lands exactly at
   `<card root>/System/frontend`. If you end up with `<card root>/SLOT2/System/frontend`, the nesting is one
   level too deep and BaseOS will not find it; move the `System` folder up.
4. Copy your ROMs into the platform folders from the table above, then eject the card safely.

Two cards (BaseOS on TF1, SLOT2 and your ROMs on TF2):

1. Keep BaseOS on TF1.
2. Prepare TF2 with a filesystem BaseOS supports. Formatting policy is BaseOS's business, not SLOT2's; no
   particular formatter or label is required by this frontend and none is claimed here.
3. Extract the zip contents to the root of TF2, so that TF2 contains `System/frontend`.
4. Put your ROMs in the `Games/<PLATFORM>` folders on TF2.
5. Insert both cards. BaseOS prefers the inserted TF2 data card and otherwise uses the TF1 data partition;
   either way the frontend is given that volume as the card root, so the layout is identical.

What to expect on the first start, conservatively: BaseOS runs `System/frontend` with the card root as the
working directory; SLOT2 creates the user-data folders that are missing; and the shelf shows the supported
ROMs it found in the direct platform folders. That is the designed behavior - a fresh-card first-user
walkthrough has **not** been run, so treat the first boot as an installation you are verifying, not as a
walkthrough that has already been accepted.

## 4. ROM folders and card layout

```text
BIOS/
Games/{GB,GBC,GBA,NES,SNES,MD,SMS}/
Labels/{GB,GBC,GBA,NES,SNES,MD,SMS}/
Saves/
States/
System/frontend
System/cores/
System/Fonts/
System/Lang/
System/cheats/{GB,GBC,GBA,NES,SNES,MD,SMS}/
System/licenses/
Wallpapers/
```

Only `System/` comes from the release zip. Everything else is yours or is created on first use. The save
data, states, state thumbnails and settings that `slot2-store` manages are written through its
temporary-file-and-rename path.

The **stem** of a ROM file name is the key for everything the card keeps for that game: its label
(`Labels/<PLATFORM>/<stem>.png`), its save data (`Saves/...`), its states (`States/...`), its per-game
settings and its cheats. That is why renaming a ROM can look like losing its saves, labels and states: the
new name is a different stem. If you rename a ROM, rename the related files with it, or restore them from a
backup. SLOT2 does not migrate them for you.

## 5. Device controls

Shelf (the cartridge list):

| Input | Action |
| --- | --- |
| Left / Right | select a cartridge |
| L1 / R1 | change platform |
| A (tap) | resume from the latest resume or numbered state, otherwise start fresh |
| A (hold) | start fresh |
| MENU (tap) | settings |
| MENU (hold) | power menu |

Playing:

| Input | Action |
| --- | --- |
| MENU (tap) | in-game menu |
| MENU (hold) | eject to the shelf |
| SELECT + R1 | quick-save the next numbered state |
| SELECT + L1 | quick-load the latest numbered state |
| R2 (hold) | fast-forward while held |
| R2 (double tap) | toggle the fast-forward latch |
| L2 (hold) | rewind; rewind outranks fast-forward while both are held |

The in-game menu holds Continue, Save state, Cheats, Display, Core, Device and Eject. In menus generally:
the D-pad navigates, A confirms, and B or MENU goes back. The state switcher (Save state) is the exception:
Left and Right select, A loads, X deletes, Y puts a deleted state back (the undo lasts 30 seconds), and B or
MENU goes back. The device's own volume keys change the volume on any screen, including while a game is
running.

The Core row is only useful where a platform has more than one shipped core - GB, GBC and GBA; on the other
platforms it lists the single core that runs them. The Device row's brightness and blue-light entries have
no backend yet, so they show as unavailable.

## 6. PC preview and host keyboard controls

The frontend has a desktop backend for development: it opens a 720x480 window and runs the same UI and
cores.

```sh
cargo run -p slot2
```

It reads its card root from the `SLOT2_ROOT` environment variable, and falls back to `./sdcard` relative to
the directory you run it from. Point `SLOT2_ROOT` at a card root (or run from a directory that contains a
`sdcard` folder) to try the shelf against real files.

| Key | Button |
| --- | --- |
| Arrow keys | D-pad |
| X / Z | device A / B |
| S / A | device X / Y |
| Q / W | L1 / R1 |
| 1 / 2 | L2 / R2 |
| Enter | Start |
| Right Shift | Select |
| M or Backspace | MENU |
| Page Up / Page Down | volume up / down |
| P | power |
| Escape | quit the preview |

The preview is a development convenience, not a device: it does **not** validate audio output on the
handheld, the device's own controls, the GPU, power management, card mounting or performance. Those need
the real hardware.

## 7. Troubleshooting / FAQ

- **The shelf is empty.** Check that the ROM is a regular, uncompressed file directly inside the right
  platform folder (`Games/GB`, `Games/GBA`, ...) with a supported extension, and that the card root the
  frontend sees is the one you copied to.
- **The frontend does not start.** Check that the executable is exactly `System/frontend` under the card
  root - not `SLOT2/System/frontend` - and that the whole release tree was extracted, not just the binary.
- **A game says a core is missing.** Extract the complete release again; `System/cores/` has to contain the
  shipped core set. SLOT2 will not start a game with a core it cannot find.
- **A GB, GBC or GBA game misbehaves.** Open the in-game menu, choose Core, and try the alternate core for
  that platform (Gambatte for GB/GBC, gpSP for GBA). Some games genuinely behave differently per core.
- **Saves or states look lost after renaming a ROM.** They are keyed by the ROM's stem, so put the old name
  back (or rename the related files) and restore from your backup. There is no automatic migration.
- **A setting is unavailable.** Sync, brightness and blue-light control have no completed backend yet, so
  their rows cannot be used. Custom shader files are not supported either; the built-in presets are.
- **Diagnostics.** `/tmp/frontend.log` is where BaseOS keeps the frontend's own output, and it is volatile.
  The card copy of the boot report is `System/slot2-diag.txt`, which a card reader can show without a
  terminal on the device.

## 8. Translation and project documentation

The UI ships with English and Korean. A card can override either, or add a language, with a Fluent pack at
`System/Lang/<code>.ftl`; a pack may translate a few strings or all of them. **Contributions are welcome,
and the whole process is written down in [docs/TRANSLATING.md](docs/TRANSLATING.md)** - rules, the
particle helper Korean needs, and how to check a pack.

- [docs/TRANSLATING.md](docs/TRANSLATING.md) - translating the UI and language packs.
- [docs/MIGRATION.md](docs/MIGRATION.md) - moving a card that already holds an original slot installation.
- [docs/DESIGN.md](docs/DESIGN.md) - architecture, contracts, crates and the card layout.
- [docs/DECISIONS.md](docs/DECISIONS.md) - the decisions behind both, with their reasons.
- [docs/MILESTONES.md](docs/MILESTONES.md) - what is done, what is in progress, and what is left.

The issue forms for bug reports, device compatibility and translation work are in
[.github/ISSUE_TEMPLATE/](.github/ISSUE_TEMPLATE/), and the migration procedure for a card that already holds
an original slot installation is in [docs/MIGRATION.md](docs/MIGRATION.md).

## 9. Licenses, upstream credit, and legal boundary

SLOT2's own frontend - the crates, build scripts and assets that are not ported from the original project -
is **MIT licensed**; the full text is [LICENSE](LICENSE) in this repository and `System/licenses/SLOT2-LICENSE`
on a card.

SLOT2 began from **[brandonkowalski/slot](https://github.com/brandonkowalski/slot)**, which is MIT licensed
as well. The cartridge and slot artwork, the slot's own sounds and the emulator host structure were ported
from it; the original license text travels with the project as
[licenses/upstream-slot/LICENSE](licenses/upstream-slot/LICENSE).

Everything distributed alongside SLOT2 keeps **its own terms and its own notices**. The bundled originals
govern - not this README, and not a summary in it:

- the six emulator cores are other people's work, each under its own license, with its pinned source and
  the build recipe next to it;
- the two UI fonts are under the SIL Open Font License;
- the Rust crates the device binary links have their own licenses, inventoried with the crate's own
  license text where it supplies one.

On a card all of that lives under `System/licenses/`: `SLOT2-LICENSE`, `CORE-NOTICES.md`, the original
project's license, the two font licenses, `cores/<name>/<license>`, a `<core>.so.meta` stamp recording
where each core binary came from, `sources/` (pinned source archives and recipes) and `rust/` notices. In
this repository the core and upstream license texts are in [CORE-NOTICES.md](CORE-NOTICES.md) and
[licenses/](licenses/).

> **Snes9x and Genesis Plus GX are not for sale and not for commercial use.** The licenses bundled with
> those two cores state that their redistributions may not be sold and may not be used in a commercial
> product or activity, and that commercial users must ask the copyright holders first. SLOT2 ships them on
> that basis only: this project grants you no commercial right to them and cannot. **Read the bundled
> originals** - `System/licenses/cores/snes9x/LICENSE` and
> `System/licenses/cores/genesis_plus_gx/LICENSE.txt`, or their copies under [licenses/cores/](licenses/cores/)
> - before you redistribute anything or use it commercially.

This section is a pointer, **not legal advice**, and it does not replace any license or notice text. Where
a notice here and a bundled original disagree, the bundled original governs. Do not assume that every
component bundled with SLOT2 permits commercial use.
