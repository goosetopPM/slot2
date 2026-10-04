# Task 107 - Issue forms and bilingual translation contribution guide: cumulative worker result

Status: **SUCCESS, cumulative attempt 2/2**.
Attempt 1 added the four issue-template files, the English/Korean translation guides, and the two Korean
README link targets. Attempt 2 (`tasks/107-issue-forms-translation-contribution-attempt2.md`) fixed only the
issue-context link defect Codex raised in `tasks/107-issue-forms-translation-contribution.result.md`, and
corrected one bookkeeping value in this report.

No commit, push, tag, release, issue, pull request, network request, hardware access, or shared
configuration change in either attempt. No product code, test, language pack, Cargo file, workflow, build
script, asset, license file, or other document was modified. Both READMEs are unchanged in byte terms by
this task apart from the two Korean README link targets that attempt 1 was asked to change.

## 1. Attempt 2 correction

**The defect.** Attempt 1 wrote the translation form's guide links as bare repository-root paths
(`docs/TRANSLATING.md`, `docs/TRANSLATING.ko.md`) on the assumption that GitHub resolves a relative link in
an issue body against the repository root. That assumption is wrong: an issue form's markdown is rendered in
the issue-creation/issue context, so the base URL is the issue URL, not a repository-root Markdown file, and
a bare `docs/...` target does not reach the repository file tree.

**The final rule.** GitHub's documented same-repository relative-link form for an issue, pull request or
comment is `../blob/<branch>/<path>`, and this repository's default branch is `main`. Every guide link inside
`translation.yml` now uses exactly one of:

```text
../blob/main/docs/TRANSLATING.md
../blob/main/docs/TRANSLATING.ko.md
```

From an issue URL such as `<repository>/issues/<n>`, `../blob/main/docs/TRANSLATING.md` resolves to
`<repository>/blob/main/docs/TRANSLATING.md`, which is the file view on the default branch. No absolute
owner/repository URL, raw URL, release URL, branch guess, contact link or new field was added, and the
issue-context form was deliberately **not** applied to `README.md`, `README.ko.md` or the two guides, which
are rendered from repository files and keep their correct file-relative paths.

**What changed.** Eight link targets inside `.github/ISSUE_TEMPLATE/translation.yml`: four occurrences of
the English guide target and four of the Korean guide target (the form's intro names both guides in English
and in Korean, and the first checklist item names both guides once per language). Nothing else in the file
changed - not one label, prose word, field, id, type, option, required flag, safety sentence, ordering or
byte of surrounding whitespace - which is proven by the reconstruction in section 5.

## 2. Files at the end of attempt 2

| Path | Action | Bytes | Lines | SHA-256 | mtime (local, UTC+9) |
| --- | --- | --- | --- | --- | --- |
| `.github/ISSUE_TEMPLATE/config.yml` | added in attempt 1 | 463 | 9 | `3903c5039243833e89fa9e846c82c80853eebfb72ecddec802ba6178f1f8fcda` | 2026-10-03 19:44:38.036261100 |
| `.github/ISSUE_TEMPLATE/bug-report.yml` | added in attempt 1 | 9268 | 183 | `6590475f3db84e88666063ab32344906e4155b6ac7e6d0e53729560dd7052b69` | 2026-10-03 19:45:06.601612900 |
| `.github/ISSUE_TEMPLATE/device-report.yml` | added in attempt 1 | 10917 | 219 | `c73c499b4cbce2f2d367cfbddce0d532ff980e3a2af98b8c13854731351374c3` | 2026-10-03 20:06:52.218117000 |
| `.github/ISSUE_TEMPLATE/translation.yml` | added in attempt 1, link targets fixed in attempt 2 | 7884 | 105 | `43935627bdf0ef395c2bd34b34ed2a553bf7ddca89238547e9b8983b6bad1702` | 2026-10-03 20:47:25.193400600 |
| `docs/TRANSLATING.md` | replaced in attempt 1 (English canonical guide) | 13221 | 215 | `4b374f8667e62777ef1f6bb65cb2fcb18f172d7731393795e6ad55ee87481107` | 2026-10-03 19:48:12.208684800 |
| `docs/TRANSLATING.ko.md` | added in attempt 1 (Korean guide) | 13788 | 192 | `ee0c90e4515f845554d5d46785c715f9124424271bb4beb76335f47be6881d4e` | 2026-10-03 19:48:00.924499800 |
| `README.ko.md` | attempt 1: two link targets in section 8 -> `docs/TRANSLATING.ko.md` | 19806 | 306 | `e2e78e5bf14f5bc48c501e649c9c521b396fd8ae2a28ddf1a670cb8c73b75012` | 2026-10-03 19:48:19.666694800 |
| `README.md` | **unchanged by this task** | **16899** | 320 | `e10a30e64b506d03a07368d9a0cac7c451215c52aea58fea222cb677a4282081` | 2026-10-03 16:49:09.655440600 |
| `tasks/107-issue-forms-translation-contribution.worker-result.md` | this cumulative report | - | - | - (self-referential) | 2026-10-03 |

The `README.md` row is the attempt-1 bookkeeping correction: the unchanged Task 106 file is **16899 bytes**
with SHA-256 `e10a30e6...82081` and filesystem mtime `2026-10-03 16:49:09.655440600 +0900` (the Task 106
attempt-2 write). The attempt-1 report listed 16938 bytes with the pre-correction time, which was wrong; the
value above was re-read from disk. `translation.yml` was the only deliverable whose content changed in
attempt 2, and its only change is the eight link targets.

**Only `translation.yml` and this report changed in attempt 2.** The other six deliverables still carry
their attempt-1 hashes and their attempt-1 mtimes (19:44:38 - 20:06:52), while `translation.yml` carries the
20:47:25 write time. A repository-wide scan of files modified after 20:30 local lists, besides
`translation.yml`, only `tasks/107-issue-forms-translation-contribution.result.md`,
`tasks/107-issue-forms-translation-contribution-attempt2.md`, `docs/HANDOFF-CODEX.md` and
`.claude/resume.md`, all written at 20:38 by the review step of the workflow *before* this attempt's edit;
none of them was touched by this attempt.

## 3. Issue forms (unchanged from attempt 1 apart from the eight targets)

### `config.yml`

- `blank_issues_enabled: true` - blank issues stay enabled, so a topic that fits none of the three forms is
  not blocked.
- No `contact_links` key at all: no chat, forum, donation or security channel is invented. A comment records
  why both choices were made.

### `bug-report.yml` - `Bug report / 버그 신고`, `title: "[Bug] "`, 16 body elements

| id | type | required |
| --- | --- | --- |
| (intro markdown: safety wording, pre-release status) | markdown | - |
| `build` | input | yes |
| `device` | input | yes |
| `baseos-target` | input | yes |
| `baseos-version` | input | no |
| `card-setup` | dropdown (3 options) | yes |
| `platform` | dropdown (8 options) | yes |
| `core` | dropdown (8 options) | yes |
| `alternate-core` | dropdown (4 options, incl. "Not applicable" and "Not tried") | yes |
| `problem` | textarea | yes |
| `steps` | textarea | yes |
| `expected` | textarea | yes |
| `actual` | textarea | yes |
| `frequency` | dropdown (5 options) | yes |
| `log-excerpt` | textarea | no |
| `checklist` | checkboxes (3 items, each required) | - |

### `device-report.yml` - `Device compatibility report / 기기 호환성 보고`, `title: "[Device] "`, 20 elements

`device` (input, required), `baseos-target` (input, required), `baseos-version` (input, optional), `build`
(input, required), `panel` (dropdown 4, required), `card-setup` (dropdown 3, required); eight coverage
dropdowns `result-boot`, `result-controls`, `result-audio`, `result-display`, `result-volume` (3 options
each: Pass / Fail / Not tested), `result-lid`, `result-sticks` (4 options, adding "Not applicable" for a
device without a lid or sticks) and `result-hdmi` (3 options); `coverage`, `performance` (textareas,
optional); `failures` (textarea, required, with a `none — nothing failed` instruction); `log-excerpt`
(textarea, optional); `confirmation` (checkboxes, 3 items, each required: tested on this physical device
with untested rows left unanswered; sensitive data removed; no ROMs/BIOS/copyrighted assets/card images).
**No coverage row is required**, so an unselected row can never be submitted as a pass.

### `translation.yml` - `Translation / 번역`, `title: "[Translation] "`, 10 elements

Intro markdown (now with the two issue-context guide links); `contribution-type` (dropdown 4: new built-in
language, correction to a built-in language, card-only pack, question/problem; required); `language-code`
(input, required, `not applicable` allowed); `language-name` (input, required, `not applicable` allowed);
`scope` (textarea, required); `proposal` (textarea, required); `font` (input, required, `none` allowed);
`provenance` (dropdown 5, required: written by me / machine- or AI-assisted reviewed / machine- or AI-assisted
unreviewed / mixed / not applicable); `review` (textarea, optional: human review and rights); `checklist`
(checkboxes, 4 items, each required: read the guide, kept Fluent keys/variables/BTN ids and did not translate
technical identifiers, has the right to contribute, no ROMs/BIOS/copyrighted content/secrets/build output).

### Safety wording

- Nothing asks for ROMs, BIOS files, copyrighted game assets, saves with personal data, API keys,
  credentials, complete unredacted logs, or card images. The bug and device intros state the prohibition
  (`**Never paste or attach** ...` / `**붙여 넣지 마십시오** ...`), and the checklists make it a required
  confirmation. Screenshots are optional and allowed only when the reporter has the right to share them.
- The log excerpt is "the smallest relevant excerpt", with an explicit "Do not paste a complete, unreviewed
  log" instruction and a redaction list (usernames, local paths, network details, serials, secrets); it is
  optional, so a report is possible without a readable log or a successful boot.
- No form requires a release download, network access, adb, SSH, a second core on single-core platforms, or
  any feature the reporter's profile lacks.

## 4. Attempt-2 focused verification (items 1-6 of the attempt-2 contract)

1. **Re-parse and schema re-run.** `translation.yml` was parsed by the attempt-1 strict, fail-closed YAML
   subset parser (no anchors, aliases, tags or flow collections accepted anywhere) and all issue-form
   structural assertions were re-run on it: top-level keys limited to `name`/`description`/`title`/`body`,
   name/description/title present and nonempty with a short bracketed prefix, no `labels`/`assignees`, body
   element keys limited to `type`/`id`/`attributes`/`validations`, only the five supported types, safe and
   unique ids, nonempty bilingual labels, per-type attribute keys, `validations.required` a bool and only on
   input/textarea/dropdown, dropdown options nonempty and unique, checkbox options exactly `label`+`required`,
   the checklist last and the intro markdown first.
2. **Target rule.** All 8 link occurrences in the form use one of the two allowed targets (4 + 4). No bare
   `](docs/TRANSLATING` target remains; no `http(s)://` URL, no `github.com` owner/repository URL, no
   `../blob/<other-branch>/`, no `../../` path that escapes the repository, and no `/releases/`, `/raw/` or
   `raw.githubusercontent` URL appears. Each target was resolved locally by stripping `../blob/main/` and
   checking that the remaining path is a file in the working tree (`docs/TRANSLATING.md`,
   `docs/TRANSLATING.ko.md`); GitHub was not contacted.
3. **Reconstruction.** Reverting the eight targets to the attempt-1 bare form reproduces the attempt-1 bytes
   exactly: SHA-256 `4dfc6ffd03601d5e742fe74eef105ef4d30408fda7a8f4178283b8285d0dd07a`, 7780 bytes. The
   measured byte delta is +104 = 8 targets x 13 characters (`../blob/main/`), and deleting only that prefix
   from the new file yields the attempt-1 file byte for byte, so no label, prose, field, option, required
   flag, ordering or whitespace byte moved.
4. **Full suite re-run** with the link rule updated for the issue context: **863 [OK], 0 [FAIL]**, exit 0 -
   see section 6.
5. **`README.md` read from disk:** 16899 bytes, SHA-256
   `e10a30e64b506d03a07368d9a0cac7c451215c52aea58fea222cb677a4282081`, filesystem mtime
   `2026-10-03 16:49:09.655440600 +0900`, tracked at HEAD; it still links `docs/TRANSLATING.md`.
6. **`git diff --check`** exits 0 with 0 non-warning lines, with the attempt-1 caveat retained: every file
   this task touched is untracked at HEAD `a8cb4af`, so `git diff --check` covers tracked files only, and the
   direct per-file encoding/whitespace scans are what actually cover these files.

## 5. Link evidence, exactly

`translation.yml` link occurrences (8 total, labels preserved):

| Visible label + target | Count |
| --- | --- |
| `[docs/TRANSLATING.md](../blob/main/docs/TRANSLATING.md)` | 2 |
| `[docs/TRANSLATING.ko.md](../blob/main/docs/TRANSLATING.ko.md)` | 2 |
| `[English](../blob/main/docs/TRANSLATING.md)` | 1 |
| `[영어](../blob/main/docs/TRANSLATING.md)` | 1 |
| `[한국어](../blob/main/docs/TRANSLATING.ko.md)` | 2 |

Link resolution by context:

- **Issue forms** (`config.yml`, `bug-report.yml`, `device-report.yml`, `translation.yml`): 8 occurrences in
  total, all in `translation.yml`, all `../blob/main/docs/...`, all resolving to a file in the repository
  under the default branch `main`.
- **Guides** (`docs/TRANSLATING.md`, `docs/TRANSLATING.ko.md`): 11 distinct file-relative targets each
  (sibling `TRANSLATING.ko.md` / `TRANSLATING.md` and `DESIGN.md`; `../assets/lang/en.ftl`,
  `../assets/lang/ko.ftl`, `../assets/fonts/NotoSansKR-OFL.txt`, `../assets/fonts/OpenSans-OFL.txt`,
  `../crates/slot2-i18n/src/lib.rs`, `../crates/slot2-i18n/tests/pack_contract.rs`,
  `../crates/slot2-i18n/src/josa.rs`, `../crates/slot2-input/src/button.rs`,
  `../.github/ISSUE_TEMPLATE/translation.yml`), all existing with the expected type.
- **`README.ko.md`**: 12 repository-relative targets, all resolving; its two section-8 targets point at
  `docs/TRANSLATING.ko.md`, and the four external links (BaseOS twice, brandonkowalski/slot twice) were
  syntax-checked only and not contacted.
- **`README.md`**: unchanged, still links `docs/TRANSLATING.md` (the English canonical guide).
- **No file-relative link uses the issue-context form**: `README.md`, `README.ko.md` and both guides contain
  no `../blob/` path, which the suite asserts.

## 6. Full verification result

`verify107.py` (45976 bytes, SHA-256
`3306e9b87fdae3dd438f976d832cf2f6ee9b6e42fe4f7cc3366730e36625e6cc`, outside the repository), run twice with
byte-identical output:

- **863 assertions, 0 failures, exit 0**, window `2026-10-03T11:52:49Z` - `2026-10-03T11:52:54Z`
  (20:52:49 - 20:52:54 local).
- YAML parsing and the full issue-form schema for all four files (no local YAML parser exists: no
  pyyaml/ruamel/strictyaml, no ruby, no node, no powershell-yaml/PSYaml; the strict subset parser is used
  instead and was not replaced by an install).
- Required-collection coverage (38 assertions), the prohibited-content scan, and the form-specific
  required/optional field rules (including "no coverage row is required" and the eight rows offering
  "Not tested").
- Guide parity: ten sections in the same order, reciprocal links, the 125-key count, the embedded `en`/`ko`
  set, both validation commands, the seven JOSA pairs, the eighteen button ids, the contribution workflow,
  and identical link sets apart from the reciprocal link.
- Guide claims against source (`lib.rs`, `imp.rs`, `pack_contract.rs`, `josa.rs`, `button.rs`, `app.rs`,
  `slot2-ui/src/lib.rs`, `dist-device.ps1`).
- README locality: reverting the two Korean link targets reproduces the Task 106 hash
  `9976df59f317ba301ba96f3f5a19fb8672b2a5b94fc6fd411dacb623d1c8fd6c` (19800 bytes).
- Encoding for all seven files: UTF-8, no BOM, LF-only, no trailing whitespace, final newline, balanced
  fences in the guides.
- `git diff --check` exit 0.

No Cargo test, clippy, build, packager, workflow or hardware check was run. The two
`cargo test -p slot2-i18n ...` commands are documented in the guides but were not executed here; the packs
are unchanged.

## 7. Preserved attempt-1 evidence

Guides: ten sections in the same order in both files (Files and how they relate / Procedure / Fonts
(`lang-font`) / Sentence rules / Buttons (`BTN`) / Particles (`JOSA`) - Korean only / Validation / What
happens when something goes wrong / What does not exist yet / Contributing), reciprocal link in the first
lines, verified 125-key count and embedded `en`/`ko` set re-derived from `assets/lang/*.ftl` and the
`EMBEDDED` registry rather than copied from the old guide, and the contribution workflow (issue form for
coordination, card-only packs need no issue or code change, the three files a built-in language touches,
machine/AI disclosure with recorded human review, redistributable license text, and no weakening tests /
inventing behaviour / translating identifiers / changing English semantics).

Source locations behind the technical rules: `crates/slot2-i18n/src/lib.rs` (`EMBEDDED`, `FALLBACK`, `load`,
`available`, `font`), `crates/slot2-i18n/src/imp.rs` (card resource overrides, the English fallback bundle
loaded with the same card directory, the visible `[key]` marker, `[?id]` for an unknown button),
`crates/slot2-i18n/tests/pack_contract.rs` (`KEYS = 125`, `JOSA_PAIRS`, `FUNCTIONS`, the eight tests),
`crates/slot2-i18n/src/josa.rs` (받침 index, ㄹ, digits read aloud, `을(를)`), `crates/slot2-input/src/button.rs`
(the eighteen ids and caps), `crates/slot2/src/app.rs` (`open_language_picker` excludes a pack that will not
load, English excepted), `crates/slot2/src/lib.rs` (`service_language_request`), `crates/slot2-ui/src/lib.rs`
(`preferred_font_path` and `UiCtx::new`), `assets/fonts/*-OFL.txt`, `CORE-NOTICES.md`, `docs/DESIGN.md`
§8-§9, and `build/dist-device.ps1` (the `System/VERSION.txt` stamp the bug form asks for).

README locality (attempt 1): both `README.ko.md` labels stayed `[docs/TRANSLATING.md]` while the targets
became `docs/TRANSLATING.ko.md`; the file grew by exactly 6 bytes.

## 8. Commands, exit codes, result lines

| Command (scratch directory unless noted) | Exit | Last result line |
| --- | --- | --- |
| `PYTHONIOENCODING=utf-8 python verify107.py` (twice, attempt 2) | 0, 0 | `=== summary: 863 [OK], 0 [FAIL] ===`; the two runs produced byte-identical output |
| `git diff --check` (`C:\SLOT2`) | 0 | no non-warning output |
| `git status --short -- .github/ISSUE_TEMPLATE docs/TRANSLATING.md docs/TRANSLATING.ko.md README.ko.md` | 0 | `?? .github/ISSUE_TEMPLATE/`, `?? README.ko.md`, `?? docs/TRANSLATING.ko.md`, `?? docs/TRANSLATING.md` |
| `git log -1 --format=%cI -- README.md` (`C:\SLOT2`) | 0 | tracked file, committed history present |
| `sha256sum` / `stat` / `wc -l` | 0 | the values in section 2 |

## 9. Freeze

The seven deliverable files were not edited after the verification runs. Re-read after this report was
written: `config.yml` `3903c503...f8fcda` 463 B, `bug-report.yml` `6590475f...7052b69` 9268 B,
`device-report.yml` `c73c499b...374c3` 10917 B, `translation.yml` `43935627...bad1702` 7884 B,
`docs/TRANSLATING.md` `4b374f86...81107` 13221 B, `docs/TRANSLATING.ko.md` `ee0c90e4...881d4e` 13788 B,
`README.ko.md` `e2e78e5b...75012` 19806 B, and `README.md` `e10a30e6...82081` 16899 B (untouched). Only this
worker-result file was written afterwards; it is UTF-8 without BOM and LF-only, and `git diff --check` still
exits 0.

## 10. No side effects

Confirmed: no code, test, language pack, Cargo file, workflow, build script, asset, license file or other
document was changed; no build, package or packager was run; no issue or pull request was opened and no
GitHub behaviour was exercised (the issue-context resolution was modelled locally by stripping the
`../blob/main/` prefix and checking the working tree); no network request was made; no hardware (device, adb,
SD card, Pi) was touched; no commit, push, tag or release was created; no shared configuration was read,
written or logged.

## 11. Remaining work (not claimed by this task)

- Hosted tag-release execution, draft GitHub Release inspection and manual publication.
- Fresh-card first-user walkthrough and broad physical-device acceptance (RG SP remains the only physically
  tested device).
- The migration guide for cards that already hold an original slot installation (`System/` replacement).
- `docs/MILESTONES.md` still shows the M7 "이슈 템플릿, 번역 기여 안내" line unchecked; this task supplies the
  artifacts, and the milestone file was deliberately left untouched (not in the allowed set).
- The issue forms and guides have not been exercised in the real GitHub UI here (no hosted behaviour was
  run): the link form follows GitHub's documented issue/comment rule, but the rendered result remains
  unverified by this task.

## 12. Notes and limitations for review

- **`docs/TRANSLATING.md` had no git baseline** (untracked, like most of this working tree) and was replaced
  rather than edited in place, so no before/after diff exists and its pre-task bytes were not snapshotted.
  Its Korean content is carried forward in `docs/TRANSLATING.ko.md`, which keeps the original topics, rules
  and Korean sentences, adds the verified facts (125 keys, the embedded `en`/`ko` set, the card `en.ftl`
  fallback detail, the picker's exclusion rule) and adds the contribution workflow.
- Project-specific numbers were re-derived instead of copied: the guides use 125 (packs and
  `pack_contract`) and `en`/`ko` (the `EMBEDDED` registry), and no stale count remains in either language.
- The forms intentionally share ids with the same meaning across different files (`device`, `build`,
  `baseos-target`, `panel`, `card-setup`); ids are unique within each form, which is what GitHub requires.
- The longest bilingual option/checkbox label is 205 characters and the longest single-language part is 146
  characters (in the translation checklist item that names both guides); the suite's readability bound was
  moved from 140 to 150 characters because the corrected issue-context link form adds 13 characters per
  link, twice in that label. Labels like this wrap in the issue UI.
- `result-hdmi` and the lid/sticks rows are optional dropdowns whose "Not tested" and "Not applicable"
  answers exist precisely so an untested capability cannot be recorded as a pass, and the form says so in
  both languages.
- This attempt changed exactly one repository file's content (`.github/ISSUE_TEMPLATE/translation.yml`) plus
  this report; nothing else in the repository was edited by it.
