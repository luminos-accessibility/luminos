# Versioning

This is the canonical versioning reference for the Luminos project. All version policy decisions, bump rules, and milestone definitions are maintained here.

Luminos follows [Semantic Versioning 2.0.0](https://semver.org/) (SemVer) with **lockstep workspace versioning**. All Rust crates in the workspace and the Tauri application share a single version number, defined once and inherited everywhere.

## Quick Reference

- **Version source of truth:** `[workspace.package] version` in root `Cargo.toml`. Crates inherit via `version.workspace = true`. Tauri `tauri.conf.json` and `package.json` must match when they exist.
- **Pre-1.0 rules (current):** version is `0.MINOR.PATCH`. Cargo treats the left-most non-zero component as the compatibility boundary:
  - `MINOR` bump (e.g., 0.1.0 --> 0.2.0) = breaking changes or major new features
  - `PATCH` bump (e.g., 0.1.0 --> 0.1.1) = bug fixes, non-breaking additions
- **Post-1.0 rules:**
  - `MAJOR` bump = breaking user-facing or API changes (`BREAKING CHANGE` footer or `!` in commit)
  - `MINOR` bump = new features, non-breaking additions (`feat` commits)
  - `PATCH` bump = bug fixes, performance improvements (`fix`, `perf` commits)
- **Conventional Commits --> version bump mapping:**
  - `BREAKING CHANGE` footer or `!` suffix --> MAJOR (post-1.0) or MINOR (pre-1.0)
  - `feat` --> MINOR (post-1.0) or PATCH (pre-1.0)
  - `fix`, `perf` --> PATCH
  - `docs`, `style`, `refactor`, `test`, `build`, `ci`, `chore` --> no bump (release discretion)
- **Pre-release format:** `0.2.0-alpha.1`, `0.2.0-rc.1`
- **Milestone:** 1.0.0 targets end of Phase 1 (production-ready X11 magnification with all features)
- Desktop apps: user-facing behavior changes (zoom modes, keybindings, settings format) are the primary breaking-change signal, not just Rust API surface

## Version Format

```
MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]
```

- **MAJOR** -- incompatible changes (breaking user-facing behavior or public API)
- **MINOR** -- new functionality added in a backward-compatible manner
- **PATCH** -- backward-compatible bug fixes and minor improvements
- **PRERELEASE** -- optional pre-release identifier (e.g., `alpha.1`, `beta.2`, `rc.1`)
- **BUILD** -- optional build metadata (e.g., `+build.42`), ignored for precedence

## Pre-1.0 Rules (Current Phase)

While Luminos is in pre-1.0 development (`0.y.z`), Cargo treats the left-most non-zero component as the compatibility boundary. In practice:

| Version change | Meaning | Example |
|----------------|---------|---------|
| `0.MINOR` bump | Breaking changes or significant new features | `0.1.0` --> `0.2.0` |
| `0.x.PATCH` bump | Bug fixes, non-breaking additions | `0.1.0` --> `0.1.1` |
| `0.0.z` bump | Always considered breaking (every change) | `0.0.1` --> `0.0.2` |

During pre-1.0, the project treats these as breaking changes:

- Removing or renaming user-facing features (zoom modes, keybindings, settings)
- Changing configuration file formats in incompatible ways
- Removing or renaming public Rust API items (traits, structs, functions)
- Changing trait signatures that platform backends implement

## Post-1.0 Rules

After the 1.0.0 release, standard SemVer applies:

| Version component | When to bump | Conventional Commits trigger |
|-------------------|-------------|------------------------------|
| **MAJOR** | Breaking user-facing or API changes | `BREAKING CHANGE` footer or `!` suffix on any type |
| **MINOR** | New features, non-breaking additions | `feat` type commits |
| **PATCH** | Bug fixes, performance improvements | `fix`, `perf` type commits |
| *No bump* | Docs, style, refactor, tests, CI, chores | `docs`, `style`, `refactor`, `test`, `build`, `ci`, `chore` |

## Version Bump Decision Tree

Use this checklist to determine which version component to bump:

1. Did you change user-facing behavior in an incompatible way (removed a feature, changed keybindings, broke config file compatibility)?
   - **Yes** --> bump MAJOR (post-1.0) or MINOR (pre-1.0)
2. Did you add a new user-facing feature or capability?
   - **Yes** --> bump MINOR (post-1.0) or PATCH (pre-1.0)
3. Did you fix a bug, improve performance, or make non-breaking internal changes?
   - **Yes** --> bump PATCH
4. Did you only change documentation, tests, CI, or formatting?
   - **Yes** --> no version bump required (bump at release discretion)

**Desktop app note:** For a desktop application like Luminos, "breaking change" is primarily defined by user-facing behavior (settings, keybindings, zoom modes, TTS behavior), not just Rust API surface. A library consumer cares about type signatures; an end user cares about whether their workflow still works after an update.

## Where the Version Lives

The version is defined in a single place and inherited by all crates:

```toml
# Root Cargo.toml
[workspace.package]
version = "0.1.0"
```

Each crate inherits it:

```toml
# crates/luminos-core/Cargo.toml (and all other crates)
[package]
version.workspace = true
```

When the Tauri app is fully configured, these files must also be updated to match:

| File | Field | Example |
|------|-------|---------|
| `Cargo.toml` (root) | `[workspace.package] version` | `"0.2.0"` |
| `tauri.conf.json` | `version` | `"0.2.0"` |
| `package.json` | `"version"` | `"0.2.0"` |

**Tooling:** Use [`cargo-edit`](https://github.com/killercup/cargo-edit) to bump the workspace version consistently:

```bash
# Install cargo-edit (provides cargo set-version)
cargo install cargo-edit

# Bump the workspace version
cargo set-version --workspace 0.2.0
```

Alternatively, [`cargo-release`](https://github.com/crate-ci/cargo-release) can automate version bumps, changelogs, and git tags in one command.

## Pre-Release and Build Metadata

Pre-release versions follow the format `MAJOR.MINOR.PATCH-PRERELEASE`:

| Stage | Format | Purpose |
|-------|--------|---------|
| Alpha | `0.2.0-alpha.1` | Early testing, unstable features |
| Beta | `0.2.0-beta.1` | Feature-complete, testing for stability |
| Release candidate | `0.2.0-rc.1` | Final validation before release |
| Release | `0.2.0` | Stable release |

Pre-release versions have lower precedence than the associated release: `0.2.0-alpha.1 < 0.2.0-beta.1 < 0.2.0-rc.1 < 0.2.0`.

## Milestone Versions

| Version range | Project phase | Milestone |
|---------------|--------------|-----------|
| `0.1.x` | Phase 0: Foundation | Project scaffolding, traits, CI/CD |
| `0.2.x` - `0.4.x` | Phase 0: Foundation | X11 capture, GPU rendering, control panel |
| `0.5.x` - `0.9.x` | Phase 1: Core Magnification | Lens/docked modes, visual enhancement, focus tracking, keybindings |
| **1.0.0** | **End of Phase 1** | **First stable release** -- production-ready screen magnification on Linux X11 |
| `1.x.y` | Phase 2+: TTS, Cross-Platform, Advanced | TTS, Wayland, macOS, OpenBSD, Windows, plugins |

The exact version-to-phase mapping will evolve as development progresses. The key milestone is **1.0.0**, which signals that Luminos delivers fully stable, production-ready screen magnification with all features working on Linux X11 systems.

## References

- [SemVer 2.0.0 Specification](https://semver.org/) -- the authoritative SemVer standard
- [Cargo SemVer Compatibility](https://doc.rust-lang.org/cargo/reference/semver.html) -- Cargo-specific SemVer rules and compatibility guidelines
- [Cargo Dependency Version Syntax](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html) -- how Cargo resolves version requirements
- [Conventional Commits v1.0.0](https://conventionalcommits.org/en/v1.0.0/) -- commit message format used by this project (see [CLAUDE.md](CLAUDE.md) for details)
