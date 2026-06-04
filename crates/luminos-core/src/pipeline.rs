//! End-to-end input processing pipeline.
//!
//! Provides [`InputProcessingTask`], which manages a dedicated thread that
//! reads [`InputEvent`] values from a bounded channel, dispatches mouse
//! moves to [`StateManager::update_mouse_position()`], and dispatches key
//! events to [`HotkeyMatcher::match_event()`] + [`dispatch_hotkey()`].
//!
//! The [`EventNotifier`] trait abstracts over `winit::event_loop::EventLoopProxy`
//! to allow unit testing without requiring an X11 display.

use std::thread::{self, JoinHandle};

use tokio::sync::mpsc;

use luminos_platform::traits::input_monitor::InputEvent;

use crate::hotkeys::{HotkeyMatcher, dispatch_hotkey};
use crate::state_manager::StateManager;

/// Trait for notifying the event loop of state changes.
///
/// Abstracts over `winit::event_loop::EventLoopProxy<LuminosEvent>` to allow
/// unit testing without requiring an X11 display.
pub trait EventNotifier: Send + 'static {
    /// Sends a notification that application state has changed.
    fn notify_state_changed(&self);
}

impl EventNotifier for winit::event_loop::EventLoopProxy<crate::event::LuminosEvent> {
    fn notify_state_changed(&self) {
        let _ = self.send_event(crate::event::LuminosEvent::StateChanged);
    }
}

/// Manages the input processing thread that dispatches input events
/// to the state manager and hotkey handler.
///
/// The thread reads [`InputEvent`] values from the receiver, dispatches
/// mouse moves to [`StateManager::update_mouse_position()`], and dispatches
/// key events to [`HotkeyMatcher::match_event()`] + [`dispatch_hotkey()`].
/// After each state mutation, sends a notification via [`EventNotifier`].
///
/// The thread exits when the receiver channel is closed (sender dropped).
pub struct InputProcessingTask {
    /// Handle to the input processing thread.
    thread_handle: Option<JoinHandle<()>>,
}

impl InputProcessingTask {
    /// Spawns the input processing thread.
    ///
    /// The thread reads `InputEvent` values from the receiver, dispatches
    /// mouse moves to `StateManager::update_mouse_position()`, and
    /// dispatches key events to `HotkeyMatcher::match_event()` +
    /// `dispatch_hotkey()`. After each state mutation, sends a notification
    /// via the `EventNotifier`.
    ///
    /// The thread exits when the receiver channel is closed (sender dropped).
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if the OS thread cannot be spawned.
    #[must_use = "the returned task must be kept alive and joined on shutdown"]
    pub fn spawn<N: EventNotifier>(
        receiver: mpsc::Receiver<InputEvent>,
        state_manager: StateManager,
        hotkey_matcher: HotkeyMatcher,
        notifier: N,
    ) -> Result<Self, std::io::Error> {
        let handle = thread::Builder::new()
            .name("luminos-input-processor".to_string())
            .spawn(move || {
                Self::run(receiver, state_manager, hotkey_matcher, notifier);
            })?;

        Ok(Self {
            thread_handle: Some(handle),
        })
    }

    /// Waits for the input processing thread to finish.
    ///
    /// Call this during shutdown after the input monitor channel is closed.
    /// The thread will exit once `blocking_recv()` returns `None`.
    pub fn join(mut self) {
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }

    /// Runs the synchronous event processing loop.
    ///
    /// Blocks the current thread, reading events from the channel via
    /// `blocking_recv()`. This does **not** require a tokio runtime --
    /// `blocking_recv()` blocks the OS thread until a message is available
    /// or the channel closes.
    #[allow(clippy::needless_pass_by_value)] // Values are moved into the thread closure in spawn(); run() consumes them for the thread's lifetime.
    fn run<N: EventNotifier>(
        mut receiver: mpsc::Receiver<InputEvent>,
        state_manager: StateManager,
        hotkey_matcher: HotkeyMatcher,
        notifier: N,
    ) {
        loop {
            if let Some(event) = receiver.blocking_recv() {
                Self::dispatch_event(&event, &state_manager, &hotkey_matcher, &notifier);
            } else {
                log::info!("Input event channel closed, stopping input processor");
                break;
            }
        }
    }

    /// Dispatches a single input event to the appropriate handler.
    ///
    /// - `MouseMoved` -> `StateManager::update_mouse_position()` + notify
    /// - `KeyEvent` -> `HotkeyMatcher::match_event()` + `dispatch_hotkey()` + notify
    /// - `MouseButton` / `Scroll` -> ignored (not in E03 scope)
    pub(crate) fn dispatch_event<N: EventNotifier>(
        event: &InputEvent,
        state_manager: &StateManager,
        hotkey_matcher: &HotkeyMatcher,
        notifier: &N,
    ) {
        match event {
            InputEvent::MouseMoved { position } => {
                state_manager.update_mouse_position(*position);
                notifier.notify_state_changed();
            }
            InputEvent::KeyEvent { .. } => {
                if let Some(action) = hotkey_matcher.match_event(event) {
                    dispatch_hotkey(action, state_manager);
                    notifier.notify_state_changed();
                }
            }
            InputEvent::MouseButton { .. } | InputEvent::Scroll { .. } => {
                // Ignored in E03. Future epics may handle these.
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use arc_swap::ArcSwap;
    use luminos_platform::traits::input_monitor::{InputEvent, KeyCode, Modifiers, MouseButton};
    use luminos_types::ScreenPoint;

    use super::*;
    use crate::state::AppState;

    // ---------------------------------------------------------------
    // Test helpers
    // ---------------------------------------------------------------

    /// Mock notifier that counts calls.
    #[derive(Clone)]
    struct MockNotifier {
        call_count: Arc<AtomicUsize>,
    }

    impl MockNotifier {
        fn new() -> Self {
            Self {
                call_count: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn count(&self) -> usize {
            self.call_count.load(Ordering::SeqCst)
        }
    }

    impl EventNotifier for MockNotifier {
        fn notify_state_changed(&self) {
            self.call_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn generate_test_state_manager() -> StateManager {
        let shared = Arc::new(ArcSwap::from_pointee(AppState::default()));
        StateManager::new(shared)
    }

    fn generate_test_state_manager_with_zoom(zoom: f32) -> StateManager {
        let mut state = AppState::default();
        state.settings.magnification.zoom_level = zoom;
        let shared = Arc::new(ArcSwap::from_pointee(state));
        StateManager::new(shared)
    }

    fn ctrl_alt() -> Modifiers {
        Modifiers {
            shift: false,
            ctrl: true,
            alt: true,
            meta: false,
        }
    }

    // ---------------------------------------------------------------
    // T003 -- dispatch_event tests
    // ---------------------------------------------------------------

    #[test]
    fn pipeline_dispatch_mouse_moved_updates_position() {
        let mgr = generate_test_state_manager();
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        let event = InputEvent::MouseMoved {
            position: ScreenPoint { x: 500, y: 300 },
        };
        InputProcessingTask::dispatch_event(&event, &mgr, &matcher, &notifier);

        let state = mgr.load();
        assert_eq!(state.mouse_position, ScreenPoint { x: 500, y: 300 });
        assert_eq!(notifier.count(), 1, "notifier should be called once");
    }

    #[test]
    fn pipeline_dispatch_key_event_matching_hotkey_changes_zoom() {
        let mgr = generate_test_state_manager_with_zoom(2.0);
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        let event = InputEvent::KeyEvent {
            code: KeyCode::Equal,
            pressed: true,
            modifiers: ctrl_alt(),
        };
        InputProcessingTask::dispatch_event(&event, &mgr, &matcher, &notifier);

        let state = mgr.load();
        assert!(
            (state.settings.magnification.zoom_level - 3.0).abs() < f32::EPSILON,
            "zoom should be 3.0 (2.0 * 1.5), got {}",
            state.settings.magnification.zoom_level
        );
        assert_eq!(notifier.count(), 1, "notifier should be called once");
    }

    #[test]
    fn pipeline_dispatch_key_event_no_match_no_state_change() {
        let mgr = generate_test_state_manager();
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        let event = InputEvent::KeyEvent {
            code: KeyCode::A,
            pressed: true,
            modifiers: ctrl_alt(),
        };
        InputProcessingTask::dispatch_event(&event, &mgr, &matcher, &notifier);

        let state = mgr.load();
        assert!(
            (state.settings.magnification.zoom_level - 2.0).abs() < f32::EPSILON,
            "zoom should remain at default 2.0"
        );
        assert_eq!(notifier.count(), 0, "notifier should not be called");
    }

    #[test]
    fn pipeline_dispatch_mouse_button_ignored() {
        let mgr = generate_test_state_manager();
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        let event = InputEvent::MouseButton {
            button: MouseButton::Left,
            pressed: true,
            position: ScreenPoint { x: 100, y: 100 },
        };
        InputProcessingTask::dispatch_event(&event, &mgr, &matcher, &notifier);

        let state = mgr.load();
        assert_eq!(
            state.mouse_position,
            ScreenPoint { x: 0, y: 0 },
            "mouse position should remain at default"
        );
        assert_eq!(notifier.count(), 0, "notifier should not be called");
    }

    #[test]
    fn pipeline_dispatch_scroll_ignored() {
        let mgr = generate_test_state_manager();
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        let event = InputEvent::Scroll {
            delta_x: 0.0,
            delta_y: 1.0,
            position: ScreenPoint { x: 100, y: 100 },
        };
        InputProcessingTask::dispatch_event(&event, &mgr, &matcher, &notifier);

        let state = mgr.load();
        assert_eq!(
            state.mouse_position,
            ScreenPoint { x: 0, y: 0 },
            "mouse position should remain at default"
        );
        assert_eq!(notifier.count(), 0, "notifier should not be called");
    }

    #[test]
    fn pipeline_dispatch_key_release_ignored() {
        let mgr = generate_test_state_manager_with_zoom(2.0);
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        // Key release should not trigger hotkey
        let event = InputEvent::KeyEvent {
            code: KeyCode::Equal,
            pressed: false,
            modifiers: ctrl_alt(),
        };
        InputProcessingTask::dispatch_event(&event, &mgr, &matcher, &notifier);

        let state = mgr.load();
        assert!(
            (state.settings.magnification.zoom_level - 2.0).abs() < f32::EPSILON,
            "zoom should remain at 2.0 on key release"
        );
        assert_eq!(
            notifier.count(),
            0,
            "notifier should not be called on key release"
        );
    }

    #[test]
    fn pipeline_dispatch_toggle_magnification() {
        let mgr = generate_test_state_manager();
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        assert!(!mgr.load().is_active, "default should be inactive");

        let event = InputEvent::KeyEvent {
            code: KeyCode::Key8,
            pressed: true,
            modifiers: ctrl_alt(),
        };
        InputProcessingTask::dispatch_event(&event, &mgr, &matcher, &notifier);

        assert!(mgr.load().is_active, "should be active after toggle");
        assert_eq!(notifier.count(), 1);
    }

    // ---------------------------------------------------------------
    // T004 -- run() tests
    // ---------------------------------------------------------------

    #[test]
    fn pipeline_run_exits_on_channel_close() {
        let (tx, rx) = mpsc::channel::<InputEvent>(4);
        let mgr = generate_test_state_manager();
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        // Drop sender immediately
        drop(tx);

        let handle = std::thread::spawn(move || {
            InputProcessingTask::run(rx, mgr, matcher, notifier);
        });

        // Should complete quickly
        handle
            .join()
            .expect("run() thread should exit when channel closes");
    }

    #[test]
    fn pipeline_run_processes_events_until_channel_close() {
        let (tx, rx) = mpsc::channel::<InputEvent>(4);
        let mgr = generate_test_state_manager();
        let mgr_clone = mgr.clone();
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        let handle = std::thread::spawn(move || {
            InputProcessingTask::run(rx, mgr_clone, matcher, notifier);
        });

        // Send events
        tx.blocking_send(InputEvent::MouseMoved {
            position: ScreenPoint { x: 10, y: 20 },
        })
        .unwrap();
        tx.blocking_send(InputEvent::MouseMoved {
            position: ScreenPoint { x: 30, y: 40 },
        })
        .unwrap();
        tx.blocking_send(InputEvent::MouseMoved {
            position: ScreenPoint { x: 50, y: 60 },
        })
        .unwrap();

        // Drop sender to close channel
        drop(tx);

        handle.join().expect("run() thread should exit cleanly");

        // Verify last position was applied
        let state = mgr.load();
        assert_eq!(state.mouse_position, ScreenPoint { x: 50, y: 60 });
    }

    // ---------------------------------------------------------------
    // T005 -- spawn() and join() tests
    // ---------------------------------------------------------------

    #[test]
    fn pipeline_spawn_creates_task_and_join_completes() {
        let (tx, rx) = mpsc::channel::<InputEvent>(4);
        let mgr = generate_test_state_manager();
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        let task =
            InputProcessingTask::spawn(rx, mgr, matcher, notifier).expect("spawn should succeed");

        // Drop sender to close channel
        drop(tx);

        // join should complete
        task.join();
    }

    #[test]
    fn pipeline_spawn_and_dispatch_events() {
        let (tx, rx) = mpsc::channel::<InputEvent>(4);
        let mgr = generate_test_state_manager();
        let mgr_clone = mgr.clone();
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        let task = InputProcessingTask::spawn(rx, mgr_clone, matcher, notifier)
            .expect("spawn should succeed");

        tx.blocking_send(InputEvent::MouseMoved {
            position: ScreenPoint { x: 42, y: 84 },
        })
        .unwrap();

        // Give the thread time to process
        std::thread::sleep(std::time::Duration::from_millis(50));

        let state = mgr.load();
        assert_eq!(state.mouse_position, ScreenPoint { x: 42, y: 84 });

        drop(tx);
        task.join();
    }

    #[test]
    fn pipeline_join_after_immediate_channel_close() {
        let (tx, rx) = mpsc::channel::<InputEvent>(4);
        let mgr = generate_test_state_manager();
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        let task =
            InputProcessingTask::spawn(rx, mgr, matcher, notifier).expect("spawn should succeed");

        // Drop sender immediately
        drop(tx);

        // join should not hang
        task.join();
    }

    // ---------------------------------------------------------------
    // T006 -- Send assertion
    // ---------------------------------------------------------------

    #[test]
    fn pipeline_input_processing_task_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<InputProcessingTask>();
    }

    // ---------------------------------------------------------------
    // T003 -- EventNotifier trait tests
    // ---------------------------------------------------------------

    #[test]
    fn pipeline_mock_notifier_counts_calls() {
        let notifier = MockNotifier::new();
        assert_eq!(notifier.count(), 0);
        notifier.notify_state_changed();
        assert_eq!(notifier.count(), 1);
        notifier.notify_state_changed();
        assert_eq!(notifier.count(), 2);
    }
}

// ---------------------------------------------------------------
// Integration tests (ci_platform_tests)
// ---------------------------------------------------------------
#[cfg(all(test, target_os = "linux", feature = "ci_platform_tests"))]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod integration_tests {
    use std::process::Command;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    use arc_swap::ArcSwap;
    use luminos_platform::linux_x11::input::X11InputMonitor;
    use luminos_platform::traits::input_monitor::InputMonitor;

    use super::*;
    use crate::state::AppState;

    // ---------------------------------------------------------------
    // Test helpers
    // ---------------------------------------------------------------

    #[derive(Clone)]
    struct MockNotifier {
        call_count: Arc<AtomicUsize>,
    }

    impl MockNotifier {
        fn new() -> Self {
            Self {
                call_count: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    impl EventNotifier for MockNotifier {
        fn notify_state_changed(&self) {
            self.call_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn generate_test_state_manager() -> StateManager {
        let shared = Arc::new(ArcSwap::from_pointee(AppState::default()));
        StateManager::new(shared)
    }

    fn generate_test_state_manager_with_zoom(zoom: f32) -> StateManager {
        let mut state = AppState::default();
        state.settings.magnification.zoom_level = zoom;
        let shared = Arc::new(ArcSwap::from_pointee(state));
        StateManager::new(shared)
    }

    /// Returns true if xdotool is available on the system.
    fn xdotool_available() -> bool {
        Command::new("xdotool").arg("--version").output().is_ok()
    }

    /// Waits for a condition to become true, polling every `poll_interval`.
    /// Returns true if the condition was met within `timeout`.
    fn wait_for_condition<F: Fn() -> bool>(
        condition: F,
        timeout: Duration,
        poll_interval: Duration,
    ) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if condition() {
                return true;
            }
            std::thread::sleep(poll_interval);
        }
        false
    }

    // ---------------------------------------------------------------
    // T007 -- Mouse move via xdotool updates state
    // ---------------------------------------------------------------

    #[test]
    fn integration_mouse_move_updates_state() {
        if !xdotool_available() {
            eprintln!("xdotool not available, skipping integration test");
            return;
        }

        let monitor = X11InputMonitor::new().expect("X11InputMonitor should initialize");
        let rx = monitor
            .subscribe_input_events(32)
            .expect("subscribe should succeed");

        let mgr = generate_test_state_manager();
        let mgr_read = mgr.clone();
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        let task =
            InputProcessingTask::spawn(rx, mgr, matcher, notifier).expect("spawn should succeed");

        // Move mouse to known position
        let status = Command::new("xdotool")
            .args(["mousemove", "500", "300"])
            .status()
            .expect("xdotool should execute");
        assert!(status.success(), "xdotool mousemove should succeed");

        // Wait for state to update
        let found = wait_for_condition(
            || {
                let state = mgr_read.load();
                let pos = state.mouse_position;
                // Allow tolerance for Xvfb cursor warping
                (pos.x - 500).abs() <= 5 && (pos.y - 300).abs() <= 5
            },
            Duration::from_millis(500),
            Duration::from_millis(10),
        );

        assert!(
            found,
            "mouse_position should be approximately (500, 300), got {:?}",
            mgr_read.load().mouse_position
        );

        // Cleanup
        drop(monitor);
        task.join();
    }

    #[test]
    fn integration_arcswap_cross_thread_visibility() {
        if !xdotool_available() {
            eprintln!("xdotool not available, skipping integration test");
            return;
        }

        let monitor = X11InputMonitor::new().expect("X11InputMonitor should initialize");
        let rx = monitor
            .subscribe_input_events(32)
            .expect("subscribe should succeed");

        let mgr = generate_test_state_manager();
        let mgr_read = mgr.clone();
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        let task =
            InputProcessingTask::spawn(rx, mgr, matcher, notifier).expect("spawn should succeed");

        // Move mouse to a distinct position
        let status = Command::new("xdotool")
            .args(["mousemove", "750", "450"])
            .status()
            .expect("xdotool should execute");
        assert!(status.success());

        // Verify the state is visible from the test thread (ArcSwap guarantee)
        let found = wait_for_condition(
            || {
                let state = mgr_read.load();
                let pos = state.mouse_position;
                (pos.x - 750).abs() <= 5 && (pos.y - 450).abs() <= 5
            },
            Duration::from_millis(500),
            Duration::from_millis(10),
        );

        assert!(
            found,
            "ArcSwap cross-thread visibility: position should be approximately (750, 450), got {:?}",
            mgr_read.load().mouse_position
        );

        drop(monitor);
        task.join();
    }

    // ---------------------------------------------------------------
    // T008 -- Hotkey via xdotool
    // ---------------------------------------------------------------

    #[test]
    fn integration_hotkey_zoom_in() {
        if !xdotool_available() {
            eprintln!("xdotool not available, skipping integration test");
            return;
        }

        let monitor = X11InputMonitor::new().expect("X11InputMonitor should initialize");
        let rx = monitor
            .subscribe_input_events(32)
            .expect("subscribe should succeed");

        let mgr = generate_test_state_manager_with_zoom(2.0);
        let mgr_read = mgr.clone();
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        let task =
            InputProcessingTask::spawn(rx, mgr, matcher, notifier).expect("spawn should succeed");

        let status = Command::new("xdotool")
            .args(["key", "ctrl+alt+equal"])
            .status()
            .expect("xdotool should execute");
        assert!(status.success());

        let found = wait_for_condition(
            || {
                let state = mgr_read.load();
                (state.settings.magnification.zoom_level - 3.0).abs() < 0.01
            },
            Duration::from_millis(500),
            Duration::from_millis(10),
        );

        assert!(
            found,
            "zoom_level should be 3.0 (2.0 * 1.5), got {}",
            mgr_read.load().settings.magnification.zoom_level
        );

        drop(monitor);
        task.join();
    }

    #[test]
    fn integration_hotkey_toggle_magnification() {
        if !xdotool_available() {
            eprintln!("xdotool not available, skipping integration test");
            return;
        }

        let monitor = X11InputMonitor::new().expect("X11InputMonitor should initialize");
        let rx = monitor
            .subscribe_input_events(32)
            .expect("subscribe should succeed");

        let mgr = generate_test_state_manager();
        let mgr_read = mgr.clone();
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        // Default is_active = false
        assert!(!mgr_read.load().is_active);

        let task =
            InputProcessingTask::spawn(rx, mgr, matcher, notifier).expect("spawn should succeed");

        let status = Command::new("xdotool")
            .args(["key", "ctrl+alt+8"])
            .status()
            .expect("xdotool should execute");
        assert!(status.success());

        let found = wait_for_condition(
            || mgr_read.load().is_active,
            Duration::from_millis(500),
            Duration::from_millis(10),
        );

        assert!(found, "is_active should be true after toggle");

        drop(monitor);
        task.join();
    }

    #[test]
    fn integration_hotkey_zoom_out() {
        if !xdotool_available() {
            eprintln!("xdotool not available, skipping integration test");
            return;
        }

        let monitor = X11InputMonitor::new().expect("X11InputMonitor should initialize");
        let rx = monitor
            .subscribe_input_events(32)
            .expect("subscribe should succeed");

        let mgr = generate_test_state_manager_with_zoom(3.0);
        let mgr_read = mgr.clone();
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        let task =
            InputProcessingTask::spawn(rx, mgr, matcher, notifier).expect("spawn should succeed");

        let status = Command::new("xdotool")
            .args(["key", "ctrl+alt+minus"])
            .status()
            .expect("xdotool should execute");
        assert!(status.success());

        let found = wait_for_condition(
            || {
                let state = mgr_read.load();
                (state.settings.magnification.zoom_level - 2.0).abs() < 0.01
            },
            Duration::from_millis(500),
            Duration::from_millis(10),
        );

        assert!(
            found,
            "zoom_level should be 2.0 (3.0 / 1.5), got {}",
            mgr_read.load().settings.magnification.zoom_level
        );

        drop(monitor);
        task.join();
    }

    #[test]
    fn integration_hotkey_zoom_reset() {
        if !xdotool_available() {
            eprintln!("xdotool not available, skipping integration test");
            return;
        }

        let monitor = X11InputMonitor::new().expect("X11InputMonitor should initialize");
        let rx = monitor
            .subscribe_input_events(32)
            .expect("subscribe should succeed");

        let mgr = generate_test_state_manager_with_zoom(4.5);
        let mgr_read = mgr.clone();
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        let task =
            InputProcessingTask::spawn(rx, mgr, matcher, notifier).expect("spawn should succeed");

        let status = Command::new("xdotool")
            .args(["key", "ctrl+alt+0"])
            .status()
            .expect("xdotool should execute");
        assert!(status.success());

        let found = wait_for_condition(
            || {
                let state = mgr_read.load();
                (state.settings.magnification.zoom_level - 2.0).abs() < 0.01
            },
            Duration::from_millis(500),
            Duration::from_millis(10),
        );

        assert!(
            found,
            "zoom_level should be reset to 2.0, got {}",
            mgr_read.load().settings.magnification.zoom_level
        );

        drop(monitor);
        task.join();
    }

    // ---------------------------------------------------------------
    // T009 -- Graceful shutdown and ArcSwap load latency
    // ---------------------------------------------------------------

    #[test]
    fn integration_graceful_shutdown_channel_close() {
        if !xdotool_available() {
            eprintln!("xdotool not available, skipping integration test");
            return;
        }

        let monitor = X11InputMonitor::new().expect("X11InputMonitor should initialize");
        let rx = monitor
            .subscribe_input_events(32)
            .expect("subscribe should succeed");

        let mgr = generate_test_state_manager();
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        let task =
            InputProcessingTask::spawn(rx, mgr, matcher, notifier).expect("spawn should succeed");

        // Drop monitor to close the channel sender
        drop(monitor);

        // join should complete within 2 seconds (thread exits on channel close)
        let join_handle = std::thread::spawn(move || {
            task.join();
        });

        let joined = wait_for_condition(
            || join_handle.is_finished(),
            Duration::from_secs(2),
            Duration::from_millis(50),
        );

        assert!(joined, "InputProcessingTask should join within 2 seconds");
    }

    #[test]
    fn integration_arcswap_load_latency_under_100ns() {
        let mgr = generate_test_state_manager();

        // Warm up
        for _ in 0..1_000 {
            let _guard = std::hint::black_box(mgr.load());
        }

        let iterations = 1_000_000_u64;
        let start = Instant::now();
        for _ in 0..iterations {
            let _guard = std::hint::black_box(mgr.load());
        }
        let elapsed = start.elapsed();
        let avg_ns = elapsed.as_nanos() / u128::from(iterations);

        // Debug builds have higher overhead; use relaxed threshold
        let threshold_ns = if cfg!(debug_assertions) { 500 } else { 100 };
        assert!(
            avg_ns < threshold_ns,
            "load() average latency should be < {threshold_ns}ns, got {avg_ns}ns"
        );
    }

    // ---------------------------------------------------------------
    // T010 -- Frame timing under rapid mouse movement
    // ---------------------------------------------------------------

    #[test]
    fn integration_rapid_mouse_movement_no_stall() {
        if !xdotool_available() {
            eprintln!("xdotool not available, skipping integration test");
            return;
        }

        let monitor = X11InputMonitor::new().expect("X11InputMonitor should initialize");
        let rx = monitor
            .subscribe_input_events(32)
            .expect("subscribe should succeed");

        let mgr = generate_test_state_manager();
        let mgr_read = mgr.clone();
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        let task =
            InputProcessingTask::spawn(rx, mgr, matcher, notifier).expect("spawn should succeed");

        // Execute rapid mouse movements
        for i in 0..50 {
            let x = 100 + i * 30;
            let y = 100 + i * 10;
            let _ = Command::new("xdotool")
                .args(["mousemove", &x.to_string(), &y.to_string()])
                .status();
        }

        // Wait for final position to propagate
        let found = wait_for_condition(
            || {
                let state = mgr_read.load();
                let pos = state.mouse_position;
                // Last position: 100 + 49*30 = 1570, 100 + 49*10 = 590
                (pos.x - 1570).abs() <= 10 && (pos.y - 590).abs() <= 10
            },
            Duration::from_secs(1),
            Duration::from_millis(10),
        );

        assert!(
            found,
            "final mouse_position should be approximately (1570, 590), got {:?}",
            mgr_read.load().mouse_position
        );

        drop(monitor);
        task.join();
    }

    #[test]
    fn integration_mouse_event_propagation_latency() {
        if !xdotool_available() {
            eprintln!("xdotool not available, skipping integration test");
            return;
        }

        let monitor = X11InputMonitor::new().expect("X11InputMonitor should initialize");
        let rx = monitor
            .subscribe_input_events(32)
            .expect("subscribe should succeed");

        let mgr = generate_test_state_manager();
        let mgr_read = mgr.clone();
        let matcher = HotkeyMatcher::default();
        let notifier = MockNotifier::new();

        let task =
            InputProcessingTask::spawn(rx, mgr, matcher, notifier).expect("spawn should succeed");

        // First move to a known starting position to ensure state is initialized
        let _ = Command::new("xdotool")
            .args(["mousemove", "100", "100"])
            .status();
        std::thread::sleep(Duration::from_millis(100));

        // Now move to target and measure propagation latency
        let before = Instant::now();
        let status = Command::new("xdotool")
            .args(["mousemove", "800", "600"])
            .status()
            .expect("xdotool should execute");
        assert!(status.success());

        // Poll for state change with 1ms intervals
        let found = wait_for_condition(
            || {
                let state = mgr_read.load();
                let pos = state.mouse_position;
                (pos.x - 800).abs() <= 5 && (pos.y - 600).abs() <= 5
            },
            Duration::from_millis(100),
            Duration::from_millis(1),
        );

        let latency = before.elapsed();

        assert!(
            found,
            "mouse_position should be approximately (800, 600), got {:?}",
            mgr_read.load().mouse_position
        );

        // Relaxed threshold for CI: < 50ms (production target < 16.67ms)
        assert!(
            latency < Duration::from_millis(50),
            "propagation latency should be < 50ms, got {latency:?}"
        );

        drop(monitor);
        task.join();
    }
}
