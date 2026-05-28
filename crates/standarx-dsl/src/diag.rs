//! Diagnostic primitives: spans, severities, error type.
//!
//! Vendored from `standarbuild`'s `diag` module (the renderer is left to
//! downstream consumers — see `standarbuild::diag::render` for a sample).

use std::ops::Range;

pub type Span = Range<usize>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }

    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned {
            node: f(self.node),
            span: self.span,
        }
    }

    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned {
            node: &self.node,
            span: self.span.clone(),
        }
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for Spanned<T> {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        self.node.serialize(ser)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    serde(rename_all = "snake_case")
)]
#[non_exhaustive]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum DiagKind {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("schema error: {0}")]
    Schema(String),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Diag {
    pub kind: DiagKind,
    pub span: Span,
    pub severity: Severity,
}

impl Diag {
    pub fn parse(span: Span, msg: impl Into<String>) -> Self {
        Self {
            kind: DiagKind::Parse(msg.into()),
            span,
            severity: Severity::Error,
        }
    }

    pub fn schema(span: Span, msg: impl Into<String>) -> Self {
        Self {
            kind: DiagKind::Schema(msg.into()),
            span,
            severity: Severity::Error,
        }
    }

    pub fn schema_warn(span: Span, msg: impl Into<String>) -> Self {
        Self {
            kind: DiagKind::Schema(msg.into()),
            span,
            severity: Severity::Warning,
        }
    }

    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self.severity, Severity::Error)
    }
}

impl std::fmt::Display for Diag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Delegate to the kind so `format!("{diag}")` reads the same as
        // `format!("{}", diag.kind)` — the span / severity belong on the
        // caller's renderer, not the bare Display.
        self.kind.fmt(f)
    }
}

impl std::error::Error for Diag {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.kind)
    }
}
