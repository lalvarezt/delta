# Delta performance in lazygit and Neovim

The best immediate improvements preserve the existing rendering. The larger
opportunity is to avoid starting and configuring a renderer for every selection,
then avoid rendering work the user never sees. Syntax highlighting remains the
dominant cost on the source-history workloads measured here.

This investigation starts from delta commit `3c2269c`. The source changes are on
`experiments`. Measurements were collected before replacing the installed
executable. The fork is now installed through `just install`; the personal
rendering configuration has not been changed by this work.

## What the integrations actually run

The chezmoi source at `~/.local/share/chezmoi/dot_config/lazygit/config.yml`
puts `lazygit-delta.sh` first and difftastic second. Delta is the intended default,
with Git color enabled. The first investigation saw a deployed `rawGit` entry
ahead of those renderers. A follow-up inspection found that the deployed order
now matches chezmoi. That earlier snapshot is not the intended configuration.
The source and deployed files changed during inspection, so the saved source
revision and hashes identify the inspected snapshot. Their renderer order and
clipboard settings matched at the final check. No configuration was applied or
synchronized by this investigation.

The wrapper reads all input into
a temporary file, creates two more temporary files, strips ANSI colors to
inspect the diff, optionally runs `git apply --stat`, expands tabs, runs delta
with file hyperlinks, and applies another awk filter. It therefore prevents
delta from receiving a streaming diff and adds several processes per selection.
Optimizing delta affects the default renderer in the chezmoi configuration.
Its tab markers, generated statistics, graph colors, and editor links are custom
behavior to preserve when removing that process chain.

`dot_config/fish/functions/setup-lazygit.fish` sets both `COLORTERM=truecolor`
and `TCELL_TRUECOLOR=true`. The Git template enables Catppuccin Mocha, syntax
colors on both sides, line numbers, moved-line style mappings, histogram diffs,
and submodule logs. `.chezmoiexternal.toml` supplies the matching bat theme.
These settings explain why plain default-theme benchmarks are insufficient.

The lazygit fork advertises `OSC1717=V1`. Delta's handshake and per-row records
support file and line actions after rendering, alongside `lazygit-edit://`
hyperlinks and lazygit's clickable diff statistics. Follow-up comparisons with
that negotiation enabled preserve exact output for ordinary unified Go changes,
tabs, Unicode, side-by-side output, and the complete chezmoi wrapper. The
[integration evidence](data/fork-chezmoi-integration.json) records source hashes,
repository revisions, and individual checks.

The Neovim configuration in `~/.config/nvim/lua/plugins/snacks.lua` supplies
`builtin = false` and `cmd = { "delta" }`. The installed Snacks implementation
selects the renderer with `previewers.diff.style`. A headless inspection of the
resolved `git_diff` and `git_log` picker options returned `style = "fancy"`.
Those previews use Snacks' native renderer. The older `builtin` settings do not
select delta in this version.

The chezmoi source also supplies `git_diff_with_untracked`, combining unstaged,
staged, and untracked changes. Untracked previews read at most 1 KiB and preserve
file-mode and symlink distinctions. `git_grep_changed_lines` reuses those hunks
and uses Snacks' native per-row metadata to select and highlight the chosen
changed line. The resolved custom picker also uses `style = "fancy"`.

To opt into delta for those previews, the relevant setting is
`previewers.diff.style = "terminal"`. Diff-item previews use
`previewers.diff.cmd`, while Git-command previews use Git's pager. Both executable
paths need checking when trying this fork. A global switch would also bypass the
native preview metadata used by the custom changed-lines picker. That behavior
needs a replacement before recommending such a switch. Keeping the native
picker and optimizing delta for lazygit and Git paging fits the current setup.

Both `delta` on PATH and the local baseline report version 0.20.1. All reported
before/after results compare local release builds from this checkout, avoiding
assumptions about how the installed binary was built.

## What the measurements establish

The comparison runner measures fresh processes with fixed width, true color,
and paging disabled. It uses synthetic tiny, Unicode, unpaired, and long-line
diffs, plus 100 delta commits and 50 lazygit commits. Each case must produce
identical bytes before timing starts. Both binaries receive three warmups and
15 measured runs in alternating order.

The isolated run disables Git configuration. The configured run uses the local
Catppuccin Mocha settings, including syntax colors on both removed and added
lines, line numbers, and style mappings. Neither matrix includes the lazygit
wrapper, hyperlinks, Neovim redraw, or terminal parsing. These are renderer
measurements, not complete UI latency measurements.

Median wall times for selected cases:

| Configuration | Workload and mode | Before | After | Time reduction |
| --- | --- | ---: | ---: | ---: |
| isolated | rust-history, unified | 269.9 ms | 250.8 ms | 7.1% |
| configured | rust-history, unified | 286.2 ms | 286.2 ms | -0.0% |
| isolated | go-history, unified | 582.8 ms | 506.3 ms | 13.1% |
| configured | go-history, unified | 646.0 ms | 561.0 ms | 13.1% |
| configured | go-history, side-by-side | 876.1 ms | 779.4 ms | 11.0% |
| configured | long-lines, unified | 261.9 ms | 117.1 ms | 55.3% |
| configured | long-lines, side-by-side | 433.0 ms | 127.8 ms | 70.5% |
| isolated | go-history, no-syntax | 154.1 ms | 83.6 ms | 45.8% |
| configured | go-history, no-syntax | 183.4 ms | 112.4 ms | 38.7% |

The raw [isolated results](data/fork-isolated.json) and
[configured results](data/fork-configured.json) retain all samples and hashes.
The [comparison chart](data/fork-comparison.html) switches between configurations
and rendering modes. The machine is an Intel Core i7-1370P on Linux x86-64.

The long-line case is deliberately hostile: eight removed and eight added lines,
each with 240 repeated tokens. It tests a real algorithmic weakness, but its
improvement is not a prediction for an ordinary source edit. The short cases
include millisecond-scale variation. These changes do not establish a startup
latency improvement.

The historical `go-small` fixture is `git show HEAD` in the lazygit checkout.
That HEAD is a merge, so it is a combined diff covering `AGENTS.md`,
`pkg/gocui/search_test.go`, and `pkg/gocui/view.go`. It is not a plain unified
single-file Go edit. Delta's metadata emitter explicitly excludes combined
content rows in unified display, although it emits the handshake and headers.
The follow-up separately checks an ordinary `git diff HEAD^ HEAD` with added,
removed, and context metadata. No lazygit UI action failure is claimed from
the combined-diff observation alone.

A separate [lazygit-path run](data/fork-lazygit.json) uses colored Go diffs and
`lazygit-edit://` hyperlinks with the Catppuccin configuration. Go history falls
from 630.2 to 557.9 ms in unified mode and from 885.4 to 796.1 ms side-by-side.
All six comparisons preserve exact output. A
[51-sample follow-up](data/fork-tiny-repeat.json) on empty and tiny inputs shows
mixed changes below one millisecond, rather than a repeatable startup gain.

A [controlled streaming probe](data/fork-streaming.json) pauses the producer for
200 ms before sending its final context line. Direct delta emits its first
output in 11.9 to 13.1 ms, while the existing wrapper emits nothing until
218.8 to 220.6 ms. Both paths use the optimized binary. This demonstrates the
wrapper's EOF barrier under delayed input, not a measured 200 ms penalty for
ordinary file navigation.

On the long-line fixture, three [memory probes](data/fork-memory.json) sample
Linux `VmHWM` every 2 ms. The baseline reaches 25,504 to 25,936 KiB and the candidate
13,500 to 13,572 KiB. These are sampled process high-water marks, separate from the
24-fold reduction in table storage. Sampling can miss a peak just before exit.

An initial `perf record` profile of Rust history attributed substantial CPU time
to Oniguruma matching, Syntect parsing, scope matching, and theme selection.
Disabling syntax highlighting reduced the baseline Rust-history time from about
253 to 48 ms, and Go history from about 561 to 146 ms in the initial probes.
That experiment establishes where most remaining throughput work belongs.
It does not justify disabling syntax highlighting by default.

## Why these source changes help

### Store the alignment path compactly

`src/align.rs` previously stored a parent index, operation, and cost for every
cell in its edit-alignment table. Each cell occupied 24 bytes on this machine.
The fill loop also visited columns in a row-major allocation.

Traceback can infer the parent from the chosen operation. The implementation
now stores one operation byte per cell and reuses a single row of costs. It
visits adjacent cells in memory order. The table storage is therefore about
24 times smaller, plus a linear cost buffer. This is a table-storage reduction,
not a claim that total process memory falls by the same factor.

The scoring rules, initial mismatch penalty, candidate order, and traceback
termination remain the same. An exhaustive test compares exact final costs and
operation sequences against the original full-table algorithm for 14,641 pairs
of short token sequences, including empty inputs and equal-cost paths.

The algorithm still performs quadratic work in the token counts of a candidate
pair. Compact storage makes that work cheaper; it does not impose a time bound.

### Compose style ranges directly

`src/paint.rs` previously expanded both annotations into per-character vectors,
collected their zip, built another vector of combined annotations, and coalesced
the result. The implementation now walks overlapping string ranges and copies
each overlap into its output section.

It preserves grouping by the original syntax/diff style pair, including when
different pairs map to the same final terminal style. It also preserves the
final newline section, null-syntax behavior, and mismatch checks. Tests cover
UTF-8, an emoji sequence split across sections, empty sections, and newline
removal. The benchmark checks the emitted ANSI bytes as well as visible text.

## Where I would invest next

1. **Remove the lazygit wrapper's buffering and process chain.** Move the
   required statistics, tab presentation, and link handling into one rendering
   path, using the fork's native Git renderer where possible. Preserve the
   existing output deliberately, especially `lazygit-edit://` links and OSC 1717
   records. The chezmoi default is delta, so replacing it with native rendering
   is a user-facing choice. Measure
   selection-to-first-output and selection-to-complete-preview through lazygit.
   A faster delta cannot compensate for an upstream wrapper waiting for EOF.

2. **Prototype a worker owned by the editor or lazygit session.** Keep syntax
   assets and compiled regexes alive across requests. Use a framed request
   containing the diff, working directory, width, and presentation settings.
   Treat cancellation as part of the interface: moving the selection should
   discard obsolete work immediately. The existing process-global caller and
   environment state must become request-scoped. Do not repeatedly call
   `run_app` in a long-lived process and assume those globals reset.
   Cache invalidation must include configuration, theme assets, and dimensions.
   The measured startup floor is only several milliseconds, so the worker must
   demonstrate a UI benefit before its maintenance cost is accepted.

3. **Offer an explicit interactive work budget.** Bound alignment cells and
   syntax work, render the visible region first, and let a settled selection
   request a complete preview. Preserve text when falling back to simpler
   coloring. The existing defaults limit buffered lines to 32 and syntax work
   to 400 characters per line, but neither bounds all candidate-pair work.
   Lowering `--max-line-length` discards content, so it is a poor general
   performance fix. A budget changes highlighting behavior and needs its own
   contract and tests.

4. **Tokenize each candidate line once.** `infer_edits` currently tokenizes both
   lines for every proposed match and allocates annotations even for rejected
   matches. Borrow cached token slices, then separate distance evaluation from
   annotation if profiles justify it. Keep the exact greedy pairing and
   tie-breaking rules. Threshold shortcuts must respect the display-width
   distance calculation, rather than treating token edit distance as equivalent.

5. **Evaluate syntax highlighting alternatives at a replaceable seam.** A worker
   can cache syntax states and results keyed by content and incoming state.
   Reusing a highlighted line solely by its text is incorrect for multiline
   strings and comments. A Neovim integration can instead keep its native
   renderer, as Snacks currently does. Replacing Oniguruma or using Tree-sitter
   is a larger experiment involving grammar coverage and output changes, not
   a dependency swap with an assumed speedup.

6. **Measure build tuning after algorithmic changes.** Compare thin LTO, fewer
   code-generation units, and profile-guided optimization using both tiny
   previews and source histories. No gain from these settings is claimed here.
   Whole-history throughput alone is insufficient evidence for a preview build.

The fork already disables last-resort full process scans on Linux and caches
Git remote URLs for hyperlinks. Those existing improvements should remain.
Removing process discovery altogether would need an explicit caller context
from integrations so relative paths and Git command handling stay correct.

## Validation and limits

`cargo fmt -- --check` and the full Rust test suite pass: 479 tests pass and
8 are ignored. `cargo clippy --all-targets` succeeds with four existing
`needless_borrow` warnings in `src/utils/process.rs` tests. No new Clippy warning
remains in the changed code.

The deprecated-option and pager-history end-to-end tests pass. The pager test
runs through a PTY with its history paths redirected into a temporary directory.
The full-history raw-output test fails on both the baseline and candidate,
with an identical failure body containing 21 omitted binary-file notices.
That pre-existing behavior is outside these rendering optimizations. The
comparison runner also rejects a deliberately wrong executable before producing
a results file. [Validation details](data/fork-validation.json) record these
outcomes.

The chezmoi follow-up reproduced a separate wrapper error: a delta stub exiting
42 produces wrapper exit status 0 because the final awk command succeeds.
This is a wrapper failure-propagation bug, not a demonstrated lazygit core bug.
It is recorded for later work in
`~/git-repos/lazygit/docs-master/dev/Delta-integration-follow-up.md`; no fix was
made to the wrapper or lazygit implementation.

The initial performance investigation needed no SSH connection or personal
configuration edit. Windows and other operating systems have not been measured.

## Scope-style caching follow-up

The complete-stack investigation resumed after installing lazygit `b077f37c`
and delta `f9f227f` with `just install`. Neovim resolved the same installed
executables as the shell. Its interactive test terminal was 120 columns by
40 rows. The configured renderer remained the chezmoi wrapper with the
Catppuccin theme, tab markers, statistics, and editor hyperlinks.

A fresh CPU profile of the configured Go-history render attributed substantial
time to Syntect's scope matching and theme-style resolution. The replacement
highlighter caches the resolved style by the complete scope stack. It continues
to use Syntect's parser and applies every scope operation, including operations
without visible text. It preserves region boundaries, multiline state, and
theme selector precedence. Identical text alone is never a cache key.

The theme is fixed for the lifetime of a painter. A hunk or syntax reset clears
parser state and its scope stack while retaining the theme cache. The cache
holds at most 1,024 distinct stacks and clears on overflow. Eviction changes
work performed, not output. The cache is local to one renderer process; it does
not reuse configuration or content across invocations.

The existing comparison runner verifies exact bytes before timing. With
`OSC1717=V1`, the configured matrix passed all 24 fixture/mode combinations,
including unified, side-by-side, and syntax-disabled rendering. Seven paired
samples after three warmups measured Go-history unified rendering at 580 ms
before and 414 ms after; side-by-side at 816 ms and 620 ms. Rust-history unified
fell from 349 ms to 306 ms. Tiny cases show millisecond-scale mixed variation,
so no startup improvement is claimed. The complete wrapper comparison also
preserved exact bytes, including its statistics, tab markers, OSC records, and
editor links. Nine paired samples measured Go history at 1,005 ms before and
548 ms after, and the synthetic 12,000-line Go change at 1,202 ms and 935 ms.
These are different workloads and timing boundaries, not interchangeable
measurements. The samples are in
[the configured comparison](data/stack-scope-cache.json) and
[the wrapper comparison](data/stack-scope-cache-wrapper.json).

Differential unit tests compare exact styled regions against Syntect across
12 languages and every bundled theme, including Unicode, comments, embedded
languages, and unterminated input. Additional checks cover syntax resets and
cache eviction. `just check` passes with 482 Rust tests and 8 ignored. The four
previously documented Clippy warnings remain; no new warning is introduced.

Terminal-buffer first-body measurements gave about 53 ms for the large change
on the baseline. Terminal-buffer last-update time was rejected as a completion
metric because unrelated refreshes contribute updates. Process completion and
first visible body are recorded as separate boundaries.

## Native lazygit compatibility mode

`--lazygit-compat` moves the personal wrapper's input and output transformations
into delta. The mode reads stdin, detects existing statistics after stripping
SGR colors, generates missing statistics with `git apply --stat`, replaces tabs
with `├──┤`, and colors the final diffstat graphs. It is a CLI-only adapter,
not a Git-config option. Ordinary delta invocations keep their behavior.

The mode preserves the existing EOF barrier for statistics: it must know whether
the input already contains a summary before generating one. This change does
not claim streaming input or a persistent renderer. It removes temporary files,
the shell pipeline, and its cat/sed/awk helper processes. Git is the only child
needed when statistics are missing. A scoped writer thread supplies Git's stdin
while its output is drained, so large input and output cannot deadlock on pipe
capacity. The final graph writer handles arbitrary write boundaries and retains
row metadata, editor hyperlinks, and the wrapper's final-newline behavior.

Thirty-four exact-byte comparisons cover the configured source fixtures,
side-by-side output, negotiated metadata on and off, binary patches, symlinks,
filenames with spaces and Unicode, invalid UTF-8, CRLF, and missing final
newlines. Rust tests cover statistics, colored summary detection, and every
write chunk size of a metadata/link/graph fixture. Full validation passes with
485 tests and 8 ignored, retaining the four documented baseline Clippy warnings.

Nine paired process measurements through the complete pipeline gave these
median completion times:

| Input | Original fork and wrapper | Scope cache and wrapper | Scope cache and native mode |
| --- | ---: | ---: | ---: |
| 30-line Go change | 24.41 ms | 22.40 ms | 18.44 ms |
| 1,000-line Go change | 118.42 ms | 96.10 ms | 89.94 ms |
| 12,000-line Go change | 1,205.45 ms | 940.42 ms | 934.06 ms |
| 50-commit Go history | 1,002.56 ms | 560.17 ms | 531.40 ms |

Go-history first output fell from 39.73 ms to 23.30 ms. These are pipe timings
with an 80-column preview width and the personal Catppuccin settings. They are
not physical key-to-pixel timings. See
[the complete-stack data](data/stack-native-compat.json) and
[the compatibility evidence](data/stack-native-parity.json).

Install this fork before changing the chezmoi wrapper to pass the new flag.
Stock delta does not implement the adapter. The wrapper can then use `exec`
and preserve the renderer's failure status directly.

The final installed wrapper passes its seven regression checks. Real 120×40
Neovim checks confirm original source tabs survive line copying, no tab-marker
glyphs or ANSI escapes enter copied text, and editing a selected row opens the
original file at its line in the existing editor. Copy uses a test-only file
clipboard. Single-file staging, unstaging, rapid navigation, and return to
Neovim also pass. First visible diff-body text improves from about 53 ms to
32 ms for the large source change, and from 28 ms to 22 ms for the medium one.

The native adapter buffers input in RAM rather than temporary files. Three
2 ms `/proc` sampling runs on the 12,000-line Go fixture observed delta process
high-water RSS of 18,856 to 18,944 KiB before and 20,788 to 21,068 KiB after.
These are sampled process values, not the aggregate old pipeline footprint;
sampling can miss a peak before exit. Memory grows with the input size, and
the adapter currently has no disk-spooling path. The
[installed-stack validation](data/stack-installed-validation.json) retains
these observations and the editor checks.

## Theme choice is not the complete rendering cost

The user challenged the roughly 500 ms result after installation. That result
is the complete rendering of a 1,886,941-byte diff covering 50 Go commits, not
time spent selecting palette colors. It includes source parsing, edit alignment,
style application, metadata and hyperlink formatting, and output.

A fresh installed-build profile points to Syntect/Oniguruma parsing and regex
matching as the main remaining work. Theme-selector matching no longer appears
among the dominant leaf costs after the scope cache. The palette itself is
data; the former expensive operation was repeatedly resolving context-sensitive
selectors for syntax scopes.

Nine paired diagnostic runs gave complete-process medians of 489.33 ms with
Catppuccin, 477.98 ms with `base16-ocean.dark`, and 155.75 ms with syntax
highlighting disabled. The small difference between themes also includes
changes in formatted output. The disabled-syntax case skips parsing as well as
theme application. Neither subtraction measures pure palette-lookup cost.
Calling all of that difference "theme cost" would be incorrect.

The [profiling evidence](data/theme-cost-attribution.json) retains the complete
timings and sampled functions. Additional theme changes are not justified by
this profile. The larger opportunity is avoiding repeated source parsing while
preserving multiline state, cancellation, and configuration invalidation.

## Release optimization after the Neovim UI profile

The 120×40 UI profile in the chezmoi repository found that opening search in a
fresh large diff waited for complete output, even though the first visible body
arrived in about 35 ms. Work is coordinated with lazygit's incremental search
and SGR ingestion changes. Search prompt latency and renderer completion are
separate measurements.

Release builds now use thin LTO and one code-generation unit. Nine alternating
configured comparisons preserved exact output and measured Go-history unified
rendering at 421.77 ms before and 392.63 ms after. Rust history changed from
322.64 ms to 312.87 ms. The subsequent full 24-case matrix also preserved bytes,
including OSC 1717 records and editor hyperlinks. Its five-sample history
medians were 411.80 to 387.51 ms for Go unified, and 322.81 to 308.92 ms for Rust
unified. Side-by-side history was essentially unchanged. Tiny cases varied by
milliseconds in both directions, so this is not a universal startup speedup.

This trades compilation time for installed runtime performance. The initial
optimized experiment rebuilt dependencies and finished in 33 seconds on this
machine. A later install compiled the executable in about 27 seconds. These
are observed warm-cache build times, not a clean-build comparison. Normal
debug and test profiles retain their settings. No output or parser behavior
changes as part of this build setting.

A bounded, full-parser-state-checked cache of stable line transitions was also
tested and rejected. On the real configured Go history it changed the median
from 418.93 ms to 428.63 ms; Rust history was essentially unchanged. Synthetic
repetition is insufficient justification for that cache. It was removed before
committing. Any future parser reuse must earn its cost on actual histories and
retain multiline and injected-language state.

The evidence is in [release optimization](data/stack-release-optimization.json)
and [the configured matrix](data/stack-release-matrix.json). `just check` passes
with 485 tests and 8 ignored, retaining the four documented Clippy warnings.

## Installed fork

On 2026-11-08, `just install` built and installed this checkout using
`cargo install --path . --locked --force`. On this host the executable is
`~/.cargo/bin/delta`. The Homebrew `git-delta` formula was then uninstalled,
with automatic dependency removal disabled. No installed formula depended on it.

Fresh Bash, Fish, and Neovim processes resolve delta to the Cargo installation.
The installed executable matches the release build and emits the negotiated
OSC 1717 handshake and lazygit editor links. The
[installation evidence](data/fork-installation.json) records paths and hashes.
Run `just install` again after updating the fork. Cargo's configured install
root determines the destination; `~/.cargo/bin` is its default on this host.
