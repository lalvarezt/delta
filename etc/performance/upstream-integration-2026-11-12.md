# Accepted upstream integration, 2026-11-12

The user accepted the measured costs after reviewing the upstream behavior
changes and requested further improvements where justified. This integration
therefore proceeds with the costs disclosed below. It does not receive a clean
no-regression performance verdict.

This decision supersedes the earlier requirement to leave this candidate
unlanded. It applies to this reviewed integration, not to arbitrary future
regressions.

## Revisions and reproducibility

| Role | Pinned identity |
| --- | --- |
| Task start and pre-merge fork | `96cee58f4715279586424fd471974a80ae17d4c6` |
| Approved upstream | `876a1624fed217d32c35007cdfbe1bf7a7ecb6c8` |
| Merge base | `ae44d3f007a4bef28c734737189eced85ee02f36` |
| Previously measured candidate tree | `91dee84ade0aff747afbe5b2e73ee064fc651d48` |
| Unchanged lazygit source | `eb0320973ffe795970ad95c047ad147a15c53e81` |

The clean merge changes the same 17 upstream files reviewed in the
[source assessment](upstream-assessment-2026-11-11.md). Its runtime source,
manifests, lockfile, build recipes, and workflows match the measured candidate.
The preserved task-start executable and final executable also match the
previously measured pair byte for byte:

```text
before aeb791234ffc2039aea6ddcc0b6e93ac9a8339dc51d09b1cac60fd1099d968a1
after  3eece00bfeadcd76ebf6fe9c0ded2b6f9c5beece1c08bc96484552c1c2e88417
```

Both the original assessment start and this task start therefore use the same
baseline executable. The [paired benchmark evidence](data/upstream-benchmarks-2026-11-11.json)
remains applicable. It includes the frozen inputs, effective configuration,
sampling controls, rejected invalid runs, and raw samples. These are reused
measurements, not newly collected timings.

## Correctness and retained improvements

The merge restores syntax highlighting for deleted source files, honors
feature-defined line-number formats with CLI side-by-side mode, adds opt-in
per-hunk layouts, and updates tmux hyperlink guidance. All upstream intent is
retained.

Compact alignment storage, direct range composition, complete-scope theme
caching, the native lazygit adapter, dependency repairs, and release
optimization remain intact. The independent benchmark-runner fixup `8d475aa`
also remains in the history.

Fresh validation passes `just format`, `just check`, and all 24 caller contracts.
There are 516 passing Rust tests, 8 ignored tests, and the same four documented
Clippy warnings. Exact executable identity supports reuse of the earlier
output matrix, adapter edge checks, streaming probes, and lazygit consumption
checks. Their coverage and limits remain in the
[benchmark report](upstream-benchmarks-2026-11-11.md).

## Accepted costs and bounded optimization work

The large deleted-file case now parses and colors source that previously lost
its language at `/dev/null`. The measured unified case increases from 41.30 to
446.43 ms. The corrected side-by-side feature precedence also changes styled
gutters and increases formatting and lazygit consumption work. The reported
consumer cases increase from 33.43 to 43.14 ms and from 441.14 to 578.70 ms.
These are separate measurement boundaries, not additive interactive timings.

The previous experiments already tested alternative regex backends and native
default match parameters. One alternative is substantially slower; the small
improvement from dependency patches leaves nearly all the deleted-file cost
and does not justify adopting those patches. No experimental parser or
dependency changes are included.

Equivalent-style renderer checks are steady. The clearest avoidable follow-up
is lazygit's rejection of complete SGR escapes followed by ordinary Unicode
gutter characters. The boundary-only prototype preserves combining-grapheme
fallbacks and is promising, but still needs complete paired and caller checks.
The more stateful SGR-cache experiments remain exploratory.

The requested [lazygit follow-up note](../../../lazygit/docs-master/dev/Delta-ansi-ingestion-follow-up.md)
records the proposed changes, prototype location, evidence, and required checks.
It is deliberately uncommitted in that repository. No lazygit source or commit
was changed. Its consumer optimization remains open.

The new hunk mode remains opt-in. Its documented lookahead cost is unchanged;
personal configuration was not changed to enable it.

## Evidence and limits

The [acceptance verification](data/upstream-integration-2026-11-12.json) records
pins, executable identities, fresh checks, and the uncommitted note's identity.
Existing benchmark artifacts retain their historical failed strict-gate verdict.

Physical terminal redraw, interactive editor actions, and Windows were not
revalidated for this integration. The prior raw-history baseline failure and
memory-sampling limitations are unchanged. This commit does not install or
push either executable or repository.
