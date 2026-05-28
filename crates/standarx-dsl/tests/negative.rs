//! Branch-by-branch negative tests for `Diag::parse(...)` emission.
//!
//! Pins the **message** (substring match) and the **span** of every
//! parse error the lexer / parser currently produces. A regression
//! that changes wording or span placement will fail here.
//!
//! Data-driven: each row is a (label, input, expected message
//! substring, expected span) tuple. Cheap to extend.

use standarx_dsl::{parse, Severity};

fn expect_err(label: &str, src: &str, msg_substring: &str, span: std::ops::Range<usize>) {
    let diag = parse(src).err().unwrap_or_else(|| {
        panic!("[{label}] expected parse error on {src:?}, got Ok");
    });
    assert!(
        diag.is_error(),
        "[{label}] expected Error severity, got {:?}",
        diag.severity
    );
    let rendered = diag.kind.to_string();
    assert!(
        rendered.contains(msg_substring),
        "[{label}] message {rendered:?} does not contain {msg_substring:?}"
    );
    assert_eq!(
        diag.span, span,
        "[{label}] span mismatch on {src:?}: got {:?}, expected {:?}",
        diag.span, span
    );
}

// Helper for cases where the exact span isn't worth pinning (still
// checks the message + Error severity).
fn expect_msg(label: &str, src: &str, msg_substring: &str) {
    let diag = parse(src).err().unwrap_or_else(|| {
        panic!("[{label}] expected parse error on {src:?}, got Ok");
    });
    assert_eq!(diag.severity, Severity::Error);
    let rendered = diag.kind.to_string();
    assert!(
        rendered.contains(msg_substring),
        "[{label}] message {rendered:?} does not contain {msg_substring:?}"
    );
}

#[test]
fn rejects_c_style_comments() {
    expect_err(
        "// line",
        "// hi\n",
        "'//' comments are not supported",
        0..2,
    );
    expect_err(
        "/* block */",
        "/* hi */",
        "'/* ... */' block comments are not supported",
        0..2,
    );
}

#[test]
fn rejects_equals_assignment() {
    expect_err(
        "= sign",
        "key = 1",
        "'=' is not used in the standar DSL",
        4..5,
    );
}

#[test]
fn rejects_unexpected_top_level_chars() {
    expect_err("@ sign", "@", "unexpected character", 0..1);
    expect_err("? sign", "?", "unexpected character", 0..1);
}

#[test]
fn rejects_number_errors() {
    expect_msg("dash alone", "k -", "expected digit after '-'");
}

#[test]
fn rejects_string_errors() {
    expect_err(
        "triple quote",
        "k \"\"\"x\"\"\"",
        "triple-quote multi-line strings are not supported",
        2..5,
    );
    expect_msg(
        "unterminated string",
        "k \"hi",
        "unterminated string literal",
    );
    expect_msg(
        "newline in basic string",
        "k \"hi\nbye\"",
        "newline in basic string",
    );
    expect_msg("dangling backslash", "k \"hi\\", "dangling backslash");
    expect_msg("unknown escape", r#"k "\q""#, "unknown escape");
}

#[test]
fn rejects_unicode_escape_errors() {
    expect_msg(
        "missing brace after \\u",
        r#"k "\u41""#,
        "expected '{' after \\u",
    );
    expect_msg(
        "invalid hex in \\u{}",
        r#"k "\u{zzz}""#,
        "invalid \\u{...} hex digits",
    );
    expect_msg(
        "missing close brace in \\u{}",
        r#"k "\u{41 ""#,
        "expected '}' to close \\u{...}",
    );
    expect_msg(
        "out-of-range codepoint",
        r#"k "\u{ffffff}""#,
        "invalid unicode codepoint",
    );
}

#[test]
fn rejects_template_errors() {
    expect_msg(
        "unterminated inline template",
        "k `hi",
        "unterminated template literal",
    );
    expect_msg(
        "newline in inline template",
        "k `hi\nbye`",
        "newline in inline template",
    );
    expect_msg(
        "unterminated multi-line template",
        "k ```hi",
        "unterminated multi-line template",
    );
    expect_msg(
        "dangling backslash in template",
        "k `hi\\",
        "dangling backslash",
    );
    expect_msg("unknown escape in template", "k `\\q`", "unknown escape");
}

#[test]
fn rejects_interpolation_errors() {
    expect_msg(
        "unclosed interp",
        "k `pre${a",
        "expected '}' to close interpolation",
    );
    expect_msg(
        "dangling dot in interp ref",
        "k `${foo.}`",
        "expected identifier or quoted segment after '.'",
    );
    expect_msg(
        "reserved word in interp ref",
        "k `${foo.true}`",
        "reserved word cannot appear in reference path",
    );
    expect_msg(
        "garbage inside interp",
        "k `${,}`",
        "expected ref, number, bool, null, or string inside ${...}",
    );
    // Note: the "nested interpolation" branch in
    // `lex_template_token` is unreachable through the public
    // `parse()` entry because `lex_interp_body` does not accept
    // template syntax (`` ` ``) — only plain strings, refs, scalars.
    // The branch survives as defense-in-depth, but no test pins
    // its message here.
}

#[test]
fn rejects_parser_top_level_errors() {
    expect_msg(
        "block without key",
        "{ broken }",
        "expected identifier at start of statement",
    );
    expect_msg(
        "key with no value",
        "k",
        "expected value or '{' after identifier",
    );
    expect_msg("unterminated block", "g { k 1", "unterminated block body");
}

#[test]
fn rejects_parser_expr_errors() {
    expect_msg("bare comma as value", "k ,", "expected expression");
    expect_msg(
        "dangling dot in ref path",
        "k foo.",
        "expected identifier or quoted segment after '.'",
    );
    expect_msg("unterminated list", "k [ 1 2", "unterminated list");
    // Map literal at expression position (inside a list) — the
    // top-level `k { ... }` form is parsed as a block, not a map.
    expect_msg("unterminated map", "k [{ a 1", "unterminated map");
}

#[test]
fn rejects_bare_text_constraints_via_helper() {
    // The shared `StringLit::try_into_bare_text` helper — exercised
    // through the parser's block-label and ref-path positions.
    // (The lexer-side ref-segment in `${...}` only accepts `"..."`
    // plain strings, which cannot be multi-line and cannot contain
    // interpolation by construction, so its branches survive as
    // defense-in-depth but are not user-reachable.)
    expect_msg(
        "block label multiline",
        "g ```\nhi\n``` { }",
        "block label cannot be a multi-line string",
    );
    expect_msg(
        "block label with interp",
        "g `pre${a}post` { }",
        "block label cannot contain interpolation",
    );
    expect_msg(
        "ref segment multiline",
        "k foo.```\nx\n```",
        "ref segment cannot be a multi-line string",
    );
    expect_msg(
        "ref segment with interp",
        "k foo.`pre${a}post`",
        "ref segment cannot contain interpolation",
    );
}
