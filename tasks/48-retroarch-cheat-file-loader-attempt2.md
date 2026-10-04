# Task 48 attempt 2/2 - Bound declared-count work and truly ignore metadata

This is the final allowed invocation for Task 48. Preserve the accepted loader behavior and fix
one denial-of-service defect plus one compatibility edge in the parser. Do not start libretro
application or UI work.

## Read first

Read only:

- C:\SLOT2\tasks\48-retroarch-cheat-file-loader-attempt2.md
- C:\SLOT2\tasks\48-retroarch-cheat-file-loader.md, Contract items 5 and 8 only
- C:\SLOT2\tasks\48-retroarch-cheat-file-loader.worker-result.md
- C:\SLOT2\crates\slot2-store\src\cheats.rs
- C:\SLOT2\crates\slot2-store\tests\cheats.rs

Do not read other source, history, handoff, logs, App, Session, UI, or libretro code.

## Defects to correct

1. The current parser calls `Vec::with_capacity(count as usize)` before proving that the file
   actually contains `count` complete owned entries. A tiny file containing
   `cheats = 4294967295` can therefore request an enormous allocation before returning the
   expected missing-entry error. No memory allocation, loop, or other work may scale from the
   declared count alone.
2. The current parser inserts an `Entry` before deciding whether an indexed field is owned.
   Therefore an unknown field such as `cheat999_handler` can create a phantom entry and affect
   gap/out-of-range validation, despite the contract saying unsupported RetroArch metadata is
   ignored.

## Required behavior

1. Create/update an `Entry` only for the three owned fields: `desc`, `code`, and `enable`.
   Unknown global and indexed fields must have no structural effect, regardless of their index.
2. After parsing, validate owned entries by iterating the actual sorted map:
   - owned indices must be contiguous from zero;
   - every owned index must be below the declared count;
   - the number of complete owned entries must equal the declared count;
   - missing/gap/out-of-range errors must retain useful index/count context.
3. Allocate the result only after structural validation and size it from the actual validated
   entry collection, never directly from an untrusted declared count. Do not loop `0..count`
   until equality with actual entries has already bounded that work.
4. Keep every accepted Task 48 behavior unchanged: numeric order, complete-file failure,
   quoting/escaping, Unicode, exact code preservation, enable parsing/default, missing-file and
   zero-count handling, path/I/O errors, and read-only operation.
5. Do not introduce an arbitrary cheat-count maximum if actual input structure already bounds
   allocation. A real large file should be limited by its actual parsed entries, not a guessed
   product cap.

## Regression tests

Add focused tests proving:

- `cheats = 4294967295` with no owned entries returns `Error::Invalid` without panic or enormous
  allocation;
- the same huge count with one complete index still returns a concise missing/count mismatch;
- a count larger than `u32` remains an invalid count;
- with `cheats = 0`, far-out unknown indexed metadata such as `cheat4294967295_handler` is ignored
  and the result is empty;
- with `cheats = 1`, unknown indexed metadata before/after the valid entry does not create a gap
  or change the record;
- an out-of-range owned field such as `cheat1_desc` when `cheats = 1` still fails;
- all existing Task 48 tests remain unchanged and pass.

Do not write a negative test that intentionally triggers the old enormous allocation before the
fix is in place. The code inspection and new bounded test are sufficient; do not risk OOM.

## Allowed files

- C:\SLOT2\crates\slot2-store\src\cheats.rs
- C:\SLOT2\crates\slot2-store\tests\cheats.rs
- C:\SLOT2\tasks\48-retroarch-cheat-file-loader.worker-result.md

Preserve unrelated uncommitted changes. Do not clean, revert, commit, or push.

## Validation

Run after the final code change, in this order:

```powershell
cargo fmt --all -- --check
cargo test -p slot2-store
cargo clippy -p slot2-store --all-targets -- -D warnings
```

All commands must exit 0. Do not run workspace-wide tests, dist, Pi, or hardware checks.

## Result report

Update `C:\SLOT2\tasks\48-retroarch-cheat-file-loader.worker-result.md` as the cumulative **2/2**
report. Put the attempt-2 delta first: bounded validation strategy, metadata behavior, new tests,
all final validation exit codes, whether code changed afterward, remaining concern, and elapsed
time. Keep it concise and do not paste source or logs.

Stop by forty-five minutes even if incomplete. There is no third invocation. Do not delegate,
change models/configuration, use network, or access hardware/ADB/Samba/SD card.
