# Task 106 - Public English and Korean README

## Goal

Replace the stale design-stage README with an accurate public guide in English and add a complete Korean
counterpart. Document what SLOT2 currently does, supported profiles and platforms, fresh one-card and
two-card installation, card layout, controls, PC preview, troubleshooting, and license boundaries.

This is documentation of the implemented product and the locally verified release layout. Do not imply
that a public release, hosted CI run, broad hardware acceptance, or migration from an existing original
slot card has happened. This task must not change product code, packaging, workflows, or contracts.

Read only the required parts of:

- `C:\SLOT2\docs\HANDOFF-CODEX.md`, top current-state section
- `C:\SLOT2\docs\MILESTONES.md`, M6 and M7
- `C:\SLOT2\docs\DESIGN.md`, supported devices, card selection/layout, launch/deploy, controls, settings,
  cores, saves/states, diagnostics, and known limits
- `C:\SLOT2\docs\DECISIONS.md`, only decisions referenced by those DESIGN sections
- root `README.md`, `Cargo.toml`, `LICENSE`, and `CORE-NOTICES.md`
- `C:\SLOT2\docs\TRANSLATING.md`
- `C:\SLOT2\cores\required.txt`
- focused source tables needed to verify public facts: device/platform/core registries, ROM extension
  discovery, input mappings, menu actions, and the host executable entry point
- `C:\SLOT2\tasks\105-tag-release-workflow.result.md`

Use `rg` to find those focused source definitions and read only necessary context. Ignore only the
orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate, commit, push, publish,
create a tag, use the network, access hardware, or change shared configuration. Maximum two attempts.
Wait up to 5 minutes for the first response and 45 minutes total.

## Allowed files

- Replace `README.md` with the English public guide.
- Add `README.ko.md` as the Korean public guide.
- Create or update `tasks/106-public-readme-en-ko.worker-result.md`.

Do not modify Rust code, tests, Cargo files, workflows, build scripts, assets, license files, DESIGN,
DECISIONS, MILESTONES, TRANSLATING, issue templates, migration documents, or any earlier task file.

## Shared README contract

The two READMEs must have the same section order, material facts, warnings, tables, commands, and links.
They need not be literal translations, but neither language may omit a limitation or add a capability.
Put a prominent reciprocal language link near the top of each file.

Use concise public prose. Prefer relative repository links. Do not add badges, screenshots, placeholder
links, guessed release URLs, or claims that require network or hardware verification. Keep both files UTF-8
without BOM and LF-only.

Both READMEs must contain these sections in this order:

1. overview and project status;
2. supported devices, platforms, and cores;
3. fresh installation for one-card and two-card setups;
4. ROM folders and card layout;
5. device controls;
6. PC preview and host keyboard controls;
7. troubleshooting / FAQ;
8. translation and project documentation;
9. licenses, upstream credit, and legal boundary.

## Product status and compatibility

Describe SLOT2 as a pre-release cartridge-style frontend for BaseOS on Anbernic H700 handhelds. State that
the implementation and local automated validation are extensive, but hosted tag-release execution, a
fresh-card first-user walkthrough, and broad physical-device acceptance remain pending. RG SP is the
development and physically tested baseline. Other implemented profiles must be labeled implemented but not
fully hardware-accepted. Do not call the project a design-stage shell.

Verify the current device registry before writing the table. The intended profile set is RG SP, RG34XX,
RG34XXSP, RG35XXSP, RG35XX Plus, RG35XXH, RG35XX Pro, RG40XXH, RG40XXV, and RG CubeXX. RG28XX is explicitly
out of scope. Do not generalize support to every H700 device.

Document the exact current platform/core relationship:

| Platform | ROM folder | Extensions | Default core | Alternate core |
| --- | --- | --- | --- | --- |
| GB | `Games/GB` | `.gb` | mGBA | Gambatte |
| GBC | `Games/GBC` | `.gbc` | mGBA | Gambatte |
| GBA | `Games/GBA` | `.gba` | mGBA | gpSP |
| NES | `Games/NES` | `.nes` | FCEUmm | - |
| SNES | `Games/SNES` | `.sfc`, `.smc` | Snes9x | - |
| MD | `Games/MD` | `.md`, `.gen`, `.bin` | Genesis Plus GX | - |
| SMS | `Games/SMS` | `.sms` | Genesis Plus GX | - |

Confirm spelling and identifiers against source and `cores/required.txt`. State that ROM discovery scans
regular files directly inside each platform folder; nested folders and compressed ROMs are not supported.
No ROMs or BIOS files are included. BIOS files are optional for the current cores and, when supplied by the
user from lawful sources, belong directly in `BIOS/`. Do not invent BIOS filenames.

Briefly list implemented user-facing functions only: shelf/cart UI, per-game core choice where the table
allows it, save data, numbered states, resume/fresh launch, rewind and fast-forward, cheats, display scale,
the built-in shader presets, overscan/overlay controls, English/Korean language packs, timezone, and volume.
Clearly label Sync, brightness, blue-light control, user GLSL files, compressed ROMs, and RG28XX support as
unavailable or deferred. Do not imply that Wi-Fi/SFTP sync exists.

## Fresh installation

This task documents fresh installation only. Add a visible warning that replacing `System/` on an existing
original slot card is a separate future migration procedure; users must back up their card and must not use
the fresh-install steps as an upgrade guide.

Link to the BaseOS repository for BaseOS flashing instructions. Do not reproduce or guess BaseOS imaging
steps. The release bundle contract is a zip whose root is `System/`, plus a `.sha256` sidecar. Because no
public release has been executed yet, describe this as the layout of a release asset the user has obtained,
not as an available GitHub download. Do not fabricate a release page URL.

Include checksum examples without hard-coded version or digest. The Windows PowerShell example may compare
`(Get-FileHash <zip> -Algorithm SHA256).Hash` with the digest in `<zip>.sha256`. On Linux, use
`sha256sum -c <zip>.sha256`. On macOS, use `shasum -a 256 <zip>` and tell the reader to compare the printed
digest with the sidecar. Keep commands syntactically valid and explain that `<zip>` is a placeholder.

One-card setup:

- install BaseOS on TF1 by following BaseOS documentation;
- mount the BaseOS data/card-root volume;
- extract the contents of the SLOT2 zip at that root so the executable is exactly
  `<card root>/System/frontend`, never `<card root>/SLOT2/System/frontend`;
- create/copy user ROM files into the platform folders shown in the table and safely eject.

Two-card setup:

- keep BaseOS on TF1;
- prepare TF2 in a filesystem supported by BaseOS, without claiming an unverified exact formatter policy;
- extract the zip contents to the TF2 root so it contains `System/frontend`;
- put ROMs in TF2 `Games/<PLATFORM>` folders;
- explain that BaseOS/SLOT2 prefers the inserted TF2 data card and otherwise uses the TF1 data partition,
  both presented to the frontend as the card root.

Describe the expected first start conservatively: BaseOS runs `System/frontend` with the card root as the
working directory, SLOT2 creates missing user-data directories, and the shelf shows supported ROMs found in
the direct platform folders. Do not claim a tested first-boot acceptance run.

## Card layout

Show a compact tree containing the user-relevant paths:

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

Explain briefly that ROM filename stems identify related labels, saves, states, per-game settings, and
cheats, so renaming a ROM can disconnect it from existing associated files. Do not document unstable
internal filename encodings or promise automatic migration.

## Controls

Verify all entries against current App input handling. Include clear device tables for at least:

Shelf:

- Left/Right: select cartridge
- L1/R1: change platform
- A tap: resume from the latest auto/resume state, otherwise start fresh
- A hold: start fresh
- MENU tap: settings
- MENU hold: power menu

Playing:

- MENU tap: in-game menu
- MENU hold: eject to shelf
- SELECT+R1: quick-save the next numbered state
- SELECT+L1: quick-load the latest numbered state
- hold R2: fast-forward
- double-tap R2: toggle the fast-forward latch
- hold L2: rewind, taking priority over fast-forward

Menus generally use D-pad to navigate, A to confirm, and B or MENU to go back. Add the state switcher
exceptions: Left/Right select, A load, X delete, Y undo, and B/MENU back. If current source differs, follow
source and call out the discrepancy in the worker report instead of silently documenting the requested row.

## PC preview

Verify the current host entry point before documenting the exact command and root behavior. Document the
minimum offline development preview command if it is valid, normally `cargo run -p slot2`, and explain any
required `SLOT2_ROOT` setup only if source actually requires or honors it. Do not imply that the PC preview
validates device-only audio, controls, GPU, power, storage mounting, or performance.

Include the verified host keyboard map: arrow keys D-pad; X/Z device A/B; S/A device X/Y; Q/W L1/R1;
1/2 L2/R2; Enter Start; Right Shift Select; M or Backspace MENU; PageUp/PageDown volume; P power; Escape
quit. Resolve any source discrepancy in favor of source and report it.

## Troubleshooting and documentation

Cover these focused cases without promising automatic repair:

- empty shelf: confirm direct platform folder, supported extension, and regular uncompressed file;
- frontend not starting: confirm exact `System/frontend` nesting and that the complete release tree was
  extracted;
- missing core: confirm the complete release was copied and `System/cores/` contains the required set;
- a GB/GBC/GBA game that misbehaves: try its supported alternate core from the game menu;
- lost-looking saves/states after rename: restore the ROM stem or associated filenames from backup;
- unavailable settings: Sync, brightness, and blue-light currently have no completed backend;
- diagnostics: `/tmp/frontend.log` is volatile and `System/slot2-diag.txt` is the card diagnostic report.

Do not claim a persistent `System/slot2.log` unless current production source proves that exact path. Do not
tell users to delete, replace, or rename an existing `System/` directory as troubleshooting.

Link to `docs/TRANSLATING.md`, `docs/DESIGN.md`, `docs/DECISIONS.md`, and `docs/MILESTONES.md`. The English and
Korean READMEs should both make the translation contribution path easy to find. Issue templates and the
migration guide remain later tasks; do not create or promise a specific filename for either.

## License boundary

Credit `brandonkowalski/slot` with its repository link and explain that SLOT2's frontend is MIT-licensed.
Link `LICENSE`, `CORE-NOTICES.md`, and `System/licenses/`. State that distributed components keep their own
terms and notices. Prominently mention that Snes9x and Genesis Plus GX include noncommercial restrictions,
and direct readers to the bundled originals before redistribution or commercial use. Do not paraphrase the
notices as legal advice and do not say that every bundled component permits commercial use.

## Verification

Do not run Cargo tests, clippy, device builds, packagers, GitHub Actions, or hardware checks. Product and
packaging code are unchanged. Perform focused offline documentation verification:

1. Prove the two READMEs have the required section order, reciprocal language links, and matching material
   table rows, warnings, commands, unavailable items, diagnostic paths, and license links.
2. Resolve every relative Markdown link and prove the target exists with the expected file/directory type.
   Check external links for syntax only; do not contact them.
3. Compare the platform folders/extensions/core choices with current source and `cores/required.txt`.
4. Compare every device and host control row with current input mappings and App actions.
5. Compare profile names/status wording, card selection, first-start behavior, and diagnostics with current
   contracts/source.
6. Scan both READMEs for stale or forbidden claims including design-stage/M0 wording, public release
   availability, hosted/tag success, all-device acceptance, implemented Sync/brightness/blue-light,
   compressed-ROM support, RG28XX support, or unrestricted commercial use.
7. Prove UTF-8 without BOM, LF-only line endings, no trailing whitespace, and no broken fenced code blocks.
8. Run `git diff --check`.

After final verification, do not change files.

## Worker report

Write `C:\SLOT2\tasks\106-public-readme-en-ko.worker-result.md` even on failure. Include:

- success/failure and cumulative attempt count, maximum 2;
- final section list and English/Korean parity evidence;
- exact source/contract locations used for devices, platforms, cores, extensions, card selection, controls,
  host keys, diagnostics, and feature limitations;
- relative-link counts and resolution result;
- every forbidden-claim scan and encoding/line-ending/fence result;
- commands, exit codes, last result lines, created/modified files, final verification time, and whether
  content changed afterward;
- explicit confirmation that no code, test, workflow, build, package, network, hardware, commit, push,
  tag, release, or shared configuration action occurred;
- remaining hosted release, fresh-card acceptance, issue-template, and migration-guide work.

Do not commit.
