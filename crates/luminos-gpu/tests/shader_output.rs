//! Shader output integration tests.
//!
//! These tests render known source textures through the magnification shaders
//! and read back the output pixels to verify correctness. They require a
//! wgpu-compatible GPU or software renderer (Mesa llvmpipe).

#![allow(
    clippy::unwrap_used,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::similar_names,
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use luminos_gpu::shaders::{
    InterpolationMethod, MagnifyUniforms, create_magnify_bind_group,
    create_magnify_bind_group_layout, create_magnify_pipeline, create_magnify_sampler,
};

/// Headless GPU test harness for shader output verification.
struct GpuTestHarness {
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl GpuTestHarness {
    /// Creates a headless GPU test harness. Returns `None` if no adapter available.
    async fn new() -> Option<Self> {
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
                label: Some("test_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                ..Default::default()
            })
            .await
            .ok()?;

        Some(Self { device, queue })
    }

    /// Creates a source texture with the given RGBA pixel data.
    fn create_source_texture(&self, width: u32, height: u32, data: &[u8]) -> wgpu::Texture {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("test_source_texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        texture
    }

    /// Creates an output texture that can be used as a render target and read back.
    fn create_output_texture(&self, width: u32, height: u32) -> wgpu::Texture {
        self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("test_output_texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    }

    /// Renders a source texture through the magnification pipeline and reads
    /// back the output pixels.
    #[allow(clippy::unused_async)]
    async fn render_and_readback(
        &self,
        source_width: u32,
        source_height: u32,
        source_data: &[u8],
        output_width: u32,
        output_height: u32,
        is_bgra: f32,
        method: InterpolationMethod,
    ) -> Vec<u8> {
        let source_texture = self.create_source_texture(source_width, source_height, source_data);
        let source_view = source_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let output_texture = self.create_output_texture(output_width, output_height);
        let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let layout = create_magnify_bind_group_layout(&self.device);
        let pipeline = create_magnify_pipeline(
            &self.device,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            &layout,
            method,
        )
        .unwrap();

        let sampler = create_magnify_sampler(&self.device);

        let uniforms = MagnifyUniforms {
            viewport_size: [output_width as f32, output_height as f32],
            source_size: [source_width as f32, source_height as f32],
            is_bgra,
            _pad: [0.0; 3],
        };
        self.queue
            .write_buffer(&pipeline.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let bind_group = create_magnify_bind_group(
            &self.device,
            &pipeline.bind_group_layout,
            &source_view,
            &sampler,
            &pipeline.uniform_buffer,
        );

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("test_encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("test_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&pipeline.pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        // Copy output texture to a readback buffer.
        let bytes_per_row = align_to(output_width * 4, 256);
        let readback_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("test_readback_buffer"),
            size: u64::from(bytes_per_row * output_height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &output_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(output_height),
                },
            },
            wgpu::Extent3d {
                width: output_width,
                height: output_height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        // Map the readback buffer and extract pixel data.
        let buffer_slice = readback_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        self.device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .ok();
        rx.recv().unwrap().unwrap();

        let mapped = buffer_slice.get_mapped_range();

        // Remove row padding to get tightly packed pixel data.
        let mut pixels = Vec::with_capacity((output_width * output_height * 4) as usize);
        for row in 0..output_height {
            let start = (row * bytes_per_row) as usize;
            let end = start + (output_width * 4) as usize;
            pixels.extend_from_slice(&mapped[start..end]);
        }

        pixels
    }
}

/// Aligns `value` up to the nearest multiple of `alignment`.
fn align_to(value: u32, alignment: u32) -> u32 {
    value.div_ceil(alignment) * alignment
}

/// Creates a solid color RGBA pixel buffer.
fn solid_color_pixels(width: u32, height: u32, r: u8, g: u8, b: u8, a: u8) -> Vec<u8> {
    let pixel_count = (width * height) as usize;
    let mut data = Vec::with_capacity(pixel_count * 4);
    for _ in 0..pixel_count {
        data.extend_from_slice(&[r, g, b, a]);
    }
    data
}

// --- Bilinear shader output tests ---

#[tokio::test]
async fn shader_bilinear_solid_red_rgba() {
    let Some(harness) = GpuTestHarness::new().await else {
        eprintln!("skipping test: no GPU adapter available");
        return;
    };

    let source_data = solid_color_pixels(64, 64, 255, 0, 0, 255);
    let pixels = harness
        .render_and_readback(
            64,
            64,
            &source_data,
            128,
            128,
            0.0,
            InterpolationMethod::Bilinear,
        )
        .await;

    // Verify output pixels are red (allow tolerance for sRGB conversion).
    // sRGB textures linearize on read and re-encode on write, so values
    // should round-trip back to approximately the same values.
    let total_pixels = 128 * 128;
    for i in 0..total_pixels {
        let offset = i * 4;
        let r = pixels[offset];
        let g = pixels[offset + 1];
        let b = pixels[offset + 2];
        let a = pixels[offset + 3];

        assert!(r > 200, "pixel {i}: red channel too low: {r}");
        assert!(g < 30, "pixel {i}: green channel too high: {g}");
        assert!(b < 30, "pixel {i}: blue channel too high: {b}");
        assert!(a > 200, "pixel {i}: alpha channel too low: {a}");
    }
}

#[tokio::test]
async fn shader_bilinear_solid_blue_bgra_swizzle() {
    let Some(harness) = GpuTestHarness::new().await else {
        eprintln!("skipping test: no GPU adapter available");
        return;
    };

    // Source data is "BGRA" in memory: B=255, G=0, R=0, A=255.
    // This means blue is in the R channel position when interpreted as RGBA.
    // With is_bgra=1.0, the shader swizzles R<->B, so output should be blue.
    let source_data = solid_color_pixels(64, 64, 255, 0, 0, 255);
    let pixels = harness
        .render_and_readback(
            64,
            64,
            &source_data,
            128,
            128,
            1.0,
            InterpolationMethod::Bilinear,
        )
        .await;

    // After swizzle: what was R (255) becomes B, what was B (0) becomes R.
    let total_pixels = 128 * 128;
    for i in 0..total_pixels {
        let offset = i * 4;
        let r = pixels[offset];
        let b = pixels[offset + 2];

        assert!(
            r < 30,
            "pixel {i}: red channel should be low after swizzle: {r}"
        );
        assert!(
            b > 200,
            "pixel {i}: blue channel should be high after swizzle: {b}"
        );
    }
}

#[tokio::test]
async fn shader_bilinear_1_5x_zoom_no_artifacts() {
    let Some(harness) = GpuTestHarness::new().await else {
        eprintln!("skipping test: no GPU adapter available");
        return;
    };

    // Green source, 1.5x zoom (86x86 source -> 128x128 output).
    let source_data = solid_color_pixels(86, 86, 0, 255, 0, 255);
    let pixels = harness
        .render_and_readback(
            86,
            86,
            &source_data,
            128,
            128,
            0.0,
            InterpolationMethod::Bilinear,
        )
        .await;

    // Verify no black pixels (artifact detection).
    let total_pixels = 128 * 128;
    for i in 0..total_pixels {
        let offset = i * 4;
        let g = pixels[offset + 1];
        let a = pixels[offset + 3];

        assert!(
            g > 200,
            "pixel {i}: green channel too low (possible artifact): {g}"
        );
        assert!(a > 200, "pixel {i}: alpha channel too low: {a}");
    }
}

#[tokio::test]
async fn shader_bilinear_20x_zoom_no_artifacts() {
    let Some(harness) = GpuTestHarness::new().await else {
        eprintln!("skipping test: no GPU adapter available");
        return;
    };

    // Small source (8x8) rendered to larger output (128x128) = ~16x zoom.
    let source_data = solid_color_pixels(8, 8, 0, 0, 255, 255);
    let pixels = harness
        .render_and_readback(
            8,
            8,
            &source_data,
            128,
            128,
            0.0,
            InterpolationMethod::Bilinear,
        )
        .await;

    // Verify output is filled with blue (no black artifacts).
    let total_pixels = 128 * 128;
    for i in 0..total_pixels {
        let offset = i * 4;
        let b = pixels[offset + 2];
        let a = pixels[offset + 3];

        assert!(b > 200, "pixel {i}: blue channel too low (artifact): {b}");
        assert!(a > 200, "pixel {i}: alpha too low: {a}");
    }
}

// --- Bicubic shader output tests ---

#[tokio::test]
async fn shader_bicubic_solid_red_rgba() {
    let Some(harness) = GpuTestHarness::new().await else {
        eprintln!("skipping test: no GPU adapter available");
        return;
    };

    let source_data = solid_color_pixels(64, 64, 255, 0, 0, 255);
    let pixels = harness
        .render_and_readback(
            64,
            64,
            &source_data,
            128,
            128,
            0.0,
            InterpolationMethod::Bicubic,
        )
        .await;

    let total_pixels = 128 * 128;
    for i in 0..total_pixels {
        let offset = i * 4;
        let r = pixels[offset];
        let g = pixels[offset + 1];
        let b = pixels[offset + 2];
        let a = pixels[offset + 3];

        assert!(r > 200, "pixel {i}: red too low: {r}");
        assert!(g < 30, "pixel {i}: green too high: {g}");
        assert!(b < 30, "pixel {i}: blue too high: {b}");
        assert!(a > 200, "pixel {i}: alpha too low: {a}");
    }
}

#[tokio::test]
async fn shader_bicubic_solid_blue_bgra_swizzle() {
    let Some(harness) = GpuTestHarness::new().await else {
        eprintln!("skipping test: no GPU adapter available");
        return;
    };

    let source_data = solid_color_pixels(64, 64, 255, 0, 0, 255);
    let pixels = harness
        .render_and_readback(
            64,
            64,
            &source_data,
            128,
            128,
            1.0,
            InterpolationMethod::Bicubic,
        )
        .await;

    let total_pixels = 128 * 128;
    for i in 0..total_pixels {
        let offset = i * 4;
        let r = pixels[offset];
        let b = pixels[offset + 2];

        assert!(r < 30, "pixel {i}: red should be low after swizzle: {r}");
        assert!(b > 200, "pixel {i}: blue should be high after swizzle: {b}");
    }
}

#[tokio::test]
async fn shader_bicubic_edge_quality() {
    let Some(harness) = GpuTestHarness::new().await else {
        eprintln!("skipping test: no GPU adapter available");
        return;
    };

    // Create a sharp black-white edge (left half black, right half white).
    let src_width: u32 = 16;
    let src_height: u32 = 16;
    let mut source_data = Vec::with_capacity((src_width * src_height * 4) as usize);
    for _y in 0..src_height {
        for x in 0..src_width {
            if x < src_width / 2 {
                source_data.extend_from_slice(&[0, 0, 0, 255]); // black
            } else {
                source_data.extend_from_slice(&[255, 255, 255, 255]); // white
            }
        }
    }

    let out_width: u32 = 128;
    let out_height: u32 = 128;

    let bilinear_pixels = harness
        .render_and_readback(
            src_width,
            src_height,
            &source_data,
            out_width,
            out_height,
            0.0,
            InterpolationMethod::Bilinear,
        )
        .await;

    let bicubic_pixels = harness
        .render_and_readback(
            src_width,
            src_height,
            &source_data,
            out_width,
            out_height,
            0.0,
            InterpolationMethod::Bicubic,
        )
        .await;

    // Count the number of "intermediate" gray pixels at the edge region.
    // The edge in the output is around x = out_width/2 (column 64).
    // Check columns 56-72 (edge region) in the middle row.
    let mid_row = out_height / 2;
    let mut bilinear_intermediates = 0u32;
    let mut bicubic_intermediates = 0u32;

    for col in (out_width / 2 - 8)..(out_width / 2 + 8) {
        let idx = ((mid_row * out_width + col) * 4) as usize;

        let bil_r = bilinear_pixels[idx];
        if bil_r > 10 && bil_r < 245 {
            bilinear_intermediates += 1;
        }

        let bic_r = bicubic_pixels[idx];
        if bic_r > 10 && bic_r < 245 {
            bicubic_intermediates += 1;
        }
    }

    // Bicubic should produce at least as many intermediate values as bilinear
    // (it has a wider sampling kernel, producing a smoother transition).
    assert!(
        bicubic_intermediates >= bilinear_intermediates,
        "bicubic ({bicubic_intermediates} intermediates) should produce \
         at least as many intermediate gray values as bilinear \
         ({bilinear_intermediates} intermediates) at the edge"
    );
}

// --- Full-screen triangle UV coverage test ---

#[tokio::test]
async fn fullscreen_triangle_covers_viewport() {
    let Some(harness) = GpuTestHarness::new().await else {
        eprintln!("skipping test: no GPU adapter available");
        return;
    };

    // Create a UV gradient source texture:
    // R = x/width * 255, G = y/height * 255, B = 0, A = 255
    let src_size: u32 = 64;
    let mut source_data = Vec::with_capacity((src_size * src_size * 4) as usize);
    for y in 0..src_size {
        for x in 0..src_size {
            let r = ((x as f32 / (src_size - 1) as f32) * 255.0) as u8;
            let g = ((y as f32 / (src_size - 1) as f32) * 255.0) as u8;
            source_data.extend_from_slice(&[r, g, 0, 255]);
        }
    }

    let out_size: u32 = 64;
    let pixels = harness
        .render_and_readback(
            src_size,
            src_size,
            &source_data,
            out_size,
            out_size,
            0.0,
            InterpolationMethod::Bilinear,
        )
        .await;

    // Check corner pixels (with tolerance for interpolation and sRGB conversion).
    let tolerance: i32 = 40; // sRGB gamma curve can shift values significantly

    // Top-left (0, 0): should be approximately (0, 0, 0, 255)
    let tl = &pixels[0..4];
    assert!(
        (tl[0] as i32) < tolerance,
        "top-left R should be near 0, got {}",
        tl[0]
    );
    assert!(
        (tl[1] as i32) < tolerance,
        "top-left G should be near 0, got {}",
        tl[1]
    );

    // Top-right (W-1, 0): should be approximately (255, 0, 0, 255)
    let tr_offset = ((out_size - 1) * 4) as usize;
    let tr = &pixels[tr_offset..tr_offset + 4];
    assert!(
        (tr[0] as i32) > 255 - tolerance,
        "top-right R should be near 255, got {}",
        tr[0]
    );
    assert!(
        (tr[1] as i32) < tolerance,
        "top-right G should be near 0, got {}",
        tr[1]
    );

    // Bottom-left (0, H-1): should be approximately (0, 255, 0, 255)
    let bl_offset = (((out_size - 1) * out_size) * 4) as usize;
    let bl = &pixels[bl_offset..bl_offset + 4];
    assert!(
        (bl[0] as i32) < tolerance,
        "bottom-left R should be near 0, got {}",
        bl[0]
    );
    assert!(
        (bl[1] as i32) > 255 - tolerance,
        "bottom-left G should be near 255, got {}",
        bl[1]
    );

    // Bottom-right (W-1, H-1): should be approximately (255, 255, 0, 255)
    let br_offset = (((out_size - 1) * out_size + (out_size - 1)) * 4) as usize;
    let br = &pixels[br_offset..br_offset + 4];
    assert!(
        (br[0] as i32) > 255 - tolerance,
        "bottom-right R should be near 255, got {}",
        br[0]
    );
    assert!(
        (br[1] as i32) > 255 - tolerance,
        "bottom-right G should be near 255, got {}",
        br[1]
    );
}
