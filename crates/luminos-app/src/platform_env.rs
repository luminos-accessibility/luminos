//! Forces the X11/`XWayland` GTK + `WebKit` backend on Linux before any GTK init.
//!
//! Phase 0 is X11-only (native Wayland is Epic E08, RISK-012). Two real runtime
//! bugs trace back to the *shipped* binary not setting the environment that
//! every test + CI harness sets (`tests/common/mod.rs`, the CI E2E job):
//!
//! * On a native **Wayland** session GTK selects its Wayland backend and aborts
//!   with `Gdk-Message: Error 71 (Protocol error) dispatching to Wayland
//!   display` while realizing the windows — the whole overlay/x11rb/xcap stack
//!   assumes an X server.
//! * Under **X11**, `WebKit`'s DMABUF/compositing renderer fails to initialize
//!   (`Failed to create GBM buffer ... Invalid argument`), leaving the webview's
//!   JS context unbootstrapped so the React control panel cannot reach the Tauri
//!   backend over IPC ("cannot connect to backend").
//!
//! Pinning `GDK_BACKEND=x11` and disabling `WebKit`'s compositing/DMABUF renderer
//! makes the shipped binary behave like the (passing) test environment. On a
//! Wayland session we additionally scrub the Wayland capture hints so `xcap`
//! selects its X11 backend against `XWayland` (mirroring the previously test-only
//! `capture_driver::force_x11_capture_backend`), and emit a clear notice that
//! native Wayland support is deferred to E08.

/// Applies the X11/XWayland backend environment on Linux. No-op on other OSes.
///
/// MUST be called at the very top of `main()` — single-threaded, before the GTK
/// toolkit, the `WebKit` web process, or any other thread reads the environment.
/// Idempotent and respectful of values already set (a user override, the test
/// harness, or a real X11 session all win).
pub fn force_x11_backend() {
    #[cfg(target_os = "linux")]
    linux::apply();
}

#[cfg(target_os = "linux")]
mod linux {
    /// Environment variable names we may adjust.
    pub(super) const GDK_BACKEND: &str = "GDK_BACKEND";
    pub(super) const WEBKIT_DISABLE_COMPOSITING_MODE: &str = "WEBKIT_DISABLE_COMPOSITING_MODE";
    pub(super) const WEBKIT_DISABLE_DMABUF_RENDERER: &str = "WEBKIT_DISABLE_DMABUF_RENDERER";
    pub(super) const WAYLAND_DISPLAY: &str = "WAYLAND_DISPLAY";
    pub(super) const XDG_SESSION_TYPE: &str = "XDG_SESSION_TYPE";

    /// A single environment adjustment to perform before GTK init.
    #[derive(Debug, PartialEq, Eq)]
    pub(super) enum EnvAction {
        /// Set `key` to `value` (only emitted when `key` was unset).
        Set(&'static str, &'static str),
        /// Remove `key` from the environment.
        Remove(&'static str),
    }

    /// Reads the live process environment, computes the plan, applies it, and
    /// logs a backend notice. Split from [`x11_backend_env_plan`] so the
    /// decision logic stays pure and unit-testable.
    pub(super) fn apply() {
        let get = |key: &str| std::env::var(key).ok();
        let wayland = is_wayland_session(&get);
        let plan = x11_backend_env_plan(get);

        for action in &plan {
            match *action {
                // SAFETY: invoked at the very top of `main()` while the process
                // is still single-threaded and before GTK/WebKit or any spawned
                // thread reads the environment, so there is no concurrent
                // access to the process environment (the 2024-edition safety
                // requirement for `set_var`/`remove_var`).
                EnvAction::Set(key, value) => unsafe { std::env::set_var(key, value) },
                EnvAction::Remove(key) => unsafe { std::env::remove_var(key) },
            }
        }

        if wayland {
            log::warn!(concat!(
                "native Wayland session detected; forcing the X11/XWayland backend ",
                "(GDK_BACKEND=x11). Native Wayland support is planned for Epic E08 ",
                "(RISK-012) -- screen capture/magnification may be limited under ",
                "XWayland. Run an X11 session for full functionality"
            ));
        } else {
            log::debug!(concat!(
                "pinned X11 GTK+WebKit backend (GDK_BACKEND=x11, WebKit ",
                "DMABUF/compositing renderer disabled)"
            ));
        }
    }

    /// Returns `true` when the environment indicates a native Wayland session.
    pub(super) fn is_wayland_session(get: &impl Fn(&str) -> Option<String>) -> bool {
        if get(WAYLAND_DISPLAY).is_some_and(|v| !v.is_empty()) {
            return true;
        }
        get(XDG_SESSION_TYPE).as_deref() == Some("wayland")
    }

    /// Computes the environment adjustments needed to force the X11/XWayland
    /// backend, given an environment lookup. Pure (no I/O): the caller applies
    /// the returned plan.
    ///
    /// Each backend var is set only when it is currently unset, so an explicit
    /// user override (e.g. `GDK_BACKEND=wayland` for a power user testing native
    /// Wayland), the test harness, or a real X11 session is never overwritten.
    /// On a Wayland session the Wayland capture hints are scrubbed so `xcap`
    /// picks X11 against `XWayland` — unless the user explicitly opted into the
    /// Wayland GDK backend.
    pub(super) fn x11_backend_env_plan(get: impl Fn(&str) -> Option<String>) -> Vec<EnvAction> {
        let mut plan = Vec::new();

        let set_if_absent = |plan: &mut Vec<EnvAction>, key: &'static str, value: &'static str| {
            if get(key).is_none() {
                plan.push(EnvAction::Set(key, value));
            }
        };

        set_if_absent(&mut plan, GDK_BACKEND, "x11");
        set_if_absent(&mut plan, WEBKIT_DISABLE_COMPOSITING_MODE, "1");
        set_if_absent(&mut plan, WEBKIT_DISABLE_DMABUF_RENDERER, "1");

        // A power user can opt into the native Wayland GDK backend; in that case
        // leave the Wayland capture hints alone (they want the Wayland path).
        let opted_into_wayland = get(GDK_BACKEND).as_deref() == Some("wayland");
        if is_wayland_session(&get) && !opted_into_wayland {
            plan.push(EnvAction::Remove(WAYLAND_DISPLAY));
            if get(XDG_SESSION_TYPE).as_deref() != Some("x11") {
                plan.push(EnvAction::Set(XDG_SESSION_TYPE, "x11"));
            }
        }

        plan
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::linux::{EnvAction, is_wayland_session, x11_backend_env_plan};

    /// Builds an env lookup over a fixed set of `(key, value)` pairs.
    fn env_of<'a>(
        pairs: &'a [(&'static str, &'static str)],
    ) -> impl Fn(&str) -> Option<String> + 'a {
        move |key: &str| {
            pairs
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| (*v).to_string())
        }
    }

    #[test]
    fn env_plan_sets_all_backend_vars_when_absent_on_x11() {
        // X11 session: no Wayland hints present.
        let plan = x11_backend_env_plan(env_of(&[("XDG_SESSION_TYPE", "x11")]));
        assert!(plan.contains(&EnvAction::Set("GDK_BACKEND", "x11")));
        assert!(plan.contains(&EnvAction::Set("WEBKIT_DISABLE_COMPOSITING_MODE", "1")));
        assert!(plan.contains(&EnvAction::Set("WEBKIT_DISABLE_DMABUF_RENDERER", "1")));
        // No Wayland scrub on an X11 session.
        assert!(!plan.contains(&EnvAction::Remove("WAYLAND_DISPLAY")));
    }

    #[test]
    fn env_plan_respects_existing_gdk_backend() {
        // The test harness / a user already pinned GDK_BACKEND — never override.
        let plan = x11_backend_env_plan(env_of(&[("GDK_BACKEND", "x11")]));
        assert!(
            !plan
                .iter()
                .any(|a| matches!(a, EnvAction::Set("GDK_BACKEND", _))),
            "must not re-set an already-set GDK_BACKEND: {plan:?}"
        );
    }

    #[test]
    fn env_plan_respects_existing_webkit_vars() {
        let plan = x11_backend_env_plan(env_of(&[
            ("WEBKIT_DISABLE_COMPOSITING_MODE", "0"),
            ("WEBKIT_DISABLE_DMABUF_RENDERER", "0"),
        ]));
        assert!(!plan.iter().any(|a| matches!(
            a,
            EnvAction::Set(
                "WEBKIT_DISABLE_COMPOSITING_MODE" | "WEBKIT_DISABLE_DMABUF_RENDERER",
                _
            )
        )));
    }

    #[test]
    fn env_plan_scrubs_wayland_hints_on_wayland_session() {
        let plan = x11_backend_env_plan(env_of(&[
            ("WAYLAND_DISPLAY", "wayland-0"),
            ("XDG_SESSION_TYPE", "wayland"),
        ]));
        // Forces the backend AND scrubs the Wayland capture hints.
        assert!(plan.contains(&EnvAction::Set("GDK_BACKEND", "x11")));
        assert!(plan.contains(&EnvAction::Remove("WAYLAND_DISPLAY")));
        assert!(plan.contains(&EnvAction::Set("XDG_SESSION_TYPE", "x11")));
    }

    #[test]
    fn env_plan_no_wayland_scrub_when_user_opts_into_wayland_backend() {
        // Power user explicitly wants the native Wayland GDK backend.
        let plan = x11_backend_env_plan(env_of(&[
            ("GDK_BACKEND", "wayland"),
            ("WAYLAND_DISPLAY", "wayland-0"),
            ("XDG_SESSION_TYPE", "wayland"),
        ]));
        assert!(!plan.contains(&EnvAction::Set("GDK_BACKEND", "x11")));
        assert!(
            !plan.contains(&EnvAction::Remove("WAYLAND_DISPLAY")),
            "must not scrub Wayland hints when the user opted into the Wayland backend: {plan:?}"
        );
    }

    #[test]
    fn is_wayland_session_detects_wayland_display() {
        assert!(is_wayland_session(&env_of(&[(
            "WAYLAND_DISPLAY",
            "wayland-0"
        )])));
        assert!(is_wayland_session(&env_of(&[(
            "XDG_SESSION_TYPE",
            "wayland"
        )])));
    }

    #[test]
    fn is_wayland_session_false_on_x11_and_empty_display() {
        assert!(!is_wayland_session(&env_of(&[("XDG_SESSION_TYPE", "x11")])));
        // An empty WAYLAND_DISPLAY must not count as a Wayland session.
        assert!(!is_wayland_session(&env_of(&[("WAYLAND_DISPLAY", "")])));
        assert!(!is_wayland_session(&env_of(&[])));
    }
}
