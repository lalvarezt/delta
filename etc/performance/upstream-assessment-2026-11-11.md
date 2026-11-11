# Upstream merge assessment, 2026-11-11

I recommend approving the merge, with the validation below required before
calling the integrated result ready. Git's merge simulation reports no conflicts,
and source review finds that it retains the fork's customizations. The candidate
has not been built or tested. No upstream merge, installation, push, or personal
configuration change has been performed.

## Pinned revisions

| Role | Revision |
| --- | --- |
| Task start and pre-merge `experiments` | `6e8a1c5c5ecf993d4a29d4e6d8b513eb2c4b1cde` |
| Fetched `upstream/main` | `876a1624fed217d32c35007cdfbe1bf7a7ecb6c8` |
| Merge base | `ae44d3f007a4bef28c734737189eced85ee02f36` |
| Simulated merge tree | `848cabc9e1692025cc3713db7d5dd8667f1f7059` |

The task started with a clean tree. Fetches confirmed that `origin/experiments`
matches the task-start revision. Local `main` remains at `c4e9a8d`, two commits
behind fetched upstream. Branch tracking already matches the repository rules.
This report is a separate investigation commit on `experiments`.

Upstream contributes four commits since the merge base. The complete upstream
diff changes 17 files, with 809 insertions and 89 deletions. The simulated result
has the same diff statistics against the task-start fork. The fork-side review
covers all 20 commits and all 35 changed paths since the merge base, including
source, dependencies, build settings, workflow changes, and recorded evidence.

## Incoming behavior

| Commit | Effect | Assessment |
| --- | --- | --- |
| `6caa28a` | Preserve the removed file's language when the new path is `/dev/null`. | Correctness fix. Deleted source files can now receive syntax colors when the minus style requests syntax. Configured output intentionally changes. |
| `c4e9a8d` | Add `--side-by-side=hunk` and `hunk,WIDTH`. Mixed replacement hunks use two panels. Addition-only and deletion-only hunks use one column. | Useful opt-in behavior. Introduces per-hunk lookahead, mutable painter layout, and separate unified line-number formats. |
| `c604395` | Update tmux hyperlink guidance in CLI help and the manual. | Documentation only. |
| `876a162` | Keep user feature formats when CLI flags enable side-by-side and other non-style features. | Intentional precedence change. Raw and other style flags retain priority. Validate other affected flags as well as side-by-side. |

The hunk-layout commit also makes a numeric CLI `--width` control minimum-width
selection when output is captured. A width from Git configuration does not take
part in that early decision. A value in the main `[delta]` section takes
precedence over the side-by-side feature, including `DELTA_FEATURES` activation.
Combined diffs and merge conflicts keep the configured side-by-side layout.
The precedence change affects `--hyperlinks`, `--keep-plus-minus-markers`,
`--line-numbers`, `--navigate`, and `--side-by-side`.

## Fork customization accounting

| Customization | Simulated result | Required integration coverage |
| --- | --- | --- |
| Compact alignment table and exact tie-breaking | `src/align.rs` is byte-identical to the fork. | Keep exhaustive alignment tests and long-line, Unicode, and unpaired fixtures. |
| Range-based style composition | The optimized implementation remains in `src/paint.rs`. | Exact ANSI bytes, empty sections, UTF-8 boundaries, and final newlines. |
| Complete-scope theme cache | `src/highlight.rs` is unchanged. Painter calls retain `CachedHighlighter` and reset parser state at hunk boundaries. | Deleted files, alternating file languages, mixed layouts, cache reuse, and existing eviction tests. |
| Native lazygit adapter | `src/lazygit.rs` and its stdin dispatch are unchanged. The CLI flag remains excluded from Git-config option processing. | Statistics, tab markers, graph colors, hyperlinks, output errors, and the configured wrapper. |
| Updated dependencies and git2 0.21 compatibility | `Cargo.toml` and `Cargo.lock` are unchanged. The clean merge retains all three `.ok()` adaptations, including the side-by-side reader. | Compile and test the merged result against the retained lockfile. |
| Thin LTO and one release code-generation unit | Release profile is unchanged. | Use matching release settings for paired measurements. |
| Local checks, CI restrictions, and 32-bit runner selection | Workflow files, `justfile`, and `AGENTS.md` are unchanged. | Keep checks local. Cross-platform behavior remains unverified here. |
| Fork documentation and benchmark runner | Existing files are unchanged. | Account for intentional output changes before using the exact-byte benchmark gate. |

OSC 1717 row identities and OSC 8 links are additional caller contracts already
present at the merge base. Their emitter is unchanged, but new layout transitions
change its callers. Source review checks both real calls to
`paint_minus_and_plus_lines`, including merge conflicts. Both receive the mutable
layout. The remaining textual occurrence is inside a combined-diff test fixture.
The previous hunk's buffered lines are painted before switching layout.

## Correctness evidence

The current fork passes `just check` on Linux x86-64 with Rust 1.99.1. It reports
501 passed tests, 8 ignored tests, and the same four documented
`needless_borrow` Clippy warnings in `src/utils/process.rs` tests.

The deprecated-options shell test passes. The pager-history test passes through
a PTY with isolated paths. Its temporary adapter changes only the deletion of
`~/.lesshst` to a temporary path, and redirects `XDG_DATA_HOME` and `LESSHISTFILE`.
The real history files were not touched.

The full-history raw-output shell test still fails on the baseline. Its diff
contains 21 omitted binary-file notices, matching the documented failure class.
This assessment does not repair that existing issue or claim that the candidate
has the same failure body. Compare both binaries on one frozen history after
integration.

No new correctness defect was established by source review. A conflict-free
merge is not a correctness verdict. Upstream adds 15 test functions covering
deleted-file syntax, mixed and one-sided hunks, layout switches after buffered
lines, line numbers, odd widths, width thresholds, feature precedence,
`--color-only`, combined diffs, merge conflicts, and `--show-config`.
Those incoming tests have not been run here.

## Performance assessment

No before-and-after performance claim is made. The pre-merge release executable
has been saved, with its SHA-256 recorded in the evidence file. Historical fork
measurements describe earlier builds and do not validate this merge.

Source review identifies three costs to measure:

- Hunk mode copies and queues input until it can decide the layout. A large
  one-sided hunk with context can require reading nearly the whole hunk before
  emitting its body. Queue memory grows with the bytes inspected. A zero-length
  side decides immediately, and mixed hunks stop lookahead once both signs appear.
- Feature gathering now runs a second time to resolve unified line-number
  formats. This setup occurs even when hunk mode is not selected. Check tiny
  processes and repeated configured invocations, where setup is most visible.
- Correct syntax highlighting of deleted files performs parsing and style work
  that was previously skipped. Measure this correctness-required cost separately
  from regressions on unchanged output.

The native lazygit adapter already buffers all input to compute statistics.
Hunk lookahead does not add another producer EOF barrier to that path, but its
queue and scans still add work and memory. Direct delta needs separate tests
for delayed input, first body output, completion, cancellation, and errors.

## Coverage required after approval

Approval covers integrating the pinned upstream revision while preserving the
fork behavior, followed by these checks:

1. Fast-forward local `main` to the pinned upstream revision and merge it into
   `experiments`. Preserve this report commit and the reviewable history.
2. Run `just format`, `just check`, and the relevant shell tests. Compare the
   raw-output baseline failure on identical frozen input. Keep pager paths isolated.
3. Check exact bytes for unchanged output under isolated and actual configured
   settings. Include ordinary and combined diffs, tabs, Unicode, unpaired and
   long lines, syntax disabled, hyperlinks, and negotiated OSC 1717 metadata.
4. Assert intentional output changes separately: deleted-file syntax, CLI
   feature formats, explicit-width thresholds, and per-hunk layout. The current
   comparison runner rejects any output difference, so those cases need explicit
   expectations before timing. Do not remove history cases because they contain
   the newly corrected deleted-file output.
5. Add caller probes for mixed-to-unified-to-mixed transitions across files and
   hunks, odd widths, custom unified and panel formats, and row identities used
   by lazygit actions. Check the full configured wrapper, including failure status.
6. Measure complete configured and isolated callers with matched fixtures,
   configuration, toolchain, and release settings. Alternate paired runs and
   retain raw samples, identities, commands, controls, and timing boundaries.
   Measure large-context lookahead, setup, deleted-file parsing, and memory
   separately. Recheck material losses before accepting the result.

The report does not authorize installing the merged binary or changing personal
rendering settings. No SSH access is needed for the local assessment. Windows
and other platforms, actual lazygit actions, and the merged performance matrix
remain coverage gaps until they are exercised.

## Evidence and reproduction

The [assessment evidence](data/upstream-assessment-2026-11-11.json) records full
revision identities, commit inventories, dependency preservation, the baseline
binary hash, toolchain, and shell-test results. Transient logs, both complete
diffs, and the saved binary are under `/tmp/delta-upstream-assessment`.

These commands reproduce the source comparison against the pinned revisions:

```sh
git log --reverse --oneline ae44d3f..876a162
git log --reverse --oneline ae44d3f..6e8a1c5
git diff --stat ae44d3f 876a162
git diff --stat ae44d3f 6e8a1c5
git diff ae44d3f 876a162
git diff ae44d3f 6e8a1c5
git merge-tree --write-tree 6e8a1c5 876a162
```
