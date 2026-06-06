//! Surface configuration for the wgpu rendering pipeline.
//!
//! Provides [`configure_surface`] to bind a wgpu surface to the overlay
//! window with appropriate format, alpha mode, and present mode settings.

use crate::error::RenderError;

/// Selects the preferred composite alpha mode from the available modes.
///
/// Priority order: `PreMultiplied` (best for transparent overlays) ->
/// `PostMultiplied` (acceptable fallback) -> `Opaque` (no transparency).
/// Logs a warning when falling back from the preferred mode.
#[must_use]
pub fn select_alpha_mode(modes: &[wgpu::CompositeAlphaMode]) -> wgpu::CompositeAlphaMode {
    if modes.contains(&wgpu::CompositeAlphaMode::PreMultiplied) {
        wgpu::CompositeAlphaMode::PreMultiplied
    } else if modes.contains(&wgpu::CompositeAlphaMode::PostMultiplied) {
        log::warn!(
            "PreMultiplied alpha unavailable, using PostMultiplied -- \
             overlay transparency may have fringing artifacts"
        );
        wgpu::CompositeAlphaMode::PostMultiplied
    } else {
        log::warn!(
            "transparent alpha modes unavailable, using Opaque -- \
             overlay will not support transparency"
        );
        wgpu::CompositeAlphaMode::Opaque
    }
}

/// Selects the preferred texture format from the available formats.
///
/// Prefers sRGB formats for gamma-correct rendering. Falls back to the
/// first available format if no sRGB format is found.
///
/// # Errors
///
/// Returns [`RenderError::SurfaceConfiguration`] if no formats are available.
pub fn select_texture_format(
    formats: &[wgpu::TextureFormat],
) -> Result<wgpu::TextureFormat, RenderError> {
    formats
        .iter()
        .find(|f| f.is_srgb())
        .copied()
        .or_else(|| formats.first().copied())
        .ok_or_else(|| RenderError::SurfaceConfiguration {
            message: "no compatible surface format found".into(),
        })
}

/// Configures the wgpu surface for the overlay window and returns the exact
/// [`wgpu::SurfaceConfiguration`] that was built and applied.
///
/// Selects an sRGB-compatible texture format for gamma-correct rendering.
/// Uses `PreMultiplied` alpha for transparent overlay compositing, with
/// fallback to `PostMultiplied` or `Opaque` if unavailable.
///
/// Returning the applied configuration makes it the single source of truth:
/// callers store it for `resize`/`Lost`/`Outdated` recovery (re-applying the
/// same struct with updated dimensions) rather than hand-rebuilding a copy that
/// could drift from what was actually configured. Callers that only need the
/// texture format read [`wgpu::SurfaceConfiguration::format`].
///
/// # Arguments
///
/// * `surface` -- The wgpu surface bound to the overlay window.
/// * `adapter` -- The GPU adapter (for capability queries).
/// * `device` -- The GPU device.
/// * `width` -- Surface width in pixels (clamped to at least 1).
/// * `height` -- Surface height in pixels (clamped to at least 1).
/// * `present_mode` -- Frame pacing strategy (typically `Fifo` for vsync).
///
/// # Errors
///
/// Returns [`RenderError::SurfaceConfiguration`] if surface capabilities
/// cannot be queried or no compatible format is found.
pub fn configure_surface(
    surface: &wgpu::Surface<'_>,
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    width: u32,
    height: u32,
    present_mode: wgpu::PresentMode,
) -> Result<wgpu::SurfaceConfiguration, RenderError> {
    let caps = surface.get_capabilities(adapter);

    let format = select_texture_format(&caps.formats)?;
    let alpha_mode = select_alpha_mode(&caps.alpha_modes);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: width.max(1),
        height: height.max(1),
        present_mode,
        alpha_mode,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    surface.configure(device, &config);

    Ok(config)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // --- select_alpha_mode tests ---

    #[test]
    fn surface_select_alpha_mode_prefers_premultiplied() {
        let modes = vec![
            wgpu::CompositeAlphaMode::Opaque,
            wgpu::CompositeAlphaMode::PreMultiplied,
            wgpu::CompositeAlphaMode::PostMultiplied,
        ];
        assert_eq!(
            select_alpha_mode(&modes),
            wgpu::CompositeAlphaMode::PreMultiplied
        );
    }

    #[test]
    fn surface_select_alpha_mode_falls_back_to_postmultiplied() {
        let modes = vec![
            wgpu::CompositeAlphaMode::Opaque,
            wgpu::CompositeAlphaMode::PostMultiplied,
        ];
        assert_eq!(
            select_alpha_mode(&modes),
            wgpu::CompositeAlphaMode::PostMultiplied
        );
    }

    #[test]
    fn surface_select_alpha_mode_falls_back_to_opaque() {
        let modes = vec![wgpu::CompositeAlphaMode::Opaque];
        assert_eq!(select_alpha_mode(&modes), wgpu::CompositeAlphaMode::Opaque);
    }

    #[test]
    fn surface_select_alpha_mode_empty_returns_opaque() {
        assert_eq!(select_alpha_mode(&[]), wgpu::CompositeAlphaMode::Opaque);
    }

    // --- select_texture_format tests ---

    #[test]
    fn surface_select_format_prefers_srgb() {
        let formats = vec![
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            wgpu::TextureFormat::Bgra8UnormSrgb,
        ];
        let format = select_texture_format(&formats).unwrap();
        assert!(format.is_srgb(), "expected sRGB format, got {format:?}");
    }

    #[test]
    fn surface_select_format_falls_back_to_first() {
        let formats = vec![
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureFormat::Rgba8Unorm,
        ];
        let format = select_texture_format(&formats).unwrap();
        assert_eq!(format, wgpu::TextureFormat::Bgra8Unorm);
    }

    #[test]
    fn surface_select_format_empty_returns_error() {
        let result = select_texture_format(&[]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("no compatible surface format"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn surface_select_format_single_srgb() {
        let formats = vec![wgpu::TextureFormat::Rgba8UnormSrgb];
        let format = select_texture_format(&formats).unwrap();
        assert_eq!(format, wgpu::TextureFormat::Rgba8UnormSrgb);
    }
}
