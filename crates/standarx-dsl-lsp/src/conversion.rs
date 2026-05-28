//! Convert `standarx_dsl::Span` (byte offsets) into LSP `Range`
//! (line + UTF-16 code unit column).
//!
//! LSP positions are 0-indexed lines and 0-indexed *UTF-16 code unit*
//! columns. The DSL source is `&str` (UTF-8 bytes), so we walk
//! line-by-line, counting UTF-16 units inside the target line until
//! we reach the byte offset.

use tower_lsp::lsp_types::{Position, Range};

/// Map a byte offset (inside `src`, may equal `src.len()`) to an LSP
/// `Position`. Out-of-range offsets clamp to the end of the document.
pub fn byte_offset_to_position(src: &str, offset: usize) -> Position {
    let clamped = offset.min(src.len());
    let mut line: u32 = 0;
    let mut line_start_byte: usize = 0;
    for (i, b) in src.as_bytes().iter().enumerate() {
        if i >= clamped {
            break;
        }
        if *b == b'\n' {
            line += 1;
            line_start_byte = i + 1;
        }
    }
    let column_str = &src[line_start_byte..clamped];
    let character: u32 = column_str
        .encode_utf16()
        .count()
        .try_into()
        .unwrap_or(u32::MAX);
    Position { line, character }
}

/// Map a `Range<usize>` byte span to an LSP `Range`.
pub fn span_to_range(src: &str, span: &std::ops::Range<usize>) -> Range {
    let start = byte_offset_to_position(src, span.start);
    let end = byte_offset_to_position(src, span.end);
    Range { start, end }
}
