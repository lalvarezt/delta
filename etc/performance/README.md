# Validate delta changes

For local edits and upstream integration, use the shared protocol at
`~/.agents/skills/merge-upstream/references/regression-validation.md`.
Read the [fork analysis](fork-performance.md) for configured callers,
customizations, baseline failures, and measurement limits.

## Caller coverage

- Rendering: tiny and ordinary diffs, long lines, Unicode, tabs, and unpaired
  changes; unified and side-by-side modes, syntax paths, pairing, and
  tie-breaking.
- Reuse: first and repeated work; syntax, scope-stack, theme, width, and color
  changes under isolated and configured settings.
- lazygit: colored input, wrapper behavior, OSC 1717 row identities, OSC 8
  links, statistics, and consuming actions; distinguish combined diffs.
- Streaming: startup, delayed first output, completion, cancellation,
  errors, and memory across the configured producer, wrapper, and renderer.

Preserve exact bytes for unchanged rendering. Assert intentional output
changes and add missing caller probes.

## Checks and paired builds

Run `just check` and relevant shell tests under `tests/`. Isolate pager-history
files. Verify recorded baseline failures on the current baseline.

Save each local baseline release build before editing; retain the task-start
build and use separate output paths for successive comparisons:

```sh
just build
cp target/release/delta /tmp/delta-before
# Make the change, then:
just build
node etc/performance/compare.mjs \
  --before /tmp/delta-before --after target/release/delta \
  --output /tmp/delta-comparison.json
```

The Node.js 20+ runner checks exact output, warms both binaries, and alternates
15 measured runs. JSON retains samples, deviations, hashes, and revisions;
mismatches or failures stop the run. Record additional working-tree changes
and controls.

## Configuration and scope

Defaults are `--no-gitconfig`, dark true color, 100 columns, and no pager;
Bat caches remain environmental. Relevant options:

- `--gitconfig`: actual Git configuration.
- `--lazygit /path/to/lazygit --colored --hyperlinks`: lazygit input and links.
- `--fixture tiny --mode side-by-side --runs 50`: recheck a short case.

Record wrapper and installed-tool identities for integrated tests. Keep builds
and tests idle during timing. The runner measures startup and complete output
to a discarded stream, excluding live producers, first output, terminal parsing,
and editor redraw. Probe those separately; measure moved setup and refresh work
and memory separately too.

## Historical throughput

<img src="./visualization.svg">
