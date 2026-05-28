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

| Crate | Path | Purpose |
|---|---|---|
| [`standarx-dsl`](crates/standarx-dsl) | `crates/standarx-dsl` | Lexer + parser + AST + diagnostics. The core. |
| [`standarx-dsl-grammar`](crates/standarx-dsl-grammar) | `crates/standarx-dsl-grammar` | Emits `*.tmLanguage.json` + `*.language-configuration.json` from a single Rust source. Pre-generated files versioned in `dist/`. |
| [`standarx-dsl-lsp`](crates/standarx-dsl-lsp) | `crates/standarx-dsl-lsp` | LSP server (`standarx-lsp` binary) wrapping `parse()`. Publishes syntactic diagnostics. Schema-aware features plug on top. |

## Status

Pre-1.0. The grammar is stable; public Rust APIs may evolve. Pin to
exact `0.1.x` until v1.0.

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

## Plugging the DSL into an editor

The grammar and LSP are **file-extension agnostic** — each editor
extension declares which extension (`.sxb`, `.sxd`, your own) maps
to the `source.standarx` scope and the `standarx-lsp` binary. See
the per-crate READMEs for VSCode / neovim recipes.

## Origin

Extracted from
[standarbuild](https://github.com/miralabs-tech/standarbuild)'s
internal `dsl/` module so multiple `standar*` projects can share a
single parser, grammar, and LSP backend.

## License

MIT
