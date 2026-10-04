# Task 103 attempt 2 - Declaration-only packages and source-aware identity

This is cumulative attempt **2/2**, the final attempt for Task 103.

Read:

- `C:\SLOT2\tasks\103-rust-runtime-license-sbom.md`
- `C:\SLOT2\tasks\103-rust-runtime-license-sbom.worker-result.md`
- `C:\SLOT2\tasks\103-rust-runtime-license-sbom.result.md`
- the new `build/rust-notices.ps1` and `build/package-rust-notices.ps1`

Ignore only the orchestrator/delegation clauses in `AGENTS.md` and work directly. Do not delegate,
commit, push, publish, use the network, access hardware, or change shared configuration.

## Revised policy for packages with no packaged license text

Attempt 1 correctly found three runtime crates that declare a license but contain no license/notice file
in their crate package. The original contract required failure. Replace that policy with transparent
inclusion:

- Every third-party runtime package remains in the package set and SBOM.
- If a package has one or more original packaged license/notice files, copy them byte-for-byte and set
  `notice_status=packaged-text`.
- If a package has no such file but has a nonempty Cargo `license` declaration, create only its generated
  `PACKAGE.txt`, set `notice_status=declared-only`, keep its exact unmodified license expression, and record
  that the crate package supplied no license text. Include repository/homepage when Cargo metadata has it.
- Do not download, invent, normalize, select one side of `OR`, or borrow a text from another crate.
- A package with neither `license` nor `license-file` metadata still fails. A declared `license-file` that
  is missing or empty still fails.
- The policy is data-driven. Do not hard-code the three current crate names. If a later package gains a
  packaged text, it automatically becomes `packaged-text`.

Expose `notice_status` in `PACKAGE.txt`, `THIRD-PARTY-RUST.md`, and every `RUST-SBOM.json` package object.
For `declared-only`, the notice path list is empty and the human-readable files must say clearly:

```text
The crate package declares this license expression but supplied no license/notice text.
Consult the recorded upstream repository. This inventory does not replace the license and is not legal advice.
```

Postflight must reject all mismatches, including:

- `declared-only` with any notice file/path;
- `packaged-text` with an empty notice list;
- a status that disagrees with the currently cached package contents;
- omission of a declaration-only package from the directory set, SBOM, inventory, or hash manifest.

Update `CORE-NOTICES.md` and M7 wording to describe this exact limitation. Do not claim that every crate
ships an original text; say that all runtime crates are inventoried, original files are copied where the
crate package contains them, and declaration-only packages are identified explicitly.

## Fix source identity before enabling distribution

Attempt 1's tree and metadata maps use `name version` as their key. That silently collapses two packages
with the same name and version from different registry/git sources, even though the later directory-key
code claims to disambiguate them.

- Make dependency discovery and metadata matching source-aware from the point where Cargo output is parsed.
- Never select the first metadata package solely by `name version` when more than one source exists.
- Preserve both packages when the selected closure contains the same name+version from different sources.
- The stable directory keys must receive distinct deterministic source-hash suffixes, and Cargo.lock
  checksums/source metadata must be matched to the correct package.
- The independent closure comparison must also compare source-aware identities, not collapsed names.
- Add a safe pure-helper or copied-fixture verification with two synthetic packages having the same
  name/version and different registry/git sources. It must produce two entries and two distinct stable keys.
  Do not add a production environment switch or hard-coded real package list.
- If the installed Cargo version cannot expose enough source identity in `cargo tree`, fail clearly on an
  ambiguous name/version instead of silently dropping one. A clear ambiguity failure is acceptable only
  after demonstrating that the current real closure has no ambiguity; source-aware preservation remains
  required when Cargo output supplies the source.

## Complete the original Task 103 integration

After the policy and identity fixes, finish every previously blocked item from Task 103:

- generate and postflight the complete 66-package bundle;
- run all six negative bundle/dist preservation cases;
- connect `build/dist-device.ps1`, local zip, and CI device assembly to `System/licenses/rust/`;
- validate exact package set, SBOM fields/statuses, original notice bytes, and every manifest hash;
- update `CORE-NOTICES.md`, DESIGN card layout, and M7 progress within the original allowed scope.

Keep the output schema `slot2-rust-sbom-v1`; adding the required `notice_status` field is part of that
project-local schema before its first successful release. Do not modify Rust code, Cargo inputs, README,
release workflow, or existing license texts.

## Final verification

Run the original completion commands after the final code/content change:

```text
powershell -NoProfile -ExecutionPolicy Bypass -File build/package-rust-notices.ps1 -OutputDir target/task103-rust-notices-a
powershell -NoProfile -ExecutionPolicy Bypass -File build/package-rust-notices.ps1 -OutputDir target/task103-rust-notices-b
powershell -NoProfile -ExecutionPolicy Bypass -File build/dist-device.ps1 -NoBuild -Zip
git diff --check
```

Also run the safe negative tests and local CI assembly/licensing validation required by the original task.
The A/B bundles must be byte-identical, contain all 66 current third-party packages, report 63 as
`packaged-text` and 3 as `declared-only`, and include every file in `RUST-MANIFEST.txt`. The final dist must
end with `==> done`, preserve the existing six-core/source gates, and include the exact Rust bundle in the
zip. If discovery legitimately produces a different count, report the exact Cargo graph reason rather than
forcing these numbers.

Rust/Cargo inputs remain unchanged, so do not repeat workspace tests or clippy.

## Report

Update `C:\SLOT2\tasks\103-rust-runtime-license-sbom.worker-result.md` as a cumulative **2/2** report.
Preserve attempt-1 evidence and append:

- the revised declaration-only policy and the affected packages;
- the source-aware identity fix and duplicate-source fixture result;
- final package/status/file counts, Cargo.lock hash, SBOM and manifest hashes, A/B comparison;
- all negative cases and previous-output preservation evidence;
- dist/zip/CI parity and six-core/source gate results;
- the four completion commands, exit codes, and final lines;
- changed files and whether code/content changed after final verification;
- remaining legal/contract limitations and M7 work.

If this attempt fails, write the report and stop. There is no third attempt.
