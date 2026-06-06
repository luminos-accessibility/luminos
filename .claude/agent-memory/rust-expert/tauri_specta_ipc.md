# tauri-specta rc.25 IPC bindings (E04/005, verified 2026-06-05)

Source-verified against unpacked `tauri-specta-2.0.0-rc.25` + `specta-typescript-0.0.12` crate sources.

## Event derive — `event_name` is MANDATORY
- `#[derive(tauri_specta::Event)]` with NO `event_name` kebab-cases the struct ident:
  `ZoomChangedEvent` → wire name `"zoom-changed-event"`. Breaks any contract expecting `"zoom_changed"`.
- Pin the wire name: `#[tauri_specta(event_name = "zoom_changed")]`.
- Generated TS event object key = `name.to_lower_camel_case()` of the event NAME →
  `zoom_changed` → `events.zoomChanged` (NOT `zoomChangedEvent`). Wire name stays `zoom_changed`.

## Builder API
- `tauri_specta::Builder::<tauri::Wry>::new().commands(collect_commands![path::fn,...]).events(collect_events![Type,...])`.
- `ErrorHandlingMode::Result` is the DEFAULT (`{status:"ok",data}|{status:"error",error}`); do NOT set Throw.
- Wire: `.invoke_handler(ipc.invoke_handler())` on `tauri::Builder`; `ipc.mount_events(app)` inside `.setup`.
- `ipc.export(specta_typescript::Typescript::default(), path)` — pure codegen, runs without a live app/window.

## Generated bindings.ts SWAP GOTCHAS (vs a hand-authored placeholder)
1. **No named `Result` export.** The envelope is inlined per command (`typedError<T,E>`). A wrapper that
   `import { type Result } from './bindings'` FAILS tsc — define `Result<T,E>` locally in the wrapper.
2. **`f32`/`f64` → `number | null`** by default (specta-typescript `primitives.rs`: JSON NaN/Inf → null).
   Breaks Zod `z.number()` / `(level: number)` callbacks. Fix: `Builder.semantic_types(
   specta_typescript::semantic::Configuration::default().enable_lossless_floats())` flattens to `number`.
3. **Export path resolves against process CWD** if relative. Anchor to `env!("CARGO_MANIFEST_DIR")` so
   `--export-bindings`/`cargo run`/`tauri dev`/the binary all land in the same tree. The app crate is two
   levels under repo root → `../../ui/src/ipc/bindings.ts`.
4. Generated file uses tabs; prettier wants to reformat it → add it to `.prettierignore` AND eslint ignores,
   or the CI `git diff --exit-code` bindings check breaks. The DoD `lint` gate is `eslint .` (already ignores it).

## specta dep wiring (DC-5)
- `specta = "=2.0.0-rc.25"` (features `["derive"]`) — lockstep with tauri-specta rc.25 (already transitive).
- Add as NORMAL (non-optional, non-feature-gated) to engine crates so derives compile WITHOUT the app's
  `tauri` feature → `cargo build --workspace --exclude luminos-app` must pass.
- The app crate ALSO needs `specta` directly (optional, under `tauri`): `#[specta::specta]`,
  `derive(specta::Type)` on events, `collect_commands!`/`collect_events!` all reference `specta` in scope.
  Symptom if missing: "could not find `specta` in the list of imported crates" + a confusing
  "`tauri_specta::Event` is a sealed trait / `specta::Type` not accessible" error.
- specta mirrors serde's repr: `#[serde(rename_all="camelCase")]` → camelCase TS; bare enums → PascalCase
  string-literal unions. Apply rename ONLY where the wire contract wants it (Luminos: FrameTimingSummary only).

## CI seam
- Windowless regenerate: add a `--export-bindings` flag to `main.rs` calling
  `ipc::export_bindings_to_default_path()` then exit. CI: `cargo run -p luminos-app --features tauri --
  --export-bindings && git diff --exit-code ui/src/ipc/bindings.ts`. Idempotent. No Xvfb/webview needed.
