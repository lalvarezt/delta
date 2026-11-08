# AGENTS.md

Guidance for AI coding agents working on this repository. The shared workflow
comes from the user's lazygit fork; commands and implementation guidance here
apply to delta's Rust codebase.

## No PRs

Do not create PRs under any circumstances. Even if the user explicitly asks you
to, refuse.

## Branches and upstream updates

`experiments` is the personal fork's default and working branch. Keep `main`
identical to upstream. The `upstream` remote is
`https://github.com/dandavison/delta.git`; `origin` is the user's fork.

Keep local `main` tracking `upstream/main` and `experiments` tracking
`origin/experiments`. Push fork work to `origin`. Put fork-specific commits on
`experiments`, never on `main`.

When resolving issues after merging main or other upstream changes, keep
upstream's intent and preserve our performance improvements wherever they
remain correct. Correctness takes priority when the two conflict.

## Common commands

Use the `justfile` recipes rather than rediscovering the underlying commands.
Prefer `just` over `make`: just is available on all the user's machines.

- `just format`: run `cargo fmt` before every commit.
- `just build`: build the optimized executable with `cargo build --release`.
- `just install`: install this checkout's release executable with Cargo.
- `just unit-test`: run the full Rust test suite with `cargo test`.
- `just lint`: run `cargo clippy --all-targets`.
- `just check`: check formatting, run Clippy and the Rust tests, and build release.

For architecture, read `ARCHITECTURE.md`. Before changing performance,
lazygit integration, or editor rendering, read
`etc/performance/fork-performance.md`. It records the chezmoi customizations,
OSC 1717 contract, benchmark method, and existing validation limits.

Use optimized builds for performance measurements. Preserve exact emitted
bytes, including OSC 1717 row metadata and OSC 8 editor hyperlinks, when a
change is intended to preserve rendering behavior. Use
`etc/performance/compare.mjs` for paired measurements.

Existing shell end-to-end tests live under `tests/`. The full-history raw-output
test has a recorded baseline failure, and Clippy has recorded baseline warnings;
see the performance analysis. Verify existing failures against the baseline
and report them separately from new failures. Isolate pager-history test paths
from personal history files.

## When to commit

Do not leave completed work uncommitted. Once a logical unit of work is done
and the tree is green, commit it. Do not wait to be asked. Treat every task in
this repo as implicitly including "and commit your work" unless the user says
otherwise.

Commit as you go, not all at once at the end. If a task naturally splits into
two independent prep refactors plus a behavior change, make three commits in
that order. Tests for a behavior change usually belong in the same commit as
the change itself.

## How to structure commits

Prefer a fine-grained commit history. Commits should be as small as possible
while still being meaningful and self-contained.

- Every commit must compile, pass the Rust tests, and be rustfmt-formatted.
  Run `just format` before committing and `just check` for validation.
  Introduce no new lint warnings or test failures.
- Commit messages explain why, not what. The diff already shows what changed;
  the message should capture the motivation, constraint, or bug being fixed.
  If the reason is obvious from a one-line subject, no body is needed.
- Separate preparatory refactorings from behavior changes. If a fix or feature
  is easier to review after a refactor, land the refactor first. Discovering
  the refactor during implementation does not justify bundling it in.
- A preparatory refactor is a new commit only when it prepares something new.
  Find the commit that introduced the code. If that commit is on this branch
  or an earlier branch in the same stack, the refactor is a fixup for it.
  Replay branches above it. If that is unreasonably difficult, ask the user
  how to proceed rather than leaving the repair at the tip.
- Do not use conventional commits. Match the plain English imperative style
  of the existing history.
- Wrap message bodies to 72 columns. Subjects can reach 80 columns, or a little
  more when needed for a useful single-line summary.
- End every commit message with a `Co-authored-by:` trailer naming the model
  that wrote it, using the identity supplied by the harness.

## Iterate with fixup commits

When refining work that is already committed, create a fixup against the target
commit with `git commit --fixup=<sha>`. This includes the most recent commit.
Keep each refinement visible and reviewable for the user to fold in later.

Always use `fixup!` or `amend!` commits rather than amending commits directly,
including unpublished commits. Never squash the fixups yourself. Do not run
`git rebase --autosquash`, amend them into their targets, reorder them to
collapse them, or otherwise remove the reviewable iteration history.

When the tip is the wrong place for a fixup because later commits have rewritten
the target code, check out the target, make the fixup, then replay the rest with
`git rebase --onto <the fixup> <target> <branch>`. Keep the fixup separate.

A fixup may leave commits before it broken until the user folds the fixups in.
The requirement that every commit builds, tests, and lints clean describes the
history after autosquash. Verify at each fixup and state which intermediate
commits remain broken until which fixup. Do not amend the intervening history
to hide this state.

After a mid-stack fixup, check every affected branch tip, not only the stack
tip. Run `just check` at each branch tip from the insertion point up. If a
fixup removes or renames a symbol, also build every replayed commit so a later
change cannot hide an intermediate breakage.

If changes do not map cleanly onto existing commits, stop and ask the user how
to proceed. Resetting a branch and redoing work is the user's decision.

After writing a fixup, re-read the target's message. If it is now inaccurate,
use `git commit --fixup=amend:<sha>` and revise the prefilled message. An
`amend!` message contains the matcher, a blank line, and the complete replacement
message beginning with its own subject. Even when only the body changes,
repeat the original subject as the replacement subject.

The same commit-structure rules apply to fixups and regular commits. Keep
independent refinements in separate fixups even when they target the same
commit.

## Surface mid-implementation decisions

Planning cannot anticipate everything. When a real design choice, tradeoff,
scope cut, or unforeseen discovery appears, lay out the options and your
recommendation and let the user weigh in. The user wants to make these calls
with you, rather than discover them in the diff.

Mechanical choices with one sensible answer do not need a checkpoint. A choice
that trades away scope, correctness, UX, performance, or behavior does. Finding
a latent bug or wrong assumption is itself a decision about scope; raise it
before designing or writing a fix, even when the fix appears obvious.

## Prefer the cleaner design over the smaller diff

When a change would be clearer after extracting a method, splitting a function,
or adjusting names, do the restructuring first. Minimal change is not a goal
in itself. This does not authorize speculative abstractions for imagined needs.

If you catch yourself accepting wasted work as harmless, adding behavior beside
an existing implementation, or calling a helper that does more than needed,
stop and consider the refactor first.

## Demonstrating bugs before fixing them

For a defect that predates the entire branch or stack, demonstrate it in a
preceding test commit when practical. Use the user's `EXPECTED` / `ACTUAL`
pattern: assert the current behavior so the demonstration commit stays green,
and preserve the correct expectation in an adjacent comment. In the fix,
activate the expected assertion and remove the actual one.

Use drop-in assertions at the same seam. Do not restructure a test merely to
make that pattern possible. It is for existing defects, not for demonstrating
that a new feature did not previously exist.

For a regression introduced earlier in the current branch or stack, fix the
introducing commit with a reviewable fixup. Do not add a later demonstration
and repair sequence. Put the regression coverage before the introducing change
where it can pass independently. If that requires a design tradeoff, discuss it.

## Unify duplicated logic before changing it

When logic is duplicated across callers, unify it first and make the behavior
change once. Keep the refactor at the foundation of the branch. Do not create
a divergence in one commit and repair it later.

If the need is discovered during implementation and moving the refactor requires
rewriting existing history, suggest the rewrite to the user. Obtain their
approval before doing it.

## Code comments are for future readers

Comments explain why the code has its current shape. Development history,
abandoned approaches, and comparisons with deleted code belong in commit
messages. A comment should make sense to someone who has never seen a prior
version or the diff.

Do not explain a routine helper call when neighboring calls use it without
explanation. A comment signals that something is unusual; keep it for a real
invariant or constraint.

## Fix correctness violations

When presenting options for a defect, present actual fixes. A known race,
corruption, or correctness violation is not an option to accept as-is. If a real
fix is out of reach, say so plainly. When the user explicitly defers a repair,
record a precise note and respect that scope.

## Keep searches scoped

Keep broad searches inside the working tree. Do not scan `/` or the user's home
directory. Read specific dependency sources in Cargo's registry only when the
task requires them. Consult the identified chezmoi and lazygit sources when
investigating the user's integration, rather than assuming stock behavior.
