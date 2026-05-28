//! Extension point for schema-aware LSP features.
//!
//! `standarx-dsl-lsp` parses the document and reports syntactic
//! errors out of the box. Downstream consumers (`standarbuild-lsp`,
//! `standardoc-lsp`, your own) implement [`Schema`] to layer
//! semantic features on top:
//!
//! - **diagnostics** beyond the syntactic ones — invalid keys per
//!   block kind, unresolved references, type mismatches;
//! - **completion** — what idents make sense at the cursor;
//! - **hover** — type / doc / shape for the symbol under the cursor;
//! - **goto-definition** — where a reference resolves to.
//!
//! Multiple schemas can be combined — their results are concatenated
//! (diagnostics, completions) or first-wins (hover, goto). All four
//! methods carry a **default impl returning the empty answer**, so
//! implementors only override what they actually want to provide.
//!
//! LSP types are re-exported from `tower_lsp::lsp_types` — no
//! intermediate translation layer.

use standarx_dsl::{Diag, File};
pub use tower_lsp::lsp_types::{CompletionItem, Hover, Location};

/// Schema-driven extension for a downstream DSL flavour.
pub trait Schema: Send + Sync {
    /// Run semantic validation over a parsed file. Returned
    /// diagnostics are forwarded alongside parser-emitted ones.
    fn validate(&self, file: &File, src: &str) -> Vec<Diag>;

    /// Propose completion items at the given byte offset.
    ///
    /// The default impl returns an empty list — implementors
    /// override only when they have semantic context to offer.
    fn completion(&self, _file: &File, _src: &str, _offset: usize) -> Vec<CompletionItem> {
        Vec::new()
    }

    /// Provide hover information (type, docstring, shape) for the
    /// symbol under the cursor.
    ///
    /// Default: `None`.
    fn hover(&self, _file: &File, _src: &str, _offset: usize) -> Option<Hover> {
        None
    }

    /// Resolve the reference at the cursor to its definition site.
    ///
    /// Default: `None`.
    fn goto_definition(&self, _file: &File, _src: &str, _offset: usize) -> Option<Location> {
        None
    }
}
