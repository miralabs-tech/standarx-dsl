# standarx-dsl-grammar

Editor-agnostic grammar definitions for the standarx DSL.

A single Rust source of truth (`SPEC: GrammarSpec` in `src/lib.rs`)
emits two consumable files:

- **`standarx.tmLanguage.json`** — TextMate grammar. Consumed
  natively by VSCode, JetBrains (via TextMate Bundles), Sublime
  Text, and as a fallback by Helix.
- **`standarx.language-configuration.json`** — VSCode-style
  brackets / comments / auto-pairs / word boundary. Other editors
  (JetBrains, Helix) tend to inline equivalents but can map this
  file 1:1.

Pre-generated copies live in [`dist/`](dist/) and are versioned —
downstream extensions can copy them as-is, no codegen required at
install time.

## Regenerating

```bash
cargo run -p standarx-dsl-grammar --bin standarx-grammar-gen -- \
    --out-dir crates/standarx-dsl-grammar/dist
```

Or print a single document to stdout:

```bash
cargo run -p standarx-dsl-grammar --bin standarx-grammar-gen -- \
    --stdout textmate    # or: --stdout config
```

## TextMate scopes

All inner scopes suffix the language slug `.standarx`. The
top-level `scopeName` is `source.standarx`. Highlights covered:

| Pattern | Scope |
|---|---|
| `# line comment` | `comment.line.number-sign.standarx` |
| `true` / `false` / `null` | `constant.language.standarx` |
| Integer / float | `constant.numeric.{integer,float}.standarx` |
| `"plain string"` | `string.quoted.double.standarx` |
| `` `inline template` `` | `string.quoted.other.template.standarx` |
| ` ```multi-line``` ` | `string.quoted.other.template.multiline.standarx` |
| `${expr}` interpolation | `meta.interpolation.standarx` |
| `{}` blocks | `punctuation.section.block.standarx` |
| `[]` arrays | `punctuation.section.brackets.standarx` |
| Identifier | `variable.other.standarx` |

## Wiring it into a VSCode extension

In your extension's `package.json`:

```jsonc
{
  "contributes": {
    "languages": [
      {
        "id": "standarx",
        "aliases": ["Standarx DSL"],
        "extensions": [".sxb", ".sxd"],
        "configuration": "./language-configuration.json"
      }
    ],
    "grammars": [
      {
        "language": "standarx",
        "scopeName": "source.standarx",
        "path": "./syntaxes/standarx.tmLanguage.json"
      }
    ]
  }
}
```

Then copy the two files from `dist/`:

```bash
mkdir -p ext/syntaxes
cp crates/standarx-dsl-grammar/dist/standarx.tmLanguage.json ext/syntaxes/
cp crates/standarx-dsl-grammar/dist/standarx.language-configuration.json \
   ext/language-configuration.json
```

`extensions` is where each downstream project picks its own file
extension(s) — the grammar itself doesn't care.

## Wiring it into other editors

- **JetBrains** (IntelliJ / RustRover / WebStorm): install the
  *TextMate Bundles* plugin (bundled since 2020+), then
  *Settings → Editor → TextMate Bundles → +* and point at the
  `dist/` directory. JetBrains will pick up the grammar and apply
  it to files matching the language definition.
- **Sublime Text** / **Helix**: drop the `.tmLanguage.json` into
  their respective grammar directories (`~/.config/sublime-text/Packages/User/` /
  Helix's `runtime/grammars/`).
- **neovim**: TextMate-based highlighting is not native; use a
  Tree-sitter grammar instead (planned: `standarx-dsl-treesitter`).
  Meanwhile, `standarx-dsl-lsp` provides live diagnostics through
  neovim's LSP client.

## License

MIT
