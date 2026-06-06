# Epic E04 — Pinned Dependency Versions

**Status:** AUTHORITATIVE for E04 implementation. Determined 2026-06-04 by three parallel
research sub-agents against live crates.io / npm registry data + RUSTSEC / GitHub advisory DBs.

**Supply-chain rule applied:** every version was published **on or before 2026-05-21**
(≥2 weeks before today, 2026-06-04), has **no open advisory**, is **not yanked/deprecated**,
is the **highest such version** unless a lower one is required for compatibility, and all
versions form a **mutually compatible set**. Pin EXACT (`=x.y.z` for Cargo, no-caret for npm).

---

## 1. Rust crates

### 1a. Already pinned in workspace `Cargo.toml` (validated safe — DO NOT change)

The workspace was pinned on 2026-06-03 with the identical methodology. Independent research
re-validated the whole Tauri set against live data — all clean, none yanked, compatible.

| Crate | Pin | Notes |
|---|---|---|
| tauri | `=2.11.2` | feat `tray-icon`. Patches CVE-2026-42184. Pulls tray-icon 0.23.1 + rwh 0.6 transitively. |
| tauri-build | `=2.6.2` | co-released with 2.11.2 |
| tauri-specta | `=2.0.0-rc.25` | **RC** — no stable 2.0 exists. feats `derive`,`typescript`. Forces specta = rc.25. |
| specta-typescript | `=0.0.12` | forces specta = rc.25 |
| (specta) | `=2.0.0-rc.25` | resolved transitively, lockstep with the two above |
| toml | `=1.1.2` | used by ConfigManager |
| raw-window-handle | `=0.6.2` | single unifying rwh across wgpu 29 / winit 0.30 / tauri 2.11 |
| wgpu | `=29.0.3` | requires rwh ^0.6.2 |
| winit | `=0.30.13` | |
| x11rb | `=0.13.2` | feats randr, shm, xinput |
| serde | `=1.0.228` | derive |
| serde_json | `=1.0.149` | |
| arc-swap | `=1.9.1` | |
| crossbeam-channel | `=0.5.15` | |
| thiserror | `=2.0.18` | |
| log | `=0.4.29` | |
| env_logger | `=0.11.10` | |
| bytemuck | `=1.25.0` | derive |
| image | `=0.25.10` | direct dep in luminos-platform; covers tray-icon decoding |
| tokio | `=1.52.3` | feats `sync` |
| tempfile | `=3.27.0` | 2026-03-11. dev-dep — added to `[workspace.dependencies]` + luminos-core `[dev-dependencies]` by story 004 (was not previously in any manifest). Highest ≤ cutoff. |

`tray-icon`: **do NOT add directly.** Tauri 2.11.2 declares `tray-icon ^0.23`; a direct
`=0.24` would force a 2nd semver-incompatible copy. Use Tauri's `tray-icon` feature (already on).

### 1b. NEW Rust crate to ADD (story 004)

| Crate | Pin | Publish date | Why |
|---|---|---|---|
| **directories** | `=6.0.0` | 2025-01-12 | `ProjectDirs::from("dev", "luminos", "luminos")` → app-scoped XDG `config_dir()` = `~/.config/luminos` (application component is lowercase `"luminos"` to match the spec-mandated path; see story 004). Preferred over `dirs` (raw base dirs only). Clean, not yanked. |

### 1c. NEW Rust crate to ADD (story 001)

| Crate | Pin | Publish date | Why |
|---|---|---|---|
| **pollster** | `=0.4.0` | 2024-10-28 | Blocks on the async `luminos_gpu::device::create_gpu_device` future from the single-threaded tao/Tauri event-loop thread, which has no tokio runtime. Minimal no-dependency executor. Highest release published ≤2026-05-21, no RUSTSEC advisory (`rustsec.org/packages/pollster.html` → 404 = no advisory page), not yanked. Added under the `tauri` feature of `luminos-app` only. |
| **libc** | `=0.2.186` | 2026-04-23 | POSIX `sigwait`/`sigaction` for catching SIGTERM/SIGINT and driving graceful event-loop shutdown (tao does not convert OS signals to `ExitRequested` on Linux GTK3). Already resolved transitively at this exact version; pinned explicitly. Published ≤2026-05-21, advisory-free, not yanked. Linux-only dependency under the `tauri` feature of `luminos-app`. |

Add to `[workspace.dependencies]`: `directories = "=6.0.0"`, and to `luminos-core`
`[dependencies]`: `directories = { workspace = true }`. Add `tempfile = "=3.27.0"` to
`[workspace.dependencies]` if not present, and to `luminos-core` `[dev-dependencies]`.

### 1d. NEW Rust crate to ADD (story 005)

| Crate | Pin | Publish date | Why |
|---|---|---|---|
| **specta** | `=2.0.0-rc.25` | 2026 (lockstep with tauri-specta rc.25) | `#[derive(specta::Type)]` on the IPC-reachable engine types (DC-5) and `#[specta::specta]` on the commands. **RC** — no stable 2.0 exists. Pinned in lockstep with the already-pinned `tauri-specta = "=2.0.0-rc.25"` / `specta-typescript = "=0.0.12"` (rc.25 forces specta = rc.25), so it brings NO new transitive version — it was already resolved transitively at exactly this version; now declared explicitly. Cargo.lock resolves a single `specta 2.0.0-rc.25` (no dual versions). `cargo deny`/`cargo audit` clean. Added with feature `["derive"]`. |

Add to `[workspace.dependencies]`: `specta = { version = "=2.0.0-rc.25", features = ["derive"] }`.
Add as a NORMAL (non-optional, non-feature-gated) dep to `luminos-types`, `luminos-core`,
`luminos-gpu` (so the derive compiles without the app's `tauri` feature →
`cargo build --workspace --exclude luminos-app` passes). Add `serde = { workspace = true }`
to `luminos-gpu` too (`FrameTimingSummary` had no serde) + `serde_json` to its `[dev-dependencies]`.
Add `specta = { workspace = true, optional = true }` to `luminos-app` under the `tauri` feature
(commands/events need it directly in scope).

**NO `tauri-plugin-shell` added (story 005 decision):** `shell:allow-open` was deferred for
Phase 0 (no shell-open consumer), so the plugin is NOT a dependency. If a future story needs it,
pin it EXACT (verify ≤ cutoff + advisory-free via crates.io), add here, and register
`.plugin(tauri_plugin_shell::init())` alongside granting the permission.

---

## 2. npm packages (frontend `ui/`) — story 006

| Package | Pin | Publish | Type | Notes |
|---|---|---|---|---|
| react | `19.2.6` | 2026-05-06 | prod | latest-safe 19.2 (19.2.7 too young) |
| react-dom | `19.2.6` | 2026-05-06 | prod | lockstep with react |
| zustand | `5.0.13` | 2026-05-05 | prod | 5.0.14 too young |
| zod | `4.4.3` | 2026-05-04 | prod | **Zod 4** (mainstream now) — target v4 API, not v3 |
| @tauri-apps/api | `2.11.0` | 2026-04-30 | prod | matches Tauri 2.11.x backend |
| @tauri-apps/cli | `2.11.2` | 2026-05-16 | dev | aligned with tauri crate 2.11.2 |
| typescript | `6.0.3` | 2026-04-16 | dev | **TS 6** (mainstream) — supported by typescript-eslint <6.1.0 |
| vite | `6.4.2` | 2026-04-06 | dev | **min safe Vite 6** (6.0–6.4.1 vulnerable: GHSA-p9ff-h696-f583, GHSA-4w7w-66w2-5vf9) |
| @vitejs/plugin-react | `5.2.0` | 2026-03-12 | dev | **stay on 5.x** — plugin-react 6.x hard-peers vite ^8 |
| vitest | `4.1.7` | 2026-05-20 | dev | peer vite ^6\|7\|8 |
| @vitest/coverage-v8 | `4.1.7` | 2026-05-20 | dev | peer-pins exactly to vitest version |
| @testing-library/react | `16.3.2` | 2026-01-19 | dev | peer react ^18\|^19 → React 19 OK |
| @testing-library/jest-dom | `6.9.1` | 2025-10-01 | dev | |
| @testing-library/user-event | `14.6.1` | 2025-01-21 | dev | |
| jsdom | `29.1.1` | 2026-04-30 | dev | required by axe (not happy-dom) |
| axe-core | `4.11.4` | 2026-04-29 | dev | pinned directly (auditable) |
| jest-axe | `10.0.0` | 2025-03-03 | dev | chosen over abandoned vitest-axe; `expect.extend({toHaveNoViolations})` |
| @types/react | `19.2.15` | 2026-05-19 | dev | |
| @types/react-dom | `19.2.3` | 2025-11-12 | dev | |
| eslint | `9.39.4` | 2026-03-06 | dev | **ESLint 9, not 10** — jsx-a11y caps peer at ^9 |
| @eslint/js | `9.39.4` | 2026-03-06 | dev | match eslint |
| typescript-eslint | `8.59.4` | 2026-05-18 | dev | 8.60.0 too young; supports TS <6.1.0 |
| eslint-plugin-react-hooks | `7.1.1` | 2026-04-17 | dev | |
| eslint-plugin-jsx-a11y | `6.10.2` | 2024-10-26 | dev | binding constraint forcing ESLint 9 |
| globals | `16.5.0` | 2025-11-01 | dev | required by `ui/eslint.config.js` flat config (`globals.browser`/`globals.node`); a transitive-of-ESLint-config dep, declared directly + exact-pinned. No advisories. |
| prettier | `3.8.3` | 2026-04-15 | dev | |

```jsonc
"packageManager": "pnpm@10.33.4",          // 2026-05-06 — pnpm 10 stable mainstream
"engines": { "node": ">=24.14.0", "pnpm": ">=10.0.0 <11.0.0" }
```

### 2a. npm packages (E2E `e2e/`) — story 007

The `tauri-driver` E2E suite (WebdriverIO 9 + Mocha, TypeScript via `tsx`). All
dev-only; researched against the live npm registry — each version is the highest
**stable** release published **on or before 2026-05-21**, advisory-free, not yanked.
Pin EXACT (no caret). `pnpm-lock.yaml` committed for `--frozen-lockfile`.

| Package | Pin | Publish | Notes |
|---|---|---|---|
| webdriverio | `9.27.1` | 2026-04-30 | WDIO 9 (9.27.2 too young). Provides `$`/`browser` runtime + types. |
| @wdio/cli | `9.27.1` | 2026-04-30 | lockstep with webdriverio |
| @wdio/local-runner | `9.27.1` | 2026-04-30 | local runner |
| @wdio/mocha-framework | `9.27.1` | 2026-04-30 | Mocha adapter (provides the global `expect` + Mocha hook types) |
| @wdio/spec-reporter | `9.27.1` | 2026-04-30 | spec reporter |
| @wdio/types | `9.27.1` | 2026-04-30 | `WebdriverIO.Config` type |
| @wdio/globals | `9.27.1` | 2026-04-30 | `@wdio/globals/types` (the `tsconfig` `types` entry that resolves `WebdriverIO.*` globals) |
| mocha | `11.7.6` | 2026-05-21 | test framework (== cutoff day; eligible) |
| @types/mocha | `10.0.10` | 2024-11-20 | highest available; mocha 11 has no bundled types |
| @types/node | `24.12.4` | 2026-05-11 | matches Node 24; `os`/`path`/`child_process`/`url` types for the config |
| tsx | `4.22.3` | 2026-05-19 | transpiles the `.ts` config + specs (no ts-node; WDIO 9 loads via tsx) |
| typescript | `6.0.3` | 2026-04-16 | TS 6, matches the `ui/` pin |

```jsonc
"packageManager": "pnpm@10.33.4",
"engines": { "node": ">=24.14.0", "pnpm": ">=10.0.0 <11.0.0" }
```

**NOT an npm dep:** the WebDriver driver is the **Rust `tauri-driver` v2.0.6** crate
(§3, `cargo install`), NOT `@crabnebula/tauri-driver` (npm) — the latter is stale.
`tauri-driver` 2.0.6's `tauri:options` supports only `application`/`args` (source-verified
— no `env` field), so the headless-WebKit env is injected into the `tauri-driver`
process env, not the capability.

---

## 3. Dev tooling (cargo-installed binaries)

```bash
cargo install cargo-llvm-cov --version 0.8.7 --locked   # 2026-05-13 — CI coverage gate
cargo install tauri-driver  --version 2.0.6 --locked    # 2026-05-06 — Tauri 2.x WebDriver (2.x line!)
```
- `tauri-driver 2.0.6` launches the system `WebKitWebDriver` (from Gentoo webkit-gtk) on Linux;
  default proxy port 4444; run under Xvfb for CI. The `0.x` line is Tauri v1 — do not use.
- Already installed & OK: cargo-nextest 0.9.132, cargo-deny 0.19.8, cargo-audit 0.22.1.

---

## 4. Deviations from spec (recorded per user decision: "pin latest safe")

The specs named some looser/older versions; the user instructed pinning the **latest safe**
versions. These supersede the spec text (update the relevant STORY/DESIGN/HIGH_LEVEL_PLAN notes):

| Item | Spec said | Pinned | Reason |
|---|---|---|---|
| pnpm | 9.x | **10.33.4** | pnpm 10 is stable mainstream; 9.x maintenance-only |
| TypeScript | 5 | **6.0.3** | TS 6 mainstream; linter supports it |
| Zod | (3 in memory) | **4.4.3** | Zod 4 mainstream — code targets v4 API |
| Vite | 6.x | **6.4.2** | min version free of the two dev-server CVEs |
| @vitejs/plugin-react | (latest) | **5.2.0** | 6.x requires Vite 8; we stay on Vite 6 |
| ESLint | (latest) | **9.39.4** | jsx-a11y has no ESLint 10 support yet |

---

## 5. Security baseline note

`deny.toml` already carries the `[advisories] ignore` list for the ~18 unavoidable transitive
GTK3/webkit2gtk warnings (RUSTSEC-2024-0411..0420, -0370, -0429, -0436, unic-* 2025 set).
These are intrinsic to Tauri 2's Linux backend and unfixable until wry leaves GTK3. `cargo deny
check advisories` and `cargo audit` should stay green. Do not remove entries without re-checking.
