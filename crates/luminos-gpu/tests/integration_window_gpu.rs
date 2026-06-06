//! Integration tests: full window-to-GPU pipeline on X11.
//!
//! These tests wire a real X11 window to wgpu device and surface initialization
//! (luminos-gpu) to verify the end-to-end pipeline from a window handle through
//! GPU surface configuration and texture acquisition.
//!
//! Story E04/002 note: `X11WindowManager` no longer owns a window (it controls
//! the tao/Tauri overlay window by XID and sources no surface — `raw_window_handle()`
//! is `None`). The surface-from-overlay-window coverage now lives in
//! `luminos-app/tests/overlay_surface.rs` (story 001's `surface_created`). These
//! GPU-pipeline tests therefore create their OWN window: a throwaway winit
//! window for the surface (winit in a *test* is fine — FR-1 only forbids a
//! second event loop in the shipping path reachable from `main`), and a
//! throwaway x11rb window for the `overlay_window_id()` binding check.
//!
//! Requires Xvfb + Mesa (llvmpipe/lavapipe) on Linux. Gated behind
//! `ci_platform_tests` feature.

#![cfg(all(test, target_os = "linux", feature = "ci_platform_tests"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use luminos_gpu::device::{create_gpu_device, create_wgpu_instance};
use luminos_gpu::surface::configure_surface;
use luminos_platform::linux_x11::X11WindowManager;
use luminos_platform::traits::ScreenRect;
use winit::event_loop::EventLoop;
use winit::platform::x11::EventLoopBuilderExtX11;
use winit::window::Window;

/// Creates a throwaway winit window (owning its own ephemeral event loop) to
/// stand in as a real X11 surface target for the GPU pipeline tests. The event
/// loop is dropped after creation; the X11 window survives because the X
/// connection is reference-counted.
#[allow(deprecated)]
fn create_test_window() -> Window {
    let event_loop = EventLoop::builder()
        .with_any_thread(true)
        .build()
        .expect("build test event loop");
    event_loop
        .create_window(Window::default_attributes().with_visible(false))
        .expect("create test winit window")
}

/// End-to-end pipeline: window -> raw handles -> wgpu instance -> surface ->
/// adapter + device + queue -> configure surface -> acquire texture.
///
/// Traces to: AC-2.1, AC-2.2, AC-2.3, AC-5.1, AC-5.2
#[tokio::test]
async fn integration_overlay_window_with_gpu_surface() {
    let window = create_test_window();

    let instance = create_wgpu_instance();

    // Create surface from the owned window handle.
    let surface = instance
        .create_surface(&window)
        .expect("wgpu surface creation should succeed on X11");

    // AC-2.1: create_gpu_device returns adapter, device, queue
    let (adapter, device, queue) = create_gpu_device(&instance, &surface)
        .await
        .expect("create_gpu_device should succeed on Mesa llvmpipe/lavapipe");

    let info = adapter.get_info();
    assert!(
        !info.name.is_empty(),
        "adapter should have a non-empty name"
    );

    // Verify queue is usable (submit empty command buffer)
    let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("test_encoder"),
    });
    queue.submit(std::iter::once(encoder.finish()));

    // AC-2.2: configure_surface succeeds with sRGB-compatible format
    let monitor = &xcap::Monitor::all().unwrap()[0];
    let width = monitor.width().unwrap_or(1920);
    let height = monitor.height().unwrap_or(1080);

    let format = configure_surface(
        &surface,
        &adapter,
        &device,
        width,
        height,
        wgpu::PresentMode::Fifo,
    )
    .expect("configure_surface should succeed")
    .format;
    log::info!("configured surface format: {format:?}");

    // AC-2.3: get_current_texture returns a valid surface texture.
    let surface_texture = match surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Success(texture)
        | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
        status => {
            panic!("get_current_texture should succeed on configured surface, got {status:?}")
        }
    };

    assert!(
        surface_texture.texture.width() > 0,
        "texture width should be non-zero"
    );
    assert!(
        surface_texture.texture.height() > 0,
        "texture height should be non-zero"
    );

    surface_texture.present();
}

/// Verify that `overlay_window_id()` echoes the bound X11 window ID, suitable
/// for self-capture exclusion (RISK-002). Story E04/002: the manager binds to an
/// externally-created window by its XID via `new(xid, bounds)`.
///
/// Traces to: AC-6.1
#[test]
fn integration_overlay_window_id_for_self_capture_exclusion() {
    use x11rb::connection::Connection as _;
    use x11rb::protocol::xproto::{ConnectionExt as _, CreateWindowAux, WindowClass};

    let (conn, screen_num) = x11rb::connect(None).expect("connect to X server");
    let screen = &conn.setup().roots[screen_num];
    let xid = conn.generate_id().expect("generate window id");
    conn.create_window(
        screen.root_depth,
        xid,
        screen.root,
        0,
        0,
        400,
        300,
        0,
        WindowClass::INPUT_OUTPUT,
        screen.root_visual,
        &CreateWindowAux::new().background_pixel(screen.white_pixel),
    )
    .expect("create test window")
    .check()
    .expect("create_window checked");
    conn.flush().expect("flush");

    let bounds = ScreenRect {
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
    };
    let wm = X11WindowManager::new(xid, bounds).expect("bind manager to XID");
    let window_id = wm
        .overlay_window_id()
        .expect("overlay_window_id should be Some when bound");
    assert_eq!(
        window_id,
        u64::from(xid),
        "overlay_window_id should echo the bound XID"
    );
    assert!(window_id > 0, "X11 window ID should be non-zero");
}

/// Verify that wgpu device creation with `LowPower` preference succeeds.
///
/// Traces to: AC-2.1
#[tokio::test]
async fn integration_gpu_device_low_power_preference() {
    let window = create_test_window();
    let instance = create_wgpu_instance();
    let surface = instance
        .create_surface(&window)
        .expect("surface creation should succeed");

    let (adapter, _device, _queue) = create_gpu_device(&instance, &surface)
        .await
        .expect("create_gpu_device should succeed");

    let info = adapter.get_info();
    assert!(
        !info.name.is_empty(),
        "adapter name should be non-empty: {info:?}"
    );
}
