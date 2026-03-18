# 08 -- Build and Distribution

**Status:** DRAFT v1.1 (post audit review)
**Date:** 2026-03-17
**Audience:** Engineers, DevOps, AI agents, release managers, contributors
**Source Documents:** [Product Strategy](../PRODUCT_STRATEGY.md) (v1.3, Sections 7, 8, 9), [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) (FINAL, Sections 4, 5), [System Architecture](./01-system-architecture.md) (Sections 7, 10, 11), [Platform Abstraction](./02-platform-abstraction.md) (Section 5), [TTS Pipeline](./04-tts-pipeline.md) (Section 8), [Control Panel](./05-control-panel.md) (Section 2.2), [Cross-Cutting Concerns](./06-cross-cutting-concerns.md) (Sections 3, 4), [Testing Strategy](./07-testing-strategy.md) (Sections 4.9, 5.3, 8)

---

## 1. Overview

### 1.1 Purpose

This document defines how the Luminos source tree is compiled, packaged, signed, and distributed to users on all target platforms. It is the engineering specification for everything between "the code compiles" and "the user has a working installation."

This document answers: **How does source code become a signed, distributable package that users can install on Linux, macOS, OpenBSD, and Windows?**

### 1.2 Scope

This document covers:
- Cargo workspace configuration (root manifest, workspace dependencies, crate layout)
- Cargo features and conditional compilation strategy for cross-platform builds
- Build profiles (development, release, distribution, CI) with optimization rationale
- Frontend build pipeline (TypeScript/React via Vite, Tauri integration)
- espeak-ng binary and data file bundling strategy per platform
- Voice model distribution (on-demand download, offline bundled installer, verification)
- Platform packaging: .deb, .rpm, AppImage, Flatpak, snap (Linux); .dmg (macOS); OpenBSD ports; .msi and NSIS (Windows)
- Code signing per platform (GPG, Apple notarization, signify, Authenticode)
- Auto-update mechanism (Tauri updater plugin, update server, policy)
- Release engineering (versioning, changelog, GitHub Releases, cadence)
- SBOM generation and reproducible build strategy

This document does NOT cover:
- CI/CD pipeline architecture and quality gates (see [07 -- Testing Strategy](./07-testing-strategy.md))
- License compliance policy and cargo-deny configuration (see [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 4)
- Supply chain security threat model (see [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 3.6)
- Per-subsystem compilation or test details (see docs 02-05)

### 1.3 Phase Attribution

Build and distribution capabilities are introduced incrementally:

| Phase | Build and Distribution Milestone |
|-------|--------------------------------|
| **Phase 0** | Cargo workspace compiles. CI release pipeline produces signed GitHub Releases with pre-built binaries (doc-07 Stage 8). Tauri bundler produces .deb, .rpm, AppImage for Linux X11.* espeak-ng bundled in AppImage; declared as dependency in .deb/.rpm. Voice model on-demand download. cargo-deny and cargo audit in CI. |
| **Phase 1** | APT/DNF repository hosting. Flatpak and snap packages. SBOM generation (CycloneDX). Auto-updater for AppImage. Reproducible build infrastructure (Docker-based). |

*\* The [Product Strategy](../PRODUCT_STRATEGY.md) Section 7.2 lists "Linux packages" as Phase 1 (P1). However, Phase 0 requires "automated releases" (Section 7.1) which necessitates at least basic packaging. The .deb, .rpm, and AppImage formats are produced by the Tauri bundler with minimal configuration and are pulled forward to Phase 0 to enable the CI release pipeline. Repository hosting (APT/DNF) and non-Tauri-native formats (Flatpak, snap) remain Phase 1.*
| **Phase 2** | macOS .dmg with Apple Developer ID signing and notarization. macOS auto-updater. |
| **Phase 3** | OpenBSD port/package with signify signing. Offline installer variant (bundled voice model). |
| **Phase 4** | Windows .msi (WiX) and NSIS installer with Authenticode signing. Windows auto-updater. GPO/MDM-compatible silent install. Enterprise deployment documentation. |

### 1.4 Relationship to Other Documents

```
01-system-architecture.md   -- Defines workspace structure (§7), binary structure (§11),
    |                          build integrity requirements (§10.4)
    v
02-platform-abstraction.md  -- Defines conditional compilation patterns (§5)
    |
    v
06-cross-cutting-concerns   -- Licensing compliance (§4), supply chain security (§3),
    |                          signing key management (§3.7)
    v
08-build-and-distribution   -- THIS DOCUMENT: how the above becomes distributable packages
(this)  |
    v
07-testing-strategy.md      -- CI pipeline (§4), release stage (§4.9), quality gates (§5.3)
    |
    v
09-implementation-roadmap   -- Milestones that trigger releases (planned)
```

The build and distribution pipeline is constrained by the workspace structure (doc-01), conditional compilation patterns (doc-02), and licensing/security policies (doc-06). It feeds into the CI release stage (doc-07 Stage 8) and is triggered by milestones in the implementation roadmap (doc-09).

---

## 2. Cargo Workspace Configuration

### 2.1 Workspace Root Cargo.toml

The workspace root `Cargo.toml` defines the five crate members, shared metadata, and centralized dependency versions. It does not produce a binary itself -- `luminos-app` is the sole binary crate.

```toml
# luminos/Cargo.toml

[workspace]
members = [
    "crates/luminos-core",
    "crates/luminos-platform",
    "crates/luminos-gpu",
    "crates/luminos-tts",
    "crates/luminos-app",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "GPL-3.0-only"
repository = "https://github.com/luminos-app/luminos"
homepage = "https://luminos.dev"
authors = ["Luminos Contributors"]
rust-version = "1.85"

# Build profiles are defined below (see Section 4)
```

**Key decisions:**
- **No explicit `resolver` field** -- Edition 2024 defaults to dependency resolver version 3, which includes MSRV-aware dependency resolution. Resolver 3 is the correct default and does not need to be specified explicitly.
- **`edition = "2024"`** -- Rust 2024 edition for all crates, requiring rustc 1.85+.
- **`license = "GPL-3.0-only"`** -- SPDX identifier per [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 4.1. Not the deprecated `GPL-3.0`.
- **`rust-version = "1.85"`** -- Minimum Supported Rust Version (MSRV). Enforced by `cargo msrv` checks in CI.

### 2.2 Workspace Dependencies

All shared dependencies are declared once in the workspace root and inherited by crates via `{ workspace = true }`. This prevents version skew across crates and simplifies dependency updates.

```toml
# luminos/Cargo.toml (continued)

[workspace.dependencies]
# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# Concurrency and state
arc-swap = "1"
crossbeam-channel = "0.5"

# Logging
log = "0.4"
env_logger = "0.11"

# Error handling
thiserror = "2"

# Platform abstraction (internal crates)
luminos-platform = { path = "crates/luminos-platform" }
luminos-core = { path = "crates/luminos-core" }
luminos-gpu = { path = "crates/luminos-gpu" }
luminos-tts = { path = "crates/luminos-tts" }

# GPU rendering
wgpu = "28.0"
winit = "0.30"

# Screen capture
xcap = "0.9"

# TTS
sherpa-rs = "0.6"
cpal = "0.17"

# Clipboard
arboard = "3"

# Application framework
tauri = { version = "2", features = ["tray-icon"] }
tauri-build = "2"
tauri-specta = { version = "2", features = ["derive", "typescript"] }
specta-typescript = "0.0.9"

# X11 protocol (Linux + OpenBSD)
x11rb = { version = "0.13", features = ["randr", "shm"] }

# Linux accessibility
atspi = "0.22"

# Input monitoring
rdev = "0.5"
```

**Version pinning strategy:** `Cargo.lock` is committed to the repository. Workspace dependency versions use minimum-compatible ranges (e.g., `"1"` not `"=1.0.85"`) for flexibility, while `Cargo.lock` ensures exact reproducibility. The `Cargo.lock` is the source of truth for CI and release builds.

### 2.3 Crate-Level Configuration

Each crate inherits workspace metadata and declares only its specific dependencies:

```toml
# crates/luminos-core/Cargo.toml

[package]
name = "luminos-core"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true

[dependencies]
luminos-platform = { workspace = true }
luminos-tts = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
toml = { workspace = true }
arc-swap = { workspace = true }
crossbeam-channel = { workspace = true }
log = { workspace = true }
thiserror = { workspace = true }
arboard = { workspace = true }

[dev-dependencies]
# Test utilities -- no workspace declaration needed for test-only deps
```

**Rules:**
- Every crate uses `version.workspace = true` for synchronized versioning.
- No crate declares a dependency version directly if it exists in `[workspace.dependencies]`.
- `[dev-dependencies]` may declare crate-local test utilities not shared across the workspace.
- Internal crate dependencies (e.g., `luminos-platform`) use `{ workspace = true }` which resolves to the `path` declaration in the workspace root.

---

## 3. Cargo Features and Conditional Compilation

### 3.1 Platform Feature Strategy

Luminos uses two complementary mechanisms for cross-platform builds:

1. **`#[cfg(target_os = "...")]` conditional compilation** -- Selects platform-specific backend code at compile time. This is the primary mechanism for the six platform traits (see [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 5).

2. **Cargo features** -- Control optional capabilities that are not platform-dependent (test utilities, profiling instrumentation, CI-specific behavior). Features are NOT used for platform selection -- `target_os` handles that automatically.

**Why features are not used for platforms:** Platform selection via `target_os` is automatic and cannot be misconfigured. Feature-based platform selection would require users to remember `--features linux_x11` when building on Linux X11, creating a footgun. The Rust convention is to use `cfg(target_os)` for platform dispatch and features for optional capabilities.

### 3.2 Feature Definitions Per Crate

#### luminos-platform

```toml
[features]
default = []
wayland = ["ashpd"]    # Enables PipeWire/XDG Portal capture dependency
xshm = ["x11rb/shm"]  # Enables XShm shared-memory capture optimization
test_utils = []        # Exports mock implementations for all six traits
ci_platform_tests = [] # Enables integration tests that require real platform APIs
```

The `wayland` feature controls whether the `ashpd` crate (Wayland XDG Portal D-Bus bindings) is compiled in. It does not gate module compilation -- the `linux_wayland` module is always compiled on Linux, but its backend returns an error if the `wayland` feature is not enabled. The `xshm` feature enables the x11rb XShm shared-memory extension for optimized X11 capture (Phase 1). Feature definitions are canonical in [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 5.2.

The `test_utils` feature is used by other crates' test suites to import mock backends. The `ci_platform_tests` feature gates tests that interact with real X11 servers, accessibility APIs, or audio devices -- these only run on CI runners with the appropriate environment (see [07 -- Testing Strategy](./07-testing-strategy.md) Section 4.7).

#### luminos-gpu

```toml
[features]
default = []
test_utils = []        # Exports mock renderer and test frame generators
update_refs = []       # Regenerate shader reference images instead of comparing
profiling = []         # Tracy profiler instrumentation spans
```

The `update_refs` feature is used when a shader changes and reference images need to be regenerated (see [07 -- Testing Strategy](./07-testing-strategy.md) Section 7.3). The `profiling` feature enables Tracy integration spans around GPU operations (see [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 2.4).

#### luminos-tts

```toml
[features]
default = []
test_utils = []        # Exports mock TTS engine and test audio generators
```

#### luminos-core

```toml
[features]
default = []
test_utils = [
    "luminos-platform/test_utils",
    "luminos-gpu/test_utils",
    "luminos-tts/test_utils",
]  # Transitively enables all mock backends
```

#### luminos-app

```toml
[features]
default = []
integration_tests = [
    "luminos-core/test_utils",
]  # Enables IPC integration tests against mock backends
ci_platform_tests = [
    "luminos-platform/ci_platform_tests",
]  # Enables real-platform CI tests
profiling = [
    "luminos-gpu/profiling",
]  # Tracy profiler for the full application
```

### 3.3 Conditional Compilation Rules

These rules govern all `#[cfg]` usage across the codebase:

1. **Platform dispatch lives in `luminos-platform` only.** No other crate uses `#[cfg(target_os)]` directly. All platform differences are abstracted behind the six traits. Exception: `luminos-app/src/main.rs` may use `cfg(target_os)` for platform-specific application lifecycle setup (e.g., macOS activation policy).

2. **Backend modules use `cfg` at the module level.** Each platform backend is declared with a `#[cfg]` attribute on the `mod` statement in `luminos-platform/src/lib.rs` (canonical definition in [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 5.2):

```rust
// luminos-platform/src/lib.rs
#[cfg(target_os = "linux")]
pub mod linux_x11;

#[cfg(target_os = "linux")]
pub mod linux_wayland;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "openbsd")]
pub mod openbsd;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(any(test, feature = "test_utils"))]
pub mod mock;
```

**Note on Linux sub-backends:** Both X11 and Wayland backend modules are compiled on Linux. The `wayland` Cargo feature (defined in doc-02 Section 5.2) controls whether the `ashpd` dependency (for PipeWire/XDG Portal capture) is included, not whether the module is compiled. At runtime, the active backend is selected by detecting the current display server. There is no `x11` Cargo feature -- X11 support is unconditional on Linux and OpenBSD.

3. **Test code uses `cfg(test)` or `cfg(feature = "test_utils")`.** Mock implementations, test generators, and fixtures are gated behind these attributes. `#[cfg(test)]` for module-private test code; `feature = "test_utils"` for publicly exported mock types.

4. **Debug-only code uses `cfg(debug_assertions)`.** This includes tauri-specta TypeScript binding generation (see [05 -- Control Panel](./05-control-panel.md) Section 2.2) and verbose diagnostic logging.

5. **No `cfg` in WGSL shaders.** GPU shaders are platform-independent by design (wgpu translates to the native GPU API). Shader variants are handled by pipeline constants or uniform buffers, not compile-time conditionals.

---

## 4. Build Profiles

### 4.1 Development Profile

The default `dev` profile prioritizes fast compilation and rich debugging:

```toml
# luminos/Cargo.toml

[profile.dev]
opt-level = 0          # No optimization -- fastest compile
debug = true           # Full debug info for debuggers
incremental = true     # Incremental compilation enabled
# debug-assertions, overflow-checks enabled by default
```

**Typical dev build time:** 10-30 seconds (incremental) after initial compilation. Use `cargo check` for sub-second type-checking feedback during development.

### 4.2 Release Profile

The standard `release` profile optimizes for runtime speed. Used for local performance testing and CI benchmarks:

```toml
[profile.release]
opt-level = 3          # Maximum speed optimization
debug = false          # No debug info
lto = "thin"           # Cross-crate thin LTO (faster than fat, good optimization)
codegen-units = 16     # Default parallelism for reasonable compile times
strip = "debuginfo"    # Strip debug info but keep symbol names for backtraces
```

**Use case:** `cargo build --release` for local performance profiling, CI benchmark runs, and the E2E smoke test binary (see [07 -- Testing Strategy](./07-testing-strategy.md) Section 4.6).

### 4.3 Distribution Profile

The `dist` profile is a custom profile that maximizes binary size reduction for shipping. This is the profile used for all release artifacts:

```toml
[profile.dist]
inherits = "release"
opt-level = "z"        # Optimize for binary size (try "s" if "z" is larger)
lto = "fat"            # Full cross-crate link-time optimization
codegen-units = 1      # Single codegen unit for maximum optimization
panic = "abort"        # Remove unwind tables (saves ~100-300KB)
strip = "symbols"      # Remove all symbols and debug info
```

**Build command:** `cargo build --profile dist -p luminos-app`

**Trade-offs:**
- `opt-level = "z"` may produce slightly larger binaries than `"s"` in some cases. Both should be benchmarked during Phase 0; the smaller result is adopted.
- `lto = "fat"` with `codegen-units = 1` increases link time significantly (2-5 minutes on CI). This is acceptable for release builds only.
- `panic = "abort"` prevents `std::panic::catch_unwind()` from working. No Luminos code uses `catch_unwind()`; if a dependency requires it, this will manifest as a compile error and must be evaluated.
- `strip = "symbols"` removes all symbol names, making crash backtraces opaque. This is acceptable for distributed binaries because: (a) users report bugs via reproduction steps, not stack traces; (b) debug symbols are preserved in CI artifacts for crash analysis (see Section 4.4).

**Debug symbol archive:** CI retains an unstripped copy of the binary as a build artifact alongside the stripped release artifact. This enables post-mortem debugging of crash reports by mapping addresses back to source via the unstripped binary.

### 4.4 CI Profile

CI test builds use the default `dev` or `release` profile depending on the stage (see [07 -- Testing Strategy](./07-testing-strategy.md) Section 4). No custom CI profile is needed because:
- Stages 1-5 (lint, test, clippy, docs, benchmarks) use `cargo check`, `cargo test`, or `cargo build --release`.
- Stage 8 (release) uses `cargo build --profile dist`.

CI sets the following environment variables for all builds:

```bash
CARGO_INCREMENTAL=0               # Disable incremental compilation for determinism
RUSTFLAGS="--deny warnings"       # Treat warnings as errors in CI
```

### 4.5 Binary Size Budget

| Component | Budget | Notes |
|-----------|--------|-------|
| `luminos` binary (stripped, dist profile) | < 50MB | Product Strategy target ([Product Strategy](../PRODUCT_STRATEGY.md) Section 8.6) |
| Tauri webview assets (compressed) | < 5MB | React control panel, bundled by Tauri |
| WGSL shader files | < 100KB | Compiled at runtime by wgpu |
| espeak-ng binary + data | ~ 6MB | Phonemizer subprocess + language data |
| **Total package (excluding voice models)** | **< 62MB** | Sum of above components |

**CI enforcement** (from [07 -- Testing Strategy](./07-testing-strategy.md) Section 4.6):
- **Warn** at 50MB binary size (the product target).
- **Fail** at 60MB binary size (hard ceiling with margin for the binary alone).

Doc-07 Stage 5 (benchmarks) checks binary size using the `release` profile as a proxy during PR validation. The release stage (Stage 8) checks the actual distribution binary built with the `dist` profile. The same thresholds apply to both; the `dist` profile produces a smaller binary due to `opt-level = "z"` and `lto = "fat"`, so if the `release` binary passes, the `dist` binary will also pass.

```bash
# CI binary size check (release stage, Stage 8)
cargo build --release -p luminos-app   # or --profile dist for final artifacts
SIZE=$(stat -f%z target/release/luminos 2>/dev/null || stat -c%s target/release/luminos)
echo "Binary size: $SIZE bytes"
if [ "$SIZE" -gt 52428800 ]; then echo "WARN: binary > 50MB"; fi
if [ "$SIZE" -gt 62914560 ]; then echo "FAIL: binary > 60MB" && exit 1; fi
```

**Architecture note:** All binary size targets and CI checks are for x86_64 (amd64) builds. ARM64 (aarch64) Linux builds are not planned for initial phases but may be added if demand warrants. macOS produces both aarch64 (Apple Silicon) and x64 (Intel) binaries from Phase 2. ARM64 AppImages can only be built on ARM64 hardware (no cross-compilation).

**Size reduction techniques applied:**
1. `opt-level = "z"` -- LLVM size-optimized code generation
2. `lto = "fat"` -- Whole-program dead code elimination (10-20% reduction)
3. `codegen-units = 1` -- Enables additional intra-crate optimizations
4. `strip = "symbols"` -- Removes symbol table and debug info
5. `panic = "abort"` -- Removes unwind tables
6. Shader files remain as source WGSL (not precompiled SPIR-V), loaded at runtime

If the binary exceeds 50MB during development, investigate with `cargo bloat --profile dist -p luminos-app` to identify the largest contributors by crate and function.

---

## 5. Frontend Build Pipeline

### 5.1 Package Manager and Toolchain

| Tool | Version | Purpose |
|------|---------|---------|
| Node.js | 20 LTS (or 22 LTS) | JavaScript runtime for build tools |
| pnpm | 9.x | Package manager (fast, disk-efficient) |
| Vite | 6.x | Frontend bundler and dev server |
| TypeScript | 5.x | Type-checked JavaScript |
| React | 19.x | UI framework |

**Why pnpm over npm/yarn:** pnpm's content-addressable storage avoids duplicate downloads across projects and AI agent workspaces. Its strict dependency resolution catches phantom dependencies that npm allows. The `pnpm-lock.yaml` lockfile is committed to the repository.

### 5.2 Build Process

The frontend build is orchestrated by Tauri's build system. When `cargo tauri build` runs, it:

1. Invokes the `beforeBuildCommand` defined in `tauri.conf.json` to build the frontend.
2. Collects the built frontend assets from the `frontendDist` directory.
3. Embeds the assets into the Rust binary via Tauri's asset bundling.

```json
// tauri.conf.json (relevant fields)
{
  "build": {
    "beforeDevCommand": "pnpm dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "pnpm build",
    "frontendDist": "../ui/dist"
  }
}
```

**Frontend build commands:**

```bash
# Development (hot reload via Vite dev server)
cd ui && pnpm dev          # Starts Vite dev server at localhost:1420

# Production build
cd ui && pnpm build        # Produces optimized assets in ui/dist/
```

**Vite production build output:**

```
ui/dist/
  index.html               # Entry point
  assets/
    index-[hash].js        # Bundled, minified JavaScript (~200-500KB gzipped)
    index-[hash].css       # Bundled, minified CSS
    *.woff2                # Font files (if any)
```

Vite's tree-shaking eliminates unused code from React, Zustand, React Router, and Zod. The total compressed frontend payload should be well under 5MB.

### 5.3 Specta Type Generation

TypeScript type bindings are generated from Rust IPC types by `tauri-specta` (see [05 -- Control Panel](./05-control-panel.md) Section 2.2 for the full type generation architecture). The generation happens in two modes:

**Development mode:** When `luminos-app` is compiled with `debug_assertions` enabled (the default for `cargo build` without `--release`), the `Builder::export()` call writes `ui/src/ipc/bindings.ts`. This file contains typed wrappers for all IPC commands and event payload types.

**Release mode:** Type generation is skipped. The last-generated `bindings.ts` (committed to the repository) is used as-is.

**Build order dependency:** The frontend build (`pnpm build`) depends on `bindings.ts` existing. During initial setup or after adding new IPC commands:

```bash
# 1. Build the Rust backend in debug mode to generate bindings
cargo build -p luminos-app

# 2. Then build the frontend
cd ui && pnpm build
```

The `bindings.ts` file is committed to the repository so that frontend development can proceed without a Rust build environment. CI validates that the committed bindings match the current Rust types by running a generation check:

```bash
# CI check: bindings are up to date
cargo build -p luminos-app   # Regenerates bindings.ts
git diff --exit-code ui/src/ipc/bindings.ts || {
    echo "FAIL: bindings.ts is out of date. Run cargo build -p luminos-app and commit."
    exit 1
}
```

---

## 6. espeak-ng Bundling Strategy

### 6.1 Platform-Specific Strategy

espeak-ng is GPL-3.0 licensed (compatible with Luminos's GPLv3 license) and is used as a subprocess for phonemization. The bundling strategy differs by platform based on package manager conventions:

| Platform | Strategy | Rationale |
|----------|----------|-----------|
| **Linux (.deb, .rpm)** | **Declare as package dependency** | espeak-ng is available in all major Linux distro repositories. The `.deb` declares `espeak-ng` in its `Depends` field; `.rpm` declares it in `Requires`. The system package manager installs it automatically. |
| **Linux (AppImage, Flatpak)** | **Bundle the binary and data** | Self-contained formats cannot rely on system packages. espeak-ng binary and data files are included in the package. |
| **Linux (Snap)** | **Stage package** | Snap's `stage-packages` mechanism pulls espeak-ng from the Ubuntu archive into the snap. |
| **macOS** | **Bundle the binary and data** | espeak-ng is not available via system package manager. A pre-built espeak-ng binary (compiled for the target architecture) and data files are bundled inside the .app bundle. |
| **OpenBSD** | **Declare as port dependency** | espeak-ng is available in OpenBSD packages. The port's `RUN_DEPENDS` includes `espeak-ng`. |
| **Windows** | **Bundle the binary and data** | espeak-ng is not available via a standard Windows package manager. The espeak-ng binaries and data from the official MSI distribution are extracted and bundled. |

**Crash isolation:** Regardless of bundling strategy, espeak-ng always runs as a subprocess (see [04 -- TTS Pipeline](./04-tts-pipeline.md) Section 3 and [01 -- System Architecture](./01-system-architecture.md) Section 10.2). The binary path is resolved at runtime: bundled path first, then system `PATH` fallback.

### 6.2 Binary and Data File Layout

**Bundled layout** (AppImage, Flatpak, macOS, Windows):

```
<app_root>/
  bin/
    espeak-ng              # (or espeak-ng.exe on Windows)
  share/
    espeak-ng-data/        # ~5MB
      lang/                # Language-specific rules
      voices/              # Voice definitions
      phondata             # Phoneme data
      phonindex            # Phoneme index
      phontab              # Phoneme table
      intonations          # Intonation patterns
```

**Resolution order** at runtime (consistent with [04 -- TTS Pipeline](./04-tts-pipeline.md) Section 5 per-platform expected locations):
1. Bundled path: `<app_resource_dir>/bin/espeak-ng`
2. System `PATH`: `espeak-ng` (for .deb, .rpm, OpenBSD where it is a system dependency)

If neither path produces a working espeak-ng binary, the TTS pipeline degrades to platform-native TTS fallback (see [04 -- TTS Pipeline](./04-tts-pipeline.md) Section 11). The control panel displays an espeak-ng availability warning via the `espeak_status_changed` event (see [05 -- Control Panel](./05-control-panel.md) Section 2.4).

### 6.3 Build-Time espeak-ng Validation

CI validates espeak-ng availability on every platform runner:

```bash
# CI step: verify espeak-ng is functional
espeak-ng --version
echo "Hello world" | espeak-ng -q --ipa
```

For bundled platforms (macOS, Windows), the CI pipeline validates that the bundled espeak-ng binary executes correctly within the package structure before producing release artifacts. For dependency platforms (Linux .deb/.rpm), the CI runner installs espeak-ng via the system package manager before testing (see [07 -- Testing Strategy](./07-testing-strategy.md) Section 4.7 for per-platform CI setup).

---

## 7. Voice Model Distribution

### 7.1 Distribution Strategy

Voice model files are large (80-327MB per model) and make the base installer impractical to distribute at the <50MB binary target. Luminos uses a two-tier distribution strategy:

| Variant | Contents | Size | Target User |
|---------|----------|------|-------------|
| **Standard installer** | Application binary + espeak-ng + shaders. No voice models. | ~55-62MB | Most users (models downloaded on first TTS use) |
| **Full installer** | Standard + Kokoro-82M q8 model bundled | ~150MB | Offline environments, institutional deployment (Phase 3) |

The standard installer is the default for all package formats. The full installer is an additional artifact produced alongside it, available on GitHub Releases for users who need offline TTS from first launch.

### 7.2 Model Storage Layout

Voice models are stored in the platform-appropriate user data directory:

| Platform | Model Directory |
|----------|----------------|
| Linux | `$XDG_DATA_HOME/luminos/models/` (defaults to `~/.local/share/luminos/models/`) |
| macOS | `~/Library/Application Support/dev.luminos.app/models/` |
| OpenBSD | `~/.local/share/luminos/models/` |
| Windows | `%APPDATA%\luminos\models\` |

**Directory structure:**

```
models/
  kokoro/
    kokoro-v1.0-q8.onnx        # ~92MB (default)
    kokoro-v1.0-q4.onnx        # ~80MB (lightweight alternative)
    kokoro-v1.0-fp16.onnx       # ~163MB (quality variant)
    voices.json                 # Speaker ID → display name mapping
  piper/
    en-US-amy-medium.onnx       # ~63MB (example Piper voice)
    en-US-amy-medium.onnx.json  # Model configuration
```

### 7.3 Model Download Protocol

When the user first activates TTS (or selects a voice whose model is not installed), Luminos initiates a model download:

1. **Discovery:** The application fetches a model manifest from a known URL (initially hosted on GitHub Releases, later on a dedicated CDN if traffic warrants). The manifest lists available models with their URLs, SHA-256 checksums, and sizes.

2. **Progress:** Download progress is reported to the control panel via the `voice_model_loading` event (see [05 -- Control Panel](./05-control-panel.md) Section 2.4). The UI displays a progress bar with the download stage (`Downloading`, `Loading`, `Complete`, `Failed`).

3. **Verification:** After download, the file's SHA-256 checksum is validated against the manifest. A mismatch triggers a re-download (once) or an error.

4. **Loading:** The verified model file is loaded into the sherpa-onnx runtime. Model loading progress transitions through the `voice_model_loading` event stages.

**Model manifest format:**

```json
{
  "manifest_version": 1,
  "models": [
    {
      "id": "kokoro-v1.0-q8",
      "name": "Kokoro 82M (q8, recommended)",
      "engine": "Kokoro",
      "url": "https://github.com/luminos-app/luminos/releases/download/models-v1/kokoro-v1.0-q8.onnx",
      "sha256": "abc123...",
      "size_bytes": 96468992,
      "license": "Apache-2.0"
    }
  ]
}
```

**Network failure handling:** If the download fails (network error, checksum mismatch), the TTS pipeline falls back to platform-native TTS if available, or displays an actionable error in the control panel. The user can retry the download from the voice selection UI. No TTS operation blocks the magnification pipeline -- magnification remains fully functional without models.

### 7.4 Offline Installation

For institutional deployments and air-gapped environments (Phase 3):

1. **Full installer artifact:** A separate installer file (e.g., `luminos-full-0.1.0-amd64.deb`) bundles the Kokoro q8 model pre-installed in the correct location. This artifact is ~150MB and is produced alongside the standard installer in the CI release pipeline.

2. **Manual model placement:** Users or IT administrators can manually place model files in the model directory (Section 7.2). Luminos discovers models by scanning the directory at startup -- no download required if files are present with valid checksums.

3. **GPO/MDM pre-seeding (Windows, Phase 4):** Enterprise deployment can pre-populate the model directory via Group Policy or MDM file distribution before or alongside the application install.

---

## 8. Platform Packaging

### 8.1 Tauri Bundler Overview

Tauri 2.0's built-in bundler (`cargo tauri build`) natively produces seven package formats:

| Format | Platform | Tauri Native | Notes |
|--------|----------|:------------:|-------|
| .deb | Linux | Yes | Debian/Ubuntu package |
| .rpm | Linux | Yes | Fedora/RHEL package |
| AppImage | Linux | Yes | Self-contained, portable |
| Flatpak | Linux | **No** | Requires external `flatpak-builder` |
| Snap | Linux | **No** | Requires external `snapcraft` (documented workflow using .deb repackage) |
| .dmg | macOS | Yes | Disk image containing .app bundle |
| .msi | Windows | Yes | WiX-based MSI installer |
| NSIS | Windows | Yes | NSIS-based .exe installer |

**Build commands:**

The Tauri CLI (`cargo tauri build`) always uses the Cargo `release` profile internally. To use the `dist` profile for optimized release artifacts, the build is split into two steps:

```bash
# Step 1: Build the Rust binary with the dist profile
cargo build --profile dist -p luminos-app

# Step 2: Bundle the pre-built binary into platform packages
cargo tauri build --bundles deb,appimage
```

When `cargo tauri build` detects a pre-existing release binary, it uses it for bundling. Alternatively, the Tauri `release` profile can be overridden by setting the `dist` profile settings directly in `[profile.release]` for the release stage (see Section 4.3 for why a separate `dist` profile is preferred during development).

```bash
# Build specific formats
cargo tauri build --bundles deb,appimage     # Linux
cargo tauri build --bundles dmg              # macOS
cargo tauri build --bundles msi,nsis         # Windows
```

**Important:** Linux packages can only be built on Linux. macOS packages can only be built on macOS. MSI requires Windows. NSIS cross-compilation from Linux/macOS is considered highly experimental by Tauri and may not work reliably -- CI builds NSIS on a Windows runner. The CI matrix (see [07 -- Testing Strategy](./07-testing-strategy.md) Section 4.7) builds each platform's packages on its native runner.

**Platform-specific configuration** uses Tauri's platform config override files which are merged with the base config via JSON Merge Patch (RFC 7396):

```
luminos-app/
  tauri.conf.json              # Base configuration (all platforms)
  tauri.linux.conf.json        # Linux-specific overrides
  tauri.macos.conf.json        # macOS-specific overrides
  tauri.windows.conf.json      # Windows-specific overrides
```

### 8.2 Linux: Debian/Ubuntu (.deb)

**Tauri configuration** (`tauri.linux.conf.json` excerpt):

```json
{
  "bundle": {
    "linux": {
      "deb": {
        "depends": [
          "libwebkit2gtk-4.1-0",
          "libgtk-3-0",
          "espeak-ng"
        ],
        "section": "utils",
        "priority": "optional",
        "desktopTemplate": "assets/linux/luminos.desktop"
      }
    }
  }
}
```

**Key points:**
- `libwebkit2gtk-4.1-0` is required by Tauri's webview (control panel).
- `espeak-ng` is declared as a dependency -- the system package manager installs it automatically.
- The `.desktop` file ensures Luminos appears in application launchers with the correct icon and category (`Accessibility`).
- Output: `target/release/bundle/deb/luminos_0.1.0_amd64.deb`

**APT repository hosting (Phase 1):** A signed APT repository is hosted on GitHub Pages or a dedicated server, enabling `apt update && apt install luminos`. The repository uses GPG-signed Release files (see Section 9.2). Configuration:

```bash
# User adds the repository
echo "deb [signed-by=/usr/share/keyrings/luminos-archive-keyring.gpg] https://apt.luminos.dev stable main" \
    | sudo tee /etc/apt/sources.list.d/luminos.list
```

### 8.3 Linux: Fedora/RHEL (.rpm)

**Tauri configuration** (`tauri.linux.conf.json` excerpt):

```json
{
  "bundle": {
    "linux": {
      "rpm": {
        "depends": [
          "webkit2gtk4.1",
          "gtk3",
          "espeak-ng"
        ],
        "release": "1"
      }
    }
  }
}
```

**Key points:**
- RPM package names differ from Debian (e.g., `webkit2gtk4.1` vs `libwebkit2gtk-4.1-0`).
- The RPM license is inherited from the `bundle.license` field in the base `tauri.conf.json` (which is set to `"GPL-3.0-only"`). There is no `license` field in `RpmConfig`.
- Output: `target/release/bundle/rpm/luminos-0.1.0-1.x86_64.rpm`
- RPM signing is handled by Tauri's built-in GPG support (see Section 9.2).

**DNF repository hosting (Phase 1):** A signed RPM repository (using `createrepo` and GPG) is hosted alongside the APT repository.

### 8.4 Linux: AppImage

**Tauri configuration** (`tauri.linux.conf.json` excerpt):

```json
{
  "bundle": {
    "linux": {
      "appimage": {
        "bundleMediaFramework": false,
        "files": {
          "/usr/bin/espeak-ng": "assets/linux/espeak-ng",
          "/usr/lib/x86_64-linux-gnu/espeak-ng-data": "assets/linux/espeak-ng-data/"
        }
      }
    }
  }
}
```

**Key points:**
- AppImage is self-contained -- it bundles all dependencies including system libraries, espeak-ng, and espeak-ng data.
- `bundleMediaFramework = false` because Luminos uses `cpal` for audio output directly, not GStreamer.
- The AppImage is significantly larger than the .deb (~70-90MB vs ~6-10MB) because it includes shared libraries.
- Output: `target/release/bundle/appimage/luminos_0.1.0_amd64.AppImage`
- No installation required: `chmod a+x luminos*.AppImage && ./luminos*.AppImage`
- GPG signing via environment variables (see Section 9.2).
- The auto-updater (Section 10) supports AppImage updates via `tauri-plugin-updater`.

### 8.5 Linux: Flatpak

Flatpak is not a native Tauri bundler target. A `flatpak-builder` manifest is maintained separately:

```yaml
# flatpak/dev.luminos.app.yml
app-id: dev.luminos.app
runtime: org.gnome.Platform
runtime-version: '46'
sdk: org.gnome.Sdk
command: luminos

finish-args:
  - --share=ipc                          # X11 shared memory
  - --socket=x11                         # X11 display access
  - --socket=wayland                     # Wayland display access
  - --socket=pulseaudio                  # Audio output
  - --device=dri                         # GPU access (wgpu)
  - --talk-name=org.a11y.Bus             # AT-SPI2 accessibility bus
  - --filesystem=xdg-config/luminos:create  # Config directory
  - --filesystem=xdg-data/luminos:create    # Data directory (models)
  - --share=network                      # Model download

modules:
  - name: espeak-ng
    buildsystem: cmake
    sources:
      - type: git
        url: https://github.com/espeak-ng/espeak-ng.git
        tag: 1.51

  - name: luminos
    buildsystem: simple
    build-commands:
      - install -Dm755 luminos /app/bin/luminos
      - install -Dm644 luminos.desktop /app/share/applications/dev.luminos.app.desktop
      - install -Dm644 luminos.svg /app/share/icons/hicolor/scalable/apps/dev.luminos.app.svg
    sources:
      - type: archive
        url: https://github.com/luminos-app/luminos/releases/download/v0.1.0/luminos-0.1.0-linux-x86_64.tar.gz
        sha256: (computed at release time)
```

**Key points:**
- The Flatpak builds espeak-ng from source within the sandbox (ensuring GPL-3.0 compliance with source availability).
- GPU access requires `--device=dri`. Vulkan and OpenGL are available via the GNOME SDK.
- Flatpak publication on Flathub is a Phase 1 goal.
- The Flatpak manifest is maintained in the repository at `flatpak/dev.luminos.app.yml`.

### 8.6 Linux: Snap

Snap packaging uses Tauri's documented external workflow: build a .deb first, then repackage:

```yaml
# snap/snapcraft.yaml
name: luminos
version: '0.1.0'
summary: Cross-platform screen magnification + TTS accessibility suite
description: |
  GPU-accelerated screen magnification with neural text-to-speech
  for low-vision users.
grade: stable
confinement: strict
base: core22

apps:
  luminos:
    command: usr/bin/luminos
    desktop: usr/share/applications/luminos.desktop
    extensions: [gnome]
    plugs:
      - x11
      - wayland
      - opengl
      - audio-playback
      - network            # Model downloads

parts:
  luminos:
    plugin: dump
    source: target/release/bundle/deb/luminos_0.1.0_amd64.deb
    source-type: deb
    stage-packages:
      - espeak-ng
      - libwebkit2gtk-4.1-0
```

**Key points:**
- The snap repackages the .deb output from the Tauri bundler.
- `confinement: strict` enforces sandboxing; the `plugs` list grants specific capabilities.
- Snap Store publication is a Phase 1 goal.

### 8.7 macOS: .dmg

**Tauri configuration** (`tauri.macos.conf.json` excerpt):

```json
{
  "bundle": {
    "macOS": {
      "hardenedRuntime": true,
      "minimumSystemVersion": "13.0",
      "frameworks": [],
      "dmg": {
        "windowSize": { "width": 600, "height": 400 },
        "appPosition": { "x": 180, "y": 170 },
        "applicationFolderPosition": { "x": 420, "y": 170 }
      }
    }
  }
}
```

**Key points:**
- `minimumSystemVersion = "13.0"` (macOS Ventura) -- ScreenCaptureKit is available from macOS 12.3, but Tauri 2.0 requires macOS 10.13+. Setting 13.0 as the minimum ensures modern API availability and aligns with the macOS Phase 2 timeline.
- `hardenedRuntime = true` is required for notarization.
- espeak-ng is bundled inside the .app bundle at `Luminos.app/Contents/Resources/bin/espeak-ng` with data at `Luminos.app/Contents/Resources/share/espeak-ng-data/`.
- The .dmg provides the standard macOS drag-to-Applications-folder installation experience.
- Output: `target/release/bundle/dmg/Luminos_0.1.0_aarch64.dmg` (Apple Silicon) and `Luminos_0.1.0_x64.dmg` (Intel).
- Code signing and notarization are handled automatically by the Tauri bundler when environment variables are set (see Section 9.3).

### 8.8 OpenBSD: Ports

OpenBSD packaging uses the ports system. There is no Tauri bundler involvement -- the port builds Luminos from source:

```makefile
# ports/accessibility/luminos/Makefile (simplified)
COMMENT =       screen magnification and TTS accessibility suite

V =             0.1.0
DISTNAME =      luminos-${V}

CATEGORIES =    accessibility x11

HOMEPAGE =      https://luminos.dev/

MAINTAINER =    Luminos Contributors <ports@luminos.dev>

# GPLv3
PERMIT_PACKAGE =        Yes

WANTLIB += ...

MODULES =       devel/cargo

BUILD_DEPENDS = devel/cargo \
                www/node
RUN_DEPENDS =   audio/espeak-ng

CONFIGURE_STYLE = cargo

do-install:
        ${INSTALL_PROGRAM} ${WRKSRC}/target/release/luminos \
                ${PREFIX}/bin/luminos
        ${INSTALL_DATA_DIR} ${PREFIX}/share/luminos/
        # Install shaders, assets, webview resources
```

**Key points:**
- OpenBSD ports build from source, ensuring the binary matches the system's libc and X11 libraries.
- espeak-ng is declared as `RUN_DEPENDS` (installed via `pkg_add` automatically).
- There are no GitHub-hosted OpenBSD CI runners. Until a self-hosted runner is provisioned (Phase 1), OpenBSD builds are validated manually (see [07 -- Testing Strategy](./07-testing-strategy.md) Section 4.7).
- The port is submitted to the OpenBSD ports tree in Phase 3 when the platform backend is implemented.
- Package signing uses `signify` (see Section 9.4).

### 8.9 Windows: MSI and NSIS

Luminos produces both MSI (for enterprise GPO deployment) and NSIS (for consumer installation) packages on Windows.

**Tauri configuration** (`tauri.windows.conf.json` excerpt):

```json
{
  "bundle": {
    "windows": {
      "allowDowngrades": true,
      "webviewInstallMode": {
        "type": "downloadBootstrapper"
      },
      "nsis": {
        "installMode": "currentUser",
        "displayLanguageSelector": false
      },
      "wix": {
        "fipsCompliant": false
      }
    }
  }
}
```

**Key points:**
- **WebView2 runtime:** Tauri requires Microsoft Edge WebView2 for the control panel webview. `downloadBootstrapper` automatically downloads and installs WebView2 if not present. Windows 11 ships with WebView2 pre-installed; Windows 10 may not.
- **MSI (WiX):** GPO-compatible for enterprise deployment. Supports silent installation (`msiexec /i luminos.msi /quiet`). Ideal for institutional deployments (Phase 4).
- **NSIS:** User-friendly installer with progress UI. `installMode: "currentUser"` avoids requiring administrator elevation for personal installations.
- espeak-ng is bundled in both installers. The espeak-ng binary and data from the official GitHub releases MSI are extracted and included (see [07 -- Testing Strategy](./07-testing-strategy.md) Section 4.7 for the espeak-ng Windows install source).
- Output: `target/release/bundle/msi/luminos_0.1.0_x64_en-US.msi` and `target/release/bundle/nsis/luminos_0.1.0_x64-setup.exe`
- Code signing via Authenticode (see Section 9.5).

---

## 9. Code Signing

### 9.1 Signing Strategy Overview

Luminos uses two independent signing layers:

| Layer | Purpose | Algorithm | Scope |
|-------|---------|-----------|-------|
| **Platform code signing** | OS trust verification (Gatekeeper, SmartScreen, package managers) | GPG (Linux/OpenBSD), Apple codesign (macOS), Authenticode (Windows) | All release artifacts |
| **Tauri updater signing** | Auto-update integrity verification | Ed25519 | AppImage, macOS .app, Windows installer updates |

Both layers are required for release artifacts. Platform signing satisfies OS-level trust; updater signing secures the in-app update channel. The keys are independent -- compromising one does not compromise the other.

### 9.2 Linux: GPG

**AppImage signing** is handled by the Tauri bundler via environment variables:

```bash
# CI environment variables for AppImage signing
SIGN=1                                    # Enable AppImage signing
SIGN_KEY=<GPG_KEY_ID>                     # GPG key ID to use
APPIMAGETOOL_SIGN_PASSPHRASE=<passphrase> # Key passphrase
APPIMAGETOOL_FORCE_SIGN=1                 # Fail build if signing fails
```

Users verify with: `gpg --verify luminos_0.1.0_amd64.AppImage.sig`

**RPM signing** is handled by the Tauri bundler via environment variables:

```bash
# CI environment variables for RPM signing
TAURI_SIGNING_RPM_KEY=<ascii_armored_private_key>   # Private GPG key content
TAURI_SIGNING_RPM_KEY_PASSPHRASE=<passphrase>       # Key passphrase (optional)
```

Users verify with: `rpm --checksig luminos-0.1.0-1.x86_64.rpm`

**Debian package signing** is NOT natively supported by the Tauri bundler. It is performed as a post-build step using `dpkg-sig`:

```bash
# Post-build .deb signing in CI
dpkg-sig --sign builder -k <GPG_KEY_ID> target/release/bundle/deb/luminos_0.1.0_amd64.deb
```

Users verify with: `dpkg-sig --verify luminos_0.1.0_amd64.deb`

**APT/DNF repository signing:** Repository metadata (Release files for APT, repodata for DNF) are signed with the same GPG key. Users import the public key once when adding the repository.

### 9.3 macOS: Apple Developer ID and Notarization

The Tauri bundler automatically signs and notarizes macOS builds when environment variables are set. Notarization uses `xcrun notarytool` (the current Apple tool, replacing the deprecated `altool`).

**CI environment variables:**

```bash
# Code signing
APPLE_SIGNING_IDENTITY="Developer ID Application: Luminos Foundation (TEAMID)"
APPLE_CERTIFICATE=<base64_encoded_p12>      # .p12 certificate for CI
APPLE_CERTIFICATE_PASSWORD=<password>

# Notarization (App Store Connect API method, recommended for CI)
APPLE_API_ISSUER=<issuer_id>
APPLE_API_KEY=<key_id>
APPLE_API_KEY_PATH=<path_to_p8_file>
```

**Requirements:**
- A paid Apple Developer account ($99/year) is required for Developer ID signing and notarization.
- `hardenedRuntime = true` is set in `tauri.macos.conf.json` (required for notarization).
- Notarization submits the .app to Apple's servers for malware scanning. This adds 2-10 minutes to the CI pipeline.

**Local development:** Use `cargo tauri build --no-sign` to bypass code signing during local testing. Unsigned builds trigger Gatekeeper warnings but run after manual approval.

### 9.4 OpenBSD: Signify

OpenBSD uses `signify` for package signing:

```bash
# Generate a signing key pair (done once, stored securely)
signify -G -p luminos.pub -s luminos.sec -c "Luminos release signing key"

# Sign a package
signify -S -s luminos.sec -m luminos-0.1.0.tgz -x luminos-0.1.0.tgz.sig
```

Users verify with: `signify -V -p luminos.pub -m luminos-0.1.0.tgz -x luminos-0.1.0.tgz.sig`

OpenBSD packages submitted to the official ports tree are signed by the OpenBSD project's own keys through the standard ports infrastructure.

### 9.5 Windows: Authenticode

The Tauri bundler automatically invokes `signtool.exe` from the Windows SDK when configured:

**Tauri configuration** (`tauri.windows.conf.json`):

```json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "<SHA-1_THUMBPRINT>",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com",
      "tsp": false
    }
  }
}
```

**CI workflow variables (user-defined, not Tauri-recognized):**

Unlike the macOS `APPLE_*` variables which Tauri reads directly, Windows certificate provisioning requires a CI workflow step that imports the certificate into the Windows certificate store before `cargo tauri build` runs:

```bash
# GitHub Actions step: import certificate into Windows cert store
WINDOWS_CERTIFICATE=<base64_encoded_pfx>      # CI secret (user-defined)
WINDOWS_CERTIFICATE_PASSWORD=<password>        # CI secret (user-defined)

# Decode and import in the CI workflow
echo "$WINDOWS_CERTIFICATE" | base64 -d > cert.pfx
certutil -f -p "$WINDOWS_CERTIFICATE_PASSWORD" -importpfx cert.pfx
```

Tauri's bundler then finds the certificate via the `certificateThumbprint` in `tauri.windows.conf.json` and invokes `signtool.exe` to sign all executables.

**Certificate strategy:**
- **Phase 4 (initial):** Standard code signing certificate. Windows SmartScreen may show warnings until the binary builds reputation.
- **Phase 4+ (when funding allows):** Extended Validation (EV) certificate. EV certificates bypass SmartScreen warnings immediately. Available via Azure Key Vault integration using the `signCommand` field in Tauri config.

The Tauri bundler signs all executables including the NSIS uninstaller and any sidecar binaries.

### 9.6 Tauri Updater Signing

The auto-update system uses a separate Ed25519 key pair for update artifact integrity:

```bash
# Generate the updater key pair (done once)
cargo tauri signer generate -w ~/.tauri/luminos-updater.key
```

This produces:
- **Private key:** Set via `TAURI_SIGNING_PRIVATE_KEY` environment variable in CI.
- **Public key:** Embedded in `tauri.conf.json`:

```json
{
  "plugins": {
    "updater": {
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ..."
    }
  }
}
```

The updater key is independent from platform signing keys. It signs `.tar.gz` update archives and their `.sig` sidecar files. The public key is compiled into the binary, making it impossible for an attacker to substitute a different key without recompiling.

### 9.7 Key Management

Key management evolves with the project's governance:

**Phase 0-1 (Year 1): Project Founder + CI Secrets**

| Key | Storage | Access |
|-----|---------|--------|
| GPG signing key (Linux) | GitHub Actions encrypted secrets | CI pipeline only |
| Tauri updater Ed25519 key | GitHub Actions encrypted secrets | CI pipeline only |
| Apple Developer ID certificate | GitHub Actions encrypted secrets | CI pipeline only |

- Private keys exist only in CI secrets. They are never stored on developer machines.
- The project founder manages the GitHub repository secrets.
- Key rotation: annually or immediately upon suspected compromise.

**Phase 2+ (Year 2+): Non-Profit Foundation + HSM/KMS**

| Key | Storage | Access |
|-----|---------|--------|
| GPG signing key (Linux) | Cloud KMS (e.g., AWS KMS, GCP Cloud KMS) | CI pipeline via service account |
| Tauri updater Ed25519 key | Cloud KMS | CI pipeline via service account |
| Apple Developer ID certificate | Apple Developer Portal (foundation account) | CI pipeline |
| Authenticode certificate | Cloud HSM or Azure Key Vault | CI pipeline via service account |
| OpenBSD signify key | Cloud KMS | CI pipeline or manual signing |

- Key custody transfers to the non-profit foundation (per [Product Strategy](../PRODUCT_STRATEGY.md) Section 12.2).
- Hardware Security Modules (HSMs) or cloud KMS ensure private keys are never extractable.
- Foundation board approves key rotation and revocation.
- Two-person rule for emergency key operations (revocation, rotation).

This key management strategy is aligned with [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 3.7.

---

## 10. Auto-Update Strategy

### 10.1 Update Mechanism

Luminos uses `tauri-plugin-updater` for in-app updates. The plugin follows a four-step process:

1. **Check:** The application sends an HTTP GET to configured endpoints. The URL supports template variables: `{{target}}` (OS-architecture, e.g., `linux-x86_64`), `{{arch}}` (architecture), and `{{current_version}}`.
2. **Verify:** The response signature is validated against the Ed25519 public key compiled into the binary (see Section 9.6).
3. **Download:** The update artifact is downloaded with progress events streamed to the control panel.
4. **Apply:** The update replaces the application binary. On Windows, Tauri automatically quits the application before installation (platform limitation). On macOS and Linux, the replacement is performed in-place.

**Tauri configuration:**

```json
{
  "bundle": {
    "createUpdaterArtifacts": true
  },
  "plugins": {
    "updater": {
      "pubkey": "dW50cnVzdGVkIGNvbW1lbnQ...",
      "endpoints": [
        "https://releases.luminos.dev/{{target}}/{{arch}}/{{current_version}}"
      ]
    }
  }
}
```

**Rust dependency:**

```toml
# luminos-app/Cargo.toml
[dependencies]
tauri-plugin-updater = "2"
```

**Update artifacts produced per platform:**

| Platform | Bundle Artifact | Updater Artifact | Signature |
|----------|----------------|-----------------|-----------|
| Linux | `luminos.AppImage` | `luminos.AppImage.tar.gz` | `luminos.AppImage.tar.gz.sig` |
| macOS | `Luminos.app` | `Luminos.app.tar.gz` | `Luminos.app.tar.gz.sig` |
| Windows (NSIS) | `luminos-setup.exe` | `luminos-setup.exe` | `luminos-setup.exe.sig` |
| Windows (MSI) | `luminos.msi` | `luminos.msi` | `luminos.msi.sig` |

### 10.2 Platform-Specific Behavior

| Platform | Update Method | Rationale |
|----------|--------------|-----------|
| **Linux (.deb, .rpm)** | **Package manager** (apt, dnf) | Users who install via .deb/.rpm expect updates from the APT/DNF repository. In-app updater is disabled for these installations. |
| **Linux (AppImage)** | **In-app updater** | AppImage has no system package manager. The Tauri updater replaces the AppImage file. |
| **Linux (Flatpak, Snap)** | **Store updates** (Flathub, Snap Store) | Store-managed updates. In-app updater is disabled. |
| **macOS** | **In-app updater** | No system package manager for GUI apps. The Tauri updater replaces the .app bundle. |
| **OpenBSD** | **Package manager** (`pkg_add -u`) | Port/package updates via the OpenBSD ports tree. No in-app updater. |
| **Windows** | **In-app updater** | Consumer installations use the in-app updater. Enterprise MSI deployments may use GPO/WSUS for managed updates (Phase 4). |

**Detection of installation method:** At startup, Luminos detects how it was installed (by checking for package manager metadata files, Flatpak/Snap environment variables, or AppImage mount points) and enables or disables the in-app updater accordingly.

### 10.3 Update Server Architecture

**Phase 0-1: Static JSON on GitHub Releases**

The simplest approach: a static JSON manifest hosted on GitHub Releases. The CI pipeline generates this manifest alongside release artifacts:

```json
{
  "version": "0.2.0",
  "notes": "Bug fixes and performance improvements. See full changelog at...",
  "pub_date": "2026-04-15T12:00:00Z",
  "platforms": {
    "linux-x86_64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ...",
      "url": "https://github.com/luminos-app/luminos/releases/download/v0.2.0/luminos.AppImage.tar.gz"
    },
    "darwin-aarch64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ...",
      "url": "https://github.com/luminos-app/luminos/releases/download/v0.2.0/Luminos.app.tar.gz"
    },
    "darwin-x86_64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ...",
      "url": "https://github.com/luminos-app/luminos/releases/download/v0.2.0/Luminos_x64.app.tar.gz"
    },
    "windows-x86_64": {
      "signature": "dW50cnVzdGVkIGNvbW1lbnQ...",
      "url": "https://github.com/luminos-app/luminos/releases/download/v0.2.0/luminos-setup.exe"
    }
  }
}
```

**Phase 2+: Dedicated update server (if needed)**

If GitHub Releases bandwidth becomes insufficient or download analytics are needed, a lightweight update server (e.g., Cloudflare Workers or a simple static file host with CDN) can be deployed. The endpoint URL in `tauri.conf.json` is the only change required.

### 10.4 Update Policy

Luminos follows these update principles, derived from the project's accessibility-first values:

1. **No forced updates.** Users are notified of available updates but never forced to install. Low-vision users may have workflows that depend on specific behavior -- unexpected changes can be disorienting.
2. **Update prompts are accessible.** The update notification in the control panel follows WCAG 2.1 AA requirements: keyboard-navigable, screen-reader-announced, sufficient contrast. It is not a modal dialog that blocks usage.
3. **Update frequency is configurable.** Users can set the check interval (daily, weekly, never) or manually check from the control panel. Default: weekly.
4. **Changelog is human-readable.** Update notes are written for end users, not developers. Technical changes are summarized in terms of impact ("Faster zoom at low magnification levels") not implementation ("Replaced XCB capture with XShm").
5. **Rollback is possible.** Previous AppImage files are preserved (renamed with version suffix) so users can revert if an update causes issues. On macOS/Windows, users can reinstall the previous version from GitHub Releases.

---

## 11. Release Engineering

### 11.1 Version Scheme

Luminos uses Semantic Versioning 2.0.0 (SemVer):

```
MAJOR.MINOR.PATCH[-pre.N]

0.1.0          # First development release
0.2.0-alpha.1  # Pre-release (alpha)
0.2.0-beta.1   # Pre-release (beta)
0.2.0-rc.1     # Release candidate
0.2.0          # Stable release
1.0.0          # First stable major release (Phase 1 completion)
```

**Version synchronization:**
- All five workspace crates share the same version via `workspace.package.version`.
- The Tauri `version` field in `tauri.conf.json` is set to the same value.
- The `package.json` version in `ui/` is synchronized.
- A `scripts/bump-version.sh` script updates all three locations atomically.

**Pre-1.0 conventions:**
- `0.x.y` releases are development releases. Breaking changes may occur in minor versions.
- Alpha/beta/RC pre-releases are published for testing before each minor release.
- The `1.0.0` release marks the first stable API and is targeted at Phase 1 completion (full Linux support).

### 11.2 Release Process

The release process is triggered by pushing a version tag to the main branch:

```
1. Update version
   └─> scripts/bump-version.sh 0.2.0
       ├── Updates Cargo.toml [workspace.package] version
       ├── Updates tauri.conf.json version
       └── Updates ui/package.json version

2. Update changelog
   └─> Review and finalize CHANGELOG.md (see §11.3)

3. Commit and tag
   └─> git commit -m "Release v0.2.0"
   └─> git tag -s v0.2.0 -m "Release v0.2.0"

4. Push tag
   └─> git push origin main --tags
       └── Triggers CI release pipeline (doc-07 Stage 8)

5. CI release pipeline (automated)
   ├── Runs all Stage 1-7 checks (full validation)
   ├── Builds dist-profile binaries for each platform
   ├── Runs E2E smoke tests against dist binaries
   ├── Signs all artifacts (§9)
   ├── Generates SBOM (§12.1, Phase 1+)
   ├── Generates updater manifest JSON
   ├── Publishes to GitHub Releases
   └── Updates APT/DNF repositories (Phase 1+)

6. Post-release (manual)
   ├── Verify GitHub Release artifacts are downloadable
   ├── Run release checklist (doc-07 §8)
   ├── Announce on community channels
   └── Bump version to next dev (0.3.0-dev)
```

**Abort conditions:** If any Stage 1-7 check fails on the release tag, the release pipeline aborts. The tag is not deleted -- the failure is investigated, fixed, and a new patch tag (e.g., `v0.2.1`) is created.

### 11.3 Changelog Generation

Luminos uses Conventional Commits for commit messages:

```
feat: add docked magnification mode
fix: resolve frame drop at 2x zoom on Intel UHD 770
perf: implement XShm capture for X11
docs: update keyboard shortcut reference
chore: bump wgpu to 28.1.0
```

**Changelog workflow:**
1. Commits follow the Conventional Commits specification during development.
2. Before release, `git-cliff` (or equivalent) generates a draft changelog from commit history.
3. The draft is manually reviewed and edited for clarity -- raw commit messages are developer-oriented; the changelog should be user-oriented.
4. The final changelog is committed as `CHANGELOG.md` in the repository root.

**Changelog format:**

```markdown
# Changelog

## [0.2.0] - 2026-04-15

### Added
- Docked magnification mode (top, bottom, left, right edges)
- Color inversion and grayscale filters
- Keyboard shortcuts for zoom in/out

### Fixed
- Frame drops at low zoom levels on Intel integrated GPUs
- Control panel not appearing on multi-monitor setups

### Changed
- Default zoom level changed from 2x to 3x

### Performance
- X11 capture now uses XShm shared memory (2x faster at low zoom)
```

### 11.4 GitHub Releases

Each version tag produces a GitHub Release with the following artifact structure:

```
v0.2.0/
  # Linux
  luminos_0.2.0_amd64.deb
  luminos_0.2.0_amd64.deb.sig              # GPG signature
  luminos-0.2.0-1.x86_64.rpm
  luminos_0.2.0_amd64.AppImage
  luminos_0.2.0_amd64.AppImage.tar.gz      # Updater artifact
  luminos_0.2.0_amd64.AppImage.tar.gz.sig  # Ed25519 updater signature

  # macOS (Phase 2+)
  Luminos_0.2.0_aarch64.dmg
  Luminos_0.2.0_x64.dmg
  Luminos.app.tar.gz                       # Updater artifact (aarch64)
  Luminos.app.tar.gz.sig
  Luminos_x64.app.tar.gz                   # Updater artifact (x64)
  Luminos_x64.app.tar.gz.sig

  # Windows (Phase 4+)
  luminos_0.2.0_x64-setup.exe              # NSIS installer
  luminos_0.2.0_x64-setup.exe.sig
  luminos_0.2.0_x64_en-US.msi             # WiX MSI
  luminos_0.2.0_x64_en-US.msi.sig

  # Metadata
  luminos-0.2.0-sbom.json                  # CycloneDX SBOM (Phase 1+)
  update-manifest.json                     # Tauri updater manifest
  SHA256SUMS                               # Checksums for all artifacts
  SHA256SUMS.sig                           # GPG signature of checksums
```

**Release notes template:**

```markdown
## Luminos v0.2.0

[User-facing summary of what changed, written for end users]

### What's New
- [Feature 1]
- [Feature 2]

### Bug Fixes
- [Fix 1]

### Compatibility
- Linux: Ubuntu 22.04+, Fedora 38+, Arch Linux (AUR)
- Requires: espeak-ng (installed automatically on .deb/.rpm)

### Checksums
See SHA256SUMS for verification.

### Full Changelog
[Link to CHANGELOG.md diff]
```

### 11.5 Release Cadence

From [Product Strategy](../PRODUCT_STRATEGY.md) Section 9.2:

| Release Type | Frequency | Content |
|-------------|-----------|---------|
| **Development releases** (0.x.0-alpha/beta) | As needed during active development | Feature previews, testing builds |
| **Monthly releases** (0.x.0) | Monthly during active development phases | New features, bug fixes, performance improvements |
| **Quarterly stable releases** | Quarterly | Accumulated monthly releases, thoroughly tested, recommended for general use |

**Pre-1.0:** All releases are development releases. Breaking changes are expected.

**Post-1.0:** SemVer guarantees apply. Patch releases (x.y.Z) for bug fixes, minor releases (x.Y.0) for new features, major releases (X.0.0) for breaking changes (expected to be rare for an accessibility tool -- stability is paramount).

---

## 12. SBOM and Supply Chain

### 12.1 SBOM Generation

A Software Bill of Materials (SBOM) is generated for each release in CycloneDX format using `cargo-cyclonedx`:

```bash
# Install (CI setup)
cargo install cargo-cyclonedx

# Generate SBOM for the workspace
cargo cyclonedx -f json --spec-version 1.5 --manifest-path Cargo.toml
```

**Output:** `bom.json` adjacent to the workspace root `Cargo.toml`. This file is renamed to `luminos-<version>-sbom.json` and attached to the GitHub Release (see Section 11.4).

**CycloneDX format** was chosen over SPDX because:
- CycloneDX is the OWASP-recommended format for software supply chain security.
- `cargo-cyclonedx` is the most Cargo-native tool, using both `Cargo.lock` and `cargo metadata` for accurate dependency resolution.
- CycloneDX JSON is machine-parseable for automated institutional security review (a key use case per [Product Strategy](../PRODUCT_STRATEGY.md) Section 12.1).

**CI integration (Phase 1+):**

```bash
# CI step in release pipeline
cargo cyclonedx -f json --spec-version 1.5
mv bom.json "luminos-${VERSION}-sbom.json"
# Upload as release artifact
```

**What the SBOM contains:** All direct and transitive Rust dependencies (from `Cargo.lock`), their versions, licenses, and package URLs (purls). Frontend dependencies (from `pnpm-lock.yaml`) are not included in `cargo-cyclonedx` output and must be generated separately if required by institutional customers:

```bash
# Frontend SBOM (if needed, using syft or cdxgen)
npx @cyclonedx/cdxgen -o ui-sbom.json --type npm ui/
```

### 12.2 Reproducible Builds

**Current state (2026):** The Rust toolchain does not guarantee reproducible builds out-of-the-box. Same-machine, same-toolchain builds are generally reproducible on Linux. Cross-machine reproducibility requires additional measures.

**Known obstacles:**
- Build paths embedded in panic messages and debug information
- macOS debuginfo non-determinism (rust-lang/rust#47086)
- LLVM-level non-determinism in rare cases
- Proc-macro HashMap iteration ordering

**Mitigations applied (Phase 0):**

```bash
# CI environment variables for improved reproducibility
CARGO_INCREMENTAL=0                           # Disable incremental compilation
RUSTFLAGS="--remap-path-prefix=$(pwd)=/build --remap-path-prefix=$HOME/.cargo/registry/src/=/cargo/"
```

- **`CARGO_INCREMENTAL=0`** disables incremental compilation, which stores build-path-dependent state.
- **`--remap-path-prefix`** replaces absolute build paths in the binary with fixed prefixes, eliminating path-dependent variation.

**Mitigations planned (Phase 1):**

- **Docker-based release builds:** All release artifacts are built inside a tagged, immutable Docker image with a pinned Rust toolchain. This eliminates host-environment variability.
- **Pinned toolchain:** `rust-toolchain.toml` in the repository root pins the exact Rust version:

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.85.0"
components = ["clippy", "rustfmt"]
```

**Future (when Rust stabilizes trim-paths):** RFC 3127 (`trim-paths`) will provide a Cargo-native solution for path remapping. The feature is currently nightly-only (tracked in rust-lang/cargo#12137). When stabilized, the `[profile.dist]` section will gain:

```toml
# Future: when trim-paths is stabilized
[profile.dist]
trim-paths = "all"
```

**Verification target:** Reproducible builds are a best-effort goal, not a hard requirement (per [01 -- System Architecture](./01-system-architecture.md) Section 10.4). The practical goal is that any developer with the same Docker image and Rust toolchain can produce a byte-identical binary. This enables independent verification of release artifacts.

### 12.3 Dependency Auditing

Dependency auditing is defined in [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 3.6 and enforced in CI (see [07 -- Testing Strategy](./07-testing-strategy.md) Section 4). The build and distribution pipeline integrates these checks:

| Tool | Purpose | When | Fail Condition |
|------|---------|------|----------------|
| `cargo audit` | Known vulnerability detection | Every CI push | CVSS >= 7.0 vulnerability |
| `cargo deny check licenses` | License allowlist enforcement | Every CI push | Any license not in allowlist |
| `cargo deny check bans` | Banned crate detection | Every CI push | Any banned crate present |
| `cargo deny check advisories` | Advisory database check | Every CI push | Unacknowledged advisory |

The `deny.toml` configuration is defined in [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) Section 4.3. It uses an `allow` list only -- any license not explicitly allowed is automatically denied.

---

## 13. Phase Rollout

This table consolidates all build and distribution capabilities by phase, showing what is new at each stage:

### 13.1 Phase 0: Foundation

| Capability | Status |
|-----------|--------|
| Cargo workspace compiles on Linux X11 | Required |
| `profile.dist` binary < 50MB | Required |
| Tauri bundler produces .deb, .rpm, AppImage | Required |
| GPG signing of AppImage and RPM | Required |
| .deb signing via dpkg-sig post-build | Required |
| espeak-ng bundled in AppImage | Required |
| espeak-ng as .deb/.rpm dependency | Required |
| Voice model on-demand download (Kokoro q8) | Required |
| CI release pipeline triggered by version tags | Required |
| GitHub Releases with signed artifacts + SHA256SUMS | Required |
| cargo-deny license enforcement in CI | Required |
| cargo audit vulnerability check in CI | Required |
| SemVer versioning with synchronized Cargo/Tauri/package.json | Required |
| Conventional Commits and CHANGELOG.md | Required |
| `CARGO_INCREMENTAL=0` and `--remap-path-prefix` in CI | Required |

### 13.2 Phase 1: Hardening

| Capability | Status |
|-----------|--------|
| APT repository for .deb packages | New |
| DNF repository for .rpm packages | New |
| Flatpak manifest and Flathub submission | New |
| Snap package via snapcraft | New |
| SBOM generation (cargo-cyclonedx, CycloneDX JSON) | New |
| Docker-based reproducible release builds | New |
| Pinned Rust toolchain via rust-toolchain.toml | New |
| tauri-plugin-updater for AppImage auto-updates | New |
| Static update manifest on GitHub Releases | New |
| Frontend SBOM generation (if required by institutions) | Optional |

### 13.3 Phase 2: macOS

| Capability | Status |
|-----------|--------|
| Tauri bundler produces macOS .dmg (aarch64 + x64) | New |
| Apple Developer ID code signing | New |
| Apple notarization via notarytool | New |
| espeak-ng bundled in .app bundle | New |
| macOS auto-updater via tauri-plugin-updater | New |

### 13.4 Phase 3: OpenBSD and Offline

| Capability | Status |
|-----------|--------|
| OpenBSD port/package with signify signing | New |
| Full installer variant (bundled Kokoro q8 model) | New |
| Offline model placement documentation | New |
| Self-hosted OpenBSD CI runner for build validation | New |

### 13.5 Phase 4: Windows and Enterprise

| Capability | Status |
|-----------|--------|
| Tauri bundler produces MSI (WiX) and NSIS installers | New |
| Authenticode code signing (standard certificate) | New |
| EV certificate for SmartScreen bypass (when funded) | Planned |
| GPO-compatible silent MSI install | New |
| Windows auto-updater via tauri-plugin-updater | New |
| Enterprise deployment documentation (GPO/MDM) | New |
| Model pre-seeding via GPO file distribution | New |

---

## 14. Cross-References

| Topic | Document | Section |
|-------|----------|---------|
| Cargo workspace structure and crate dependency graph | [01 -- System Architecture](./01-system-architecture.md) | 7.1, 7.2 |
| Binary structure and platform distribution overview | [01 -- System Architecture](./01-system-architecture.md) | 11.1, 11.2 |
| Build integrity requirements (signed releases, SBOM, reproducible builds) | [01 -- System Architecture](./01-system-architecture.md) | 10.4 |
| Conditional compilation patterns for platform backends | [02 -- Platform Abstraction](./02-platform-abstraction.md) | 5 |
| espeak-ng subprocess protocol and binary resolution | [04 -- TTS Pipeline](./04-tts-pipeline.md) | 3 |
| Voice model management and discovery | [04 -- TTS Pipeline](./04-tts-pipeline.md) | 8 |
| tauri-specta type generation architecture | [05 -- Control Panel](./05-control-panel.md) | 2.2 |
| Tauri configuration and IPC bindings | [05 -- Control Panel](./05-control-panel.md) | 2, 3 |
| Supply chain security and dependency auditing | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | 3.6 |
| Code signing key management strategy | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | 3.7 |
| License compliance and cargo-deny configuration | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | 4 |
| CI/CD pipeline architecture | [07 -- Testing Strategy](./07-testing-strategy.md) | 4 |
| Release stage (Stage 8) of CI pipeline | [07 -- Testing Strategy](./07-testing-strategy.md) | 4.9 |
| Release quality gates | [07 -- Testing Strategy](./07-testing-strategy.md) | 5.3 |
| Release checklist (pre-release verification) | [07 -- Testing Strategy](./07-testing-strategy.md) | 8 |
| Platform CI runner matrix and per-platform setup | [07 -- Testing Strategy](./07-testing-strategy.md) | 4.7 |
| GPLv3 licensing rationale | [Product Strategy](../PRODUCT_STRATEGY.md) | 8.4 |
| Release cadence | [Product Strategy](../PRODUCT_STRATEGY.md) | 9.2 |
| Non-profit foundation and governance transition | [Product Strategy](../PRODUCT_STRATEGY.md) | 12 |
| Technology stack and crate versions | [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) | 3-4 |
| Phased milestones and delivery timeline | [09 -- Implementation Roadmap](./09-implementation-roadmap.md) (planned) | (TBD) |

---

## 15. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-17 | Initial build and distribution document |
| 1.1 | 2026-03-17 | Post audit review: fixed Tauri CLI flags (F-001, F-004), removed fabricated RPM license field (F-003), aligned conditional compilation with doc-02 (F-002), corrected cross-reference section numbers (F-005, F-006, F-007), clarified Phase 0 packaging justification (F-008), updated cpal version (F-009), fixed resolver version for edition 2024 (F-010), aligned binary size check with doc-07 (F-011), clarified Windows signing env var distinction (P-002), added NSIS cross-compilation caveat (P-003), added ARM64 note (P-005) |
