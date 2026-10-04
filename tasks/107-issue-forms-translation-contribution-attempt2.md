# Task 107 attempt 2 - Fix issue-context translation guide links

This is cumulative attempt 2/2. Fix only the issue-context link defect identified by Codex in
`C:\SLOT2\tasks\107-issue-forms-translation-contribution.result.md`. Keep all other successful attempt-1
content and update the existing worker report as a cumulative 2/2 report.

Read only:

- `C:\SLOT2\tasks\107-issue-forms-translation-contribution.md`, issue-form and verification contracts
- `C:\SLOT2\tasks\107-issue-forms-translation-contribution.result.md`
- `C:\SLOT2\.github\ISSUE_TEMPLATE\translation.yml`
- `C:\SLOT2\tasks\107-issue-forms-translation-contribution.worker-result.md`

Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate, commit,
push, publish, create a tag, open an issue or pull request, use the network, access hardware, or change
shared configuration. This is the last allowed attempt. Wait up to 5 minutes for the first response and 45
minutes total.

## Allowed files

- Modify `.github/ISSUE_TEMPLATE/translation.yml` only to replace translation-guide link targets.
- Update `tasks/107-issue-forms-translation-contribution.worker-result.md` as the cumulative attempt 2/2
  report.

Do not modify the other issue forms/config, either translation guide, either README, code, tests, language
packs, workflows, build scripts, assets, license files, project docs, or any Task 107 instruction/result
file.

## Exact correction

GitHub renders issue-form markdown in the issue creation/issue context, not relative to the YAML file or a
repository-root Markdown file. The repository's default branch is `main`.

In every Markdown/checklist occurrence inside `translation.yml`, replace only the two guide targets:

```text
docs/TRANSLATING.md       -> ../blob/main/docs/TRANSLATING.md
docs/TRANSLATING.ko.md    -> ../blob/main/docs/TRANSLATING.ko.md
```

Preserve all visible English/Korean link labels, prose, form fields, ids, types, options, required flags,
safety wording, ordering, and whitespace outside those target substitutions. Do not add an owner/repository
absolute URL, raw URL, release URL, branch guess, contact link, or new field.

The resulting relative targets follow GitHub's official same-repository issue/comment form: from an issue
URL, `../blob/main/<path>` resolves to the file view on the main branch. Do not apply this issue-context form
to README or guide links; those are rendered from repository files and already use the correct file-relative
paths.

## Focused verification

Do not use the network or run Cargo tests, clippy, builds, packagers, workflows, or hardware checks.

1. Parse `translation.yml` again with the attempt-1 strict parser and re-run all issue-form structural
   assertions for that file.
2. Prove every translation-guide link occurrence in the form uses exactly one of the two allowed
   `../blob/main/docs/...` targets. Prove no bare `](docs/TRANSLATING`, absolute URL, owner/repository URL,
   raw URL, or other branch appears.
3. Prove that replacing the new targets with the old bare `docs/...` targets reconstructs the attempt-1
   `translation.yml` hash `4dfc6ffd03601d5e742fe74eef105ef4d30408fda7a8f4178283b8285d0dd07a`.
4. Re-run the complete attempt-1 offline verification suite so guide parity, required fields, safety scans,
   README locality, encoding, line endings, fences, and whitespace remain valid. Update its link-resolution
   rule for issue-context links; do not contact GitHub.
5. Re-read the unchanged `README.md` metadata and record its actual Task 106 final values: 16899 bytes and
   SHA-256 `e10a30e64b506d03a07368d9a0cac7c451215c52aea58fea222cb677a4282081`.
6. Run `git diff --check`, while retaining the report's note that untracked files need the direct text-file
   scans.

After final verification, do not change files.

## Cumulative worker report

Update `C:\SLOT2\tasks\107-issue-forms-translation-contribution.worker-result.md` and clearly state attempt
2/2. Preserve useful attempt-1 evidence and add:

- the incorrect repository-root assumption and the final issue-context target rule;
- exact link occurrence counts and reconstruction hash evidence;
- the corrected unchanged `README.md` size/hash/time read from disk;
- complete focused/full verification results, commands, exit codes, final hashes/sizes/times, and whether
  content changed afterward;
- confirmation that only `translation.yml` link targets and this cumulative report changed in attempt 2.

Do not commit.
