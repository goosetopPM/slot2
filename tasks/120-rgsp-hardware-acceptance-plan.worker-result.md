# Task 120 - RG SP hardware acceptance checklist (worker result)

## Verdict

PASS. Cumulative attempt 2/2. Attempt 1 created the checklist; this attempt applied every required correction from
`tasks/120-rgsp-hardware-acceptance-plan-attempt2.md` and re-verified the document. No listed contradiction remains,
so the verdict is PASS rather than FAIL.

## Files changed

Modified (only these two):

- `tasks/120-rgsp-hardware-acceptance-checklist.ko.md` — created in attempt 1, corrected in attempt 2. Now 468
  lines, 54805 bytes, UTF-8 without BOM, LF-terminated.
- `tasks/120-rgsp-hardware-acceptance-plan.worker-result.md` — this cumulative report.

Nothing else was modified, staged, committed, or pushed. `docs/HANDOFF-CODEX.md` and the four Task117-119 tracked
source/test modifications were already dirty before Task 120 began and were left exactly as found. One empty stale
lock directory, `tasks/120-rgsp-hardware-acceptance-checklist.ko.md.lock/`, was left behind by the editor tooling
after a failed multi-edit write; it contained no file and was removed so the corrected write could be published.

## Attempt-2 corrections (exact sections / IDs)

### Correction 1 - artifact identity uses the actual stamp fields

The supplied contract: hosted `System/VERSION.txt` is exactly three LF lines, first line
`SLOT2 <workspace-version> (<short-commit>)`, with no target field; About shows frontend version + current target,
with no commit field. Rewritten:

- Section 1 form row `artifact VERSION.txt 내용`: now requires all three lines verbatim and states the first-line
  format and that the file has no target field.
- `GATE-01`: now commits and pushes **all accepted work through Task 120 including the final Codex review record**
  (not merely Tasks 117-119) and records the exact full 40-char SHA.
- `GATE-03`: now copies all three `VERSION.txt` lines verbatim.
- `GATE-04`: now compares field by field — (a) the CI artifact was built from the exact full SHA recorded for the
  run, (b) the `VERSION.txt` short commit is a prefix of that full SHA, (c) the `VERSION.txt` version equals the
  About version, (d) the About target equals `rgsp`; `VERSION.txt` has no target field and About has no commit
  field, so values are compared only through those correspondences.
- `IDN-01`: checks About version vs `VERSION.txt` version and About target == `rgsp`; explicitly states About shows
  only version and target and has no commit field; demands no commit value and no release/tag claim.
- Section 12 gate evidence row: records the full 40-char SHA and all three `VERSION.txt` lines.

No statement claims a target is in `VERSION.txt` or a commit is in About. The clean-tree and exact-SHA CI gate is
kept, and nothing instructs a tag or release.

### Correction 2 - conditional migration safety does not block a fresh-card-only pass

- `SAFE-02` now applies **only when scenario B is selected**. It is a hard gate (`PASS` required, `SKIP` forbidden)
  when scenario B runs, and a legitimate `SKIP` with a reason when scenario B is skipped.
- Section 3 title marks `SAFE-02` conditional; `SAFE-04` expectation changed to "백업이 있으면 그 백업을 그대로
  보존했다" so it stays true when no migration backup exists.
- Section 4 required-ID list changed from `SAFE-*` to `SAFE-01`, `SAFE-03`~`SAFE-06`, with a new bullet declaring
  `SAFE-02` conditional, so a skipped migration leaves no required `NOT RUN`.
- Section 11 header now states the off-card backup and the `MIGRATION.ko.md` §6 rollback route are hard gates when
  scenario B is run and cannot be `SKIP`ped, while a scenario-B `SKIP` leaves `SAFE-02` `SKIP` and creates no
  required `NOT RUN`.
- `MIG-01` now backticks `SAFE-02` and adds that the run does not proceed if backup or rollback is not ready.
- Exit criteria now state plainly: scenario A is mandatory, scenario B may be `SKIP`ped with a reason, and skipping
  B cannot produce a required `NOT RUN`.
- `MIG-01`..`MIG-14` remain optional and otherwise unchanged; no new check was added.

### Correction 3 - lid timing internally consistent and accurately labelled

- Section 9.1 now states that M4 records the lid contract ("닫으면 스테이트 저장 + 화면 끔, 3분 뒤 전원 끔, 3분
  전에 열면 복귀") but leaves the work **unchecked**, so `LID-01`~`LID-04` are unverified release acceptance
  targets that may surface `FAIL` or `BLOCKED`, and host/CI evidence cannot fill the physical result.
- `LID-02` labels the resume expectation as M4's unverified acceptance sentence and requires `FAIL`/`BLOCKED` when
  the behavior is absent.
- `LID-04` replaced the unspecified "몇 분" wait with an explicit **pre-threshold** round trip: reopen after about
  30-60 seconds (well under three minutes) and expect resume; if the device is already off at that point, record it.
- Section 9.1.1 keeps the three-minute power-off observation separate, optional, spare-card only, and now says it
  expects the intended power-off behavior and is distinct from `LID-04`'s pre-threshold round trip.
- `LID-OPT-01` labels the power-off as M4's unverified acceptance target.

No row waits an unspecified few minutes while expecting the device to remain on.

### Correction 4 - remaining execution ambiguity

- `INP-15` replaced the "discover and copy the on-screen hint" instruction with Task 63's confirmed contract: in the
  in-game Device menu select **Volume** and press **A** to toggle mute; left/right change the level; mute is not a
  separate physical key.
- `AUD-04` now names the same in-game Device menu -> Volume row + A procedure.
- `INP-12` remains the separate physical volume-key test, unchanged.
- Section 0's claim that every expected result is a currently implemented product behavior was replaced with three
  explicit categories: (1) behavior described as current in README/reviewed tasks, (2) unchecked M4/M6 acceptance
  targets, especially lid (9.1) and HDMI (9.4), which may fail, (3) optional/out-of-scope observations.

## Row counts and coverage map (updated)

**155 executable check rows**, each a five-column row (`ID / 절차 / 기대 결과 / 결과 / 증거·메모`) with result
`NOT RUN` and a unique stable ID.

Per prefix: `GATE` 6, `SAFE` 6, `INST` 8, `IDN` 6, `SHF` 10, `INP` 15, `PLT` 7, `COR` 8, `STT` 9, `TIM` 5, `CHT` 6,
`DSP` 9, `L10` 9, `AUD` 7, `LID` 5 (`LID-01`~`LID-04` + `LID-OPT-01`), `PWR` 7, `STB` 6, `HDM` 3
(`HDM-OPT-01`~`HDM-OPT-03`), `UNAV` 9 (`UNAV-01`~`UNAV-07` + `UNAV-OPT-08`~`UNAV-OPT-09`), `MIG` 14.

Correction to attempt 1: its report counted **152** rows and omitted the three `HDM-OPT-*` rows from both the
per-prefix list and the total. The document always contained them; the counts above are the machine-verified ones.

Coverage map (this map now includes the previously missing HDMI row):

| Required item | Checklist section | IDs |
| --- | --- | --- |
| 1. artifact/revision identity, complete install, first boot, splash, en/ko selection, About version+target, no false unsupported-device claim | 2, 6, 7.1 | GATE-01..06, INST-01..08, IDN-01..06 |
| 2. 720x480 shelf quality: safe area, cartridge/port, stable `A play` hint, insert/eject, menus, selection contrast, disabled rows, toasts, refusal/error | 7.2 (disabled rows in section 10) | SHF-01..10, UNAV-02..05 |
| 3. all documented physical inputs: D-pad, face buttons, shoulders/triggers, MENU tap/hold, SELECT combinations, volume/mute, power | 7.3 (power also 9.2) | INP-01..15, PWR-01 |
| 4. one legal test game on each of GB, GBC, GBA, NES, SNES, MD, SMS with platform/core/boot/input/video/audio/exit, no ROM filename | 7.4 | PLT-01..07 |
| 5. alternate cores GB/GBC mGBA+Gambatte and GBA mGBA+gpSP: per-game selection, relaunch, persistence, shared save RAM, core-scoped state lists, no format interchange | 8.1 | COR-01..08 |
| 6. save RAM after clean exit/relaunch, resume vs fresh, quick save/load, state switcher load/delete/undo, thumbnails, failure feedback | 8.2 | STT-01..09 |
| 7. rewind and fast-forward controls, latch/release, HUD indicators | 8.3 | TIM-01..05 |
| 8. cheat discovery, enable/disable, application, persistence/failure feedback with a legal synthetic case | 8.4 | CHT-01..06 |
| 9. scale modes, built-in shaders, overscan only where supported, overlay, immediate preview/commit/cancel, visual correctness, performance | 8.5 | DSP-01..09 |
| 10. full English and Korean traversal, Korean glyphs, time-zone preview/apply/cancel/persistence, UTC display contract, About | 7.1, 8.6 | IDN-04..06, L10-01..09 |
| 11. game audio, platform insert/eject SFX, volume/mute, menu pause/mute, resume, pop/click or channel loss | 8.7 | AUD-01..07 |
| 12. lid close auto-save/screen-off and reopen resume + optional 3-minute power-off | 9.1, 9.1.1 (optional, spare card only) | LID-01..04, LID-OPT-01 |
| 13. power menu restart and shutdown on a spare card plus file-system and save/state integrity | 9.2 | PWR-01..07 |
| 14. battery/time HUD, frame pacing, thermal/stability, bounded play period, subjective vs measured separated | 9.3 | STB-01..06 |
| 15. optional HDMI mirroring with SKIP otherwise, no V-13 or dedicated 720p claim | 9.4 | HDM-OPT-01..03 |
| documented unavailable/out-of-scope: Sync, brightness, blue light, Display defaults, Boot logo, user shader files, compressed ROMs, RG28XX | 10 | UNAV-01..07, UNAV-OPT-08..09 |
| scenario A fresh installation (M7 path) | 6 | INST-01..08 (+ GATE, IDN) |
| scenario B existing original-slot card migration (optional) | 11 | MIG-01..14 (+ conditional SAFE-02) |
| evidence form / defect template / exit criteria / sign-off | 12, 13, 14, 15 | (forms and criteria, no check IDs) |

## Verification (documentation only - no build, no test)

| # | Required check | Result |
| --- | --- | --- |
| 1 | Every executable row is a five-column table row with a stable ID and result `NOT RUN` | PASS: 155 of 155 rows have exactly `ID / 절차 / 기대 결과 / 결과 / 증거·메모`; all 155 result cells are `NOT RUN`; all IDs unique; no other result value appears in any row |
| 2 | No physical result pre-filled and no hardware result invented | PASS: zero cells contain `PASS`/`FAIL`/`SKIP`/`BLOCKED`; no observation, measurement, or outcome is stated as having happened |
| 3 | No statement claims target is in `VERSION.txt` or commit is in About | PASS: searches over `VERSION.txt` and About return only field-by-field comparisons and explicit negative statements (form row and `GATE-03` say the file has no target field; `IDN-01` says About has no commit field) |
| 4 | A fresh-card-only run can satisfy exit criteria while scenario B is explicitly skipped | PASS: `SAFE-02` is conditional; section 4, section 11 header, `MIG-01`, and exit criteria all state a scenario-B `SKIP` leaves no required `NOT RUN` and is not a failure |
| 5 | The pre-three-minute reopen and the optional three-minute power-off no longer contradict | PASS: `LID-04` reopens after about 30-60 s and expects resume; `LID-OPT-01` (separate, optional, spare-card only) waits past three minutes and expects power-off |
| 6 | Mute says Volume row + A, while volume keys remain a separate physical input | PASS: `INP-15` and `AUD-04` use in-game Device menu -> Volume row + A; `INP-12` keeps physical volume keys as their own test |
| 7 | UTF-8 no BOM, file ends with LF, Markdown tables consistent, scoped whitespace clean | PASS: first bytes `# SLOT` (no BOM); 0 CR bytes; file ends with LF; 26 table blocks all have constant pipe width; 0 trailing-whitespace lines; 0 tab characters |

Additional document checks carried over from attempt 1 and re-run after the edits: no tag/release instruction
(`GATE-05` and exit criteria still forbid creating a tag or release); no direct hardware-access command (no ssh,
sftp, adb, samba, serial, or IP-literal command); no secret; no ROM filename; no absolute local machine path.

## Source cross-checks

- Stamp fields and identity rule: taken verbatim from the attempt-2 contract (`VERSION.txt` = three LF lines, first
  line `SLOT2 <workspace-version> (<short-commit>)`, no target; About = version + target, no commit). Workflow and
  packager code were not inspected, as instructed.
- Mute contract: `tasks/63-device-menu-app-wiring.result.md` confirms Left/Right/A and physical VolUp/VolDown all
  drive one `App.volume`, matching the supplied "Volume row + A toggles mute, left/right changes level, no separate
  physical mute key" contract; `README.ko.md` §5 documents device volume keys only.
- Unchecked targets: `docs/MILESTONES.md` M4 leaves the in-game menu, lid, power menu, clock, and About unchecked;
  M6 leaves HDMI unchecked and defers dedicated 720p. These are now labelled as unverified in section 0 and 9.1/9.4.
- Lid intended contract: `docs/MILESTONES.md` M4 records "닫으면 스테이트 저장 + 화면 끔, 3분 후 전원 끔, 열면
  복귀", which section 9.1/9.1.1 now mirror without asserting it is already implemented.
- Migration: `docs/MIGRATION.ko.md` §3-§6 remain the source of the `MIG-*` procedures; `System/` full replacement and
  off-card backup/rollback are unchanged in substance.
- Covered coverage from attempt 1 (controls README §5, platform/core pairing README §2, install paths and card
  layout README §3-§4, cheats D-21, time zone D-25, safe area D-09, unavailable features README §1, M3/M4
  acceptance sentences) is preserved; only `MIG-08`, `DSP-06`, `PWR-02..05`, and `CHT-06` remain recording-style
  rows for the reasons below.

## Unresolved contract conflicts and ambiguities

Kept from attempt 1; the first three are now handled inside the document rather than left implicit.

1. Public README vs milestone tracking. `README.ko.md` §1 presents the in-game menu, shelf settings, cheats, power
   menu, clock, and About as product features while M4 leaves them unchecked. Resolution: section 0 now separates
   "current behavior" from "unchecked M4/M6 acceptance targets", so an absent M4 feature is reported as
   `FAIL`/`BLOCKED`, never silently passed.
2. Compressed/unsupported behaviors and overscan platform list. README §1 names no overscan platform list, so
   `DSP-06` records per platform whether the row exists and tests only where it appears.
3. "Settings preservation" in scenario B. `docs/MIGRATION.ko.md` §4 replaces the whole `System/` directory, so
   `MIG-08` checks that no old `System/` file leaked into the new tree instead of asserting settings carry over.
4. CI artifact naming. Workflow files are outside the allowed read list, so the checklist records the artifact name,
   digest, and the three `VERSION.txt` lines as fields to fill in and asserts only the `check`+`device` job success
   described in the handoff; no artifact or job name is claimed.
5. Power menu item semantics. README §5 documents MENU long -> power menu but not what the items do, so `PWR-02`
   records the labels verbatim and `PWR-03`..`PWR-05` assert only restart and shutdown.
6. Cheat quirks pre-validation. D-21's quirks pre-validation is marked unfinished in M4, so `CHT-06` is an
   observation row and records the actual behavior.
7. Identity fields. Resolved by correction 1: the short commit comparison uses `VERSION.txt` -> full SHA prefix,
   and no commit value is demanded from About.

## Compliance statement

No hardware, Raspberry Pi, SD card, ADB, Samba, SSH, SFTP, or serial access occurred. No network use, no software
installation, no build, no Cargo run, no device packaging, and no test execution occurred. Nothing was staged,
committed, pushed, tagged, published, or released. No shared configuration was changed and no ROM was inspected.
Only the two allowed files were written; the three `HDM-OPT-*` rows were already present and were only re-counted.
