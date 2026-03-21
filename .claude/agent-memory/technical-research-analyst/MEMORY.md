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

## Rust TDD Best Practices (researched 2026-03-21)
- See `rust-tdd-research.md` for full findings
- Compiler as "phase zero": type system catches structural bugs, tests catch behavioral bugs
- Hand-written fakes preferred over mockall for core trait abstractions (more realistic, better error injection)
- mockall reserved for interaction verification (call counting, argument checking)
- proptest for algorithmic invariants (viewport calcs, text processing)
- insta for serialization/snapshot stability (settings JSON, IPC payloads)
- rstest (v0.26, 48.8M downloads) for fixtures + parameterized tests
- pretty_assertions as global drop-in for assert_eq!
- cargo nextest mandatory for wgpu projects (EGL requires one GPU context per process)
- Nextest 35% faster than cargo test in CI benchmarks (depot.dev)
- Key limitation: nextest does not support doctests; run `cargo test --doc` separately
- Rust 2024 edition: doctests compiled into single binary (much faster)
- Async testing: use `#[tokio::test]`, never nest runtimes, use `start_paused = true` for time control
- Jorge Ortiz-Fuentes blog series (jorgeortiz.dev) is comprehensive Rust testing reference

## AI-Assisted TDD for Coding Agents (researched 2026-03-21)
- See `ai-tdd-research.md` for full findings and source list
- AI agents default to implementation-first; structural enforcement (not just prompting) required
- Red-phase failure verification is the most commonly skipped step
- Context pollution: same agent writing tests+impl couples tests to implementation details
- TDAD paper (arxiv 2603.17973): contextual info > procedural instructions for smaller models
- Superpowers framework (99k+ stars): enforces 7-phase workflow including TDD
- Key anti-patterns: excessive mocking, tautological assertions, testing impl details, AI "cheating" on tests
- Luminos SDD/SUBTASKS.md already well-aligned; add explicit Red-phase verification checkbox

## TypeScript TDD Best Practices (researched 2026-03-21)
- See `typescript-tdd-research.md` for full findings
- Vitest Context7 ID: `/vitest-dev/vitest` (benchmark 88.26, High reputation)
- Zustand Context7 ID: `/pmndrs/zustand` (benchmark 80.77, High reputation)
- Tauri v2 `shouldMockEvents` option requires v2.7.0+; without it, `listen()` throws in tests
- Zustand official testing pattern: `__mocks__/zustand.ts` with `storeResetFns` Set
- vitest-axe REQUIRES jsdom (NOT happy-dom); alternative: `@chialab/vitest-axe`
- zod-fast-check v0.9.0 bridges Zod v3 to fast-check; WARNING: low-probability refinements fail
- Vitest watch mode is default; `test.only()` for focused TDD cycles
- Test naming: `{module}_{behavior}_{condition}` matches Rust convention in CLAUDE.md

## Research Methodology Notes
- Third-party "guide" sites often contain AI-generated content with fabricated CLI flags. Always cross-reference with official docs.rs, GitHub README, or crates.io pages.
- For Cargo profile settings, the official Cargo Book is the single source of truth.
- DeepWiki (deepwiki.com) provides useful source-code-level analysis of open source projects.
