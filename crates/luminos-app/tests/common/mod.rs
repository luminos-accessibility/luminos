//! Shared harness for the Story E04/001 subprocess integration tests.
//!
//! `tauri::App::run` never returns and must own the main thread, so the running
//! app cannot be asserted in-process. Each test spawns the real `luminos-app`
//! binary under a DEDICATED `Xvfb` display + `picom` compositor, drives it
//! externally, and asserts via:
//! - structured stdout log lines (`redraw=N`, `surface_created`, `shutdown=clean`),
//! - the X11 window tree (via `x11rb`, which sees override-redirect/WM-less windows
//!   that `xdotool --name` does not), and
//! - the process exit code.
//!
//! Tests gracefully SKIP (return without asserting) when `Xvfb`/`picom` are not
//! available, mirroring the E03 platform-test pattern. CI MUST provide them.
//!
//! Only compiled on Linux under the `ci_platform_tests` feature.
#![cfg(all(target_os = "linux", feature = "ci_platform_tests"))]
#![allow(dead_code)]

use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, MapState};

/// Picks a per-test X display number to avoid collisions across parallel tests.
fn next_display() -> u32 {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    // Base well clear of the live :1 and the CI default :99.
    180 + COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Returns whether a binary is on `PATH`.
fn tool_available(name: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {name}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// A running `Xvfb` + `picom` display that tears itself down on drop.
pub struct TestDisplay {
    pub display: String,
    xvfb: Child,
    picom: Option<Child>,
}

impl TestDisplay {
    /// Launches `Xvfb` + `picom` on a fresh display. Returns `None` (skip) if
    /// `Xvfb` is unavailable.
    pub fn launch() -> Option<Self> {
        if !tool_available("Xvfb") {
            eprintln!("SKIP: Xvfb not available");
            return None;
        }
        let n = next_display();
        let display = format!(":{n}");
        let _ = std::fs::remove_file(format!("/tmp/.X{n}-lock"));

        let xvfb = Command::new("Xvfb")
            .args([&display, "-screen", "0", "1920x1080x24"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;

        // Wait for the X server to accept connections.
        let mut this = Self {
            display: display.clone(),
            xvfb,
            picom: None,
        };
        if !this.wait_for_x(Duration::from_secs(10)) {
            eprintln!("SKIP: Xvfb '{display}' did not become ready");
            return None;
        }

        if tool_available("picom") {
            this.picom = Command::new("picom")
                .args(["--backend", "xrender", "--no-vsync"])
                .env("DISPLAY", &display)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .ok();
            // Give picom a moment to acquire the _NET_WM_CM_S0 selection.
            std::thread::sleep(Duration::from_millis(800));
        } else {
            eprintln!("note: picom not available; transparency path untested");
        }
        Some(this)
    }

    /// Launches `Xvfb` WITHOUT a compositor (no picom), to exercise the
    /// `NoCompositor` path (NFR-3). Returns `None` (skip) if `Xvfb` is missing.
    pub fn launch_without_compositor() -> Option<Self> {
        if !tool_available("Xvfb") {
            eprintln!("SKIP: Xvfb not available");
            return None;
        }
        let n = next_display();
        let display = format!(":{n}");
        let _ = std::fs::remove_file(format!("/tmp/.X{n}-lock"));
        let xvfb = Command::new("Xvfb")
            .args([&display, "-screen", "0", "1920x1080x24"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        let this = Self {
            display,
            xvfb,
            picom: None,
        };
        if !this.wait_for_x(Duration::from_secs(10)) {
            eprintln!("SKIP: Xvfb did not become ready");
            return None;
        }
        Some(this)
    }

    fn wait_for_x(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if x11rb::connect(Some(&self.display)).is_ok() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        false
    }

    /// Whether picom is running on this display.
    pub fn has_compositor(&self) -> bool {
        self.picom.is_some()
    }
}

impl Drop for TestDisplay {
    fn drop(&mut self) {
        if let Some(mut p) = self.picom.take() {
            let _ = p.kill();
            let _ = p.wait();
        }
        let _ = self.xvfb.kill();
        let _ = self.xvfb.wait();
    }
}

/// Path to the compiled `luminos-app` binary under test.
pub fn app_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_luminos-app"))
}

/// A spawned `luminos-app` process with captured stdout+stderr.
pub struct RunningApp {
    child: Child,
    log_path: PathBuf,
}

impl RunningApp {
    /// Spawns the app on the given display with the headless-webkit env vars
    /// required for GTK window realization + software GL under Xvfb.
    pub fn spawn(display: &str, extra_env: &[(&str, &str)]) -> std::io::Result<Self> {
        let log_path = std::env::temp_dir().join(format!(
            "luminos_app_{}_{}.log",
            std::process::id(),
            next_display()
        ));
        let log_file = std::fs::File::create(&log_path)?;
        let err_clone = log_file.try_clone()?;

        let mut cmd = Command::new(app_binary());
        // Put the app (and its webkit subprocesses) in its OWN process group so
        // the SIGTERM we send to it — and any signals it blocks/handles — never
        // propagate to the nextest runner's process group.
        cmd.process_group(0)
            .env("DISPLAY", display)
            // `luminos_app=debug` so the per-frame heartbeat/magnify markers
            // (`redraw=N`, `magnify_present`, …) — emitted at DEBUG to keep a
            // normal `cargo run` quiet — remain visible as subprocess-test log
            // oracles; dependency crates stay at INFO to limit noise.
            .env("RUST_LOG", "info,luminos_app=debug")
            // Force the GDK X11 backend: under a headless Xvfb, GDK's backend
            // auto-detection can fail to realize windows; pinning x11 fixes it.
            .env("GDK_BACKEND", "x11")
            // Headless webkit: without these the GTK windows never realize under
            // Xvfb (the dmabuf/compositing renderer fails on the broken DRI).
            .env("WEBKIT_DISABLE_COMPOSITING_MODE", "1")
            .env("WEBKIT_DISABLE_DMABUF_RENDERER", "1")
            .env("LIBGL_ALWAYS_SOFTWARE", "1")
            // Disable the GTK AT-SPI accessibility bridge. On a CI runner there
            // is no session/a11y D-Bus, and GTK's accessibility init (the
            // atk-bridge module load during the app's GTK setup) blocks ~9s
            // (measured) trying to reach the absent a11y bus — long enough on a
            // slow runner to blow the 20s `wait_for_log` boot-marker timeouts and
            // fail the boot tests. The bridge is irrelevant to these
            // window/IPC/shutdown tests, so turning it off removes the stall
            // entirely (verified: the gap between the config-load log and the
            // first setup-hook log drops from ~9s to ~0). Independent of
            // `DBUS_SESSION_BUS_ADDRESS`, so it also covers the tray tests that
            // deliberately unset that.
            .env("NO_AT_BRIDGE", "1")
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::from(err_clone));
        for (k, v) in extra_env {
            cmd.env(k, v);
        }
        let child = cmd.spawn()?;
        Ok(Self { child, log_path })
    }

    /// The OS process id.
    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    /// Reads the current captured log contents.
    pub fn read_log(&self) -> String {
        std::fs::read_to_string(&self.log_path).unwrap_or_default()
    }

    /// Blocks until `needle` appears in the log or the timeout elapses.
    pub fn wait_for_log(&self, needle: &str, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if self.read_log().contains(needle) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        false
    }

    /// Counts occurrences of lines containing `needle` in the log.
    pub fn count_log_lines(&self, needle: &str) -> usize {
        self.read_log()
            .lines()
            .filter(|l| l.contains(needle))
            .count()
    }

    /// Sends SIGTERM and waits up to `timeout` for the process to exit,
    /// returning the exit code (or `None` on timeout/no code).
    pub fn terminate_and_wait(&mut self, timeout: Duration) -> Option<i32> {
        let pid = i32::try_from(self.child.id()).unwrap_or(-1);
        // SAFETY: `kill(2)` with a valid pid and SIGTERM (15).
        unsafe {
            let _ = kill(pid, 15);
        }
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            match self.child.try_wait() {
                Ok(Some(status)) => return status.code(),
                Ok(None) => std::thread::sleep(Duration::from_millis(50)),
                Err(_) => return None,
            }
        }
        // Hung — force kill and report failure.
        let _ = self.child.kill();
        let _ = self.child.wait();
        None
    }

    /// Waits for the child to exit ON ITS OWN (no signal sent), returning its
    /// exit code, or `None` if it is still running when `timeout` elapses. Used
    /// to assert a self-initiated shutdown (e.g. close-quits-app) rather than a
    /// SIGTERM-driven one.
    pub fn wait_for_exit(&mut self, timeout: Duration) -> Option<i32> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            match self.child.try_wait() {
                Ok(Some(status)) => return status.code(),
                Ok(None) => std::thread::sleep(Duration::from_millis(50)),
                Err(_) => return None,
            }
        }
        None
    }
}

impl Drop for RunningApp {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.log_path);
    }
}

// Minimal `kill(2)` binding for sending SIGTERM to the child under test.
unsafe extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}

/// An observed top-level X11 window.
#[derive(Debug, Clone)]
pub struct XWindow {
    pub id: u32,
    pub name: String,
    pub mapped: bool,
    pub override_redirect: bool,
    pub width: u16,
    pub height: u16,
    pub has_motif_undecorated: bool,
}

/// Walks the X11 root window tree (one level) and returns top-level windows
/// whose `WM_NAME` contains `name_substr`. Uses `x11rb` so override-redirect and
/// WM-less windows are visible (unlike `xdotool --name`).
pub fn find_windows(display: &str, name_substr: &str) -> Vec<XWindow> {
    let Ok((conn, screen_num)) = x11rb::connect(Some(display)) else {
        return Vec::new();
    };
    let root = conn.setup().roots[screen_num].root;
    let Some(tree) = conn.query_tree(root).ok().and_then(|c| c.reply().ok()) else {
        return Vec::new();
    };

    let motif_atom = conn
        .intern_atom(true, b"_MOTIF_WM_HINTS")
        .ok()
        .and_then(|c| c.reply().ok())
        .map(|r| r.atom);

    let mut out = Vec::new();
    for w in tree.children {
        let name = window_name(&conn, w);
        if !name.contains(name_substr) {
            continue;
        }
        let Some(attrs) = conn
            .get_window_attributes(w)
            .ok()
            .and_then(|c| c.reply().ok())
        else {
            continue;
        };
        let geo = conn.get_geometry(w).ok().and_then(|c| c.reply().ok());
        let (width, height) = geo.map_or((0, 0), |g| (g.width, g.height));
        let has_motif_undecorated =
            motif_atom.is_some_and(|atom| motif_undecorated(&conn, w, atom));

        out.push(XWindow {
            id: w,
            name,
            mapped: attrs.map_state == MapState::VIEWABLE,
            override_redirect: attrs.override_redirect,
            width,
            height,
            has_motif_undecorated,
        });
    }
    out
}

fn window_name<C: Connection>(conn: &C, w: u32) -> String {
    conn.get_property(false, w, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 1024)
        .ok()
        .and_then(|c| c.reply().ok())
        .map(|r| String::from_utf8_lossy(&r.value).to_string())
        .unwrap_or_default()
}

/// `_MOTIF_WM_HINTS` decorations bit: set in `flags` when `decorations` is valid.
const MWM_HINTS_DECORATIONS: u32 = 1 << 1;

/// Reads `_MOTIF_WM_HINTS` and returns whether decorations are disabled.
///
/// Motif hints layout: `[flags, functions, decorations, input_mode, status]`.
/// `flags & MWM_HINTS_DECORATIONS` set with `decorations == 0` = undecorated.
fn motif_undecorated<C: Connection>(conn: &C, w: u32, motif_atom: u32) -> bool {
    let Some(reply) = conn
        .get_property(false, w, motif_atom, motif_atom, 0, 5)
        .ok()
        .and_then(|c| c.reply().ok())
    else {
        return false;
    };
    // Property is an array of 32-bit values.
    let vals: Vec<u32> = reply
        .value32()
        .map(std::iter::Iterator::collect)
        .unwrap_or_default();
    if vals.len() < 3 {
        return false;
    }
    (vals[0] & MWM_HINTS_DECORATIONS) != 0 && vals[2] == 0
}
