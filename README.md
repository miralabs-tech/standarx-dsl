# standarx-dsl

Reusable DSL parser for the `standar*` ecosystem (lexer + parser + AST),
plus editor-integration crates so any consumer (`standarbuild`,
`standardoc`, future projects) gets syntax highlighting and live
diagnostics for free in VSCode, JetBrains, neovim, Helix, Emacs, Zed.

The DSL grammar supports nested blocks, arrays, scalar literals, and
interpolated strings — designed to host config files like
`standarbuild`'s `.sxb` (task definitions) and `standardoc`'s `.sxd`
(workspace config).

## Workspace layout

| Path | Version | Purpose |
|---|---|---|
| [`crates/standarx-dsl`](crates/standarx-dsl) | `1.0.0` | Lexer + parser + AST + diagnostics. The core. **Frozen semver contract.** |
| [`crates/standarx-dsl-grammar`](crates/standarx-dsl-grammar) | `0.1.x` | Emits `*.tmLanguage.json` + `*.language-configuration.json` from a single Rust source. Pre-generated files versioned in `dist/`. |
| [`crates/standarx-dsl-lsp`](crates/standarx-dsl-lsp) | `0.1.x` | LSP server (`standarx-lsp` binary) wrapping `parse()`. Publishes syntactic diagnostics. Schema-aware features (completion / hover / goto-def) plug on top via the `Schema` trait. |
| [`tree-sitter-standarx`](tree-sitter-standarx) | `0.1.x` | Tree-sitter grammar covering native highlighting in neovim / Helix / Zed / GitHub Linguist. Companion to the Rust parser; corpus tests pin the parse-tree shape. |
| [`fuzz`](fuzz) | `0.0.0` | Cargo-fuzz harnesses on `parse()` and `parse_with_recovery()`. Nightly + libFuzzer; excluded from the stable workspace. |

Versions are deliberately decoupled — `standarx-dsl` is stable; the
editor-integration crates are still moving.

## Consuming this in your project

### From crates.io (once published)

```toml
[dependencies]
standarx-dsl = "1"
# Optional editor backends:
standarx-dsl-lsp = "0.1"
standarx-dsl-grammar = "0.1"
```

Publish order is `standarx-dsl` → `standarx-dsl-grammar` →
`standarx-dsl-lsp` (the LSP depends on the parser; the grammar is
independent).

### From this GitHub repo

Pick any tag, branch, or revision and Cargo resolves the workspace
member by name. No special configuration required.

```toml
# Pinned to a release tag (recommended for downstream stability).
[dependencies]
standarx-dsl = { git = "https://github.com/miralabs-tech/standarx-dsl", tag = "v1.0.0" }

# Or follow main:
standarx-dsl = { git = "https://github.com/miralabs-tech/standarx-dsl", branch = "main" }
```

Same shape for `standarx-dsl-lsp` and `standarx-dsl-grammar` —
all three live in the same workspace, so a single git source feeds
every dependency line.

### Disable serde

`standarx-dsl` derives `serde::Serialize` on its AST and `Diag`
types under the default `serde` feature. Disable it for a leaner
build:

```toml
standarx-dsl = { version = "1", default-features = false }
```

The grammar and LSP crates do not toggle features today.

## Quick parser usage

```rust
let src = r#"
    version "0.1.0"
    project "ext" {
        path "./ext"
        tasks ["build" "test"]
    }
"#;
let file = standarx_dsl::parse(src).expect("parse ok");
assert_eq!(file.stmts.len(), 2);
```

The parser is consumer-agnostic: it produces a generic `ast::File`
tree. Downstream crates lower it into their own typed schemas (no
schema opinions baked in here).

### Multi-error recovery

For editor / LSP flows where fail-fast hurts UX, use:

```rust
let (file, diags) = standarx_dsl::parse_with_recovery(src);
// file is always a (possibly partial) tree; diags lists every error.
```

## Plugging the DSL into an editor

The grammar and LSP are **file-extension agnostic** — each editor
extension declares which extension (`.sxb`, `.sxd`, your own) maps
to the `source.standarx` scope and the `standarx-lsp` binary. See
the per-crate READMEs for VSCode / neovim recipes:

- [`crates/standarx-dsl-grammar`](crates/standarx-dsl-grammar) —
  TextMate grammar + language-configuration (VSCode, JetBrains,
  Sublime, Helix-fallback).
- [`crates/standarx-dsl-lsp`](crates/standarx-dsl-lsp) — LSP
  server with extensible `Schema` trait for completion / hover /
  goto-definition. Wiring snippets for VSCode, neovim, Helix, Zed.
- [`tree-sitter-standarx`](tree-sitter-standarx) — Tree-sitter
  grammar for native highlighting in neovim, Helix, Zed, and
  GitHub Linguist.

## Versioning policy

- **`standarx-dsl 1.x`** — public API is **frozen under semver**.
  Breaking changes require a 2.0. New variants on
  `#[non_exhaustive]` enums and new fields on `#[non_exhaustive]`
  structs land in minor versions without breaking downstream.
- **`standarx-dsl-lsp 0.1.x` / `standarx-dsl-grammar 0.1.x`** —
  pre-1.0; minor versions may change the API surface. Pin exact
  versions until 1.0 if your downstream needs stability.
- **`tree-sitter-standarx 0.1.x`** — moves alongside the Rust
  parser; corpus tests in `tree-sitter-standarx/test/corpus/`
  pin the parse-tree shape and act as the regression net.


## Origin

Extracted from
[standarbuild](https://github.com/miralabs-tech/standarbuild)'s
internal `dsl/` module so multiple `standar*` projects can share a
single parser, grammar, and LSP backend.

## License

MIT
