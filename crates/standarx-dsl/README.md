# standarx-dsl

[![Crates.io](https://img.shields.io/crates/v/standarx-dsl.svg)](https://crates.io/crates/standarx-dsl)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Reusable DSL parser for the `standar*` ecosystem (lexer + parser + AST).

The DSL grammar supports nested blocks, arrays, scalar literals, and
interpolated strings — designed to host config files like
`standarbuild`'s `.sxb` (task definitions) and `standardoc`'s `.sxd`
(workspace config). The parser is **consumer-agnostic**: it produces a
generic [`ast::File`](src/ast.rs) tree; downstream crates lower it into
their own typed schemas.

## Usage

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

## Diagnostics

`parse()` returns `Result<File, Diag>`. Every `Diag` carries a byte
[`Span`](src/diag.rs) and a structured `DiagKind` — downstream renderers
turn these into editor squiggles, terminal output, or anything else.

What makes the diagnostics worth looking at: they refuse common
foreign syntax with **explicit guidance** instead of a cryptic
"unexpected character":

| Source | Diagnostic |
|---|---|
| `// hi` | `'//' comments are not supported, use '#' instead` |
| `/* hi */` | `'/* ... */' block comments are not supported, use '#' line comments` |
| `key = 1` | `'=' is not used in the standar DSL — write \`key value\` instead of \`key = value\`` |
| `k """hi"""` | `triple-quote multi-line strings are not supported, use \`\`\` ... \`\`\` for multi-line templates` |
| `k "hi\nbye"` (raw newline) | `unterminated string literal (newline in basic string — use \`\`\` ... \`\`\` for multi-line)` |
| `${foo.true}` | `reserved word cannot appear in reference path` |
| `g \`pre${a}\` { }` | `block label cannot contain interpolation` |

The full set is pinned with byte-exact spans by
[`tests/negative.rs`](tests/negative.rs) — wording is part of the
public contract.

### Sample renderer

`standarx-dsl` does not bundle a renderer (to stay dependency-light).
A minimal one looks like this:

```rust
use standarx_dsl::Diag;

fn render(src: &str, diag: &Diag) -> String {
    // Compute the 1-indexed line / column from `diag.span.start`.
    let prefix = &src[..diag.span.start];
    let line = prefix.matches('\n').count() + 1;
    let col = prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1;
    format!("{}:{}: {}", line, col, diag.kind)
}
```

`standarbuild::diag::render` ships a polished terminal renderer with
source frames and ANSI colors — borrow it if you need a richer output.
For editors, the [`standarx-dsl-lsp`](../standarx-dsl-lsp) crate maps
the same `Diag` to LSP `Diagnostic` automatically.

## Status

`1.0.0` — the public API surface (`parse()`, `ast::*`, `diag::*`) is
frozen under semver. The `lexer` and `parser` modules stay `pub` for
advanced consumers (formatters, alternative drivers) but are
`#[doc(hidden)]` and not part of the semver contract — they may
change between minor versions.

**Future-proofing.** All public enums and metadata-carrying structs
are `#[non_exhaustive]`. This means downstreams must:

- include `_ =>` arms in matches against `Stmt`, `Expr`, `InterpExpr`,
  `StringPart`, `TriviaKind`, `Severity`, `DiagKind`;
- construct `Diag` via `Diag::parse` / `Diag::schema` / `Diag::schema_warn`
  instead of struct literals.

In exchange, new variants and fields can land in 1.x minors without a
forced 2.0. `Ident(pub String)` and `Spanned<T>` stay open for
ergonomic construction.

## Ecosystem

| Crate | Purpose |
|---|---|
| `standarx-dsl` (this crate) | Parser core. |
| [`standarx-dsl-grammar`](../standarx-dsl-grammar) | TextMate + language-configuration emitters for editor highlighting. |
| [`standarx-dsl-lsp`](../standarx-dsl-lsp) | Universal LSP backend. Downstream `standarbuild-lsp` / `standardoc-lsp` plug their schema via the [`Schema`](../standarx-dsl-lsp/src/schema.rs) trait. |

## License

MIT
