//! Smoke tests for the LSP backend conversion + diagnostic helpers.
//!
//! We do NOT spin up a real LSP transport here — the tower-lsp
//! server contract is tested upstream. We pin the byte→Position
//! conversion (UTF-16 semantics, line counting) and the Diag→LSP
//! Diagnostic shape, which are the bits we own.

use standarx_dsl::{Diag, File};
use standarx_dsl_lsp::{collect_diagnostics, conversion, diag_to_lsp, Schema, DIAGNOSTIC_SOURCE};
use tower_lsp::lsp_types::{
    CompletionItem, DiagnosticSeverity, Hover, HoverContents, Location, MarkupContent, MarkupKind,
    Position, Range, Url,
};

#[test]
fn byte_offset_at_start_is_origin() {
    let src = "abc";
    assert_eq!(
        conversion::byte_offset_to_position(src, 0),
        Position {
            line: 0,
            character: 0
        }
    );
}

#[test]
fn byte_offset_counts_lines() {
    let src = "a\nbb\nccc";
    // After first \n, line=1, character=0.
    assert_eq!(
        conversion::byte_offset_to_position(src, 2),
        Position {
            line: 1,
            character: 0
        }
    );
    // Inside line 2.
    assert_eq!(
        conversion::byte_offset_to_position(src, 6),
        Position {
            line: 2,
            character: 1
        }
    );
}

#[test]
fn byte_offset_uses_utf16_code_units() {
    // 'é' is one UTF-16 code unit (BMP) → character: 1 after it.
    let src = "é";
    assert_eq!(
        conversion::byte_offset_to_position(src, src.len()),
        Position {
            line: 0,
            character: 1
        }
    );
    // '🦀' is U+1F980, outside BMP → 2 UTF-16 code units (surrogate pair).
    let src = "🦀";
    assert_eq!(
        conversion::byte_offset_to_position(src, src.len()),
        Position {
            line: 0,
            character: 2
        }
    );
}

#[test]
fn byte_offset_clamps_out_of_range() {
    let src = "abc";
    assert_eq!(
        conversion::byte_offset_to_position(src, 999),
        Position {
            line: 0,
            character: 3
        }
    );
}

#[test]
fn diag_to_lsp_carries_severity_and_source() {
    let diag = Diag::parse(0..3, "boom");
    let src = "abc";
    let lsp_diag = diag_to_lsp(src, &diag);
    assert_eq!(lsp_diag.severity, Some(DiagnosticSeverity::ERROR));
    assert_eq!(lsp_diag.source.as_deref(), Some(DIAGNOSTIC_SOURCE));
    assert!(lsp_diag.message.contains("boom"));
    assert_eq!(
        lsp_diag.range.start,
        Position {
            line: 0,
            character: 0
        }
    );
    assert_eq!(
        lsp_diag.range.end,
        Position {
            line: 0,
            character: 3
        }
    );
}

#[test]
fn diag_to_lsp_maps_warning() {
    let diag = Diag::schema_warn(0..1, "deprecated key");
    let lsp_diag = diag_to_lsp("x", &diag);
    assert_eq!(lsp_diag.severity, Some(DiagnosticSeverity::WARNING));
}

#[test]
fn parse_error_round_trip_through_diag_to_lsp() {
    // Real parser-emitted diagnostic — verifies the integration end-to-end
    // for the byte-range → LSP-range path on actual lexer output.
    let src = "{ broken }";
    let diag = standarx_dsl::parse(src).expect_err("expected parse error");
    let lsp_diag = diag_to_lsp(src, &diag);
    assert_eq!(lsp_diag.severity, Some(DiagnosticSeverity::ERROR));
    assert!(lsp_diag.range.start.line == 0);
}

struct CountStmtsSchema;
impl Schema for CountStmtsSchema {
    fn validate(&self, file: &File, _src: &str) -> Vec<Diag> {
        // Toy schema: warns when the file has > 1 stmt. Just here to
        // exercise the collect_diagnostics path with a real schema.
        if file.stmts.len() > 1 {
            vec![Diag::schema_warn(0..0, "more than one stmt")]
        } else {
            Vec::new()
        }
    }
}

#[test]
fn collect_diagnostics_runs_no_schema_path() {
    // Valid source, no schemas → empty diag list.
    assert!(collect_diagnostics("k \"v\"", &[]).is_empty());
}

#[test]
fn collect_diagnostics_surfaces_parser_error_even_without_schemas() {
    let out = collect_diagnostics("{ broken }", &[]);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].severity, Some(DiagnosticSeverity::ERROR));
}

#[test]
fn collect_diagnostics_runs_schemas_on_successful_parse() {
    let schemas: Vec<Box<dyn Schema>> = vec![Box::new(CountStmtsSchema)];
    // Two stmts → schema fires.
    let out = collect_diagnostics("k \"v\"\nk2 \"v2\"", &schemas);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].severity, Some(DiagnosticSeverity::WARNING));
    assert!(out[0].message.contains("more than one stmt"));
}

#[test]
fn collect_diagnostics_skips_schemas_on_parse_failure() {
    // Parser error short-circuits schemas — only the parse error is reported.
    let schemas: Vec<Box<dyn Schema>> = vec![Box::new(CountStmtsSchema)];
    let out = collect_diagnostics("{ broken }", &schemas);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].severity, Some(DiagnosticSeverity::ERROR));
}

#[test]
fn position_round_trip_single_line() {
    let src = "abc";
    for byte in 0..=src.len() {
        let pos = conversion::byte_offset_to_position(src, byte);
        let back = conversion::position_to_byte_offset(src, pos);
        assert_eq!(back, byte, "round-trip failed at byte {byte}");
    }
}

#[test]
fn position_round_trip_multi_line() {
    let src = "abc\ndef\nghi";
    for byte in 0..=src.len() {
        let pos = conversion::byte_offset_to_position(src, byte);
        let back = conversion::position_to_byte_offset(src, pos);
        assert_eq!(back, byte, "round-trip failed at byte {byte}");
    }
}

#[test]
fn position_round_trip_multibyte_chars() {
    // café 🦀 — Latin-1 supplement + supplementary-plane emoji.
    let src = "café 🦀";
    for byte in 0..=src.len() {
        if !src.is_char_boundary(byte) {
            continue;
        }
        let pos = conversion::byte_offset_to_position(src, byte);
        let back = conversion::position_to_byte_offset(src, pos);
        assert_eq!(back, byte, "round-trip failed at byte {byte}");
    }
}

#[test]
fn position_to_byte_offset_clamps_past_end() {
    let src = "abc";
    let past = Position {
        line: 999,
        character: 999,
    };
    assert_eq!(conversion::position_to_byte_offset(src, past), src.len());
}

struct ToySchema;

impl Schema for ToySchema {
    fn validate(&self, _file: &File, _src: &str) -> Vec<Diag> {
        Vec::new()
    }

    fn completion(&self, _file: &File, _src: &str, _offset: usize) -> Vec<CompletionItem> {
        vec![
            CompletionItem::new_simple("project".into(), "block kind".into()),
            CompletionItem::new_simple("task".into(), "block kind".into()),
        ]
    }

    fn hover(&self, _file: &File, _src: &str, _offset: usize) -> Option<Hover> {
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: "**toy**".into(),
            }),
            range: None,
        })
    }

    fn goto_definition(&self, _file: &File, _src: &str, _offset: usize) -> Option<Location> {
        Some(Location {
            uri: Url::parse("file:///stub.sxb").unwrap(),
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 0,
                },
            },
        })
    }
}

#[test]
fn schema_completion_default_impl_returns_empty() {
    // CountStmtsSchema only implements `validate`; completion etc.
    // come from the default impls.
    let schema = CountStmtsSchema;
    let file = standarx_dsl::parse("k 1").expect("parse");
    assert!(schema.completion(&file, "k 1", 0).is_empty());
    assert!(schema.hover(&file, "k 1", 0).is_none());
    assert!(schema.goto_definition(&file, "k 1", 0).is_none());
}

#[test]
fn schema_completion_returns_items_when_overridden() {
    let schema = ToySchema;
    let file = standarx_dsl::parse("k 1").expect("parse");
    let items = schema.completion(&file, "k 1", 0);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].label, "project");
}

#[test]
fn schema_hover_returns_some_when_overridden() {
    let schema = ToySchema;
    let file = standarx_dsl::parse("k 1").expect("parse");
    let h = schema.hover(&file, "k 1", 0).expect("hover");
    let HoverContents::Markup(m) = h.contents else {
        panic!("expected markup hover");
    };
    assert_eq!(m.value, "**toy**");
}

#[test]
fn schema_goto_definition_returns_some_when_overridden() {
    let schema = ToySchema;
    let file = standarx_dsl::parse("k 1").expect("parse");
    let loc = schema.goto_definition(&file, "k 1", 0).expect("goto");
    assert_eq!(loc.uri.scheme(), "file");
}
