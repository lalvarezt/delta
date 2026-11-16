# Upstream integration, 2026-11-16

This merge retains upstream's published TUI and the fork's rendering
optimizations. The user directed use of upstream's dependencies after
rejecting a vendored compatibility patch. That restores two known `git2`
soundness advisories. It does not qualify as a clean soundness result.

## Revisions and source review

| Role | Revision |
| --- | --- |
| Task start and pre-merge fork | `09fb84e0038c6dfdaa055e9bfb95324cf0a3a784` |
| Upstream | `54578752f462b75bccc941c361060e9e75f5b783` |
| Merge base | `876a1624fed217d32c35007cdfbe1bf7a7ecb6c8` |
| Validated source tree before evidence files | `1f4fbe06ecea20159680eac808b7bf2ae3c912d7` |

The working tree was clean at task start. Both comparisons therefore use the
same baseline. Review covered all nine incoming commits and the complete fork
diff, including clean merges, new callers, themes, documentation, demo tools,
and the dependency graph. Seven files overlap the fork changes.

Upstream adds the default-enabled `--tui` mode, feature-precedence fixes,
line-number field styling, complete color-only file-header identities, and
hunk identities when no visible header row exists. The TUI invokes this delta
executable for each render. Its feature callback reloads configuration during
the session. It does not add a persistent renderer or share the fork's
highlighter cache between processes.

## Fork customizations

| Customization | Integration result |
| --- | --- |
| Compact alignment table and tie-breaking | `src/align.rs` is byte-identical to task start; exhaustive reference comparisons remain in the Rust suite. |
| Direct style-range composition | Retained; the only incoming `src/paint.rs` change passes the panel side to line-number styling. |
| Complete-scope theme cache | `src/highlight.rs` is byte-identical; cache ownership and invalidation stay within each highlighter. |
| Native lazygit presentation and status propagation | `src/lazygit.rs` is byte-identical; CLI and stdin dispatch remain alongside the new TUI dispatch. |
| Release optimization | Thin LTO and one codegen unit remain enabled. |
| Dependency updates | All prior locked packages remain except `git2 0.21.0`. The TUI requires `0.20.4`; its API also replaces the fork's fallible string-accessor adaptations. |
| Local checks, disabled hosted checks, branch workflow | Retained. `main` tracks `upstream/main`; `experiments` tracks `origin/experiments`. |
| Benchmark feature activation and prior evidence | Retained unchanged. No historical timing samples establish this merge's performance. |

The lockfile grows from 257 to 348 packages. It adds 92 package identities and
removes only `git2 0.21.0`. `libgit2-sys 0.18.8+1.9.7` remains unchanged.
No vendored source or dependency patch remains.

## Correctness

`just format` and `just check` pass. There are 556 passing unit tests, two
passing TUI executable tests, and eight ignored tests. Clippy reports no
warnings. Upstream removes the four baseline needless-borrow warnings.
The merge fixes an upstream Rust 2018 assertion-format warning and gates
test-only TUI imports and the shared config opener for builds without the TUI.
`cargo test --no-default-features` passes 546 tests with eight ignored and no
warnings.

The 66 complete-caller output comparisons cover isolated settings, the actual
Catppuccin settings, native lazygit input, and hunk layouts. Thirty cases
preserve exact bytes. The other 36 preserve all visible text and differ only
in SGR color sequences. Giving minus, plus, and context number fields equal
styles makes all 36 outputs byte-identical, isolating upstream's intended
number-field correction. The configuration snapshot matches the current
effective delta keys.

All 24 retained caller contracts and 13 added contracts pass. They cover
mixed hunk layouts, widths, line-number toggles, deleted-file syntax,
color-only headers, EOF release, hunk identities, grep input, and TUI CLI
dispatch. The actual lazygit consumer preserves all expected identities in
three small outputs and identical identity counts in both large outputs.

The deprecated-option and isolated pager-history shell tests pass on both
binaries. The full-history raw-output test retains its baseline failure with
21 omitted binary-file notices. Its failure body is identical after removing
the timestamp headers.

`git2 0.20.4` restores `RUSTSEC-2026-0183` and `RUSTSEC-2026-0184`.
Both concern undefined behavior and require `git2 >=0.21.0` for repair.
Cargo audit reports zero vulnerabilities and the same four unmaintained
package warnings as the baseline, alongside those two soundness advisories.
The user chose the published upstream dependency after these advisories and
the compile-time API conflict were disclosed. Dependency soundness repair
remains open. The earlier `git2 0.21` improvement is not preserved.

## Performance

Fresh paired results and final caller probes are recorded in the accompanying
[evidence](data/upstream-integration-2026-11-16.json).

The baseline executable SHA-256 is
`3eece00bfeadcd76ebf6fe9c0ded2b6f9c5beece1c08bc96484552c1c2e88417`.
The candidate SHA-256 is
`581c1ad719c6eae28435fc2a4af65e789a185e500885482ad48a72d9f8bbfd85`.
Both use Rust 1.99.1 and the fork's release profile. Executable size increases
from 7,033,360 to 10,814,960 bytes with the default-enabled TUI.

Each complete-renderer case uses three warmups and 15 alternating paired
samples on CPU 2. Input, configuration, theme assets, width, true color, and
OSC 1717 negotiation match between binaries. Timed boundaries include process
startup, input transfer, and complete rendering to a discarded output stream.
Builds and tests remain idle during timing. CPU frequency is not locked.

The first combined timing call hit the MCP response limit. Its isolated
suite completed and retains all raw samples. The interrupted configured log
is retained as rejected evidence. Subsequent runs split the suites and save
raw samples after each case.

No primary case is more than 3.8% slower in these measurements. The complete
wrapper cases are within 2.1%. These ranges do not prove equality below the
measurement noise. The specialized color-only metadata cost below is
reproducible and accepted by the user.

| Boundary | Before | After | Interpretation |
| --- | --- | --- | --- |
| Color-only, OSC 1717, 2,000 mode-only files, 51 pairs | 12.83 ms | 14.20 ms | Adds 158,670 metadata bytes and 1.37 ms, or 10.7%. The user accepted the cost to retain correct file identities. |
| Fresh lazygit view and identity extraction, 1,000-line input | 36.57 ms | 35.12 ms | Both produce 3,804 identities. |
| Fresh lazygit view and identity extraction, 12,000-line input | 417.04 ms | 386.21 ms | Both produce 47,804 identities. |
| Partial hunk, first body after a 200 ms producer delay | 204.22 ms | 204.33 ms | Both use `--side-by-side=hunk`; lookahead remains. |
| Sampled long-line high-water memory | 18,452 KiB | 19,480 KiB | Seven samples per binary; about 1 MiB more process memory. |

The other two sampled memory cases rise by 904 and 936 KiB. The compact
alignment table and style cache remain intact. Direct output, native adapter
buffering, cancellation, and closed-output behavior retain their contracts.
Consumer timings include fresh view allocation, content ingestion, and
identity extraction with one Go worker. They exclude renderer execution and
terminal redraw, so they are not added to renderer timings.

The lazygit consumer uses a frozen source snapshot at
`c468e9da0bb363c67eb98c1cdf00457024265269`, plus its existing tracked working
changes. The evidence pins that patch. A Go overlay adds the probe only to
the snapshot. This task does not alter the live lazygit repository.

An inherited partial-hunk probe initially gave the two binaries different
modes. Those samples are retained as rejected evidence. The matched recheck
above replaces them. Two exploratory assertions also needed correction for
the initial OSC handshake and the known number-color change. Their failed
logs remain in the archive; they are not renderer failures or timing evidence.

## Coverage and limits

The evidence retains raw samples, controls, commands, scripts, fixture and
executable identities, source changes, and validation logs. A local archive
under `.git/performance` also retains frozen inputs and both executables.
Rendered output duplicates are omitted from that archive because the inputs,
arguments, scripts, and output hashes reproduce them.

Physical terminal redraw, editor actions, Windows, and cross-architecture
builds are outside this validation. TUI CLI and configuration contracts run,
but live TUI staging, discard, refresh, and cancellation are not validated.
The sampled Linux memory probes can miss peaks immediately before exit.
No claim extends to those gaps.

Upstream's optional `make video` recipe points at absent `etc/demo/demo`;
the new files live under `etc/tui-demo`. Demo recording is not exercised or
repaired by this integration. Personal configuration and installed tools are
not changed. This task does not push the branch.
