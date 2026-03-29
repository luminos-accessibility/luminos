// Linux X11 backend (E2+).

pub(crate) mod capture;
pub mod input;
pub mod keymap;
pub mod window;

// XcbCapture is not yet wired into PlatformBackends (pending Story 005).
// Suppress dead_code warning until then.
#[allow(unused_imports)]
pub use capture::XcbCapture;
pub use input::X11InputMonitor;
// X11WindowManager is not yet wired into PlatformBackends (pending Story 002 integration).
#[allow(unused_imports)]
pub use window::X11WindowManager;
