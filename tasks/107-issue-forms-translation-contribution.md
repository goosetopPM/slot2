# Task 107 - Issue forms and bilingual translation contribution guide

## Goal

Finish the M7 issue-template and translation-contribution entry point. Add structured GitHub issue forms for
bugs, device compatibility reports, and translation coordination. Make the existing Korean translation
guide available as an equivalent English/Korean pair and connect the Korean public README to the Korean
guide.

The repository is still pre-release. The forms and guides must not imply that a public release, hosted tag
run, broad device acceptance, or a formal support service exists. This is documentation and repository
metadata only; do not change product code or run hosted GitHub behavior.

Read only the required parts of:

- `C:\SLOT2\docs\HANDOFF-CODEX.md`, top current-state section
- `C:\SLOT2\docs\MILESTONES.md`, M5 translation and M7
- `C:\SLOT2\README.md` and `README.ko.md`, status, troubleshooting, translation, and license sections
- `C:\SLOT2\docs\TRANSLATING.md`, complete current Korean guide
- `C:\SLOT2\assets\lang\en.ftl` and `ko.ftl`, metadata and key/placeholder conventions
- `C:\SLOT2\crates\slot2-i18n\src\lib.rs`, embedded-pack registry
- `C:\SLOT2\crates\slot2-i18n\tests\pack_contract.rs`, contributor-facing validation contract
- `C:\SLOT2\crates\slot2-input\src\button.rs`, valid button ids only if needed to check the guide
- current files under `C:\SLOT2\.github`, only to avoid path/config conflicts
- `C:\SLOT2\tasks\106-public-readme-en-ko.result.md`

Use `rg` to locate any additional directly referenced translation test or path, then read only necessary
context. Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate,
commit, push, publish, create a tag, open an issue/PR, use the network, access hardware, or change shared
configuration. Maximum two attempts. Wait up to 5 minutes for the first response and 45 minutes total.

## Allowed files

- Add `.github/ISSUE_TEMPLATE/config.yml`.
- Add `.github/ISSUE_TEMPLATE/bug-report.yml`.
- Add `.github/ISSUE_TEMPLATE/device-report.yml`.
- Add `.github/ISSUE_TEMPLATE/translation.yml`.
- Replace `docs/TRANSLATING.md` with the English canonical guide.
- Add `docs/TRANSLATING.ko.md` containing the equivalent Korean guide.
- Modify `README.ko.md` only to point its two translation-guide links to `docs/TRANSLATING.ko.md`.
- Create or update `tasks/107-issue-forms-translation-contribution.worker-result.md`.

Do not modify `README.md`; its existing `docs/TRANSLATING.md` link must resolve to the new English guide.
Do not modify Rust code, tests, language packs, Cargo files, workflows, build scripts, assets, license files,
DESIGN, DECISIONS, MILESTONES, migration documents, or any earlier task file.

## Shared public contract

All new public repository metadata must be understandable in both English and Korean. Issue-form names,
descriptions, field labels, and safety instructions may put the two languages together rather than duplicate
files. Keep prompts concise enough for GitHub's issue UI.

Do not request or encourage uploads of commercial ROMs, BIOS files, save files containing personal data,
API keys, credentials, complete unredacted logs, card images, or copyrighted game assets. Ask reporters to
paste only the smallest relevant log excerpt after removing usernames, local paths, network data, serials,
and secrets. Screenshots are optional and must not show copyrighted game content unless the reporter has the
right to share it.

Do not pre-assign users, projects, milestones, or labels whose repository existence cannot be proven. Do not
add external contact links, donation links, Discord links, security-reporting promises, release URLs, or
support response-time promises. Keep blank issues enabled so topics outside the three forms are not blocked.

## GitHub issue-form configuration

Create `.github/ISSUE_TEMPLATE/config.yml` with blank issues enabled and no contact links. Use only keys
supported by GitHub issue-template configuration.

Each `.yml` form must use the documented GitHub issue-form shape:

- nonempty top-level `name`, `description`, and `body`;
- a short title prefix appropriate to the form;
- no speculative labels or assignees;
- unique, safe `id` values for every interactive field;
- only supported body types (`markdown`, `input`, `textarea`, `dropdown`, `checkboxes`);
- nonempty labels/options and correct `validations` / checkbox-option `required` placement;
- no duplicate YAML keys, anchors, aliases, custom tags, or multiline content that accidentally changes
  field structure.

### Bug report form

Create `bug-report.yml`. It must collect:

- SLOT2 build identity from About or `System/VERSION.txt`; allow `unknown` because there is no public release;
- exact device and `BASEOS_TARGET`, BaseOS version if known, and one-card/two-card setup;
- platform, selected core, and whether the problem also occurs with the supported alternate core, with
  `not applicable` available;
- a concise problem description, exact reproduction steps, expected behavior, actual behavior, and
  reproducibility/frequency;
- the smallest redacted excerpt from `/tmp/frontend.log` and/or `System/slot2-diag.txt`, with an explicit
  warning not to paste complete unreviewed logs;
- a final required checklist confirming the reporter searched existing issues, removed sensitive data, and
  did not attach ROM/BIOS/copyrighted game files.

Do not require a release download, network access, adb, SSH, a second core on single-core platforms, or a
successful boot before a bug can be filed.

### Device compatibility report form

Create `device-report.yml` for user-run physical-device evidence. It must collect:

- exact device name and `BASEOS_TARGET`, BaseOS version, SLOT2 build/commit identity, panel geometry, and
  one-card/two-card setup;
- what was actually tested, with clear `pass`, `fail`, and `not tested/not applicable` choices or text;
- boot/shelf, controls, audio, display scaling/overlay, volume, suspend/lid where applicable, sticks where
  applicable, game/core/platform coverage, performance observations, and HDMI only if tested;
- concrete failure/reproduction notes and the smallest redacted diagnostic excerpt;
- a required confirmation that results came from physical hardware and no ROM/BIOS/game asset or secret is
  attached.

The form must not turn an unchecked optional item into a pass. It must not claim compatibility or close any
M6/M7 acceptance item by itself. Do not require users to test features their profile lacks.

### Translation form

Create `translation.yml` for coordination or reporting a translation problem. It must collect:

- contribution type: new built-in language, correction to an existing built-in language, card-only pack,
  or translation question/problem;
- BCP-47-style language code and native language name, with `not applicable` allowed for a general question;
- affected message keys or scope;
- proposed wording/context or a precise description of the translation problem;
- font name/license needs if a new font is proposed, with `none` allowed;
- translation provenance and human-review status, including disclosure of machine/AI assistance without
  forbidding it or treating disclosure as a license grant;
- a required checklist confirming the contributor read the appropriate language guide, preserved Fluent
  keys/variables/BTN ids, has the right to contribute the text/assets, and did not attach ROM/BIOS files.

Link both language guides from the form's introductory markdown using repository-relative paths. Do not put
the translated `.ftl` payload in the issue form as a substitute for a reviewed repository change.

## Bilingual translation guides

Preserve the current Korean guide's useful technical content in `docs/TRANSLATING.ko.md`. Write an equivalent
English canonical guide at `docs/TRANSLATING.md`. Add a prominent reciprocal language link near the top of
each file. The two files must have the same section order and material rules; natural translation is
preferred over line-for-line wording.

Keep and source-check all existing topics:

1. canonical `assets/lang/en.ftl`, complete embedded packs, and partial card packs;
2. UTF-8 file naming, language codes, `lang-name`, fallback/override and live picker discovery;
3. card-only pack workflow versus a built-in language contribution;
4. `lang-font` safe leaf-name rule, lookup order, fallback behavior, and font-license requirement;
5. whole-sentence translation, variable-set preservation, BTN id order/count, and technical identifiers;
6. valid BTN ids and no literal button caps;
7. Korean-only JOSA rules and supported pairs;
8. exact local validation commands and what `pack_contract` checks;
9. failure/fallback behavior and current limitations.

Do not silently preserve stale numeric facts. Derive the current canonical key count and embedded language
set from the files/source and use the verified values in both guides. If source and the old guide disagree,
follow source and report the correction.

Add a concise contribution workflow to both guides:

- use the translation issue form to coordinate a new built-in language or a change needing discussion;
- card-only packs can be tested without changing SLOT2 code or opening an issue;
- for a built-in contribution, identify the exact files/registry/tests that current source requires, run the
  documented focused commands, and submit the change for review;
- a translation correction should name the affected keys and context;
- disclose machine/AI assistance and record human review, but do not claim authorship, copyright ownership,
  or legal conclusions on the contributor's behalf;
- fonts/assets require redistributable original license text; do not include ROMs, BIOS files, screenshots
  with unlicensed game content, secrets, generated build output, or card data;
- contributors must not weaken tests, invent missing product behavior, translate technical identifiers, or
  change English semantics merely to make a translation pass.

The guides may mention GitHub issue/PR concepts but must not fabricate public issue numbers, PR URLs, release
URLs, branch names, response times, maintainer identities, or acceptance promises.

## README link adjustment

In `README.ko.md`, change only the two `docs/TRANSLATING.md` link targets in section 8 to
`docs/TRANSLATING.ko.md`. Preserve their Korean labels and all other README bytes. `README.md` remains
unchanged and continues to link the English canonical path.

## Verification

Do not run Cargo tests, clippy, builds, packagers, workflows, or hardware checks. Product and translation
assets are unchanged, and the focused commands were already validated before this documentation task.

Perform offline metadata/document verification:

1. Parse all four issue-template YAML files with every safe local YAML parser already available. Do not
   install one. Also perform a parser-independent structural check for the GitHub issue-form keys/types,
   unique ids, supported types, nonempty labels/options, required placement, duplicate keys, and absence of
   anchors/aliases/custom tags.
2. Prove the three forms collect every required field/checklist above and contain no unsupported promise,
   guessed repository object, secret request, or prohibited upload request.
3. Prove both translation guides have reciprocal links, the same section order/material rules, the same
   commands/current key count/embedded language set, and matching contribution/safety requirements.
4. Resolve every relative Markdown link in the forms, guides, and the changed Korean README. For links
   written relative to an issue body as GitHub will render them, explicitly document the assumed repository
   root behavior and use a path form that works there. Do not contact external links.
5. Compare guide claims with the current language packs, i18n registry, pack-contract test, and button ids.
6. Reconstruct the pre-task `README.ko.md` by reverting only the two link targets and prove its SHA-256 is
   the Task106 final hash `9976df59f317ba301ba96f3f5a19fb8672b2a5b94fc6fd411dacb623d1c8fd6c`.
7. Check all new/modified text files for UTF-8 without BOM, LF-only endings, trailing whitespace, balanced
   Markdown fences where applicable, and final newline.
8. Run `git diff --check`.

After final verification, do not change files.

## Worker report

Write `C:\SLOT2\tasks\107-issue-forms-translation-contribution.worker-result.md` even on failure. Include:

- success/failure and cumulative attempt count, maximum 2;
- issue-form file list, exact field ids/types/required status, blank-issue behavior, and safety wording;
- YAML parser names/versions or availability limits plus the complete structural-schema result;
- English/Korean guide section/parity evidence, current key count, embedded language set, commands, and
  contribution workflow;
- source locations used for every technical translation rule;
- link counts/resolution, README locality reconstruction, encoding/fence/whitespace results, and
  `git diff --check`;
- commands, exit codes, last result lines, created/modified files, final verification time, hashes, and
  whether content changed afterward;
- explicit confirmation that no code, test, language pack, workflow, build, package, issue/PR, network,
  hardware, commit, push, tag, release, or shared configuration action occurred;
- remaining M7 hosted-release, fresh-card acceptance, and migration-guide work.

Do not commit.
