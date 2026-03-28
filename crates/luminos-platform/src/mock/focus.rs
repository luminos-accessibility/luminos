//! Mock implementation of [`FocusTracker`].

use crate::traits::{FocusError, FocusTracker, ScreenRect, focus_tracker::FocusChangedEvent};
use tokio::sync::mpsc;

/// Mock implementation of `FocusTracker` for unit testing.
///
/// Returns a pre-configured focused element. Supports error injection.
pub struct MockFocusTracker {
    /// The focused element returned by `get_focused_element()`.
    focused_element: Option<FocusChangedEvent>,
    /// Error factory for error injection.
    error_factory: Option<Box<dyn Fn() -> FocusError + Send + Sync>>,
}

impl MockFocusTracker {
    /// Creates a mock with an optional pre-configured focused element.
    pub fn generate_test_mock_focus_tracker(focused_element: Option<FocusChangedEvent>) -> Self {
        Self {
            focused_element,
            error_factory: None,
        }
    }

    /// Configures the mock to return an error on every method call.
    pub fn with_error<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> FocusError + Send + Sync + 'static,
    {
        self.error_factory = Some(Box::new(factory));
        self
    }
}

impl FocusTracker for MockFocusTracker {
    fn subscribe_focus_changes(
        &self,
        buffer_size: usize,
    ) -> Result<mpsc::Receiver<FocusChangedEvent>, FocusError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        let (_tx, rx) = mpsc::channel(buffer_size);
        Ok(rx)
    }

    fn get_focused_element(&self) -> Result<Option<FocusChangedEvent>, FocusError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        Ok(self.focused_element.clone())
    }

    fn get_element_bounds(&self, _element_id: &str) -> Result<Option<ScreenRect>, FocusError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        // Return the bounds of the focused element if it exists
        Ok(self.focused_element.as_ref().map(|e| e.bounds))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::focus_tracker::ElementType;

    fn generate_test_focus_event() -> FocusChangedEvent {
        FocusChangedEvent {
            element_id: "elem-1".to_string(),
            bounds: ScreenRect {
                x: 100,
                y: 200,
                width: 300,
                height: 50,
            },
            element_type: ElementType::TextInput,
            label: Some("Username".to_string()),
            pid: Some(1234),
        }
    }

    #[test]
    fn mock_focus_tracker_get_focused_element_returns_configured_element() {
        let event = generate_test_focus_event();
        let tracker = MockFocusTracker::generate_test_mock_focus_tracker(Some(event.clone()));

        let result = tracker.get_focused_element().unwrap();
        assert!(result.is_some());
        let element = result.unwrap();
        assert_eq!(element.element_id, "elem-1");
        assert_eq!(element.bounds.x, 100);
        assert_eq!(element.bounds.y, 200);
    }

    #[test]
    fn mock_focus_tracker_get_focused_element_returns_none_when_unconfigured() {
        let tracker = MockFocusTracker::generate_test_mock_focus_tracker(None);

        let result = tracker.get_focused_element().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn mock_focus_tracker_subscribe_focus_changes_returns_channel() {
        let tracker = MockFocusTracker::generate_test_mock_focus_tracker(None);

        let result = tracker.subscribe_focus_changes(16);
        assert!(result.is_ok());
    }

    #[test]
    fn mock_focus_tracker_subscribe_with_error_returns_injected_error() {
        let tracker = MockFocusTracker::generate_test_mock_focus_tracker(None).with_error(|| {
            FocusError::ApiUnavailable {
                reason: "test".into(),
            }
        });

        let result = tracker.subscribe_focus_changes(16);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("test"));
    }

    #[test]
    fn mock_focus_tracker_get_element_bounds_returns_configured_bounds() {
        let event = generate_test_focus_event();
        let expected_bounds = event.bounds;
        let tracker = MockFocusTracker::generate_test_mock_focus_tracker(Some(event));

        let result = tracker.get_element_bounds("any-id").unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), expected_bounds);
    }

    #[test]
    fn mock_focus_tracker_get_element_bounds_with_error() {
        let tracker = MockFocusTracker::generate_test_mock_focus_tracker(None).with_error(|| {
            FocusError::ApiUnavailable {
                reason: "denied".into(),
            }
        });

        let result = tracker.get_element_bounds("elem-1");
        assert!(result.is_err());
    }
}
