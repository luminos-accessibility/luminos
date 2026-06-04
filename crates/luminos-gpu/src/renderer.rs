//! Rendering pipeline orchestration for screen magnification.
//!
//! Contains [`Renderer`], which holds all persistent GPU resources and
//! drives the per-frame capture-upload-render-present cycle. Created once
//! at startup and reused every frame.

use crate::error::RenderError;
use crate::frame_timings::FrameTimings;
use crate::shaders::{InterpolationMethod, MagnifyUniforms};
use crate::texture::SourceTextureManager;

use luminos_types::CaptureFrame;

/// Holds all persistent GPU resources for the rendering pipeline.
///
/// Orchestrates the per-frame capture-upload-render-present cycle:
/// upload `CaptureFrame` to source texture, execute magnification shader,
/// and present the result on the swap chain surface.
///
/// # Construction
///
/// Use [`Renderer::new`] to create a renderer with all GPU resources
/// initialized. The renderer requires a pre-configured wgpu device,
/// queue, and surface format.
///
/// # Frame Cycle
///
/// Each frame, call [`Renderer::render_frame`] with the current
/// `CaptureFrame` and surface. On capture failure, call
/// [`Renderer::handle_capture_failure`] to render the stale frame.
pub struct Renderer {
    /// wgpu device for resource creation.
    device: wgpu::Device,
    /// wgpu queue for command submission.
    queue: wgpu::Queue,
    /// The magnification shader pipeline (bilinear or bicubic).
    magnify_pipeline: crate::shaders::MagnifyPipeline,
    /// Source texture manager (upload, reallocation, stale tracking).
    source_texture_manager: SourceTextureManager,
    /// Frame timing ring buffer.
    frame_timings: FrameTimings,
    /// Texture sampler for the magnification shader.
    sampler: wgpu::Sampler,
    /// Surface format (stored for potential future reconfiguration).
    #[allow(dead_code)]
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
    /// * `device` -- The wgpu device.
    /// * `queue` -- The wgpu queue.
    /// * `surface_format` -- The swap chain surface texture format.
    /// * `viewport_width` -- Initial overlay viewport width.
    /// * `viewport_height` -- Initial overlay viewport height.
    /// * `method` -- The interpolation method (Bilinear or Bicubic).
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
        let source_texture_manager =
            SourceTextureManager::new(device.clone(), viewport_width / 2, viewport_height / 2);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("luminos_magnify_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
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
    /// Uploads the [`CaptureFrame`] to the GPU source texture, creates a
    /// per-frame bind group, encodes the magnification render pass, and
    /// presents the result to the swap chain surface.
    ///
    /// # Errors
    ///
    /// Returns [`RenderError::SurfaceTexture`] if the swap chain surface
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

        // Acquire swap chain surface texture. wgpu 29 replaced the
        // `Result<SurfaceTexture, SurfaceError>` return with the
        // `CurrentSurfaceTexture` enum; treat `Suboptimal` as usable and map
        // every non-texture status to a render error.
        let output = match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            status => {
                return Err(RenderError::SurfaceTexture {
                    message: format!("{status:?}"),
                });
            }
        };
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Update uniforms
        let (source_w, source_h) = self.source_texture_manager.current_dimensions();
        #[allow(clippy::cast_precision_loss)]
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
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
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

    /// Handles a capture failure by recording the stale frame.
    ///
    /// Delegates to [`SourceTextureManager::record_capture_failure`] for
    /// stale frame tracking. The next `render_frame()` call with a valid
    /// [`CaptureFrame`] will reset the stale state automatically.
    pub fn handle_capture_failure(&mut self) {
        self.source_texture_manager.record_capture_failure();
    }

    /// Handles a window resize by updating the viewport dimensions.
    ///
    /// The caller is responsible for reconfiguring the wgpu surface
    /// with the new dimensions before calling this method. Zero-width
    /// or zero-height resizes are silently ignored.
    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width > 0 && new_height > 0 {
            self.viewport_width = new_width;
            self.viewport_height = new_height;
        }
    }

    /// Returns a reference to the frame timings for performance monitoring.
    #[must_use]
    pub fn frame_timings(&self) -> &FrameTimings {
        &self.frame_timings
    }
}
