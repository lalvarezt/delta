# Keeping the delta fork maintainable

The fork remains justified by the measured improvements to the personal
lazygit workflow. Keeping it current at any cost is harder to justify. My
recommendation is to retain the fork with a narrow scope and accept deliberate
upstream lag when an integration fails the performance gate.

This is a recommendation, not a change to merge policy. The strict gate remains
in force, and the rejected upstream candidate remains unlanded.

## What earns its maintenance cost

The [performance investigation](fork-performance.md) records useful gains from
compact alignment storage, direct style-range composition, scope-style caching,
and the native wrapper adapter. The complete-wrapper comparisons and installed
editor checks establish benefits beyond a synthetic renderer benchmark.

For example, the scope-cache follow-up reduces a complete Go-history wrapper
call from about 1,005 ms to 548 ms. Later integration checks preserve copying,
editing, staging, and row identities. These are historical incremental
comparisons, not a fresh measurement against today's stock upstream executable.
The old runner's configured results have a documented feature-selection bug;
they cannot establish that Git-selected feature sets were active. The separate
complete-wrapper measurements remain useful evidence.

The [source inventory](upstream-assessment-2026-11-11.md) shows a concentrated
implementation. At assessment start `ce4a212`, the merge-base diff against the
pinned `upstream/main` modifies seven Rust files, including tests. Most added
lines in the repository diff are benchmark evidence and documentation. Raw
commit counts and total diff size overstate the runtime maintenance scope.

Reproduce that inventory with:

```sh
git diff --stat 876a1624fed217d32c35007cdfbe1bf7a7ecb6c8...ce4a212
git diff --numstat 876a1624fed217d32c35007cdfbe1bf7a7ecb6c8...ce4a212 -- src
```

## What makes the burden expensive

The candidate merged without textual conflicts. The expensive work was
establishing behavior and performance across configuration, rendering, ANSI
ingestion, and the actual lazygit consumer. Corrected upstream output can make
downstream work more expensive even when our optimizations remain intact.
That is a recurring integration risk.

The [failed-gate report](upstream-benchmarks-2026-11-11.md) also shows the limit
of the current approach. Pursuing parser backends, dependency patches, and
lazygit optimizations did not produce a complete acceptable candidate. Making
every merge depend on solving all such performance problems would create an
open-ended maintenance commitment.

Some repeated work came from our own measurement process. Incorrect feature
activation invalidated configured runs, and a shared build directory left a
stale executable that required rebuilding and identity checks. These are
repairable process costs. They do not establish that the fork's core design is
too expensive to maintain.

We have not recorded engineering hours or a maintenance budget, so the evidence
does not support a numerical return-on-effort claim.

## Recommended limits

Keep the existing optimizations and caller contracts. Concentrate new work on
measured problems in the deployed workflow. Treat parser replacements and
cross-repository consumer changes as separately scoped work.

Set an investigation budget before each upstream integration. A candidate that
still fails at that limit stays unlanded, with pinned evidence and explicit
missing fixes. Your no-regression requirement does not require unlimited work
on each incoming change.

Prioritize security and required correctness work over optional new behavior.
The deleted-file highlighting defect remains unresolved under the combined
correctness and performance requirements. Delaying the merge preserves current
speed, but it does not fix that defect. Any selective backport still needs its
own correctness review and performance validation.

Retain the strict gate over documented callers and representative fixtures.
Passing that coverage provides evidence; it cannot prove performance for every
possible input. Keep measurement noise separate from repeatable losses.

Reconsider the fork if required security or correctness fixes repeatedly cannot
be integrated within the maintenance budget, or if a validated upstream build
provides equivalent behavior and performance. The work so far supports keeping
the fork with these limits. It does not support a promise of immediate upstream
parity at zero performance cost.
