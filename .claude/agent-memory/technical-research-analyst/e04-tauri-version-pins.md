# E04 Tauri Control Panel — Dependency Version Pins (verified 2026-06-04)

Cutoff used: published <= 2026-05-21 (>=2 weeks before 2026-06-04). RustSec DB had 1119 advisories loaded.

## Verified pin set (crates.io + cargo-audit, real data)
- tauri =2.11.2 (2026-05-16), tauri-build =2.6.2 (2026-05-16) — co-released, independently versioned
- tauri-specta =2.0.0-rc.25 (2026-05-08) — STILL RC, no stable 2.0.0 (latest stable is 1.0.2 from 2023)
- specta =2.0.0-rc.25 (2026-05-07), specta-typescript =0.0.12 (2026-05-07)
- toml =1.1.2 (full: 1.1.2+spec-1.1.0, 2026-04-01)
- directories / dirs both top out at 6.0.0 (2025-01-12), same author (soc). dirs has 4.4x downloads; directories better for app config (ProjectDirs)
- tempfile =3.27.0 (2026-03-11) — already pinned
- raw-window-handle =0.6.2 (2024-05-17) — unified across wgpu/winit/tauri
- tray-icon: latest eligible 0.24.0 (2026-05-07) but tauri 2.11.2 requires ^0.23 → resolves to 0.23.1 transitively. DO NOT add 0.24 directly (would force 2nd semver-incompatible version)
- image =0.25.10 already direct in luminos-platform → covers tray icon decoding, no new pin

## Cross-crate compatibility (from crates.io /dependencies endpoint)
- tauri 2.11.2 deps: raw-window-handle ^0.6, tauri-build ^2.6.2, specta ^2.0.0-rc.16 [opt], tray-icon ^0.23 [opt]
- tauri-specta rc.25 deps: specta =2.0.0-rc.25 (EXACT), tauri ^2, specta-typescript ^0.0.12 [opt]
- specta-typescript 0.0.12 deps: specta =2.0.0-rc.25 (EXACT)
- wgpu 29.0.3 deps: raw-window-handle ^0.6.2
- Net: all 3 specta crates MUST be rc.25 together (exact pins chain). tauri's ^2.0.0-rc.16 is satisfied by rc.25.

## Advisory status (cargo-audit on Cargo.lock, 799 deps scanned)
- VULNERABILITIES: ZERO
- Warnings: 17 unmaintained + 1 unsound, ALL transitive via Tauri 2 Linux GTK3/webkit2gtk stack:
  - glib 0.18.5 unsound RUSTSEC-2024-0429 (VariantStrIter iterator) — unavoidable, we never touch it
  - GTK3 bindings unmaintained RUSTSEC-2024-0411..0420,0436 (atk/gdk/gtk/glib/gdkx11 etc)
  - proc-macro-error RUSTSEC-2024-0370, paste RUSTSEC-2024-0436, unic-* family
- tauri RUSTSEC advisories (2022-0088, 2022-0091) are Tauri 1.x only — do not affect 2.x
- These GTK warnings are intrinsic to Tauri 2 on Linux until wry moves off GTK3 (it hasn't)

## Already-pinned workspace deps (Cargo.toml / Cargo.lock at 2026-06-04)
wgpu=29.0.3, winit=0.30.13, arc-swap=1.9.1, x11rb=0.13.2, serde=1.0.228, serde_json=1.0.149,
thiserror=2.0.18 (direct; v1.0.69 also in lock transitively), log=0.4.29, env_logger=0.11.10,
bytemuck=1.25.0, raw-window-handle=0.6.2. pollster & futures NOT used by this workspace.
NOTE: workspace Cargo.toml already contained the full Tauri pin set as of 2026-06-03 commit.

## Method notes
- crates.io API reachable via curl in sandbox (UA required). /versions and /<v>/dependencies endpoints are authoritative for dates + version reqs.
- cargo-audit + cargo-deny both installed in this env; cargo-audit auto-fetches RustSec DB.
