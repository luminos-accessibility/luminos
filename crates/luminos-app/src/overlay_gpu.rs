//! wgpu surface and magnification rendering bound to the overlay window.
//!
//! [`OverlayGpu`] owns the wgpu objects tied to the transparent overlay window
//! and hosts a `luminos_gpu::Renderer` (built once in [`OverlayGpu::new`]).
//! Each active redraw, [`OverlayGpu::render`] feeds a freshly captured
//! [`CaptureFrame`] to `Renderer::render_frame` against the same surface
//! (story 003). When magnification is inactive, [`OverlayGpu::render_clear`]
//! presents a transparent frame instead.
//!
//! The surface is `Surface<'static>`: it is created from an **owned**
//! `tauri::WebviewWindow` clone (Arc-backed, `'static`, `HasWindowHandle +
//! HasDisplayHandle + Send + Sync`), and the original owned window is stored
//! next to the surface so the render target outlives it.
//!
//! `wgpu::Device` and `wgpu::Queue` are both `Clone` (Arc-backed). The
//! `Renderer` owns its own clones for the magnify path; [`OverlayGpu`] keeps
//! cloned `device`/`queue` of its own for surface (re)configuration and the
//! inactive clear path (story-003 §A).

use luminos_gpu::Renderer;
use luminos_gpu::device::{create_gpu_device, create_wgpu_instance};
use luminos_gpu::surface::configure_surface;
use luminos_types::{CaptureFrame, PixelFormat};

use crate::app_error::AppError;

/// Returns whether a captured frame's pixel format is BGRA (so the magnify
/// shader must swizzle). Derived from the frame, never assumed: the shipped
/// X11 capture yields `Rgba8` today, but Windows DXGI will yield `Bgra8`
/// (story-003 §D.3 / FR-2).
#[must_use]
pub fn is_bgra_format(format: PixelFormat) -> bool {
    matches!(format, PixelFormat::Bgra8)
}

/// Transparent clear color used for the overlay frame. Fully transparent so
/// nothing is painted over the desktop until story 003 draws the magnified
/// frame. (Pre-multiplied alpha: RGB must be 0 when alpha is 0.)
const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.0,
};

/// Owns the wgpu surface bound to the overlay window plus the magnification
/// [`Renderer`] (built once at construction).
pub struct OverlayGpu {
    surface: wgpu::Surface<'static>,
    /// Keeps the surface's render target (the owned overlay window) alive for
    /// as long as the surface exists. Never read directly.
    _window: tauri::WebviewWindow,
    /// Cloned device for surface (re)configuration on resize / `Lost` /
    /// `Outdated`. The `Renderer` owns its own clone.
    device: wgpu::Device,
    /// Cloned queue for the inactive (clear) path; the `Renderer` owns its own.
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    /// The E2 magnification renderer, built once and reused every frame. Owns
    /// the per-frame `FrameTimings` (recorded inside `render_frame`).
    renderer: Renderer,
    /// Target frame rate for the frame-timing summary (from settings).
    target_fps: u32,
}

impl OverlayGpu {
    /// Builds the wgpu surface/device/queue from an **owned** overlay window
    /// and the hosted magnification [`Renderer`].
    ///
    /// Passing the owned, `'static` window into `Instance::create_surface`
    /// yields a `Surface<'static>` that does not borrow the window. The
    /// `Renderer` is built once here from cloned device/queue and the surface
    /// format; `method` (bilinear/bicubic) is baked at construction.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Gpu`] if the surface, adapter, device, surface
    /// configuration, or renderer cannot be created.
    pub fn new(
        window: tauri::WebviewWindow,
        width: u32,
        height: u32,
        method: luminos_gpu::InterpolationMethod,
        target_fps: u32,
    ) -> Result<Self, AppError> {
        let instance = create_wgpu_instance();

        // `create_surface` takes ownership of the window clone, producing a
        // `Surface<'static>`. The original `window` is retained in `_window`.
        let surface = instance
            .create_surface(window.clone())
            .map_err(|e| AppError::Gpu(format!("create_surface failed: {e}")))?;
        // RISK-001 evidence: a `Surface<'static>` was built from the tao/Tauri
        // WebviewWindow's rwh-0.6 handle. This is the coexistence linchpin.
        log::info!("surface_created: wgpu Surface built from owned overlay WebviewWindow handle");

        // `create_gpu_device` is async; there is no tokio runtime on the loop
        // thread, so block on it with pollster (see PINNED_VERSIONS §1c).
        let (adapter, device, queue) =
            pollster::block_on(create_gpu_device(&instance, &surface))
                .map_err(|e| AppError::Gpu(format!("device init failed: {e}")))?;

        // Fifo is the universally-available present mode (vsync); story 003 may
        // negotiate Mailbox where supported. `configure_surface` returns the
        // exact config it applied; we store THAT as the single source of truth
        // for `resize`/`Lost`/`Outdated` recovery (no hand-rebuilt copy that
        // could drift from what was configured).
        let config = configure_surface(
            &surface,
            &adapter,
            &device,
            width,
            height,
            wgpu::PresentMode::Fifo,
        )
        .map_err(|e| AppError::Gpu(format!("surface config failed: {e}")))?;

        // Build the magnification Renderer ONCE (device/queue are Arc-backed
        // clones; OverlayGpu keeps its own for surface ops + the clear path).
        let renderer = Renderer::new(
            device.clone(),
            queue.clone(),
            config.format,
            config.width.max(1),
            config.height.max(1),
            method,
        )
        .map_err(|e| AppError::Gpu(format!("renderer init failed: {e}")))?;

        log::info!(
            "overlay surface ready: '{}x{}' format '{:?}' alpha '{:?}' method '{:?}'",
            config.width,
            config.height,
            config.format,
            config.alpha_mode,
            method,
        );

        Ok(Self {
            surface,
            _window: window,
            device,
            queue,
            config,
            renderer,
            target_fps,
        })
    }

    /// Magnifies and presents one captured frame against the overlay surface.
    ///
    /// `is_bgra` is derived from `frame.format` (never assumed). Frame timings
    /// are recorded inside `Renderer::render_frame` (FR-6, free). `Lost` /
    /// `Outdated` surfaces are reconfigured before retrying once.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Gpu`] if the swapchain surface texture cannot be
    /// acquired (after a reconfigure retry).
    pub fn render(&mut self, frame: &CaptureFrame) -> Result<(), AppError> {
        let is_bgra = is_bgra_format(frame.format);
        match self.renderer.render_frame(&self.surface, frame, is_bgra) {
            Ok(()) => Ok(()),
            Err(e) if is_recoverable_surface_error(&e) => {
                // Reconfigure to the cached size and retry once (Lost/Outdated).
                self.surface.configure(&self.device, &self.config);
                self.renderer
                    .render_frame(&self.surface, frame, is_bgra)
                    .map_err(|e| {
                        AppError::Gpu(format!("render_frame failed after reconfigure: {e}"))
                    })
            }
            Err(e) => Err(AppError::Gpu(format!("render_frame failed: {e}"))),
        }
    }

    /// Records a capture failure so the renderer reuses its last source texture
    /// (stale-frame handling, FR-7). Never panics.
    pub fn handle_capture_failure(&mut self) {
        self.renderer.handle_capture_failure();
    }

    /// Returns the current frame-timing summary for IPC exposure (story 005).
    #[must_use]
    pub fn frame_timing_summary(&self) -> luminos_gpu::FrameTimingSummary {
        self.renderer.frame_timings().summary(self.target_fps)
    }

    /// Returns the overlay surface dimensions `(width, height)`, used as the
    /// magnification viewport size for region computation.
    #[must_use]
    pub fn viewport_size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// Presents one transparent clear frame (inactive / toggle-off path).
    ///
    /// Kept private-in-crate: the loop calls this when magnification is
    /// inactive so the overlay shows the desktop, not stale magnified content.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Gpu`] if the swapchain texture cannot be acquired.
    /// `Lost`/`Outdated` surfaces are reconfigured and retried once.
    pub fn render_clear(&mut self) -> Result<(), AppError> {
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                // Reconfigure to the cached size and retry once.
                self.surface.configure(&self.device, &self.config);
                match self.surface.get_current_texture() {
                    wgpu::CurrentSurfaceTexture::Success(texture)
                    | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
                    status => {
                        return Err(AppError::Gpu(format!(
                            "surface texture unavailable after reconfigure: {status:?}"
                        )));
                    }
                }
            }
            status => {
                return Err(AppError::Gpu(format!(
                    "surface texture unavailable: {status:?}"
                )));
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        encode_clear(&self.device, &self.queue, &view, CLEAR_COLOR);
        output.present();
        Ok(())
    }

    /// Reconfigures the surface and the renderer's viewport after a resize.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(&self.device, &self.config);
        // Keep the renderer's viewport in lock-step so the magnify shader's
        // uniforms match the surface dimensions.
        self.renderer.resize(self.config.width, self.config.height);
        log::debug!("overlay surface resized to '{width}x{height}'");
    }
}

/// Returns whether a render error is a recoverable surface state (`Lost` or
/// `Outdated`), which a single reconfigure-and-retry can clear.
fn is_recoverable_surface_error(error: &luminos_gpu::error::RenderError) -> bool {
    matches!(
        error,
        luminos_gpu::error::RenderError::SurfaceTexture { message }
            if message.contains("Lost") || message.contains("Outdated")
    )
}

/// Encodes and submits a single clear pass against `view`.
///
/// Extracted so the clear/submit logic can be unit-tested headlessly against
/// an offscreen `TextureView` (no window surface required).
fn encode_clear(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    view: &wgpu::TextureView,
    color: wgpu::Color,
) {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("overlay_clear_encoder"),
    });
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("overlay_clear_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    queue.submit(std::iter::once(encoder.finish()));
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    /// Builds a headless device+queue using the GL backend (Mesa llvmpipe in
    /// CI). Returns `None` if no adapter is available so the test gracefully
    /// skips on machines without a software renderer.
    fn try_headless_device() -> Option<(wgpu::Device, wgpu::Queue)> {
        let instance = create_wgpu_instance();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .ok()?;
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("overlay_test_device"),
            required_features: wgpu::Features::empty(),
            required_limits:
                wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            ..Default::default()
        }))
        .ok()?;
        Some((device, queue))
    }

    // ── T003: is_bgra derived from CaptureFrame.format ────────────────

    #[test]
    fn overlay_gpu_is_bgra_true_for_bgra8() {
        assert!(
            is_bgra_format(luminos_types::PixelFormat::Bgra8),
            "Bgra8 frames must set is_bgra=true (shader swizzle)"
        );
    }

    #[test]
    fn overlay_gpu_is_bgra_false_for_rgba8() {
        assert!(
            !is_bgra_format(luminos_types::PixelFormat::Rgba8),
            "Rgba8 frames (shipped X11 capture) must set is_bgra=false"
        );
    }

    // ── T002: Renderer integration + frame-timing summary seam ────────

    #[test]
    #[cfg_attr(
        not(feature = "ci_platform_tests"),
        ignore = "requires a wgpu adapter (Mesa llvmpipe); enable with ci_platform_tests"
    )]
    fn overlay_gpu_renderer_summary_zeroed_before_render() {
        let Some((device, queue)) = try_headless_device() else {
            eprintln!("skipping: no wgpu adapter available (no Mesa llvmpipe?)");
            return;
        };

        // Build the same Renderer OverlayGpu hosts, proving construction works
        // headlessly and the frame-timing summary seam (story 005) is reachable.
        let renderer = luminos_gpu::Renderer::new(
            device,
            queue,
            wgpu::TextureFormat::Bgra8UnormSrgb,
            64,
            64,
            luminos_gpu::InterpolationMethod::Bilinear,
        )
        .expect("Renderer::new should succeed headlessly");

        let summary = renderer.frame_timings().summary(60);
        assert_eq!(summary.target_fps, 60);
        assert!(
            summary.p99_ms.abs() < f64::EPSILON,
            "p99 should be zero before any frame is recorded, got {}",
            summary.p99_ms
        );
    }

    #[test]
    #[cfg_attr(
        not(feature = "ci_platform_tests"),
        ignore = "requires a wgpu adapter (Mesa llvmpipe); enable with ci_platform_tests"
    )]
    fn overlay_gpu_offscreen_render_clear() {
        let Some((device, queue)) = try_headless_device() else {
            eprintln!("skipping: no wgpu adapter available (no Mesa llvmpipe?)");
            return;
        };

        // Offscreen render target standing in for the swapchain texture.
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen_target"),
            size: wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        // The clear/submit path must not panic and must drain the queue.
        encode_clear(&device, &queue, &view, CLEAR_COLOR);
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .unwrap();
    }
}
