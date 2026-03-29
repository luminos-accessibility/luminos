# Design: Story E02/005 -- Render Loop, Frame Pacing & CI

**Story:** [STORY.md](./STORY.md)
**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** APPROVED
**Author:** Spec Writer Agent
**Risk Refs:** [RISK-002](../../tech-strategy/10-risk-register.md#risk-002-self-capture-infinite-feedback-loop) (self-capture, validated here), [RISK-004](../../tech-strategy/10-risk-register.md#risk-004-render-thread-starvation-under-load) (render thread starvation), [RISK-007](../../tech-strategy/10-risk-register.md#risk-007-x11-capture-bottleneck-at-low-zoom-on-high-resolution-displays) (X11 capture bottleneck), [RISK-027](../../tech-strategy/10-risk-register.md#risk-027-ci-pipeline-performance-and-platform-coverage-gaps) (CI pipeline gaps)

---

## Overview

This design assembles the complete rendering pipeline from the components delivered by Stories 001-004. The `Renderer` struct orchestrates the per-frame cycle: capture screen pixels via `XcbCapture`, upload them via `SourceTextureManager`, execute the magnification shader via `MagnifyPipeline`, and present the result on the swap chain surface. The `FrameTimings` ring buffer tracks frame-to-frame performance, enabling degradation detection.

The render loop is driven by winit's event loop in `RedrawRequested` mode with `PresentMode::Fifo` (vsync). This is a standalone E02 demo loop -- the full application event loop with `EventLoopProxy`, `ArcSwap<AppState>`, and Tauri integration comes in E03/E04.

The CI pipeline is extended with two new jobs: `test-platform` (X11 capture tests under Xvfb) and `test-gpu` (shader compilation and rendering tests under Xvfb with Mesa llvmpipe/lavapipe). These validate that the rendering pipeline works in headless CI environments. The `mesa-vulkan-drivers` package provides lavapipe (software Vulkan), enabling wgpu's Vulkan backend in CI.

**RISK-004 monitoring:** The `FrameTimings` struct is the primary instrument for detecting render thread starvation. P99 thresholds (20ms warn, 33ms critical) are checked after each frame recording. This story establishes the measurement infrastructure; the actual degradation response (control panel notification, performance mode switch) comes in E04/E05.

**RISK-027 mitigation:** Adding Xvfb, Mesa llvmpipe (GL), and Mesa lavapipe (Vulkan via `mesa-vulkan-drivers`) to CI closes the "no GPU testing in CI" gap identified in the risk register.

## Architecture

### Component Diagram

```
crates/luminos-gpu/src/
  |
  +-- lib.rs                     # Add: pub mod renderer; pub mod frame_timings;
  |
  +-- renderer.rs                # Renderer struct, render_frame(), resize() (NEW)
  +-- frame_timings.rs           # FrameTimings ring buffer, FrameTimingSummary (NEW)
  |
  +-- texture.rs                 # SourceTextureManager (Story 003)
  +-- device.rs                  # GPU device/queue creation (Story 002)
  +-- viewport.rs                # compute_source_region() (Story 004)
  +-- shaders/                   # MagnifyPipeline, MagnifyUniforms (Story 004)
  |
  +-- tests/
        +-- integration.rs       # render_pipeline_capture_to_present test (NEW)

.github/workflows/
  +-- ci.yml                     # Add: test-platform, test-gpu jobs (MODIFIED)
```

### Affected Traits / Modules

| Trait / Module | Change Type | Description |
|----------------|-------------|-------------|
| `luminos-gpu::renderer` | New | `Renderer` struct orchestrating the full pipeline |
| `luminos-gpu::frame_timings` | New | `FrameTimings` ring buffer and `FrameTimingSummary` |
| `luminos-gpu::lib` | Modified | Add `pub mod renderer;` and `pub mod frame_timings;` |
| `.github/workflows/ci.yml` | Modified | Add `test-platform` and `test-gpu` CI jobs |

### Data Flow

```
Every frame (~16.67ms at 60fps):

  winit RedrawRequested event
       |
       v
  1. Compute source region:
     viewport::compute_source_region(cursor_pos, zoom, viewport_size, screen_bounds)
       -> ScreenRect
       |
       v
  2. Capture screen pixels:
     xcb_capture.capture_frame(display_id, Some(source_rect))
       -> Result<CaptureFrame, CaptureError>
       |
       +-- Ok(frame) -----> 3. Upload to GPU
       +-- Err(_) ---------> Renderer::handle_capture_failure() -> render stale frame
       |
       v
  3. Upload to GPU texture:
     source_texture_manager.upload(queue, &frame)
       (may trigger reallocation if dimensions changed)
       |
       v
  4. Create per-frame bind group:
     device.create_bind_group(source_texture_view + sampler + uniforms_buffer)
       |
       v
  5. Encode render pass:
     encoder.begin_render_pass(swap_chain_texture_view)
       .set_pipeline(magnify_pipeline)
       .set_bind_group(0, bind_group)
       .draw(0..3, 0..1)  // full-screen triangle
       |
       v
  6. Submit and present:
     queue.submit([encoder.finish()])
     surface_texture.present()
       |
       v
  7. Record frame timing:
     frame_timings.record(frame_duration)
     Check performance thresholds
```

---

## API Design

### `FrameTimings` -- `crates/luminos-gpu/src/frame_timings.rs`

```rust
use std::time::Duration;

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

/// Performance threshold: P99 > 20ms triggers warning.
const WARN_THRESHOLD: Duration = Duration::from_millis(20);

/// Performance threshold: P99 > 33ms triggers critical alert.
const CRITICAL_THRESHOLD: Duration = Duration::from_millis(33);

/// Duration (in frame recordings) before threshold alerts fire.
/// 300 recordings at 60fps = 5 seconds.
const THRESHOLD_STREAK_LIMIT: u32 = 300;

/// IPC-ready frame timing summary.
///
/// Contains aggregate statistics suitable for transmission to the
/// control panel via Tauri IPC (E04+).
#[derive(Debug, Clone, PartialEq)]
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

    /// Returns the P99 frame time over the last 120 frames.
    pub fn p99(&self) -> Duration {
        if self.count == 0 {
            return Duration::ZERO;
        }
        let mut sorted: Vec<Duration> = self.history[..self.count].to_vec();
        sorted.sort_unstable();
        // 99th percentile: index ceil(0.99 * count) - 1
        let idx = ((self.count as f64 * 0.99).ceil() as usize).saturating_sub(1);
        sorted[idx.min(self.count - 1)]
    }

    /// Returns the average frame time over recorded frames.
    pub fn average(&self) -> Duration {
        if self.count == 0 {
            return Duration::ZERO;
        }
        let sum: Duration = self.history[..self.count].iter().sum();
        sum / self.count as u32
    }

    /// Returns the minimum frame time over recorded frames.
    pub fn min(&self) -> Duration {
        self.history[..self.count]
            .iter()
            .copied()
            .min()
            .unwrap_or(Duration::ZERO)
    }

    /// Returns the maximum frame time over recorded frames.
    pub fn max(&self) -> Duration {
        self.history[..self.count]
            .iter()
            .copied()
            .max()
            .unwrap_or(Duration::ZERO)
    }

    /// Creates a `FrameTimingSummary` for IPC reporting.
    pub fn summary(&self, target_fps: u32) -> FrameTimingSummary {
        FrameTimingSummary {
            average_ms: self.average().as_secs_f64() * 1000.0,
            p99_ms: self.p99().as_secs_f64() * 1000.0,
            min_ms: self.min().as_secs_f64() * 1000.0,
            max_ms: self.max().as_secs_f64() * 1000.0,
            target_fps,
        }
    }

    /// Checks performance thresholds and logs warnings.
    fn check_thresholds(&mut self, p99: Duration) {
        // Warn threshold
        if p99 > WARN_THRESHOLD {
            self.warn_streak += 1;
            if self.warn_streak == THRESHOLD_STREAK_LIMIT {
                log::warn!(
                    "Performance degradation: P99 frame time '{}ms' exceeded '{}ms' threshold for 5 seconds",
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
                    "Critical performance degradation: P99 frame time '{}ms' exceeded '{}ms' threshold for 5 seconds",
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
```

### `Renderer` -- `crates/luminos-gpu/src/renderer.rs`

```rust
use luminos_platform::traits::screen_capture::CaptureError;
use luminos_platform::traits::types::CaptureFrame;

use crate::frame_timings::FrameTimings;
use crate::shaders::{InterpolationMethod, MagnifyPipeline, MagnifyUniforms};
use crate::texture::SourceTextureManager;

/// Error types for the rendering pipeline.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// The swap chain surface texture could not be acquired.
    #[error("surface texture acquisition failed: {message}")]
    SurfaceError { message: String },

    /// The wgpu adapter could not be found.
    #[error("no suitable GPU adapter found")]
    NoAdapter,

    /// Device creation failed.
    #[error("GPU device creation failed: {message}")]
    DeviceCreation { message: String },

    /// Shader compilation failed.
    #[error("shader compilation failed: {message}")]
    ShaderCompilation { message: String },
}

/// Holds all persistent GPU resources for the rendering pipeline.
///
/// Orchestrates the per-frame capture-upload-render-present cycle.
/// Created once at startup; reused every frame.
pub struct Renderer {
    /// wgpu device for resource creation.
    device: wgpu::Device,
    /// wgpu queue for command submission.
    queue: wgpu::Queue,
    /// The magnification shader pipeline (bilinear or bicubic).
    magnify_pipeline: MagnifyPipeline,
    /// Source texture manager (upload, reallocation, stale tracking).
    source_texture_manager: SourceTextureManager,
    /// Frame timing ring buffer.
    frame_timings: FrameTimings,
    /// Texture sampler for the magnification shader.
    sampler: wgpu::Sampler,
    /// Surface format (for resize reconfiguration).
    surface_format: wgpu::TextureFormat,
    /// Current viewport width.
    viewport_width: u32,
    /// Current viewport height.
    viewport_height: u32,
}

impl Renderer {
    /// Creates a new renderer with all GPU resources initialized.
    ///
    /// # Arguments
    ///
    /// * `device` - The wgpu device.
    /// * `queue` - The wgpu queue.
    /// * `surface_format` - The swap chain surface texture format.
    /// * `viewport_width` - Initial overlay viewport width.
    /// * `viewport_height` - Initial overlay viewport height.
    /// * `method` - The interpolation method (Bilinear or Bicubic).
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::ShaderCompilation`] if shader compilation fails.
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        viewport_width: u32,
        viewport_height: u32,
        method: InterpolationMethod,
    ) -> Result<Self, RenderError> {
        let bind_group_layout = crate::shaders::create_magnify_bind_group_layout(&device);
        let magnify_pipeline = crate::shaders::create_magnify_pipeline(
            &device,
            surface_format,
            &bind_group_layout,
            method,
        )?;

        // Initial source region estimate: half viewport at 2x zoom
        let source_texture_manager = SourceTextureManager::new(
            device.clone(),
            viewport_width / 2,
            viewport_height / 2,
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("luminos_magnify_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        Ok(Self {
            device,
            queue,
            magnify_pipeline,
            source_texture_manager,
            frame_timings: FrameTimings::new(),
            sampler,
            surface_format,
            viewport_width,
            viewport_height,
        })
    }

    /// Executes one frame of the rendering pipeline.
    ///
    /// Uploads the `CaptureFrame` to the GPU source texture, creates a
    /// per-frame bind group, encodes the magnification render pass, and
    /// presents the result to the swap chain surface.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::SurfaceError`] if the swap chain surface
    /// texture cannot be acquired.
    pub fn render_frame(
        &mut self,
        surface: &wgpu::Surface<'_>,
        frame: &CaptureFrame,
        is_bgra: bool,
    ) -> Result<(), RenderError> {
        let frame_start = std::time::Instant::now();

        // Upload capture frame to GPU
        self.source_texture_manager.upload(&self.queue, frame);

        // Acquire swap chain surface texture
        let output = surface.get_current_texture().map_err(|e| {
            RenderError::SurfaceError {
                message: format!("{e}"),
            }
        })?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Update uniforms
        let (source_w, source_h) = self.source_texture_manager.current_dimensions();
        let uniforms = MagnifyUniforms {
            viewport_size: [self.viewport_width as f32, self.viewport_height as f32],
            source_size: [source_w as f32, source_h as f32],
            is_bgra: if is_bgra { 1.0 } else { 0.0 },
            _pad: [0.0; 3],
        };
        self.queue.write_buffer(
            &self.magnify_pipeline.uniform_buffer,
            0,
            bytemuck::bytes_of(&uniforms),
        );

        // Create per-frame bind group
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("magnify_bind_group"),
            layout: &self.magnify_pipeline.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        self.source_texture_manager.texture_view(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.magnify_pipeline.uniform_buffer.as_entire_binding(),
                },
            ],
        });

        // Encode render pass
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("magnify_encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("magnify_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.magnify_pipeline.pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..3, 0..1); // Full-screen triangle
        }

        // Submit and present
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // Record frame timing
        self.frame_timings.record(frame_start.elapsed());

        Ok(())
    }

    /// Handles a capture failure by rendering the stale frame.
    ///
    /// Delegates to `SourceTextureManager::record_capture_failure()` for
    /// stale frame tracking. The next `render_frame()` call with a valid
    /// `CaptureFrame` will reset the stale state.
    pub fn handle_capture_failure(&mut self) {
        self.source_texture_manager.record_capture_failure();
    }

    /// Handles a window resize by updating the viewport dimensions.
    ///
    /// The caller is responsible for reconfiguring the wgpu surface
    /// with the new dimensions before calling this method.
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width > 0 && new_height > 0 {
            self.viewport_width = new_width;
            self.viewport_height = new_height;
        }
    }

    /// Returns a reference to the frame timings for performance monitoring.
    pub fn frame_timings(&self) -> &FrameTimings {
        &self.frame_timings
    }
}
```

### CI Pipeline Additions -- `.github/workflows/ci.yml`

```yaml
  test-platform:
    name: Platform Tests (X11)
    runs-on: ubuntu-latest
    needs: [lint]
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Install system dependencies
        run: >
          sudo apt-get update && sudo apt-get install -y
          xvfb
          picom
          mesa-utils
          mesa-vulkan-drivers
          libegl-dev
          libgl1-mesa-dri
          libpipewire-0.3-dev
          libasound2-dev
          libx11-dev
          libxi-dev
          libxtst-dev

      - name: Cache Cargo registry and build artifacts
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: platform-${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            platform-${{ runner.os }}-cargo-

      - name: Install cargo-nextest
        uses: taiki-e/install-action@nextest

      - name: Run platform tests under Xvfb
        run: >
          xvfb-run -s "-screen 0 1920x1080x24" bash -c
          "picom --backend xrender --daemon && cargo nextest run
          --profile ci
          -p luminos-platform
          --features ci_platform_tests"

  test-gpu:
    name: GPU Tests (Mesa llvmpipe)
    runs-on: ubuntu-latest
    needs: [lint]
    env:
      MESA_GL_VERSION_OVERRIDE: "4.5"
      LIBGL_ALWAYS_SOFTWARE: "1"
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Install system dependencies
        run: >
          sudo apt-get update && sudo apt-get install -y
          xvfb
          picom
          mesa-utils
          mesa-vulkan-drivers
          libegl-dev
          libgl1-mesa-dri
          libpipewire-0.3-dev
          libasound2-dev
          libx11-dev
          libxi-dev
          libxtst-dev

      - name: Cache Cargo registry and build artifacts
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: gpu-${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            gpu-${{ runner.os }}-cargo-

      - name: Install cargo-nextest
        uses: taiki-e/install-action@nextest

      - name: Run GPU tests under Xvfb
        run: >
          xvfb-run -s "-screen 0 1920x1080x24" bash -c
          "picom --backend xrender --daemon && cargo nextest run
          --profile ci
          -p luminos-gpu"
```

---

## Error Handling

| Error Scenario | Error Type | Recovery |
|----------------|-----------|----------|
| Swap chain surface lost | `RenderError::SurfaceError` | Caller reconfigures surface and retries |
| Capture failure | `CaptureError` (from Story 001) | `handle_capture_failure()` renders stale frame |
| GPU adapter not found | `RenderError::NoAdapter` | Application exits with descriptive message |
| Shader compilation failure | `RenderError::ShaderCompilation` | Application exits (fatal: shaders are embedded) |

The `Renderer::render_frame()` method returns `Result<(), RenderError>`. The render loop caller handles `SurfaceError` by reconfiguring the surface (similar to `winit::WindowEvent::Resized`). Other errors are fatal and propagated to the application level.

---

## Platform Considerations

| Platform | Approach | Notes |
|----------|----------|-------|
| Linux X11 | **This story.** Full pipeline on Vulkan. | Primary target. Tested on Xvfb + Mesa llvmpipe in CI. |
| Linux Wayland | Same `Renderer`, different `ScreenCapture` impl. | No changes to this story's code. |
| macOS | Same `Renderer`, Metal backend. | No changes. `is_bgra` flag handles pixel format. |
| OpenBSD | Same `Renderer`, Vulkan backend. | No changes. |
| Windows | Same `Renderer`, DX12/Vulkan backend. | No changes. |

The `Renderer` is platform-independent. Only the `ScreenCapture` implementation and the `is_bgra` flag are platform-specific.

---

## Testing Strategy

### Unit Tests

Unit tests require no GPU or display server:

- **`FrameTimings::record()`:** Record 120 durations, verify circular buffer wraps correctly.
- **`FrameTimings::p99()`:** Record 120 durations with known distribution, verify P99 value.
- **`FrameTimings::average()`:** Record known durations, verify average.
- **`FrameTimings::min()` / `max()`:** Record known durations, verify extremes.
- **`FrameTimings::summary()`:** Verify all fields populated correctly.
- **Threshold detection:** Record durations exceeding thresholds, verify streak counters.
- **Threshold reset:** Record sub-threshold durations, verify streak resets.
- **Empty buffer:** Verify `p99()`, `average()`, `min()`, `max()` return `Duration::ZERO` when no frames recorded.

### Integration Tests

Integration tests require wgpu (Mesa llvmpipe in CI) and Xvfb:

- **Full pipeline:** Create `XcbCapture` + `Renderer`, capture a frame, render it, verify output is non-blank.
- **Shader selection:** Create `Renderer` with bilinear, then with bicubic, verify both render successfully.
- **Resize:** Create `Renderer`, render a frame, call `resize()`, render another frame at new dimensions.
- **Stale frame:** Create `Renderer`, render a frame, call `handle_capture_failure()`, verify `texture_view()` still valid.

### Acceptance Tests

| Acceptance Criterion | Test Type | Verification Method |
|---------------------|-----------|-------------------|
| AC-1.1 | Integration | Full pipeline test: capture + upload + render + present |
| AC-1.2 | Integration | Render 120 frames under Fifo vsync, verify ~60fps (relaxed for CI) |
| AC-1.3 | Unit | Record 120 frame times, verify `p99()` < 20ms |
| AC-2.1 | Unit | Record known durations, verify `p99()` correctness |
| AC-2.2 | Unit | Record known durations, verify `average()`, `min()`, `max()` |
| AC-2.3 | Unit | Verify `summary()` fields match individual method outputs |
| AC-2.4 | Unit | Record >20ms durations 300 times, verify warn streak counter |
| AC-2.5 | Unit | Record >33ms durations 300 times, verify critical streak counter |
| AC-3.1 | Integration | Render frame, then `handle_capture_failure()`, render stale, verify non-blank |
| AC-3.2 | Integration | After failures, upload new frame, verify fresh render |
| AC-4.1 | Integration | Create Renderer with Bilinear, render successfully |
| AC-4.2 | Integration | Create Renderer with Bicubic, render successfully |
| AC-5.1 | Integration | Trigger resize, verify surface reconfigured |
| AC-5.2 | Integration | Render after resize, verify output fills new dimensions |
| AC-6.1 | CI | `test-platform` job runs under Xvfb with 1920x1080 screen |
| AC-6.2 | CI | `test-gpu` job runs with Mesa llvmpipe |
| AC-6.3 | CI | `test-platform` executes `cargo nextest run -p luminos-platform --features ci_platform_tests` |
| AC-6.4 | CI | `test-gpu` executes `cargo nextest run -p luminos-gpu` under Xvfb |
| AC-7.1 | Integration | `render_pipeline_capture_to_present` test completes without error |
| AC-7.2 | Integration | Output frame has non-zero pixels and correct dimensions |
| AC-7.3 | Integration | `FrameTimings` has at least one recorded frame |

---

## Performance Targets

| Metric | Target | Source | Measurement |
|--------|--------|--------|-------------|
| Full pipeline frame time (2x zoom, 1080p) | < 16.67ms (60fps) | doc-03 Section 2.3 | `FrameTimings::record()` |
| P99 frame time (2x zoom, 1080p) | < 20ms | doc-03 Section 8.3 | `FrameTimings::p99()` |
| `FrameTimings::p99()` computation | < 10us | NFR-2 | Benchmark |
| CI pipeline addition | < 5 min wall time | NFR-6 | GitHub Actions timing |

---

## Security Considerations

- **RISK-017:** The `Renderer` processes screen content as GPU textures. No pixel data is logged. Frame timing data (durations, percentiles) does not contain screen content and is safe to log/expose.
- **CI secrets:** No secrets or credentials are needed for the new CI jobs. Mesa llvmpipe is a software renderer that requires no GPU driver authentication.

---

## Alternatives Considered

1. **Async render loop with tokio:** Would allow the render loop to yield during vsync wait. Decision: rejected. The render loop runs on the main thread (winit requirement) and is not I/O-bound. Vsync idle time is handled by the GPU driver, not by the application.

2. **Separate capture thread:** Would decouple capture from rendering. Decision: deferred to E03/E04 where the full event loop architecture with `EventLoopProxy` is implemented. The Phase 0 sequential pipeline is simpler and sufficient.

3. **`FrameTimings` using `VecDeque` instead of fixed array:** Would allow variable history size. Decision: rejected. Fixed 120-element array is cache-friendly, avoids allocation, and matches the 2-second window at 60fps. The size is a design constant, not a runtime parameter.

4. **Benchmark CI on self-hosted runners:** Would provide consistent performance baselines. Decision: deferred per STORY.md out-of-scope. `ubuntu-latest` runners have variable performance, making strict benchmark assertions unreliable. CI benchmarks use relaxed thresholds; accurate benchmarks require dedicated hardware.
