# Design: Story E02/002 -- X11 Overlay Window & GPU Surface

**Story:** [STORY.md](./STORY.md)
**Epic:** [HIGH_LEVEL_PLAN.md](../HIGH_LEVEL_PLAN.md)
**Status:** APPROVED
**Author:** Spec Writer 2
**Risk Refs:** [RISK-001](../../tech-strategy/10-risk-register.md) (dual event loop coexistence), [RISK-016](../../tech-strategy/10-risk-register.md) (wgpu backend compatibility)

---

## Overview

This design implements the X11 overlay window via `winit` and the wgpu device/surface initialization for the Luminos magnification pipeline. The overlay is a transparent, borderless, always-on-top, override-redirect native window that serves as the render target for GPU-accelerated magnification in FullScreen mode (docked and lens modes are deferred to Epic 5). The design also unifies the `DockEdge`/`LensShape` type duplication discovered in E01, establishing a single source of truth in `luminos-platform` with serde derives.

The approach follows the dual-window architecture from doc-01 Section 3: the overlay is a pure `winit` + `wgpu` window, independent of Tauri. The `WindowManager` trait (defined in E01) provides the interface; this story provides the first real implementation.

**RISK-001 awareness:** This story creates the winit overlay window but does NOT start the Tauri control panel. The dual event loop coexistence risk is deferred to E04. The `X11WindowManager` is designed to be usable independently of Tauri.

**RISK-016 awareness:** wgpu surface configuration uses capability queries (`surface.get_capabilities()`) to handle driver-specific differences. `PreMultiplied` alpha fallback, sRGB format fallback, and `PresentMode::Fifo` (universally supported) are all implemented defensively.

## Architecture

### Component Diagram

```
crates/luminos-types/src/         # NEW CRATE -- canonical shared types (zero workspace deps)
  |
  +-- lib.rs                     # Re-exports all type modules
  +-- display.rs                 # ScreenRect, ScreenPoint, DisplayInfo (+ serde)
  +-- capture.rs                 # CaptureFrame, PixelFormat (CaptureFrame skips serde)
  +-- overlay.rs                 # DockEdge, LensShape, OverlayMode (+ serde)
  +-- state.rs                   # MagnificationMode, TrackingMode, ColorFilterType, TtsStatus
  +-- gpu.rs                     # PresentMode, GpuPreference, InterpolationMode

crates/luminos-platform/src/
  |
  +-- traits/
  |     +-- window_manager.rs    # Re-exports DockEdge, LensShape, OverlayMode from luminos-types
  |     +-- types.rs             # Re-exports ScreenRect, DisplayInfo etc. from luminos-types
  |
  +-- linux_x11/
        +-- mod.rs               # Re-exports X11WindowManager
        +-- window.rs            # X11WindowManager struct (NEW)

crates/luminos-gpu/src/
  |
  +-- lib.rs                     # Module declarations
  +-- error.rs                   # RenderError enum (NEW)
  +-- device.rs                  # create_wgpu_instance(), create_gpu_device() (NEW)
  +-- surface.rs                 # configure_surface(), select_alpha_mode(), select_texture_format() (NEW)

crates/luminos-core/src/
  |
  +-- config/
        +-- schema.rs            # DockEdge, LensShape etc. re-exported from luminos-types (MODIFIED)
```

> **Deviation from original design:** The original DESIGN.md proposed keeping canonical type definitions in `luminos-platform` and re-exporting from `luminos-core`. During pre-implementation planning, a user-directed decision created `luminos-types` as a separate crate with zero workspace dependencies, providing a cleaner dependency graph. Both `luminos-platform` and `luminos-core` now re-export from `luminos-types`.

### Affected Traits / Modules

| Trait / Module | Change Type | Description |
|----------------|-------------|-------------|
| `traits::window_manager` | Modified | Add `Serialize`/`Deserialize` derives to `DockEdge`, `LensShape`, `OverlayMode` |
| `linux_x11::window` | New | `X11WindowManager` implementing `WindowManager` trait |
| `linux_x11::mod` | Modified | Re-export `X11WindowManager` |
| `luminos-gpu::error` | New | `RenderError` enum for GPU failures |
| `luminos-gpu::device` | New | `create_gpu_device()` async function |
| `luminos-gpu::surface` | New | `configure_surface()` function |
| `luminos-core::config::schema` | Modified | Remove duplicate `DockEdge`/`LensShape`, re-export from `luminos-platform` |

### Data Flow

```
1. Application startup calls X11WindowManager::create_overlay(display_id)
   |
   v
2. winit creates a native X11 window with attributes:
   - transparent = true
   - decorations = false (borderless)
   - window_level = AlwaysOnTop
   - override_redirect = true (bypass WM, critical for overlay)
   |
   v
3. X11WindowManager stores winit::Window, provides HasWindowHandle/HasDisplayHandle
   |
   v
3b. overlay_window_id() extracts the X11 window ID (u64) from RawWindowHandle::Xlib
    -> passed to XcbCapture (Story 001) for self-capture exclusion (RISK-002)
   |
   v
4. create_gpu_device() requests wgpu adapter (LowPower, Vulkan on Linux)
   |
   v
5. configure_surface() binds wgpu surface to overlay window via raw handles
   |
   v
6. Surface is ready for rendering (Stories 003-005 use it)
```

---

## API Design

### `X11WindowManager` -- `crates/luminos-platform/src/linux_x11/window.rs`

```rust
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::window::{Window, WindowAttributes};

use crate::traits::{
    DockEdge, LensShape, OverlayMode, ScreenRect, WindowError, WindowManager,
};

/// X11 overlay window manager using winit.
///
/// Creates and manages a transparent, borderless, always-on-top,
/// override-redirect window for magnification rendering on X11. Uses
/// winit for window creation. Only FullScreen mode is implemented in
/// E02; docked and lens modes are deferred to Epic 5.
///
/// # Override-Redirect
///
/// The overlay uses `with_override_redirect(true)` to bypass the window
/// manager. This is critical: it prevents the WM from adding decorations,
/// applying focus policies, or interfering with always-on-top behavior.
/// Override-redirect windows are also naturally excluded from some X11
/// composite capture paths, which aids self-capture prevention (RISK-002).
///
/// # Platform Notes
///
/// - Transparency requires a compositing WM (Mutter, KWin, Picom).
///   On non-compositing WMs, the window background will be opaque black.
/// - Always-on-top is implemented via EWMH `_NET_WM_STATE_ABOVE`.
/// - Docked/Lens modes are deferred to Epic 5.
pub struct X11WindowManager {
    /// The winit window for the overlay. `None` before `create_overlay()`.
    window: Option<Window>,
    /// Current overlay mode.
    current_mode: OverlayMode,
    /// Display bounds for the target display.
    display_bounds: Option<ScreenRect>,
}

impl X11WindowManager {
    /// Creates a new `X11WindowManager` with no active overlay.
    pub fn new() -> Self {
        Self {
            window: None,
            current_mode: OverlayMode::FullScreen,
            display_bounds: None,
        }
    }
}

impl WindowManager for X11WindowManager {
    fn create_overlay(&mut self, display_id: &str) -> Result<(), WindowError> {
        // 1. Find the target display by display_id
        // 2. Create winit window with:
        //    - transparent = true
        //    - decorations = false (borderless)
        //    - window_level = AlwaysOnTop
        //    - with_override_redirect(true) (bypass WM)
        // 3. Set window bounds to cover the full display
        // 4. Store window in self.window
        // Returns WindowError::DisplayNotFound if display_id is invalid
        // Returns WindowError::CreationFailed if window creation fails
        todo!()
    }

    fn set_overlay_bounds(&self, bounds: ScreenRect) -> Result<(), WindowError> {
        // Set window position and size via winit
        // Returns WindowError::PropertyFailed if no overlay exists
        todo!()
    }

    fn set_overlay_mode(&mut self, mode: OverlayMode) -> Result<(), WindowError> {
        // E02 scope: FullScreen only.
        // FullScreen -> resize to full display
        // Docked/Lens -> return WindowError::PropertyFailed (deferred to E05)
        todo!()
    }

    fn set_always_on_top(&self, always_on_top: bool) -> Result<(), WindowError> {
        // Set winit::window::WindowLevel::AlwaysOnTop or Normal
        todo!()
    }

    fn set_visible(&self, visible: bool) -> Result<(), WindowError> {
        // Call window.set_visible(visible)
        todo!()
    }

    fn raw_window_handle(&self) -> Option<&dyn raw_window_handle::HasWindowHandle> {
        self.window.as_ref().map(|w| w as &dyn raw_window_handle::HasWindowHandle)
    }

    fn raw_display_handle(&self) -> Option<&dyn raw_window_handle::HasDisplayHandle> {
        self.window.as_ref().map(|w| w as &dyn raw_window_handle::HasDisplayHandle)
    }
}

impl X11WindowManager {
    /// Returns the X11 window ID of the overlay window for self-capture
    /// exclusion (RISK-002).
    ///
    /// The returned ID is passed to `XcbCapture::new()` (Story 001) so that
    /// the capture backend can exclude this window from captured frames,
    /// preventing infinite feedback loops.
    ///
    /// # Returns
    ///
    /// `Some(window_id)` if the overlay has been created, `None` otherwise.
    pub fn overlay_window_id(&self) -> Option<u64> {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let handle = self.window.as_ref()?.window_handle().ok()?;
        match handle.as_raw() {
            RawWindowHandle::Xlib(xlib) => Some(xlib.window as u64),
            RawWindowHandle::Xcb(xcb) => Some(u64::from(xcb.window.get())),
            _ => None,
        }
    }
}
```

**Note on winit event loop:** `X11WindowManager::create_overlay()` requires access to a winit `ActiveEventLoop` to create windows. The exact integration pattern (passing the event loop handle vs. creating the window in the event loop callback) will be determined during implementation. The trait signature `create_overlay(&mut self, display_id: &str)` may need the event loop handle passed separately or stored at construction time. This is an implementation detail that the subtasks will resolve.

### `RenderError` -- `crates/luminos-gpu/src/error.rs`

```rust
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
```

### `create_gpu_device()` -- `crates/luminos-gpu/src/device.rs`

```rust
use crate::error::RenderError;

/// Creates the wgpu device and queue for the rendering pipeline.
///
/// Requests `LowPower` adapter preference (integrated GPU) to minimize
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
        .ok_or(RenderError::NoAdapter)?;

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("luminos_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                ..Default::default()
            },
            None,
        )
        .await
        .map_err(|e| RenderError::DeviceCreation {
            message: e.to_string(),
        })?;

    Ok((adapter, device, queue))
}
```

### `configure_surface()` -- `crates/luminos-gpu/src/surface.rs`

```rust
use crate::error::RenderError;

/// Configures the wgpu surface for the overlay window.
///
/// Selects an sRGB-compatible texture format for gamma-correct rendering.
/// Uses `PreMultiplied` alpha for transparent overlay compositing, with
/// fallback to `PostMultiplied` or `Opaque` if unavailable.
///
/// # Arguments
///
/// * `surface` -- The wgpu surface bound to the overlay window.
/// * `adapter` -- The GPU adapter (for capability queries).
/// * `device` -- The GPU device.
/// * `width` -- Surface width in pixels.
/// * `height` -- Surface height in pixels.
/// * `present_mode` -- Frame pacing strategy (default: `Fifo` for vsync).
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
) -> Result<wgpu::TextureFormat, RenderError> {
    let caps = surface.get_capabilities(adapter);

    // Prefer sRGB format for gamma-correct rendering.
    let format = caps
        .formats
        .iter()
        .find(|f| f.is_srgb())
        .copied()
        .or_else(|| caps.formats.first().copied())
        .ok_or_else(|| RenderError::SurfaceConfiguration {
            message: "no compatible surface format found".into(),
        })?;

    // Prefer PreMultiplied alpha for transparent overlay compositing.
    // Fall back to PostMultiplied, then Opaque with a warning.
    let alpha_mode = if caps
        .alpha_modes
        .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
    {
        wgpu::CompositeAlphaMode::PreMultiplied
    } else if caps
        .alpha_modes
        .contains(&wgpu::CompositeAlphaMode::PostMultiplied)
    {
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
    };

    surface.configure(
        device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        },
    );

    Ok(format)
}
```

### Type Unification -- `luminos-types` Crate (Deviation from Original Design)

> **Original approach:** Add serde derives to `luminos-platform` definitions, re-export from `luminos-core`.
> **Actual approach (user-directed):** Create `luminos-types` crate as canonical source for all shared types.

The `luminos-types` crate has zero workspace dependencies (only `serde`), preventing circular dependency risk. All shared data types were moved to `luminos-types` with full serde support. Both `luminos-platform` and `luminos-core` re-export from `luminos-types` for backward compatibility.

**Types in `crates/luminos-types/src/overlay.rs`:**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DockEdge { Top, Bottom, Left, Right }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LensShape { Rectangle, Ellipse }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverlayMode { FullScreen, Lens { width: u32, height: u32, shape: LensShape }, Docked { edge: DockEdge, size_px: u32 } }
```

**Re-exports in `crates/luminos-platform/src/traits/window_manager.rs`:**

```rust
pub use luminos_types::{DockEdge, LensShape, OverlayMode};
```

**Re-exports in `crates/luminos-core/src/config/schema.rs`:**

```rust
pub use luminos_types::{DockEdge, GpuPreference, InterpolationMode, LensShape, PresentMode};
```

**Note:** `CaptureFrame` is in `luminos-types` but does NOT derive `Serialize`/`Deserialize` because it contains `Arc<[u8]>` (GPU pixel data) which is a runtime type not suitable for serialization.

---

## Error Handling

All error handling follows CLAUDE.md conventions:

- **`?` propagation:** `create_overlay` and `create_gpu_device` propagate errors via `?`.
- **`From` conversions:** `wgpu::RequestDeviceError` is converted to `RenderError::DeviceCreation` via `map_err`.
- **No `unwrap()`/`expect()`:** All fallible operations return `Result`. Surface capability queries use `ok_or_else` for missing values.
- **Graceful degradation:** Surface alpha mode falls back (`PreMultiplied` -> `PostMultiplied` -> `Opaque`) with `log::warn!` messages.

---

## Platform Considerations

| Platform | Approach | Notes |
|----------|----------|-------|
| Linux X11 | `winit` window + `wgpu` Vulkan backend | This story's scope. Transparency requires compositing WM. |
| Linux Wayland | Deferred (E02 is X11-only) | Will use `winit` + layer-shell for docked mode. |
| macOS | Deferred | `winit` + Metal backend, NSPanel for floating overlay. |
| OpenBSD | Deferred | Shares X11 backend from `common/x11_common`. |
| Windows | Deferred | `winit` + DX12 backend, AppBar for docked mode. |

---

## Testing Strategy

### Unit Tests

- **`X11WindowManager` construction:** Verify `new()` returns default state with `window: None`.
- **`raw_window_handle` before creation:** Verify `raw_window_handle()` returns `None` before `create_overlay`.
- **`overlay_window_id` before creation:** Verify `overlay_window_id()` returns `None` before `create_overlay`.
- **`RenderError` display messages:** Verify all `RenderError` variants produce correct display strings.
- **Surface format selection logic:** Verify sRGB format is preferred, with fallback.
- **Alpha mode fallback logic:** Verify `PreMultiplied` -> `PostMultiplied` -> `Opaque` cascade.

### Integration Tests

- **Window creation on Xvfb:** Create overlay on headless X11, verify it is created without error.
- **wgpu device creation on Mesa llvmpipe:** Verify `create_gpu_device()` succeeds on software renderer.
- **Surface configuration on Mesa llvmpipe:** Verify `configure_surface()` returns a valid format.

### Acceptance Tests

| Acceptance Criterion | Test Type | Verification Method |
|---------------------|-----------|-------------------|
| AC-1.1 | Integration | Create overlay on Xvfb, verify no error returned |
| AC-1.2 | Integration | Call `set_visible(true/false)` on Xvfb, verify no error |
| AC-1.3 | Integration | Call `set_always_on_top(true)` on Xvfb, verify no error |
| AC-1.4 | Integration | Call `set_overlay_bounds` with valid rect, verify no error |
| AC-2.1 | Integration | `create_gpu_device` on llvmpipe, verify device/queue returned |
| AC-2.2 | Integration | `configure_surface` on llvmpipe, verify sRGB format |
| AC-2.3 | Integration | `get_current_texture` on configured surface, verify success |
| AC-2.4 | Unit | Mock adapter request returning None, verify `NoAdapter` error |
| AC-3.1 | Integration | Set FullScreen mode, verify window covers display |
| AC-4.1 | Unit | Verify `DockEdge` serializes/deserializes correctly |
| AC-4.2 | Unit | Verify `LensShape` serializes/deserializes correctly |
| AC-4.3 | Unit | Run existing tests, verify they pass with re-exports |
| AC-4.4 | Unit | Verify `OverlayMode` serializes/deserializes correctly |
| AC-5.1 | Integration | After create_overlay, verify `raw_window_handle` is Some |
| AC-5.2 | Integration | After create_overlay, verify `raw_display_handle` is Some |
| AC-5.3 | Unit | Before create_overlay, verify handles are None |
| AC-6.1 | Integration | After create_overlay on Xvfb, verify `overlay_window_id()` returns `Some(u64)` with non-zero value |
| AC-6.2 | Unit | Before create_overlay, verify `overlay_window_id()` returns `None` |

---

## Performance Targets

| Metric | Target | Source |
|--------|--------|--------|
| wgpu device creation | <200ms | doc-03 Section 1.3 (startup budget) |
| Surface configuration | <10ms | Trivial operation after device creation |
| Window creation | <50ms | winit window creation is fast |
| Total initialization | <300ms | Within 400ms startup-to-first-frame budget |

---

## Security Considerations

- **RISK-017:** The overlay window may display sensitive screen content. The window itself does not log pixel data. Render targets are GPU-only and not accessible via CPU without explicit readback.
- **No network access:** The overlay window and wgpu device have no network capabilities.
- **X11 trust model:** The overlay uses the existing X11 connection. No additional permissions are required beyond what the user's session provides.

---

## Alternatives Considered

1. **Using raw `x11rb` for window creation instead of `winit`:** Rejected because `winit` provides cross-platform window creation that works on all target platforms. Using raw X11 calls would require duplicating window management code for each platform. `x11rb` may be needed in E05 for X11-specific operations (EWMH struts) that `winit` does not expose, but E02 (FullScreen only) does not require it.

2. **Creating a separate `GpuContext` struct instead of free functions:** Considered but deferred. Story 005 (Render Loop) will likely introduce a `Renderer` struct that bundles device, queue, and pipelines. For this story, free functions (`create_gpu_device`, `configure_surface`) are sufficient and avoid premature abstraction.

3. **Keeping `DockEdge`/`LensShape` duplicated with conversion traits:** Rejected. The unification is straightforward (add serde derives, change imports) and eliminates the conversion layer that would be needed in E04 when the control panel bridges settings to overlay mode changes.
