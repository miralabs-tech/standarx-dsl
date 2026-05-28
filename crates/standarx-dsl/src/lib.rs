//! Reusable DSL parser for the `standar*` ecosystem.
//!
//! The DSL grammar supports nested blocks, arrays, scalar literals, and
//! interpolated strings — designed to host config files like
//! `standarbuild`'s `.sxb` (task definitions) and `standardoc`'s `.sxd`
//! (workspace config). The parser is consumer-agnostic: it produces a
//! generic [`ast::File`] tree ; downstream crates lower it into their
//! own typed schemas.
//!
//! # Example
//!
//! ```no_run
//! let src = r#"
//!     version "0.1.0"
//!     project "ext" {
//!         path "./ext"
//!         tasks ["build" "test"]
//!     }
//! "#;
//! let file = standarx_dsl::parse(src).expect("parse ok");
//! assert_eq!(file.stmts.len(), 2);
//! ```

pub mod ast;
pub mod diag;

// `lexer` and `parser` are kept `pub` so adventurous consumers (formatters,
// linters, alternative drivers) can still reach the token stream and the
// parser entry point. They are NOT part of the semver public contract —
// the supported API surface is `parse()` + `ast::*` + `diag::*`. Anything
// else may break between minor versions until the modules stabilise.
#[doc(hidden)]
pub mod lexer;
#[doc(hidden)]
pub mod parser;

pub use ast::{File, Stmt};
pub use diag::{Diag, DiagKind, Severity, Span, Spanned};

/// Parse a `.sxb` / `.sxd` source string into an [`ast::File`] tree.
///
/// Errors carry a [`Span`] referring to the offending byte range in `src`
/// — downstream consumers can render them however they prefer (see
/// `standarbuild`'s `diag::render` module for a sample renderer).
#[must_use = "parse() returns a Result that carries diagnostics; ignoring it discards parse errors"]
pub fn parse(src: &str) -> Result<File, Diag> {
    let tokens = lexer::tokenize(src)?;
    parser::parse_tokens(tokens, src.len()..src.len())
}

/// Parse a source string with **error recovery**: always returns a
/// (partial) [`File`] plus the list of every parser diagnostic that
/// fired during the walk.
///
/// Useful for editor / LSP workflows where the user is mid-edit and
/// reporting every problem in one pass is more helpful than
/// fail-fast. The recovery strategy is intentionally conservative:
///
/// - Lexer errors are still fatal (there's no token stream to walk).
///   The returned `File` is empty and the single diagnostic is in
///   the `Vec`.
/// - At the top level, every failed statement is dropped, the
///   parser syncs to the next likely statement start (next `Ident`
///   at brace-depth 0), and parsing continues.
/// - Block-internal errors fail the whole enclosing top-level
///   statement; recovery resumes at the next top-level boundary.
///   This keeps diagnostic positions intuitive (every reported error
///   lives where the user can find it) at the cost of some lost
///   coverage inside large failing blocks.
///
/// The returned `Vec` is empty iff parsing fully succeeded — at
/// which point `parse_with_recovery(src)` and `parse(src).ok()`
/// agree on the file.
#[must_use = "parse_with_recovery() returns the diagnostic list; ignoring it loses the errors"]
pub fn parse_with_recovery(src: &str) -> (File, Vec<Diag>) {
    let tokens = match lexer::tokenize(src) {
        Ok(t) => t,
        Err(diag) => {
            return (
                File {
                    stmts: Vec::new(),
                    trailing_trivia: Vec::new(),
                },
                vec![diag],
            );
        }
    };
    parser::parse_tokens_with_recovery(tokens, src.len()..src.len())
}
