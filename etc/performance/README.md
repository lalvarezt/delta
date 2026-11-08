# Performance measurements

Read [the fork analysis](fork-performance.md) for measured results and priorities
for lazygit and Neovim.

## Compare two builds

1. Build and save the original executable before changing the source:

	```sh
	cargo build --release
	cp target/release/delta /tmp/delta-before
	```

2. Apply the change and build the candidate:

	```sh
	cargo build --release
	```

3. Compare the executables:

	```sh
	node etc/performance/compare.mjs \
	  --before /tmp/delta-before \
	  --after target/release/delta \
	  --output /tmp/delta-comparison.json
	```

The runner requires Node.js 20 or later. It checks exact output equality before
timing each case, warms both binaries, and alternates their order across 15
measured runs. The JSON records every sample, median absolute deviations,
fixture hashes, binary hashes, and source revisions. A mismatch or command
failure stops the run.

By default, delta runs with `--no-gitconfig`, a dark background, true color,
100 columns, and no pager. Bat's syntax and theme cache still comes from your
environment. Add `--gitconfig` to measure your Git configuration. Add
`--lazygit /path/to/lazygit` to include Go changes from that checkout.
For lazygit's colored input and editor links, add `--colored --hyperlinks`.

To repeat a small case with more samples, add `--fixture tiny`,
`--mode side-by-side`, and `--runs 50`. Repeat `--fixture` or `--mode` to select
several cases.

These measurements cover process startup and complete rendering to a discarded
output stream. They do not measure editor redraw, terminal parsing, a live Git
producer, or time to first output. Keep other builds and tests idle during timing.

## Historical throughput

The older history benchmark and chronologer visualization remain available:

<img src="./visualization.svg">
