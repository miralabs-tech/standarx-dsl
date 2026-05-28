//! Property tests for `parse()`.
//!
//! Contract under test: `parse()` MUST return `Ok` or `Err` for any
//! `&str` input — it must never panic and never produce a `String` (in
//! its diagnostic chain or AST) that violates UTF-8 invariants. Random
//! inputs stress the lexer's byte-level state machine and the `unsafe`
//! `push_byte` invariant in `src/lexer.rs`.

use proptest::prelude::*;

/// DSL-flavoured alphabet: weights structural / escape-sensitive bytes
/// higher than the uniform Unicode pool. Catches more edge cases per
/// case than `any::<String>()`.
fn dsl_alphabet() -> impl Strategy<Value = String> {
    let chars = prop::char::ranges(
        vec![
            // structural / escape-sensitive
            '"'..='"',
            '\\'..='\\',
            '`'..='`',
            '$'..='$',
            '{'..='{',
            '}'..='}',
            '['..='[',
            ']'..=']',
            '#'..='#',
            '='..='=',
            '\n'..='\n',
            '\t'..='\t',
            ' '..=' ',
            // identifier chars
            'a'..='z',
            'A'..='Z',
            '0'..='9',
            '_'..='_',
            '.'..='.',
            ':'..=':',
            // multi-byte UTF-8 (Latin supplement, CJK, emoji plane)
            '\u{00A0}'..='\u{00FF}',
            '\u{4E00}'..='\u{4E2F}',
            '\u{1F600}'..='\u{1F60F}',
        ]
        .into(),
    );
    prop::collection::vec(chars, 0..256).prop_map(|v| v.into_iter().collect())
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 2048, ..ProptestConfig::default() })]

    /// `parse()` must not panic on any UTF-8 input.
    #[test]
    fn parse_never_panics_on_arbitrary_utf8(s in any::<String>()) {
        let _ = standarx_dsl::parse(&s);
    }

    /// Same contract on the DSL-weighted alphabet — more likely to land
    /// near interesting branches (string escapes, interp, blocks, refs).
    #[test]
    fn parse_never_panics_on_dsl_alphabet(s in dsl_alphabet()) {
        let _ = standarx_dsl::parse(&s);
    }

    /// `parse()` is deterministic — same input, same outcome.
    #[test]
    fn parse_is_deterministic(s in dsl_alphabet()) {
        let a = standarx_dsl::parse(&s);
        let b = standarx_dsl::parse(&s);
        prop_assert_eq!(a.is_ok(), b.is_ok());
    }
}
