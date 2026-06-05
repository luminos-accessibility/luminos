---
name: risk001-dual-event-loop-research
description: RISK-001 research — winit + Tauri 2 dual event loop coexistence in one process. Recommendation, validated facts, citations.
metadata:
  type: project
---

# RISK-001: Dual Event Loop Coexistence (winit + Tauri 2) — Research Findings (2026-06-04)

**Why:** #1 project risk (score 8, P0). Must validate before Phase 1. Luminos needs a wgpu transparent overlay window + Tauri webview control panel in ONE process.
**How to apply:** Fold into E04 DESIGN.md. Drives the overlay window creation architecture.

## Decisive conclusion
- You CANNOT run a second/separate `winit::EventLoop` inside a Tauri process. Hard blocker on macOS: "winit requires control over the principal class. You must create the event loop before other parts of your application initialize NSApplication" (winit issue #3772). One process = one NS/event loop on macOS.
- Recommended: ONE event loop = Tauri's tao-based loop. Build via `tauri::Builder::build()?.run(|app, event| ...)`. Drive wgpu render from `RunEvent::MainEventsCleared`. Create overlay as a SEPARATE tao/Tauri window (NOT a separate winit EventLoop). Get rwh from Tauri `Window`/`WebviewWindow` (implements `HasWindowHandle`+`HasDisplayHandle` since tauri 2.0.0-beta.13) → build wgpu Surface on it.
- DROP winit for the overlay. Keep wgpu unchanged (renders into any rwh-0.6 window).

## Validated version compat (rwh 0.6 — all aligned)
- tao 0.24.0+ uses raw-window-handle 0.6 (default feature `rwh_06`); supports rwh 0.4/0.5/0.6 via feature flags.
- Project pins (verified in Cargo.toml 2026-06-04): wgpu `=29.0.3` (NOT 28 as sometimes stated), winit `=0.30.13`, raw-window-handle `=0.6.2`. All rwh 0.6 compatible.
- tao IS a winit fork; core API similar. BIG difference: tao uses GTK3 on Linux (winit uses direct X11/Wayland).

## Key failure modes / gotchas
- Shared-window flicker: putting wgpu + webview in ONE window → surface contention "fight" → flicker (tauri #9220, closed not-planned, NixOS/Hyprland). Luminos avoids by using TWO separate windows.
- Linux GTK ordering: webkit2gtk needs GTK. tao handles GTK init itself. If using raw winit (don't), must call `gtk::init` before webview + `gtk::main_iteration_do(false)` each loop iter (wry README). Using tao removes this hazard.
- Click-through: tao `set_ignore_cursor_events` works (X11 via XShape). Tauri exposes `set_ignore_cursor_events` (issue #5265). But no per-region hit-test; overlay-of-game projects poll cursor at ~60fps toggling it (manasight blog 2026-03-04).
- tao GTK3 overlay: transparent needs a compositing manager running on X11 (picom — already in CI). `set_decorated(false)`, `set_keep_above(true)`, `with_transparent`. Note tao #7369: stray "Tao Window" GTK label can show in transparent undecorated fullscreen — watch for it.
- override_redirect: current code uses winit `with_override_redirect(true)`. tao/GTK path differs — RISK-002 self-capture mitigation (unmap/remap) must be re-validated under tao/GTK windows. OPEN question, flag for E04 spike.

## Impact on existing E1-E3 code (verified 2026-06-04)
- GOOD: code already abstracted the coupling points.
  - `luminos-core::pipeline::EventNotifier` trait wraps `winit::EventLoopProxy<LuminosEvent>` → swap impl to tao's `EventLoopProxy` / Tauri `AppHandle.run_on_main_thread` or channel. Trait insulates callers.
  - `luminos-platform::WindowManager` trait wraps the winit Window in `linux_x11::window::X11WindowManager`.
- TO CHANGE: `X11WindowManager` (winit `EventLoop::create_window`, `WindowAttributesExtX11`, `with_override_redirect`), `luminos-app/main.rs`, winit dep in luminos-gpu/luminos-core/luminos-platform Cargo.toml.
- wgpu surface creation is unaffected (rwh-based).

## Cross-platform
- macOS: separate winit loop impossible (NSApplication principal class). Must use Tauri/tao loop. Overlay above fullscreen needs NSPanel (screenpipe uses tauri-nspanel at CGShieldingWindowLevel+1; manasight needs macOSPrivateApi + ActivationPolicy::Accessory).
- Windows: technically more permissive, but keep single-loop for portability. WebView2 bootstrapping gotcha on Win10.

## Precedent
- screenpipe (closest precedent): does NOT use winit for a 2nd window. Pure Tauri windows + NSPanel on macOS. Embeds server as library in Tauri process.
- wry has official `examples/wgpu.rs`: winit EventLoop + wgpu + `WebViewBuilder::build_as_child` — but that's ONE window, and uses raw winit (not full Tauri). Useful as wgpu-in-rwh-window proof, not as the Luminos pattern.
- No universally-adopted "wgpu in separate window alongside Tauri webview" example exists yet (2026). Two-window approach is the consensus suggestion.

## Confidence
- "Drop separate winit EventLoop, use single tao/Tauri loop" — HIGH confidence (macOS constraint is a hard architectural fact).
- "Separate transparent overlay window via tao works on X11 with picom + click-through" — MEDIUM-HIGH (well-supported APIs; needs the E04 PoC + override_redirect/self-capture re-validation).

## Key citations (dated)
- winit #3772 NSApplication principal class constraint
- tauri #9220 (2024-03-19) shared-window flicker, closed not-planned
- tauri/wry examples/wgpu.rs (build_as_child, winit+wgpu+webview one window)
- wry README (GTK init ordering for non-GTK windowing)
- tao 0.24.0 release notes (rwh 0.6), DeepWiki tao (GTK3 fork of winit)
- tauri RunEvent docs (MainEventsCleared per-frame tick)
- manasight blog 2026-03-04 (Tauri v2 overlay, click-through polling, macOS specifics)
- screenpipe DeepWiki 2.1 (no winit 2nd window; NSPanel)
- tauri-wgpu-cam (clearlysid) — wgpu texture into Tauri v2 native surface, same window
