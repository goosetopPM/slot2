# Migrating an original slot card to SLOT2

한국어: [docs/MIGRATION.ko.md](MIGRATION.ko.md)

This guide is for a card that already holds an original `slot` installation: user data at the card root
(`Games/`, `Labels/`, `Saves/`, `States/`, usually also `BIOS/` and `Wallpapers/`) plus the old `System/`
tree. The migration keeps that user data exactly where it is and replaces `System/` with SLOT2's, as one
complete directory. SLOT2 keeps the original card layout compatible on purpose - the card layout contract
in [docs/DESIGN.md](DESIGN.md) §9 says an existing slot user only has to swap `System/` - so games, labels,
saves and states keep the paths they already have.

What this guide is **not**:

- not a fresh installation on an empty card; that is the fresh-install section of the repository README;
- not an update policy between SLOT2 versions; how `System/` is replaced between SLOT2 releases is a
  separate decision and is not defined here;
- not BaseOS imaging; BaseOS comes from its own project, and nothing here writes or formats a BaseOS image.

**No public release has been published yet.** These steps are written from the card-layout and state
contracts and from the implementation. No original card has been migrated on hardware while writing them,
so treat the first run as a migration you are verifying, and keep the backups below until you are satisfied.

## 1. Identify and prepare

1. **Shut the device down completely before removing a card**, then take the card out and put it in a PC
   card reader. Editing card files while the frontend is running invites a half-written card; when you are
   finished, eject the volume safely and only then put the card back.
2. **Identify the data card root, not a BaseOS system or boot partition.** The data card root is the volume
   that holds `System/` together with one or more of `Games/`, `Labels/`, `Saves/`, `States/`. With two
   cards, BaseOS uses the inserted TF2 data card for this, and otherwise it uses the TF1 data partition -
   never the partition BaseOS boots its own system from. If you cannot tell which volume that is, **stop**:
   do not format, do not repartition, and do not delete anything recursively. Formatting policy belongs to
   BaseOS, not to this guide and not to SLOT2.
3. **Obtain a SLOT2 zip and its matching `.sha256` sidecar through a trusted path.** No public release page
   exists yet, so there is nothing here to download and no file name or digest to quote. What you need is
   one zip whose root is `System/` and the sidecar beside it that carries the zip's SHA-256. Check the
   digest before extracting, with the same commands the fresh-install section of the repository README
   gives, where `<zip>` stands for the real file name and the expected digest is the text inside
   `<zip>.sha256`:

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

## 2. Make and verify an off-card backup

1. **Copy the entire data-card root to separate PC storage** - another drive or folder that is not the card.
   At minimum the copy must contain the original `System/`, `Games/`, `Labels/`, `Saves/` and `States/`, and
   it should also take `BIOS/` and `Wallpapers/` if they are there. Copy the backup *before* touching
   anything on the card.
2. **A renamed folder on the same card is not a backup.** It shares the card's fate: one bad format, one
   pulled card mid-write, and both copies are gone together. The independent PC copy is the recovery source
   for everything below.
3. **Verify the copy by opening it, not by trusting the copy dialog.** Look at its top-level directories,
   and sample a few important files - a save you care about, a state, a label - to confirm they are there
   and not zero bytes.
4. **`States/` is part of the backup even though only `System/` is replaced.** On the first scan SLOT2 moves
   recognized older flat states into a per-core directory inside `States/` (section 5). A backup taken
   before that first scan is what lets you put the card back exactly as it was.

## 3. Inspect the release away from the card

1. **Verify the zip's checksum first** (section 1), then **extract into an empty staging folder on the PC** -
   never over the card, and never over the old `System/` on the card or in your backup.
2. **Require `System/frontend` directly under the staging root.** The release tree's root is exactly one
   `System` directory, and `System/frontend` inside it is a non-empty file; that is what BaseOS executes.
   Reject the archive if you get `SLOT2/System/frontend` or any other extra nesting, if `System/frontend`
   is missing or empty, or if you only have loose files without the complete `System/` tree.
3. **Do not merge old and new `System/` contents.** The old executables, libraries, settings and unknown
   files are recovery material, not files to import one by one. SLOT2's settings are its own
   (`System/slot2.ini`, and per-game `System/games/<PLATFORM>/<stem>.ini`); arbitrary settings from the old
   `System/` are not read and are not imported.
4. Keep the verified staging folder until the device has started successfully at least once - it is the
   source you copy from if the card copy has to be repeated.

## 4. Replace only `System/`

1. **Leave the card root alone**: `Games/`, `Labels/`, `Saves/`, `States/`, `BIOS/` and `Wallpapers/` stay
   exactly where they are, with the same names. Only `System/` is being replaced.
2. **After the off-card backup is verified**, move or rename the original card-root `System/` as **one
   directory** - it must not be left in place for the new tree to be copied over, because a mix of old and
   new files in one `System/` is exactly the broken state this guide exists to avoid.
3. **Copy the staged SLOT2 `System/` as one complete directory** to the card root, so that the card ends up
   with `<card root>/System/` containing the whole release tree.
4. If you temporarily keep the old tree on the card for convenience, give it a **unique name** and do not
   overwrite any earlier backup that may already be there. The independent PC copy, not the on-card folder,
   is what you restore from.
5. **Verify that `<card root>/System/frontend` exists and is not empty**, and that the rest of the release
   tree came along. Never copy just that one executable: a `System/` with the binary but not the cores is a
   frontend that cannot start a game.
6. **On any checksum, move, rename, copy or verification failure, stop before booting** and restore the
   original `System/` from the verified PC backup. Never continue with a partial or merged tree, and never
   "finish it later" on a card you are about to put back in the device.
7. Nothing else in the layout changes. `Games/`, `Labels/`, `Saves/` and `States/` stay at the card root,
   while SLOT2's program files, its fonts, its language files, its licenses and its new settings live in the
   replacement `System/`. Save data stays at `Saves/<PLATFORM>/<stem>.sav`, the path the original card
   already used, so the in-game saves you had remain the saves SLOT2 reads.

## 5. First SLOT2 start and the state move

1. **BaseOS launches `System/frontend` in preference to an old `System/slot`.** BaseOS checks
   `System/frontend`, then `System/launch_frontend.sh`, then `System/slot`. Replacing `System/` completely
   is the intended handoff between the two; leaving an old `System/slot` in place does not take priority
   over SLOT2, and this guide does not mix the two trees.
2. **SLOT2 scans supported ROM files that are regular, uncompressed files directly inside
   `Games/<PLATFORM>/`.** A ROM inside a subfolder, or a compressed archive, is not a shelf entry. Your
   existing folder layout is what the scanner expects, so the same games generally appear.
3. **Older flat states are moved once, on the first scan.** A card written before a game could be given a
   core keeps its states in the game's own directory - `States/<PLATFORM>/<stem>/resume.state`,
   `States/<PLATFORM>/<stem>/<n>.state`, and the matching `<n>.png` - while SLOT2 keeps them one level
   deeper, in the namespace of the core that wrote them: `States/<PLATFORM>/<stem>/<core>/...`, where
   `<core>` is the core's own base name such as `mgba_libretro`. During the shelf scan, the recognized flat
   `resume.state` and positive numbered `<n>.state` files, together with their matching PNG files, are moved
   by a same-card rename into the **platform's default core** namespace. A rename inside one card keeps the
   bytes and timestamps that were already there.
4. **The platform default core is where they go**, even if you have since selected an alternative core for
   that game: those flat files were written before a core could be chosen at all, so nothing else could have
   written them. GB, GBC and GBA default to mGBA (`mgba_libretro`); NES to FCEUmm; SNES to Snes9x; MD and
   SMS to Genesis Plus GX.
5. **States are core-specific, and this migration does not change that.** Gambatte and gpSP cannot be
   assumed to read states mGBA wrote, and the alternate core's state list will not show them. Save RAM is
   shared: `Saves/<PLATFORM>/<stem>.sav` is the same file for every core and is not part of this move.
6. **What is left alone.** A PNG with no state file beside it, a file whose name is not a number, `0.state`,
   a number that does not fit, another extension, and every subdirectory - including the per-core
   namespaces - stay where they are. Nothing is deleted or renamed by guesswork.
7. **One game is moved all or not at all.** Every destination is checked before the first rename. If a
   state file or a PNG already exists at any destination for that game - a **collision** - the whole game's
   move is refused **before the first rename** and the flat originals are left untouched. SLOT2 does not
   merge them and does not try to decide which one is newer. A collision refusal is not the same as an I/O
   failure after the move has begun: if a rename fails partway, SLOT2 attempts a best-effort reverse rename
   of the files it had already moved and reports the original error, but a reverse rename can fail as well,
   so it does not guarantee that every flat file is back where it started. In that case do not keep playing
   and do not tidy either state directory by hand: shut the device down, inspect and preserve **both**
   locations (`States/<PLATFORM>/<stem>/` and its core namespace) without editing them, and restore the
   original layout from the verified pre-migration backup (section 6).
8. **Check before meaningful play.** After the first start, look at a few representative games, then open an
   ordinary in-game save, then a state you cared about - under the **default** core. If a state is not listed
   while an alternate core is selected for that game, switch the game's core back to the default and look
   again before concluding anything is lost. Once you have started playing again, see section 6: the
   cleanest way back is a full restore, which discards everything done since the backup.

## 6. Rollback, including after SLOT2 has already scanned the card

A rollback after the first start is not just putting one folder back, because the first scan may have moved
files inside `States/`. Do it in this order:

1. **Shut the device down completely before removing the card.** The move happens while the frontend is
   running, so a card removed while the frontend is writing is left in an unknown state.
2. **Separately preserve any new save you actually want.** Copy out, to fresh storage, the save or state
   files SLOT2 created or updated after the migration. Deciding later whether to keep the old or the new
   file is a deliberate choice; do not let a restore make it for you by overwriting the only copy.
3. **Restore the original `System/`** from the verified PC backup - the whole tree, as one directory.
4. **If SLOT2 has started at least once, restore `States/` from the same pre-migration backup.** The first
   scan moves recognized flat states into the default core's namespace, so a restored `System/` alone would
   leave the old frontend looking at state directories that are no longer where it left them.
5. **Know what a full restore costs.** Restoring the complete snapshot is the cleanest rollback, and it
   discards every change made after the backup - saves and states included. That is why any file you want to
   keep is copied out in step 2, and why it is worth testing the migration before meaningful play.
6. **Verify, eject, then boot.** Check the restored card root (the original `System/` is the one you expect;
   `Games/`, `Labels/`, `Saves/`, `States/` are intact), eject the volume safely, put the card back, and
   start the device.

A rollback is not a repair for a half-finished copy: if the restoration itself fails partway, stay with the
PC backup and repeat it rather than booting the card.

## 7. Troubleshooting

- **Wrong card or partition.** You replaced `System/` on a volume that is not the one BaseOS runs the
  frontend from - typically the BaseOS system/boot partition instead of the data card root. The data card
  root is the volume that holds `Games/`, `Saves/`, `States/` and `Labels/` next to `System/`. Stop, restore
  that partition from your backup if you changed it, and re-identify the data card root. Do not format or
  repartition anything to "make it match".
- **Extra zip nesting.** If the card ended up with `<card root>/SLOT2/System/frontend`, the contents were
  extracted one level too deep and BaseOS will not find the frontend. Move the `System` folder up to the
  card root (or copy again from the verified staging folder), and delete nothing from the user-data folders.
- **Partial `System/`.** `System/frontend` missing, empty, or a `System/` that has the binary but not the
  cores means an incomplete copy. Boot the old tree again by restoring the original `System/` from the backup
  first, then copy the staged tree as one directory in a single operation. Do not boot a partial tree and do
  not fill the gaps by hand from the old `System/`.
- **Games missing from the shelf.** Check that the ROM is a regular, uncompressed file directly inside the
  right platform folder (`Games/GB`, `Games/GBA`, ...) with a supported extension, and that the card root the
  frontend sees is the one you edited. Nested or compressed ROMs are not scanned.
- **Missing in-game save.** Save data is keyed by the ROM file name's stem: `Saves/<PLATFORM>/<stem>.sav`.
  If a ROM was renamed at any point, its old save belongs to the old stem - put the old name back rather than
  hand-editing paths. Nothing in this migration renames or migrates saves automatically.
- **An old state shows up only under the default core.** The flat states were adopted into the platform's
  default core namespace (mGBA for GB/GBC/GBA). States are core-specific, so an alternate core such as gpSP
  or Gambatte will not list them. Set the game's core back to the default and check there before concluding
  it is lost; do not move state files between core directories by hand.
- **A legacy state could not be adopted (destination collision).** Both the old flat state and a namespaced
  state (or PNG) exist for the same game, so SLOT2 refused the move **before the first rename** and left the
  flat originals untouched. SLOT2 keeps listing and using the state already in the namespace of the core
  that resolves for that game; the refused flat state stays outside SLOT2's state list and cannot be
  selected in the frontend. Stop: keep the card and its backup, copy only the smallest relevant redacted
  excerpt from `/tmp/frontend.log` if it is available, and report the conflict (section 8). The pre-migration
  backup and the rollback in section 6 are the safe route back to the original flat layout - do not merge or
  move the files by hand.
- **Diagnostics.** A state-adoption failure is written to the frontend's standard error output (stderr),
  which BaseOS keeps in the volatile `/tmp/frontend.log` on the device: copy the lines you need before the
  next boot. `System/slot2-diag.txt` on the card is a different thing - the boot survey the frontend writes
  when it starts, readable in a card reader - and runtime state-adoption errors are **not** appended to it.

## 8. Reporting a problem

The repository's bilingual bug form is
[.github/ISSUE_TEMPLATE/bug-report.yml](../.github/ISSUE_TEMPLATE/bug-report.yml); it asks for the device,
the `BASEOS_TARGET` value, the build line, the card setup and the smallest redacted log excerpt. It is a
repository form, not a support service: filing one does not promise a fix and there is no release to fall
back to yet. Do not attach ROMs, BIOS files, copyrighted game assets, or a save with personal data in it,
and redact paths, names and secrets from anything you paste.
