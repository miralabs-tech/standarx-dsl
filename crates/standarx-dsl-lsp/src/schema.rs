//! Extension point for schema-aware diagnostics.
//!
//! `standarx-dsl-lsp` parses the document and reports syntactic
//! errors out of the box. Downstream consumers (`standarbuild-lsp`,
//! `standardoc-lsp`, your own) implement [`Schema`] to layer
//! semantic checks (valid keys per block kind, ref resolution,
//! type matching) on the parsed [`standarx_dsl::File`] tree.
//!
//! Multiple schemas can be combined — their diagnostics are
//! concatenated in registration order.
//!
//! The trait is intentionally minimal in this first iteration.
//! Completion / hover / go-to-def extension methods will be added
//! once a real consumer's needs clarify the shape — premature
//! abstraction otherwise.

use standarx_dsl::{Diag, File};

/// Schema-driven semantic validator over a parsed standarx file.
///
/// Implementors run after `parse()` succeeds and return any
/// additional diagnostics — typically schema violations like
/// "unknown key `xyz` in `project` block" or "task ref points to
/// undeclared `build`".
pub trait Schema: Send + Sync {
    /// Validate the parsed file. Returned diagnostics are forwarded
    /// to the LSP client alongside parser-emitted diagnostics. May
    /// produce both errors and warnings (see
    /// [`Diag::schema`](standarx_dsl::Diag::schema) and
    /// [`Diag::schema_warn`](standarx_dsl::Diag::schema_warn)).
    ///
    /// `src` is the raw source text — useful for context-dependent
    /// messages or for re-parsing sub-spans.
    fn validate(&self, file: &File, src: &str) -> Vec<Diag>;
}
