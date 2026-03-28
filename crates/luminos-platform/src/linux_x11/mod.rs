// Linux X11 backend (E2+).

pub(crate) mod capture;

// XcbCapture is not yet wired into PlatformBackends (pending Story 005).
// Suppress dead_code warning until then.
#[allow(unused_imports)]
pub use capture::XcbCapture;
