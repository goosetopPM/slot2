# Task 120 - Prepare the RG SP hardware acceptance checklist

## Goal

Create the Korean checklist and evidence form that the user will later execute on an RG SP. This task is
documentation only. It must convert the current product contracts and the remaining Task 119 hardware boundary
into a safe, reproducible acceptance procedure without claiming that any physical test has run.

The checklist must require an exact committed revision and its successful hosted CI device artifact. The current
dirty local tree is not an acceptable device-test input. A tag or release is neither required nor allowed.

## Read only what is needed

Read these sources in this order:

1. The top current-state section of `docs/HANDOFF-CODEX.md`.
2. `tasks/119-host-ui-visual-gallery.result.md`.
3. The relevant sections of `README.ko.md`: current status and limits, supported devices/platforms/cores, fresh
   installation, card layout, device controls, in-game menus, PC preview boundary, and troubleshooting.
4. M3 through M7 acceptance items in `docs/MILESTONES.md`.
5. `docs/MIGRATION.ko.md`, only for the existing-card scenario, backup, state migration, and rollback rules.
6. Only if a statement remains ambiguous, the exact relevant paragraphs for D-03, D-09, D-18, D-22, D-23,
   D-24, and D-25 in `docs/DECISIONS.md` or their matching contract paragraphs in `docs/DESIGN.md`.

Do not read production source merely to make the checklist longer. If the public and design documents disagree,
do not guess: record the exact conflict in the worker report and use the narrower confirmed behavior.

## Allowed output

Create or modify only:

- `tasks/120-rgsp-hardware-acceptance-checklist.ko.md`
- `tasks/120-rgsp-hardware-acceptance-plan.worker-result.md`

Do not modify source, tests, manifests, workflows, public documentation, milestones, handoff, generated output, or
any earlier task file.

## Hard restrictions

- Do not access an RG SP, Raspberry Pi, SD card, ADB, Samba, SSH, SFTP, serial console, or any mounted device.
- Do not use the network, install software, build, run Cargo, run device packaging, or execute a test.
- Do not stage, commit, push, tag, publish, or create a release.
- Do not invent control mappings, paths, supported features, or success evidence.
- Do not include commands that tell this worker, Codex, or another agent to access hardware. The checklist is for
  the human user to follow later.
- Do not mark a physical result PASS. Leave all result cells blank or as `NOT RUN`.
- Do not expose ROM names, credentials, tokens, personal paths, or machine-specific secrets.

## Checklist language and structure

Write `tasks/120-rgsp-hardware-acceptance-checklist.ko.md` in clear Korean. Preserve literal UI labels, button
names, paths, target identifiers, and core names where accuracy requires them.

The document must begin with:

1. A prominent `NOT RUN` status and a statement that only the user can complete physical acceptance.
2. A preflight identity form for date, tester, device, exact `BASEOS_TARGET`, BaseOS version, exact Git commit SHA,
   hosted CI run URL, artifact name, artifact digest or SHA-256, artifact `VERSION.txt`, and card layout.
3. A hard gate: do not begin with a dirty or uncommitted local build. First commit and push the reviewed Task
   117-119 set, obtain a successful hosted CI device artifact for that exact SHA, and verify the identities match.
   This gate must not instruct the user to tag or release.
4. Safety rules: prefer a spare test card, back up at least `Games/`, `Labels/`, `Saves/`, `States/`, and `System/`
   before any existing-card migration, preserve `BIOS/` and `Wallpapers/` when present, use only legal test ROMs,
   and stop on corruption or data-loss symptoms. Destructive lid/power checks must not use the primary card.

Use a checkbox table for executable checks. Each row must have these fields:

- ID
- procedure
- expected result
- result (`PASS`, `FAIL`, `SKIP`, or `BLOCKED`)
- evidence/notes

Give every row a stable ID suitable for a defect report. Default the result to `NOT RUN` outside the allowed
result vocabulary explanation; do not pre-fill success.

## Required test scenarios

Keep these as two separate scenarios:

### A. Fresh installation

This is the required M7 acceptance path: supported BaseOS on a spare/fresh card, then the exact successful CI
artifact. Verify artifact identity, complete package layout, boot/splash, target/version display, and clean first
use. Do not replace README installation instructions with guessed abbreviated commands; link to the exact section.

### B. Existing original-slot card migration

Run only after scenario A passes and only with a verified pre-migration backup and rollback route. Refer to
`docs/MIGRATION.ko.md` instead of duplicating risky file operations. Include evidence for preservation of games,
labels, saves, settings, and old states; default-core state namespace migration; shared save RAM; rollback; and
the rule that state files must not be moved manually between core directories. Make this scenario optional for a
user who has no suitable legacy test card, with an explicit `SKIP` reason.

## Required coverage matrix

The checklist must cover all of the following, using the exact supported behavior and control mappings from the
source documents:

1. Artifact/revision identity, complete install, first boot, splash, Korean/English selection, About version and
   target, and absence of a false unsupported-device claim.
2. RG SP 720x480 shelf quality: safe area, cartridge/port, stable `A play` hint, insert/eject motion, menus,
   selection contrast, disabled rows, toasts, and refusal/error presentation.
3. All documented physical inputs: D-pad, face buttons, shoulders/triggers, MENU tap/hold, SELECT combinations,
   volume/mute controls, and power behavior. Use the README mapping verbatim in substance; do not infer mappings.
4. One legal test game on each supported platform: GB, GBC, GBA, NES, SNES, MD, and SMS. Record platform, selected
   core, boot result, input, video, audio, and exit result without requiring the private ROM filename.
5. Alternate-core behavior where supported: GB/GBC mGBA and Gambatte, and GBA mGBA and gpSP. Check per-game core
   selection, relaunch, persistence, shared save RAM, and core-scoped state lists. Do not imply that state formats
   are interchangeable.
6. Save RAM after clean exit/relaunch, resume versus fresh launch, quick save/load, numbered state switcher
   load/delete/undo, thumbnails, and failure feedback.
7. Rewind and fast-forward controls, latch/release behavior where documented, and the matching HUD indicators.
8. Cheat discovery, enable/disable, application, persistence/failure feedback using only a legal synthetic or
   disposable test case.
9. Scale modes, built-in shaders, overscan only where supported, overlay, immediate preview/commit/cancel, visual
   correctness, and obvious performance regressions.
10. Full English and Korean traversal, Korean glyph rendering, time-zone preview/apply/cancel/persistence, UTC
    display contract, and About information.
11. Game audio, platform insert/eject sound effects, volume/mute, menu pause/mute, resume, and audible pop/click or
    channel-loss observations.
12. RG SP lid close auto-save/screen-off and reopen resume. Put any three-minute power-off observation in a
    clearly optional, spare-card-only subsection that the user must explicitly choose to run.
13. Power menu restart and shutdown on a spare card, followed by file-system and save/state integrity checks.
14. Battery/time HUD, frame pacing, thermal/stability observations, and a bounded play period on representative
    demanding platforms. Separate subjective observations from any measured value.
15. HDMI mirroring only when the user has the cable/setup. Make it optional and require `SKIP` otherwise; do not
    claim V-13 or dedicated 720p output is validated.

Documented unavailable or out-of-scope rows must be checked as unavailable presentation, not treated as product
failures: Sync, brightness, blue light, Display defaults, Boot logo, user shader files, compressed ROMs, and
unsupported devices such as RG28XX. Use the current README wording and avoid extending the support claim.

## Evidence and defect form

For each section, specify useful evidence such as a photo/screenshot filename, artifact checksum record, or short
observation. Evidence naming must avoid ROM titles and personal data.

Add a reusable defect template with:

- checklist ID and severity (`BLOCKER`, `MAJOR`, or `MINOR`)
- exact reproduction steps
- expected and actual behavior
- language, platform, core, device/target, and artifact SHA
- photo/log reference
- repeatability and workaround

State that secrets, copyrighted ROMs, BIOS files, private save data, and raw personal paths must not be attached.

## Execution order and exit criteria

Provide a compact, risk-ordered plan that can be split into sessions, for example:

- installation, identity, shelf, controls, and seven-platform boot
- states, alternate cores, display, cheats, language, time, and audio
- lid/power, stability, optional HDMI, and optional migration

The exit criteria must state all of these:

- Every required release-blocking item is `PASS`; optional items are either `PASS` or have an explicit `SKIP`
  reason; no required item remains `NOT RUN` or `BLOCKED`.
- No unresolved `BLOCKER` or `MAJOR` defect remains.
- The fresh-card scenario passes on the exact committed/CI artifact identity.
- Migration is accepted only if it was actually run; otherwise it remains explicitly skipped and is not claimed.
- The user signs the final hardware acceptance line. Gajae, Codex, host evidence, or CI cannot sign it for them.
- On failure, create a narrow fix task and rerun the affected checks plus named regressions.
- Even a full pass does not authorize a tag or release. Those require a later explicit user instruction.

Clearly distinguish the Task 117-119 host evidence already reviewed from the remaining physical evidence. Host
evidence may reduce repetition but must never fill a physical `PASS` result.

## Verification

Before writing the worker report:

1. Cross-check every control mapping, platform/core pairing, feature availability, path, and acceptance claim
   against the specified sources.
2. Confirm the checklist contains no pre-filled physical PASS, invented hardware result, release/tag instruction,
   direct hardware-access command, secret, ROM name, or absolute local machine path.
3. Confirm the Markdown tables are structurally consistent, the file is UTF-8 without BOM, and it ends with a
   newline.
4. Run only a scoped whitespace check for the two allowed files. Do not run build or test commands.

## Worker report

Write `tasks/120-rgsp-hardware-acceptance-plan.worker-result.md` before stopping. Include:

- `PASS` or `FAIL` and cumulative attempt `1/2`
- files changed
- a coverage map from every required matrix item to checklist section/IDs
- source cross-checks and the scoped document verification result
- any unresolved contract conflict or ambiguity
- an explicit statement that no hardware, network, build, test, commit, push, tag, or release action occurred

If a contract cannot be made accurate within the allowed documents, report `FAIL` rather than inventing it.
