# Changelog

All notable changes to `standarx-dsl` are documented here.
Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versioning follows [SemVer](https://semver.org/spec/v2.0.0.html).

## [1.0.0] — 2026-05-28

First stable release. The public API surface — `parse()`, `ast::*`,
`diag::*` — is frozen under semver from this point on. The `lexer`
and `parser` modules remain `pub` for advanced consumers (formatters,
linters, alternative drivers) but are `#[doc(hidden)]` and not part
of the semver contract.

### Added

- `impl std::fmt::Display for Diag` — `format!("{diag}")` now reads
  the same as `format!("{}", diag.kind)`.
- `impl std::error::Error for Diag` — sourced from `kind`. `Diag`
  can now be used with `?`, `Box<dyn Error>`, `anyhow`, etc. without
  intermediate `.kind.to_string()` dances.
- `#[must_use]` on `parse()` with a custom message explaining that
  ignoring the result discards parse errors.
- `#[must_use]` on `Diag::is_error()`.
- `StringLit::try_into_bare_text(span, context)` — decodes a string
  literal as a single-line, non-interpolated bare-name payload.
  Used at three sites (lexer ref-segment, parser ref-segment, parser
  block-label) that previously duplicated the same validation rules.
- Doc-comment on `ast::InterpExpr` explaining the deliberate
  asymmetry with `Expr` (interpolation is scalar-only by design).
- Property tests on `parse()` via `proptest` — ~6k random inputs
  per CI run cover never-panic and determinism.
- Branch-by-branch negative tests in `tests/negative.rs` pinning the
  message text and byte span of every reachable parser / lexer
  diagnostic. Wording is now part of the public contract.
- SAFETY block on the unsafe `buf.as_mut_vec()` write in
  `lexer::push_byte`, with multi-byte (UTF-8 2/3/4-byte) round-trip
  tests across plain, inline-template, and multi-line-template
  string contexts.
- `README.md` with a focused "Diagnostics" section showcasing the
  pedagogical error messages and pointing at the pin file.
- `readme=` + workspace-shared `homepage=` in Cargo.toml metadata.

### Changed

- The `lexer` and `parser` modules are now `#[doc(hidden)]`. They
  stay `pub` but are excluded from the semver public contract — the
  supported surface is `parse()` + `ast::*` + `diag::*`. Anything
  else may change between minor versions.
- Diagnostic emission for ref-segment and block-label validation now
  flows through `StringLit::try_into_bare_text` — single source of
  truth, single message format `"{context} cannot be a multi-line string"`
  / `"{context} cannot contain interpolation"`. No user-facing wording
  changes vs 0.1.x.

### Fixed

- Eliminated three independent copies of the
  "single-line, no-interp" string-literal validation that risked
  drifting apart. Failure modes for ref segments and block labels
  are now guaranteed identical (modulo the context label).

## [0.1.0] — 2026-05-27

Initial extraction from
[`standarbuild`](https://github.com/miralabs-tech/standarbuild)'s
internal `dsl/` module so multiple `standar*` projects can share a
single parser. Unpublished to crates.io; lived as an intermediate
working version until the audit follow-up landed.
