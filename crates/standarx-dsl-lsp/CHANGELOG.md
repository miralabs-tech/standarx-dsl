# Changelog

All notable changes to `standarx-dsl-lsp` are documented here.
Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versioning follows [SemVer](https://semver.org/spec/v2.0.0.html).

## [0.1.0] — 2026-05-28

Initial release alongside `standarx-dsl` 1.0.0.

### Core

- `Backend` implementing `tower_lsp::LanguageServer` — wraps
  `standarx_dsl::parse` with FULL text sync, publishes syntactic
  diagnostics on `did_open` / `did_change`.
- `byte_offset_to_position` / `position_to_byte_offset` —
  UTF-16-aware conversions (handles surrogate pairs for
  supplementary-plane characters). Clamping past EOF is defined.
- `span_to_range` — maps a `standarx_dsl::Span` to an
  `lsp_types::Range`.
- `diag_to_lsp(src, &Diag)` — promotes a `Diag` (severity, span,
  message) to an `lsp_types::Diagnostic`. Severity falls back to
  `INFORMATION` for future `#[non_exhaustive]` variants.
- `collect_diagnostics(src, &schemas)` — pure helper:
  parse + schema fan-out + lsp_types conversion. Exposed for
  embedders driving validation outside tower-lsp.

### `Schema` trait

- `validate(file, src) -> Vec<Diag>` — required, semantic
  diagnostics on top of the syntactic ones.
- `completion(file, src, offset) -> Vec<CompletionItem>` — default
  empty. Concatenates across schemas.
- `hover(file, src, offset) -> Option<Hover>` — default `None`.
  First-wins composition.
- `goto_definition(file, src, offset) -> Option<Location>` —
  default `None`. First-wins composition.

The relevant `lsp_types` re-exports (`CompletionItem`, `Hover`,
`Location`, …) live on the `schema` module so downstream crates
add one dep, not two.

### Entry points

- `Backend::new(client)` — schema-less backend.
- `Backend::new_with(client, schemas)` — backend with semantic
  schemas plugged in.
- `make_service()` / `make_service_with_schemas(schemas)` —
  return an `LspService` for embedding inside a larger server.
- `run_stdio()` / `run_stdio_with_schemas(schemas)` — convenience
  one-liner for stdio binaries.
- Binary `standarx-lsp` (stdio).

### Capabilities

`ServerCapabilities` advertises `textDocumentSync = FULL`,
`completionProvider` (trigger chars `.` and `$`), `hoverProvider`,
and `definitionProvider` unconditionally — when no schema
overrides a method, the editor sees empty answers instead of
"feature unsupported".
