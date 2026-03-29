//! Thread-safe application state management.
//!
//! Provides [`StateManager`], a convenience wrapper around
//! `Arc<ArcSwap<AppState>>` with typed methods for lock-free reads and
//! `rcu()` (read-copy-update) writes. The render thread reads via
//! [`StateManager::load()`] every frame; input/control threads write via
//! the `update_*()` methods.

use std::sync::Arc;

use arc_swap::ArcSwap;
use luminos_types::ScreenPoint;

use crate::state::AppState;

/// Minimum allowed zoom level.
pub const MIN_ZOOM: f32 = 1.5;
/// Maximum allowed zoom level.
pub const MAX_ZOOM: f32 = 20.0;
/// Default zoom level.
pub const DEFAULT_ZOOM: f32 = 2.0;

/// Thread-safe application state manager.
///
/// Wraps `Arc<ArcSwap<AppState>>` with typed methods for lock-free reads
/// and `rcu()` (read-copy-update) writes. The render thread reads via
/// [`load()`](Self::load) every frame; input/control threads write via
/// the `update_*()` methods.
///
/// The `StateManager` does **not** own or call `EventLoopProxy`. The caller
/// is responsible for sending wake events after state mutations.
#[derive(Clone)]
pub struct StateManager {
    state: Arc<ArcSwap<AppState>>,
}

impl StateManager {
    /// Creates a new state manager wrapping the given shared state.
    pub fn new(state: Arc<ArcSwap<AppState>>) -> Self {
        Self { state }
    }

    /// Returns a lock-free guard to the current application state.
    ///
    /// The returned [`Guard`] dereferences to `&AppState`. This is the
    /// render thread's primary state access method (< 100ns per call).
    #[must_use]
    pub fn load(&self) -> arc_swap::Guard<Arc<AppState>> {
        self.state.load()
    }

    /// Returns a clone of the inner `Arc<ArcSwap<AppState>>` for sharing.
    #[must_use]
    pub fn inner(&self) -> Arc<ArcSwap<AppState>> {
        Arc::clone(&self.state)
    }

    /// Updates the current mouse position via read-copy-update.
    ///
    /// The render thread will see the new position on the next
    /// [`load()`](Self::load) call.
    pub fn update_mouse_position(&self, position: ScreenPoint) {
        self.state.rcu(|current| {
            let mut new_state = (**current).clone();
            new_state.mouse_position = position;
            new_state
        });
    }

    /// Updates the zoom level via read-copy-update.
    ///
    /// Clamps the value to the valid range \[`MIN_ZOOM`, `MAX_ZOOM`\].
    pub fn update_zoom_level(&self, level: f32) {
        let clamped = level.clamp(MIN_ZOOM, MAX_ZOOM);
        self.state.rcu(|current| {
            let mut new_state = (**current).clone();
            new_state.settings.magnification.zoom_level = clamped;
            new_state
        });
    }

    /// Toggles magnification on/off via read-copy-update.
    pub fn toggle_magnification(&self) {
        self.state.rcu(|current| {
            let mut new_state = (**current).clone();
            new_state.is_active = !new_state.is_active;
            new_state
        });
    }

    /// Resets zoom level to the default ([`DEFAULT_ZOOM`]) via read-copy-update.
    pub fn reset_zoom(&self) {
        self.state.rcu(|current| {
            let mut new_state = (**current).clone();
            new_state.settings.magnification.zoom_level = DEFAULT_ZOOM;
            new_state
        });
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn generate_test_state_manager() -> StateManager {
        let shared = Arc::new(ArcSwap::from_pointee(AppState::default()));
        StateManager::new(shared)
    }

    #[test]
    fn state_manager_new_and_load() {
        let mgr = generate_test_state_manager();
        let guard = mgr.load();
        assert_eq!(**guard, AppState::default());
    }

    #[test]
    fn state_manager_load_returns_guard() {
        let mgr = generate_test_state_manager();
        let guard = mgr.load();
        let state: &AppState = &guard;
        assert!(!state.is_active);
    }

    #[test]
    fn state_manager_is_clone() {
        let mgr = generate_test_state_manager();
        let cloned = mgr.clone();
        let guard = cloned.load();
        assert_eq!(**guard, AppState::default());
    }

    #[test]
    fn state_manager_inner_returns_shared_arc() {
        let shared = Arc::new(ArcSwap::from_pointee(AppState::default()));
        let mgr = StateManager::new(Arc::clone(&shared));
        let inner = mgr.inner();
        assert!(Arc::ptr_eq(&shared, &inner));
    }

    // T004 tests -- update_mouse_position

    #[test]
    fn state_manager_update_mouse_position_basic() {
        let mgr = generate_test_state_manager();
        mgr.update_mouse_position(ScreenPoint { x: 500, y: 300 });
        let guard = mgr.load();
        assert_eq!(guard.mouse_position, ScreenPoint { x: 500, y: 300 });
    }

    #[test]
    fn state_manager_update_mouse_position_preserves_other_fields() {
        let mgr = generate_test_state_manager();
        mgr.update_zoom_level(5.0);
        mgr.update_mouse_position(ScreenPoint { x: 42, y: 99 });
        let guard = mgr.load();
        assert!(
            (guard.settings.magnification.zoom_level - 5.0).abs() < f32::EPSILON,
            "zoom_level should be preserved after mouse position update"
        );
    }

    #[test]
    fn state_manager_update_mouse_position_negative_coords() {
        let mgr = generate_test_state_manager();
        mgr.update_mouse_position(ScreenPoint { x: -100, y: -50 });
        let guard = mgr.load();
        assert_eq!(guard.mouse_position, ScreenPoint { x: -100, y: -50 });
    }

    // T005 tests -- zoom, toggle, reset

    #[test]
    fn state_manager_update_zoom_level_valid() {
        let mgr = generate_test_state_manager();
        mgr.update_zoom_level(5.0);
        let guard = mgr.load();
        assert!((guard.settings.magnification.zoom_level - 5.0).abs() < f32::EPSILON,);
    }

    #[test]
    fn state_manager_update_zoom_level_clamp_high() {
        let mgr = generate_test_state_manager();
        mgr.update_zoom_level(25.0);
        let guard = mgr.load();
        assert!(
            (guard.settings.magnification.zoom_level - MAX_ZOOM).abs() < f32::EPSILON,
            "zoom should be clamped to MAX_ZOOM (20.0)"
        );
    }

    #[test]
    fn state_manager_update_zoom_level_clamp_low() {
        let mgr = generate_test_state_manager();
        mgr.update_zoom_level(0.5);
        let guard = mgr.load();
        assert!(
            (guard.settings.magnification.zoom_level - MIN_ZOOM).abs() < f32::EPSILON,
            "zoom should be clamped to MIN_ZOOM (1.5)"
        );
    }

    #[test]
    fn state_manager_toggle_magnification_on_to_off() {
        let mgr = generate_test_state_manager();
        // Default is false, so first toggle to true, then toggle to false
        mgr.toggle_magnification();
        assert!(mgr.load().is_active, "should be active after first toggle");
        mgr.toggle_magnification();
        assert!(
            !mgr.load().is_active,
            "should be inactive after second toggle"
        );
    }

    #[test]
    fn state_manager_toggle_magnification_off_to_on() {
        let mgr = generate_test_state_manager();
        assert!(!mgr.load().is_active, "default should be inactive");
        mgr.toggle_magnification();
        assert!(mgr.load().is_active, "should be active after toggle");
    }

    #[test]
    fn state_manager_toggle_magnification_double_toggle() {
        let mgr = generate_test_state_manager();
        let original = mgr.load().is_active;
        mgr.toggle_magnification();
        mgr.toggle_magnification();
        assert_eq!(mgr.load().is_active, original);
    }

    #[test]
    fn state_manager_reset_zoom_to_default() {
        let mgr = generate_test_state_manager();
        mgr.update_zoom_level(10.0);
        mgr.reset_zoom();
        let guard = mgr.load();
        assert!(
            (guard.settings.magnification.zoom_level - DEFAULT_ZOOM).abs() < f32::EPSILON,
            "zoom should be reset to DEFAULT_ZOOM (2.0)"
        );
    }

    #[test]
    fn state_manager_reset_zoom_preserves_other_settings() {
        let mgr = generate_test_state_manager();
        mgr.toggle_magnification(); // set is_active = true
        mgr.update_zoom_level(10.0);
        mgr.reset_zoom();
        let guard = mgr.load();
        assert!(
            guard.is_active,
            "is_active should be preserved after reset_zoom"
        );
    }

    // T006 tests -- Send + Sync

    #[test]
    fn state_manager_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<StateManager>();
    }

    #[test]
    fn state_manager_is_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<StateManager>();
    }

    // T007 tests -- cross-thread visibility

    #[test]
    fn state_manager_cross_thread_visibility() {
        use std::sync::Barrier;

        let mgr = generate_test_state_manager();
        let barrier = Arc::new(Barrier::new(2));

        let writer_mgr = mgr.clone();
        let writer_barrier = Arc::clone(&barrier);

        let writer = std::thread::spawn(move || {
            writer_mgr.update_mouse_position(ScreenPoint { x: 999, y: 888 });
            writer_barrier.wait();
        });

        // Wait for writer to finish
        barrier.wait();
        writer.join().unwrap();

        let guard = mgr.load();
        assert_eq!(guard.mouse_position, ScreenPoint { x: 999, y: 888 });
    }

    #[test]
    fn state_manager_concurrent_writers_no_lost_updates() {
        for _ in 0..100 {
            let mgr = generate_test_state_manager();
            let barrier = Arc::new(std::sync::Barrier::new(3));

            let mgr_a = mgr.clone();
            let barrier_a = Arc::clone(&barrier);
            let thread_a = std::thread::spawn(move || {
                barrier_a.wait();
                mgr_a.toggle_magnification();
            });

            let mgr_b = mgr.clone();
            let barrier_b = Arc::clone(&barrier);
            let thread_b = std::thread::spawn(move || {
                barrier_b.wait();
                mgr_b.update_zoom_level(5.0);
            });

            // Release both threads simultaneously
            barrier.wait();
            thread_a.join().unwrap();
            thread_b.join().unwrap();

            let guard = mgr.load();
            assert!(
                guard.is_active,
                "toggle should have activated magnification"
            );
            assert!(
                (guard.settings.magnification.zoom_level - 5.0).abs() < f32::EPSILON,
                "zoom level should be 5.0"
            );
        }
    }

    // T008 tests -- load latency benchmark

    #[test]
    fn state_manager_load_latency_under_100ns() {
        let mgr = generate_test_state_manager();

        // Warm up
        for _ in 0..1_000 {
            let _guard = std::hint::black_box(mgr.load());
        }

        let iterations = 100_000_u64;
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let _guard = std::hint::black_box(mgr.load());
        }
        let elapsed = start.elapsed();
        let avg_ns = elapsed.as_nanos() / u128::from(iterations);

        // NFR-1 target is <100ns in release builds. Debug builds have
        // higher overhead, so use a relaxed 500ns threshold for CI.
        // Release-mode benchmarks consistently measure <30ns.
        let threshold_ns = if cfg!(debug_assertions) { 500 } else { 100 };
        assert!(
            avg_ns < threshold_ns,
            "load() average latency should be < {threshold_ns}ns, got {avg_ns}ns"
        );
    }
}
