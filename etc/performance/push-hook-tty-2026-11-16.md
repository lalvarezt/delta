# Push checks with terminal stdout

The user's remaining failure is a terminal-dependent test assumption.
The terminal-color test expects captured stdout, but `should_detect_color_mode`
checked the actual test runner's stdout. In a terminal, it chose the terminal
query path. That query is already stubbed to return `None` in unit tests, so
the light GitHub theme won over the fixture's dark environment value.

The earlier OSC 1717 investigation found a separate environment-dependent
failure. Redirected push checks did not cover this terminal-dependent case.

## Reproduction and fix

This exact command passes with captured output and fails with terminal stdout:

```sh
cargo test --bin delta options::theme::tests::test_terminal_color_mode_from_env_var_when_stdout_is_not_the_terminal -- --exact
```

The PTY failure matches the user's assertion: actual `Light`, expected `Dark`.
The existing test provides regression coverage; no assertion is removed.

Automatic detection now treats unit-test stdout as captured output, consistent
with the stubbed terminal query. The compile-time condition is false in normal
executables, preserving production terminal detection and environment fallback.
The hook remains unchanged.

## Validation

The exact test fails with exit 101 before the change and passes with exit 0
afterward in a PTY. `just format` and a full `just check` with terminal stdout
pass: 560 tests, eight ignored, no Clippy warnings, and a release build.
Legacy snapshot format messages remain warnings in the normal test run.

The [evidence](data/push-hook-tty-2026-11-16.json) retains the PTY transcripts,
source identities, and validation commands. This is a fixup for the upstream
integration that introduced the terminal-color test. Earlier metadata fixes,
accepted integration costs, and deferred advisories remain unchanged.
