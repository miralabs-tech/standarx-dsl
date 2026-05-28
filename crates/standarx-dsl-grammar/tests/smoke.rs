//! Smoke tests for the grammar emitters.
//!
//! The output is consumed by editor extensions — verify the shape
//! does not silently regress (top-level keys, scope naming,
//! bracket pairs).

use serde_json::Value;
use standarx_dsl_grammar::{lang_config, textmate, SPEC};

#[test]
fn textmate_grammar_has_required_top_level_keys() {
    let g = textmate::grammar_json();
    assert_eq!(g["name"], SPEC.display_name);
    assert_eq!(g["scopeName"], SPEC.scope_name);
    assert!(g["patterns"].is_array());
    assert!(g["repository"].is_object());
}

#[test]
fn textmate_inner_scopes_use_lang_slug_not_full_scope_name() {
    // Regression: scopes like `comment.line.number-sign.standarx`,
    // NOT `...source.standarx`.
    let g = textmate::grammar_json();
    let comment_name = g["repository"]["comment"]["name"].as_str().unwrap();
    assert!(
        comment_name.ends_with(&format!(".{}", SPEC.lang_slug)),
        "comment scope ends with .{}, got {comment_name}",
        SPEC.lang_slug
    );
    assert!(
        !comment_name.contains("source."),
        "inner scopes must not embed the full scope_name (got {comment_name})"
    );
}

#[test]
fn textmate_grammar_covers_all_token_categories() {
    let g = textmate::grammar_json();
    let repo = g["repository"].as_object().unwrap();
    for required in [
        "comment",
        "constant",
        "number",
        "string-plain",
        "string-inline-template",
        "string-multiline-template",
        "interpolation",
        "punctuation",
        "identifier",
    ] {
        assert!(repo.contains_key(required), "missing repository.{required}");
    }
}

#[test]
fn language_configuration_has_required_keys() {
    let c = lang_config::config_json();
    assert_eq!(c["comments"]["lineComment"], SPEC.line_comment);
    assert!(c["brackets"].is_array());
    assert!(c["autoClosingPairs"].is_array());
    assert!(c["surroundingPairs"].is_array());
    assert_eq!(c["wordPattern"], SPEC.word_pattern);
}

#[test]
fn language_configuration_brackets_match_spec() {
    let c = lang_config::config_json();
    let brackets: &Vec<Value> = c["brackets"].as_array().unwrap();
    assert_eq!(brackets.len(), SPEC.brackets.len());
    for (got, (open, close)) in brackets.iter().zip(SPEC.brackets) {
        assert_eq!(got[0], *open);
        assert_eq!(got[1], *close);
    }
}
