# standarx-dsl-lsp

Universal Language Server Protocol backend for the standarx DSL.

Wraps [`standarx_dsl::parse`](../standarx-dsl) in a tower-lsp
server and publishes syntactic diagnostics to any LSP-aware editor
(VSCode, JetBrains, neovim, Helix, Emacs, Zed).

The backend is **schema-agnostic** — only syntactic errors emitted
by the parser are surfaced. Schema-aware features (key validation,
ref resolution, completion) belong in downstream crates that wrap
this server.

## Build

```bash
cargo build --release -p standarx-dsl-lsp
# binary at target/release/standarx-lsp(.exe)
```

Or install globally:

```bash
cargo install --path crates/standarx-dsl-lsp
```

## Wiring into VSCode

In your extension's `package.json`:

```jsonc
{
  "main": "./out/extension.js",
  "activationEvents": ["onLanguage:standarx"],
  "contributes": {
    "languages": [{
      "id": "standarx",
      "extensions": [".sxb", ".sxd"]
    }]
  }
}
```

In `src/extension.ts` (uses [`vscode-languageclient`](https://www.npmjs.com/package/vscode-languageclient)):

```ts
import { ExtensionContext } from "vscode";
import {
  LanguageClient, LanguageClientOptions, ServerOptions, TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(_ctx: ExtensionContext) {
  const serverOptions: ServerOptions = {
    command: "standarx-lsp",
    transport: TransportKind.stdio,
  };
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "standarx" }],
  };
  client = new LanguageClient(
    "standarx-lsp",
    "Standarx LSP",
    serverOptions,
    clientOptions,
  );
  client.start();
}

export function deactivate() {
  return client?.stop();
}
```

## Wiring into neovim

With `nvim-lspconfig` (or directly via `vim.lsp.start`):

```lua
vim.filetype.add({
  extension = {
    sxb = "standarx",
    sxd = "standarx",
  },
})

vim.api.nvim_create_autocmd("FileType", {
  pattern = "standarx",
  callback = function(args)
    vim.lsp.start({
      name = "standarx-lsp",
      cmd = { "standarx-lsp" },
      root_dir = vim.fs.dirname(args.file),
    })
  end,
})
```

## Wiring into Helix

In `~/.config/helix/languages.toml`:

```toml
[[language]]
name = "standarx"
scope = "source.standarx"
file-types = ["sxb", "sxd"]
roots = []
comment-token = "#"
language-servers = ["standarx-lsp"]

[language-server.standarx-lsp]
command = "standarx-lsp"
```

## Wiring into Zed

In `~/.config/zed/settings.json`:

```jsonc
{
  "languages": {
    "Standarx": {
      "language_servers": ["standarx-lsp"]
    }
  },
  "lsp": {
    "standarx-lsp": {
      "binary": { "path": "standarx-lsp" }
    }
  }
}
```

## Extending with a schema

Implement the [`Schema`](src/schema.rs) trait to layer semantic
features on top of the parser's syntactic ones. The trait surface:

| Method | Default | Returns |
|---|---|---|
| `validate(file, src)` | required | `Vec<Diag>` — extra diagnostics. |
| `completion(file, src, offset)` | empty | `Vec<CompletionItem>` — completion candidates. |
| `hover(file, src, offset)` | `None` | `Option<Hover>` — type / doc / shape under the cursor. |
| `goto_definition(file, src, offset)` | `None` | `Option<Location>` — definition target. |

Implement only what you need; the defaults make every method
optional. `CompletionItem`, `Hover`, `Location` are re-exported
from `tower_lsp::lsp_types` so there is no intermediate translation
layer.

```rust
use standarx_dsl::{Diag, File};
use standarx_dsl_lsp::{
    run_stdio_with_schemas, CompletionItem, Hover, HoverContents,
    Location, MarkupContent, MarkupKind, Schema,
};

struct SxbSchema;

impl Schema for SxbSchema {
    fn validate(&self, _file: &File, _src: &str) -> Vec<Diag> {
        // Walk file.stmts, check each block's `kind` matches a
        // known `.sxb` entity ("project", "task", …), validate
        // the keys inside, etc.
        Vec::new()
    }

    fn completion(
        &self,
        _file: &File,
        _src: &str,
        _offset: usize,
    ) -> Vec<CompletionItem> {
        vec![
            CompletionItem::new_simple("project".into(), "block kind".into()),
            CompletionItem::new_simple("task".into(),    "block kind".into()),
        ]
    }

    fn hover(&self, _file: &File, _src: &str, _offset: usize) -> Option<Hover> {
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "**project** — top-level entity.".into(),
            }),
            range: None,
        })
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    run_stdio_with_schemas(vec![Box::new(SxbSchema)]).await;
}
```

The backend advertises `completionProvider`, `hoverProvider`, and
`definitionProvider` capabilities unconditionally — when no schema
overrides a method, the editor sees "no candidates" / "no info"
rather than "feature unsupported".

Composition rules across multiple schemas:

- `validate` and `completion` results are **concatenated** in
  registration order.
- `hover` and `goto_definition` are **first-wins** — composing
  these across schemas yields a worse UX than picking the most
  specific answer.

For embedding inside a larger server (custom LSP methods, custom
transport) use `make_service` / `make_service_with_schemas` instead
and drive the `tower_lsp::Server` yourself.

## License

MIT
