//! GPU device and instance creation for the Luminos rendering pipeline.
//!
//! Provides [`create_wgpu_instance`] for creating a wgpu instance with
//! appropriate backend selection, and [`create_gpu_device`] for requesting
//! a GPU adapter, device, and queue suitable for magnification rendering.

use crate::error::RenderError;

/// Creates a wgpu instance configured for the current platform.
///
/// On Linux, enables the Vulkan and GL backends to support both native
/// Vulkan drivers and Mesa software renderers (llvmpipe/lavapipe).
///
/// # Examples
///
/// ```no_run
/// let instance = luminos_gpu::device::create_wgpu_instance();
/// ```
#[must_use]
pub fn create_wgpu_instance() -> wgpu::Instance {
    let backends = if cfg!(target_os = "linux") {
        wgpu::Backends::VULKAN | wgpu::Backends::GL
    } else if cfg!(target_os = "macos") {
        wgpu::Backends::METAL
    } else if cfg!(target_os = "windows") {
        wgpu::Backends::VULKAN | wgpu::Backends::DX12
    } else {
        wgpu::Backends::all()
    };

    // wgpu 29 dropped `Default` on `InstanceDescriptor` (the `display` field
    // holds a `Box<dyn ...>`) and `Instance::new` now takes the descriptor by
    // value, so every field is supplied explicitly.
    wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        flags: wgpu::InstanceFlags::default(),
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        backend_options: wgpu::BackendOptions::default(),
        display: None,
    })
}

/// Creates the wgpu adapter, device, and queue for the rendering pipeline.
///
/// Requests a `LowPower` adapter preference (integrated GPU) to minimize
/// power consumption. Uses `downlevel_webgl2_defaults` for maximum
/// hardware compatibility, raised to actual adapter limits via
/// `using_resolution()`.
///
/// # Errors
///
/// Returns [`RenderError::NoAdapter`] if no compatible GPU is found.
/// Returns [`RenderError::DeviceCreation`] if the device cannot be created.
pub async fn create_gpu_device(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'_>,
) -> Result<(wgpu::Adapter, wgpu::Device, wgpu::Queue), RenderError> {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: Some(surface),
            force_fallback_adapter: false,
        })
        .await
        .map_err(|_| RenderError::NoAdapter)?;

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("luminos_device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                .using_resolution(adapter.limits()),
            memory_hints: wgpu::MemoryHints::MemoryUsage,
            ..Default::default()
        })
        .await
        .map_err(|e| RenderError::DeviceCreation {
            message: e.to_string(),
        })?;

    Ok((adapter, device, queue))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn gpu_device_create_wgpu_instance_does_not_panic() {
        let _instance = create_wgpu_instance();
    }

    #[test]
    fn gpu_device_create_wgpu_instance_returns_instance() {
        // Verify the instance is a valid object by calling a method on it.
        let instance = create_wgpu_instance();
        // `poll_all` is a valid operation on any instance; should not panic.
        instance.poll_all(false);
    }

    // Full GPU device creation on hardware is tested in the cross-crate
    // integration test module: `tests/integration_window_gpu.rs`.
}
