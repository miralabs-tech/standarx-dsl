//! TextMate grammar emitter — produces a `.tmLanguage.json` document
//! that VSCode, JetBrains (via TextMate Bundles), Sublime, and
//! many other editors consume directly.
//!
//! The output is regex-based; semantic highlighting (key validity,
//! ref resolution, type checks) is the LSP's job, not this file's.

use serde_json::{Value, json};

use crate::SPEC;

/// Returns the TextMate grammar as a `serde_json::Value`.
///
/// Callers typically `serde_json::to_string_pretty(&grammar_json())`
/// and write the result to `<lang>.tmLanguage.json`.
pub fn grammar_json() -> Value {
    let scope = SPEC.lang_slug;
    json!({
        "$schema": "https://raw.githubusercontent.com/martinring/tmlanguage/master/tmlanguage.json",
        "name": SPEC.display_name,
        "scopeName": SPEC.scope_name,
        "patterns": [
            { "include": "#comment" },
            { "include": "#constant" },
            { "include": "#number" },
            { "include": "#string-multiline-template" },
            { "include": "#string-inline-template" },
            { "include": "#string-plain" },
            { "include": "#punctuation" },
            { "include": "#identifier" },
        ],
        "repository": {
            "comment": {
                "name": format!("comment.line.number-sign.{scope}"),
                "match": r"#.*$",
            },
            "constant": {
                "name": format!("constant.language.{scope}"),
                "match": r"\b(true|false|null)\b",
            },
            "number": {
                "patterns": [
                    {
                        "name": format!("constant.numeric.float.{scope}"),
                        "match": r"-?\b\d+\.\d+([eE][+-]?\d+)?\b",
                    },
                    {
                        "name": format!("constant.numeric.integer.{scope}"),
                        "match": r"-?\b\d+\b",
                    },
                ],
            },
            "string-plain": {
                "name": format!("string.quoted.double.{scope}"),
                "begin": r#"""#,
                "end": r#"""#,
                "patterns": [
                    {
                        "name": format!("constant.character.escape.{scope}"),
                        "match": r#"\\([\\"`nrt0]|u\{[0-9a-fA-F]{1,6}\})"#,
                    },
                ],
            },
            "string-inline-template": {
                "name": format!("string.quoted.other.template.{scope}"),
                "begin": r"`",
                "end": r"`",
                "patterns": [
                    { "include": "#string-escape" },
                    { "include": "#interpolation" },
                ],
            },
            "string-multiline-template": {
                "name": format!("string.quoted.other.template.multiline.{scope}"),
                "begin": r"```",
                "end": r"```",
                "patterns": [
                    { "include": "#string-escape" },
                    { "include": "#interpolation" },
                ],
            },
            "string-escape": {
                "name": format!("constant.character.escape.{scope}"),
                "match": r#"\\([\\"`$nrt0]|u\{[0-9a-fA-F]{1,6}\})"#,
            },
            "interpolation": {
                "name": format!("meta.interpolation.{scope}"),
                "begin": r"\$\{",
                "beginCaptures": {
                    "0": { "name": format!("punctuation.section.interpolation.begin.{scope}") },
                },
                "end": r"\}",
                "endCaptures": {
                    "0": { "name": format!("punctuation.section.interpolation.end.{scope}") },
                },
                "patterns": [
                    { "include": "#constant" },
                    { "include": "#number" },
                    { "include": "#string-plain" },
                    {
                        "name": format!("variable.other.{scope}"),
                        "match": SPEC.word_pattern,
                    },
                ],
            },
            "punctuation": {
                "patterns": [
                    {
                        "name": format!("punctuation.section.block.{scope}"),
                        "match": r"[{}]",
                    },
                    {
                        "name": format!("punctuation.section.brackets.{scope}"),
                        "match": r"[\[\]]",
                    },
                    {
                        "name": format!("punctuation.separator.comma.{scope}"),
                        "match": r",",
                    },
                    {
                        "name": format!("punctuation.accessor.{scope}"),
                        "match": r"\.",
                    },
                ],
            },
            "identifier": {
                "name": format!("variable.other.{scope}"),
                "match": SPEC.word_pattern,
            },
        },
    })
}
