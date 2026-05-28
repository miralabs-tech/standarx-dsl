//! Smoke test : parse a real-world `.sxb` file (the standardoc workspace's
//! standarbuild config) to verify the DSL round-trips intact through the
//! extracted crate.

const SXB_SAMPLE: &str = r#"
# Auto-generated from {WorkspaceFolder} by `standarbuild init`.
version "0.1.0"

project "ext-vscode" {
  path "./ext/vscode"
  name "ext-vscode"
  type "bun"

  metadata {
    env_keys ["VSCE_PAT" "OVSX_PAT"]
  }

  task "install" { cmd "bun install" depends_on [task.version] }
  task "build" { cmd "bun run build" }
}
"#;

#[test]
fn parses_standarbuild_sample() {
    let file = standarx_dsl::parse(SXB_SAMPLE).expect("parse sxb sample");
    assert!(!file.stmts.is_empty(), "expected at least one stmt");
    // version "0.1.0" + project { ... } => 2 top-level stmts
    assert_eq!(file.stmts.len(), 2);
}

#[test]
fn parses_proposed_sxd_template() {
    // Proposed standardoc.sxd v0.1 template — verifies the DSL supports
    // the schema this ADR locks in.
    // Multi-line raw strings use triple-backticks (markdown-style).
    let src = r#"
version "0.1.0"

ignore {
  patterns ```
.git/
node_modules/
target/
```
}

projects {
  exclude ["crates-standardoc-graph-viz-pkg"]
}

group "standardoc" {
  label "Standardoc"
  members [
    "standardoc-core"
    "standardoc-ir"
  ]
}
"#;
    let file = standarx_dsl::parse(src).expect("parse sxd template");
    // version + ignore + projects + group  = 4 stmts
    assert_eq!(file.stmts.len(), 4);
}

#[test]
fn empty_input_parses() {
    let file = standarx_dsl::parse("").expect("empty parse");
    assert!(file.stmts.is_empty());
}

#[test]
fn parse_error_carries_span() {
    // `{` without preceding ident is a parse error.
    let err = standarx_dsl::parse("{ broken }").expect_err("expected parse error");
    assert!(err.is_error());
}

fn parse_single_string(src: &str) -> String {
    use standarx_dsl::ast::{Expr, Stmt, StringPart};
    let file = standarx_dsl::parse(src).expect("parse");
    assert_eq!(file.stmts.len(), 1, "expected exactly one stmt: {src:?}");
    let Stmt::Assign(assign) = &file.stmts[0].node else {
        panic!("expected assign stmt, got {:?}", file.stmts[0].node);
    };
    let Expr::String(lit) = &assign.value.node else {
        panic!("expected string expr, got {:?}", assign.value.node);
    };
    assert_eq!(lit.parts.len(), 1, "expected single lit part: {src:?}");
    let StringPart::Lit(s) = &lit.parts[0] else {
        panic!("expected Lit part, got {:?}", lit.parts[0]);
    };
    s.clone()
}

#[test]
fn multibyte_chars_in_plain_string_roundtrip() {
    // 2-byte (Latin-1 supplement).
    assert_eq!(parse_single_string("k \"café\""), "café");
    // 3-byte (CJK ideograms).
    assert_eq!(parse_single_string("k \"日本語\""), "日本語");
    // 4-byte (emoji, supplementary plane).
    assert_eq!(parse_single_string("k \"🦀\""), "🦀");
    // Mixed + multi-byte adjacent to an ASCII escape — stresses the
    // "no special-case byte between UTF-8 continuation bytes" invariant
    // of `push_byte`.
    assert_eq!(parse_single_string(r#"k "café \"x\" 🦀""#), "café \"x\" 🦀");
}

#[test]
fn multibyte_chars_in_template_string_roundtrip() {
    // Same alphabet through the inline template lexer (`...`).
    assert_eq!(parse_single_string("k `café`"), "café");
    assert_eq!(parse_single_string("k `日本語 🦀`"), "日本語 🦀");
}

#[test]
fn multibyte_chars_in_multiline_template_roundtrip() {
    // Through `lex_multiline_template` (triple-backtick block).
    let src = "k ```\ncafé\n日本語\n🦀\n```";
    assert_eq!(parse_single_string(src), "\ncafé\n日本語\n🦀\n");
}
