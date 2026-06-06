//! Integration tests: overlay mode + control on X11 via the x11rb backend.
//!
//! Tests `X11WindowManager` against a real X11 display server (Xvfb in CI). The
//! manager binds to an externally-created window by its XID and drives geometry,
//! visibility, stacking, and `FullScreen` mode through raw X11 requests. Each
//! test creates its OWN throwaway x11rb window (the manager never creates one)
//! and asserts via `GetGeometry`/`GetWindowAttributes`/`GetProperty`.
//!
//! Requires Xvfb on Linux. Gated behind the `ci_platform_tests` feature.

#![cfg(all(test, target_os = "linux", feature = "ci_platform_tests"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use luminos_platform::linux_x11::X11WindowManager;
use luminos_platform::traits::{OverlayMode, ScreenRect, WindowManager};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, CreateWindowAux, MapState, WindowClass};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

/// Creates a throwaway, unmapped top-level X11 window. The returned connection
/// owns the X resource, so it must be kept alive for the window to survive.
fn create_test_window() -> (RustConnection, u32) {
    let (conn, screen_num) = x11rb::connect(None).expect("connect to test X server");
    let screen = &conn.setup().roots[screen_num];
    let xid = conn.generate_id().expect("generate window id");
    let aux = CreateWindowAux::new().background_pixel(screen.white_pixel);
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
        &aux,
    )
    .expect("create test window")
    .check()
    .expect("create_window checked");
    conn.flush().expect("flush after create");
    (conn, xid)
}

/// Binds a manager to a fresh throwaway window with the given display bounds.
fn bind_manager(bounds: ScreenRect) -> (RustConnection, u32, X11WindowManager) {
    let (conn, xid) = create_test_window();
    let wm = X11WindowManager::new(xid, bounds).expect("manager should bind to the XID");
    (conn, xid, wm)
}

fn test_bounds() -> ScreenRect {
    ScreenRect {
        x: 0,
        y: 0,
        width: 1280,
        height: 720,
    }
}

/// Verify that `FullScreen` mode sizes the bound window to the display bounds.
///
/// Traces to: AC-1.2
#[test]
fn integration_overlay_mode_fullscreen_sizes_to_display() {
    let bounds = test_bounds();
    let (conn, xid, mut wm) = bind_manager(bounds);

    wm.set_overlay_mode(OverlayMode::FullScreen)
        .expect("set_overlay_mode(FullScreen) should succeed");
    conn.sync().expect("sync");

    let geo = conn.get_geometry(xid).unwrap().reply().unwrap();
    assert_eq!(
        u32::from(geo.width),
        bounds.width,
        "fullscreen width applied"
    );
    assert_eq!(
        u32::from(geo.height),
        bounds.height,
        "fullscreen height applied"
    );
}

/// Verify that Docked mode is deferred (Ok + warn) and does not resize.
///
/// Traces to: AC-1.2 (Lens/Docked deferral)
#[test]
fn integration_overlay_mode_docked_deferred() {
    let (conn, xid, mut wm) = bind_manager(test_bounds());

    let result = wm.set_overlay_mode(OverlayMode::Docked {
        edge: luminos_types::DockEdge::Bottom,
        size_px: 540,
    });
    assert!(result.is_ok(), "Docked mode is deferred to E05, returns Ok");
    conn.sync().expect("sync");

    // The window must be unchanged (still 400x300 from create_test_window).
    let geo = conn.get_geometry(xid).unwrap().reply().unwrap();
    assert_eq!(geo.width, 400, "deferred Docked must not resize");
    assert_eq!(geo.height, 300, "deferred Docked must not resize");
}

/// Verify that Lens mode is deferred (Ok + warn) and does not resize.
///
/// Traces to: AC-1.2 (Lens/Docked deferral)
#[test]
fn integration_overlay_mode_lens_deferred() {
    let (conn, xid, mut wm) = bind_manager(test_bounds());

    let result = wm.set_overlay_mode(OverlayMode::Lens {
        width: 400,
        height: 300,
        shape: luminos_types::LensShape::Ellipse,
    });
    assert!(result.is_ok(), "Lens mode is deferred to E05, returns Ok");
    conn.sync().expect("sync");

    let geo = conn.get_geometry(xid).unwrap().reply().unwrap();
    assert_eq!(geo.width, 400, "deferred Lens must not resize");
    assert_eq!(geo.height, 300, "deferred Lens must not resize");
}

/// Verify that `set_visible` maps and unmaps the bound window after `FullScreen`.
///
/// Traces to: AC-1.1
#[test]
fn integration_overlay_mode_fullscreen_then_visible() {
    let (conn, xid, mut wm) = bind_manager(test_bounds());
    wm.set_overlay_mode(OverlayMode::FullScreen)
        .expect("FullScreen mode should succeed");

    wm.set_visible(true).expect("set_visible(true)");
    conn.sync().expect("sync");
    let attrs = conn.get_window_attributes(xid).unwrap().reply().unwrap();
    assert_eq!(attrs.map_state, MapState::VIEWABLE, "window mapped");

    wm.set_visible(false).expect("set_visible(false)");
    conn.sync().expect("sync");
    let attrs = conn.get_window_attributes(xid).unwrap().reply().unwrap();
    assert_eq!(attrs.map_state, MapState::UNMAPPED, "window unmapped");
}

/// Verify that geometry + always-on-top + visibility all apply together
/// (AC-1.1 end-to-end through the trait against one bound window).
///
/// Traces to: AC-1.1
#[test]
fn integration_overlay_geometry_stacking_visibility() {
    let (conn, xid, wm) = bind_manager(test_bounds());

    let rect = ScreenRect {
        x: 5,
        y: 7,
        width: 800,
        height: 600,
    };
    wm.set_overlay_bounds(rect).expect("set_overlay_bounds");
    wm.set_always_on_top(true).expect("set_always_on_top(true)");
    wm.set_visible(true).expect("set_visible(true)");
    conn.sync().expect("sync");

    let geo = conn.get_geometry(xid).unwrap().reply().unwrap();
    assert_eq!(u32::from(geo.width), rect.width);
    assert_eq!(u32::from(geo.height), rect.height);
    assert_eq!(i32::from(geo.x), rect.x);
    assert_eq!(i32::from(geo.y), rect.y);

    let attrs = conn.get_window_attributes(xid).unwrap().reply().unwrap();
    assert_eq!(attrs.map_state, MapState::VIEWABLE);

    // _NET_WM_STATE_ABOVE membership (property set even under WM-less Xvfb).
    let wm_state = conn
        .intern_atom(false, b"_NET_WM_STATE")
        .unwrap()
        .reply()
        .unwrap()
        .atom;
    let wm_state_above = conn
        .intern_atom(false, b"_NET_WM_STATE_ABOVE")
        .unwrap()
        .reply()
        .unwrap()
        .atom;
    let prop = conn
        .get_property(false, xid, wm_state, AtomEnum::ATOM, 0, 16)
        .unwrap()
        .reply()
        .unwrap();
    let members: Vec<u32> = prop.value32().map(Iterator::collect).unwrap_or_default();
    assert!(
        members.contains(&wm_state_above),
        "_NET_WM_STATE_ABOVE should be set"
    );
}
