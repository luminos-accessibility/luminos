# Technical Research Analyst Memory

## Rust Build Configuration Research

### SBOM Tools for Rust
- `cargo-cyclonedx` (v0.5.7, Apache-2.0) is the primary CycloneDX SBOM generator for Rust
- `cargo-sbom` (v0.10.0, by psastras/sbom-rs) is an alternative supporting both SPDX and CycloneDX
- WARNING: sbomgenerator.com fabricates CLI flags for cargo-cyclonedx (e.g., `--output`, `--dev-dependencies`). Always verify against docs.rs or GitHub README.
- Authoritative sources: docs.rs/crate/cargo-cyclonedx, github.com/CycloneDX/cyclonedx-rust-cargo

### Cargo Profile Settings (verified sources)
- Official Cargo profiles docs: doc.rust-lang.org/cargo/reference/profiles.html
- Rust Performance Book (nnethercote): nnethercote.github.io/perf-book/build-configuration.html
- min-sized-rust guide: github.com/johnthagen/min-sized-rust
- `lto = true` is equivalent to `lto = "fat"` (confirmed in Cargo source code)
- `strip = true` is equivalent to `strip = "symbols"` (confirmed in Cargo docs)
- Since Rust 1.77, stdlib debuginfo is implicitly stripped in release builds (except MSVC)

### Reproducible Builds in Rust
- Tracking issue: rust-lang/rust#129080
- trim-paths RFC 3127: nightly-only as of early 2026, stabilization PR #147611 in progress
- Yocto Project reproducibility report (Feb 2025): Rust 1.82, patches for 1.83-1.84 in review
- Best approach: Docker with pinned toolchain + CARGO_INCREMENTAL=0 + --remap-path-prefix

## Tauri 2.0 Bundler (verified 2026-03-17)
- See `tauri-bundler-research.md` for full findings
- Native targets enum: deb, rpm, appimage, nsis, msi, app, dmg (from schema 2.10.3)
- Flatpak/Snap NOT native targets; require external tooling
- Updater plugin: `tauri-plugin-updater` / `@tauri-apps/plugin-updater`
- Key docs: v2.tauri.app/reference/config/, schema.tauri.app/config/2
- Context7 IDs: `/tauri-apps/tauri-docs` (docs), `/websites/v2_tauri_app` (site)
- CAUTION: web results often mix Tauri v1 and v2; filter by URL prefix

## Research Methodology Notes
- Third-party "guide" sites often contain AI-generated content with fabricated CLI flags. Always cross-reference with official docs.rs, GitHub README, or crates.io pages.
- For Cargo profile settings, the official Cargo Book is the single source of truth.
- DeepWiki (deepwiki.com) provides useful source-code-level analysis of open source projects.
