//! `tauri-specta` IPC handler: collects the seven Phase-0 commands and the two
//! engine->panel events, and (in debug builds) exports the TypeScript bindings
//! to `ui/src/ipc/bindings.ts` (story E04/005).
//!
//! The [`Builder`] is the single source of truth for the IPC surface:
//! - `invoke_handler()` is wired onto the `tauri::Builder` so the commands are
//!   reachable from the webview;
//! - `mount_events(app)` registers the events so `.emit(app)` reaches the
//!   webview's listeners;
//! - `export(...)` regenerates `bindings.ts` in debug builds (the CI
//!   bindings-up-to-date check diffs the committed file against a fresh export).
//!
//! Error-handling mode is `tauri-specta`'s default ([`ErrorHandlingMode::Result`]),
//! matching the `{ status: "ok" | "error" }` envelope the story-006 wrappers
//! consume — do NOT switch to `Throw`.

use tauri_specta::{Builder, collect_commands, collect_events};

use crate::events::{ModeChangedEvent, ZoomChangedEvent};
use crate::tauri_commands;

/// Builds the `tauri-specta` [`Builder`] holding the Phase-0 IPC surface.
///
/// The command list order is the bindings' generation order; keep it stable to
/// minimise churn in the generated `bindings.ts` (the CI diff check is
/// order-sensitive).
///
/// `semantic_types(enable_lossless_floats)` flattens `f32`/`f64` from the
/// `number | null` default to plain `number` in the generated TypeScript. The
/// engine never emits `NaN`/`Infinity` over IPC (zoom NaN is rejected
/// server-side; the timing fields are finite), so the flattened `number` type is
/// accurate and — critically — matches story 006's Zod schemas (`z.number()`)
/// and the `onZoomChanged(level: number)` wrapper, keeping the cross-language
/// contract green. Without it, `ZoomChangedEvent`/`zoom_level`/`averageMs`/… all
/// type as `number | null` and break the frontend's `tsc`.
pub(crate) fn build_ipc_handler() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            tauri_commands::get_current_settings,
            tauri_commands::get_frame_timings,
            tauri_commands::set_zoom_level,
            tauri_commands::set_magnification_mode,
            tauri_commands::toggle_magnification,
            tauri_commands::save_settings,
            tauri_commands::reset_settings,
        ])
        .events(collect_events![ZoomChangedEvent, ModeChangedEvent])
        .semantic_types(
            specta_typescript::semantic::Configuration::default().enable_lossless_floats(),
        )
}

/// Resolves the absolute path the bindings are exported to.
///
/// The crate lives two levels under the repo root (`crates/luminos-app/`), so
/// the UI tree is `../../ui/...` relative to the crate manifest dir (the
/// `DESIGN`'s `../ui/...` is stale — `IMPLEMENTATION_NOTES` §D). The path is
/// anchored to `CARGO_MANIFEST_DIR` (a compile-time constant) rather than the
/// process CWD, so the export lands in the right tree no matter where the binary
/// is launched from (running the binary directly, `cargo run`, `tauri dev`, or
/// the CI seam all differ in CWD).
#[must_use]
fn bindings_export_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ui/src/ipc/bindings.ts")
}

/// Exports the TypeScript bindings to [`bindings_export_path`] (debug only).
///
/// Called from `app::run`'s setup in debug builds; release builds skip export
/// entirely (the committed `bindings.ts` is authoritative at runtime).
///
/// # Errors
/// Returns the `specta-typescript` export error if writing the file fails.
#[cfg(debug_assertions)]
pub(crate) fn export_bindings(builder: &Builder<tauri::Wry>) -> Result<(), String> {
    builder
        .export(
            specta_typescript::Typescript::default(),
            bindings_export_path(),
        )
        .map_err(|e| e.to_string())
}

/// Builds the handler and exports `bindings.ts` to the canonical path, without
/// starting the event loop or opening any window.
///
/// This is the deterministic CI seam (story 005 T009): the `--export-bindings`
/// flag in `main` calls it so CI can run a fresh export and `git diff
/// --exit-code ui/src/ipc/bindings.ts` to fail on stale committed bindings — no
/// Xvfb/webview required. Available in all build profiles so the CI job, which
/// builds the app normally, can invoke it.
///
/// # Errors
/// Returns the `specta-typescript` export error if writing the file fails.
pub fn export_bindings_to_default_path() -> Result<(), String> {
    build_ipc_handler()
        .export(
            specta_typescript::Typescript::default(),
            bindings_export_path(),
        )
        .map_err(|e| e.to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    /// T007 / AC-3.1: exporting the handler produces a `bindings.ts` containing
    /// all seven Phase-0 commands and both events (by their wire names). This is
    /// pure codegen — no Tauri runtime needed — so it doubles as the smoke test
    /// the CI bindings-diff check builds on.
    #[test]
    fn bindings_export_smoke() {
        let builder = build_ipc_handler();
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("bindings.ts");
        builder
            .export(specta_typescript::Typescript::default(), &out)
            .unwrap();
        let generated = std::fs::read_to_string(&out).unwrap();

        // All seven Tauri command names (snake_case wire names) appear.
        for cmd in [
            "get_current_settings",
            "get_frame_timings",
            "set_zoom_level",
            "set_magnification_mode",
            "toggle_magnification",
            "save_settings",
            "reset_settings",
        ] {
            assert!(
                generated.contains(cmd),
                "generated bindings should reference command '{cmd}'"
            );
        }

        // Both events appear by their explicit wire names (NOT the kebab-cased
        // default) and by their camelCased object keys.
        assert!(
            generated.contains("zoom_changed"),
            "missing event wire name 'zoom_changed'"
        );
        assert!(
            generated.contains("mode_changed"),
            "missing event wire name 'mode_changed'"
        );
        assert!(
            generated.contains("zoomChanged"),
            "missing event key 'zoomChanged'"
        );
        assert!(
            generated.contains("modeChanged"),
            "missing event key 'modeChanged'"
        );

        // The frame-timing summary is camelCase; AppSettings stays snake_case.
        assert!(
            generated.contains("averageMs"),
            "FrameTimingSummary should be camelCase"
        );
        assert!(
            generated.contains("zoom_level"),
            "AppSettings should stay snake_case"
        );
    }

    /// T008 / AC-3.1 / FR-8: the Tauri capability grants only the minimal,
    /// least-privilege permission set and scopes both Phase-0 windows.
    ///
    /// `shell:allow-open` is intentionally absent in Phase 0 — no command/UI
    /// opens an external shell, and granting it would require registering the
    /// `tauri-plugin-shell` plugin (a new dependency) purely for an unused
    /// permission. Deferred until a real shell-open need lands (SUBTASKS
    /// Deviations). The capability stays `core:default` + `core:event:default`,
    /// the latter so the webview may listen for the zoom/mode events (FR-6).
    #[test]
    fn capability_minimal() {
        let raw = include_str!("../capabilities/default.json");
        let json: serde_json::Value = serde_json::from_str(raw).unwrap();

        let permissions: Vec<&str> = json["permissions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(
            permissions,
            vec!["core:default", "core:event:default"],
            "Phase-0 capability must grant only core:default + core:event:default \
             (no fs/http; shell:allow-open deferred — see SUBTASKS Deviations)"
        );

        let windows: Vec<&str> = json["windows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(
            windows,
            vec!["control-panel", "overlay"],
            "capability must scope both Phase-0 windows (real labels, not 'main')"
        );

        // Explicitly reject the broad permissions RISK-020 warns against.
        for forbidden in ["fs:default", "http:default", "shell:allow-open"] {
            assert!(
                !permissions.contains(&forbidden),
                "capability must NOT grant '{forbidden}' in Phase 0"
            );
        }
    }
}
