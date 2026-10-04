# Task 48 - Load per-game RetroArch cheat files from the card

Work directly in the current checkout. Implement the card-layout and parser layer for D-21's
per-game RetroArch `.cht` files. This task only turns a card file into ordered cheat records;
later tasks will validate codes per core, call libretro, and build the in-game UI.

## Read first

Read only these files initially:

- C:\SLOT2\tasks\48-retroarch-cheat-file-loader.md
- C:\SLOT2\crates\slot2-store\src\lib.rs
- C:\SLOT2\crates\slot2-store\src\card.rs, only `DIRS`, `Card` path helpers, and `Cart`
- C:\SLOT2\crates\slot2-store\src\ini.rs
- C:\SLOT2\crates\slot2-store\tests\card.rs, only path/layout test conventions
- C:\SLOT2\docs\DECISIONS.md, D-21 only
- C:\SLOT2\docs\DESIGN.md, card-layout cheat path only

Do not read App, Session, UI, libretro host, handoff/history, old task reports, logs, or repository
history. The format contract needed for this task is copied below.

## Fixed source format

RetroArch/libretro-database cheat files are plain config text. The official database uses this
shape:

```text
cheats = 2

cheat0_desc = "Infinite lives"
cheat0_code = "7E007C9A"
cheat0_enable = false

cheat1_desc = "Max hearts"
cheat1_code = "7E13F2FF"
cheat1_enable = true
```

Official files use zero-based contiguous indices, quoted descriptions/codes, full-line `#`
comments, and `+` inside a code to join multiple code parts. RetroArch also recognizes many
memory-search fields; SLOT2 does not need those to pass emulator codes to `retro_cheat_set`.

## Contract

1. Add a focused public `cheats` module in `slot2-store` and re-export its record type. Use a
   small owned value such as `Cheat { description: String, code: String, enabled: bool }` with
   Debug/Clone/Eq derives. Do not expose parser internals or create a generic config framework.
2. Add `Card::cheat_path(&Cart)` returning exactly
   `System/cheats/<PLAT>/<stem>.cht`, using the existing platform folder and cart stem contracts.
3. Extend `ensure_layout` with `System/cheats` and its seven platform subdirectories, parents
   first. Preserve every existing directory and count behavior.
4. Add `Card::read_cheats(&Cart) -> Result<Vec<Cheat>, slot2_store::Error>`:
   - a missing file is `Ok(Vec::new())`;
   - I/O and invalid UTF-8 are `Error::Io` with the cheat path;
   - malformed structure or values are `Error::Invalid` with a concise message that includes
     enough key/index context to diagnose the file, but never dumps the whole file.
5. Parse the declared `cheats = N` and return records in numeric index order `0..N`. `N = 0` is
   a valid empty file. Missing count, invalid/negative count, duplicate count, missing index,
   out-of-range indexed entry, duplicate owned field, or missing/empty description/code makes
   the complete load fail; never return a silently partial list.
6. For each declared index, read `cheatN_desc`, `cheatN_code`, and optional
   `cheatN_enable`. Missing enable defaults to false. Accept enable values `true`/`false` and
   `1`/`0` case-insensitively; reject other values.
7. Accept official double-quoted values and reasonable bare values. Trim syntax whitespace, remove
   one matching outer quote pair, and support escaped quote/backslash inside quoted text. Keep
   Unicode descriptions intact. Preserve the decoded code string exactly: do not uppercase,
   split/rejoin `+`, remove punctuation, interpret wildcards, or validate a core-specific format.
8. Ignore unknown non-owned keys and known RetroArch fields such as handler/search/rumble values,
   so database files with extra metadata remain compatible. Full-line comments and blank lines
   are ignored. A `#`, `+`, or `=` inside a quoted description/code is content, not syntax.
9. Keep parsing read-only. Do not write `.cht`, persist toggles, edit enable fields, scan external
   cheat databases, or add fallback name matching.
10. Do not reuse or modify the settings `Ini` parser if doing so would change the existing game
    settings format. A private cheat parser is preferred because `.cht` needs quoted strings and
    indexed structural validation.

## Tests

Add focused `slot2-store` tests proving at least:

- `ensure_layout` creates `System/cheats/<PLAT>` for all seven platforms without losing existing
  layout behavior;
- cheat paths use platform folder plus exact cart stem, and equal stems on different platforms do
  not collide;
- a missing file and `cheats = 0` both return an empty list;
- a representative official-style file returns ordered records with descriptions, multi-part
  `+` codes, explicit true/false and missing-enable default false;
- input line order does not change numeric record order;
- quoted Unicode and embedded `#`, `=`, escaped quote, and escaped backslash decode correctly;
- unknown global and per-cheat RetroArch metadata is ignored without altering owned fields;
- malformed count, duplicate count/owned fields, gaps, out-of-range indices, missing or empty
  description/code, invalid enable, unmatched quotes, invalid escape, invalid UTF-8, and a
  directory at the `.cht` path return an error with no partial records;
- code text containing platform-specific punctuation or wildcard characters survives byte for
  byte after quote decoding; this task must not reject it as a core format.

Use only temporary card roots and synthetic `.cht` text. Do not download or copy a large database
fixture into the repository. Do not weaken existing tests.

## Allowed files

- C:\SLOT2\crates\slot2-store\src\cheats.rs (new)
- C:\SLOT2\crates\slot2-store\src\lib.rs
- C:\SLOT2\crates\slot2-store\src\card.rs
- C:\SLOT2\crates\slot2-store\tests\cheats.rs (new, preferred)
- C:\SLOT2\crates\slot2-store\tests\card.rs only if existing layout assertions need extension
- C:\SLOT2\tasks\48-retroarch-cheat-file-loader.worker-result.md

Preserve unrelated uncommitted changes. Do not clean, revert, or reformat unrelated files.

## Out of scope and forbidden

- No `retro_cheat_set`/`retro_cheat_reset`, FFI, Core, Session, App, screen state, menu UI,
  localization, toast, or input changes.
- No core-specific code validation or conversion. That belongs to the quirks/application task.
- No `.cht` writer, toggle persistence, libretro-database import, network access, or new dependency.
- No workspace-wide test, device distribution build, Raspberry Pi test, or RG SP access.
- No adb, Samba, SD-card, or `D:\Refrom\SpruceOS\dist_final\pack` access.
- No shared GJC, BAI, OpenCodex, or Codex configuration changes.
- No delegation, recursive task creation, commit, or push.

## Validation

Run in this exact order after the final code change:

```powershell
cargo fmt --all -- --check
cargo test -p slot2-store
cargo clippy -p slot2-store --all-targets -- -D warnings
```

All commands must exit 0. If code changes after a validation command, rerun the affected
commands. Do not run workspace-wide tests or `build/dist-device.ps1`.

## Result report

Write `C:\SLOT2\tasks\48-retroarch-cheat-file-loader.worker-result.md` with at most about 35
lines:

- Success, failure, or partial completion and the true cumulative invocation count out of two.
- Changed files and final path, layout, parser, ordering, quoting, error, and compatibility rules.
- Each validation command, exit code, and concise test result.
- Whether code changed after final validation.
- Remaining issue or contract concern, and elapsed time.

Do not paste source or logs. A path/option error or timeout counts as an invocation. Stop at
forty-five minutes and write a partial report; do not make a third invocation. Do not change
models or configuration.
