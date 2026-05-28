//! `parse_with_recovery` contract — multi-error recovery on
//! malformed input.
//!
//! Pins the recovery strategy: every reported diagnostic sits at a
//! top-level statement boundary, the surviving `File` contains only
//! fully-parsed statements, and a valid input round-trips with an
//! empty diagnostic list.

use standarx_dsl::{parse, parse_with_recovery};

#[test]
fn valid_input_returns_empty_diag_list() {
    let src = r#"
        version "0.1.0"
        project "ext" {
            path "./ext"
        }
    "#;
    let (file, diags) = parse_with_recovery(src);
    assert!(diags.is_empty(), "expected no diags, got {diags:?}");
    let ok = parse(src).expect("fail-fast variant should agree");
    assert_eq!(file.stmts.len(), ok.stmts.len());
}

#[test]
fn collects_multiple_top_level_errors() {
    // Two malformed `{ ... }` lines, scalars in between so the
    // block-label lookahead doesn't bridge them. The sync routine
    // consumes each malformed block whole; surviving statements
    // parse normally.
    let src = "\
{ bad1 }
ok1 42
{ bad2 again }
ok2 100
";
    let (file, diags) = parse_with_recovery(src);
    assert_eq!(diags.len(), 2, "expected 2 errors, got {diags:?}");
    assert_eq!(file.stmts.len(), 2);
}

#[test]
fn block_internal_error_skips_to_next_top_level() {
    // The first block has a malformed inner stmt (`1` at stmt
    // start position — an Int is not a valid statement head).
    // Recovery drops the whole containing block and continues
    // with the next top-level entry.
    let src = "\
broken { 1 2 3 }
ok \"value\"
";
    let (file, diags) = parse_with_recovery(src);
    assert!(!diags.is_empty(), "expected at least one error");
    // `ok` survives.
    assert!(
        file.stmts
            .iter()
            .any(|s| matches!(&s.node, standarx_dsl::Stmt::Assign(_))),
        "expected the trailing `ok` assignment to survive recovery"
    );
}

#[test]
fn lexer_error_is_single_fatal_diag() {
    // `// hi` is a lexer-level refusal (`//` comments). No tokens
    // → no recovery possible.
    let src = "// hi\nok \"v\"";
    let (file, diags) = parse_with_recovery(src);
    assert_eq!(diags.len(), 1);
    assert!(file.stmts.is_empty());
}

#[test]
fn recovery_terminates_on_pathological_input() {
    // A long string of garbage at top level — recovery must NOT
    // spin forever. We just check it returns in finite time.
    let src = "@".repeat(200);
    let (file, diags) = parse_with_recovery(&src);
    assert!(!diags.is_empty());
    assert!(file.stmts.is_empty());
}

#[test]
fn empty_input_recovers_cleanly() {
    let (file, diags) = parse_with_recovery("");
    assert!(file.stmts.is_empty());
    assert!(diags.is_empty());
}
