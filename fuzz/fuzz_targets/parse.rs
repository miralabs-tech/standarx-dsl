//! libFuzzer target on `standarx_dsl::parse`.
//!
//! Contract under test: `parse()` must never panic on any `&str`
//! input. Bytes are filtered through `from_utf8` because the public
//! API takes `&str`; bad UTF-8 is not in-contract.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = standarx_dsl::parse(s);
    }
});
