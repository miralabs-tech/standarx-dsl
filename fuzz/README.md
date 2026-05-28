# standarx-dsl-fuzz

libFuzzer / cargo-fuzz harnesses for the `standarx-dsl` parser.

Runs on Linux / macOS with nightly Rust + libFuzzer (LLVM
sanitizers). Windows MSVC is not supported by cargo-fuzz; use WSL
or a Linux runner.

## Setup

```bash
cargo install cargo-fuzz
```

## Run

```bash
# from the workspace root:
cargo +nightly fuzz run parse
# or with a runtime budget (seconds):
cargo +nightly fuzz run parse -- -max_total_time=300
```

The first run picks up the seed corpus from `corpus/parse/`.

## Targets

| Target | Contract under test |
|---|---|
| `parse` | `standarx_dsl::parse` never panics on any `&str`. |
| `parse_with_recovery` *(post phase 2)* | `standarx_dsl::parse_with_recovery` always returns a `File`, never panics, never produces an inconsistent diagnostic list. |

## CI

A nightly Linux job in `.github/workflows/ci.yml` runs
`cargo fuzz build` to catch target-compile regressions. Full
fuzz sessions are runtime-budget-heavy and run on demand only.
