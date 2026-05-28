# Releasing standarx-dsl

This repo cuts releases by pushing an annotated git tag matching
`v<semver>`. The [`Release`](.github/workflows/release.yml) workflow
then auto-creates a GitHub Release, attaches prebuilt binaries for
Linux / macOS / Windows, and (optionally) publishes to crates.io.

## What gets published

The workspace ships at three independent paces:

| Crate / artefact | Versioning policy |
|---|---|
| `standarx-dsl` | Semver-stable (1.x). Public API frozen; new variants and fields land in minors. |
| `standarx-dsl-grammar` | Pre-1.0 (0.1.x). API may evolve. |
| `standarx-dsl-lsp` | Pre-1.0 (0.1.x). API may evolve. |
| `tree-sitter-standarx` | 0.1.x, moves alongside the Rust parser. |
| Editor grammar files (`*.tmLanguage.json`, `*.language-configuration.json`) | Attached to each GitHub Release for non-Rust consumers. |

A release tag is shared across all crates — every git tag captures
the full workspace state at that moment. Consumers pin to the tag,
not to per-crate versions.

## Manual flow

```bash
# 1. Bump the affected crates' versions in Cargo.toml.
#    standarx-dsl    : crates/standarx-dsl/Cargo.toml
#    standarx-dsl-*  : crates/standarx-dsl-*/Cargo.toml
#    (Only bump what actually changed — versions are decoupled.)

# 2. Update the CHANGELOG entries:
$EDITOR crates/standarx-dsl/CHANGELOG.md
$EDITOR crates/standarx-dsl-grammar/CHANGELOG.md
$EDITOR crates/standarx-dsl-lsp/CHANGELOG.md

# 3. Sanity check — same as the CI gate:
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo doc --no-deps --workspace
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace

# 4. Dry-run publish (catches metadata issues before tagging):
cargo publish --dry-run -p standarx-dsl --allow-dirty
cargo publish --dry-run -p standarx-dsl-grammar --allow-dirty
# -lsp dry-runs only after standarx-dsl is on crates.io.

# 5. Commit, tag, push:
git add -A
git commit -m "release: vX.Y.Z"
git tag vX.Y.Z
git push origin main
git push origin vX.Y.Z
```

The push of the tag triggers `release.yml`:

1. **Create the GitHub Release** with auto-generated notes from
   commits since the previous tag. Marked `prerelease` automatically
   when the tag contains a `-` (e.g. `v1.1.0-rc1`).
2. **Build prebuilt binaries** for each `(os, arch)` matrix entry —
   `standarx-lsp` + `standarx-grammar-gen` packaged with the LICENSE,
   README, and pre-generated editor grammar files.
3. **Attach editor-ready grammar files** separately so a VSCode /
   JetBrains extension developer can download just the JSON without
   the binary archive.
4. **Publish to crates.io** if (and only if) the repo variable
   `PUBLISH_CRATES` is `true` AND the secret
   `CARGO_REGISTRY_TOKEN` is set. Publishes in dependency order
   (`standarx-dsl → -grammar → -lsp`); each step `|| true`s so
   re-running on an already-published tag is a no-op.

## Pre-release sanity (cheat sheet)

These are the same checks CI runs on every PR — pasted here for
copy-paste before tagging:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo test -p standarx-dsl --no-default-features
cargo build --workspace --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace
cargo run -p standarx-dsl-grammar --bin standarx-grammar-gen -- \
  --out-dir crates/standarx-dsl-grammar/dist
git diff --quiet -- crates/standarx-dsl-grammar/dist || \
  echo "dist drifted — commit the regen"
cd tree-sitter-standarx
bun install
bunx tree-sitter generate
bunx tree-sitter test
git diff --quiet -- src || echo "tree-sitter parser drifted — commit the regen"
```

## Pre-release tags

Tags with a hyphen (`v1.1.0-rc1`, `v2.0.0-alpha.1`) are marked as
GitHub pre-releases automatically and *do* publish prebuilt binaries
but *do not* trigger crates.io publish — crates.io rejects prerelease
identifiers in stable channels.

## Yanking a release

If a release ships a critical bug:

```bash
# crates.io:
cargo yank -p standarx-dsl --version X.Y.Z
# GitHub Release: mark as draft / delete via the UI.
# Tag stays — never delete a pushed tag (downstream pins break).
```

The next release cuts a higher version and overrides the yanked one.

## Consumer instructions (cross-reference)

See the top-level [README](README.md) — "Consuming this in your
project" — for the downstream `Cargo.toml` snippets (both crates.io
and `git = "...", tag = "v1.0.0"`).
