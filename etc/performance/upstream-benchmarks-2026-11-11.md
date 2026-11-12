# Upstream performance gate, 2026-11-11

On 2026-11-12, the user accepted the measured costs after reviewing their causes.
The [subsequent integration report](upstream-integration-2026-11-12.md) records
that decision. The strict-gate result and measurements below remain historical.

The upstream merge fails the required performance gate and remains unlanded.
No correctness-cost exception applies. The tested changes do not preserve
before-and-after performance for every affected caller.

The working fork retains its Rust implementation. The only landed repair from
this investigation is `8d475aa`, a separate fixup for the benchmark runner.
No installation or push was performed. Experimental renderer binaries and
lazygit overlays were kept outside the working source trees.

## Pinned comparisons

| Role | Revision or identity |
| --- | --- |
| Benchmark task start | `c07cd0ab8c992e9a7ef93fc31d216fc9510f0bc0` |
| Fork after the independent runner fix | `8d475aa98ff8785501520eea57e83436ee7513f1` |
| Upstream candidate | `876a1624fed217d32c35007cdfbe1bf7a7ecb6c8` |
| Merge base | `ae44d3f007a4bef28c734737189eced85ee02f36` |
| Tested merge tree | `91dee84ade0aff747afbe5b2e73ee064fc651d48` |
| lazygit source and installed commit | `eb0320973ffe795970ad95c047ad147a15c53e81` |

The task-start and current fork have identical Rust sources, Cargo manifests,
lockfiles, and release binaries. The original assessment start at `6e8a1c5`
also uses that baseline executable. This compares the candidate against both
the task start and the pre-merge fork, rather than an older historical base.

The [source assessment](upstream-assessment-2026-11-11.md) accounts for all
upstream changes and fork customizations. The candidate retains compact
alignment storage, range-based style composition, complete-scope theme caching,
the native lazygit adapter, dependency repairs, release optimization, and the
local-check workflow. Retaining those implementations does not establish that
the complete new callers have retained their performance.

## Measurements

The [interactive comparison](data/upstream-benchmarks-2026-11-11.html) separates
complete callers, renderer costs, and downstream ingestion.

Inputs, binary copies, Git configuration, and source histories were frozen
before integration. Histories use the pinned delta and lazygit revisions.
The configured runs explicitly load the frozen configuration and assert that
both the Catppuccin feature and theme are active.

Calls run serially on CPU 2, with builds and tests idle during timing. Both
release builds use Rust 1.99.1, thin LTO, and one code-generation unit. The
governor remains `powersave`; frequency is not locked. Raw samples retain
variation rather than treating every difference in medians as a regression.

The initial matrices use three warmups and 15 alternating pairs. Suspicious
isolated cases receive 101 pairs. Further comparisons align styles, use
identical arguments where needed, and retain the original slower results.

The 66-case output matrix has 44 exact-byte matches and 22 intentional output
changes. Checks for the changed output distinguish restored deleted-file
syntax colors, corrected CLI feature precedence, and new hunk layouts.
Comparisons against equivalent baseline Git-config activation verify the
feature-precedence change. Deleted-file differences are restricted to SGR
colors on the affected content and hunk-header rows.

## Performance preserved in existing paths

These are complete wrapper calls through the frozen deployed script, including
native statistics, tab markers, graph colors, hyperlinks, and row metadata.
Values are medians in milliseconds. No loss was detected by the paired checks.

| Input | Before | Candidate |
| --- | ---: | ---: |
| 30-line Go change | 14.94 | 14.72 |
| 1,000-line Go change | 81.78 | 81.14 |
| 12,000-line Go change | 863.34 | 856.00 |
| Colored Go history | 1320.38 | 1315.20 |

The initially slower isolated cases did not reproduce as losses in the
101-pair rechecks. With equivalent configured styles, 21-pair native
side-by-side rechecks were also steady. A 51-pair Rust-history recheck using
identical CLI arguments measured 477.735 ms before and 477.748 ms after, with
no detected loss.
An earlier 35-pair style-aligned Rust run showed a 0.79% paired median loss;
the tighter identical-argument recheck did not reproduce it.

These findings do not cancel the losses below. Equivalent-output comparisons
separate implementation overhead from the cost of changed output. Actual
calls still have to process that changed output.

## Regressions that block integration

| Caller | Before | Candidate | Finding |
| --- | ---: | ---: | --- |
| Deleted 12,000-line Go file, unified | 41.30 ms | 446.43 ms | Restoring syntax highlighting adds substantial parsing and painting work. |
| Deleted 12,000-line Go file, side-by-side | 117.51 ms | 568.55 ms | Restored syntax and corrected gutter styles exceed the baseline. |
| Configured Rust history, CLI side-by-side | 474.97 ms | 491.41 ms | Corrected feature precedence changes the gutters and their formatting cost. |
| Configured Go history, CLI side-by-side | 1817.22 ms | 1859.06 ms | The actual caller is slower despite steady equivalent-style comparisons. |
| lazygit fresh true-color view, 1,000-line Go change | 33.43 ms | 43.14 ms | Ingestion and identity extraction process the corrected ANSI stream. |
| lazygit fresh true-color view, 12,000-line Go change | 441.14 ms | 578.70 ms | The downstream cost also exceeds the baseline. |

The last two rows measure view population and diff identity extraction in the
actual clean lazygit source. They exclude renderer execution, physical redraw,
and user-driven actions. They must not be added to unrelated medians from
other runs or presented as complete key-to-pixel timings.

For the 12,000-line side-by-side input, corrected output grows from 15,016,960
to 19,125,954 bytes, and from 743,586 to 839,144 SGR sequences. The existing
SGR fast path also rejects every sequence followed by a non-ASCII character,
including ordinary gutter box characters. Longer RGB escapes then take its
streaming path one character at a time.

The native 12,000-line side-by-side sample has disagreeing summary measures:
its ratio of medians looks faster, while its paired ratios indicate a loss.
Both distributions remain in the evidence. That case is not counted as a gain.

## Optimization experiments

No parser experiment meets the deleted-file baseline. No experimental
dependency patch or lazygit change has been adopted.

| Experiment | Result |
| --- | --- |
| Supported fancy-regex backend in an isolated build | Exact output on the deletion probe, but about 984 ms versus 400 ms with Oniguruma. Rejected. |
| Native stack-allocated default regex parameters | Exact output; about 403 ms to 393 ms. This does not remove the regression or justify adopting the dependency patches. |
| SGR cache keyed by complete incoming style | Passes view tests; partial ingestion improvement, insufficient to establish the required gate. |
| SGR operation cache and bounded validated-sequence lookup | Passes view tests; partial improvement, with more state and incomplete performance coverage. Not adopted. |
| Precise Unicode boundary check using the existing grapheme engine | Passes view tests and original chunk-boundary snapshots. Promising, but not a complete accepted integration. |

The boundary-only prototype measures 387.69 ms on the old large-case stream
and 387.33 ms on the corrected stream within that prototype. Its smaller case
still differs, at 30.76 ms and 32.21 ms. Cross-code paired confirmation and
broader performance checks are required before claiming that the downstream
regression is resolved. It does not address deleted-file parsing.

The [boundary prototype patch](data/lazygit-sgr-unicode-boundary-prototype.patch)
is reviewable separately. It examines only the SGR terminator and the next
complete rune. Combining marks and incomplete UTF-8 retain the streaming path.
Tests compare every selected chunk boundary against snapshots produced by the
original consumer, rather than comparing the prototype against itself.

## Streaming, memory, and correctness

Existing direct and native paths retain first-body behavior under a controlled
200 ms producer pause. Direct rendering measures about 11 ms to first body in
both builds. Native mode retains its existing input EOF barrier at about 207 ms.
Cancellation and closed-output behavior match.

The new `--side-by-side=hunk` mode has a separate cost. When a producer pauses
midway through an undecided one-sided hunk, first body changes from about
10 ms in unified mode to 205 ms with hunk selection. Once all context is
available, it can decide without waiting beyond the hunk. This new option was
not enabled in personal configuration.

Seven alternating memory probes sample Linux `VmHWM` every 2 ms. Median sampled
peaks are essentially unchanged on long lines and the native 12,000-line case.
The one-sided context case changes from 18,520 to 18,632 KiB. Sampling can miss
a final peak; these are renderer-process values, not aggregate pipeline memory
or proof of bounded memory for arbitrary lookahead prefixes.

The candidate passes `just check`: 516 tests pass, 8 are ignored, and the four
documented Clippy warnings remain. The preserved fork passes with 501 tests.
Additional checks cover 24 caller contracts, 24 adapter edge comparisons,
three actual lazygit identity-consumption cases, deprecated options, and
isolated pager history. The known raw-history failure retains the same body
with 21 omitted binary-file notices after excluding two volatile diff headers.

Windows, physical terminal redraw, and interactive staging, copying, and
editing were not revalidated. Perf recording was unavailable with the current
permissions, so attribution uses controlled caller and option comparisons.

## Independent benchmark repair

The old `compare.mjs --gitconfig` cleared `DELTA_FEATURES`, disabling feature
sets selected by Git configuration. A real-feature probe fails with the
original runner and passes with the repaired configured and isolated modes.
Fixup `8d475aa` targets `df58082`; it remains separate for review.

Historical configured results from that runner do not establish performance
with those feature sets active. Existing recorded results were not rewritten.

## Evidence and current gate

The [raw measurement evidence](data/upstream-benchmarks-2026-11-11.json) retains
samples, controls, hashes, identities, validation results, and exploratory
scripts. A local raw bundle is stored at
`.git/performance/delta-upstream-2026-11-11.tar.gz`; it includes the frozen
inputs, binaries, patches, and logs without adding other repositories' source
histories or executable files to this repository's tracked data.

I have not found a complete implementation that preserves correct upstream
output and meets the pre-merge performance budget. The merge stays blocked.
The deleted-file highlighting defect therefore remains open in `experiments`.
A complete fix must meet both the correctness and performance requirements.
