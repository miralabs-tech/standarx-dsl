//! Smoke tests for the LSP backend conversion + diagnostic helpers.
//!
//! We do NOT spin up a real LSP transport here — the tower-lsp
//! server contract is tested upstream. We pin the byte→Position
//! conversion (UTF-16 semantics, line counting) and the Diag→LSP
//! Diagnostic shape, which are the bits we own.

use standarx_dsl::Diag;
use standarx_dsl_lsp::{conversion, diag_to_lsp, DIAGNOSTIC_SOURCE};
use tower_lsp::lsp_types::{DiagnosticSeverity, Position};

#[test]
fn byte_offset_at_start_is_origin() {
    let src = "abc";
    assert_eq!(
        conversion::byte_offset_to_position(src, 0),
        Position { line: 0, character: 0 }
    );
}

#[test]
fn byte_offset_counts_lines() {
    let src = "a\nbb\nccc";
    // After first \n, line=1, character=0.
    assert_eq!(
        conversion::byte_offset_to_position(src, 2),
        Position { line: 1, character: 0 }
    );
    // Inside line 2.
    assert_eq!(
        conversion::byte_offset_to_position(src, 6),
        Position { line: 2, character: 1 }
    );
}

#[test]
fn byte_offset_uses_utf16_code_units() {
    // 'é' is one UTF-16 code unit (BMP) → character: 1 after it.
    let src = "é";
    assert_eq!(
        conversion::byte_offset_to_position(src, src.len()),
        Position { line: 0, character: 1 }
    );
    // '🦀' is U+1F980, outside BMP → 2 UTF-16 code units (surrogate pair).
    let src = "🦀";
    assert_eq!(
        conversion::byte_offset_to_position(src, src.len()),
        Position { line: 0, character: 2 }
    );
}

#[test]
fn byte_offset_clamps_out_of_range() {
    let src = "abc";
    assert_eq!(
        conversion::byte_offset_to_position(src, 999),
        Position { line: 0, character: 3 }
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
    assert_eq!(lsp_diag.range.start, Position { line: 0, character: 0 });
    assert_eq!(lsp_diag.range.end, Position { line: 0, character: 3 });
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
