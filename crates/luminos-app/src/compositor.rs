//! X11 compositor detection for the overlay transparency warning (NFR-3).
//!
//! Transparency on X11 requires a running compositing manager. The EWMH
//! convention is that a compositor owns the `_NET_WM_CM_S<screen>` selection
//! (e.g. `_NET_WM_CM_S0` for screen 0). When no compositor is present the
//! overlay still opens but renders opaque; the app logs a warning and continues
//! rather than panicking.

/// Whether an X11 compositing manager is currently running.
///
/// Returns `true` if the `_NET_WM_CM_S<screen>` selection has an owner on the
/// default screen. On any X11 error (no display, intern/selection query
/// failure) returns `false` — the conservative answer that triggers the
/// "no compositor" warning path without crashing.
#[cfg(target_os = "linux")]
#[must_use]
pub fn compositor_running() -> bool {
    use x11rb::protocol::xproto::ConnectionExt;

    let Ok((conn, screen_num)) = x11rb::connect(None) else {
        return false;
    };
    let atom_name = format!("_NET_WM_CM_S{screen_num}");
    let Ok(cookie) = conn.intern_atom(false, atom_name.as_bytes()) else {
        return false;
    };
    let Ok(reply) = cookie.reply() else {
        return false;
    };
    let Ok(owner_cookie) = conn.get_selection_owner(reply.atom) else {
        return false;
    };
    let Ok(owner) = owner_cookie.reply() else {
        return false;
    };
    owner.owner != x11rb::NONE
}

/// Non-Linux stub: assume a compositor so the warning path is Linux-only.
#[cfg(not(target_os = "linux"))]
#[must_use]
pub fn compositor_running() -> bool {
    true
}

#[cfg(all(test, target_os = "linux"))]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn compositor_running_does_not_panic() {
        // Result depends on the live environment (CI runs picom → true; a bare
        // Xvfb → false). Either way the call must not panic.
        let _ = compositor_running();
    }
}
