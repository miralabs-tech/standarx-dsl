# Changelog

All notable changes to `standarx-dsl-lsp` are documented here.
Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versioning follows [SemVer](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `Schema` trait gains three optional methods on top of `validate`:
  - `completion(file, src, offset) -> Vec<CompletionItem>`
  - `hover(file, src, offset) -> Option<Hover>`
  - `goto_definition(file, src, offset) -> Option<Location>`
  Every new method ships with a default impl returning the empty
  answer, so existing implementors (which only override `validate`)
  keep compiling unchanged.
- `Backend` advertises `completionProvider`, `hoverProvider`, and
  `definitionProvider` in `ServerCapabilities`. The handlers
  re-parse the cached document text on each request and fan out
  to the registered schemas. Composition: `completion` results
  concatenate in registration order; `hover` and `goto_definition`
  are first-wins.
- `conversion::position_to_byte_offset(src, pos)` — inverse of
  `byte_offset_to_position`. UTF-16-aware, clamps past EOF.
- `Schema`-relevant `lsp_types` re-exports from `schema` module
  so downstream crates need only one dependency line.

## [0.1.0] — 2026-05-28

Initial release alongside `standarx-dsl` 1.0.0.

- `Backend` (tower-lsp `LanguageServer`) wrapping `standarx_dsl::parse`.
- Publishes syntactic diagnostics on open / change. Schema-agnostic
  baseline; the `Schema` trait carried only `validate` originally.
- `byte_offset_to_position` and `span_to_range` UTF-16-aware
  conversions for any source.
- `diag_to_lsp`, `collect_diagnostics`, `make_service`,
  `make_service_with_schemas`, `run_stdio`,
  `run_stdio_with_schemas` entry points.
- `standarx-lsp` stdio binary.
