//! Universal Language Server Protocol backend for the standarx DSL.
//!
//! Wraps [`standarx_dsl::parse`] and publishes syntactic diagnostics
//! to any LSP-aware editor (VSCode, JetBrains, neovim, Helix, Emacs,
//! Zed, …). Out of the box the backend is syntax-only; downstream
//! crates inject semantic checks via the [`Schema`] trait.
//!
//! # Quick start
//!
//! Embed in your own binary with no schema:
//!
//! ```no_run
//! # async fn run() {
//! standarx_dsl_lsp::run_stdio().await;
//! # }
//! ```
//!
//! Or with one or more schemas:
//!
//! ```no_run
//! # use standarx_dsl_lsp::Schema;
//! # use standarx_dsl::{Diag, File};
//! # struct MySchema;
//! # impl Schema for MySchema {
//! #     fn validate(&self, _f: &File, _s: &str) -> Vec<Diag> { Vec::new() }
//! # }
//! # async fn run() {
//! standarx_dsl_lsp::run_stdio_with_schemas(vec![Box::new(MySchema)]).await;
//! # }
//! ```

pub mod conversion;
pub mod schema;

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

pub use schema::Schema;

/// Source label attached to every emitted diagnostic.
pub const DIAGNOSTIC_SOURCE: &str = "standarx-dsl";

/// LSP backend wrapping the standarx DSL parser. Holds per-document
/// text state and emits diagnostics on open / change. Optional
/// [`Schema`] implementors layer semantic checks on top.
pub struct Backend {
    client: Client,
    docs: RwLock<HashMap<Url, String>>,
    schemas: Vec<Box<dyn Schema>>,
}

impl Backend {
    /// Backend with no semantic schemas — syntactic diagnostics only.
    pub fn new(client: Client) -> Self {
        Self::new_with(client, Vec::new())
    }

    /// Backend with a set of semantic schemas. Their diagnostics are
    /// concatenated in registration order after parser-emitted
    /// diagnostics.
    pub fn new_with(client: Client, schemas: Vec<Box<dyn Schema>>) -> Self {
        Self {
            client,
            docs: RwLock::new(HashMap::new()),
            schemas,
        }
    }

    async fn validate(&self, uri: Url, text: String) {
        let diagnostics = collect_diagnostics(&text, &self.schemas);
        self.docs.write().await.insert(uri.clone(), text);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

/// Pure helper: parse `src`, run every schema's `validate`, and
/// turn the resulting diagnostics into LSP-shaped `Diagnostic`s.
///
/// Exposed for testing and for embedders that want to drive the
/// validation pipeline without going through tower-lsp.
pub fn collect_diagnostics(src: &str, schemas: &[Box<dyn Schema>]) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    match standarx_dsl::parse(src) {
        Ok(file) => {
            for schema in schemas {
                for diag in schema.validate(&file, src) {
                    out.push(diag_to_lsp(src, &diag));
                }
            }
        }
        Err(diag) => out.push(diag_to_lsp(src, &diag)),
    }
    out
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
            // `standarx_dsl::Severity` is #[non_exhaustive]; future
            // additions (e.g. Info, Note) land here as the closest
            // LSP equivalent. Bump this arm when standarx-dsl grows
            // a variant we want to surface distinctly.
            _ => DiagnosticSeverity::INFORMATION,
        }),
        source: Some(DIAGNOSTIC_SOURCE.into()),
        message: diag.kind.to_string(),
        ..Diagnostic::default()
    }
}

/// Construct an `LspService` wrapping a fresh schema-less [`Backend`].
pub fn make_service() -> (LspService<Backend>, tower_lsp::ClientSocket) {
    LspService::new(Backend::new)
}

/// Construct an `LspService` wrapping a [`Backend`] equipped with
/// the given semantic schemas.
pub fn make_service_with_schemas(
    schemas: Vec<Box<dyn Schema>>,
) -> (LspService<Backend>, tower_lsp::ClientSocket) {
    LspService::new(move |client| Backend::new_with(client, schemas))
}

/// Run the standarx LSP server over stdio with no semantic schemas.
pub async fn run_stdio() {
    let (service, socket) = make_service();
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;
}

/// Run the standarx LSP server over stdio with the given semantic
/// schemas plugged in.
pub async fn run_stdio_with_schemas(schemas: Vec<Box<dyn Schema>>) {
    let (service, socket) = make_service_with_schemas(schemas);
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;
}
