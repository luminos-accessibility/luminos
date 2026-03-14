# Luminos Technology Stack Evaluation Report

**Research Type:** Technical Stack Validation & Recommendation
**Date:** 2026-03-13
**Status:** FINAL (post-audit revision)
**Audit:** 12 findings applied from TECH_STACK_AUDIT_REPORT.md (1 Critical, 3 High, 5 Medium, 3 Low)
**Scope:** P0 features — cross-platform screen magnification with GPU rendering and basic TTS

---

## 1. Executive Summary

This report evaluates the technology stack proposed in the Luminos Product Strategy (v1.1) against the project's P0 requirements: cross-platform support (macOS Tahoe, Windows 11, Linux X11 on KDE/GNOME), GPU-accelerated magnification at up to 20x with games-level performance, docked and magnifying-glass zoom modes, mouse/keyboard focus tracking, basic text-to-speech, and optimization for AI-agent-driven development.

The core architecture — Rust backend, Tauri 2.0 control panel, wgpu GPU rendering, and a dual-window design — is validated as sound. However, three material changes to the proposed stack are recommended. First, the `scap` screen capture crate should be replaced with `xcap` (v0.9.1, Apache 2.0), which provides direct X11 support that `scap` lacks. Second, the primary TTS engine should shift from the archived, GPL-licensed Piper to Kokoro (Apache 2.0 model) via the `sherpa-rs`/sherpa-onnx runtime (MIT/Apache 2.0), with espeak-ng phonemization isolated in a subprocess to avoid GPL propagation. Third, `winit` should be explicitly adopted as the window management layer for the native magnification overlay, providing transparent, always-on-top, borderless windows integrated with wgpu across all three platforms.

The recommended stack is well-suited for AI-agent-driven development: Rust's strict compiler catches errors in generated code, TypeScript/React has the largest LLM training corpus, and trait-based abstractions define clear implementation contracts.

---

## 2. Background

The Luminos product strategy proposes building a cross-platform, open-source screen magnification and TTS accessibility suite. The strategy identifies a specific technology stack centered on Tauri 2.0, Rust, wgpu, the `scap` crate, and Piper TTS. This evaluation was commissioned to validate those choices against the following P0 requirements before development begins:

1. Cross-platform: macOS Tahoe, Windows 11, Linux X11 (KDE and GNOME)
2. Docked zoom view that reserves screen space and prevents window overlap
3. Magnifying glass mode following mouse cursor
4. Mouse cursor tracking with centered zoomed view
5. Keyboard focus tracking
6. 20x magnification with hardware-accelerated edge smoothing and anti-aliasing
7. Adjustable refresh rate (20-30fps performance mode, 60+fps quality mode)
8. Basic read-aloud TTS for selected/clipboard text
9. Games-level GPU rendering performance
10. AI-agent optimized development workflow
11. Prefer client-side implementation over cloud APIs

The user specifically required **Linux X11** support (not Wayland), which materially changes the screen capture and window management landscape from the Wayland-focused product strategy.

---

## 3. Recommended Technology Stack

### 3.1 Final Stack Summary

| Component | Recommended Technology | Version | License | Change from Strategy |
|-----------|----------------------|---------|---------|---------------------|
| **Core language** | Rust (2024 edition) | 1.85+ | MIT/Apache 2.0 | No change |
| **Application framework** | Tauri 2.0 (control panel only) | 2.x stable | MIT/Apache 2.0 | Clarified scope |
| **Frontend UI** | TypeScript + React | Latest | MIT | No change |
| **GPU rendering** | wgpu | 28.0.0 | MIT/Apache 2.0 | No change |
| **Window management** | winit | 0.30.13 | Apache 2.0 | **New: explicitly added** |
| **Screen capture** | xcap (primary) | 0.9.1 | Apache 2.0 | **Changed from scap** |
| **Screen capture (Win fallback)** | windows-capture (DXGI) | Latest | MIT | **New: added for performance** |
| **TTS runtime** | sherpa-onnx via sherpa-rs | 0.6.8 | MIT (wrapper) / Apache 2.0 (runtime) | **Changed from Piper** |
| **TTS model (primary)** | Kokoro-82M ONNX (q8 quantized) | v1.0 | Apache 2.0 (model) | **Changed from Piper** |
| **TTS model (lightweight)** | Piper VITS (via sherpa-onnx) | Latest | MIT (model weights) | Retained as option |
| **Phonemizer** | espeak-ng (subprocess) | Latest | GPL-3.0 (isolated) | **Isolation strategy clarified** |
| **TTS fallback** | Platform-native | N/A | N/A | No change |
| **Accessibility (Linux)** | atspi crate | Latest | MIT/Apache 2.0 | No change |
| **Accessibility (macOS)** | AXUIElement via objc2 | Latest | MIT/Apache 2.0 | No change |
| **Accessibility (Windows)** | UI Automation via windows crate | Latest | MIT | No change |
| **Audio output** | cpal | Latest | Apache 2.0 | New: explicitly added |
| **Clipboard** | arboard | Latest | MIT/Apache 2.0 | New: explicitly added |

### 3.2 Architecture Overview (Revised)

```
┌────────────────────────────────────────────────────────┐
│              Tauri Control Panel Window                 │
│     (TypeScript/React: settings, zoom controls,        │
│      voice selection, profile management)               │
│     [Runs in system WebView — NOT performance-critical] │
└─────────────────────┬──────────────────────────────────┘
                      │ Tauri IPC (typed commands)
┌─────────────────────┴──────────────────────────────────┐
│                   Rust Core Engine                      │
│  ┌──────────────────────────────────────────────────┐  │
│  │         Platform Abstraction Layer (traits)       │  │
│  │  ScreenCapture  │ FocusTracker │ TtsEngine       │  │
│  │  WindowManager  │ InputMonitor │ AudioOutput     │  │
│  └──────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────┐  │
│  │            Magnification Pipeline                 │  │
│  │  capture → GPU texture → shader transform →       │  │
│  │  anti-alias → composite → present                 │  │
│  └──────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────┐  │
│  │            TTS Pipeline                           │  │
│  │  text → espeak-ng subprocess (phonemes) →         │  │
│  │  Kokoro ONNX inference → cpal audio output        │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────┬──────────────────────────────────┘
                      │ Platform-specific backends
┌─────────────────────┴──────────────────────────────────┐
│  macOS Backend      │ Windows Backend   │ Linux Backend │
│  ScreenCaptureKit   │ DXGI Duplication  │ X11/XCB       │
│  (via xcap)         │ (via win-capture)  │ (via xcap)    │
│  AXUIElement        │ UI Automation     │ AT-SPI2/D-Bus │
│  (via objc2)        │ (via windows)     │ (via atspi)   │
│  AVSpeech fallback  │ SAPI fallback     │ speech-disp.  │
│  Metal (via wgpu)   │ DX12 (via wgpu)   │ Vulkan(wgpu)  │
│  NSPanel (dock)     │ AppBar API (dock) │ EWMH struts   │
└────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────┐
│          Magnification Overlay Window                   │
│  [winit: transparent, borderless, always-on-top]       │
│  [wgpu: GPU-accelerated rendering surface]             │
│  [Fully independent of Tauri WebView]                  │
└────────────────────────────────────────────────────────┘
```

---

## 4. Data Analysis and Rationale

### 4.1 Application Framework: Tauri 2.0 (Validated)

**Decision: Keep Tauri 2.0 for the control panel; use winit+wgpu directly for the magnification overlay.**

Tauri 2.0 adoption has grown 35% year-over-year since its stable release in late 2024. Benchmarks consistently show ~58% less memory than Electron (~30-40MB idle vs Electron's 80-120MB) and ~96% smaller bundle sizes. The screenpipe project (17K+ GitHub stars) validates this exact stack — Tauri + Rust backend — for a screen-capture-heavy application.

The critical architectural insight, already present in the product strategy, is that the magnification overlay must **bypass Tauri's WebView entirely**. The overlay is a native winit window with wgpu rendering, not a web page. This eliminates all WebkitGTK performance concerns on Linux, since the WebView is only used for settings UI where rendering performance is irrelevant.

Tauri's role is limited to:
- Providing the settings/control panel UI (React/TypeScript)
- Managing IPC between the frontend and Rust backend
- Handling application lifecycle, auto-updates, and packaging
- Providing system tray integration

**Risk: Tauri transparent window support has known bugs** (macOS transparency loss after bundling, Windows resize artifacts). These do not affect Luminos because the magnification overlay uses winit directly, not Tauri's window management.

**AI-development note:** Tauri's typed IPC command system maps cleanly to Rust function signatures, making it straightforward for AI agents to generate both the TypeScript caller and Rust handler from a shared type definition.

### 4.2 GPU Rendering: wgpu (Validated, Strong Recommendation)

**Decision: wgpu is the correct choice for games-level GPU-accelerated magnification.**

wgpu (v28.0.0) is the most mature cross-platform GPU abstraction in the Rust ecosystem, with 17.9M+ total downloads. It powers Firefox's WebGPU implementation, the Bevy game engine, and Deno's GPU compute. It translates a single WebGPU-inspired API to Vulkan (Linux), Metal (macOS), DX12 (Windows), and OpenGL ES as a fallback for older hardware.

For Luminos's magnification pipeline, wgpu provides:

**Shader-based magnification transforms:** A WGSL fragment shader samples the captured screen texture and applies the zoom transform. Bilinear interpolation comes for free via GPU texture sampling (`textureSample` with linear filtering). Bicubic interpolation requires a custom shader (16 texture lookups per pixel) but runs at trivial cost on any GPU made in the last decade.

**Anti-aliasing at high zoom:** At 20x magnification, jagged edges on text and UI elements become pronounced. wgpu supports MSAA (multisample anti-aliasing) natively, and shader-based techniques (FXAA, SMAA) can be implemented as post-processing passes. The `wgpu::MultisampleState` configuration requires 2 lines of code to enable 4x MSAA.

**Performance model:** The magnification pipeline is trivially simple by GPU standards: capture screen to texture, sample texture in fragment shader with transform, output to overlay window. This is far less demanding than any 3D game. On integrated GPUs (Intel UHD 630, Apple M1, AMD Vega), this pipeline will maintain 60fps with negligible GPU utilization.

**Frame rate control:** wgpu's `PresentMode` enum directly maps to the P0 refresh rate requirement:
- `PresentMode::Fifo` — vsync'd at display refresh rate (60fps quality mode)
- `PresentMode::Immediate` — uncapped (for custom frame limiting to 20-30fps performance mode)
- `PresentMode::Mailbox` — low-latency vsync (ideal when available)

**Integration with winit:** wgpu creates a rendering surface from a winit window handle via the `raw-window-handle` trait. This is the standard pattern used by virtually all Rust graphics applications.

### 4.3 Screen Capture: xcap Replaces scap (Material Change)

**Decision: Replace `scap` with `xcap` as the primary screen capture library. Use `windows-capture` as a Windows performance fallback.**

This is the most significant technology change from the product strategy. The `scap` crate (v0.1.0-beta.1, MIT, by CapSoftware) was proposed for unified cross-platform capture. However, `scap` uses **PipeWire** for Linux, which is a Wayland-era technology. While PipeWire can function on X11 sessions with XDG Desktop Portal, this is not reliable across all X11 configurations, and many X11-only setups (especially KDE on X11) may not have the necessary portal infrastructure running.

**xcap** (v0.9.1, Apache 2.0, by nashaofu) is the recommended replacement:

| Criterion | scap (v0.1.0-beta.1) | xcap (v0.9.1) |
|-----------|----------------------|---------------|
| Linux X11 support | Indirect (PipeWire) | **Direct (XCB/XRandR)** |
| Maturity | Beta, docs.rs build failed | Stable, 85K monthly downloads |
| License | MIT | Apache 2.0 |
| macOS support | ScreenCaptureKit | ScreenCaptureKit |
| Windows support | WGC | Windows 8.1+ |
| Video recording | Configurable FPS | Start/stop with frame receiver |
| Downstream usage | ~1 dependent crate | 19 dependent crates |
| Last release | Aug 2025 | Mar 2026 |

xcap is already used by `tauri-plugin-screenshots`, validating its integration with the Tauri ecosystem.

**Platform-specific capture strategy:**

| Platform | Primary | Mechanism | Performance Notes |
|----------|---------|-----------|-------------------|
| Linux X11 | xcap | XCB + xcb_get_image | xcap uses the standard XCB capture path (not the higher-performance XShm shared-memory path, as its `xcb` dependency does not enable the `shm` feature). This is adequate for small source regions at high zoom but may be a bottleneck at low zoom levels with large capture areas. If profiling confirms this, a direct `x11rb`-based capture backend with XShm support should be implemented as a Phase 1 optimization. |
| macOS | xcap | ScreenCaptureKit | Apple's modern API, mandatory from macOS 15. Requires Screen Recording permission. |
| Windows | xcap or windows-capture | DXGI Desktop Duplication | DXGI DD provides dirty-rectangle metadata and GPU-texture output. Higher performance than WGC for continuous capture. No yellow border. windows-capture crate (MIT) supports both WGC and DXGI DD. |

**Capture-to-GPU pipeline:** The performance-critical path is minimizing CPU copies between capture and rendering. The ideal pipeline is:
1. Capture returns a GPU texture handle (DXGI on Windows, IOSurface on macOS)
2. Import that texture directly into wgpu as a source texture
3. Render magnified view via GPU shader — zero CPU pixel manipulation

xcap returns frame data as CPU buffers (BGRA pixels). This means a GPU upload is required each frame. For 1080p at 60fps, this is ~8MB/frame × 60 = 480MB/s of PCIe bandwidth, well within the capability of any modern system. For higher resolutions, capturing only the viewport region (the area visible in the magnified view at the current zoom level) reduces this proportionally. At 20x zoom on a 1080p display, the source region is only 96×54 pixels — negligible bandwidth.

**Optimization path:** For Windows, the `windows-capture` crate can provide DXGI Desktop Duplication output as a D3D11 texture, which can potentially be shared with wgpu's DX12 backend via cross-API texture sharing. This eliminates the CPU copy entirely. This optimization can be pursued after the initial implementation is working.

### 4.4 TTS Engine: Kokoro via sherpa-onnx Replaces Piper (Material Change)

**Decision: Replace Piper with Kokoro as the primary TTS model, delivered via the sherpa-onnx runtime. Use espeak-ng as a subprocess for GPL isolation.**

Three developments since the product strategy was written necessitate this change:

1. **Piper was archived** on October 6, 2025. The project moved to `OHF-Voice/piper1-gpl`, explicitly acknowledging its GPL-3.0 status due to espeak-ng. Development momentum has shifted away from the original project.

2. **Kokoro emerged as the quality leader** among lightweight offline TTS models. Released under Apache 2.0 (model weights), Kokoro-82M produces markedly more natural speech than Piper at comparable inference speed. Community consensus on r/LocalLLaMA and r/TextToSpeech is that Kokoro represents a generational quality improvement.

3. **sherpa-onnx** (Apache 2.0, 10.8K GitHub stars) provides a unified C/C++/Rust runtime that supports Kokoro, Piper (VITS), KittenTTS, Matcha, and other models through a single API. The `sherpa-rs` crate (v0.6.8, MIT) wraps sherpa-onnx for Rust.

**The espeak-ng GPL problem persists regardless of model choice.** Both Piper and Kokoro use espeak-ng for grapheme-to-phoneme (G2P) conversion. This is not a Piper-specific issue — it is structural to current open-source TTS. The mitigation strategies are:

| Strategy | Description | Legal Clarity | Implementation Complexity |
|----------|-------------|---------------|--------------------------|
| **Subprocess isolation** | Run espeak-ng as a separate process; communicate via stdin/stdout | Medium — the FSF GPL FAQ states that pipe communication "normally" makes programs separate, but "if the semantics of the communication are intimate enough, exchanging complex internal data structures, that too could be a basis to consider the two parts as combined." Simple text-in/phonemes-out is likely safe, but **requires legal counsel review** of the specific IPC protocol. | Low — spawn process, pipe text, receive phonemes |
| **GPL for entire project** | License Luminos as GPL-3.0 | Certain — no legal ambiguity | None — but limits downstream adoption |
| **Transformer-based G2P** | Use Kokoro's `misaki` library (released on PyPI, used by Kokoro itself) with transformer-based phonemizer | High — eliminates espeak-ng entirely | Medium — misaki is released and functional for English, Japanese, Korean, and Chinese, but may have accuracy gaps for other languages compared to espeak-ng. Requires evaluation of accuracy for each target language. |
| **Platform-native TTS only** | Drop espeak-ng; use AVSpeech/SAPI/speech-dispatcher | Certain — no GPL code | Low — but lower quality, no offline consistency |

**Recommended approach:** A dual strategy combining subprocess isolation (short-term) with misaki G2P migration (medium-term). In the short term, run espeak-ng as a standalone binary that receives text on stdin and outputs phonemes on stdout. The Luminos binary never links espeak-ng. Ship espeak-ng as a separate bundled executable with its own GPL-3.0 license notice. Keep the IPC protocol deliberately simple (plain text in, phoneme strings out) to strengthen the legal argument for separation — the FSF GPL FAQ considers "intimate semantics" in pipe communication a factor that could make two programs a combined work. **Legal counsel should review the specific IPC protocol before release.** In the medium term, evaluate `misaki` (hexgrad's transformer-based G2P library, already released on PyPI and used by Kokoro itself) as a replacement for espeak-ng phonemization. If misaki's accuracy proves sufficient for Luminos's supported languages, it eliminates the GPL dependency entirely.

**TTS performance comparison:**

| Model | RTF (lower=faster) | Benchmark HW | Model Size | License | Quality (subjective) |
|-------|---------------------|-------------|------------|---------|---------------------|
| Kokoro-82M | 0.25-0.4 | RPi 4 (sherpa-onnx) | ~327MB fp32 ONNX; ~80MB q4 quantized | Apache 2.0 (model) | Near-commercial |
| Piper VITS (medium) | 0.1-0.2 | RPi 4 (sherpa-onnx) | ~60-75MB | MIT (model weights) | Good, slightly robotic |
| KittenTTS | 0.05-0.1 | RPi 4 (sherpa-onnx) | ~25MB | MIT | Good for size |
| Supertonic 2 | 0.006-0.013 | M4 Pro CPU | ~66M params | **OpenRAIL-M** (model weights; use-restriction license) | Good, 5 languages only |

*Note: Supertonic benchmarks are from Apple M4 Pro, not Raspberry Pi 4. Direct RTF comparison with the other models (benchmarked on RPi 4 via sherpa-onnx) is not apples-to-apples. Supertonic's OpenRAIL-M license includes behavioral use restrictions and is not equivalent to permissive licenses like MIT or Apache 2.0.*

All models achieve real-time or faster on desktop CPUs. For the "read aloud selected text" P0 requirement, latency to first audio is more important than throughput. Kokoro's first-chunk latency is under 200ms on modern desktop CPUs, meeting the product strategy's target.

### 4.5 Window Management: winit (New Explicit Recommendation)

**Decision: Use winit as the window creation and event loop library for the magnification overlay.**

winit (v0.30.13, 34.3M total downloads) is the standard Rust cross-platform window management library. It provides the capabilities required for the magnification overlay:

| Capability | winit Support | P0 Feature Served |
|------------|---------------|-------------------|
| Transparent window | `with_transparent(true)` | Magnifying glass mode |
| Borderless window | `with_decorations(false)` | All zoom modes |
| Always-on-top | `WindowLevel::AlwaysOnTop` | All zoom modes |
| Window positioning | `set_outer_position()` | Docked mode, lens mode |
| Window resizing | `set_surface_size()` | Adjustable dock size |
| Click-through | `set_cursor_hittest(false)` | Magnifying glass mode |
| wgpu integration | `raw-window-handle` trait | GPU rendering |
| X11 platform extensions | `winit::platform::x11` | Linux strut properties |
| Mouse/keyboard events | `WindowEvent`, `DeviceEvent` | Cursor tracking |

**Critical X11 detail:** winit's `WindowLevel::AlwaysOnTop` maps to `_NET_WM_STATE_ABOVE` on X11, which is supported by both KDE (KWin) and GNOME (Mutter). For the docked mode's screen reservation, platform-specific X11 code is required to set `_NET_WM_STRUT_PARTIAL` and `_NET_WM_WINDOW_TYPE_DOCK` — this must be done via raw X11 calls (using the `x11rb` or `xcb` crate) on the window handle obtained from winit.

### 4.6 Docked Mode Implementation (Per-Platform Analysis)

The "docked" magnification mode requires the zoomed view to cling to one edge of the screen with a customizable size, and **prevent other windows from overlapping it**. This is the single most platform-divergent P0 feature.

**Linux X11 (KDE + GNOME):**
The EWMH (Extended Window Manager Hints) specification defines `_NET_WM_STRUT_PARTIAL`, which is the standard mechanism for reserving screen space. Both KWin and Mutter respect this property. Implementation:
1. Set `_NET_WM_WINDOW_TYPE` to `_NET_WM_WINDOW_TYPE_DOCK`
2. Set `_NET_WM_STRUT_PARTIAL` with 12 cardinals specifying reserved space
3. Position the window at the edge
4. Set `_NET_WM_STATE_STICKY` to appear on all workspaces

This is well-tested — every Linux panel/taskbar (GNOME Panel, KDE Panel, Polybar, Waybar on X11 fallback) uses this mechanism. Example for reserving 300px at the top:
```
_NET_WM_STRUT_PARTIAL = 0, 0, 300, 0, 0, 0, 0, 0, 0, screen_width, 0, 0
```

**Windows 11:**
The AppBar API (`SHAppBarMessage` + `ABM_NEW`) is the official Windows mechanism. It reserves desktop space identically to the taskbar. The `windows` crate (Microsoft-maintained) provides full access to this API. Implementation:
1. Register the window as an AppBar with `ABM_NEW`
2. Set position with `ABM_SETPOS`
3. The system automatically adjusts the work area, preventing other windows from maximizing behind the appbar

**macOS Tahoe:**
macOS does **not** provide a public API for third-party apps to reserve screen space the way the Dock does. The Dock's behavior is a privileged system feature. The available approaches are:
1. **NSPanel with `NSWindowLevel.floating`** — keeps the window on top, but other windows can still be dragged behind it
2. **Accessibility API manipulation** — monitor other windows via `AXUIElement` and adjust them if they overlap, but this is fragile and requires Accessibility permission
3. **`NSScreen.visibleFrame` monitoring** — not a reservation mechanism; read-only

**Recommended macOS approach:** Use `NSPanel` with floating level and accept that the docked view overlays rather than reserves space. For the P0 release, document this as a known macOS limitation. An always-on-top floating panel that overlays the screen edge provides 90% of the desired UX. Maximized windows will extend behind it, but the magnified content remains visible. This matches how third-party macOS utilities (BetterTouchTool, Rectangle) handle similar scenarios.

### 4.7 Keyboard Focus Tracking (Per-Platform Analysis)

**macOS:** Use `AXUIElementCreateSystemWide()` + `kAXFocusedUIElementAttribute` to get the currently focused element. Register per-application `AXObserver` callbacks for `kAXFocusedUIElementChangedNotification`. Retrieve element bounds via `kAXPositionAttribute` and `kAXSizeAttribute`. The `objc2` crate provides safe Rust bindings to these CoreFoundation/Accessibility APIs. **Requires Accessibility permission.**

**Windows:** Use UI Automation's `IUIAutomationFocusChangedEventHandler`. The `windows` crate provides full UIA bindings. When focus changes, call `get_CurrentBoundingRectangle()` on the focused element to get screen coordinates. UIA is the most reliable method — MSAA is deprecated and has known gaps.

**Linux X11:** The `atspi` crate (MIT/Apache 2.0, by the Odilia screen reader project) provides pure-Rust, async AT-SPI2 bindings over D-Bus. Register for `focus:` events on the AT-SPI bus. When focus changes, query the component's screen extents via the `Component` interface. AT-SPI2 works on X11 (it uses D-Bus, not display server protocols). **Important:** not all applications expose complete accessibility trees — Electron apps, games, and custom-rendered UIs may not report focus changes via AT-SPI2.

**Fallback for applications without accessibility support:** Monitor the mouse pointer position (always available) and offer a manual "follow pointer" mode as default. The accessibility-based focus tracking is a best-effort enhancement.

### 4.8 Magnification Rendering Pipeline (Technical Design)

The following describes the rendering pipeline for a single frame of magnified output:

```
1. INPUT: mouse position (x, y) and zoom level (z)
2. COMPUTE source region:
   - source_width  = viewport_width / z
   - source_height = viewport_height / z
   - source_x = mouse_x - source_width/2  (clamped to screen bounds)
   - source_y = mouse_y - source_height/2  (clamped to screen bounds)
3. CAPTURE: xcap captures the source region (or full screen with region crop)
   → Returns BGRA pixel buffer
4. UPLOAD: Copy pixel buffer to wgpu texture (gpu_texture)
   → wgpu::Queue::write_texture()
5. SHADER: Fragment shader samples gpu_texture with transform:
   - Bicubic interpolation for smooth scaling
   - Gamma-correct resampling
   - Optional: FXAA anti-aliasing pass
6. PRESENT: wgpu renders to the overlay window's swap chain
   → PresentMode controls vsync behavior
```

**Performance budget at 60fps (16.67ms per frame):**
| Step | Expected Time | Notes |
|------|--------------|-------|
| Source region calculation | <0.01ms | Pure math |
| xcap capture (source region) | 1-8ms | xcb_get_image on X11 (non-SHM; see note below); DXGI on Windows |
| GPU texture upload | 0.5-2ms | Depends on region size; at 20x on 1080p: 96×54px = negligible |
| Shader execution | <1ms | Trivial for GPU |
| Present | 0-16ms | Vsync wait (Fifo mode) |
| **Total (excluding vsync)** | **2-8ms** | Well within 16.67ms budget |

At 20x magnification, the source region is extremely small (1/20th of the viewport in each dimension), making both capture and upload nearly instant. The performance bottleneck appears at low zoom levels (1.5-3x) on high-resolution displays, where the source region approaches full-screen size. **X11 capture note:** xcap's XCB dependency does not enable the `shm` feature, meaning it uses `xcb_get_image` (a full X server round-trip per capture) rather than the zero-copy `xcb_shm_get_image` path. At high zoom levels this is immaterial (tiny regions), but at low zoom it may push capture latency above the 1-5ms range. Plan for an `x11rb`-based XShm capture backend as an early optimization target if profiling confirms this bottleneck.

### 4.9 AI-Development Optimization Assessment

The recommended stack scores well on AI-agent development criteria:

| Criterion | Assessment |
|-----------|-----------|
| **Compiler as reviewer** | Rust's borrow checker, type system, and exhaustive match catch errors in AI-generated code at compile time. A study by Strand found Rust-specialized AI coders achieved 73% pass rate on benchmarks, and errors manifest as compiler messages rather than runtime crashes. |
| **LLM training corpus** | TypeScript has the largest LLM training corpus of any typed language. React is the most-used frontend framework. Rust is the fastest-growing systems language in AI training data. All three are well-represented in code generation models. |
| **Clear contracts** | Trait-based platform abstractions (`ScreenCapture`, `FocusTracker`, `TtsEngine`) define precise interfaces that AI agents can implement independently. The Rust compiler enforces trait conformance. |
| **Local testing** | `cargo test` runs unit tests, `cargo clippy` catches anti-patterns, `cargo bench` measures performance. The entire CI pipeline can run locally. wgpu supports a `wgpu::Backends::GL` fallback for CI environments without GPU drivers. |
| **Independent implementation** | Each platform backend is a separate module implementing shared traits. AI agents can work on macOS, Windows, and Linux backends in parallel without merge conflicts. |
| **Compile time trade-off** | Rust compile times are the primary developer experience cost. Initial builds of a Tauri+wgpu project take 2-5 minutes; incremental rebuilds take 10-30 seconds. For AI agents that generate larger code changes, full rebuilds are common. Mitigation: use `cargo check` (type checking without codegen) for fast feedback loops. |

---

## 5. Per-Feature Technology Mapping

### 5.1 P0 Feature Implementation Summary

| P0 Feature | Primary Technology | Secondary/Fallback | Platform Notes |
|------------|-------------------|-------------------|----------------|
| Cross-platform | Rust traits + conditional compilation | — | `#[cfg(target_os)]` for platform backends |
| Docked zoom view | winit window + platform-specific dock API | — | X11: EWMH struts; Win: AppBar; macOS: floating NSPanel |
| Magnifying glass mode | winit (transparent, borderless, always-on-top) + wgpu | — | Follows cursor; click-through on non-magnified area |
| Follow mouse cursor | winit `DeviceEvent::MouseMotion` or `rdev` crate | Platform-native mouse hooks | rdev for global mouse tracking when app not focused |
| Follow keyboard focus | atspi (Linux), AXUIElement (macOS), UIA (Windows) | Mouse-follow fallback | Not all apps expose focus via accessibility API |
| 20x magnification | wgpu shader with bicubic interpolation + MSAA | — | Zoom range 1.5x-20x configurable |
| Adjustable refresh rate | wgpu `PresentMode` + frame limiter | — | Fifo (60fps), manual throttle (20-30fps) |
| Basic TTS | sherpa-rs (Kokoro model) + cpal audio output | Platform-native TTS | espeak-ng subprocess for phonemization |
| Games-level performance | wgpu GPU rendering pipeline | — | Zero CPU pixel manipulation |
| AI-dev optimized | Rust compiler + TypeScript + trait abstractions | — | See Section 4.9 |

### 5.2 Key Rust Crates (Revised)

| Crate | Purpose | Version | License | Maturity | Downloads/mo |
|-------|---------|---------|---------|----------|--------------|
| `tauri` | Application framework (control panel) | 2.x | MIT/Apache 2.0 | Mature | High |
| `wgpu` | GPU rendering | 28.0.0 | MIT/Apache 2.0 | Mature | High |
| `winit` | Window creation/management | 0.30.13 | Apache 2.0 | Mature | 34.3M total |
| `xcap` | Cross-platform screen capture | 0.9.1 | Apache 2.0 | Stable | 85K/mo |
| `windows-capture` | Windows DXGI/WGC capture | Latest | MIT | Mature | Medium |
| `sherpa-rs` | TTS runtime (wraps sherpa-onnx) | 0.6.8 | MIT | Active | Low-Medium |
| `ort` | ONNX Runtime bindings (alternative to sherpa-rs) | 2.0.0-rc.12 | MIT/Apache 2.0 | Active (pre-release but production-recommended) | High |
| `atspi` | Linux AT-SPI2 accessibility | Latest | MIT/Apache 2.0 | Active | Low |
| `objc2` | macOS Objective-C FFI | Latest | MIT/Apache 2.0 | Mature | High |
| `windows` | Windows API bindings | Latest | MIT | Mature (MS) | Very High |
| `cpal` | Cross-platform audio output | Latest | Apache 2.0 | Mature | High |
| `arboard` | Cross-platform clipboard | Latest | MIT/Apache 2.0 | Stable | Medium |
| `rdev` | Global input event monitoring | Latest | MIT | Stable | Medium |
| `x11rb` | X11 protocol bindings (for EWMH struts) | Latest | MIT/Apache 2.0 | Mature | Medium |

---

## 6. Risk and Trade-off Analysis

### 6.1 Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| **espeak-ng GPL propagation** despite subprocess isolation | Low-Medium | Critical | **Requires immediate legal counsel review.** The FSF GPL FAQ states that pipe-based communication "normally" makes programs separate, but warns that "intimate semantics" in the communication protocol can make two programs a combined work. Luminos's use case (plain text in, phoneme strings out) is relatively simple and likely qualifies as separate — but this is ultimately a legal determination, not a technical one. Ship espeak-ng as a separate binary with its own GPL-3.0 license notice. Keep the IPC protocol simple (no structured data, no shared state). Consider `misaki` G2P as a near-term path to eliminating the dependency entirely (see Section 4.4). |
| **xcap X11 capture performance insufficient** for 60fps at low zoom | Medium | High | xcap uses `xcb_get_image` (non-SHM path), which requires a full X server round-trip per capture. At high zoom (small source region) this is fast; at low zoom (large region) it may exceed the frame budget. Mitigation: implement a direct `x11rb`-based capture backend with XShm support as a Phase 1 task. OBS achieves 60fps+ X11 capture via XShm. |
| **macOS docked mode cannot reserve screen space** | Certain | Medium | Accept floating overlay behavior for P0. Document as known limitation. Investigate `CGSSetWorkspace` private API (used by some tiling WMs) as a future option. |
| **Accessibility API coverage gaps** prevent keyboard focus tracking | Medium | Medium | Many apps (legacy Win32, games, PDF viewers, some Electron apps) expose minimal accessibility trees. OCR-based text detection is the long-term fallback. For P0, mouse-follow mode is the reliable default. |
| **sherpa-rs/sherpa-onnx Rust bindings immature** | Medium | Low | sherpa-onnx has a stable C API. If sherpa-rs bindings have gaps, write thin Rust FFI wrappers directly against the C API. Alternatively, use `kokoroxide` crate for Kokoro-specific workloads, or run sherpa-onnx as a subprocess. |
| **wgpu Vulkan driver issues on older Linux hardware** | Low | Medium | wgpu falls back to OpenGL (via `Backends::GL`). Vulkan support is nearly universal on Linux systems shipped in the last 8 years. Test on Intel integrated graphics specifically. |
| **Rust compile times slow AI development iteration** | Certain | Medium | Use `cargo check` for fast type-checking (2-5s). Use incremental compilation. Structure code as a Cargo workspace with small crates to minimize recompilation scope. Pre-built dependencies via `sccache`. |

### 6.2 Trade-offs Explicitly Accepted

1. **xcap captures to CPU buffer, not GPU texture.** This adds a CPU→GPU copy per frame. At the source region sizes involved in magnification (small at high zoom), this is negligible. A zero-copy GPU capture path can be pursued as a Phase 1+ optimization for Windows (DXGI texture sharing) and macOS (IOSurface import).

2. **macOS cannot reserve screen space like Windows/Linux.** The docked mode on macOS will float on top of other windows rather than preventing overlap. This is a macOS platform limitation, not an application limitation.

3. **espeak-ng subprocess adds ~10-50ms latency to TTS.** Process spawning and IPC add overhead compared to in-process linking. For the "read aloud" use case (not real-time conversation), this is acceptable. The subprocess can be kept warm (long-running process) to amortize spawn cost. This trade-off may be eliminated entirely if `misaki` (Kokoro's transformer-based G2P, already released and functional for English/Japanese/Korean/Chinese) proves accurate enough to replace espeak-ng for Luminos's supported languages.

4. **Kokoro supports fewer languages than Piper.** Kokoro v1.0 supports 10 language codes (American English, British English, Spanish, French, Hindi, Italian, Japanese, Korean, Brazilian Portuguese, Mandarin Chinese) — roughly 8 unique languages depending on how dialects are counted. Piper supports 30+. For languages not covered by Kokoro, Piper VITS models (also available via sherpa-onnx) serve as a fallback with the same subprocess isolation architecture.

---

## 7. Conclusion

The Luminos product strategy's core technical architecture is validated: Rust + Tauri 2.0 + wgpu for GPU rendering is the right foundation for a cross-platform, high-performance screen magnification tool developed primarily by AI agents. The dual-window architecture (Tauri WebView for settings, native winit+wgpu window for magnification) is essential and correct.

Three material changes are required to align the stack with P0 requirements:

1. **Replace `scap` with `xcap`** for screen capture. The `scap` crate's PipeWire-based Linux support does not meet the X11 requirement. `xcap` is more mature, has direct X11 support, and is already integrated in the Tauri ecosystem.

2. **Replace Piper with Kokoro via sherpa-onnx** for TTS. Piper is archived and GPL-licensed. Kokoro is higher quality, Apache-2.0 licensed (model weights), and actively maintained. sherpa-onnx provides a unified runtime that supports both Kokoro and Piper models, with Rust bindings via `sherpa-rs`.

3. **Explicitly adopt `winit`** as the window management layer for the magnification overlay, and implement platform-specific docked mode via EWMH struts (X11), AppBar API (Windows), and floating NSPanel (macOS).

With these changes, the technology stack is ready for Phase 0 development: proving the capture → GPU upload → shader magnification → overlay rendering pipeline on a single platform (macOS), then extending to Windows and Linux X11 in Phase 1.

---

## 8. References

### Crate Repositories and Documentation
1. xcap — https://github.com/nashaofu/xcap (v0.9.1, Apache 2.0)
2. scap — https://github.com/CapSoftware/scap (v0.1.0-beta.1, MIT)
3. wgpu — https://github.com/gfx-rs/wgpu (v28.0.0, MIT/Apache 2.0)
4. winit — https://github.com/rust-windowing/winit (v0.30.13, Apache 2.0)
5. Tauri — https://github.com/tauri-apps/tauri (v2.x, MIT/Apache 2.0)
6. sherpa-onnx — https://github.com/k2-fsa/sherpa-onnx (Apache 2.0, 10.8K stars)
7. sherpa-rs — https://crates.io/crates/sherpa-rs (v0.6.8, MIT)
8. kokoroxide — https://lib.rs/crates/kokoroxide (v0.1.5, MIT/Apache 2.0)
9. atspi — https://github.com/odilia-app/atspi (MIT/Apache 2.0)
10. windows-capture — https://github.com/NiiightmareXD/windows-capture (MIT)
11. cpal — https://github.com/RustAudio/cpal (Apache 2.0)
12. arboard — https://github.com/1Password/arboard (MIT/Apache 2.0)

### TTS Models and Engines
13. Kokoro TTS model — https://huggingface.co/onnx-community/Kokoro-82M-v1.0-ONNX (Apache 2.0; use q8 or q4 quantized variant for deployment)
13b. misaki G2P — https://github.com/hexgrad/misaki (transformer-based phonemizer for Kokoro; available on PyPI)
14. Piper TTS (archived) — https://github.com/rhasspy/piper (archived Oct 2025)
15. Piper GPL fork — https://github.com/OHF-Voice/piper1-gpl (GPL-3.0)
16. espeak-ng — https://github.com/espeak-ng/espeak-ng (GPL-3.0)
17. Kokoro espeak-ng GPL discussion — https://github.com/hexgrad/kokoro/issues/247
18. espeak-ng license discussion — https://github.com/espeak-ng/espeak-ng/issues/2131
19. Supertonic TTS — https://github.com/supertone-inc/supertonic

### Platform APIs
20. Windows AppBar API — https://learn.microsoft.com/en-us/windows/win32/shell/application-desktop-toolbars
21. EWMH _NET_WM_STRUT — https://specifications.freedesktop.org/wm-spec/latest/
22. AT-SPI2 — https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/
23. macOS AXUIElement — Apple Developer Documentation
24. Windows UI Automation — https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-entry-overview
25. DXGI Desktop Duplication — https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api

### Benchmarks and Comparisons
26. Tauri vs Electron Benchmark — https://www.reddit.com/r/programming/comments/1jwjw7b/
27. screenpipe architecture — https://github.com/screenpipe/screenpipe (17K+ stars)
28. wgpu GPGPU guide — https://dev.to/jaysmito101/high-performance-gpgpu-with-rust-and-wgpu-4l9i
29. TTS quality comparison — https://portalzine.de/text-to-speech-solutions-ranked-by-speech-quality/
30. sherpa-onnx TTS benchmarks — https://k2-fsa.github.io/sherpa/onnx/tts/pretrained_models/

---

## Appendix A: Alternative Options Considered

### A.1 Screen Capture: `scap` (Discarded)

**Description:** Unified cross-platform screen capture crate by CapSoftware, wrapping ScreenCaptureKit (macOS), WGC (Windows), and PipeWire (Linux).

**Reason for discard:** Linux implementation uses PipeWire, which is a Wayland-era technology. PipeWire can work on X11 sessions, but requires XDG Desktop Portal infrastructure that is not reliably present on X11-only configurations, especially KDE on X11. The crate is also in beta (v0.1.0-beta.1), with a failed docs.rs build on the published version and only 1 downstream dependent.

**Supporting data:** The `xcap` crate (v0.9.1) provides direct X11 capture via XCB with 19 downstream dependents and 85K monthly downloads, representing a materially lower risk choice.

### A.2 TTS Engine: Piper (Discarded as Primary)

**Description:** Neural TTS engine using VITS architecture, originally MIT-licensed, now GPL-3.0 due to espeak-ng dependency.

**Reason for discard:** Archived on October 6, 2025. Moved to `OHF-Voice/piper1-gpl` with explicit GPL-3.0 licensing. Kokoro-82M provides higher speech quality at comparable performance and is Apache 2.0 licensed (model weights). Piper VITS models remain available as a fallback through sherpa-onnx for languages not covered by Kokoro.

**Supporting data:** Reddit/HN community consensus consistently rates Kokoro above Piper for speech naturalness. Piper's advantage is in language breadth (30+ languages) and extreme lightweight operation (runs on Raspberry Pi).

### A.3 Screen Capture: Direct Platform APIs (Considered but Deferred)

**Description:** Use platform-specific crates directly (screencapturekit-rs, windows-capture, x11rb) instead of xcap.

**Reason for deferral:** This approach provides maximum performance and control but triples the API surface area to maintain. xcap provides a reasonable abstraction for P0 while `windows-capture` with DXGI DD is recommended as a Windows performance fallback. Direct platform APIs are the correct approach for Phase 1+ optimization.

### A.4 GPU Rendering: Skia (via skia-safe) (Discarded)

**Description:** Google's 2D rendering engine used by Chrome and Flutter, with Rust bindings via the `skia-safe` crate.

**Reason for discard:** Skia is a 2D rendering library, not a direct GPU API. For the magnification pipeline (texture sampling with custom shaders), wgpu provides more direct control and better performance. Skia's strengths (text rendering, path drawing, SVG) are not needed for screen magnification. Additionally, Skia's build system is notoriously complex (requires ~8GB build from source or pre-built binaries), which complicates CI/CD and AI-agent development workflows.

### A.5 Application Framework: Pure Native Rust (No Tauri) (Considered but Discarded)

**Description:** Build both the control panel UI and magnification overlay in pure Rust using a native GUI framework (iced, egui, GPUI).

**Reason for discard:** Native Rust GUI frameworks lack the UI component richness needed for a settings panel (dropdowns, sliders, color pickers, tabbed forms). React provides a vastly larger component ecosystem and is far more productive for AI agents generating UI code. Since the performance-critical magnification overlay already uses winit+wgpu directly, Tauri's WebView overhead applies only to the non-critical settings panel. The AI-development productivity gain from TypeScript/React outweighs the minimal overhead.

### A.6 TTS Engine: Platform-Native Only (Considered but Discarded)

**Description:** Use only AVSpeechSynthesizer (macOS), SAPI (Windows), and speech-dispatcher (Linux) without any bundled TTS engine.

**Reason for discard:** Platform-native TTS engines provide inconsistent quality across platforms and offer no offline voice consistency. speech-dispatcher on Linux defaults to espeak-ng (the same GPL dependency). The cross-platform value proposition requires a consistent, high-quality voice that sounds the same on all three platforms. Platform-native TTS remains the recommended **fallback** for languages not supported by Kokoro.

### A.7 Window Management: tao (Tauri's fork of winit) (Considered)

**Description:** Tauri's maintained fork of winit with additional features (system tray, custom menus).

**Reason for deferral:** tao is tightly coupled to Tauri's internals and may have different API stability guarantees than mainline winit. Since the magnification overlay is independent of Tauri, using mainline winit (v0.30.13, 34.3M total downloads) provides a cleaner dependency graph and broader community support. If tao's additional features become needed (e.g., system tray for the overlay), this decision can be revisited.

---

## Appendix B: Glossary

| Term | Definition |
|------|-----------|
| AppBar API | Windows API for creating application-docked windows that reserve screen space |
| AT-SPI2 | Assistive Technology Service Provider Interface 2; Linux accessibility protocol over D-Bus |
| DXGI DD | DirectX Graphics Infrastructure Desktop Duplication; high-performance Windows screen capture API |
| EWMH | Extended Window Manager Hints; freedesktop.org specification for X11 window manager behavior |
| G2P | Grapheme-to-Phoneme; converting written text to pronunciation symbols |
| MSAA (graphics) | Multisample Anti-Aliasing; GPU technique for reducing jagged edges |
| RTF | Real-Time Factor; ratio of synthesis time to audio duration (< 1.0 = faster than real-time) |
| WGC | Windows.Graphics.Capture; modern Windows screen capture API |
| WGSL | WebGPU Shading Language; shader language used by wgpu |
| XCB | X C Bindings; modern C library for X11 protocol communication |
| XShm | X Shared Memory Extension; high-performance X11 image capture via shared memory |
