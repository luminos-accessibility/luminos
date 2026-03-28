# Design: Story E01/001 -- Cargo Workspace & Build Profiles

**Story:** [STORY.md](./STORY.md)
**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** DONE
**Author:** Principal Architect Agent
**Risk Refs:** RISK-001 (dual event loop), RISK-022 (license compatibility), RISK-024 (binary size budget), RISK-030 (wgpu/winit/Tauri version cascade)

---

## Overview

This design establishes the Cargo workspace skeleton, build profiles, and project-level configuration files that all subsequent stories and epics depend on. The approach is to create the exact five-crate workspace structure from doc-01 Section 7, define inter-crate dependencies matching the architectural layering, and populate every configuration file (`deny.toml`, `.clippy.toml`, `rustfmt.toml`, `.config/nextest.toml`, `rust-toolchain.toml`) with the values mandated by the tech strategy documents (doc-07 and doc-08).

The workspace uses Rust 2024 edition (resolver 3 by default, no explicit `resolver` field), centralized `[workspace.dependencies]` for version deduplication, and three build profiles (`dev`, `release`, `dist`) matching doc-08 Section 4. Each crate stub contains a minimal `src/lib.rs` (or `src/main.rs` for `luminos-app`) that compiles with zero warnings. Cargo features are pre-defined per crate per doc-08 Section 3.2 so that later stories can use conditional compilation without modifying manifests.

## Architecture

### Component Diagram

```
luminos/                          (workspace root)
├── Cargo.toml                    (workspace manifest)
├── Cargo.lock                    (committed for reproducibility)
├── rust-toolchain.toml           (stable channel, rustc 1.85+)
├── .clippy.toml                  (clippy thresholds)
├── rustfmt.toml                  (formatting rules)
├── deny.toml                     (license + advisory config)
├── .config/
│   └── nextest.toml              (test runner profiles)
├── LICENSE                       (GPL-3.0-only full text)
├── CHANGELOG.md                  (Keep a Changelog header)
└── crates/
    ├── luminos-platform/         (foundation: zero internal deps)
    │   ├── Cargo.toml
    │   └── src/lib.rs
    ├── luminos-gpu/              (depends on: luminos-platform)
    │   ├── Cargo.toml
    │   └── src/lib.rs
    ├── luminos-tts/              (depends on: luminos-platform)
    │   ├── Cargo.toml
    │   └── src/lib.rs
    ├── luminos-core/             (depends on: luminos-platform, luminos-tts)
    │   ├── Cargo.toml
    │   └── src/lib.rs
    └── luminos-app/              (depends on: all four, binary crate)
        ├── Cargo.toml
        └── src/main.rs
```

### Crate Dependency Graph

```
luminos-app (binary)
  ├── luminos-core
  │   ├── luminos-platform
  │   └── luminos-tts
  │       └── luminos-platform
  ├── luminos-gpu
  │   └── luminos-platform
  ├── luminos-tts
  │   └── luminos-platform
  └── luminos-platform
```

### Affected Traits / Modules

| Trait / Module | Change Type | Description |
|----------------|-------------|-------------|
| Workspace root `Cargo.toml` | New | Workspace members, metadata, dependencies, profiles |
| `luminos-platform` crate | New stub | `src/lib.rs` with doc-comment, no code |
| `luminos-gpu` crate | New stub | `src/lib.rs` with doc-comment, no code |
| `luminos-tts` crate | New stub | `src/lib.rs` with doc-comment, no code |
| `luminos-core` crate | New stub | `src/lib.rs` with doc-comment, no code |
| `luminos-app` crate | New stub | `src/main.rs` with `fn main() {}` |

### Data Flow

No runtime data flow in this story. This story produces the build infrastructure that enables data flow in later stories.

## API Design

### Workspace Root Cargo.toml

```toml
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

# Build profiles
[profile.dev]
opt-level = 0
debug = true
incremental = true

[profile.release]
opt-level = 3
debug = false
lto = "thin"
codegen-units = 16
strip = "debuginfo"

[profile.dist]
inherits = "release"
opt-level = "z"
lto = "fat"
codegen-units = 1
panic = "abort"
strip = "symbols"
```

### Per-Crate Cargo.toml Examples

**luminos-platform** (foundation crate, zero internal deps):

```toml
[package]
name = "luminos-platform"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true

[features]
default = []
wayland = ["ashpd"]
xshm = ["x11rb/shm"]
test_utils = []
ci_platform_tests = []

[dependencies]
thiserror = { workspace = true }
log = { workspace = true }

[target.'cfg(target_os = "linux")'.dependencies]
x11rb = { workspace = true }
xcap = { workspace = true }
atspi = { workspace = true }
rdev = { workspace = true }

[target.'cfg(target_os = "linux")'.dependencies.ashpd]
version = "0.10"
optional = true

[target.'cfg(target_os = "macos")'.dependencies]
xcap = { workspace = true }

[target.'cfg(target_os = "openbsd")'.dependencies]
x11rb = { workspace = true }
xcap = { workspace = true }

[target.'cfg(target_os = "windows")'.dependencies]
xcap = { workspace = true }
```

**luminos-gpu** (depends on luminos-platform):

```toml
[package]
name = "luminos-gpu"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true

[features]
default = []
test_utils = []
update_refs = []
profiling = []

[dependencies]
luminos-platform = { workspace = true }
wgpu = { workspace = true }
winit = { workspace = true }
log = { workspace = true }
```

**luminos-tts** (depends on luminos-platform):

```toml
[package]
name = "luminos-tts"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true

[features]
default = []
test_utils = []

[dependencies]
luminos-platform = { workspace = true }
sherpa-rs = { workspace = true }
cpal = { workspace = true }
log = { workspace = true }
```

**luminos-core** (depends on luminos-platform, luminos-tts):

```toml
[package]
name = "luminos-core"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true

[features]
default = []
test_utils = [
    "luminos-platform/test_utils",
    "luminos-gpu/test_utils",
    "luminos-tts/test_utils",
]

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
```

**luminos-app** (binary crate, depends on all four):

```toml
[package]
name = "luminos-app"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true

[features]
default = []
integration_tests = ["luminos-core/test_utils"]
ci_platform_tests = ["luminos-platform/ci_platform_tests"]
profiling = ["luminos-gpu/profiling"]

[dependencies]
luminos-core = { workspace = true }
luminos-platform = { workspace = true }
luminos-gpu = { workspace = true }
luminos-tts = { workspace = true }
tauri = { workspace = true }
tauri-specta = { workspace = true }
specta-typescript = { workspace = true }
log = { workspace = true }
env_logger = { workspace = true }
arc-swap = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }

[build-dependencies]
tauri-build = { workspace = true }
```

### Crate Stub Source Files

Each `src/lib.rs` contains only a doc-comment and linting attributes. No business logic. Example for `luminos-platform`:

```rust
//! Platform abstraction layer for Luminos.
//!
//! Defines the six platform traits (`ScreenCapture`, `FocusTracker`,
//! `TtsEngine`, `WindowManager`, `InputMonitor`, `AudioOutput`) and
//! their per-platform backend implementations.
```

`luminos-app/src/main.rs`:

```rust
//! Luminos application entry point.
//!
//! Initializes the Tauri control panel and winit magnification overlay.

fn main() {}
```

### Configuration Files

**rust-toolchain.toml:**

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

**.clippy.toml:**

```toml
cognitive-complexity-threshold = 25
too-many-arguments-threshold = 7
type-complexity-threshold = 250
```

**rustfmt.toml:**

```toml
edition = "2024"
```

**deny.toml:**

```toml
[graph]
targets = []
all-features = true

[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Zlib",
    "MPL-2.0",
    "GPL-3.0-only",
    "GPL-3.0-or-later",
    "LGPL-2.1-only",
    "LGPL-2.1-or-later",
    "LGPL-3.0-only",
    "LGPL-3.0-or-later",
    "Unicode-3.0",
    "Unicode-DFS-2016",
]
confidence-threshold = 0.8

[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "warn"
notice = "warn"

[bans]
multiple-versions = "warn"
wildcards = "deny"
```

**.config/nextest.toml:**

```toml
[store]
dir = "target/nextest"

[profile.default]
retries = 0
slow-timeout = { period = "30s", terminate-after = 2 }
fail-fast = true

[profile.ci]
retries = 2
slow-timeout = { period = "60s", terminate-after = 3 }
fail-fast = false

[[profile.ci.overrides]]
filter = "test(~platform_integration_)"
slow-timeout = { period = "120s", terminate-after = 2 }

[[profile.ci.overrides]]
filter = "test(~tts_pipeline_integration_)"
slow-timeout = { period = "180s", terminate-after = 2 }
```

## Error Handling

No error handling in this story -- all files are static configuration. Build errors manifest as `cargo build` failures with explicit compiler messages. `cargo deny check` failures produce structured output identifying the offending crate and license.

## Platform Considerations

| Platform | Approach | Notes |
|----------|----------|-------|
| Linux X11 | Primary development target | Platform-specific deps gated by `cfg(target_os = "linux")` |
| Linux Wayland | Compiles on Linux | `wayland` feature optional; `ashpd` dep is optional |
| macOS | Cross-compiles | `cfg(target_os = "macos")` deps declared |
| OpenBSD | Cross-compiles | Shares X11 deps with Linux |
| Windows | Cross-compiles | `cfg(target_os = "windows")` deps declared |

All platform-specific dependencies are declared under `[target.'cfg(...)'.dependencies]` sections so they only resolve on the correct platform. The workspace compiles on all platforms from day one.

## Testing Strategy

### Unit Tests

No unit tests required -- this story produces configuration files and empty crate stubs, not executable logic.

### Integration Tests

Build verification is the primary testing approach. Each AC is verified by running a specific command and inspecting the result.

### Acceptance Tests

| Acceptance Criterion | Test Type | Verification Method |
|---------------------|-----------|-------------------|
| AC-1.1 | Build verification | Run `cargo build --workspace` on a fresh clone; assert exit code 0 and zero warnings in stderr |
| AC-1.2 | Lint verification | Run `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::unwrap_used -W clippy::expect_used -W clippy::pedantic -A clippy::module_name_repetitions`; assert exit code 0 |
| AC-1.3 | Format verification | Run `cargo fmt --all -- --check`; assert exit code 0 and no diff output |
| AC-2.1 | File inspection | Parse root `Cargo.toml`; assert `[workspace].members` contains exactly the five crate paths |
| AC-2.2 | Dependency tree | Run `cargo tree -p luminos-app`; assert dependency edges match the specified graph |
| AC-2.3 | File inspection | Parse each crate `Cargo.toml`; assert `version`, `edition`, `license`, `rust-version` all use `{ workspace = true }` or `.workspace = true` |
| AC-2.4 | File inspection | Parse root `Cargo.toml` `[workspace.package]`; assert `edition = "2024"`, `license = "GPL-3.0-only"`, `rust-version = "1.85"`, `version = "0.1.0"` |
| AC-3.1 | File inspection | Parse root `Cargo.toml` `[profile.dev]`; assert `opt-level = 0`, `debug = true`, `incremental = true` |
| AC-3.2 | File inspection | Parse root `Cargo.toml` `[profile.release]`; assert `opt-level = 3`, `lto = "thin"`, `codegen-units = 16`, `strip = "debuginfo"` |
| AC-3.3 | File inspection | Parse root `Cargo.toml` `[profile.dist]`; assert `inherits = "release"`, `opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, `strip = "symbols"` |
| AC-3.4 | Build verification | Run `cargo build --profile dist -p luminos-app`; assert exit code 0 and binary exists at `target/dist/luminos-app` |
| AC-4.1 | File inspection | Parse `rust-toolchain.toml`; assert `channel = "stable"` and components include `rustfmt`, `clippy` |
| AC-4.2 | File inspection | Parse `.clippy.toml`; assert thresholds match specified values |
| AC-4.3 | File inspection | Parse `deny.toml` `[licenses]`; assert `allow` list contains all specified licenses and `confidence-threshold = 0.8` |
| AC-4.4 | License check | Run `cargo deny check licenses advisories`; assert exit code 0 |
| AC-4.5 | File inspection | Parse `.config/nextest.toml`; assert `default` profile has `fail-fast = true`, `ci` profile has `retries = 2` and `fail-fast = false` |
| AC-5.1 | File inspection | Parse `luminos-platform/Cargo.toml` `[features]`; assert contains `default = []`, `wayland`, `xshm`, `test_utils`, `ci_platform_tests` |
| AC-5.2 | File inspection | Parse `luminos-gpu/Cargo.toml` `[features]`; assert contains `default = []`, `test_utils`, `update_refs`, `profiling` |
| AC-5.3 | File inspection | Parse `luminos-core/Cargo.toml` `[features]`; assert `test_utils` transitively enables the three sub-crate `test_utils` features |
| AC-5.4 | File inspection | Parse `luminos-tts/Cargo.toml` `[features]`; assert contains `default = []` and `test_utils` |
| AC-5.5 | File inspection | Parse `luminos-app/Cargo.toml` `[features]`; assert contains `integration_tests`, `ci_platform_tests`, `profiling` with correct transitive enables |

## Performance Targets

| Target | Source | Verification |
|--------|--------|-------------|
| Clean build < 5min (cold cache on CI) | NFR-1 | Measured in Story 005 CI pipeline |
| Clean build < 2min (warm cache on CI) | NFR-1 | Measured in Story 005 CI pipeline |
| Zero warnings with `RUSTFLAGS="--deny warnings"` | NFR-2 | Verified by AC-1.1 build check |

## Security Considerations

- **License compliance** (RISK-022): `deny.toml` enforces a GPLv3-compatible allowlist. Any dependency with a license not in the list causes `cargo deny check` to fail, preventing accidental introduction of incompatible licenses.
- **Dependency version pinning** (RISK-030): `Cargo.lock` is committed to the repository. Workspace-level dependency declarations centralize version management to prevent transitive version skew.
- **Transitive dependency audit** (RISK-001): After initial workspace setup, run `cargo tree -d` to detect duplicate transitive dependencies between Tauri and winit/wgpu. Document any conflicts found.

## Alternatives Considered

### Alternative: Monolithic single-crate architecture

**Approach:** Put all code in a single crate instead of a five-crate workspace.

**Rejected because:**
- Single crate prevents parallel compilation of independent subsystems.
- Larger blast radius for changes (any file change recompiles everything).
- Cannot enforce architectural layering at the crate boundary level.
- Conflicts with doc-01 Section 7 architecture which mandates the five-crate structure.
- AI agents working on different subsystems would have higher merge conflict rates.

### Alternative: Use `resolver = "2"` explicitly

**Approach:** Set `resolver = "2"` in the workspace root for backward compatibility.

**Rejected because:** Edition 2024 defaults to resolver 3 (MSRV-aware), which is strictly better. Adding `resolver = "2"` would disable MSRV-aware resolution and is unnecessary given the MSRV of 1.85+ (which supports edition 2024). No explicit resolver field is needed.
