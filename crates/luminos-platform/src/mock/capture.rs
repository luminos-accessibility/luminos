//! Mock implementation of [`ScreenCapture`].

use crate::traits::{
    CaptureError, CaptureFrame, DisplayInfo, ScreenCapture, ScreenRect,
    screen_capture::DisplayChangeEvent,
};
use tokio::sync::mpsc;

/// Mock implementation of `ScreenCapture` for unit testing.
///
/// Returns pre-configured display lists and capture frames.
/// Supports error injection via `with_error()` builder.
///
/// # Example
///
/// ```rust,ignore
/// use luminos_platform::mock::MockScreenCapture;
/// use luminos_platform::traits::types::test_utils::*;
///
/// let displays = vec![generate_test_display_info("test-0", 1920, 1080, true)];
/// let frame = generate_test_capture_frame(64, 48, [0, 0, 255, 255]);
/// let capture = MockScreenCapture::generate_test_mock_screen_capture(
///     displays.clone(), frame,
/// );
/// assert_eq!(capture.list_displays().unwrap(), displays);
/// ```
pub struct MockScreenCapture {
    /// Display list returned by `list_displays()`.
    displays: Vec<DisplayInfo>,
    /// Frame returned by `capture_frame()` on success.
    frame: CaptureFrame,
    /// Error factory: called to produce an error when set.
    error_factory: Option<Box<dyn Fn() -> CaptureError + Send + Sync>>,
    /// Window IDs set via `set_excluded_windows()`, for test assertions.
    excluded_window_ids: Vec<u64>,
}

impl MockScreenCapture {
    /// Creates a mock that returns fixed display info and capture frames.
    #[must_use]
    pub fn generate_test_mock_screen_capture(
        displays: Vec<DisplayInfo>,
        frame: CaptureFrame,
    ) -> Self {
        Self {
            displays,
            frame,
            error_factory: None,
            excluded_window_ids: Vec::new(),
        }
    }

    /// Configures the mock to return an error on every method call
    /// that returns `Result<_, CaptureError>`.
    ///
    /// The factory is called each time to produce a fresh error value
    /// (error types are not `Clone`).
    #[must_use]
    pub fn with_error<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> CaptureError + Send + Sync + 'static,
    {
        self.error_factory = Some(Box::new(factory));
        self
    }

    /// Returns the currently excluded window IDs (for test assertions).
    #[must_use]
    pub fn excluded_window_ids(&self) -> &[u64] {
        &self.excluded_window_ids
    }
}

impl ScreenCapture for MockScreenCapture {
    fn list_displays(&self) -> Result<Vec<DisplayInfo>, CaptureError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(self.displays.clone())
    }

    fn capture_frame(
        &self,
        display_id: &str,
        _region: Option<ScreenRect>,
    ) -> Result<CaptureFrame, CaptureError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        // Validate display_id even in mock -- matches real backend behavior
        if !self.displays.iter().any(|d| d.id == display_id) {
            return Err(CaptureError::DisplayNotFound(display_id.to_string()));
        }
        Ok(self.frame.clone())
    }

    fn subscribe_display_changes(
        &self,
        buffer_size: usize,
    ) -> Result<mpsc::Receiver<DisplayChangeEvent>, CaptureError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        // Return an empty channel -- no real display changes in mock mode
        let (_tx, rx) = mpsc::channel(buffer_size);
        Ok(rx)
    }

    fn set_excluded_windows(&mut self, window_ids: &[u64]) {
        self.excluded_window_ids = window_ids.to_vec();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::traits::types::test_utils::*;

    #[test]
    fn mock_screen_capture_list_displays_returns_configured_displays() {
        let displays = vec![
            generate_test_display_info("test-0", 1920, 1080, true),
            generate_test_display_info("test-1", 2560, 1440, false),
        ];
        let frame = generate_test_capture_frame(64, 48, [0, 0, 255, 255]);
        let capture = MockScreenCapture::generate_test_mock_screen_capture(displays.clone(), frame);

        let result = capture.list_displays().unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "test-0");
        assert!(result[0].is_primary);
        assert_eq!(result[1].id, "test-1");
        assert!(!result[1].is_primary);
    }

    #[test]
    fn mock_screen_capture_capture_frame_returns_configured_frame() {
        let displays = vec![generate_test_display_info("test-0", 1920, 1080, true)];
        let frame = generate_test_capture_frame(64, 48, [0, 0, 255, 255]);
        let capture = MockScreenCapture::generate_test_mock_screen_capture(displays, frame.clone());

        let result = capture.capture_frame("test-0", None).unwrap();
        assert_eq!(result.width, frame.width);
        assert_eq!(result.height, frame.height);
        assert_eq!(result.stride, frame.stride);
        assert_eq!(result.format, frame.format);
        assert_eq!(result.data.len(), frame.data.len());
    }

    #[test]
    fn mock_screen_capture_capture_frame_unknown_display_returns_not_found() {
        let displays = vec![generate_test_display_info("test-0", 1920, 1080, true)];
        let frame = generate_test_capture_frame(64, 48, [0, 0, 0, 255]);
        let capture = MockScreenCapture::generate_test_mock_screen_capture(displays, frame);

        let result = capture.capture_frame("nonexistent", None);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CaptureError::DisplayNotFound(id) if id == "nonexistent"
        ));
    }

    #[test]
    fn mock_screen_capture_capture_frame_with_error_returns_injected_error() {
        let displays = vec![generate_test_display_info("test-0", 1920, 1080, true)];
        let frame = generate_test_capture_frame(64, 48, [0, 0, 0, 255]);
        let capture = MockScreenCapture::generate_test_mock_screen_capture(displays, frame)
            .with_error(|| CaptureError::PermissionDenied);

        let result = capture.capture_frame("test-0", None);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CaptureError::PermissionDenied
        ));
    }

    #[test]
    fn mock_screen_capture_list_displays_with_error_returns_injected_error() {
        let displays = vec![generate_test_display_info("test-0", 1920, 1080, true)];
        let frame = generate_test_capture_frame(64, 48, [0, 0, 0, 255]);
        let capture = MockScreenCapture::generate_test_mock_screen_capture(displays, frame)
            .with_error(|| CaptureError::PermissionDenied);

        let result = capture.list_displays();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CaptureError::PermissionDenied
        ));
    }

    #[test]
    fn mock_screen_capture_subscribe_display_changes_returns_channel() {
        let displays = vec![generate_test_display_info("test-0", 1920, 1080, true)];
        let frame = generate_test_capture_frame(64, 48, [0, 0, 0, 255]);
        let capture = MockScreenCapture::generate_test_mock_screen_capture(displays, frame);

        let result = capture.subscribe_display_changes(16);
        assert!(result.is_ok());
    }

    #[test]
    fn mock_screen_capture_set_excluded_windows_stores_ids() {
        let displays = vec![generate_test_display_info("test-0", 1920, 1080, true)];
        let frame = generate_test_capture_frame(64, 48, [0, 0, 0, 255]);
        let mut capture = MockScreenCapture::generate_test_mock_screen_capture(displays, frame);

        capture.set_excluded_windows(&[42, 99]);
        assert_eq!(capture.excluded_window_ids(), &[42, 99]);
    }

    #[test]
    fn mock_screen_capture_set_excluded_windows_clear() {
        let displays = vec![generate_test_display_info("test-0", 1920, 1080, true)];
        let frame = generate_test_capture_frame(64, 48, [0, 0, 0, 255]);
        let mut capture = MockScreenCapture::generate_test_mock_screen_capture(displays, frame);

        capture.set_excluded_windows(&[42, 99]);
        assert_eq!(capture.excluded_window_ids(), &[42, 99]);
        capture.set_excluded_windows(&[]);
        assert!(capture.excluded_window_ids().is_empty());
    }

    #[test]
    fn mock_screen_capture_subscribe_display_changes_with_error() {
        let displays = vec![generate_test_display_info("test-0", 1920, 1080, true)];
        let frame = generate_test_capture_frame(64, 48, [0, 0, 0, 255]);
        let capture = MockScreenCapture::generate_test_mock_screen_capture(displays, frame)
            .with_error(|| CaptureError::PermissionDenied);

        let result = capture.subscribe_display_changes(16);
        assert!(result.is_err());
    }
}
