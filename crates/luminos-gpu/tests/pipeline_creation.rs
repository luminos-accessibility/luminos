//! Integration tests for magnification shader compilation and pipeline creation.
//!
//! These tests require a wgpu-compatible GPU or software renderer (Mesa llvmpipe).
//! They verify that both bilinear and bicubic shaders compile and that render
//! pipelines can be created successfully.

#![allow(clippy::unwrap_used)]

use luminos_gpu::shaders::{
    InterpolationMethod, MagnifyUniforms, create_magnify_bind_group,
    create_magnify_bind_group_layout, create_magnify_pipeline, create_magnify_sampler,
};

/// Creates a headless wgpu device using the GL backend (Mesa llvmpipe in CI).
///
/// Returns `None` if no compatible adapter is available.
async fn create_headless_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::GL | wgpu::Backends::VULKAN,
        ..Default::default()
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

    Some((device, queue))
}

/// Creates a test texture for bind group creation tests.
fn create_test_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
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
    })
}

#[tokio::test]
async fn shader_bilinear_compiles() {
    let Some((device, _queue)) = create_headless_device().await else {
        eprintln!("skipping test: no GPU adapter available");
        return;
    };

    let layout = create_magnify_bind_group_layout(&device);
    let result = create_magnify_pipeline(
        &device,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        &layout,
        InterpolationMethod::Bilinear,
    );
    assert!(
        result.is_ok(),
        "bilinear pipeline creation failed: {:?}",
        result.as_ref().err()
    );
}

#[tokio::test]
async fn shader_bicubic_compiles() {
    let Some((device, _queue)) = create_headless_device().await else {
        eprintln!("skipping test: no GPU adapter available");
        return;
    };

    let layout = create_magnify_bind_group_layout(&device);
    let result = create_magnify_pipeline(
        &device,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        &layout,
        InterpolationMethod::Bicubic,
    );
    assert!(
        result.is_ok(),
        "bicubic pipeline creation failed: {:?}",
        result.as_ref().err()
    );
}

#[tokio::test]
async fn pipeline_bilinear_creates() {
    let Some((device, _queue)) = create_headless_device().await else {
        eprintln!("skipping test: no GPU adapter available");
        return;
    };

    let layout = create_magnify_bind_group_layout(&device);
    let pipeline = create_magnify_pipeline(
        &device,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        &layout,
        InterpolationMethod::Bilinear,
    );
    assert!(pipeline.is_ok());
}

#[tokio::test]
async fn pipeline_bicubic_creates() {
    let Some((device, _queue)) = create_headless_device().await else {
        eprintln!("skipping test: no GPU adapter available");
        return;
    };

    let layout = create_magnify_bind_group_layout(&device);
    let pipeline = create_magnify_pipeline(
        &device,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        &layout,
        InterpolationMethod::Bicubic,
    );
    assert!(pipeline.is_ok());
}

#[tokio::test]
async fn pipeline_both_variants_compile() {
    let Some((device, _queue)) = create_headless_device().await else {
        eprintln!("skipping test: no GPU adapter available");
        return;
    };

    let layout = create_magnify_bind_group_layout(&device);

    let bilinear = create_magnify_pipeline(
        &device,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        &layout,
        InterpolationMethod::Bilinear,
    );
    assert!(
        bilinear.is_ok(),
        "bilinear failed: {:?}",
        bilinear.as_ref().err()
    );

    let bicubic = create_magnify_pipeline(
        &device,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        &layout,
        InterpolationMethod::Bicubic,
    );
    assert!(
        bicubic.is_ok(),
        "bicubic failed: {:?}",
        bicubic.as_ref().err()
    );
}

#[tokio::test]
async fn pipeline_variant_swap() {
    let Some((device, _queue)) = create_headless_device().await else {
        eprintln!("skipping test: no GPU adapter available");
        return;
    };

    let layout = create_magnify_bind_group_layout(&device);

    let bilinear = create_magnify_pipeline(
        &device,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        &layout,
        InterpolationMethod::Bilinear,
    )
    .unwrap();

    let _bicubic = create_magnify_pipeline(
        &device,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        &layout,
        InterpolationMethod::Bicubic,
    )
    .unwrap();

    // Verify the same bind group can be used with either pipeline's layout.
    let texture = create_test_texture(&device, 64, 64);
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = create_magnify_sampler(&device);

    let _bind_group = create_magnify_bind_group(
        &device,
        &bilinear.bind_group_layout,
        &texture_view,
        &sampler,
        &bilinear.uniform_buffer,
    );
}

#[tokio::test]
async fn pipeline_uniform_buffer_write() {
    let Some((device, queue)) = create_headless_device().await else {
        eprintln!("skipping test: no GPU adapter available");
        return;
    };

    let layout = create_magnify_bind_group_layout(&device);
    let pipeline = create_magnify_pipeline(
        &device,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        &layout,
        InterpolationMethod::Bilinear,
    )
    .unwrap();

    let uniforms = MagnifyUniforms {
        viewport_size: [1920.0, 1080.0],
        source_size: [960.0, 540.0],
        is_bgra: 0.0,
        _pad: [0.0; 3],
    };

    // write_buffer is infallible in wgpu's API.
    queue.write_buffer(&pipeline.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
}

#[tokio::test]
async fn bind_group_layout_creates() {
    let Some((device, _queue)) = create_headless_device().await else {
        eprintln!("skipping test: no GPU adapter available");
        return;
    };

    let _layout = create_magnify_bind_group_layout(&device);
    // If we reach here without panic, the layout was created successfully.
}

#[tokio::test]
async fn bind_group_creates_with_texture() {
    let Some((device, _queue)) = create_headless_device().await else {
        eprintln!("skipping test: no GPU adapter available");
        return;
    };

    let layout = create_magnify_bind_group_layout(&device);
    let texture = create_test_texture(&device, 64, 64);
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = create_magnify_sampler(&device);

    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test_uniform_buffer"),
        size: std::mem::size_of::<MagnifyUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let _bind_group =
        create_magnify_bind_group(&device, &layout, &texture_view, &sampler, &uniform_buffer);
}

#[tokio::test]
async fn sampler_linear_filtering() {
    let Some((device, _queue)) = create_headless_device().await else {
        eprintln!("skipping test: no GPU adapter available");
        return;
    };

    let layout = create_magnify_bind_group_layout(&device);
    let sampler = create_magnify_sampler(&device);
    let texture = create_test_texture(&device, 64, 64);
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("test_uniform_buffer"),
        size: std::mem::size_of::<MagnifyUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Verify sampler can be bound to the magnify bind group.
    let _bind_group =
        create_magnify_bind_group(&device, &layout, &texture_view, &sampler, &uniform_buffer);
}
