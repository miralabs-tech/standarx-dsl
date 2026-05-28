//! VSCode-style `language-configuration.json` emitter — describes
//! brackets, comments, auto-pairs, and the word boundary for the
//! standarx DSL.
//!
//! Consumed by VSCode, Cursor, and most LSP-aware editors. Other
//! editors (JetBrains, Helix) tend to inline these as native config
//! but can map this file 1:1.

use serde_json::{json, Value};

use crate::SPEC;

/// Returns the `language-configuration.json` document as a
/// `serde_json::Value`. Callers serialize with
/// `serde_json::to_string_pretty`.
pub fn config_json() -> Value {
    let brackets: Vec<Value> = SPEC.brackets.iter().map(|(o, c)| json!([o, c])).collect();
    let auto_closing: Vec<Value> = SPEC
        .auto_closing_pairs
        .iter()
        .map(|(o, c)| json!({ "open": o, "close": c }))
        .collect();
    let surrounding: Vec<Value> = SPEC
        .surrounding_pairs
        .iter()
        .map(|(o, c)| json!([o, c]))
        .collect();

    json!({
        "comments": {
            "lineComment": SPEC.line_comment,
        },
        "brackets": brackets,
        "autoClosingPairs": auto_closing,
        "surroundingPairs": surrounding,
        "wordPattern": SPEC.word_pattern,
    })
}
