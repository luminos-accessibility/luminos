// Linux X11 backend (E2+).

pub(crate) mod capture;
pub mod window;

// XcbCapture is not yet wired into PlatformBackends (pending Story 005).
// Suppress dead_code warning until then.
#[allow(unused_imports)]
pub use capture::XcbCapture;
// X11WindowManager is not yet wired into PlatformBackends (pending Story 002 integration).
#[allow(unused_imports)]
pub use window::X11WindowManager;
