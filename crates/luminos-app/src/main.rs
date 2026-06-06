//! Luminos application entry point.
//!
//! Initializes logging and runs the single tao/Tauri event loop that hosts the
//! control-panel webview and the transparent, click-through magnification
//! overlay (see `luminos_app::app::run`).

// Prevent the extra console window on Windows release builds.
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[cfg(feature = "tauri")]
fn main() {
    // Write logs through an auto-flushing stderr adapter so the structured
    // markers (`redraw=N`, `shutdown=clean`, …) reach the stream in real time
    // even when stderr is redirected to a file (block-buffered by default),
    // which the subprocess integration tests rely on.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .target(env_logger::Target::Pipe(Box::new(FlushingStderr)))
        .init();

    // CI seam (story 005 T009): `--export-bindings` regenerates
    // `ui/src/ipc/bindings.ts` and exits, WITHOUT opening a window or starting
    // the event loop. The CI `test-app` job runs this then `git diff
    // --exit-code` on the file to fail on stale committed bindings — no Xvfb or
    // webview needed.
    if std::env::args().any(|arg| arg == "--export-bindings") {
        match luminos_app::ipc::export_bindings_to_default_path() {
            Ok(()) => {
                log::info!("bindings exported");
                return;
            }
            Err(e) => {
                log::error!("bindings export failed: '{e}'");
                std::process::exit(1);
            }
        }
    }

    if let Err(e) = luminos_app::app::run() {
        log::error!("fatal: '{e}'");
        std::process::exit(1);
    }
}

/// `io::Write` adapter that flushes after every write, so redirected stderr is
/// not block-buffered (keeps the subprocess test log real-time).
#[cfg(feature = "tauri")]
struct FlushingStderr;

#[cfg(feature = "tauri")]
impl std::io::Write for FlushingStderr {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = std::io::stderr().write(buf)?;
        std::io::stderr().flush()?;
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        std::io::stderr().flush()
    }
}

/// Without the `tauri` feature the binary cannot run (the webview backend is
/// unavailable). Build with `--features tauri` (the crate default).
#[cfg(not(feature = "tauri"))]
fn main() {
    eprintln!("luminos-app must be built with the `tauri` feature enabled");
    std::process::exit(1);
}
