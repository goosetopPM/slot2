# Task 120 - Correct the RG SP acceptance checklist (attempt 2 of 2)

## Goal

Correct three acceptance-contract errors in the otherwise useful Task 120 checklist. Preserve the valid Korean
checklist, its stable IDs where possible, all `NOT RUN` results, and its evidence/defect forms. This is the final
allowed worker call for Task 120.

## Read only

Read:

1. `tasks/120-rgsp-hardware-acceptance-plan.md`
2. `tasks/120-rgsp-hardware-acceptance-checklist.ko.md`
3. `tasks/120-rgsp-hardware-acceptance-plan.worker-result.md`
4. `tasks/119-host-ui-visual-gallery.result.md`
5. `README.ko.md` sections 1 through 6 only as needed
6. `docs/MILESTONES.md` M4, M6, and M7 acceptance/status lines only
7. `tasks/63-device-menu-app-wiring.result.md` only for the confirmed Volume-row mute action

The exact hosted stamp contract is supplied below; do not inspect workflows or packager code.

## Allowed files

Modify only:

- `tasks/120-rgsp-hardware-acceptance-checklist.ko.md`
- `tasks/120-rgsp-hardware-acceptance-plan.worker-result.md`

Do not modify any other file.

## Required corrections

### 1. Artifact identity must use the actual stamp fields

The hosted `System/VERSION.txt` has exactly three LF-terminated lines. Its first line is:

```text
SLOT2 <workspace-version> (<short-commit>)
```

It has no target field. About displays the frontend package version and the current device profile target; it has
no commit field.

Correct the identity form and GATE/IDN expectations so they independently prove:

- the CI artifact came from the exact full SHA recorded for the run;
- the short commit in `VERSION.txt` is a prefix of that full SHA;
- the version in `VERSION.txt` equals the About version;
- the About target equals `rgsp`;
- no text claims that `VERSION.txt` contains a target or that About contains a commit.

The reviewed change set that must be committed and pushed before hardware execution is all accepted work through
Task 120, including the final Codex review record, not merely Tasks 117-119. Keep the clean-tree and exact-SHA CI
gate. Do not instruct a tag or release.

### 2. Conditional migration safety must not block a fresh-card-only pass

The existing-card migration scenario is optional. A user without a suitable legacy test card must be able to
finish fresh-card acceptance with an explicit migration `SKIP` reason.

Correct SAFE-02 and any related safety/exit text so the off-card full backup is conditionally mandatory before
scenario B, but is not a required fresh-card row when scenario B is skipped. You may move it into the MIG section,
mark it conditional, or change its ID if necessary. The result must make these rules unambiguous:

- scenario A remains mandatory;
- scenario B may be skipped with a reason;
- if scenario B is run, the verified backup and rollback route are hard gates and cannot be skipped;
- skipping scenario B does not leave a required `NOT RUN` row that makes the entire acceptance fail.

Apply the same conditional logic to any other row whose procedure only makes sense when migration is selected.

### 3. Lid timing must be internally consistent and accurately labelled

M4 records the intended lid contract as: close -> save state and screen off; after three minutes closed -> power
off; reopening before that resumes. M4 still leaves that work unchecked, so this is an unverified release
acceptance target, not an already confirmed product claim.

Correct the lid section so:

- the normal close/open round trip explicitly reopens well before the three-minute threshold, for example after
  30-60 seconds, and expects resume;
- no row says to wait an unspecified "few minutes" while also expecting the device not to power off;
- the optional spare-card three-minute observation remains separate and expects the intended power-off behavior;
- the checklist explicitly says the M4 lid implementation/status is unchecked and this hardware row may expose a
  missing implementation; it must be `FAIL` or `BLOCKED`, never silently passed, if the behavior is absent;
- host/CI evidence still cannot fill the physical result.

### 4. Remove remaining ambiguity that affects execution

Use Task 63's confirmed contract for mute: in the in-game Device menu, select Volume and press A to toggle mute;
left/right change the level; mute is not a separate physical key. Replace the current instruction to discover and
copy a hint with that exact action. Keep physical volume keys as their separate test.

Revise the introductory claim that every expected result is a currently implemented product behavior. It must
distinguish:

- behavior described as current in README or already reviewed tasks;
- unchecked M4/M6 acceptance targets, especially lid and HDMI, which remain unverified and may fail;
- optional/out-of-scope observations.

Do not expand the checklist with new checks. Do not change a physical result from `NOT RUN`.

## Preserve

Preserve all valid coverage from attempt 1, including the two scenarios, seven platforms, alternate cores,
save/state/time controls, cheats, display, languages, audio, power, stability, unavailable features, evidence,
defect template, exit criteria, and user-only sign-off. Do not weaken the no-tag/no-release rule.

## Verification

Perform documentation checks only. Do not build or test the product.

Confirm all of the following in the report:

1. Every executable row is still a five-column table row with a stable ID and result `NOT RUN`.
2. No physical result is pre-filled and no hardware result is invented.
3. No statement claims target is in `VERSION.txt` or commit is in About.
4. A fresh-card-only run can satisfy exit criteria while scenario B is explicitly skipped.
5. The pre-three-minute reopen and the optional three-minute power-off procedures no longer contradict each
   other.
6. Mute says Volume row + A, while volume keys remain a separate physical input.
7. UTF-8 has no BOM, the file ends with LF, Markdown tables are consistent, and scoped whitespace check passes.

## Worker report

Replace `tasks/120-rgsp-hardware-acceptance-plan.worker-result.md` with a cumulative attempt `2/2` report. It
must state `PASS` or `FAIL`, list the exact corrected sections/IDs, update all row counts and the coverage map,
describe the three contract corrections, report every verification above, and explicitly state that no hardware,
network, build, test, commit, push, tag, or release action occurred.

If any listed contradiction remains, report `FAIL` rather than accepting it.
