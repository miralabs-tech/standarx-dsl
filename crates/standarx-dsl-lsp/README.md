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

The backend is exposed as a library so downstream consumers can
register additional LSP methods on top:

```rust
use standarx_dsl_lsp::{make_service, Backend};
use tower_lsp::Server;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let (service, socket) = make_service();
    // Wrap `service` with your own LanguageServer impl that
    // delegates syntactic methods to standarx-dsl-lsp's Backend
    // and adds schema-aware completion / hover / go-to-def.
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;
}
```

(A first-class `Schema` trait extension point will be added once
both `standarbuild-lsp` and `standardoc-lsp` clarify their
requirements — premature abstraction otherwise.)

## License

MIT
