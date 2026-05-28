# Changelog

All notable changes to `standarx-dsl-grammar` are documented here.
Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versioning follows [SemVer](https://semver.org/spec/v2.0.0.html).

## [0.1.0] — 2026-05-28

Initial release alongside `standarx-dsl` 1.0.0.

- `GrammarSpec` — Rust source of truth describing the standarx DSL
  surface syntax: keywords, brackets, comment style, string fences,
  identifier shape, TextMate scope name and lang slug.
- `textmate::grammar_json()` — emits `*.tmLanguage.json`
  consumable by VSCode, JetBrains (TextMate Bundles), Sublime,
  Helix.
- `lang_config::config_json()` — emits `*.language-configuration.json`
  with brackets, comments, auto-closing pairs, word boundary.
- Binary `standarx-grammar-gen` (`--out-dir DIR` or `--stdout {textmate|config}`)
  writes the two JSON documents.
- Pre-generated `dist/standarx.tmLanguage.json` and
  `dist/standarx.language-configuration.json` versioned in the
  repo — downstream editor extensions can copy them as-is.
- CI grammar-drift job (`.github/workflows/ci.yml`) regenerates the
  dist files and asserts no diff vs the committed copies.
