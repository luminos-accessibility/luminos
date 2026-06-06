//! X11 overlay window manager using x11rb.
//!
//! Controls an **externally-created** overlay window (the tao/Tauri overlay
//! window opened by `luminos-app`) by its X11 window id, via raw X11 protocol
//! requests over an `x11rb` `RustConnection`. It NEVER creates a window and uses
//! **no winit** and **no `tauri` dependency** (RISK-001 / AD-1 / AD-3).
//!
//! `luminos-app` bridges the two layers: it extracts the overlay's XID from the
//! Tauri window (`raw_window_handle`) and constructs this manager with
//! [`X11WindowManager::new`].

use x11rb::connection::Connection as _;
use x11rb::protocol::xproto::{
    AtomEnum, ClientMessageEvent, ConfigureWindowAux, ConnectionExt as _, EventMask, PropMode,
    Window,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

use crate::traits::{OverlayMode, ScreenRect, WindowError, WindowManager};

/// EWMH `_NET_WM_STATE` source action: remove a state hint.
const NET_WM_STATE_REMOVE: u32 = 0;
/// EWMH `_NET_WM_STATE` source action: add a state hint.
const NET_WM_STATE_ADD: u32 = 1;

/// X11 overlay window manager backed by `x11rb`.
///
/// Binds to an already-created overlay window by its X11 window id and drives
/// geometry, visibility, and stacking via raw X11 requests. Only `FullScreen`
/// mode is implemented in E04; `Docked`/`Lens` are deferred to Epic 5 and log a
/// warning while returning `Ok(())` (so callers do not break during E04).
///
/// # No Window Creation
///
/// Unlike the retired winit backend, this manager does not create a window and
/// instantiates no event loop. `create_overlay` only *resolves* the target
/// display's bounds and confirms an XID is bound; the window itself is opened by
/// `luminos-app` (story 001) as a tao/Tauri `WebviewWindow`.
///
/// # Surface Handles
///
/// `raw_window_handle()`/`raw_display_handle()` return `None` in this backend:
/// the wgpu surface is sourced by `luminos-app`'s `OverlayGpu` from the owned
/// Tauri window, not through this trait (AD-3).
///
/// # Platform Notes
///
/// - Transparency requires a compositing WM (Mutter, `KWin`, Picom).
/// - Always-on-top is requested via EWMH `_NET_WM_STATE_ABOVE`. Under a WM-less
///   environment (e.g. CI Xvfb) the property is set but stacking is not enforced.
/// - The bound window is a GTK `ApplicationWindow` (tao/GTK3), not an
///   override-redirect window; raw `ConfigureWindow`/`_NET_WM_STATE` may race
///   with GDK and, under a real WM, may need `_NET_MOVERESIZE_WINDOW` (E05).
pub struct X11WindowManager {
    /// Connection to the X server used for all overlay control requests.
    conn: RustConnection,
    /// Root window of the screen the overlay lives on (EWMH client messages
    /// are sent to the root).
    root: Window,
    /// The bound overlay window's X11 id.
    overlay_xid: u32,
    /// Bounds of the target display (resolved by `create_overlay`).
    display_bounds: ScreenRect,
    /// Current overlay mode.
    current_mode: OverlayMode,
}

impl X11WindowManager {
    /// Binds to an externally-created overlay window by its X11 window id.
    ///
    /// Opens a `RustConnection` to the X server (via `$DISPLAY`) and records the
    /// overlay XID and the target display bounds. Performs no window creation.
    ///
    /// # Errors
    ///
    /// Returns [`WindowError::Platform`] if the X server connection cannot be
    /// established.
    pub fn new(overlay_xid: u32, display_bounds: ScreenRect) -> Result<Self, WindowError> {
        let (conn, screen_num) = x11rb::connect(None).map_err(|e| WindowError::Platform {
            message: format!("failed to connect to X server: {e}"),
        })?;
        let root = conn.setup().roots[screen_num].root;

        log::info!(
            "X11WindowManager bound to overlay window '{overlay_xid}' \
             (display {}x{} at {},{})",
            display_bounds.width,
            display_bounds.height,
            display_bounds.x,
            display_bounds.y
        );

        Ok(Self {
            conn,
            root,
            overlay_xid,
            display_bounds,
            current_mode: OverlayMode::FullScreen,
        })
    }

    /// Returns the bound overlay window's X11 id for the capture path's
    /// self-capture exclusion (RISK-002).
    ///
    /// The returned id is passed to
    /// [`set_excluded_windows`](crate::traits::ScreenCapture::set_excluded_windows)
    /// (story 003) so the capture backend excludes this window from captured
    /// frames, preventing a feedback loop.
    ///
    /// Returned as `u64` to match the `set_excluded_windows(&[u64])` contract.
    #[must_use]
    pub fn overlay_window_id(&self) -> Option<u64> {
        Some(u64::from(self.overlay_xid))
    }

    /// Interns an atom by name, mapping failures to [`WindowError::Platform`].
    fn intern(&self, name: &[u8]) -> Result<u32, WindowError> {
        let name_str = String::from_utf8_lossy(name).into_owned();
        self.conn
            .intern_atom(false, name)
            .map_err(|e| WindowError::Platform {
                message: format!("intern_atom('{name_str}') request failed: {e}"),
            })?
            .reply()
            .map(|r| r.atom)
            .map_err(|e| WindowError::Platform {
                message: format!("intern_atom('{name_str}') reply failed: {e}"),
            })
    }

    /// Flushes pending requests, mapping the failure to [`WindowError::Platform`].
    fn flush(&self) -> Result<(), WindowError> {
        self.conn.flush().map_err(|e| WindowError::Platform {
            message: format!("X11 flush failed: {e}"),
        })
    }
}

impl WindowManager for X11WindowManager {
    fn create_overlay(&mut self, display_id: &str) -> Result<(), WindowError> {
        let bounds = find_display_bounds(display_id)?;
        self.display_bounds = bounds;
        log::info!(
            "create_overlay: bound overlay '{}' resolved to display '{display_id}' \
             ({}x{} at {},{})",
            self.overlay_xid,
            bounds.width,
            bounds.height,
            bounds.x,
            bounds.y
        );
        Ok(())
    }

    fn set_overlay_bounds(&self, bounds: ScreenRect) -> Result<(), WindowError> {
        let aux = ConfigureWindowAux::new()
            .x(bounds.x)
            .y(bounds.y)
            .width(bounds.width)
            .height(bounds.height);

        self.conn
            .configure_window(self.overlay_xid, &aux)
            .map_err(|e| WindowError::Platform {
                message: format!("configure_window('{}') failed: {e}", self.overlay_xid),
            })?;
        self.flush()?;

        log::debug!(
            "set_overlay_bounds: overlay '{}' -> '{}x{}' at '{},{}'",
            self.overlay_xid,
            bounds.width,
            bounds.height,
            bounds.x,
            bounds.y
        );
        Ok(())
    }

    fn set_overlay_mode(&mut self, mode: OverlayMode) -> Result<(), WindowError> {
        match mode {
            OverlayMode::FullScreen => {
                self.set_overlay_bounds(self.display_bounds)?;
                self.current_mode = OverlayMode::FullScreen;
                Ok(())
            }
            OverlayMode::Docked { .. } | OverlayMode::Lens { .. } => {
                log::warn!(
                    "set_overlay_mode: '{mode:?}' deferred to E05; ignoring (overlay '{}')",
                    self.overlay_xid
                );
                Ok(())
            }
        }
    }

    fn set_always_on_top(&self, always_on_top: bool) -> Result<(), WindowError> {
        let wm_state = self.intern(b"_NET_WM_STATE")?;
        let wm_state_above = self.intern(b"_NET_WM_STATE_ABOVE")?;

        let action = if always_on_top {
            NET_WM_STATE_ADD
        } else {
            NET_WM_STATE_REMOVE
        };

        // EWMH _NET_WM_STATE client message: data[0]=action, data[1]=property.
        // Source indication (data[3]) = 1 (normal application).
        let event = ClientMessageEvent::new(
            32,
            self.overlay_xid,
            wm_state,
            [action, wm_state_above, 0, 1, 0],
        );

        self.conn
            .send_event(
                false,
                self.root,
                EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT,
                event,
            )
            .map_err(|e| WindowError::Platform {
                message: format!("send_event(_NET_WM_STATE_ABOVE) failed: {e}"),
            })?;

        // Also set the property directly so a WM-less environment (CI Xvfb) can
        // observe the requested state; under a real WM the client message above
        // is the authoritative path.
        if always_on_top {
            self.conn
                .change_property32(
                    PropMode::REPLACE,
                    self.overlay_xid,
                    wm_state,
                    AtomEnum::ATOM,
                    &[wm_state_above],
                )
                .map_err(|e| WindowError::Platform {
                    message: format!("change_property(_NET_WM_STATE) failed: {e}"),
                })?;
        } else {
            self.conn
                .change_property32(
                    PropMode::REPLACE,
                    self.overlay_xid,
                    wm_state,
                    AtomEnum::ATOM,
                    &[],
                )
                .map_err(|e| WindowError::Platform {
                    message: format!("change_property(_NET_WM_STATE clear) failed: {e}"),
                })?;
        }

        self.flush()?;
        log::debug!(
            "set_always_on_top('{always_on_top}') applied to overlay '{}'",
            self.overlay_xid
        );
        Ok(())
    }

    fn set_visible(&self, visible: bool) -> Result<(), WindowError> {
        if visible {
            self.conn
                .map_window(self.overlay_xid)
                .map_err(|e| WindowError::Platform {
                    message: format!("map_window('{}') failed: {e}", self.overlay_xid),
                })?;
        } else {
            self.conn
                .unmap_window(self.overlay_xid)
                .map_err(|e| WindowError::Platform {
                    message: format!("unmap_window('{}') failed: {e}", self.overlay_xid),
                })?;
        }
        self.flush()?;
        log::debug!(
            "set_visible('{visible}') applied to overlay '{}'",
            self.overlay_xid
        );
        Ok(())
    }

    fn raw_window_handle(&self) -> Option<&dyn raw_window_handle::HasWindowHandle> {
        // The wgpu surface is sourced by `luminos-app`'s `OverlayGpu` from the
        // owned Tauri window, not through this trait (AD-3).
        None
    }

    fn raw_display_handle(&self) -> Option<&dyn raw_window_handle::HasDisplayHandle> {
        None
    }
}

/// Finds a monitor matching `display_id` via xcap and returns its bounds.
///
/// Matches against the monitor name (e.g., "HDMI-1") or the string
/// representation of its numeric ID. Returns the first match.
fn find_display_bounds(display_id: &str) -> Result<ScreenRect, WindowError> {
    let monitors = xcap::Monitor::all().map_err(|e| {
        WindowError::DisplayNotFound(format!("{display_id} (monitor enumeration failed: {e})"))
    })?;

    for monitor in &monitors {
        let name_matches = monitor.name().is_ok_and(|n| n == display_id);
        let id_matches = monitor.id().is_ok_and(|id| id.to_string() == display_id);

        if name_matches || id_matches {
            let x = monitor.x().unwrap_or(0);
            let y = monitor.y().unwrap_or(0);
            let width = monitor.width().unwrap_or(0);
            let height = monitor.height().unwrap_or(0);

            if width == 0 || height == 0 {
                return Err(WindowError::CreationFailed {
                    message: format!(
                        "display '{display_id}' reported zero dimensions ({width}x{height})"
                    ),
                });
            }

            return Ok(ScreenRect {
                x,
                y,
                width,
                height,
            });
        }
    }

    Err(WindowError::DisplayNotFound(display_id.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a test display rect for binding the manager in Xvfb tests.
    /// Only the `ci_platform_tests` integration submodule consumes it.
    #[cfg(all(target_os = "linux", feature = "ci_platform_tests"))]
    pub(crate) fn generate_test_display_bounds() -> ScreenRect {
        ScreenRect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        }
    }

    #[test]
    fn x11_window_manager_constants_match_ewmh() {
        // EWMH _NET_WM_STATE source actions per the spec.
        assert_eq!(NET_WM_STATE_REMOVE, 0);
        assert_eq!(NET_WM_STATE_ADD, 1);
    }

    // --- Integration tests requiring an X11 display server (Xvfb in CI) ---
    //
    // These create their OWN throwaway x11rb window, bind the manager to its
    // XID, and assert geometry/state via GetGeometry/GetWindowAttributes/
    // GetProperty. They never create a winit window and never depend on the
    // tao/Tauri overlay (that path is exercised by the luminos-app subprocess
    // tests). Run under a dedicated Xvfb; gated behind `ci_platform_tests`.

    #[cfg(all(target_os = "linux", feature = "ci_platform_tests"))]
    #[allow(clippy::unwrap_used, clippy::expect_used)]
    mod integration {
        use super::*;
        use x11rb::protocol::xproto::{CreateWindowAux, MapState, WindowClass};
        use x11rb::rust_connection::RustConnection;

        /// Creates a throwaway, unmapped top-level X11 window for binding and
        /// returns its connection + XID. The connection must be kept alive for
        /// the window to survive (the X resource is owned by the connection).
        fn create_test_window() -> (RustConnection, u32, u32) {
            let (conn, screen_num) = x11rb::connect(None).expect("connect to test X server");
            let screen = &conn.setup().roots[screen_num];
            let root = screen.root;
            let xid = conn.generate_id().expect("generate window id");

            let aux = CreateWindowAux::new().background_pixel(screen.white_pixel);
            conn.create_window(
                screen.root_depth,
                xid,
                root,
                100,
                100,
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
            (conn, xid, root)
        }

        /// Reads `_NET_WM_STATE` and returns whether `_NET_WM_STATE_ABOVE` is a
        /// member. Uses a fresh connection (the manager owns its own).
        fn net_wm_state_has_above(xid: u32) -> bool {
            let (conn, _) = x11rb::connect(None).expect("connect for state read");
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
            let reply = conn
                .get_property(false, xid, wm_state, AtomEnum::ATOM, 0, 16)
                .unwrap()
                .reply()
                .unwrap();
            reply
                .value32()
                .map(Iterator::collect::<Vec<u32>>)
                .unwrap_or_default()
                .contains(&wm_state_above)
        }

        // T002: bind to an externally-created XID; overlay_window_id() echoes it.
        #[test]
        fn x11_window_manager_new_binds_xid() {
            let (_conn, xid, _root) = create_test_window();
            let wm = X11WindowManager::new(xid, generate_test_display_bounds())
                .expect("manager should bind to the test window XID");
            assert_eq!(
                wm.overlay_window_id(),
                Some(u64::from(xid)),
                "overlay_window_id must echo the bound XID"
            );
        }

        // T003: set_overlay_bounds applies geometry observable via GetGeometry.
        #[test]
        fn x11_window_manager_set_bounds_applies() {
            let (conn, xid, _root) = create_test_window();
            let wm =
                X11WindowManager::new(xid, generate_test_display_bounds()).expect("bind manager");
            let bounds = ScreenRect {
                x: 10,
                y: 20,
                width: 640,
                height: 480,
            };
            wm.set_overlay_bounds(bounds).expect("set_overlay_bounds");
            conn.sync().expect("sync");

            let geo = conn.get_geometry(xid).unwrap().reply().unwrap();
            assert_eq!(u32::from(geo.width), bounds.width, "width applied");
            assert_eq!(u32::from(geo.height), bounds.height, "height applied");
            // x/y are relative to the parent (root here, no reparenting WM).
            assert_eq!(i32::from(geo.x), bounds.x, "x applied");
            assert_eq!(i32::from(geo.y), bounds.y, "y applied");
        }

        // T004: set_always_on_top toggles the _NET_WM_STATE_ABOVE property.
        #[test]
        fn x11_window_manager_always_on_top_sets_state() {
            let (conn, xid, _root) = create_test_window();
            let wm =
                X11WindowManager::new(xid, generate_test_display_bounds()).expect("bind manager");

            wm.set_always_on_top(true).expect("set always-on-top");
            conn.sync().expect("sync");
            assert!(
                net_wm_state_has_above(xid),
                "_NET_WM_STATE_ABOVE should be present after set_always_on_top(true)"
            );

            wm.set_always_on_top(false).expect("clear always-on-top");
            conn.sync().expect("sync");
            assert!(
                !net_wm_state_has_above(xid),
                "_NET_WM_STATE_ABOVE should be absent after set_always_on_top(false)"
            );
        }

        // T005: set_visible maps and unmaps the window (observable map state).
        #[test]
        fn x11_window_manager_visible_maps_unmaps() {
            let (conn, xid, _root) = create_test_window();
            let wm =
                X11WindowManager::new(xid, generate_test_display_bounds()).expect("bind manager");

            wm.set_visible(true).expect("map");
            conn.sync().expect("sync");
            let attrs = conn.get_window_attributes(xid).unwrap().reply().unwrap();
            assert_eq!(
                attrs.map_state,
                MapState::VIEWABLE,
                "window should be mapped"
            );

            wm.set_visible(false).expect("unmap");
            conn.sync().expect("sync");
            let attrs = conn.get_window_attributes(xid).unwrap().reply().unwrap();
            assert_eq!(
                attrs.map_state,
                MapState::UNMAPPED,
                "window should be unmapped"
            );
        }

        // T006: FullScreen mode sizes the overlay to the bound display bounds.
        #[test]
        fn x11_window_manager_fullscreen_sizes_to_display() {
            let (conn, xid, _root) = create_test_window();
            let bounds = ScreenRect {
                x: 0,
                y: 0,
                width: 1280,
                height: 720,
            };
            let mut wm = X11WindowManager::new(xid, bounds).expect("bind manager");

            wm.set_overlay_mode(OverlayMode::FullScreen)
                .expect("FullScreen mode");
            conn.sync().expect("sync");

            let geo = conn.get_geometry(xid).unwrap().reply().unwrap();
            assert_eq!(u32::from(geo.width), bounds.width, "fullscreen width");
            assert_eq!(u32::from(geo.height), bounds.height, "fullscreen height");
        }

        // T006: Lens/Docked are deferred (Ok + warn) — they must not error and
        // must not resize the bound window.
        #[test]
        fn x11_window_manager_lens_docked_deferred() {
            let (conn, xid, _root) = create_test_window();
            let mut wm =
                X11WindowManager::new(xid, generate_test_display_bounds()).expect("bind manager");

            let docked = wm.set_overlay_mode(OverlayMode::Docked {
                edge: luminos_types::DockEdge::Bottom,
                size_px: 540,
            });
            assert!(
                docked.is_ok(),
                "Docked must return Ok (deferred), got {docked:?}"
            );

            let lens = wm.set_overlay_mode(OverlayMode::Lens {
                width: 400,
                height: 300,
                shape: luminos_types::LensShape::Ellipse,
            });
            assert!(lens.is_ok(), "Lens must return Ok (deferred), got {lens:?}");
            conn.sync().expect("sync");

            // The deferred modes must not have resized the window (still 400x300
            // from create_test_window).
            let geo = conn.get_geometry(xid).unwrap().reply().unwrap();
            assert_eq!(geo.width, 400, "deferred mode must not resize");
            assert_eq!(geo.height, 300, "deferred mode must not resize");
        }

        // T007: the X11 backend sources no surface handle (AD-3).
        #[test]
        fn x11_window_manager_handles_return_none() {
            let (_conn, xid, _root) = create_test_window();
            let wm =
                X11WindowManager::new(xid, generate_test_display_bounds()).expect("bind manager");
            assert!(
                wm.raw_window_handle().is_none(),
                "raw_window_handle must be None (surface sourced in luminos-app)"
            );
            assert!(
                wm.raw_display_handle().is_none(),
                "raw_display_handle must be None (surface sourced in luminos-app)"
            );
        }

        // T002/FR-2: create_overlay resolves a real display's bounds without
        // creating a window (the bound XID is unchanged).
        #[test]
        fn x11_window_manager_create_overlay_resolves_bounds() {
            let (_conn, xid, _root) = create_test_window();
            let mut wm =
                X11WindowManager::new(xid, generate_test_display_bounds()).expect("bind manager");

            let monitors = xcap::Monitor::all().expect("enumerate monitors");
            assert!(!monitors.is_empty(), "Xvfb provides at least one monitor");
            let first = &monitors[0];
            let display_id = first
                .name()
                .unwrap_or_else(|_| first.id().unwrap().to_string());

            wm.create_overlay(&display_id)
                .expect("create_overlay resolves bounds");
            // The bound XID is preserved (create_overlay creates nothing).
            assert_eq!(wm.overlay_window_id(), Some(u64::from(xid)));
        }
    }
}
