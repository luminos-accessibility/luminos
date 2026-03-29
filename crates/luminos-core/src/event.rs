//! Custom event types for inter-thread communication with the winit event loop.
//!
//! Defines [`LuminosEvent`], the custom event enum sent via
//! `EventLoopProxy<LuminosEvent>` from input/control threads to wake the
//! render loop.

/// Custom event type for inter-thread communication with the winit event loop.
///
/// Sent via `EventLoopProxy<LuminosEvent>` from input/control threads to
/// wake the render loop. The winit event loop receives these as
/// `Event::UserEvent(LuminosEvent)`.
#[derive(Debug, Clone)]
pub enum LuminosEvent {
    /// Application state was updated (mouse position, zoom, mode, etc.).
    ///
    /// The render loop should call `window.request_redraw()` to render
    /// the next frame with the updated state.
    StateChanged,

    /// Graceful shutdown requested.
    ///
    /// The render loop should stop the input monitor, clean up resources,
    /// and exit the event loop.
    RequestExit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn luminos_event_state_changed_debug() {
        let event = LuminosEvent::StateChanged;
        let debug = format!("{event:?}");
        assert!(
            debug.contains("StateChanged"),
            "Debug output should contain 'StateChanged', got: '{debug}'"
        );
    }

    #[test]
    fn luminos_event_request_exit_debug() {
        let event = LuminosEvent::RequestExit;
        let debug = format!("{event:?}");
        assert!(
            debug.contains("RequestExit"),
            "Debug output should contain 'RequestExit', got: '{debug}'"
        );
    }

    #[test]
    fn luminos_event_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<LuminosEvent>();
    }

    #[test]
    fn luminos_event_is_clone() {
        let event = LuminosEvent::StateChanged;
        let cloned = event.clone();
        assert!(matches!(cloned, LuminosEvent::StateChanged));
    }
}
