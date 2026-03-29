//! Magnification shader compilation and render pipeline creation.
//!
//! Provides the [`MagnifyUniforms`] struct for GPU uniform data,
//! [`InterpolationMethod`] for selecting bilinear vs bicubic shaders,
//! and functions to compile shaders and create wgpu render pipelines.

use crate::error::RenderError;

/// GPU uniform buffer data for the magnification shader.
///
/// This struct is uploaded to a wgpu uniform buffer every frame.
/// All fields use 16-byte aligned layout per WebGPU uniform buffer
/// requirements. Total size: 32 bytes (2 `vec2f` + 1 `f32` + 3 `f32` padding).
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MagnifyUniforms {
    /// Output viewport dimensions (width, height) in pixels.
    pub viewport_size: [f32; 2],
    /// Source texture dimensions (width, height) in pixels.
    pub source_size: [f32; 2],
    /// Pixel format flag: 0.0 = RGBA (X11 via xcap, macOS), 1.0 = BGRA (Windows DXGI).
    pub is_bgra: f32,
    /// Padding for 16-byte alignment. Do not use directly.
    #[allow(clippy::pub_underscore_fields)]
    pub _pad: [f32; 3],
}

/// Selects the magnification interpolation algorithm.
///
/// Bilinear is the Phase 0 default (single texture sample per pixel).
/// Bicubic provides higher quality at higher GPU cost (16 samples per pixel).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpolationMethod {
    /// Single-sample bilinear interpolation (hardware texture filtering).
    Bilinear,
    /// 4x4 Catmull-Rom bicubic interpolation (16 manual texture lookups).
    Bicubic,
}

/// Resources for the magnification shader pipeline.
///
/// Contains the compiled render pipeline, bind group layout, and
/// uniform buffer. Created once at startup via [`create_magnify_pipeline`].
pub struct MagnifyPipeline {
    /// The compiled render pipeline (bilinear or bicubic variant).
    pub pipeline: wgpu::RenderPipeline,
    /// Bind group layout for source texture + sampler + uniforms.
    pub bind_group_layout: wgpu::BindGroupLayout,
    /// Uniform buffer for [`MagnifyUniforms`].
    pub uniform_buffer: wgpu::Buffer,
}

/// Creates the bind group layout shared by both shader variants.
///
/// Layout:
/// - Binding 0: source texture (2D, float, sampled)
/// - Binding 1: source sampler (filtering)
/// - Binding 2: uniform buffer ([`MagnifyUniforms`])
#[must_use]
pub fn create_magnify_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("magnify_bind_group_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

/// Creates a magnification render pipeline for the specified shader variant.
///
/// Both bilinear and bicubic variants share the same vertex shader,
/// bind group layout, and pipeline layout. They differ only in the
/// fragment shader (interpolation function).
///
/// # Arguments
///
/// * `device` -- The wgpu device.
/// * `surface_format` -- The swap chain surface texture format (sRGB).
/// * `bind_group_layout` -- The bind group layout from [`create_magnify_bind_group_layout`].
/// * `method` -- The interpolation method (Bilinear or Bicubic).
///
/// # Errors
///
/// Returns [`RenderError::ShaderCompilation`] if the shader fails to compile.
pub fn create_magnify_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    bind_group_layout: &wgpu::BindGroupLayout,
    method: InterpolationMethod,
) -> Result<MagnifyPipeline, RenderError> {
    let shader_source = match method {
        InterpolationMethod::Bilinear => include_str!("magnify_bilinear.wgsl"),
        InterpolationMethod::Bicubic => include_str!("magnify_bicubic.wgsl"),
    };

    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(match method {
            InterpolationMethod::Bilinear => "magnify_bilinear_shader",
            InterpolationMethod::Bicubic => "magnify_bicubic_shader",
        }),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("magnify_pipeline_layout"),
        bind_group_layouts: &[bind_group_layout],
        immediate_size: 0,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(match method {
            InterpolationMethod::Bilinear => "magnify_bilinear_pipeline",
            InterpolationMethod::Bicubic => "magnify_bicubic_pipeline",
        }),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            buffers: &[], // Full-screen triangle: no vertex buffer needed.
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("magnify_uniforms_buffer"),
        size: std::mem::size_of::<MagnifyUniforms>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    Ok(MagnifyPipeline {
        pipeline,
        bind_group_layout: bind_group_layout.clone(),
        uniform_buffer,
    })
}

/// Creates a bind group for a specific source texture.
///
/// Called each frame (or when the source texture changes) to bind
/// the current source texture to the shader.
#[must_use]
pub fn create_magnify_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    source_texture_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    uniform_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("magnify_bind_group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(source_texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: uniform_buffer.as_entire_binding(),
            },
        ],
    })
}

/// Creates a linear filtering sampler for magnification.
///
/// Uses `FilterMode::Linear` for both min and mag filtering, which
/// provides hardware-accelerated bilinear interpolation. The bicubic
/// shader performs its own interpolation and reads texels via
/// `textureSampleLevel` with this sampler for consistent addressing.
#[must_use]
pub fn create_magnify_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("magnify_sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn magnify_uniforms_size_32_bytes() {
        assert_eq!(
            std::mem::size_of::<MagnifyUniforms>(),
            32,
            "MagnifyUniforms must be exactly 32 bytes for GPU uniform buffer alignment"
        );
    }

    #[test]
    fn magnify_uniforms_bytemuck_cast() {
        let uniforms = MagnifyUniforms {
            viewport_size: [1920.0, 1080.0],
            source_size: [960.0, 540.0],
            is_bgra: 0.0,
            _pad: [0.0; 3],
        };
        let bytes = bytemuck::bytes_of(&uniforms);
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    #[allow(clippy::float_cmp, clippy::used_underscore_binding)]
    fn magnify_uniforms_default_values() {
        let uniforms = <MagnifyUniforms as bytemuck::Zeroable>::zeroed();
        assert_eq!(uniforms.viewport_size, [0.0, 0.0]);
        assert_eq!(uniforms.source_size, [0.0, 0.0]);
        assert!((uniforms.is_bgra - 0.0).abs() < f32::EPSILON);
        assert_eq!(uniforms._pad, [0.0; 3]);
    }

    #[test]
    fn interpolation_method_bilinear_ne_bicubic() {
        assert_ne!(InterpolationMethod::Bilinear, InterpolationMethod::Bicubic);
    }
}
