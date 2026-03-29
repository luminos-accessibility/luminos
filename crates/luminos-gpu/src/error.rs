//! GPU rendering pipeline error types.

/// Errors that can occur during GPU rendering pipeline operations.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// No compatible GPU adapter was found.
    #[error("no compatible GPU adapter found")]
    NoAdapter,

    /// GPU device creation failed.
    #[error("GPU device creation failed: {message}")]
    DeviceCreation {
        /// Description of the device creation failure.
        message: String,
    },

    /// Surface configuration failed.
    #[error("surface configuration failed: {message}")]
    SurfaceConfiguration {
        /// Description of the surface configuration failure.
        message: String,
    },

    /// Surface texture acquisition failed (e.g., window resized, surface lost).
    #[error("surface texture unavailable: {message}")]
    SurfaceTexture {
        /// Description of the surface texture failure.
        message: String,
    },

    /// Shader compilation failed.
    #[error("shader compilation failed: {message}")]
    ShaderCompilation {
        /// Description of the shader compilation failure.
        message: String,
    },

    /// A render pass or command submission failed.
    #[error("render error: {message}")]
    RenderFailed {
        /// Description of the render failure.
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_error_display_no_adapter() {
        let err = RenderError::NoAdapter;
        assert_eq!(err.to_string(), "no compatible GPU adapter found");
    }

    #[test]
    fn render_error_display_device_creation() {
        let err = RenderError::DeviceCreation {
            message: "Vulkan init failed".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("GPU device creation failed"));
        assert!(msg.contains("Vulkan init failed"));
    }

    #[test]
    fn render_error_display_surface_configuration() {
        let err = RenderError::SurfaceConfiguration {
            message: "no compatible format".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("surface configuration failed"));
        assert!(msg.contains("no compatible format"));
    }

    #[test]
    fn render_error_display_surface_texture() {
        let err = RenderError::SurfaceTexture {
            message: "surface lost".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("surface texture unavailable"));
        assert!(msg.contains("surface lost"));
    }

    #[test]
    fn render_error_display_shader_compilation() {
        let err = RenderError::ShaderCompilation {
            message: "syntax error at line 42".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("shader compilation failed"));
        assert!(msg.contains("syntax error at line 42"));
    }

    #[test]
    fn render_error_display_render_failed() {
        let err = RenderError::RenderFailed {
            message: "command buffer submission timeout".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("render error"));
        assert!(msg.contains("command buffer submission timeout"));
    }
}
