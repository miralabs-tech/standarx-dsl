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
diagnostics on top of the parser's syntactic ones. The backend
runs every registered schema after a successful parse and
forwards their `Vec<Diag>` to the LSP client.

```rust
use standarx_dsl::{Diag, File};
use standarx_dsl_lsp::{run_stdio_with_schemas, Schema};

struct SxbSchema;

impl Schema for SxbSchema {
    fn validate(&self, file: &File, _src: &str) -> Vec<Diag> {
        let mut diags = Vec::new();
        // Walk file.stmts, check each block's `kind` matches a
        // known `.sxb` entity ("project", "task", …), validate the
        // keys inside, etc. Emit `Diag::schema(span, msg)` for
        // errors and `Diag::schema_warn(span, msg)` for warnings.
        diags
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    run_stdio_with_schemas(vec![Box::new(SxbSchema)]).await;
}
```

Multiple schemas can be combined — their diagnostics are
concatenated in registration order:

```rust
run_stdio_with_schemas(vec![
    Box::new(SxbSchema),
    Box::new(DeprecationSchema),
]).await;
```

For embedding inside a larger server (custom LSP methods, custom
transport) use `make_service` / `make_service_with_schemas`
instead and drive the `tower_lsp::Server` yourself.

The trait is intentionally minimal in this first iteration —
only `validate` is on the contract. Completion / hover / go-to-def
extension methods will be added once a real consumer's needs
clarify their shape.

## License

MIT
