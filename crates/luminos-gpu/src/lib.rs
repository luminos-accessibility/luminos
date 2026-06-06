//! GPU-accelerated rendering pipeline for Luminos.
//!
//! Implements screen magnification via wgpu shaders and frame-capture
//! processing. The magnification overlay surface itself is owned by the
//! tao/Tauri application shell (`luminos-app`), not this crate; this crate
//! renders into a `wgpu::Surface` the app supplies (E04 single-event-loop
//! design, see `specs/E04-tauri-control-panel/`).

pub mod device;
pub mod error;
pub mod frame_timings;
pub mod renderer;
pub mod shaders;
pub mod surface;
pub mod texture;
pub mod viewport;

// Convenience crate-root re-exports for the application shell (E04/003). The
// loop wiring in `luminos-app` references these by short path; without the
// re-exports it would reach through `renderer::`, `frame_timings::`, and
// `shaders::` module paths (DC-5 / story-003 §D.6).
pub use frame_timings::{FrameTimingSummary, FrameTimings};
pub use renderer::Renderer;
pub use shaders::InterpolationMethod;
pub use viewport::compute_source_region;
