# Task 41 - Resume or start fresh from the shelf

Work directly in the current checkout. Complete the existing Resume flow: stopping a Session
already writes `resume.state`; now a tap of A on a shelf cart should load that Resume when it
exists, while holding A should deliberately start the game fresh.

## Read first

Read only these files initially:

- C:\SLOT2\tasks\41-resume-or-fresh-launch.md
- C:\SLOT2\crates\slot2\src\app.rs
- C:\SLOT2\crates\slot2\src\session.rs, only `Session::start`, `load_state`, and `stop`
- C:\SLOT2\crates\slot2\tests\insert_app.rs
- C:\SLOT2\crates\slot2\tests\session.rs, only Resume-related tests
- C:\SLOT2\crates\slot2-ui\src\shelf_view.rs
- C:\SLOT2\assets\lang\en.ftl
- C:\SLOT2\assets\lang\ko.ftl
- C:\SLOT2\crates\slot2-i18n\tests\i18n.rs

Read directly related shelf/UI test helpers only when needed. Do not read the full handoff,
milestone history, old task reports, logs, or repository history. The complete contract follows.

## Existing behavior

- A tap on a non-empty shelf starts the insert animation. The core is opened only when the cart
  reaches the seat, then the existing dwell finishes before `Screen::Playing`.
- `Session::stop` already atomically writes `StateKind::Resume` plus an optional thumbnail.
- `Session::load_state` can load Resume and resets the resampler and rewind history.
- The gesture layer emits `Tap(A)` on a short release and one `Hold(A)` after 600 ms; release
  after a hold does not emit a tap.
- `Screen` is Copy/Debug/Eq and must remain lightweight. Launch intent and clocks do not belong
  in the enum.
- The shelf currently has no dynamic Resume hint. Filesystem checks must not occur every draw.

## Contract

1. Add a small private App launch intent with two values: resume-if-present and fresh. Keep it
   outside `Screen`; preserve `Screen`'s Copy/Debug/Eq contract.
2. On a non-empty `Screen::List`, Tap(A) starts the existing insert animation with
   resume-if-present intent. Hold(A) starts the same animation with fresh intent. Both use the
   currently selected cart and preserve the existing sound, timing, sink, and refusal behavior.
3. Decide and store the selected cart/intent when the gesture is accepted. Navigation or a
   rescan must not make the seated animation start a different cart. Do not load the core before
   the existing `SEATED_AT` boundary.
4. Resume-if-present starts the normal Session, then loads `StateKind::Resume` before the
   session is exposed as Playing or any core frame advances. If no Resume exists, start fresh
   without an error message.
5. A valid Resume restores the exact saved core state through `Session::load_state`, keeps the
   resume file for future recovery, opens one audio sink, and completes the normal insert.
6. A missing Resume race between shelf display and launch falls back to a fresh game. A corrupt,
   incompatible, or unreadable Resume also continues with a usable fresh session, logs one
   concise error, and shows a localized `resume-load-failed` toast. Do not delete that failed
   Resume automatically.
7. Fresh intent never reads or loads Resume. Only after the core Session has started
   successfully, permanently remove the old Resume state and thumbnail so a power loss during
   the new run cannot resurrect the abandoned playthrough. Deletion failure logs one concise
   error but does not abort the fresh game.
8. If core/session startup fails, retain the old Resume untouched and follow the existing failed
   insertion/ejection/refusal path. Do not open or close extra sinks.
9. An A hold used for fresh launch must not leak as an A press into the running core if the
   player is still holding it when insertion finishes. Suppress that launch press until its
   matching release; later A presses reach the game normally. A tap is already released and
   needs no suppression.
10. Cache only a per-cart Resume-availability boolean during the existing card rescan. Do not
    retain state bytes or duplicate `StateSlot` lists in App, and do not stat/read the filesystem
    in the draw loop. Refresh the cache whenever the cart list is rescanned.
11. On a non-empty shelf, show a localized bottom hint that fits the safe area: without Resume,
    A means play; with Resume, A means resume and holding A means new game. Empty-shelf behavior
    stays unchanged. Add the smallest ShelfView/UI API needed; do not create a generic hint
    framework.
12. Power, volume, platform switching, insert/eject visuals, MENU behavior, numbered states,
    state-switcher undo, quick save/load, and Resume exclusion from the switcher remain unchanged.
    Update the App state-machine comments for Tap(A)/Hold(A).

Do not change the Resume storage format, add a confirmation dialog, or introduce automatic boot
resume. This is the selected shelf cart's launch behavior only.

## Tests

Add focused tests proving at least:

- stopping a real session writes Resume, a later Tap(A) starts the same cart and loads that exact
  core state before the first Playing frame;
- Tap(A) with no Resume starts fresh without failure feedback;
- a Resume that disappears after the cached hint but before seating falls back fresh;
- corrupt Resume reports `resume-load-failed`, still reaches a usable Playing session, remains
  on disk unchanged, and does not request a second sink;
- Hold(A) starts fresh, removes an existing Resume state and thumbnail only after successful
  session startup, and release after the hold cannot trigger a second insert;
- failed core startup after Hold(A) preserves Resume and follows the existing failure path;
- a launch A that remains physically held when Playing begins is suppressed until release, while
  a later A press reaches the core normally;
- the cart and launch intent are fixed at gesture acceptance even if selection data changes
  before seating;
- cached Resume availability updates on rescan and draw performs no filesystem access;
- English and Korean directly define the new failure message and shelf hints; play/resume/fresh
  hint variants fit `rgsp`, `rg35xxsp`, and `rgcubexx` safe areas;
- existing insert timing, sounds, wallpaper/HUD, sink requests, numbered-state, quick-state, and
  state-switcher tests continue to pass.

Use temporary card roots and explicit `Instant` values. Use the established explicit real-core
skip pattern for tests that need a core; keep gesture, hint, cached-availability, failure-path,
and pure state tests active without a core. Do not weaken or rewrite existing tests.

## Allowed files

- C:\SLOT2\crates\slot2\src\app.rs
- C:\SLOT2\crates\slot2\tests\insert_app.rs
- C:\SLOT2\crates\slot2\tests\resume_app.rs (new, preferred for integration tests)
- C:\SLOT2\crates\slot2-ui\src\shelf_view.rs
- C:\SLOT2\crates\slot2-ui\tests\shelf_resume.rs (new, preferred for hint tests)
- C:\SLOT2\assets\lang\en.ftl
- C:\SLOT2\assets\lang\ko.ftl
- C:\SLOT2\crates\slot2-i18n\tests\i18n.rs
- C:\SLOT2\tasks\41-resume-or-fresh-launch.worker-result.md

Do not modify `slot2-store`, `Session`, `slot2-input`, audio/runtime loops, manifests, or
documentation. Preserve unrelated uncommitted changes from earlier tasks. Do not clean, revert,
or reformat unrelated files.

## Out of scope and forbidden

- No automatic boot resume, lid behavior, persistent launch preference, confirmation dialog,
  state format change, retention policy, or numbered-state behavior change.
- No full workspace test, device distribution build, or Raspberry Pi test for this host-level
  App/UI change.
- No RG SP, Pi, adb, Samba, SD-card, or `D:\Refrom\SpruceOS\dist_final\pack` access.
- No shared GJC, BAI, OpenCodex, or Codex configuration changes.
- No delegation, recursive task creation, commit, or push.

## Validation

Run in this exact order after the final code change:

```powershell
cargo fmt --all -- --check
cargo test -p slot2 -p slot2-ui -p slot2-i18n
cargo clippy -p slot2 -p slot2-ui -p slot2-i18n --all-targets -- -D warnings
```

All commands must exit 0. If code changes after a validation command, rerun the affected
commands. Do not run workspace-wide tests, `build/dist-device.ps1`, or `build/pi-test.ps1`.

## Result report

Write `C:\SLOT2\tasks\41-resume-or-fresh-launch.worker-result.md` with at most about 30 lines
unless a failure needs more evidence:

- Success, failure, or partial completion and the true cumulative invocation count out of two.
- Changed files and final tap-resume/hold-fresh, failure fallback, hint, and A-suppression behavior.
- Each validation command, exit code, and concise result.
- Whether code changed after final validation.
- Remaining issue or contract concern, and elapsed time.

Do not paste code, logs, or repeat this specification. A path/option error or tool timeout counts
as an invocation. Stop at forty-five minutes and write a partial report; do not make a third
invocation. Do not change models or configuration.

## Attempt 2/2 correction contract

Attempt 1 is **not accepted yet**. Work only on the remaining items below, then update the same
worker result with the cumulative count `2/2`. Do not broaden the implementation.

1. Fix the unreadable-Resume branch. The current `resume_into` calls `Card::read_state` as a
   presence probe, but that API maps every read error to `None`; an existing unreadable Resume is
   therefore mistaken for a missing file and gets no `resume-load-failed` toast. Avoid reading the
   full state twice. Use the state path's existence/error result around `Session::load_state` so a
   genuinely missing race stays silent, while a path that still exists (or whose metadata cannot
   be queried) and fails to load logs once and shows the localized failure toast. Keep `Card` and
   `Session` unchanged.
2. Extend the fresh-launch test to create both `resume.state` and its `.png` thumbnail, and prove
   both remain before successful startup and both are gone afterward.
3. Add direct tests for every contract item attempt 1 reported as indirect or missing:
   - launch A remains hidden from the core while physically held, its release clears suppression,
     and a later A press reaches the core;
   - selected cart and Resume/Fresh intent remain the values captured at gesture acceptance even
     if selection or a rescan changes before seating;
   - Resume availability changes only on rescan, and drawing after the card root is changed or
     made unavailable keeps using the cached boolean without filesystem-dependent output;
   - `en` and `ko` directly define `resume-load-failed`, `hint-resume`, and `hint-new-game`;
   - an existing path that cannot be read as a state takes the failure-toast fallback, while the
     already-covered disappeared-file race stays silent.
4. Preserve the six existing integration tests and three shelf tests. Do not weaken assertions or
   count an explicit real-core skip as evidence for the pure gesture/cache/i18n cases.
5. Run the original three validation commands in the original order after the last code change.
   Stop at 45 minutes and report honestly; this is the final allowed invocation for Task 41.
