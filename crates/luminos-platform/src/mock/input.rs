//! Mock implementation of [`InputMonitor`].

use crate::traits::{InputError, InputMonitor, ScreenPoint, input_monitor::InputEvent};
use tokio::sync::mpsc;

/// Mock implementation of `InputMonitor` for unit testing.
///
/// Returns a pre-configured mouse position. The event subscription
/// returns an empty channel (no real input events in mock mode).
pub struct MockInputMonitor {
    /// Mouse position returned by `get_mouse_position()`.
    mouse_position: ScreenPoint,
    /// Error factory for error injection.
    error_factory: Option<Box<dyn Fn() -> InputError + Send + Sync>>,
}

impl MockInputMonitor {
    /// Creates a mock with a pre-configured mouse position.
    pub fn generate_test_mock_input_monitor(mouse_position: ScreenPoint) -> Self {
        Self {
            mouse_position,
            error_factory: None,
        }
    }

    /// Configures the mock to return an error on every method call.
    pub fn with_error<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> InputError + Send + Sync + 'static,
    {
        self.error_factory = Some(Box::new(factory));
        self
    }
}

impl InputMonitor for MockInputMonitor {
    fn subscribe_input_events(
        &self,
        buffer_size: usize,
    ) -> Result<mpsc::Receiver<InputEvent>, InputError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        let (_tx, rx) = mpsc::channel(buffer_size);
        Ok(rx)
    }

    fn get_mouse_position(&self) -> Result<ScreenPoint, InputError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(self.mouse_position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_input_monitor_get_mouse_position_returns_configured_position() {
        let monitor =
            MockInputMonitor::generate_test_mock_input_monitor(ScreenPoint { x: 100, y: 200 });

        let result = monitor.get_mouse_position().unwrap();
        assert_eq!(result, ScreenPoint { x: 100, y: 200 });
    }

    #[test]
    fn mock_input_monitor_subscribe_input_events_returns_channel() {
        let monitor =
            MockInputMonitor::generate_test_mock_input_monitor(ScreenPoint { x: 0, y: 0 });

        let result = monitor.subscribe_input_events(32);
        assert!(result.is_ok());
    }

    #[test]
    fn mock_input_monitor_subscribe_with_error_returns_injected_error() {
        let monitor =
            MockInputMonitor::generate_test_mock_input_monitor(ScreenPoint { x: 0, y: 0 })
                .with_error(|| InputError::Unavailable {
                    reason: "denied".into(),
                });

        let result = monitor.subscribe_input_events(32);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("denied"));
    }

    #[test]
    fn mock_input_monitor_get_mouse_position_with_error() {
        let monitor =
            MockInputMonitor::generate_test_mock_input_monitor(ScreenPoint { x: 0, y: 0 })
                .with_error(|| InputError::Unavailable {
                    reason: "denied".into(),
                });

        let result = monitor.get_mouse_position();
        assert!(result.is_err());
    }
}
