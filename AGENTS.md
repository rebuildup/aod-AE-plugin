# aod-ae-utils Development Guide

## Overview

- This repository is a utility library for Adobe After Effects plugin development in Rust.
- Main crate: `crates/aod-ae-utils` — published to crates.io.

## CRITICAL: Documentation First

**Before modifying ANY code in this library, read `docs/INDEX.md`.**

The docs contain:
- Correct API usage and parameter domains
- AE SDK quirks not visible in source code
- Algorithm background for graphics processing
- Optimization patterns specific to this library

Source code alone is NOT sufficient for correct changes.

## Development

- Source: `crates/aod-ae-utils/src/lib.rs`
- Build: `just build` or `cargo build -p aod-ae-utils`
- Test: `cargo test`
- Check: `just check` (fmt + clippy + test)

## Publishing

- `cargo publish --dry-run -p aod-ae-utils` (test)
- `cargo publish -p aod-ae-utils` (production)
- Version: update `version` in `crates/aod-ae-utils/Cargo.toml`

## Rules

- Pass `cargo fmt` and `cargo clippy`
- Add doc comments to all public APIs
- Add tests for new functionality
- Update `docs/INDEX.md` if adding new modules

## Commit Convention

- Format: `TAG: Summary` (English, single line)
- TAG: `ADD` / `REFACTOR` / `CHORE` / `FIX` / `DOCS`
