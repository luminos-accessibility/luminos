//! X11 keysym-to-[`KeyCode`] mapping and modifier extraction.
//!
//! Converts X11 keysym values (from `GetKeyboardMapping`) to the
//! platform-independent [`KeyCode`] enum, and extracts [`Modifiers`] from
//! `XInput2` modifier state bitmasks.

use crate::traits::input_monitor::{KeyCode, Modifiers};

// ── X11 modifier bitmask constants ──────────────────────────────────────────

/// Bit 0: Shift modifier.
const SHIFT_MASK: u32 = 1 << 0;
/// Bit 2: Control modifier.
const CTRL_MASK: u32 = 1 << 2;
/// Bit 3: Mod1 (Alt) modifier.
const ALT_MASK: u32 = 1 << 3;
/// Bit 6: Mod4 (Super/Meta) modifier.
const META_MASK: u32 = 1 << 6;

/// Maps an X11 keysym to a Luminos [`KeyCode`].
///
/// Keysyms are obtained from X11 keycodes via `GetKeyboardMapping` (core protocol).
/// Both lowercase and uppercase alphabetic keysyms map to the same [`KeyCode`] variant.
///
/// Returns [`KeyCode::Unknown`] with the raw keysym for unmapped values.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn x11_keysym_to_key_code(keysym: u32) -> KeyCode {
    match keysym {
        // ── Alphabetic (lowercase 0x61-0x7A, uppercase 0x41-0x5A) ───────
        0x61 | 0x41 => KeyCode::A,
        0x62 | 0x42 => KeyCode::B,
        0x63 | 0x43 => KeyCode::C,
        0x64 | 0x44 => KeyCode::D,
        0x65 | 0x45 => KeyCode::E,
        0x66 | 0x46 => KeyCode::F,
        0x67 | 0x47 => KeyCode::G,
        0x68 | 0x48 => KeyCode::H,
        0x69 | 0x49 => KeyCode::I,
        0x6A | 0x4A => KeyCode::J,
        0x6B | 0x4B => KeyCode::K,
        0x6C | 0x4C => KeyCode::L,
        0x6D | 0x4D => KeyCode::M,
        0x6E | 0x4E => KeyCode::N,
        0x6F | 0x4F => KeyCode::O,
        0x70 | 0x50 => KeyCode::P,
        0x71 | 0x51 => KeyCode::Q,
        0x72 | 0x52 => KeyCode::R,
        0x73 | 0x53 => KeyCode::S,
        0x74 | 0x54 => KeyCode::T,
        0x75 | 0x55 => KeyCode::U,
        0x76 | 0x56 => KeyCode::V,
        0x77 | 0x57 => KeyCode::W,
        0x78 | 0x58 => KeyCode::X,
        0x79 | 0x59 => KeyCode::Y,
        0x7A | 0x5A => KeyCode::Z,

        // ── Numeric (0x30-0x39) ─────────────────────────────────────────
        0x30 => KeyCode::Key0,
        0x31 => KeyCode::Key1,
        0x32 => KeyCode::Key2,
        0x33 => KeyCode::Key3,
        0x34 => KeyCode::Key4,
        0x35 => KeyCode::Key5,
        0x36 => KeyCode::Key6,
        0x37 => KeyCode::Key7,
        0x38 => KeyCode::Key8,
        0x39 => KeyCode::Key9,

        // ── Function keys (XK_F1=0xFFBE through XK_F12=0xFFC9) ─────────
        0xFFBE => KeyCode::F1,
        0xFFBF => KeyCode::F2,
        0xFFC0 => KeyCode::F3,
        0xFFC1 => KeyCode::F4,
        0xFFC2 => KeyCode::F5,
        0xFFC3 => KeyCode::F6,
        0xFFC4 => KeyCode::F7,
        0xFFC5 => KeyCode::F8,
        0xFFC6 => KeyCode::F9,
        0xFFC7 => KeyCode::F10,
        0xFFC8 => KeyCode::F11,
        0xFFC9 => KeyCode::F12,

        // ── Navigation ──────────────────────────────────────────────────
        0xFF52 => KeyCode::Up,       // XK_Up
        0xFF54 => KeyCode::Down,     // XK_Down
        0xFF51 => KeyCode::Left,     // XK_Left
        0xFF53 => KeyCode::Right,    // XK_Right
        0xFF50 => KeyCode::Home,     // XK_Home
        0xFF57 => KeyCode::End,      // XK_End
        0xFF55 => KeyCode::PageUp,   // XK_Page_Up / XK_Prior
        0xFF56 => KeyCode::PageDown, // XK_Page_Down / XK_Next

        // ── Modifier keys ───────────────────────────────────────────────
        0xFFE1 => KeyCode::ShiftLeft,  // XK_Shift_L
        0xFFE2 => KeyCode::ShiftRight, // XK_Shift_R
        0xFFE3 => KeyCode::CtrlLeft,   // XK_Control_L
        0xFFE4 => KeyCode::CtrlRight,  // XK_Control_R
        0xFFE9 => KeyCode::AltLeft,    // XK_Alt_L
        0xFFEA => KeyCode::AltRight,   // XK_Alt_R
        0xFFEB => KeyCode::MetaLeft,   // XK_Super_L
        0xFFEC => KeyCode::MetaRight,  // XK_Super_R

        // ── Common keys ─────────────────────────────────────────────────
        0x0020 => KeyCode::Space,     // XK_space
        0xFF0D => KeyCode::Enter,     // XK_Return
        0xFF1B => KeyCode::Escape,    // XK_Escape
        0xFF09 => KeyCode::Tab,       // XK_Tab
        0xFF08 => KeyCode::Backspace, // XK_BackSpace
        0xFFFF => KeyCode::Delete,    // XK_Delete

        // ── Punctuation used in shortcuts ───────────────────────────────
        0x002B => KeyCode::Plus,         // XK_plus
        0x002D => KeyCode::Minus,        // XK_minus
        0x003D => KeyCode::Equal,        // XK_equal
        0x005B => KeyCode::BracketLeft,  // XK_bracketleft
        0x005D => KeyCode::BracketRight, // XK_bracketright

        // ── Numpad ──────────────────────────────────────────────────────
        0xFFB0 => KeyCode::Numpad0,        // XK_KP_0
        0xFFB1 => KeyCode::Numpad1,        // XK_KP_1
        0xFFB2 => KeyCode::Numpad2,        // XK_KP_2
        0xFFB3 => KeyCode::Numpad3,        // XK_KP_3
        0xFFB4 => KeyCode::Numpad4,        // XK_KP_4
        0xFFB5 => KeyCode::Numpad5,        // XK_KP_5
        0xFFB6 => KeyCode::Numpad6,        // XK_KP_6
        0xFFB7 => KeyCode::Numpad7,        // XK_KP_7
        0xFFB8 => KeyCode::Numpad8,        // XK_KP_8
        0xFFB9 => KeyCode::Numpad9,        // XK_KP_9
        0xFFAB => KeyCode::NumpadAdd,      // XK_KP_Add
        0xFFAD => KeyCode::NumpadSubtract, // XK_KP_Subtract
        0xFFAA => KeyCode::NumpadMultiply, // XK_KP_Multiply
        0xFFAF => KeyCode::NumpadDivide,   // XK_KP_Divide

        // ── Catch-all ───────────────────────────────────────────────────
        other => KeyCode::Unknown(other),
    }
}

/// Extracts [`Modifiers`] from `XInput2` modifier state.
///
/// Maps X11 modifier bits to the [`Modifiers`] struct.
///
/// Bit mapping: 0=Shift, 2=Control, 3=Mod1/Alt, 6=Mod4/Super.
#[must_use]
pub fn x11_mods_to_modifiers(mods_effective: u32) -> Modifiers {
    Modifiers {
        shift: mods_effective & SHIFT_MASK != 0,
        ctrl: mods_effective & CTRL_MASK != 0,
        alt: mods_effective & ALT_MASK != 0,
        meta: mods_effective & META_MASK != 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── T003: x11_keysym_to_key_code tests ──────────────────────────────

    #[test]
    fn keymap_keysym_a_lowercase() {
        assert_eq!(x11_keysym_to_key_code(0x61), KeyCode::A);
    }

    #[test]
    fn keymap_keysym_a_uppercase() {
        assert_eq!(x11_keysym_to_key_code(0x41), KeyCode::A);
    }

    #[test]
    fn keymap_keysym_0_through_9() {
        let expected = [
            KeyCode::Key0,
            KeyCode::Key1,
            KeyCode::Key2,
            KeyCode::Key3,
            KeyCode::Key4,
            KeyCode::Key5,
            KeyCode::Key6,
            KeyCode::Key7,
            KeyCode::Key8,
            KeyCode::Key9,
        ];
        for (i, &exp) in expected.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let keysym = 0x30 + i as u32;
            assert_eq!(x11_keysym_to_key_code(keysym), exp, "keysym 0x{keysym:02X}");
        }
    }

    #[test]
    fn keymap_keysym_f1_through_f12() {
        let expected = [
            KeyCode::F1,
            KeyCode::F2,
            KeyCode::F3,
            KeyCode::F4,
            KeyCode::F5,
            KeyCode::F6,
            KeyCode::F7,
            KeyCode::F8,
            KeyCode::F9,
            KeyCode::F10,
            KeyCode::F11,
            KeyCode::F12,
        ];
        for (i, &exp) in expected.iter().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let keysym = 0xFFBE + i as u32;
            assert_eq!(x11_keysym_to_key_code(keysym), exp, "keysym 0x{keysym:04X}");
        }
    }

    #[test]
    fn keymap_keysym_arrow_keys() {
        assert_eq!(x11_keysym_to_key_code(0xFF52), KeyCode::Up);
        assert_eq!(x11_keysym_to_key_code(0xFF54), KeyCode::Down);
        assert_eq!(x11_keysym_to_key_code(0xFF51), KeyCode::Left);
        assert_eq!(x11_keysym_to_key_code(0xFF53), KeyCode::Right);
    }

    #[test]
    fn keymap_keysym_modifiers() {
        assert_eq!(x11_keysym_to_key_code(0xFFE1), KeyCode::ShiftLeft);
        assert_eq!(x11_keysym_to_key_code(0xFFE2), KeyCode::ShiftRight);
        assert_eq!(x11_keysym_to_key_code(0xFFE3), KeyCode::CtrlLeft);
        assert_eq!(x11_keysym_to_key_code(0xFFE4), KeyCode::CtrlRight);
        assert_eq!(x11_keysym_to_key_code(0xFFE9), KeyCode::AltLeft);
        assert_eq!(x11_keysym_to_key_code(0xFFEA), KeyCode::AltRight);
        assert_eq!(x11_keysym_to_key_code(0xFFEB), KeyCode::MetaLeft);
        assert_eq!(x11_keysym_to_key_code(0xFFEC), KeyCode::MetaRight);
    }

    #[test]
    fn keymap_keysym_numpad() {
        let expected = [
            (0xFFB0, KeyCode::Numpad0),
            (0xFFB1, KeyCode::Numpad1),
            (0xFFB2, KeyCode::Numpad2),
            (0xFFB3, KeyCode::Numpad3),
            (0xFFB4, KeyCode::Numpad4),
            (0xFFB5, KeyCode::Numpad5),
            (0xFFB6, KeyCode::Numpad6),
            (0xFFB7, KeyCode::Numpad7),
            (0xFFB8, KeyCode::Numpad8),
            (0xFFB9, KeyCode::Numpad9),
            (0xFFAB, KeyCode::NumpadAdd),
            (0xFFAD, KeyCode::NumpadSubtract),
            (0xFFAA, KeyCode::NumpadMultiply),
            (0xFFAF, KeyCode::NumpadDivide),
        ];
        for (keysym, exp) in expected {
            assert_eq!(x11_keysym_to_key_code(keysym), exp, "keysym 0x{keysym:04X}");
        }
    }

    #[test]
    fn keymap_keysym_punctuation() {
        assert_eq!(x11_keysym_to_key_code(0x002B), KeyCode::Plus);
        assert_eq!(x11_keysym_to_key_code(0x002D), KeyCode::Minus);
        assert_eq!(x11_keysym_to_key_code(0x003D), KeyCode::Equal);
        assert_eq!(x11_keysym_to_key_code(0x005B), KeyCode::BracketLeft);
        assert_eq!(x11_keysym_to_key_code(0x005D), KeyCode::BracketRight);
    }

    #[test]
    fn keymap_keysym_common() {
        assert_eq!(x11_keysym_to_key_code(0x0020), KeyCode::Space);
        assert_eq!(x11_keysym_to_key_code(0xFF0D), KeyCode::Enter);
        assert_eq!(x11_keysym_to_key_code(0xFF1B), KeyCode::Escape);
        assert_eq!(x11_keysym_to_key_code(0xFF09), KeyCode::Tab);
        assert_eq!(x11_keysym_to_key_code(0xFF08), KeyCode::Backspace);
        assert_eq!(x11_keysym_to_key_code(0xFFFF), KeyCode::Delete);
    }

    #[test]
    fn keymap_keysym_unknown() {
        assert_eq!(x11_keysym_to_key_code(0xDEAD), KeyCode::Unknown(0xDEAD));
        assert_eq!(x11_keysym_to_key_code(0), KeyCode::Unknown(0));
    }

    // ── T004: x11_mods_to_modifiers tests ───────────────────────────────

    #[test]
    fn keymap_mods_none() {
        let m = x11_mods_to_modifiers(0);
        assert!(!m.shift);
        assert!(!m.ctrl);
        assert!(!m.alt);
        assert!(!m.meta);
    }

    #[test]
    fn keymap_mods_shift() {
        let m = x11_mods_to_modifiers(1);
        assert!(m.shift);
        assert!(!m.ctrl);
        assert!(!m.alt);
        assert!(!m.meta);
    }

    #[test]
    fn keymap_mods_ctrl() {
        let m = x11_mods_to_modifiers(4);
        assert!(!m.shift);
        assert!(m.ctrl);
        assert!(!m.alt);
        assert!(!m.meta);
    }

    #[test]
    fn keymap_mods_alt() {
        let m = x11_mods_to_modifiers(8);
        assert!(!m.shift);
        assert!(!m.ctrl);
        assert!(m.alt);
        assert!(!m.meta);
    }

    #[test]
    fn keymap_mods_meta() {
        let m = x11_mods_to_modifiers(64);
        assert!(!m.shift);
        assert!(!m.ctrl);
        assert!(!m.alt);
        assert!(m.meta);
    }

    #[test]
    fn keymap_mods_ctrl_alt() {
        let m = x11_mods_to_modifiers(12); // 4 | 8
        assert!(!m.shift);
        assert!(m.ctrl);
        assert!(m.alt);
        assert!(!m.meta);
    }

    #[test]
    fn keymap_mods_all() {
        let m = x11_mods_to_modifiers(SHIFT_MASK | CTRL_MASK | ALT_MASK | META_MASK);
        assert!(m.shift);
        assert!(m.ctrl);
        assert!(m.alt);
        assert!(m.meta);
    }
}
