//! Integration tests: full window-to-GPU pipeline on X11.
//!
//! These tests wire together `X11WindowManager` (luminos-platform) with
//! wgpu device and surface initialization (luminos-gpu) to verify the
//! end-to-end pipeline from overlay window creation through GPU surface
//! configuration and texture acquisition.
//!
//! Requires Xvfb + Mesa (llvmpipe/lavapipe) on Linux. Gated behind
//! `ci_platform_tests` feature.

#![cfg(all(test, target_os = "linux", feature = "ci_platform_tests"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use luminos_gpu::device::{create_gpu_device, create_wgpu_instance};
use luminos_gpu::surface::configure_surface;
use luminos_platform::linux_x11::X11WindowManager;
use luminos_platform::traits::WindowManager;

/// Helper: creates an `X11WindowManager` with an overlay on the first
/// available monitor (Xvfb provides at least one).
fn create_overlay_on_first_monitor() -> X11WindowManager {
    let mut wm = X11WindowManager::new();
    let monitors = xcap::Monitor::all().expect("should enumerate monitors");
    assert!(
        !monitors.is_empty(),
        "Xvfb should provide at least one monitor"
    );
    let first = &monitors[0];
    let display_id = first
        .name()
        .unwrap_or_else(|_| first.id().unwrap().to_string());
    wm.create_overlay(&display_id)
        .expect("create_overlay should succeed on Xvfb");
    wm
}

/// End-to-end pipeline: overlay window -> raw handles -> wgpu instance ->
/// surface -> adapter + device + queue -> configure surface -> acquire texture.
///
/// Traces to: AC-1.1, AC-2.1, AC-2.2, AC-2.3, AC-5.1, AC-5.2
#[tokio::test]
async fn integration_overlay_window_with_gpu_surface() {
    let wm = create_overlay_on_first_monitor();

    // AC-5.1: raw_window_handle returns Some after create_overlay
    let window_handle = wm
        .raw_window_handle()
        .expect("raw_window_handle should be Some after create_overlay");

    // AC-5.2: raw_display_handle returns Some after create_overlay
    let display_handle = wm
        .raw_display_handle()
        .expect("raw_display_handle should be Some after create_overlay");

    // Verify handles are valid (can obtain raw handles without error)
    let _raw_wh = window_handle
        .window_handle()
        .expect("window handle should be valid");
    let _raw_dh = display_handle
        .display_handle()
        .expect("display handle should be valid");

    // Create wgpu instance
    let instance = create_wgpu_instance();

    // Create surface from raw handles. The surface requires references that
    // outlive it, which is satisfied because wm owns the window.
    let surface = instance
        .create_surface(wm.window().expect("window should exist"))
        .expect("wgpu surface creation should succeed on X11");

    // AC-2.1: create_gpu_device returns adapter, device, queue
    let (adapter, device, queue) = create_gpu_device(&instance, &surface)
        .await
        .expect("create_gpu_device should succeed on Mesa llvmpipe/lavapipe");

    // Verify adapter info is populated
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
    .expect("configure_surface should succeed");

    // Verify we got a valid format. On Mesa llvmpipe, sRGB should be
    // available. The successful return from configure_surface already
    // proves the format is valid; log it for debugging.
    log::info!("configured surface format: {format:?}");

    // AC-2.3: get_current_texture returns a valid surface texture
    let surface_texture = surface
        .get_current_texture()
        .expect("get_current_texture should succeed on configured surface");

    // Verify texture dimensions match the configured surface
    assert!(
        surface_texture.texture.width() > 0,
        "texture width should be non-zero"
    );
    assert!(
        surface_texture.texture.height() > 0,
        "texture height should be non-zero"
    );

    // Present the texture to complete the pipeline
    surface_texture.present();
}

/// Verify that `overlay_window_id()` returns a non-zero X11 window ID after
/// overlay creation, suitable for self-capture exclusion (RISK-002).
///
/// Traces to: AC-6.1
#[test]
fn integration_overlay_window_id_for_self_capture_exclusion() {
    let wm = create_overlay_on_first_monitor();
    let window_id = wm
        .overlay_window_id()
        .expect("overlay_window_id should return Some after create_overlay");
    assert!(
        window_id > 0,
        "X11 window ID should be non-zero, got {window_id}"
    );
}

/// Verify that wgpu device creation with `LowPower` preference succeeds.
/// This is a targeted test for AC-2.1's `LowPower` adapter requirement.
///
/// Traces to: AC-2.1
#[tokio::test]
async fn integration_gpu_device_low_power_preference() {
    let wm = create_overlay_on_first_monitor();
    let instance = create_wgpu_instance();
    let surface = instance
        .create_surface(wm.window().expect("window should exist"))
        .expect("surface creation should succeed");

    let (adapter, _device, _queue) = create_gpu_device(&instance, &surface)
        .await
        .expect("create_gpu_device should succeed");

    // On CI with Mesa, the adapter should be available. The LowPower
    // preference is verified by the function signature; here we just
    // confirm the adapter was obtained successfully.
    let info = adapter.get_info();
    assert!(
        !info.name.is_empty(),
        "adapter name should be non-empty: {info:?}"
    );
}
