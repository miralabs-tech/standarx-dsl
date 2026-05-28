use crate::diag::{Diag, Span, Spanned};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct File {
    pub stmts: Vec<StmtNode>,
    pub trailing_trivia: Vec<Trivia>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StmtNode {
    pub leading: Vec<Trivia>,
    pub trailing: Option<Trivia>,
    pub node: Stmt,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", content = "text", rename_all = "snake_case")]
pub enum TriviaKind {
    LineComment(String),
    BlockComment(String),
    BlankLine,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Trivia {
    pub kind: TriviaKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Stmt {
    Assign(Assign),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Assign {
    pub key: Spanned<Ident>,
    pub value: Spanned<Expr>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Block {
    pub kind: Spanned<Ident>,
    pub label: Option<Spanned<String>>,
    pub stmts: Vec<StmtNode>,
    pub trailing_trivia: Vec<Trivia>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct Ident(pub String);

impl Ident {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    String(StringLit),
    Ref(Ref),
    List(Vec<Spanned<Expr>>),
    Map(Vec<MapEntry>),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Ref {
    pub path: Vec<Spanned<Ident>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MapEntry {
    pub key: Spanned<Ident>,
    pub value: Spanned<Expr>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StringLit {
    pub parts: Vec<StringPart>,
    pub multiline: bool,
    /// true for `` `...` `` and `` ```...``` `` (interpolation allowed),
    /// false for plain `"..."` (literal, no `${}` substitution).
    pub template: bool,
}

impl StringLit {
    /// Decode this string literal as a single-line, non-interpolated
    /// text payload — the shape required for bare-name positions:
    /// reference-path segments, block labels, future map keys.
    ///
    /// Rejects multi-line strings and embedded `${...}`
    /// interpolations with a diagnostic naming `context` so the
    /// user knows where the constraint applies (e.g. "ref segment",
    /// "block label"). `span` is the source span of the literal
    /// token, reused for the diagnostic.
    ///
    /// Shared by [`crate::lexer`]'s interpolation-body lexer and
    /// [`crate::parser`]'s ref-path / block-label paths so the
    /// rules cannot drift apart.
    pub fn try_into_bare_text(self, span: Span, context: &str) -> Result<String, Diag> {
        if self.multiline {
            return Err(Diag::parse(
                span,
                format!("{context} cannot be a multi-line string"),
            ));
        }
        if self
            .parts
            .iter()
            .any(|p| matches!(p, StringPart::Interp(_)))
        {
            return Err(Diag::parse(
                span,
                format!("{context} cannot contain interpolation"),
            ));
        }
        match self.parts.into_iter().next() {
            Some(StringPart::Lit(s)) => Ok(s),
            None => Ok(String::new()),
            Some(StringPart::Interp(_)) => unreachable!("filtered above"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum StringPart {
    Lit(String),
    Interp(Spanned<InterpExpr>),
}

/// Expression body legal inside a `${...}` interpolation segment.
///
/// Parallel to [`Expr`] but **intentionally narrower** — `List` and
/// `Map` are excluded. Interpolation is for substituting *scalar*
/// values into a surrounding string, so collection literals would
/// produce a meaningless `Display` and invite footguns ("did the
/// user mean to splat?"). Refs to scalar values are allowed because
/// they resolve to scalars by the time interpolation runs.
///
/// `String` carries an owned literal already lex-decoded (no nested
/// interpolation — `${...}` does not recurse inside its own body).
///
/// Keep this enum in sync with [`Expr`] when adding scalar variants;
/// do **not** unify the two without explicit design discussion.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum InterpExpr {
    Ref(Ref),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    String(String),
}
