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

**Future-proofing policy.** All public enums in `ast` (`Stmt`, `Expr`,
`InterpExpr`, `StringPart`, `TriviaKind`) and `diag` (`Severity`,
`DiagKind`), plus the `Diag` struct and the AST structs that carry
optional metadata (`File`, `StmtNode`, `Trivia`, `Assign`, `Block`,
`Ref`, `MapEntry`, `StringLit`), are marked `#[non_exhaustive]`.
New variants and new fields can land in 1.x minors without breaking
downstreams that have `_ =>` arms / non-literal construction. Two
escape hatches stay open by design: `Ident(pub String)` (consumers
build idents in tests) and `Spanned<T>` (generic wrapper, shape can't
grow without changing the generic itself).

**Locked-in design choices for v1.0.** Three behaviors are now part
of the contract; changing any of them in a 1.x is forbidden, and
each can have a *new* API added next to it without breaking:

- `parse()` returns `Result<File, Diag>` — a single diagnostic on
  failure, fail-fast (no error recovery). If multi-error reporting
  is needed later, it ships as a new function (e.g.
  `parse_with_recovery() -> (File, Vec<Diag>)`) rather than
  changing the existing signature.
- `Spanned<T>: Serialize` delegates to `T` — the span is dropped
  during JSON / msgpack / etc. encoding. Tooling that needs spans
  works with the `Spanned` struct directly or implements its own
  serializer. Changing this would silently churn every downstream
  AST dump.
- `serde::Serialize` derives on AST/Diag types ship under the
  default `serde` feature (on by default). Consumers can opt out
  with `default-features = false` for a leaner build. Adding a
  `Deserialize` derive later goes behind the same feature.

### Added

- `impl std::fmt::Display for Diag` — `format!("{diag}")` now reads
  the same as `format!("{}", diag.kind)`.
- `impl std::error::Error for Diag` — sourced from `kind`. `Diag`
  can now be used with `?`, `Box<dyn Error>`, `anyhow`, etc. without
  intermediate `.kind.to_string()` dances.
- `#[must_use]` on `parse()` with a custom message explaining that
  ignoring the result discards parse errors.
- `#[must_use]` on `Diag::is_error()`.
- `#[non_exhaustive]` on 7 public enums and 9 public structs (see
  "Future-proofing policy" above) so 1.x minors can grow variants
  and fields additively.
- `parse_with_recovery(src) -> (File, Vec<Diag>)` — recovery-aware
  parser. Always returns a (possibly partial) `File` plus every
  diagnostic that fired. Top-level statement errors trigger a sync
  to the next likely statement start; block-internal errors fail
  the enclosing top-level statement. Lexer errors remain fatal
  (single diagnostic, empty file). Useful for editor / LSP workflows
  that want to show every problem in one pass instead of fail-fast.
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
