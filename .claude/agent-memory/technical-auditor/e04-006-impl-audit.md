---
name: e04-006-impl-audit
description: E04 Story 006 (frontend control panel) IPC-contract audit (2026-06-04, commit 25f8359) — verdict, verified wire format, test-count reconciliation
metadata:
  type: project
---

# E04 Story 006 (Frontend UI) Implementation Audit — 2026-06-04 (commit 25f8359)

**Verdict: AUDIT PASS.** IPC wire-format contract 006 assumed is CORRECT against the real Rust. Story 005 can safely generate matching bindings. 0 blocking, 1 non-blocking pointer.

**Why:** Primary concern was 005↔006 silent IPC mismatch (005 will generate TS bindings from Rust types; any divergence breaks the app at runtime via Zod reject).
**How to apply:** Reuse these verified wire-format facts when auditing story 005 (the generator side). If 005 produces anything different from below, it is the bug — 006 matched Rust exactly.

## IPC wire format — VERIFIED against real Rust (the headline)
- **NO `#[serde(rename_all)]` anywhere** in `luminos-core::config::schema` (AppSettings + MagnificationSettings/ColorFilterConfig/CursorConfig/SpeechSettings). Derives are plain `Serialize, Deserialize`. → struct keys are **snake_case** (`zoom_level`, `color_filter`, `tracking_mode`, ...). schema.rs lines 86, 137, 162, 198, 234.
- **All IPC enums use serde DEFAULT = bare PascalCase externally-tagged.** Each enum has a `*_serde_roundtrip` unit test asserting the EXACT JSON string (`"FullScreen"`, `"Cursor"`, `"Bilinear"`, `"None"`, etc.):
  - `MagnificationMode`/`TrackingMode`/`ColorFilterType`/`TtsStatus` — luminos-types/src/state.rs:9,20,31,48 (tests :67-130). Re-exported via luminos-core/src/state.rs.
  - `PresentMode`/`GpuPreference`/`InterpolationMode` — luminos-types/src/gpu.rs:9,20,29 (tests :42-83).
  - `DockEdge`/`LensShape` — luminos-types/src/overlay.rs:9,22 (tests :58-86).
  - `ModelVariant`/`HotkeyAction`/`ModifierKey` — schema.rs:22,35,67 (no rename).
- `Option::None` → `null`. `keybindings` = serde `HashMap<HotkeyAction,Option<KeyBinding>>` → sparse JSON object, PascalCase action keys, value `KeyBinding`|null. TS uses `z.partialRecord` (correct — `z.record` over enum would force all keys).
- TS side mirrors EXACTLY: `ui/src/types/enums.ts` (PascalCase z.enum unions), `ui/src/types/settings.ts` (snake_case z.object keys). Variant sets match Rust enum variants 1:1.

## FrameTimingSummary asymmetry — HONEST, correctly flagged as a story-005 OBLIGATION
- `luminos-gpu/src/frame_timings.rs:50` `FrameTimingSummary` derives ONLY `Debug, Clone, PartialEq` — **NO serde, NO specta, NO rename today.** Fields snake_case: `average_ms,p99_ms,min_ms,max_ms,target_fps` (f64 x4 + u32).
- TS placeholder (`settings.ts:120`) assumes **camelCase** `{averageMs,p99Ms,minMs,maxMs,targetFps}`. This is NOT a present-state claim — it's documented (settings.ts:112-119 + HLP Shared Context DC-5 line 259, 271) as work story 005 MUST do: add serde + `#[serde(rename_all="camelCase")]` + `specta::Type`. The ONE camelCase IPC type by design (asymmetry intentional, recorded). If 005 forgets the rename → Zod reject; this is the single sharpest 005↔006 integration risk and it IS written down.

## DEFAULT_SETTINGS mirror — EXACT match to Rust AppSettings::default()
- `ui/src/constants/defaults.ts` field-by-field equals schema.rs defaults (zoom 2.0, FullScreen, Cursor, target_fps 60, Quality, LowPower, Bilinear, smooth true; color None/0.0/1.0; cursor #ff0000/#ffff0080/2/50; speech ""/Q8/1.0; minimize_to_tray true, show_panel_on_start true, start_on_login false, keybindings {}). Used as hydration-on-error fallback.

## Test count "70 passed / 70" — EXACT (reconciled)
- 68 literal `it(`/`test(` calls across 15 files. ONE is `test.each(VALID_MODES)` (enums.schema.test.ts:13) over 3 modes → 68 − 1 + 3 = **70 runtime cases.** Claim correct. (Anchored `^\s*` grep undercounts to 67/69 — use loose regex.)
- 6 real `expect(await axe(container)).toHaveNoViolations()` in accessibility.test.tsx:55-82; matcher registered in src/test/setup.ts:18. "0 axe violations" is genuinely test-backed across 6 scopes (slider, mode-selector, page, shell, app-post-hydration, error-toast).
- 98.13% stmt / 98.43% line coverage + bundle 83.1 kB gz: build/coverage outputs, NOT independently re-run (QA owns full suite) — infra present and consistent.

## Deferrals — LEGITIMATE
- `ui/src/ipc/bindings.ts:1` clearly marked PLACEHOLDER; only `commands.ts:4` + `events.ts:4` import from `./bindings`. All components/store consume `ipc/{commands,events}` → generated-file swap is genuinely ONE file. Verified via grep (no other importer).
- E2E (tauri-driver) deferral to 007 legitimate: toolchain (Tauri/webkit) absent on machine, everything verified Node-only. Matches HLP D4 future-validation column.

## globals@16.5.0 devDep — REQUIRED + SAFE
- Genuinely used: `ui/eslint.config.js:5,29,46,52` (`globals.browser`, `globals.node`) for the flat-config languageOptions. Not vestigial.
- Published 2025-11-01 (npm registry) — well outside 2-week window. Advisory bulk query → `{}` (no advisory). Newer 17.x exist; staying on 16.5.0 (no auto-upgrade) is consistent with project pin policy.

## Pointer (non-blocking)
- P-001: The entire snake_case-AppSettings + camelCase-FrameTimingSummary contract is enforced only by Zod at the TS boundary + HLP prose. There is NO automated cross-language conformance test until story 005's tauri-specta round-trip. The Rust serde roundtrip tests assert JSON shape on the Rust side; the TS schema tests assert it on the TS side; nothing yet asserts they AGREE. Story 005 must be the contract-closing check (it is named as such in DC-5). Until then, drift is caught only if a human keeps both sides in sync.
