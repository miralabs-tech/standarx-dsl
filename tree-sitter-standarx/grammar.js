/**
 * Tree-sitter grammar for the standarx DSL.
 *
 * Mirrors the semantics of the canonical Rust parser in
 * `crates/standarx-dsl`. The Rust parser is authoritative — keep
 * this grammar in sync when grammar shape changes there.
 *
 * Coverage:
 * - Top-level statements: `block` (`ident [label] { ... }`) and
 *   `assignment` (`ident value`).
 * - Scalars: int, float, bool, null, plain string, inline template,
 *   multi-line template.
 * - References: `ident (. (ident | "quoted"))*`.
 * - Collections: list `[ value ... ]`, map `{ key value ... }`.
 * - Interpolations `${ expr }` inside templates.
 * - Line comments `# ...`.
 *
 * Design notes:
 * - Map literals only appear at **container value position** (inside
 *   list or inside another map). At statement-value position
 *   (`ident <value>`) the value cannot be a bare map — matching the
 *   Rust parser, where `ident { ... }` is ALWAYS a block, never an
 *   assignment-of-map. The grammar enforces this with two distinct
 *   value rules (`_stmt_value` and `_value`) instead of relying on
 *   precedence games.
 *
 * Source of truth: `crates/standarx-dsl/src/{lexer,parser,ast}.rs`.
 */

module.exports = grammar({
  name: 'standarx',

  extras: $ => [/\s/, $.comment],

  word: $ => $.identifier,

  rules: {
    source_file: $ => repeat($._stmt),

    _stmt: $ => choice($.block, $.assignment),

    block: $ => seq(
      field('kind', $.identifier),
      field('label', optional($.plain_string)),
      '{',
      repeat($._stmt),
      '}'
    ),

    assignment: $ => seq(
      field('key', $.identifier),
      field('value', $._stmt_value)
    ),

    // Value at statement position — bare map excluded because
    // `ident {` is unambiguously a block at this level.
    _stmt_value: $ => choice(
      $.integer,
      $.float,
      $.boolean,
      $.null,
      $.plain_string,
      $.template_inline,
      $.template_multiline,
      $.ref,
      $.list
    ),

    // Value at container position (inside list or map) — includes
    // map. No ambiguity here because the enclosing `[`/`{` token
    // tells the parser we are NOT at statement position.
    _value: $ => choice(
      $.integer,
      $.float,
      $.boolean,
      $.null,
      $.plain_string,
      $.template_inline,
      $.template_multiline,
      $.ref,
      $.list,
      $.map
    ),

    list: $ => seq(
      '[',
      repeat(seq($._value, optional(','))),
      ']'
    ),

    map: $ => seq(
      '{',
      repeat(seq(
        field('key', $.identifier),
        field('value', $._value),
        optional(',')
      )),
      '}'
    ),

    ref: $ => prec.right(1, seq(
      $.identifier,
      repeat1(seq('.', choice($.identifier, $.plain_string)))
    )),

    plain_string: $ => seq(
      '"',
      repeat(choice(
        /[^"\\\n]+/,
        $.escape_sequence
      )),
      '"'
    ),

    template_inline: $ => seq(
      '`',
      repeat(choice(
        /[^`\\$\n]+/,
        '$',
        $.escape_sequence,
        $.interpolation
      )),
      '`'
    ),

    template_multiline: $ => seq(
      '```',
      repeat(choice(
        /[^`\\$]+/,
        /`[^`]/,
        /``[^`]/,
        '$',
        $.escape_sequence,
        $.interpolation
      )),
      '```'
    ),

    interpolation: $ => seq(
      '${',
      $._interp_expr,
      '}'
    ),

    _interp_expr: $ => choice(
      $.integer,
      $.float,
      $.boolean,
      $.null,
      $.plain_string,
      $.ref,
      $.identifier
    ),

    escape_sequence: $ => /\\([\\"'`$nrt0]|u\{[0-9a-fA-F]{1,6}\})/,

    integer: $ => /-?[0-9]+/,
    float: $ => /-?[0-9]+\.[0-9]+([eE][+-]?[0-9]+)?/,

    boolean: $ => choice('true', 'false'),
    null: $ => 'null',

    identifier: $ => /[A-Za-z_][A-Za-z0-9_]*/,

    comment: $ => /#[^\n]*/,
  }
});
