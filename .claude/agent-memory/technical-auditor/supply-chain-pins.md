---
name: supply-chain-pins
description: Verified facts about the dependency-pinning supply-chain audit (workspace Cargo.toml exact pins, MSRV, advisory clearances) as of 2026-06-03
metadata:
  type: project
---

# Supply-Chain Dependency Pinning Audit (2026-06-03)

Workspace pins live in root `Cargo.toml` `[workspace.dependencies]`, all `=x.y.z` exact. Eligibility rule: newest stable published on/before 2026-05-20 (>=2wk old), no open RUSTSEC advisory, not yanked. MSRV target 1.92.

## Verdict: PASS. All 25 pins eligible and correct.

## Key verified version/advisory facts (crates.io API + OSV.dev, 2026-06-03)
- serde_json 1.0.150 = 2026-05-21 (too new, correctly rejected) -> 1.0.149 (2026-01-06) correct
- log 0.4.30=2026-05-25, 0.4.31=2026-06-02 both too new -> 0.4.29 (2025-12-02) correct
- xcap 0.9.5=2026-05-23, 0.9.6=2026-05-24 both too new -> 0.9.4 (2026-04-09) correct
- cpal 0.17.2 IS yanked -> 0.17.3 (2026-02-18) correct
- crossbeam-channel 0.5.12/0.5.13/0.5.14 all yanked; RUSTSEC-2025-0024 (double-free) introduced 0.5.12, FIXED 0.5.15 -> 0.5.15 correct
- tauri Origin Confusion CVE-2026-42184 / GHSA-7gmj-67g7-phm9: affected <=2.11.0, FIXED 2.11.1 -> 2.11.2 (2026-05-16) clear
- tokio RUSTSEC-2025-0023 (broadcast unsound): fixed >=1.44.2 -> 1.52.3 (2026-05-08) clear
- arc-swap RUSTSEC-2020-0091: fixed >=1.1.0 -> 1.9.1 clear (1.9.1 is newest)
- image advisories are only RUSTSEC-2019-0014 + RUSTSEC-2020-0073 (old 0.x) -> 0.25.10 clear
- tauri-specta: NO stable 2.x exists; highest is 2.0.0-rc.25 (2026-05-08). specta-typescript: no stable, highest 0.0.12 (2026-05-07). RC/0.0.x pins justified.
- winit: only 0.31.0-beta.* above 0.30.13; 0.30.13 (2026-03-02) is newest STABLE
- atspi 0.30.0 (2026-05-06) is newest; 0.29.0 was prior

## MSRV justification ERROR in Cargo.toml (lines 19-20)
Comment claims "wgpu 29 and ashpd 0.13 require >= 1.92". FALSE per crates.io rust_version:
- wgpu 29.0.3 declares MSRV 1.87.0
- ashpd 0.13.11 declares MSRV 1.87
- image 0.25.10 declares MSRV 1.88.0
None require 1.92. The 1.92 floor is the team's chosen toolchain or a transitive dep, NOT these two crates. Eligibility unaffected (all < 1.92 target), but the stated rationale is inaccurate. The task brief's claim "ashpd 0.13.11 needs 1.92" is also wrong (it's 1.87).

## rdev removal
rdev fully removed as a dependency (grep of all Cargo.toml = none). BUT doc-comment table in
crates/luminos-platform/src/traits/input_monitor.rs lines 217-221 still names "rdev" as the
intended per-platform InputMonitor backend for X11/Wayland/macOS/OpenBSD/Windows. So "zero code
use-sites" is true for compiled deps but the design still references rdev as future backend.
Removal of the unused dep is sound (last release 2023-06-26, unmaintained); just note the doc still points at it.

## crates.io API field gotcha
`newest_version` field can be misleading (ashpd showed "0.9.3" as newest_version while
max_stable_version was correctly "0.13.11"). Trust max_stable_version / the versions list, not newest_version.
