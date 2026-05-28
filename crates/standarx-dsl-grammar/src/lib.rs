//! Editor-agnostic grammar definitions for the standarx DSL.
//!
//! [`SPEC`] is the single source of truth describing the surface
//! syntax (keywords, brackets, comment style, string fences,
//! identifier shape). The [`textmate`] and [`lang_config`] modules
//! turn it into the JSON files that VSCode / JetBrains / Sublime /
//! Helix / neovim (via tree-sitter is a separate story) can consume
//! directly.
//!
//! The companion binary `standarx-grammar-gen` emits both files to
//! `--out-dir`. The pre-generated files live in
//! `crates/standarx-dsl-grammar/dist/` and are versioned — downstream
//! editor extensions can copy them as-is.
//!
//! The grammar is intentionally **file-extension agnostic**: each
//! downstream extension declares its own extensions (`.sxb`, `.sxd`,
//! …) and maps them to the `source.standarx` TextMate scope.

pub mod lang_config;
pub mod textmate;

/// Canonical surface-syntax description of the standarx DSL.
///
/// Mirror of what `crates/standarx-dsl/src/lexer.rs` accepts. Update
/// this AND the generated `dist/` files when the lexer grows / shrinks
/// a token kind.
pub struct GrammarSpec {
    /// Human-readable language name (shown in editor pickers).
    pub display_name: &'static str,
    /// TextMate scope root — extensions reference this in their
    /// `grammars[].scopeName` and language picker.
    pub scope_name: &'static str,
    /// Short slug used as the trailing component of every TextMate
    /// inner scope (e.g. `comment.line.number-sign.<slug>`).
    /// Typically the last segment of `scope_name`.
    pub lang_slug: &'static str,
    /// Reserved bare-word literals (no `kind` keywords yet — only
    /// `true`, `false`, `null` are reserved at the lexer level).
    pub language_constants: &'static [&'static str],
    /// Line comment prefix.
    pub line_comment: &'static str,
    /// Matched bracket pairs. Order matters for VSCode bracket
    /// matching (block pairs first, then enclosing pairs).
    pub brackets: &'static [(&'static str, &'static str)],
    /// Pairs that auto-close as the user types.
    pub auto_closing_pairs: &'static [(&'static str, &'static str)],
    /// Surrounding pairs (wraps selection on type).
    pub surrounding_pairs: &'static [(&'static str, &'static str)],
    /// Regex matching an identifier (used as VSCode `wordPattern`).
    pub word_pattern: &'static str,
}

/// The live spec. Touch this when the DSL surface changes.
pub const SPEC: GrammarSpec = GrammarSpec {
    display_name: "Standarx DSL",
    scope_name: "source.standarx",
    lang_slug: "standarx",
    language_constants: &["true", "false", "null"],
    line_comment: "#",
    brackets: &[("{", "}"), ("[", "]")],
    auto_closing_pairs: &[
        ("{", "}"),
        ("[", "]"),
        ("\"", "\""),
        ("`", "`"),
    ],
    surrounding_pairs: &[
        ("{", "}"),
        ("[", "]"),
        ("\"", "\""),
        ("`", "`"),
    ],
    // Bare identifiers: [A-Za-z_][A-Za-z0-9_]* per `is_ident_start`
    // / `is_ident_continue` in the lexer. Ref-path segments and map
    // keys reuse this shape.
    word_pattern: r"[A-Za-z_][A-Za-z0-9_]*",
};
