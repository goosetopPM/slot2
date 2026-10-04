# Task 106 attempt 2 - Remove the broad atomic-write promise

This is cumulative attempt 2/2. Fix only the public data-safety overclaim identified by Codex in
`C:\SLOT2\tasks\106-public-readme-en-ko.result.md`. Keep all other successful attempt-1 content and update
the existing worker report as a cumulative 2/2 report.

Read only:

- `C:\SLOT2\tasks\106-public-readme-en-ko.md`, shared README and verification contracts
- `C:\SLOT2\tasks\106-public-readme-en-ko.result.md`
- `C:\SLOT2\README.md`, card-layout paragraph and directly related context
- `C:\SLOT2\README.ko.md`, matching paragraph and directly related context
- `C:\SLOT2\crates\slot2-store\src\lib.rs` and `atomic.rs`, write-safety scope
- `C:\SLOT2\crates\slot2\src\diag.rs` and `probe.rs`, card-report write paths
- `C:\SLOT2\tasks\106-public-readme-en-ko.worker-result.md`

Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate, commit,
push, publish, create a tag, use the network, access hardware, or change shared configuration. This is the
last allowed attempt. Wait up to 5 minutes for the first response and 45 minutes total.

## Allowed files

- Modify `README.md` only in the paragraph immediately after the card-layout tree.
- Modify `README.ko.md` only in the matching paragraph.
- Update `tasks/106-public-readme-en-ko.worker-result.md` as the cumulative attempt 2/2 report.

Do not modify product code, tests, Cargo files, workflows, build scripts, assets, license files, other docs,
or any Task 106 instruction/result file.

## Exact correction

The current English paragraph says every write SLOT2 makes is atomic and a card pulled mid-write cannot
leave half a file. The Korean paragraph makes the equivalent claim. That is broader than the application:
diagnostic and probe reports use direct writes outside `slot2-store`.

In both languages:

- preserve the accurate statement that only `System/` comes from the release zip and the other top-level
  content belongs to the user or is created on first use;
- remove the claim that every SLOT2 write is atomic;
- remove the blanket physical-card-removal / half-file guarantee;
- either stop after the ownership/creation statement, or state only that save data, states, thumbnails, and
  settings managed by `slot2-store` use its temporary-file-and-rename path;
- if the narrower implementation detail is retained, do not call it a guarantee for diagnostic/probe files,
  arbitrary files, power loss, filesystem corruption, or safe card removal;
- keep the English and Korean meaning equivalent and do not change any other README content.

Do not add a new warning, troubleshooting item, implementation essay, or legal disclaimer. This should be
a minimal public wording correction.

## Focused verification

Do not run Cargo tests, clippy, builds, packagers, workflows, or hardware checks.

1. Prove the two corrected paragraphs have equivalent facts.
2. Scan both READMEs and confirm that no broad phrase remains: `every write`, `all writes`, their Korean
   equivalent, or any claim that pulling/removing a card cannot leave a partial file.
3. If a narrow atomic-write statement remains, compare every named file class with the public
   `slot2-store` paths and confirm diagnostic/probe output is excluded.
4. Re-run the attempt-1 README parity/link/forbidden-claim/UTF-8/LF/whitespace/fence checks because the file
   bytes changed. External links remain syntax-only and must not be contacted.
5. Run `git diff --check`.

After final verification, do not change files.

## Cumulative worker report

Update `C:\SLOT2\tasks\106-public-readme-en-ko.worker-result.md` and clearly state attempt 2/2. Preserve the
useful attempt-1 evidence and add:

- the prior overbroad data-safety claim and the exact final English/Korean wording;
- source evidence distinguishing `slot2-store` atomic writes from direct diagnostic/probe writes;
- all focused verification results, commands, exit codes, final README hashes/sizes/times, and whether
  content changed afterward;
- confirmation that no other README paragraph or repository file was changed in attempt 2.

Do not commit.
