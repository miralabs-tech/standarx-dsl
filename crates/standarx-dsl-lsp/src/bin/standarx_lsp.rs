//! `standarx-lsp` — universal LSP server for the standarx DSL.
//!
//! Speaks LSP over stdio. Pair with an editor extension that
//! registers `.sxb` / `.sxd` (or any other extension) as a language
//! whose server command is this binary.

#[tokio::main(flavor = "current_thread")]
async fn main() {
    standarx_dsl_lsp::run_stdio().await;
}
