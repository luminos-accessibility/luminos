---
name: e04-pinned-versions
description: Audit of specs/E04-tauri-control-panel/PINNED_VERSIONS.md dependency manifest (2026-06-04). Verdict APPROVED.
metadata:
  type: project
---

# E04 PINNED_VERSIONS.md Audit (2026-06-04)

**Verdict: APPROVED.** Every claimed publish date matched primary sources exactly; all peer/advisory/compatibility claims verified. No corrections needed. Only nitpicks: tempfile "pinned in a crate's dev-deps" is slightly off (not in any manifest yet, but manifest hedges "if not present"); and "highest such version" rule is intentionally relaxed for pnpm/vite/eslint/plugin-react/TS with stated compatibility reasons (defensible, not errors).

**Why:** Supply-chain rule = published ≤2026-05-21, no open advisory, not yanked, highest-such unless compat needs lower, mutually compatible.

**How to apply:** Reuse these verified facts when re-auditing E04 deps or downstream stories. Re-verify dates if manifest is regenerated.

## Verification method (reusable)
- npm dates/peers: `https://registry.npmjs.org/<name>` JSON `time` map + `versions[v].peerDependencies`. URL-encode scoped names (`@x/y` -> `@x%2fy`).
- crates.io dates/yank: `https://crates.io/api/v1/crates/<name>/versions` (needs User-Agent header).
- Advisories: `https://api.github.com/advisories?ecosystem=npm|rust&affects=<pkg>&per_page=100` and `/advisories/<GHSA>` (has cve_id + vulnerable_version_range + first_patched_version per package). Covers RUSTSEC via ecosystem=rust.
- Python3 IS available in this env despite project Python ban (ban is for project code, not ad-hoc audit scripts). Used it for all registry queries.

## Verified publish dates (all matched manifest exactly, all <=2026-05-21)
react/react-dom 19.2.6=2026-05-06 (19.2.7=2026-06-01 too young); zustand 5.0.13=2026-05-05; zod 4.4.3=2026-05-04 (is latest); typescript 6.0.3=2026-04-16 (latest); vite 6.4.2=2026-04-06; @vitejs/plugin-react 5.2.0=2026-03-12; vitest+coverage-v8 4.1.7=2026-05-20; typescript-eslint 8.59.4=2026-05-18; eslint+@eslint/js 9.39.4=2026-03-06 (highest 9.x); @types/react 19.2.15=2026-05-19; @tauri-apps/api 2.11.0=2026-04-30; @tauri-apps/cli 2.11.2=2026-05-16; jsdom 29.1.1=2026-04-30; axe-core 4.11.4=2026-04-29; pnpm 10.33.4=2026-05-06; prettier 3.8.3=2026-04-15; eslint-plugin-react-hooks 7.1.1=2026-04-17.
Rust: directories 6.0.0=2025-01-12 (is latest, no 6.0.1/6.1.0); cargo-llvm-cov 0.8.7=2026-05-13; tauri-driver 2.0.6=2026-05-06; tauri 2.11.2=2026-05-16; tauri-build 2.6.2=2026-05-16; tauri-specta 2.0.0-rc.25=2026-05-08 (newest RC, no rc.26); specta 2.0.0-rc.25=2026-05-07; specta-typescript 0.0.12=2026-05-07; tempfile 3.27.0=2026-03-11 (highest).

## Peer-dependency facts verified
- @vitejs/plugin-react 6.x peers vite `^8.0.0` (hard) -> forces 5.2.0 on Vite 6 path. 5.2.0 peers vite `^4.2||^5||^6||^7||^8` (works w/ Vite 6).
- vitest 4.1.7 peers vite `^6||^7||^8` (manifest correctly says ^6|7|8, NOT just ^6).
- @vitest/coverage-v8 4.1.7 peers vitest `4.1.7` exactly.
- @testing-library/react 16.3.2 peers react `^18||^19`.
- eslint-plugin-jsx-a11y 6.10.2 peers eslint `^3..^9` (NO ^10) -> the binding constraint forcing ESLint 9. Latest jsx-a11y is 6.10.2 (2024-10-26).
- typescript-eslint 8.59.4 peers typescript `>=4.8.4 <6.1.0` (supports TS 6.0.x) and eslint `^8.57||^9||^10` (does NOT itself force ESLint 9; jsx-a11y does).

## Security findings (all clean for pinned versions)
- Vite GHSA-p9ff-h696-f583 (HIGH) + GHSA-4w7w-66w2-5vf9 (MED): vite 6.0.0-6.4.1 vulnerable, fixed 6.4.2. ALSO GHSA-93m4-6634-74q7 fixed 6.4.1 and GHSA-g4jq fixed 6.3.6. So 6.4.2 = min safe Vite 6, clears ALL vite-6 advisories. Confirmed.
- tauri CVE-2026-42184 = GHSA-7gmj-67g7-phm9 (MED, Origin Confusion / remote pages invoke local IPC): affects `>=2.0.0,<=2.11.0`, patched 2.11.1. Pin 2.11.2 is safe; manifest's "Patches CVE-2026-42184" claim CONFIRMED.
- vitest GHSA-5xrq-8626-4rwp (critical) affects <4.1.0 -> 4.1.7 safe.
- react/react-dom/zod advisories only affect ancient versions (react <0.14, react-dom 16.x, zod <=3.22.2) -> n/a.
- tokio 1.52.3, image 0.25.10 clear all advisories. zustand/typescript/plugin-react/coverage-v8/typescript-eslint/eslint/jsdom/axe-core: 0 applicable advisories.

## Workspace consistency
- 1a "already pinned" table matches actual `Cargo.toml [workspace.dependencies]` EXACTLY (versions + features). directories correctly absent (it's the NEW crate). tempfile genuinely not yet in any manifest (manifest hedges "if not present").

## Intentional down-pins (NOT errors, stated compat reasons correct)
vite 6.4.2 (not 8.x: stay on Vite 6 because plugin-react 5.x); plugin-react 5.2.0 (not 6.x: 6 needs Vite 8); eslint 9.39.4 (not 10.x: jsx-a11y caps ^9); TS 6.0.3 fine (latest); pnpm 10.33.4 (not 11.x: engines cap <11, stability). All deviation-table reasons are accurate.
