//! Mock implementation of [`WindowManager`].

use crate::traits::{OverlayMode, ScreenRect, WindowError, WindowManager};

/// Mock implementation of `WindowManager` for unit testing.
///
/// All methods succeed by default. `raw_window_handle()` and
/// `raw_display_handle()` return `None` (no real window exists).
pub struct MockWindowManager {
    /// Whether `create_overlay` has been called.
    overlay_created: bool,
    /// Error factory for error injection.
    error_factory: Option<Box<dyn Fn() -> WindowError + Send + Sync>>,
}

impl MockWindowManager {
    /// Creates a mock window manager with default (success) behavior.
    pub fn generate_test_mock_window_manager() -> Self {
        Self {
            overlay_created: false,
            error_factory: None,
        }
    }

    /// Configures the mock to return an error on every method call
    /// that returns `Result<_, WindowError>`.
    pub fn with_error<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> WindowError + Send + Sync + 'static,
    {
        self.error_factory = Some(Box::new(factory));
        self
    }
}

impl WindowManager for MockWindowManager {
    fn create_overlay(&mut self, _display_id: &str) -> Result<(), WindowError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        self.overlay_created = true;
        Ok(())
    }

    fn set_overlay_bounds(&self, _bounds: ScreenRect) -> Result<(), WindowError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn set_overlay_mode(&mut self, _mode: OverlayMode) -> Result<(), WindowError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn set_always_on_top(&self, _always_on_top: bool) -> Result<(), WindowError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn set_visible(&self, _visible: bool) -> Result<(), WindowError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(())
    }

    fn raw_window_handle(&self) -> Option<&dyn raw_window_handle::HasWindowHandle> {
        // No real window exists in mock mode
        None
    }

    fn raw_display_handle(&self) -> Option<&dyn raw_window_handle::HasDisplayHandle> {
        // No real display exists in mock mode
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::window_manager::OverlayMode;

    #[test]
    fn mock_window_manager_create_overlay_success() {
        let mut wm = MockWindowManager::generate_test_mock_window_manager();

        let result = wm.create_overlay("display-0");
        assert!(result.is_ok());
    }

    #[test]
    fn mock_window_manager_create_overlay_with_error_returns_injected_error() {
        let mut wm = MockWindowManager::generate_test_mock_window_manager().with_error(|| {
            WindowError::CreationFailed {
                message: "test".into(),
            }
        });

        let result = wm.create_overlay("display-0");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("test"));
    }

    #[test]
    fn mock_window_manager_set_overlay_bounds_success() {
        let wm = MockWindowManager::generate_test_mock_window_manager();
        let rect = ScreenRect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        };

        let result = wm.set_overlay_bounds(rect);
        assert!(result.is_ok());
    }

    #[test]
    fn mock_window_manager_set_overlay_mode_success() {
        let mut wm = MockWindowManager::generate_test_mock_window_manager();

        let result = wm.set_overlay_mode(OverlayMode::FullScreen);
        assert!(result.is_ok());
    }

    #[test]
    fn mock_window_manager_set_always_on_top_success() {
        let wm = MockWindowManager::generate_test_mock_window_manager();

        let result = wm.set_always_on_top(true);
        assert!(result.is_ok());
    }

    #[test]
    fn mock_window_manager_set_visible_success() {
        let wm = MockWindowManager::generate_test_mock_window_manager();

        let result = wm.set_visible(true);
        assert!(result.is_ok());
    }

    #[test]
    fn mock_window_manager_raw_window_handle_returns_none() {
        let wm = MockWindowManager::generate_test_mock_window_manager();

        assert!(wm.raw_window_handle().is_none());
    }

    #[test]
    fn mock_window_manager_raw_display_handle_returns_none() {
        let wm = MockWindowManager::generate_test_mock_window_manager();

        assert!(wm.raw_display_handle().is_none());
    }
}
