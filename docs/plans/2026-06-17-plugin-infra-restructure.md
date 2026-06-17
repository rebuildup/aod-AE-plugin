# Plugin Infrastructure Restructure Plan

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Restructure aod-AE-plugin into a publishable library (`aod-ae-utils`) on crates.io, removing all existing plugins and templates.

**Architecture:** Keep only `crates/utils` (renamed to `aod-ae-utils`) as a publishable library. Remove `plugins/`, `templates/`, and plugin-specific build infrastructure. Users create independent Rust projects depending on `aod-ae-utils` + `after-effects`.

**Tech Stack:** Rust, Cargo, crates.io publishing

---

## Task 1: Remove existing plugins and templates

**Files:**
- Delete: `plugins/` (entire directory)
- Delete: `templates/` (entire directory)
- Delete: `dist/` (built .aex files)

**Step 1: Delete plugins directory**

```bash
rm -rf plugins/
```

**Step 2: Delete templates directory**

```bash
rm -rf templates/
```

**Step 3: Delete dist directory**

```bash
rm -rf dist/
```

**Step 4: Verify deletion**

```bash
ls
# Should NOT contain: plugins/, templates/, dist/
```

---

## Task 2: Rename utils crate to aod-ae-utils

**Files:**
- Rename: `crates/utils/` → `crates/aod-ae-utils/`
- Modify: `crates/aod-ae-utils/Cargo.toml`

**Step 1: Rename directory**

```bash
mv crates/utils crates/aod-ae-utils
```

**Step 2: Update Cargo.toml**

Replace `crates/aod-ae-utils/Cargo.toml` with publishable metadata:

```toml
[package]
name = "aod-ae-utils"
version = "0.1.0"
edition = "2021"
description = "Utility library for Adobe After Effects plugin development in Rust"
license = "MPL-2.0"
repository = "https://github.com/Aodaruma/aod-AE-plugin"
keywords = ["after-effects", "adobe", "plugin", "video", "effects"]
categories = ["multimedia::video"]

[dependencies]
after-effects = "0.3"
```

---

## Task 3: Update workspace Cargo.toml

**Files:**
- Modify: `Cargo.toml` (root)

**Step 1: Simplify workspace**

```toml
[workspace]
members = ["crates/aod-ae-utils"]
resolver = "2"
```

---

## Task 4: Remove plugin-specific files

**Files:**
- Delete: `AdobePlugin.just` (plugin build rules, not needed for library)
- Delete: `.cargo/config.toml` (cargo aliases for plugin building)
- Keep: `Justfile` (update for library)
- Keep: `AGENTS.md` (update)
- Keep: `README.md` (rewrite)
- Keep: `LICENSE`
- Keep: `rustfmt.toml`

**Step 1: Delete AdobePlugin.just**

```bash
rm AdobePlugin.just
```

**Step 2: Delete .cargo/config.toml**

```bash
rm .cargo/config.toml
```

---

## Task 5: Update Justfile for library

**Files:**
- Modify: `Justfile`

**Step 1: Replace Justfile content**

```just
set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

build:
    cargo build -p aod-ae-utils

release:
    cargo build -p aod-ae-utils --release

check:
    cargo fmt --all -- --check
    cargo clippy --workspace
    cargo test

publish:
    cargo publish -p aod-ae-utils
```

---

## Task 6: Rewrite README.md

**Files:**
- Modify: `README.md`

**Step 1: Replace README content**

Write a concise README explaining:
- What this library is
- How to use it (add dependency to Cargo.toml)
- What it provides (pixel conversion, color utilities)
- Example usage

---

## Task 7: Update AGENTS.md

**Files:**
- Modify: `AGENTS.md`

**Step 1: Simplify AGENTS.md**

Remove plugin-specific rules. Keep:
- Library development guidelines
- Publishing workflow
- Code style rules

---

## Task 8: Verify build

**Step 1: Build**

```bash
cargo build -p aod-ae-utils
```

**Step 2: Run checks**

```bash
cargo fmt --all -- --check
cargo clippy --workspace
cargo test
```

**Step 3: Verify publish dry-run**

```bash
cargo publish --dry-run -p aod-ae-utils
```
