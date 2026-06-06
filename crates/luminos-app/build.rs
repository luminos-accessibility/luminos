//! Build script for the Luminos application crate.
//!
//! When the `tauri` feature is enabled, this runs `tauri_build::build()` to
//! generate the Tauri context (parsing `tauri.conf.json`, embedding the
//! frontend dist, and emitting capability/permission metadata). Without the
//! feature it is a no-op so that the crate skeleton still builds on machines
//! lacking the webview system libraries.

fn main() {
    #[cfg(feature = "tauri")]
    tauri_build::build();
}
