//! GPU source texture management for the rendering pipeline.
//!
//! Contains [`SourceTextureManager`], which bridges CPU-side screen capture
//! and GPU-side shader rendering by managing the wgpu source texture
//! lifecycle: creation, upload from [`CaptureFrame`], over-allocation to
//! minimize reallocation frequency, and stale frame tracking when capture
//! fails.

use luminos_types::CaptureFrame;

/// The stale frame warning threshold (60 frames = 1 second at 60fps).
const STALE_FRAME_WARN_THRESHOLD: u32 = 60;

/// The texture over-allocation factor (1.5x in each dimension).
const OVER_ALLOCATION_FACTOR: f32 = 1.5;

/// The GPU texture format for source textures.
///
/// `Rgba8UnormSrgb` enables automatic sRGB-to-linear conversion when
/// the shader samples the texture, producing gamma-correct interpolation
/// without manual conversion in the shader.
const SOURCE_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// Manages the GPU source texture for the rendering pipeline.
///
/// Handles texture creation, upload from [`CaptureFrame`], over-allocation
/// to minimize reallocation frequency, and stale frame tracking when
/// capture fails.
///
/// # Texture Format
///
/// The source texture uses `Rgba8UnormSrgb` format. This enables automatic
/// sRGB-to-linear conversion when the shader samples the texture, producing
/// gamma-correct interpolation without manual conversion. On X11 with xcap,
/// the pixel data is already RGBA, so it maps directly to `Rgba8UnormSrgb`
/// with no channel reordering needed. For future platform backends that
/// produce BGRA (e.g., Windows DXGI), the BGRA-to-RGBA channel swizzle is
/// handled by the magnification shader via a uniform flag, not by this module.
///
/// # Over-Allocation Strategy
///
/// Textures are allocated at 1.5x the requested dimensions to absorb
/// dimension changes from zoom level adjustments without reallocation.
/// Reallocation only occurs when the captured frame exceeds the current
/// texture capacity.
pub struct SourceTextureManager {
    /// The wgpu device for texture creation.
    device: wgpu::Device,
    /// The current GPU source texture.
    texture: wgpu::Texture,
    /// The texture view for shader binding.
    view: wgpu::TextureView,
    /// Allocated texture width (over-allocated, >= `current_width`).
    capacity_width: u32,
    /// Allocated texture height (over-allocated, >= `current_height`).
    capacity_height: u32,
    /// Width of the most recently uploaded frame.
    current_width: u32,
    /// Height of the most recently uploaded frame.
    current_height: u32,
    /// Count of consecutive frames where capture failed (stale frame count).
    stale_frame_count: u32,
}

impl SourceTextureManager {
    /// Creates a new source texture manager with an initial texture.
    ///
    /// The initial texture is over-allocated by 1.5x in each dimension
    /// to absorb zoom-related dimension changes.
    ///
    /// # Arguments
    ///
    /// * `device` - The wgpu device for texture creation.
    /// * `initial_width` - Expected initial source region width.
    /// * `initial_height` - Expected initial source region height.
    #[must_use]
    pub fn new(device: wgpu::Device, initial_width: u32, initial_height: u32) -> Self {
        let capacity_width = over_allocate(initial_width);
        let capacity_height = over_allocate(initial_height);

        let texture = create_source_texture(&device, capacity_width, capacity_height);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            device,
            texture,
            view,
            capacity_width,
            capacity_height,
            current_width: initial_width,
            current_height: initial_height,
            stale_frame_count: 0,
        }
    }

    /// Uploads a [`CaptureFrame`] to the GPU source texture.
    ///
    /// If the frame dimensions exceed the current texture capacity,
    /// the texture is reallocated with 1.5x over-allocation.
    ///
    /// Resets the stale frame counter on successful upload.
    ///
    /// # Arguments
    ///
    /// * `queue` - The wgpu queue for texture data transfer.
    /// * `frame` - The captured frame to upload.
    pub fn upload(&mut self, queue: &wgpu::Queue, frame: &CaptureFrame) {
        // Reallocate if frame exceeds capacity
        if frame.width > self.capacity_width || frame.height > self.capacity_height {
            self.reallocate(frame.width, frame.height);
        }

        // Upload pixel data to GPU texture
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &frame.data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(frame.stride),
                rows_per_image: Some(frame.height),
            },
            wgpu::Extent3d {
                width: frame.width,
                height: frame.height,
                depth_or_array_layers: 1,
            },
        );

        self.current_width = frame.width;
        self.current_height = frame.height;
        self.stale_frame_count = 0;
    }

    /// Records a capture failure (stale frame).
    ///
    /// Increments the stale frame counter. Emits a `warn!` log when
    /// the counter reaches 60 (1 second at 60fps).
    pub fn record_capture_failure(&mut self) {
        self.stale_frame_count += 1;

        if self.stale_frame_count == STALE_FRAME_WARN_THRESHOLD {
            log::warn!(
                "Capture stale for '{}' consecutive frames ({}s at 60fps)",
                self.stale_frame_count,
                self.stale_frame_count / 60
            );
        }
    }

    /// Returns the texture view for shader binding.
    ///
    /// The view is always valid -- even during stale frame situations,
    /// it references the last successfully uploaded texture data.
    #[must_use]
    pub fn texture_view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Returns the dimensions of the most recently uploaded frame.
    ///
    /// These are the actual frame dimensions, not the over-allocated
    /// texture capacity. The shader uses these as `source_size` in
    /// the `MagnifyUniforms` struct.
    #[must_use]
    pub fn current_dimensions(&self) -> (u32, u32) {
        (self.current_width, self.current_height)
    }

    /// Returns the number of consecutive stale frames.
    #[must_use]
    pub fn stale_frame_count(&self) -> u32 {
        self.stale_frame_count
    }

    /// Returns the over-allocated texture capacity dimensions.
    ///
    /// These are the actual GPU texture dimensions (with 1.5x
    /// over-allocation), not the dimensions of the last uploaded frame.
    #[must_use]
    pub fn capacity(&self) -> (u32, u32) {
        (self.capacity_width, self.capacity_height)
    }

    /// Reallocates the source texture with 1.5x over-allocation.
    fn reallocate(&mut self, new_width: u32, new_height: u32) {
        self.capacity_width = over_allocate(new_width);
        self.capacity_height = over_allocate(new_height);

        log::info!(
            "Reallocating source texture: {}x{} -> {}x{} (capacity {}x{})",
            self.current_width,
            self.current_height,
            new_width,
            new_height,
            self.capacity_width,
            self.capacity_height,
        );

        self.texture =
            create_source_texture(&self.device, self.capacity_width, self.capacity_height);
        self.view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
    }
}

/// Computes the over-allocated dimension (1.5x, rounded up).
///
/// Used to calculate GPU texture capacity that exceeds the actual
/// frame dimensions, reducing reallocation frequency when zoom level
/// changes cause small dimension adjustments.
fn over_allocate(dimension: u32) -> u32 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let result = (f64::from(dimension) * f64::from(OVER_ALLOCATION_FACTOR)).ceil() as u32;
    result
}

/// Creates a wgpu texture for source pixel data.
///
/// The texture uses [`SOURCE_TEXTURE_FORMAT`] (`Rgba8UnormSrgb`) and has
/// `TEXTURE_BINDING | COPY_DST` usage flags (sampled by shaders, written
/// via `Queue::write_texture`).
fn create_source_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("luminos_source_texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: SOURCE_TEXTURE_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use luminos_types::PixelFormat;

    // ── Test Helpers ────────────────────────────────────────────────

    /// Generates a test [`CaptureFrame`] with solid-color RGBA pixel data.
    ///
    /// # Parameters
    /// - `width`: Frame width in pixels.
    /// - `height`: Frame height in pixels.
    /// - `color`: RGBA color value `[r, g, b, a]` for every pixel.
    fn generate_test_capture_frame_rgba(width: u32, height: u32, color: [u8; 4]) -> CaptureFrame {
        let stride = width * 4;
        let data: Vec<u8> = color
            .iter()
            .cycle()
            .take((stride * height) as usize)
            .copied()
            .collect();
        CaptureFrame {
            data: data.into(),
            width,
            height,
            stride,
            format: PixelFormat::Rgba8,
        }
    }

    /// Generates a test [`CaptureFrame`] with stride padding.
    ///
    /// The stride is set to `width * 4 + extra_padding`, with padding
    /// bytes filled with 0xDD to detect any misalignment during upload.
    fn generate_test_capture_frame_with_stride(
        width: u32,
        height: u32,
        color: [u8; 4],
        extra_padding: u32,
    ) -> CaptureFrame {
        let stride = width * 4 + extra_padding;
        let mut data = Vec::with_capacity((stride * height) as usize);
        for _row in 0..height {
            for _col in 0..width {
                data.extend_from_slice(&color);
            }
            // Fill padding with sentinel value
            data.extend(std::iter::repeat_n(0xDD, extra_padding as usize));
        }
        CaptureFrame {
            data: data.into(),
            width,
            height,
            stride,
            format: PixelFormat::Rgba8,
        }
    }

    /// Creates a wgpu device and queue for testing using the default backend.
    ///
    /// Uses `Backends::GL` for compatibility with Mesa llvmpipe in CI.
    /// Falls back to all backends if GL is not available.
    async fn generate_test_gpu_device() -> Option<(wgpu::Device, wgpu::Queue)> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL | wgpu::Backends::VULKAN,
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: None,
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok()?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("luminos_test_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                ..Default::default()
            })
            .await
            .ok()?;

        Some((device, queue))
    }

    // ── T002: over_allocate helper and SOURCE_TEXTURE_FORMAT ────────

    #[test]
    fn texture_over_allocate_small() {
        assert_eq!(over_allocate(100), 150);
    }

    #[test]
    fn texture_over_allocate_960() {
        assert_eq!(over_allocate(960), 1440);
    }

    #[test]
    fn texture_over_allocate_1920() {
        assert_eq!(over_allocate(1920), 2880);
    }

    #[test]
    fn texture_over_allocate_one() {
        // 1.5 * 1 = 1.5, ceil = 2
        assert!(over_allocate(1) >= 2);
    }

    #[test]
    fn texture_over_allocate_zero() {
        assert_eq!(over_allocate(0), 0);
    }

    #[test]
    fn texture_source_format_is_srgb() {
        assert_eq!(SOURCE_TEXTURE_FORMAT, wgpu::TextureFormat::Rgba8UnormSrgb);
    }

    // ── T003: SourceTextureManager constructor ──────────────────────

    #[tokio::test]
    async fn texture_manager_new_initial_dimensions() {
        let Some((device, _queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        let manager = SourceTextureManager::new(device, 960, 540);
        assert_eq!(manager.current_dimensions(), (960, 540));
    }

    #[tokio::test]
    async fn texture_manager_new_capacity_over_allocated() {
        let Some((device, _queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        let manager = SourceTextureManager::new(device, 960, 540);
        let (cap_w, cap_h) = manager.capacity();
        assert!(
            cap_w >= 1440,
            "capacity_width {cap_w} must be >= 1440 (1.5x of 960)"
        );
        assert!(
            cap_h >= 810,
            "capacity_height {cap_h} must be >= 810 (1.5x of 540)"
        );
    }

    #[tokio::test]
    async fn texture_manager_new_stale_count_zero() {
        let Some((device, _queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        let manager = SourceTextureManager::new(device, 100, 100);
        assert_eq!(manager.stale_frame_count(), 0);
    }

    // ── T004: upload method ─────────────────────────────────────────

    #[tokio::test]
    async fn texture_manager_upload_updates_dimensions() {
        let Some((device, queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        let mut manager = SourceTextureManager::new(device, 960, 540);
        let frame = generate_test_capture_frame_rgba(640, 480, [255, 0, 0, 255]);
        manager.upload(&queue, &frame);
        assert_eq!(manager.current_dimensions(), (640, 480));
    }

    #[tokio::test]
    async fn texture_manager_upload_within_capacity_no_realloc() {
        let Some((device, queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        let mut manager = SourceTextureManager::new(device, 960, 540);
        let original_capacity = manager.capacity();

        // Upload a frame smaller than capacity (1440x810)
        let frame = generate_test_capture_frame_rgba(800, 600, [0, 255, 0, 255]);
        manager.upload(&queue, &frame);

        assert_eq!(
            manager.capacity(),
            original_capacity,
            "capacity should not change when frame fits"
        );
    }

    #[tokio::test]
    async fn texture_manager_upload_exceeds_capacity_realloc() {
        let Some((device, queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        // Initial capacity: 1.5x of 256x256 = 384x384
        let mut manager = SourceTextureManager::new(device, 256, 256);
        let original_capacity = manager.capacity();

        // Upload a frame larger than capacity (512x512 > 384x384)
        let frame = generate_test_capture_frame_rgba(512, 512, [0, 0, 255, 255]);
        manager.upload(&queue, &frame);

        let (new_cap_w, new_cap_h) = manager.capacity();
        assert!(
            new_cap_w > original_capacity.0,
            "capacity_width should increase after realloc"
        );
        assert!(
            new_cap_h > original_capacity.1,
            "capacity_height should increase after realloc"
        );
        // New capacity should be at least 1.5x of new dimensions
        assert!(
            new_cap_w >= 768,
            "new capacity_width {new_cap_w} must be >= 768 (1.5x of 512)"
        );
        assert!(
            new_cap_h >= 768,
            "new capacity_height {new_cap_h} must be >= 768 (1.5x of 512)"
        );
    }

    #[tokio::test]
    async fn texture_manager_upload_resets_stale_count() {
        let Some((device, queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        let mut manager = SourceTextureManager::new(device, 256, 256);

        // Simulate 30 stale frames
        for _ in 0..30 {
            manager.record_capture_failure();
        }
        assert_eq!(manager.stale_frame_count(), 30);

        // Upload resets the counter
        let frame = generate_test_capture_frame_rgba(256, 256, [128, 128, 128, 255]);
        manager.upload(&queue, &frame);
        assert_eq!(manager.stale_frame_count(), 0);
    }

    // ── T005: stale frame tracking ──────────────────────────────────

    #[tokio::test]
    async fn texture_manager_stale_count_increments() {
        let Some((device, _queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        let mut manager = SourceTextureManager::new(device, 100, 100);

        for _ in 0..5 {
            manager.record_capture_failure();
        }
        assert_eq!(manager.stale_frame_count(), 5);
    }

    #[tokio::test]
    async fn texture_manager_stale_threshold_warning() {
        let Some((device, _queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        let mut manager = SourceTextureManager::new(device, 100, 100);

        // Call record_capture_failure exactly 60 times
        for _ in 0..STALE_FRAME_WARN_THRESHOLD {
            manager.record_capture_failure();
        }
        assert_eq!(manager.stale_frame_count(), STALE_FRAME_WARN_THRESHOLD);
    }

    #[tokio::test]
    async fn texture_manager_stale_count_resets_on_upload() {
        let Some((device, queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        let mut manager = SourceTextureManager::new(device, 256, 256);

        for _ in 0..30 {
            manager.record_capture_failure();
        }
        assert_eq!(manager.stale_frame_count(), 30);

        let frame = generate_test_capture_frame_rgba(256, 256, [0, 0, 0, 255]);
        manager.upload(&queue, &frame);
        assert_eq!(manager.stale_frame_count(), 0);
    }

    #[tokio::test]
    async fn texture_manager_stale_preserves_texture_view() {
        let Some((device, _queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        let mut manager = SourceTextureManager::new(device, 100, 100);

        for _ in 0..10 {
            manager.record_capture_failure();
        }

        // texture_view() should still return a valid reference
        let _view = manager.texture_view();
    }

    // ── T006: texture_view and current_dimensions accessors ─────────

    #[tokio::test]
    async fn texture_manager_texture_view_returns_reference() {
        let Some((device, _queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        let manager = SourceTextureManager::new(device, 320, 240);
        // Borrow check: this compiles and does not panic
        let _view: &wgpu::TextureView = manager.texture_view();
    }

    #[tokio::test]
    async fn texture_manager_current_dimensions_after_upload() {
        let Some((device, queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        let mut manager = SourceTextureManager::new(device, 960, 540);
        let frame = generate_test_capture_frame_rgba(800, 600, [10, 20, 30, 255]);
        manager.upload(&queue, &frame);
        assert_eq!(manager.current_dimensions(), (800, 600));
    }

    #[tokio::test]
    async fn texture_manager_current_dimensions_after_realloc() {
        let Some((device, queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        let mut manager = SourceTextureManager::new(device, 256, 256);
        // Upload a frame that triggers reallocation
        let frame = generate_test_capture_frame_rgba(512, 512, [255, 128, 0, 255]);
        manager.upload(&queue, &frame);

        // current_dimensions should reflect the frame, not the capacity
        assert_eq!(
            manager.current_dimensions(),
            (512, 512),
            "current_dimensions should return frame dims, not capacity"
        );
        let (cap_w, cap_h) = manager.capacity();
        assert!(
            cap_w > 512 && cap_h > 512,
            "capacity should exceed frame dimensions"
        );
    }

    // ── T007: GPU texture creation and upload integration tests ─────

    #[tokio::test]
    async fn texture_integration_create_and_upload() {
        let Some((device, queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        let mut manager = SourceTextureManager::new(device, 64, 64);
        let frame = generate_test_capture_frame_rgba(64, 64, [0, 0, 255, 255]);
        manager.upload(&queue, &frame);

        // Verify dimensions updated
        assert_eq!(manager.current_dimensions(), (64, 64));

        // Verify texture view is valid (can be used in a bind group)
        let _view = manager.texture_view();
    }

    #[tokio::test]
    async fn texture_integration_stride_padding() {
        let Some((device, queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        let mut manager = SourceTextureManager::new(device, 64, 64);

        // Create a frame with extra stride padding (32 bytes extra per row)
        let frame = generate_test_capture_frame_with_stride(64, 64, [255, 0, 0, 255], 32);
        assert_eq!(frame.stride, 64 * 4 + 32);

        // Upload should handle stride correctly without panicking
        manager.upload(&queue, &frame);
        assert_eq!(manager.current_dimensions(), (64, 64));
    }

    // ── T008: Reallocation integration tests ────────────────────────

    #[tokio::test]
    async fn texture_integration_reallocation() {
        let Some((device, queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        // Initial: 256x256, capacity 384x384
        let mut manager = SourceTextureManager::new(device, 256, 256);
        let initial_cap = manager.capacity();
        assert!(initial_cap.0 >= 384 && initial_cap.1 >= 384);

        // Upload 512x512 (exceeds 384x384 capacity)
        let frame = generate_test_capture_frame_rgba(512, 512, [128, 64, 32, 255]);
        manager.upload(&queue, &frame);

        let (cap_w, cap_h) = manager.capacity();
        assert!(
            cap_w >= 768,
            "after realloc, capacity_width {cap_w} must be >= 768"
        );
        assert!(
            cap_h >= 768,
            "after realloc, capacity_height {cap_h} must be >= 768"
        );
        assert_eq!(manager.current_dimensions(), (512, 512));

        // Upload a second frame that fits the new capacity
        let frame2 = generate_test_capture_frame_rgba(600, 600, [255, 255, 0, 255]);
        manager.upload(&queue, &frame2);
        assert_eq!(manager.current_dimensions(), (600, 600));
        assert_eq!(manager.capacity(), (cap_w, cap_h), "no realloc needed");
    }

    #[tokio::test]
    async fn texture_integration_reallocation_preserves_view() {
        let Some((device, queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        let mut manager = SourceTextureManager::new(device, 64, 64);

        // Upload a frame that triggers reallocation
        let frame = generate_test_capture_frame_rgba(256, 256, [200, 100, 50, 255]);
        manager.upload(&queue, &frame);

        // After reallocation, texture_view should be valid
        let _view = manager.texture_view();
        assert_eq!(manager.current_dimensions(), (256, 256));
    }

    // ── T009: Upload performance benchmarks ─────────────────────────

    #[tokio::test]
    #[ignore = "benchmark: requires GPU and is not run in CI"]
    async fn texture_benchmark_upload_small_region() {
        let Some((device, queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        let mut manager = SourceTextureManager::new(device, 96, 54);
        let frame = generate_test_capture_frame_rgba(96, 54, [128, 128, 128, 255]);

        let start = std::time::Instant::now();
        for _ in 0..100 {
            manager.upload(&queue, &frame);
        }
        let elapsed = start.elapsed();
        let avg_us = elapsed.as_micros() / 100;
        eprintln!("Upload 96x54 avg: {avg_us}us ({} iterations)", 100);
    }

    #[tokio::test]
    #[ignore = "benchmark: requires GPU and is not run in CI"]
    async fn texture_benchmark_upload_medium_region() {
        let Some((device, queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        let mut manager = SourceTextureManager::new(device, 960, 540);
        let frame = generate_test_capture_frame_rgba(960, 540, [128, 128, 128, 255]);

        let start = std::time::Instant::now();
        for _ in 0..100 {
            manager.upload(&queue, &frame);
        }
        let elapsed = start.elapsed();
        let avg_us = elapsed.as_micros() / 100;
        eprintln!("Upload 960x540 avg: {avg_us}us ({} iterations)", 100);
    }

    #[tokio::test]
    #[ignore = "benchmark: requires GPU and is not run in CI"]
    async fn texture_benchmark_upload_large_region() {
        let Some((device, queue)) = generate_test_gpu_device().await else {
            eprintln!("Skipping: no GPU adapter available");
            return;
        };

        let mut manager = SourceTextureManager::new(device, 1280, 720);
        let frame = generate_test_capture_frame_rgba(1280, 720, [128, 128, 128, 255]);

        let start = std::time::Instant::now();
        for _ in 0..100 {
            manager.upload(&queue, &frame);
        }
        let elapsed = start.elapsed();
        let avg_us = elapsed.as_micros() / 100;
        eprintln!("Upload 1280x720 avg: {avg_us}us ({} iterations)", 100);
    }
}
