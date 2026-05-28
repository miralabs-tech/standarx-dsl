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

/// Inverse of [`byte_offset_to_position`]. Walks lines until the
/// target row, then converts UTF-16 code units back to a UTF-8 byte
/// offset within that line. Out-of-range positions clamp to
/// `src.len()`.
pub fn position_to_byte_offset(src: &str, pos: Position) -> usize {
    if pos.line == 0 {
        return utf16_units_to_byte_offset(src, 0, pos.character);
    }
    let mut line: u32 = 0;
    for (i, b) in src.as_bytes().iter().enumerate() {
        if *b == b'\n' {
            line += 1;
            if line == pos.line {
                return utf16_units_to_byte_offset(src, i + 1, pos.character);
            }
        }
    }
    // Past last line — clamp to end.
    src.len()
}

fn utf16_units_to_byte_offset(src: &str, line_start: usize, target_units: u32) -> usize {
    let tail = &src[line_start..];
    let mut units_consumed: u32 = 0;
    for (idx, ch) in tail.char_indices() {
        if units_consumed >= target_units {
            return line_start + idx;
        }
        // Stop at end of line.
        if ch == '\n' {
            return line_start + idx;
        }
        units_consumed = units_consumed.saturating_add(ch.len_utf16() as u32);
    }
    line_start + tail.len()
}
