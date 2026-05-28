//! Universal Language Server Protocol backend for the standarx DSL.
//!
//! Wraps [`standarx_dsl::parse`] and publishes syntactic diagnostics
//! to any LSP-aware editor (VSCode, JetBrains, neovim, Helix, Emacs,
//! Zed, …). The backend is **schema-agnostic** — semantic checks
//! (valid keys, ref resolution, type matching) belong in downstream
//! crates that wrap this server and inject their own schema.
//!
//! # Quick start
//!
//! Embed in your own binary:
//!
//! ```no_run
//! # async fn run() {
//! standarx_dsl_lsp::run_stdio().await;
//! # }
//! ```
//!
//! Or instantiate the [`Backend`] yourself via [`make_service`] when
//! you need to register additional LSP methods (e.g. completion based
//! on a downstream schema).

pub mod conversion;

use std::collections::HashMap;

use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, InitializeParams, InitializeResult, InitializedParams, MessageType,
    ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, Url,
};
use tower_lsp::{Client, LanguageServer, LspService, Server};

use standarx_dsl::{Diag, Severity};

/// Source label attached to every emitted diagnostic.
pub const DIAGNOSTIC_SOURCE: &str = "standarx-dsl";

/// LSP backend wrapping the standarx DSL parser. Holds per-document
/// text state and emits syntactic diagnostics on open / change.
pub struct Backend {
    client: Client,
    docs: RwLock<HashMap<Url, String>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            docs: RwLock::new(HashMap::new()),
        }
    }

    async fn validate(&self, uri: Url, text: String) {
        let diagnostics = match standarx_dsl::parse(&text) {
            Ok(_) => Vec::new(),
            Err(diag) => vec![diag_to_lsp(&text, &diag)],
        };
        self.docs.write().await.insert(uri.clone(), text);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "standarx-dsl LSP ready")
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.validate(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        // FULL sync — single change carries the entire new text.
        let Some(change) = params.content_changes.pop() else {
            return;
        };
        self.validate(params.text_document.uri, change.text).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.docs.write().await.remove(&uri);
        self.client.publish_diagnostics(uri, Vec::new(), None).await;
    }
}

/// Convert a `standarx_dsl::Diag` into an LSP `Diagnostic` anchored
/// at the source byte span.
pub fn diag_to_lsp(src: &str, diag: &Diag) -> Diagnostic {
    Diagnostic {
        range: conversion::span_to_range(src, &diag.span),
        severity: Some(match diag.severity {
            Severity::Error => DiagnosticSeverity::ERROR,
            Severity::Warning => DiagnosticSeverity::WARNING,
        }),
        source: Some(DIAGNOSTIC_SOURCE.into()),
        message: diag.kind.to_string(),
        ..Diagnostic::default()
    }
}

/// Construct an `LspService` wrapping a fresh [`Backend`]. Use this
/// when embedding the backend in a larger LSP server or testing.
pub fn make_service() -> (
    LspService<Backend>,
    tower_lsp::ClientSocket,
) {
    LspService::new(Backend::new)
}

/// Run the standarx LSP server over stdio. Blocks until the client
/// closes the stream.
pub async fn run_stdio() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = make_service();
    Server::new(stdin, stdout, socket).serve(service).await;
}
