//! tao/Tauri-backed wake mechanism for the single event loop.
//!
//! Tauri's `WebviewWindow` has no `request_redraw()` and `App::run` exposes no
//! `ControlFlow`/`Poll`, so a redraw cannot be marshaled the way winit does.
//! Instead [`AppNotifier`] holds a shared `Arc<AtomicBool>` "render-dirty" flag:
//! [`AppNotifier::notify_state_changed`] sets it, and the `App::run` callback
//! reads-and-clears it each `MainEventsCleared` (see `main.rs`). The flag is
//! `Send + Sync`, so input/IPC worker threads (stories 003/005) set it with no
//! main-thread marshaling.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use luminos_core::pipeline::EventNotifier;

/// A clonable [`EventNotifier`] that wakes the tao/Tauri loop by setting a
/// shared dirty flag. The loop owns a clone of the same `Arc<AtomicBool>` and
/// drains it via `swap(false, Acquire)`.
#[derive(Clone)]
pub struct AppNotifier {
    dirty: Arc<AtomicBool>,
}

impl AppNotifier {
    /// Creates a notifier over the shared dirty flag.
    #[must_use]
    pub fn new(dirty: Arc<AtomicBool>) -> Self {
        Self { dirty }
    }

    /// Returns a clone of the shared dirty flag for the run loop to drain.
    #[must_use]
    pub fn dirty_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.dirty)
    }
}

impl EventNotifier for AppNotifier {
    fn notify_state_changed(&self) {
        // `Release` pairs with the loop's `Acquire` swap so a writer's prior
        // `ArcSwap` store is visible to the render that observes the flag.
        self.dirty.store(true, Ordering::Release);
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
pub(crate) mod tests {
    use super::*;

    /// Test fixture: a fresh notifier with a cleared dirty flag.
    #[must_use]
    pub fn generate_test_notifier() -> AppNotifier {
        AppNotifier::new(Arc::new(AtomicBool::new(false)))
    }

    #[test]
    fn app_notifier_sets_dirty_flag() {
        let notifier = generate_test_notifier();
        let flag = notifier.dirty_flag();
        assert!(
            !flag.load(Ordering::Acquire),
            "dirty flag should start clear"
        );

        notifier.notify_state_changed();

        assert!(
            flag.load(Ordering::Acquire),
            "notify_state_changed should set the dirty flag"
        );
    }

    #[test]
    fn app_notifier_dirty_flag_is_shared() {
        // The flag returned by `dirty_flag()` is the *same* atomic the loop
        // drains, so a `swap` by the loop is observed by the notifier's clone.
        let notifier = generate_test_notifier();
        let loop_flag = notifier.dirty_flag();
        notifier.notify_state_changed();

        let was_set = loop_flag.swap(false, Ordering::Acquire);
        assert!(was_set, "loop should observe the set flag");
        assert!(
            !notifier.dirty_flag().load(Ordering::Acquire),
            "draining the flag should clear it for all holders"
        );
    }

    #[test]
    fn app_notifier_is_clone_send_sync() {
        fn assert_clone_send_sync<T: Clone + Send + Sync + 'static>() {}
        assert_clone_send_sync::<AppNotifier>();
    }

    #[test]
    fn app_notifier_usable_as_dyn_event_notifier() {
        // Worker threads (003/005) only ever see `EventNotifier`.
        let notifier = generate_test_notifier();
        let flag = notifier.dirty_flag();
        let dynamic: Box<dyn EventNotifier> = Box::new(notifier);
        dynamic.notify_state_changed();
        assert!(flag.load(Ordering::Acquire));
    }
}
