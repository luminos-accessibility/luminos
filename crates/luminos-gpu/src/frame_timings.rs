//! Frame timing measurement and performance degradation detection.
//!
//! Contains [`FrameTimings`], a circular buffer of the last 120 frame
//! durations (2 seconds at 60fps), with aggregate statistics (P99,
//! average, min, max) and threshold-based performance alerting per
//! doc-03 Section 8.3.

use std::time::Duration;

/// Performance threshold: P99 > 20ms triggers warning (below 50fps effective).
const WARN_THRESHOLD: Duration = Duration::from_millis(20);

/// Performance threshold: P99 > 33ms triggers critical alert (below 30fps).
const CRITICAL_THRESHOLD: Duration = Duration::from_millis(33);

/// Number of consecutive above-threshold recordings before an alert fires.
/// 300 recordings at 60fps = 5 seconds.
const THRESHOLD_STREAK_LIMIT: u32 = 300;

/// Frame timing statistics for performance monitoring.
///
/// Maintains a circular buffer of the last 120 frame times (2 seconds
/// at 60fps). Provides aggregate statistics (P99, average, min, max)
/// and performance degradation detection.
///
/// # Performance Thresholds
///
/// | Level | Condition | Response |
/// |-------|-----------|----------|
/// | Warning | P99 > 20ms for 5 seconds (300 recordings) | `warn!` log |
/// | Critical | P99 > 33ms for 5 seconds (300 recordings) | `error!` log |
pub struct FrameTimings {
    /// Circular buffer of the last 120 frame times.
    history: [Duration; 120],
    /// Write index into the circular buffer.
    index: usize,
    /// Number of frames recorded (saturates at 120).
    count: usize,
    /// Consecutive recordings where P99 exceeded the warn threshold.
    warn_streak: u32,
    /// Consecutive recordings where P99 exceeded the critical threshold.
    critical_streak: u32,
}

/// IPC-ready frame timing summary.
///
/// Contains aggregate statistics suitable for transmission to the
/// control panel via Tauri IPC (E04+). All time fields are in
/// milliseconds.
///
/// # Wire format (DC-5)
///
/// This is the **one** IPC type renamed to `camelCase`: the JSON keys are
/// `averageMs`, `p99Ms`, `minMs`, `maxMs`, `targetFps` (story 006's Zod schema
/// and `tauri-specta` bindings depend on exactly these keys). All other IPC
/// types (`AppSettings` + sub-structs) stay `snake_case`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct FrameTimingSummary {
    /// Average frame time in milliseconds.
    pub average_ms: f64,
    /// P99 frame time in milliseconds.
    pub p99_ms: f64,
    /// Minimum frame time in milliseconds.
    pub min_ms: f64,
    /// Maximum frame time in milliseconds.
    pub max_ms: f64,
    /// Target frame rate.
    pub target_fps: u32,
}

impl FrameTimings {
    /// Creates a new `FrameTimings` with all-zero history.
    #[must_use]
    pub fn new() -> Self {
        Self {
            history: [Duration::ZERO; 120],
            index: 0,
            count: 0,
            warn_streak: 0,
            critical_streak: 0,
        }
    }

    /// Records a frame duration and checks performance thresholds.
    ///
    /// Writes the duration to the circular buffer at the current index,
    /// advances the index, and saturates the count at 120. Threshold
    /// checks only fire after the buffer is full (120 frames recorded).
    pub fn record(&mut self, frame_time: Duration) {
        self.history[self.index] = frame_time;
        self.index = (self.index + 1) % self.history.len();
        if self.count < self.history.len() {
            self.count += 1;
        }

        // Check thresholds only after buffer is full (120 frames)
        if self.count == self.history.len() {
            let p99 = self.p99();
            self.check_thresholds(p99);
        }
    }

    /// Returns the P99 frame time over recorded frames.
    ///
    /// Computes the 99th percentile by sorting a copy of the filled
    /// portion of the buffer and selecting the element at index
    /// `ceil(0.99 * count) - 1`.
    ///
    /// Returns `Duration::ZERO` when no frames have been recorded.
    #[must_use]
    pub fn p99(&self) -> Duration {
        if self.count == 0 {
            return Duration::ZERO;
        }
        let mut sorted: Vec<Duration> = self.history[..self.count].to_vec();
        sorted.sort_unstable();
        // 99th percentile: index ceil(0.99 * count) - 1
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let idx = ((self.count as f64 * 0.99).ceil() as usize).saturating_sub(1);
        sorted[idx.min(self.count - 1)]
    }

    /// Returns the average frame time over recorded frames.
    ///
    /// Returns `Duration::ZERO` when no frames have been recorded.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn average(&self) -> Duration {
        if self.count == 0 {
            return Duration::ZERO;
        }
        let sum: Duration = self.history[..self.count].iter().sum();
        sum / self.count as u32
    }

    /// Returns the minimum frame time over recorded frames.
    ///
    /// Returns `Duration::ZERO` when no frames have been recorded.
    #[must_use]
    pub fn min(&self) -> Duration {
        self.history[..self.count]
            .iter()
            .copied()
            .min()
            .unwrap_or(Duration::ZERO)
    }

    /// Returns the maximum frame time over recorded frames.
    ///
    /// Returns `Duration::ZERO` when no frames have been recorded.
    #[must_use]
    pub fn max(&self) -> Duration {
        self.history[..self.count]
            .iter()
            .copied()
            .max()
            .unwrap_or(Duration::ZERO)
    }

    /// Creates a [`FrameTimingSummary`] for IPC reporting.
    ///
    /// All duration fields are converted to milliseconds. The
    /// `target_fps` is passed through unchanged.
    #[must_use]
    pub fn summary(&self, target_fps: u32) -> FrameTimingSummary {
        FrameTimingSummary {
            average_ms: self.average().as_secs_f64() * 1000.0,
            p99_ms: self.p99().as_secs_f64() * 1000.0,
            min_ms: self.min().as_secs_f64() * 1000.0,
            max_ms: self.max().as_secs_f64() * 1000.0,
            target_fps,
        }
    }

    /// Returns the current warn streak count (for testing).
    #[cfg(test)]
    pub(crate) fn warn_streak(&self) -> u32 {
        self.warn_streak
    }

    /// Returns the current critical streak count (for testing).
    #[cfg(test)]
    pub(crate) fn critical_streak(&self) -> u32 {
        self.critical_streak
    }

    /// Checks performance thresholds and logs warnings.
    ///
    /// Increments streak counters when P99 exceeds the respective
    /// threshold, resets them when P99 drops below. Emits a log
    /// message exactly once when a streak reaches the limit.
    fn check_thresholds(&mut self, p99: Duration) {
        // Warn threshold
        if p99 > WARN_THRESHOLD {
            self.warn_streak += 1;
            if self.warn_streak == THRESHOLD_STREAK_LIMIT {
                log::warn!(
                    concat!(
                        "Performance degradation: P99 frame time '{:.2}ms' ",
                        "exceeded '{:.2}ms' threshold for 5 seconds"
                    ),
                    p99.as_secs_f64() * 1000.0,
                    WARN_THRESHOLD.as_secs_f64() * 1000.0,
                );
            }
        } else {
            self.warn_streak = 0;
        }

        // Critical threshold
        if p99 > CRITICAL_THRESHOLD {
            self.critical_streak += 1;
            if self.critical_streak == THRESHOLD_STREAK_LIMIT {
                log::error!(
                    concat!(
                        "Critical performance degradation: P99 frame time '{:.2}ms' ",
                        "exceeded '{:.2}ms' threshold for 5 seconds"
                    ),
                    p99.as_secs_f64() * 1000.0,
                    CRITICAL_THRESHOLD.as_secs_f64() * 1000.0,
                );
            }
        } else {
            self.critical_streak = 0;
        }
    }
}

impl Default for FrameTimings {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // ── T003: Constructor, record, and circular buffer ──────────────

    #[test]
    fn frame_timings_new_returns_zero_count() {
        let ft = FrameTimings::new();
        assert_eq!(ft.p99(), Duration::ZERO);
    }

    #[test]
    fn frame_timings_record_single_increments_count() {
        let mut ft = FrameTimings::new();
        ft.record(Duration::from_millis(10));
        assert_eq!(ft.p99(), Duration::from_millis(10));
    }

    #[test]
    fn frame_timings_record_fills_buffer() {
        let mut ft = FrameTimings::new();
        // 118 frames at 10ms, 2 frames at 20ms
        for _ in 0..118 {
            ft.record(Duration::from_millis(10));
        }
        ft.record(Duration::from_millis(20));
        ft.record(Duration::from_millis(20));

        // P99 of 120 samples: ceil(0.99 * 120) - 1 = 119 - 1 = 118
        // Sorted: 118 x 10ms, 2 x 20ms. Index 118 = 20ms
        assert_eq!(ft.p99(), Duration::from_millis(20));
    }

    #[test]
    fn frame_timings_record_wraps_buffer() {
        let mut ft = FrameTimings::new();
        // Record 130 durations: first 10 at 5ms, then 120 at 15ms
        for _ in 0..10 {
            ft.record(Duration::from_millis(5));
        }
        for _ in 0..120 {
            ft.record(Duration::from_millis(15));
        }
        // Buffer should only contain the last 120 entries (all 15ms)
        assert_eq!(ft.p99(), Duration::from_millis(15));
        assert_eq!(ft.average(), Duration::from_millis(15));
    }

    #[test]
    fn frame_timings_default_equals_new() {
        let from_new = FrameTimings::new();
        let from_default = FrameTimings::default();
        assert_eq!(from_new.p99(), from_default.p99());
        assert_eq!(from_new.average(), from_default.average());
        assert_eq!(from_new.min(), from_default.min());
        assert_eq!(from_new.max(), from_default.max());
    }

    // ── T004: Aggregate statistics (p99, average, min, max) ─────────

    #[test]
    fn frame_timings_p99_empty_returns_zero() {
        let ft = FrameTimings::new();
        assert_eq!(ft.p99(), Duration::ZERO);
    }

    #[test]
    fn frame_timings_p99_known_distribution() {
        let mut ft = FrameTimings::new();
        // 100 frames at 10ms + 20 frames at 15ms = 120 total
        for _ in 0..100 {
            ft.record(Duration::from_millis(10));
        }
        for _ in 0..20 {
            ft.record(Duration::from_millis(15));
        }
        // P99: ceil(0.99 * 120) - 1 = 119 - 1 = 118
        // Sorted: 100 x 10ms, 20 x 15ms. Index 118 = 15ms
        assert_eq!(ft.p99(), Duration::from_millis(15));
    }

    #[test]
    fn frame_timings_p99_uniform_distribution() {
        let mut ft = FrameTimings::new();
        for _ in 0..120 {
            ft.record(Duration::from_millis(16));
        }
        assert_eq!(ft.p99(), Duration::from_millis(16));
    }

    #[test]
    fn frame_timings_average_known_values() {
        let mut ft = FrameTimings::new();
        for _ in 0..60 {
            ft.record(Duration::from_millis(10));
        }
        for _ in 0..60 {
            ft.record(Duration::from_millis(20));
        }
        assert_eq!(ft.average(), Duration::from_millis(15));
    }

    #[test]
    fn frame_timings_average_empty_returns_zero() {
        let ft = FrameTimings::new();
        assert_eq!(ft.average(), Duration::ZERO);
    }

    #[test]
    fn frame_timings_min_known_values() {
        let mut ft = FrameTimings::new();
        ft.record(Duration::from_millis(15));
        ft.record(Duration::from_millis(5));
        ft.record(Duration::from_millis(10));
        assert_eq!(ft.min(), Duration::from_millis(5));
    }

    #[test]
    fn frame_timings_max_known_values() {
        let mut ft = FrameTimings::new();
        ft.record(Duration::from_millis(5));
        ft.record(Duration::from_millis(15));
        ft.record(Duration::from_millis(10));
        assert_eq!(ft.max(), Duration::from_millis(15));
    }

    #[test]
    fn frame_timings_min_empty_returns_zero() {
        let ft = FrameTimings::new();
        assert_eq!(ft.min(), Duration::ZERO);
    }

    #[test]
    fn frame_timings_max_empty_returns_zero() {
        let ft = FrameTimings::new();
        assert_eq!(ft.max(), Duration::ZERO);
    }

    // ── T005: FrameTimingSummary and summary() ──────────────────────

    #[test]
    fn frame_timings_summary_fields_match_individual_methods() {
        let mut ft = FrameTimings::new();
        for i in 0..120 {
            ft.record(Duration::from_millis(10 + (i % 5)));
        }
        let summary = ft.summary(60);
        let epsilon = 0.001;

        assert!(
            (summary.average_ms - ft.average().as_secs_f64() * 1000.0).abs() < epsilon,
            "average_ms mismatch"
        );
        assert!(
            (summary.p99_ms - ft.p99().as_secs_f64() * 1000.0).abs() < epsilon,
            "p99_ms mismatch"
        );
        assert!(
            (summary.min_ms - ft.min().as_secs_f64() * 1000.0).abs() < epsilon,
            "min_ms mismatch"
        );
        assert!(
            (summary.max_ms - ft.max().as_secs_f64() * 1000.0).abs() < epsilon,
            "max_ms mismatch"
        );
        assert_eq!(summary.target_fps, 60);
    }

    #[test]
    fn frame_timings_summary_empty_returns_zeros() {
        let ft = FrameTimings::new();
        let summary = ft.summary(60);
        let epsilon = 0.001;

        assert!(summary.average_ms.abs() < epsilon);
        assert!(summary.p99_ms.abs() < epsilon);
        assert!(summary.min_ms.abs() < epsilon);
        assert!(summary.max_ms.abs() < epsilon);
        assert_eq!(summary.target_fps, 60);
    }

    #[test]
    fn frame_timings_summary_partial_buffer() {
        let mut ft = FrameTimings::new();
        for _ in 0..50 {
            ft.record(Duration::from_millis(12));
        }
        let summary = ft.summary(60);
        let epsilon = 0.001;

        assert!(
            (summary.average_ms - 12.0).abs() < epsilon,
            "partial buffer average should be 12.0ms, got {}",
            summary.average_ms
        );
    }

    // ── T006: Performance threshold detection ───────────────────────

    #[test]
    fn frame_timings_warn_streak_increments_above_threshold() {
        let mut ft = FrameTimings::new();
        // Fill buffer with 120 frames at 25ms (above 20ms warn threshold)
        for _ in 0..120 {
            ft.record(Duration::from_millis(25));
        }
        // Buffer is now full; check_thresholds was called once
        assert_eq!(ft.warn_streak(), 1);

        // Record 10 more, each triggers check_thresholds
        for _ in 0..10 {
            ft.record(Duration::from_millis(25));
        }
        assert_eq!(ft.warn_streak(), 11);
    }

    #[test]
    fn frame_timings_warn_streak_resets_below_threshold() {
        let mut ft = FrameTimings::new();
        // Fill buffer with 120 frames at 25ms
        for _ in 0..120 {
            ft.record(Duration::from_millis(25));
        }
        assert!(ft.warn_streak() > 0);

        // Replace all entries with 10ms (below threshold)
        for _ in 0..120 {
            ft.record(Duration::from_millis(10));
        }
        assert_eq!(ft.warn_streak(), 0);
    }

    #[test]
    fn frame_timings_critical_streak_increments_above_threshold() {
        let mut ft = FrameTimings::new();
        // Fill buffer with 120 frames at 40ms (above 33ms critical threshold)
        for _ in 0..120 {
            ft.record(Duration::from_millis(40));
        }
        assert_eq!(ft.critical_streak(), 1);

        for _ in 0..10 {
            ft.record(Duration::from_millis(40));
        }
        assert_eq!(ft.critical_streak(), 11);
    }

    #[test]
    fn frame_timings_critical_streak_resets_below_threshold() {
        let mut ft = FrameTimings::new();
        for _ in 0..120 {
            ft.record(Duration::from_millis(40));
        }
        assert!(ft.critical_streak() > 0);

        for _ in 0..120 {
            ft.record(Duration::from_millis(10));
        }
        assert_eq!(ft.critical_streak(), 0);
    }

    #[test]
    fn frame_timings_threshold_no_check_before_buffer_full() {
        let mut ft = FrameTimings::new();
        // Record only 50 frames (buffer not full), all at 40ms
        for _ in 0..50 {
            ft.record(Duration::from_millis(40));
        }
        // No threshold checks should have fired
        assert_eq!(ft.warn_streak(), 0);
        assert_eq!(ft.critical_streak(), 0);
    }

    #[test]
    fn frame_timings_warn_fires_at_streak_limit() {
        let mut ft = FrameTimings::new();
        // Fill buffer + 299 more recordings = 300 threshold checks total
        // (first check at record #120, then 299 more = records 120..419)
        for _ in 0..(120 + 299) {
            ft.record(Duration::from_millis(25));
        }
        // Streak should be exactly 300
        assert_eq!(ft.warn_streak(), THRESHOLD_STREAK_LIMIT);
    }

    // ── E04/005 T002: FrameTimingSummary serde camelCase + specta::Type ──────

    #[test]
    fn frame_timing_summary_serde_camelcase() {
        // DC-5: `FrameTimingSummary` is the ONE IPC type renamed to camelCase.
        // The control-panel Zod schema (story 006) expects exactly these keys:
        // `averageMs`, `p99Ms`, `minMs`, `maxMs`, `targetFps`.
        let summary = FrameTimingSummary {
            average_ms: 8.0,
            p99_ms: 12.5,
            min_ms: 6.0,
            max_ms: 18.0,
            target_fps: 60,
        };
        let json = serde_json::to_value(&summary).unwrap();
        let obj = json.as_object().unwrap();
        assert!(
            obj.contains_key("averageMs"),
            "expected camelCase 'averageMs'"
        );
        assert!(obj.contains_key("p99Ms"), "expected camelCase 'p99Ms'");
        assert!(obj.contains_key("minMs"), "expected camelCase 'minMs'");
        assert!(obj.contains_key("maxMs"), "expected camelCase 'maxMs'");
        assert!(
            obj.contains_key("targetFps"),
            "expected camelCase 'targetFps'"
        );
        // And NOT the snake_case Rust field names.
        assert!(
            !obj.contains_key("average_ms"),
            "snake_case must be renamed"
        );
        assert!(
            !obj.contains_key("target_fps"),
            "snake_case must be renamed"
        );
    }

    #[test]
    fn frame_timing_summary_serde_roundtrip() {
        let summary = FrameTimingSummary {
            average_ms: 8.0,
            p99_ms: 12.5,
            min_ms: 6.0,
            max_ms: 18.0,
            target_fps: 60,
        };
        let json = serde_json::to_string(&summary).unwrap();
        let back: FrameTimingSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(summary, back, "camelCase JSON must round-trip");
    }

    #[test]
    fn frame_timing_summary_implements_specta_type() {
        // Compile-time bound check: the type must derive `specta::Type` so it
        // can appear as a `#[specta::specta]` command return type (DC-5).
        fn assert_specta_type<T: specta::Type>() {}
        assert_specta_type::<FrameTimingSummary>();
    }
}
