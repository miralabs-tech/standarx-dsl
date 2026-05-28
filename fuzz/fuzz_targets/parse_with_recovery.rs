//! libFuzzer target on `standarx_dsl::parse_with_recovery`.
//!
//! Contract under test:
//! - Never panics on any `&str`.
//! - Always returns a `File` plus a (possibly empty) error list.
//! - Empty error list ⇔ `parse(src).is_ok()` (agreement with the
//!   fail-fast variant).
//! - Recovery makes forward progress — no infinite loop possible.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(s) = std::str::from_utf8(data) else {
        return;
    };
    let (_file, diags) = standarx_dsl::parse_with_recovery(s);
    // Agreement check: both entry points must agree on success.
    let ok_via_parse = standarx_dsl::parse(s).is_ok();
    let ok_via_recovery = diags.is_empty();
    assert_eq!(
        ok_via_parse, ok_via_recovery,
        "parse() and parse_with_recovery() disagreed on success for {s:?}"
    );
});
