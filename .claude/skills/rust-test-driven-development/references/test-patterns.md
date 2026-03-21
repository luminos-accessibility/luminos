# Rust Test Patterns Reference

Detailed testing patterns for the Luminos project. Read this file when you need guidance on
specific testing techniques beyond the core TDD workflow in SKILL.md.

## Table of Contents

1. [Test Double Taxonomy](#1-test-double-taxonomy)
2. [Async Testing Patterns](#2-async-testing-patterns)
3. [Property-Based Testing with proptest](#3-property-based-testing-with-proptest)
4. [Parameterized Tests with rstest](#4-parameterized-tests-with-rstest)
5. [Snapshot Testing with insta](#5-snapshot-testing-with-insta)
6. [Error Path Testing](#6-error-path-testing)
7. [Testing State Machines](#7-testing-state-machines)
8. [Platform-Gated Integration Tests](#8-platform-gated-integration-tests)
9. [Performance and Regression Tests](#9-performance-and-regression-tests)
10. [Recommended Crate Stack](#10-recommended-crate-stack)

---

## 1. Test Double Taxonomy

Luminos uses three kinds of test doubles. Understanding which to use prevents excessive mocking.

### Fakes (Preferred Default)

A fake is a working simplified implementation. It exercises real logic paths but without
real platform dependencies. Luminos's mock backends for the six platform traits (doc-02 Section 7.1)
are actually fakes.

```rust
#[cfg(test)]
pub(crate) struct FakeScreenCapture {
    frames: Vec<CaptureFrame>,
    call_count: std::cell::Cell<usize>,
    error_factory: Option<Box<dyn Fn() -> CaptureError>>,
}

#[cfg(test)]
impl FakeScreenCapture {
    /// Creates a fake that returns the given frames in sequence.
    pub fn new(frames: Vec<CaptureFrame>) -> Self {
        Self {
            frames,
            call_count: std::cell::Cell::new(0),
            error_factory: None,
        }
    }

    /// Creates a fake that always returns an error (for error path testing).
    /// Uses a factory closure because CaptureError may not implement Clone.
    pub fn with_error(factory: impl Fn() -> CaptureError + 'static) -> Self {
        Self {
            frames: vec![],
            call_count: std::cell::Cell::new(0),
            error_factory: Some(Box::new(factory)),
        }
    }
}

#[cfg(test)]
impl ScreenCapture for FakeScreenCapture {
    fn capture_frame(&self) -> Result<CaptureFrame, CaptureError> {
        if let Some(ref factory) = self.error_factory {
            return Err(factory());
        }
        let idx = self.call_count.get();
        self.call_count.set(idx + 1);
        self.frames.get(idx % self.frames.len())
            .cloned()
            .ok_or_else(|| CaptureError::NoFrameAvailable)
    }
}
```

**When to use fakes:** Most unit tests. When you want to verify that your code handles
the trait's return values correctly.

### Stubs

A stub returns hardcoded values. Simpler than a fake — use when you need a trait
implementation but don't care about its behavior for this test.

```rust
#[cfg(test)]
struct StubAudioOutput;

#[cfg(test)]
impl AudioOutput for StubAudioOutput {
    fn play(&self, _samples: &[f32], _sample_rate: u32) -> Result<(), AudioError> {
        Ok(()) // Always succeeds — we're testing something else
    }
}
```

**When to use stubs:** When a trait is a dependency but not the subject of the test.

### Mocks (mockall — Use Sparingly)

Mocks verify that specific methods were called with specific arguments. Use mockall
only when the behavior you're testing is "did we call this correctly?"

```rust
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait TtsEngine {
    fn synthesize(&self, phonemes: &str, voice_id: &str) -> Result<Vec<f32>, TtsError>;
}

#[test]
fn tts_coordinator_passes_correct_voice_to_engine() {
    let mut mock = MockTtsEngine::new();
    mock.expect_synthesize()
        .withf(|_phonemes, voice_id| voice_id == "af_heart")
        .times(1)
        .returning(|_, _| Ok(vec![0.0; 1024]));

    let coordinator = TtsCoordinator::new(Box::new(mock));
    coordinator.speak("hello", "af_heart").unwrap();
}
```

**When to use mocks:** Only for interaction verification — confirming that your code
calls a dependency correctly. Not for testing return value handling (use fakes for that).

---

## 2. Async Testing Patterns

Luminos uses tokio for async I/O tasks. Async testing has specific pitfalls.

### Basic Async Test

```rust
#[tokio::test]
async fn focus_tracker_emits_focus_change_event() {
    let tracker = FakeFocusTracker::new(vec![
        FocusEvent::new("Firefox", Rect::new(0, 0, 800, 600)),
    ]);
    let event = tracker.next_event().await.unwrap();
    assert_eq!(event.app_name, "Firefox");
}
```

### Deterministic Time Control

Use `start_paused = true` when testing timeouts or delays. This makes `tokio::time::sleep`
resolve instantly when advanced, so tests don't actually wait.

```rust
#[tokio::test(start_paused = true)]
async fn frame_limiter_enforces_minimum_interval() {
    let limiter = FrameLimiter::new(Duration::from_millis(16)); // 60fps

    limiter.wait().await; // First frame: immediate
    let start = tokio::time::Instant::now();
    limiter.wait().await; // Second frame: should wait
    assert!(start.elapsed() >= Duration::from_millis(16));
}
```

### Channel Timeout Pattern

Always wrap channel receives in a timeout to prevent tests from hanging forever:

```rust
#[tokio::test]
async fn coordinator_sends_frames_to_renderer() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);
    // ... set up coordinator with tx ...

    let frame = tokio::time::timeout(Duration::from_secs(1), rx.recv())
        .await
        .expect("timed out waiting for frame")
        .expect("channel closed unexpectedly");

    assert_eq!(frame.width, 1920);
}
```

### Pitfalls

- **Never nest tokio runtimes.** Calling `Runtime::block_on()` inside a `#[tokio::test]`
  panics. If you need sync code in an async test, use `tokio::task::spawn_blocking`.
- **Don't use `std::thread::sleep()` in async tests.** It blocks the executor thread.
  Use `tokio::time::sleep()` instead.
- **Don't make sync code async just for testing.** If the production code is sync,
  test it with sync tests.

---

## 3. Property-Based Testing with proptest

Use proptest when you need to verify invariants across many inputs — particularly
for algorithmic code like viewport calculations, coordinate transforms, or text
segmentation.

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn viewport_zoom_preserves_center(
        center_x in 0.0f64..3840.0,
        center_y in 0.0f64..2160.0,
        zoom in 1.0f64..36.0,
    ) {
        let viewport = Viewport::centered_at(center_x, center_y, zoom);
        let actual_center = viewport.center();

        // Center should be preserved within floating-point tolerance
        prop_assert!((actual_center.x - center_x).abs() < 0.01);
        prop_assert!((actual_center.y - center_y).abs() < 0.01);
    }

    #[test]
    fn viewport_zoom_area_inversely_proportional(
        zoom_a in 1.0f64..36.0,
        zoom_b in 1.0f64..36.0,
    ) {
        let vp_a = Viewport::new(1920, 1080, zoom_a);
        let vp_b = Viewport::new(1920, 1080, zoom_b);

        // Higher zoom = smaller visible area
        if zoom_a > zoom_b {
            prop_assert!(vp_a.visible_area() < vp_b.visible_area());
        }
    }
}
```

**When to use:** Viewport math, coordinate transforms, color space conversions,
text segmentation boundary detection, settings validation (any valid combination
of settings should not panic).

---

## 4. Parameterized Tests with rstest

Use rstest when you have a clear set of input/output pairs — more readable than
copy-pasting the same test with different values.

```rust
use rstest::rstest;

#[rstest]
#[case(PixelFormat::Bgra8, 4)]
#[case(PixelFormat::Rgba8, 4)]
fn pixel_format_bytes_per_pixel(#[case] format: PixelFormat, #[case] expected: usize) {
    assert_eq!(format.bytes_per_pixel(), expected);
}

#[rstest]
#[case("hello world", vec!["hello world"])]
#[case("First. Second.", vec!["First.", "Second."])]
#[case("Dr. Smith went home.", vec!["Dr. Smith went home."])]
fn text_sentence_segmentation(
    #[case] input: &str,
    #[case] expected: Vec<&str>,
) {
    let segments = segment_sentences(input);
    assert_eq!(segments, expected);
}
```

### rstest Fixtures

For shared setup across multiple tests in a module:

```rust
use rstest::fixture;

#[fixture]
fn capture_config() -> CaptureConfig {
    generate_test_capture_config()
}

#[rstest]
fn screen_capture_returns_correct_dimensions(capture_config: CaptureConfig) {
    let capture = FakeScreenCapture::new_with_config(&capture_config);
    let frame = capture.capture_frame().unwrap();
    assert_eq!(frame.width, capture_config.width);
}
```

---

## 5. Snapshot Testing with insta

Use insta for verifying complex output that would be tedious to assert field-by-field —
especially serialized config, IPC message formats, and error display strings.

```rust
use insta::assert_snapshot;
use insta::assert_json_snapshot;

#[test]
fn settings_default_serialization_stable() {
    let settings = Settings::default();
    assert_json_snapshot!(settings);
}

#[test]
fn capture_error_display_messages() {
    assert_snapshot!(CaptureError::PermissionDenied.to_string());
    assert_snapshot!(CaptureError::DisplayNotFound { id: 42 }.to_string());
}
```

After writing snapshot tests, run `cargo insta review` to accept/reject the initial
snapshots. The `.snap` files become the reference — future changes that alter the output
will fail until reviewed.

**When to use:** Settings serialization, error message formats, IPC command/response
structures, any complex structured output.

---

## 6. Error Path Testing

Every `LuminosError` variant and every `Result::Err` path in the code should have a test.
Error paths are where bugs hide because they're exercised least in manual testing.

```rust
#[test]
fn screen_capture_x11_permission_denied_returns_error() {
    let capture = FakeScreenCapture::with_error(|| CaptureError::PermissionDenied);

    let result = capture.capture_frame();

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CaptureError::PermissionDenied));
}

#[test]
fn tts_coordinator_espeak_crash_triggers_degradation() {
    let engine = FakeTtsEngine::with_error(|| TtsError::ProcessCrashed {
        exit_code: Some(139),
    });
    let coordinator = TtsCoordinator::new(Box::new(engine));

    let result = coordinator.speak("hello");

    assert!(result.is_err());
    assert_eq!(coordinator.degradation_level(), DegradationLevel::EspeakUnavailable);
}
```

**Pattern:** Use the factory closure pattern (`with_error(|| ...)`) for error injection
because error types often don't implement `Clone`. The factory creates a fresh error
each time it's called.

---

## 7. Testing State Machines

Luminos has several state machines (TTS Coordinator states, degradation levels,
capture session states). Test both valid and invalid transitions.

```rust
#[test]
fn tts_state_idle_to_phonemizing_on_speak() {
    let mut coordinator = TtsCoordinator::new_idle();
    assert_eq!(coordinator.state(), TtsState::Idle);

    coordinator.begin_speak("hello").unwrap();
    assert_eq!(coordinator.state(), TtsState::Phonemizing);
}

#[test]
fn tts_state_cannot_speak_while_phonemizing() {
    let mut coordinator = TtsCoordinator::new_in_state(TtsState::Phonemizing);

    let result = coordinator.begin_speak("world");
    assert!(matches!(result, Err(TtsError::Busy)));
}

// Test the full happy-path cycle
#[test]
fn tts_state_full_cycle_idle_to_idle() {
    let mut coordinator = generate_test_tts_coordinator();
    assert_eq!(coordinator.state(), TtsState::Idle);

    coordinator.begin_speak("hello").unwrap();
    assert_eq!(coordinator.state(), TtsState::Phonemizing);

    coordinator.on_phonemes_ready("h@loU").unwrap();
    assert_eq!(coordinator.state(), TtsState::Synthesizing);

    coordinator.on_synthesis_complete(vec![0.0; 1024]).unwrap();
    assert_eq!(coordinator.state(), TtsState::Playing);

    coordinator.on_playback_complete().unwrap();
    assert_eq!(coordinator.state(), TtsState::Idle);
}
```

---

## 8. Platform-Gated Integration Tests

Integration tests that require real platform APIs (X11, Wayland, GPU) are gated behind
feature flags so they don't run in environments that lack those APIs.

```rust
// tests/x11_capture_integration.rs

#![cfg(feature = "integration_tests")]

#[test]
#[cfg(target_os = "linux")]
fn x11_screen_capture_real_display() {
    // This test requires a running X11 server
    let display = std::env::var("DISPLAY")
        .expect("DISPLAY not set — skip this test in headless CI");

    let capture = X11ScreenCapture::new(&display).unwrap();
    let frame = capture.capture_frame().unwrap();

    assert!(frame.width > 0);
    assert!(frame.height > 0);
    assert_eq!(frame.data.len(), (frame.width * frame.height * 4) as usize);
}
```

Run with: `cargo nextest run --features integration_tests`

In CI, these run only on runners with the appropriate platform (see doc-07 Section 3).

---

## 9. Performance and Regression Tests

Performance tests ensure code stays within the project's budgets (16ms frame time,
200ms TTS latency, etc.). They're not TDD in the strict sense — add them after
the core logic is working.

```rust
#[test]
fn viewport_calculation_under_budget() {
    let config = generate_test_capture_config_with_size(3840, 2160);

    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = calculate_viewport(&config, 2.0, (960.0, 540.0));
    }
    let elapsed = start.elapsed();

    // 1000 iterations in under 1ms = well within 16ms frame budget
    assert!(elapsed < std::time::Duration::from_millis(1),
        "viewport calculation took {:?} for 1000 iterations", elapsed);
}
```

---

## 10. Recommended Crate Stack

| Crate | Version | Purpose | When to Use |
|-------|---------|---------|-------------|
| `pretty_assertions` | 1.x | Better diff output on assertion failures | Add to every test module |
| `proptest` | 1.x | Property-based / fuzz testing | Algorithmic invariants, math |
| `rstest` | 0.26+ | Parameterized tests and fixtures | Multiple input/output pairs |
| `insta` | 1.x | Snapshot testing | Serialized data stability |
| `mockall` | 0.13+ | Auto-generated trait mocks | Interaction verification only |
| `assert_cmd` | 2.x | Testing CLI binaries | E2E smoke tests |
| `tokio-test` | (bundled) | Async test utilities | Channel/timeout patterns |

Add to `[dev-dependencies]` in the appropriate crate's `Cargo.toml`. These are
compile-time-only dependencies that don't affect the release binary size.

### pretty_assertions Global Setup

Add to every test module for readable diffs:

```rust
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    // ... your tests ...
}
```

This replaces the standard `assert_eq!` with one that shows colored diffs on failure,
making it much easier to spot what changed.
