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

| Crate | Version | Path | Purpose |
|---|---|---|---|
| [`standarx-dsl`](crates/standarx-dsl) | `1.0.0` | `crates/standarx-dsl` | Lexer + parser + AST + diagnostics. The core. **Frozen semver contract.** |
| [`standarx-dsl-grammar`](crates/standarx-dsl-grammar) | `0.1.x` | `crates/standarx-dsl-grammar` | Emits `*.tmLanguage.json` + `*.language-configuration.json` from a single Rust source. Pre-generated files versioned in `dist/`. |
| [`standarx-dsl-lsp`](crates/standarx-dsl-lsp) | `0.1.x` | `crates/standarx-dsl-lsp` | LSP server (`standarx-lsp` binary) wrapping `parse()`. Publishes syntactic diagnostics. Schema-aware features plug on top. |

Versions are deliberately decoupled — `standarx-dsl` is stable; the
editor-integration crates are still moving.

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
