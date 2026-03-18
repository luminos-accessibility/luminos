# 01 -- System Architecture

**Status:** DRAFT v1.0
**Date:** 2026-03-15
**Audience:** Everyone (engineers, product managers, contributors, AI agents)
**Source Documents:** [Product Strategy](../PRODUCT_STRATEGY.md) (v1.3, Sections 5, 7, 8), [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) (FINAL, Sections 3-4)

---

## 1. Overview

### 1.1 Purpose

This document defines the system architecture for Luminos: the overall structure, the major components, how they connect, and how data flows through the system. It is the architectural foundation that all subsequent technical strategy documents build upon.

This document answers: **How is Luminos structured, and why?**

Individual subsystems are covered in depth by their own documents:
- How platform-specific code is organized: [02 -- Platform Abstraction](./02-platform-abstraction.md)
- How screen capture becomes magnified pixels: [03 -- Rendering Pipeline](./03-rendering-pipeline.md)
- How text becomes speech: [04 -- TTS Pipeline](./04-tts-pipeline.md)
- How the settings UI talks to the engine: [05 -- Control Panel](./05-control-panel.md)

### 1.2 The Architectural Challenge

Luminos must deliver GPU-accelerated screen magnification at 60fps with integrated neural text-to-speech across four platforms (Linux, macOS, OpenBSD, Windows) -- with Linux requiring two distinct display server backends (X11 and Wayland) -- on hardware as modest as integrated GPUs with 4GB total system RAM. It must coexist with existing screen readers (NVDA and JAWS on Windows, Orca on Linux), ship under the GPLv3 license, and be buildable primarily by AI agents with the Rust compiler as automated reviewer.

The core tension is between **performance** (magnification must be imperceptibly smooth at 60fps) and **portability** (every platform has fundamentally different screen capture, window management, and accessibility APIs). The architecture resolves this tension through a dual-window design that isolates the performance-critical rendering path from all web rendering overhead, and a trait-based platform abstraction layer that confines platform-specific code behind compiler-enforced contracts.

### 1.3 Scope

This document covers:
- The dual-window design and its rationale
- The component model and major subsystems
- Data flow through the magnification and TTS pipelines
- Process and thread model
- Module organization (Cargo workspace)
- Technology stack summary with decision rationale
- Performance and security architecture constraints

This document does NOT cover:
- Trait definitions and per-platform backend implementations (see [02](./02-platform-abstraction.md))
- GPU shader details, frame pacing, or anti-aliasing strategies (see [03](./03-rendering-pipeline.md))
- espeak-ng subprocess protocol, voice model management, or audio mixing (see [04](./04-tts-pipeline.md))
- Tauri IPC command definitions or React component architecture (see [05](./05-control-panel.md))

---

## 2. Architecture Principles

These principles govern all architectural decisions in Luminos. They are listed in priority order -- when principles conflict, higher-priority principles win.

### 2.1 Performance Is Accessibility

For a screen magnification tool, performance is not a nice-to-have quality attribute -- it is a core accessibility requirement. A magnified view that stutters, lags, or drops frames is an inaccessible magnified view. The user's entire visual interaction with their computer passes through Luminos; any latency is directly experienced as impairment.

**Architectural implication:** The magnification rendering path must never share resources with, wait on, or be blocked by any non-rendering subsystem. TTS inference, settings UI rendering, file I/O, and IPC cannot interrupt the frame production loop.

### 2.2 Separate by Performance Criticality

The application has two fundamentally different performance profiles:

| Concern | Latency Requirement | Technology Fit |
|---------|---------------------|----------------|
| Magnification overlay | 16ms per frame (60fps) | Native GPU rendering (wgpu), no web layer |
| Settings / control panel | Human interaction speed (~100ms) | Web UI (React), rich controls, rapid iteration |

These two concerns must not share a rendering pipeline. A web rendering engine (WebkitGTK, WebView2) introduces GC pauses, layout reflows, and compositor overhead that are acceptable for a settings panel but catastrophic for a real-time magnification overlay.

**Architectural implication:** Two separate windows, two separate rendering stacks, connected by a shared Rust core.

### 2.3 Platform Abstraction via Compiler-Enforced Contracts

Every platform (Linux X11, Linux Wayland, macOS, OpenBSD, Windows) has different APIs for screen capture, window management, focus tracking, and accessibility. Rather than scattering `#[cfg]` conditionals throughout the codebase, the architecture defines six Rust traits that encode what every platform must provide. The core engine programs exclusively against these traits. The Rust compiler ensures that every platform backend fully implements every required behavior.

**Architectural implication:** Platform-specific code lives behind trait boundaries. The core engine never imports platform modules directly. See [02 -- Platform Abstraction](./02-platform-abstraction.md) for trait definitions and backend implementations.

### 2.4 Privacy by Design

All processing happens on-device. No screen content, text, or usage data leaves the user's machine. There is no telemetry by default. Neural TTS inference, OCR, and future AI features all run locally. This is a hard architectural constraint, not a policy preference -- the system has no network communication layer for user data.

**Architectural implication:** No cloud API clients, no analytics SDKs, no network dependencies for core functionality.

### 2.5 AI-Agent Development Optimization

The codebase is designed to be built and maintained primarily by AI coding agents. This influences technology choices and code organization:

- **Rust's strict compiler** catches type errors, memory errors, and concurrency bugs in AI-generated code at compile time, serving as an automated reviewer.
- **TypeScript + React** for the UI has the largest LLM training corpus of any typed frontend stack, maximizing generation quality.
- **Trait-based contracts** define precise interfaces that AI agents can implement independently per platform, without needing to understand other platforms.
- **Small, focused crates** in a Cargo workspace limit the blast radius of AI-generated changes and reduce recompilation scope.

**Architectural implication:** Clear module boundaries, exhaustive type definitions, and trait contracts that serve as implementation specifications for AI agents.

---

## 3. Dual-Window Design

The dual-window architecture is the single most important architectural decision in Luminos. It separates the performance-critical magnification rendering from the settings UI, allowing each to use the optimal technology for its requirements.

### 3.1 Why Two Windows

A screen magnification tool could, in theory, render the magnified view inside a web-based application framework like Electron or Tauri. This approach fails for three reasons:

1. **Frame budget.** A webview introduces 2-8ms of overhead per frame (compositor, layout, paint) before any application logic runs. On a 16.67ms frame budget, this consumes 12-48% of available time before a single magnified pixel is produced.

2. **Garbage collection pauses.** JavaScript GC pauses of 5-50ms are normal in webview-based applications. A single 50ms pause causes 3 dropped frames at 60fps -- visible as a stutter in the magnified view.

3. **Linux WebkitGTK concerns.** Tauri on Linux uses WebkitGTK, which has known performance and rendering issues. By confining WebkitGTK to the settings panel (where it handles simple forms), these concerns become irrelevant to the user's primary interaction.

The dual-window design eliminates all three problems: the magnification overlay is a native Rust window that talks directly to the GPU, with zero web rendering in its path.

### 3.2 Window 1: Magnification Overlay

The magnification overlay is the user's primary interaction surface. It captures screen content, applies GPU-accelerated magnification transforms, and renders the result as a transparent, always-on-top window.

**Technology:** `winit` (window creation) + `wgpu` (GPU rendering)

**Characteristics:**
- Transparent, borderless, always-on-top
- Renders at 60fps via GPU shaders
- Click-through in magnifying glass mode (user interacts with content beneath)
- In docked mode, reserves screen space so other windows do not overlap (via EWMH struts on X11, AppBar API on Windows)
- No web rendering engine involved -- pure Rust + GPU

**Modes:**

| Mode | Description | Window Behavior |
|------|-------------|-----------------|
| Full-screen | Entire screen is magnified | Overlay covers full display, magnified view follows cursor |
| Docked | Magnified view docked to screen edge | Overlay reserves a portion of the screen (top/bottom/left/right) |
| Lens | Magnifying glass follows cursor | Overlay is a movable, resizable rectangle/ellipse near cursor |

### 3.3 Window 2: Control Panel

The control panel provides the settings UI: zoom level, magnification mode, color filters, TTS voice selection, keybinding configuration, and profile management.

**Technology:** Tauri 2.0 (application framework) + React + TypeScript (UI)

**Characteristics:**
- Standard desktop window with native decorations
- Not performance-critical (human interaction speed)
- Rich UI controls: sliders, dropdowns, color pickers, toggle switches
- Communicates with the Rust core engine via Tauri's typed IPC
- Can be minimized to system tray during normal magnification use
- Must be fully accessible (keyboard navigable, screen reader compatible)

**Tauri's role is deliberately scoped:**
- Settings UI rendering (React/TypeScript webview)
- IPC between frontend and Rust backend (typed commands and events)
- Application lifecycle management (startup, shutdown, auto-update)
- System tray integration
- Packaging and distribution

Tauri does NOT manage the magnification overlay window. That is created and managed by `winit` directly.

### 3.4 How the Two Windows Connect

Both windows are created within a single OS process. They share a Rust core engine that owns all application state. The magnification overlay and control panel are two views into the same engine state.

```
+-------------------------------------------------------------------+
|                         Single OS Process                          |
|                                                                    |
|  +------------------------------+  +---------------------------+  |
|  |   Magnification Overlay      |  |     Control Panel         |  |
|  |   (winit + wgpu)             |  |     (Tauri + React)       |  |
|  |                              |  |                           |  |
|  |   Reads:                     |  |   Reads:                  |  |
|  |   - Zoom level               |  |   - Current settings      |  |
|  |   - Magnification mode       |  |   - TTS status            |  |
|  |   - Color filter config      |  |   - Available voices      |  |
|  |   - Cursor position          |  |                           |  |
|  |   - Focus position           |  |   Writes:                 |  |
|  |                              |  |   - Zoom level changes    |  |
|  |   Writes:                    |  |   - Mode changes          |  |
|  |   - Nothing (pure consumer)  |  |   - TTS config changes    |  |
|  +-------------+----------------+  +------------+--------------+  |
|                |                                |                  |
|                v                                v                  |
|  +-------------------------------------------------------------+  |
|  |                    Rust Core Engine                           |  |
|  |                                                               |  |
|  |  +-------------------+  +-------------------+                 |  |
|  |  | Config Store      |  | App State         |                 |  |
|  |  | (settings, prefs) |  | (zoom, mode, etc) |                 |  |
|  |  +-------------------+  +-------------------+                 |  |
|  |                                                               |  |
|  |  +-------------------+  +-------------------+                 |  |
|  |  | Magnification     |  | TTS Pipeline      |                 |  |
|  |  | Pipeline          |  |                   |                 |  |
|  |  +-------------------+  +-------------------+                 |  |
|  |                                                               |  |
|  |  +---------------------------------------------------+       |  |
|  |  | Platform Abstraction Layer (6 traits)               |       |  |
|  |  | ScreenCapture | FocusTracker | TtsEngine            |       |  |
|  |  | WindowManager | InputMonitor | AudioOutput           |       |  |
|  |  +---------------------------------------------------+       |  |
|  +-------------------------------------------------------------+  |
|                                                                    |
+-------------------------------------------------------------------+
```

**Communication patterns:**

| Direction | Mechanism | Example |
|-----------|-----------|---------|
| Control Panel --> Core Engine | Tauri IPC command (typed) | User changes zoom level from 5x to 10x |
| Core Engine --> Control Panel | Tauri event emission | TTS finishes speaking, status updates to "idle" |
| Core Engine --> Overlay | Shared state (atomic/mutex) | Zoom level change immediately affects next rendered frame |
| Overlay --> Core Engine | winit event callback | Cursor position change triggers viewport recalculation |
| Input devices --> Core Engine | InputMonitor trait | Global hotkey (Ctrl+= to zoom in) captured even when Luminos unfocused |

---

## 4. Component Architecture

### 4.1 High-Level Component Diagram

```
+------------------------------------------------------------------+
|                          Luminos Application                       |
|                                                                    |
|  +---------+  +---------+  +---------+  +---------+  +---------+ |
|  | Screen  |  | Focus   |  | Input   |  | Window  |  | Audio   | |
|  | Capture |  | Tracker |  | Monitor |  | Manager |  | Output  | |
|  +---------+  +---------+  +---------+  +---------+  +---------+ |
|       |            |            |            |             ^       |
|       |  Platform Abstraction Layer (Rust traits)          |       |
|  -----+------------+------------+------------+-------------+----  |
|       |            |            |            |             |       |
|       v            v            v            |             |       |
|  +-----------------------------------------------+        |       |
|  |            Magnification Pipeline              |        |       |
|  |  capture --> GPU upload --> shader --> present  |        |       |
|  +-----------------------------------------------+        |       |
|       |                                                    |       |
|       |  +--------------------------------------------+    |       |
|       |  |              TTS Pipeline                   |   |       |
|       |  |  text --> espeak-ng --> Kokoro --> audio     |---+       |
|       |  +--------------------------------------------+            |
|       |       ^                                                    |
|       |       |  +-----------------------------+                   |
|       |       +--| Configuration Manager       |                   |
|       |          | (settings, profiles, state)  |                   |
|       |          +-----------------------------+                   |
|       |                    ^                                       |
|       |                    |  Tauri IPC                             |
|       |          +-----------------------------+                   |
|       |          | Control Panel (React/TS)     |                   |
|       |          +-----------------------------+                   |
|       |                                                            |
|  +-----------------------------------------------+                |
|  |          External Processes                     |               |
|  |  +------------------+  +--------------------+   |               |
|  |  | espeak-ng        |  | Voice Model Files  |   |               |
|  |  | (subprocess)     |  | (Kokoro, Piper)    |   |               |
|  |  +------------------+  +--------------------+   |               |
|  +-----------------------------------------------+                |
+------------------------------------------------------------------+
```

### 4.2 Core Engine

The Rust core engine is the application's brain. It owns all state, coordinates all subsystems, and implements all logic that is not platform-specific or UI-specific. Every interaction -- whether from the magnification overlay, the control panel, or an external input device -- is processed by the core engine.

**Responsibilities:**
- Owns the magnification state (zoom level, mode, viewport position, color filters)
- Drives the magnification pipeline (capture --> transform --> render loop)
- Drives the TTS pipeline (text extraction --> phonemization --> synthesis --> playback)
- Manages configuration persistence (settings load/save, profile management)
- Routes input events (hotkeys, mouse movements) to the appropriate handler
- Provides Tauri IPC command handlers for the control panel

**The core engine never calls platform APIs directly.** All platform interaction goes through the six abstraction traits. This means the core engine can be fully unit-tested with mock backends, without a display server, GPU, or audio device.

### 4.3 Platform Abstraction Layer

The platform abstraction layer defines six Rust traits that encode every platform-dependent behavior:

| Trait | Responsibility | Example Operation |
|-------|----------------|-------------------|
| `ScreenCapture` | Capture screen pixels | `capture_frame(display_id, region) -> CaptureFrame` |
| `FocusTracker` | Track keyboard focus position | `subscribe_focus_changes() -> Receiver<FocusEvent>` |
| `TtsEngine` | Convert text to speech | `speak(text, interrupt) -> Result<(), TtsError>` |
| `WindowManager` | Create and manage overlay windows | `set_overlay_bounds(rect)`, `set_dock_edge(edge, size)` |
| `InputMonitor` | Monitor global input events | `subscribe_input_events() -> Receiver<InputEvent>` |
| `AudioOutput` | Play audio to device | `create_output_stream(sample_rate) -> AudioStream` |

*Signatures above are simplified. Voice selection is handled separately via `TtsEngine::set_voice()`. Full canonical trait definitions are in [02 -- Platform Abstraction](./02-platform-abstraction.md).*

Each trait has per-platform backend implementations selected at compile time (or at runtime on Linux, where X11 and Wayland coexist). Full trait definitions and backend details are in [02 -- Platform Abstraction](./02-platform-abstraction.md).

### 4.4 Magnification Pipeline

The magnification pipeline is the performance-critical hot path. It runs at 60fps and must complete each frame within 16.67ms on integrated GPUs. The pipeline is a five-stage loop:

```
Every 16ms:
  1. CALCULATE source region from cursor/focus position + zoom level
  2. CAPTURE source region pixels (ScreenCapture trait)
  3. UPLOAD pixel buffer to GPU texture (wgpu::Queue::write_texture)
  4. RENDER magnified view via GPU shader (bilinear Phase 0, bicubic Phase 1+ interpolation)
  5. PRESENT to overlay window (wgpu swap chain, vsync)
```

At 20x magnification on a 1080p display, the source region is only 96x54 pixels -- both capture and upload are nearly instant. The performance bottleneck appears at low zoom levels (1.5-3x) on high-resolution displays, where the source region approaches full-screen size. See [03 -- Rendering Pipeline](./03-rendering-pipeline.md) for shader design, frame pacing, anti-aliasing, and zoom mode rendering.

### 4.5 TTS Pipeline

The TTS pipeline converts on-screen text into spoken audio through three stages:

```
text --> espeak-ng subprocess (phonemes) --> Kokoro ONNX inference (synthesis) --> cpal (audio output)
```

The TTS pipeline runs on dedicated threads, fully decoupled from the magnification render loop. A speech request never blocks a frame render. espeak-ng runs as a long-lived subprocess (kept warm to avoid repeated process spawn overhead) for crash isolation -- an espeak-ng crash or memory leak does not affect the main application.

TTS is a Phase 2 feature, but the architecture accommodates it from Phase 0: the `TtsEngine` and `AudioOutput` traits exist as stubs during Phase 0 to validate the trait boundary. See [04 -- TTS Pipeline](./04-tts-pipeline.md) for the full pipeline design, latency budget, voice model management, and concurrency model.

### 4.6 Configuration Manager

The configuration manager handles persistent settings, user profiles, and runtime application state.

**Settings hierarchy:**

```
Defaults (compiled-in)
  |
  v
User config file (~/.config/luminos/config.toml or platform equivalent)
  |
  v
Profile overrides (condition-based: "AMD profile", "Glaucoma profile")
  |
  v
Runtime overrides (e.g., zoom level changed via hotkey since last save)
```

**Responsibilities:**
- Load configuration at startup, applying the cascade above
- Persist setting changes when the user explicitly saves or when the app exits
- Provide atomic access to configuration values from both the render thread and IPC thread
- Support configuration import/export (JSON format, Git-friendly for institutional deployment)
- Support per-application profiles (Phase 4: different zoom levels for different apps)

**Configuration access pattern:** Runtime application state is stored in `Arc<ArcSwap<AppState>>`, providing lock-free reads from the render thread. The render thread reads state values (zoom level, mode, color filter) every frame; these reads must never block. `AppState` is the runtime shared state (a superset of the persisted `AppSettings` schema), and `ArcSwap` allows the IPC thread and hotkey handlers to update it via `rcu()` without contending with the render thread's reads. See [05 -- Control Panel](./05-control-panel.md) for the `LuminosHandle` struct that owns this state.

### 4.7 IPC Layer (Tauri Commands)

The IPC layer connects the Control Panel (TypeScript/React) to the Rust core engine via Tauri's typed command system. Each IPC command maps to a Rust function with typed parameters and return values. Tauri generates TypeScript bindings automatically.

**Design rules:**
- Commands are request/response (control panel calls, engine responds)
- Events are push notifications (engine emits, control panel subscribes)
- Commands must never block the render thread (they run on Tauri's async runtime)
- All command parameters and return values are serializable (serde)

**Example commands (illustrative, not exhaustive):**

| Command | Direction | Purpose |
|---------|-----------|---------|
| `set_zoom_level(level: f32)` | Panel --> Engine | Change magnification level |
| `set_magnification_mode(mode: MagnificationMode)` | Panel --> Engine | Switch between full/docked/lens |
| `get_current_settings() -> AppSettings` | Panel --> Engine | Load current settings for display |
| `speak_text(text: String)` | Panel --> Engine | Trigger TTS for selected text |
| `list_voices() -> Vec<VoiceInfo>` | Panel --> Engine | Get available TTS voices |
| `on_settings_changed` | Engine --> Panel | Notify panel when settings change (e.g., via hotkey) |
| `on_tts_status_changed` | Engine --> Panel | Notify panel of TTS state (speaking/idle/error) |

Full IPC design is in [05 -- Control Panel](./05-control-panel.md).

---

## 5. Data Flow

### 5.1 Magnification Frame Cycle

This is the primary data flow, executing 60 times per second:

```
Input Events                  Platform State
(mouse move, focus change)    (cursor position, focus bounds)
         |                              |
         v                              v
+--------------------------------------------------+
| Viewport Calculator                               |
| - source_width  = viewport_width / zoom_level     |
| - source_height = viewport_height / zoom_level    |
| - source_origin = tracking_target - source_size/2 |
| - clamp to screen bounds                          |
+--------------------------------------------------+
                    |
                    v  ScreenRect (source region)
+--------------------------------------------------+
| ScreenCapture::capture_frame(display, region)     |
| Returns: CaptureFrame { data: Arc<[u8]>,          |
|          width, height, stride, format }          |
+--------------------------------------------------+
                    |
                    v  BGRA pixel buffer (CPU)
+--------------------------------------------------+
| GPU Texture Upload                                |
| wgpu::Queue::write_texture(gpu_texture, pixels)   |
+--------------------------------------------------+
                    |
                    v  GPU texture
+--------------------------------------------------+
| Fragment Shader                                   |
| - Sample source texture with zoom transform       |
| - Bicubic interpolation for smooth scaling         |
| - Gamma-correct resampling                        |
| - Color filter application (inversion, contrast)  |
| - Cursor overlay rendering                        |
+--------------------------------------------------+
                    |
                    v  Rendered frame
+--------------------------------------------------+
| Present to Overlay Window                          |
| wgpu swap chain (PresentMode::Fifo for vsync)     |
+--------------------------------------------------+
                    |
                    v
              User's display
```

**Performance budget (16.67ms total):**

| Step | Typical Time | Notes |
|------|-------------|-------|
| Viewport calculation | <0.01ms | Pure math |
| Screen capture | 1-8ms | Depends on region size and platform |
| GPU texture upload | 0.5-2ms | Proportional to region size |
| Shader execution | <1ms | Trivial for modern GPUs |
| Present (vsync wait) | 0-16ms | Fills remaining time in Fifo mode |
| **Total (excluding vsync)** | **2-8ms** | Well within budget |

### 5.2 Input Event Flow

Input events (mouse movement, keyboard presses, focus changes) flow from platform-specific sources through the abstraction layer to the core engine, which updates the magnification viewport:

```
+-------------------+     +-------------------+     +-------------------+
| Physical Input    |     | OS Event System   |     | InputMonitor      |
| (mouse, keyboard) | --> | (X11, Cocoa, Win) | --> | trait impl        |
+-------------------+     +-------------------+     +-------------------+
                                                             |
                                                             v
                                                    InputEvent channel
                                                             |
                          +---------------------------------+
                          |                                 |
                          v                                 v
                 +------------------+             +------------------+
                 | Hotkey Handler   |             | Tracking Engine  |
                 | (zoom in/out,    |             | (update viewport |
                 |  toggle mode)    |             |  follow target)  |
                 +------------------+             +------------------+
                          |                                 |
                          v                                 v
                 +--------------------------------------------------+
                 | App State (zoom, mode, viewport position)         |
                 | Read by render loop on next frame                 |
                 +--------------------------------------------------+
```

**Focus tracking** (via `FocusTracker` trait) provides an additional input source: when keyboard focus moves to a new UI element, the magnification viewport smoothly pans to center that element. This is essential for keyboard-only users who cannot use a mouse to guide the viewport.

```
+-------------------+     +-------------------+
| Accessibility API |     | FocusTracker      |
| (AT-SPI2, AX, UIA)| --> | trait impl        |
+-------------------+     +-------------------+
                                   |
                                   v
                          FocusEvent channel
                          (element bounds)
                                   |
                                   v
                          +------------------+
                          | Tracking Engine  |
                          | (smooth pan to   |
                          |  focused element)|
                          +------------------+
```

### 5.3 TTS Data Flow

The TTS data flow is triggered by user action ("read what I see" or "read selection") and runs entirely on dedicated threads:

```
User Trigger ("Read this")
         |
         v
+------------------+
| Text Extraction  |
| - FocusTracker   |  (read what's under focus)
|   or             |
| - Clipboard      |  (read selected text via arboard)
|   or             |
| - OCR (Phase 3)  |  (read from image/screenshot)
+------------------+
         |
         v  UTF-8 text
+------------------+
| Text Preprocessor|
| - Sentence split |
| - Abbreviations  |
| - Number expand  |
+------------------+
         |
         v  Sentence chunks
+------------------+          +------------------+
| espeak-ng        | stdin -> | espeak-ng process|
| Subprocess Mgr   | stdout<- | (phonemization)  |
+------------------+          +------------------+
         |
         v  IPA phonemes
+------------------+
| sherpa-onnx      |
| (Kokoro-82M)     |
| Neural synthesis  |
+------------------+
         |
         v  Audio samples (f32, 24kHz)
+------------------+
| cpal AudioOutput |
| (ring buffer)    |
+------------------+
         |
         v
    Speaker / Headphones
```

Sentences are pipelined: while Kokoro synthesizes sentence N, espeak-ng phonemizes sentence N+1. This hides the phonemization latency for all sentences after the first. See [04 -- TTS Pipeline](./04-tts-pipeline.md) for the full latency budget and concurrency model.

### 5.4 Settings Data Flow

Settings changes flow from the control panel to the core engine and immediately affect both rendering and TTS behavior:

```
+---------------------+
| Control Panel UI    |
| (React component)   |
+---------------------+
         |
         v  Tauri IPC command (e.g., set_zoom_level(10.0))
+---------------------+
| Tauri IPC Handler   |
| (Rust async fn)     |
+---------------------+
         |
         v  Update shared state
+---------------------+
| Configuration       |
| Manager             |
| ArcSwap<AppState>   |
|                     |
+---------------------+
    |              |
    v              v
+--------+   +--------+
| Render |   | TTS    |
| Thread |   | Thread |
| (reads |   | (reads |
| config |   | voice  |
| each   |   | config |
| frame) |   | on req)|
+--------+   +--------+
```

Settings changes from hotkeys follow the same path but originate from the `InputMonitor` instead of Tauri IPC:

```
Global Hotkey (e.g., Ctrl+=) --> InputMonitor --> Hotkey Handler --> Configuration Manager
                                                                          |
                                                                          v
                                                               Tauri event emission
                                                               (on_settings_changed)
                                                                          |
                                                                          v
                                                               Control Panel UI
                                                               (updates display)
```

This ensures the control panel always reflects the current state, even when settings are changed via keyboard shortcuts while the panel is open.

---

## 6. Process and Thread Model

### 6.1 Single-Process Architecture

Luminos runs as a single OS process with multiple threads. Both the magnification overlay and the control panel webview exist within this process. The only separate process is the espeak-ng subprocess for phonemization (crash-isolated by design).

**Why single-process (for the Rust core):**
- Shared memory access between the render loop and configuration state (no serialization overhead)
- Simpler lifecycle management (one process to start, one to stop)
- Tauri 2.0's Core process hosts the Rust backend and manages webview child processes through a shared process boundary for IPC (the webview renderer itself runs in OS-managed child processes, but the Rust backend and webview management layer share a process)
- Lower resource overhead than multi-process alternatives

### 6.2 Thread Architecture

```
Main Thread (winit event loop)
  |
  +--- Render Thread              [produces magnified frames at 60fps]
  |       |
  |       +--- reads from: App State (zoom, mode, viewport)
  |       +--- reads from: HighlightEvent channel (word highlight, non-blocking)
  |       +--- calls: ScreenCapture::capture_frame (sync, per frame)
  |       +--- calls: wgpu Queue::write_texture, Queue::submit (GPU work)
  |
  +--- TTS Coordinator Thread     [manages speech lifecycle]
  |       |
  |       +--- espeak-ng Subprocess Reader  [reads phonemes from stdout pipe]
  |       |
  |       +--- Inference Thread             [sherpa-onnx Kokoro/Piper synthesis]
  |
  +--- Audio Thread (cpal callback)  [reads ring buffer, writes to audio device]
  |
  +--- Tauri IPC Thread(s)        [handles control panel commands]
  |
  +--- Focus Monitor Thread       [listens for accessibility API focus changes]
  |
  +--- Input Monitor Thread       [listens for global input events]
```

### 6.3 Thread Responsibilities and Constraints

| Thread | Priority | Can Block? | Constraint |
|--------|----------|------------|------------|
| **Main (winit)** | Normal | No | Drives the event loop; dispatches to other threads |
| **Render** | High | No (except vsync) | Must complete non-vsync work in <8ms; reads shared state via lock-free or RwLock reads |
| **TTS Coordinator** | Normal | Yes | Waits on phoneme results and manages inference scheduling; does not touch render state |
| **Audio (cpal)** | Real-time | No | Callback must fill buffer within deadline; reads from ring buffer only |
| **Tauri IPC** | Normal | Yes | Handles async IPC commands; writes to shared state (acquires write lock briefly) |
| **Focus Monitor** | Normal | Yes | Blocks on D-Bus/accessibility API events; writes focus position to atomic or channel |
| **Input Monitor** | Normal | Yes | Blocks on input events; dispatches to hotkey handler or tracking engine |

### 6.4 Inter-Thread Communication

Threads communicate through bounded channels and shared atomic/lock-protected state. The design avoids unbounded channels to prevent memory growth under load.

**Channel-based communication** (for event streams):

| Channel | Sender | Receiver | Type | Capacity | Back-pressure |
|---------|--------|----------|------|----------|----------------|
| `input_events` | Input Monitor | Main thread | `InputEvent` | 32 | Drop oldest |
| `focus_events` | Focus Monitor | Main thread | `FocusEvent` | 4 | Drop oldest |
| `speech_request` | Main / IPC | TTS Coordinator | `SpeechRequest` | 1 | Replace (newest wins) |
| `highlight_events` | TTS pipeline | Render thread | `HighlightEvent` | 8 | Drop oldest |

**Shared state** (for continuously-read values):

| State | Writer(s) | Reader(s) | Mechanism | Why |
|-------|-----------|-----------|-----------|-----|
| Zoom level, mode, filters | IPC thread, hotkey handler | Render thread (every frame) | `ArcSwap<AppState>` | Lock-free reads on hot path |
| Viewport position | Tracking engine | Render thread (every frame) | Atomic (x, y as AtomicI32) | Updated from multiple sources |
| TTS status | TTS Coordinator | IPC thread (on query) | `Arc<Mutex<TtsStatus>>` | Infrequently read |

### 6.5 Event Loop Integration

The `winit` event loop is the application's primary event loop, running on the main thread. It processes window events (resize, close, focus) and dispatches custom events. The render thread is driven by `winit`'s `RedrawRequested` event or by a timer at the target frame rate.

```rust
// Simplified event loop structure (illustrative)
event_loop.run(move |event, target| {
    match event {
        Event::WindowEvent { event, window_id } => {
            if window_id == overlay_window.id() {
                handle_overlay_event(event); // resize, close, etc.
            }
        }
        Event::DeviceEvent { event, .. } => {
            handle_input_event(event); // mouse motion, key press
        }
        Event::UserEvent(custom) => {
            handle_custom_event(custom); // settings change, TTS trigger
        }
        Event::AboutToWait => {
            overlay_window.request_redraw(); // trigger next frame
        }
        _ => {}
    }
});
```

**Tauri integration:** Tauri 2.0 manages its own webview event loop internally. The Tauri application and winit overlay coexist in the same process. Coordination between the two event loops uses `winit::event_loop::EventLoopProxy` to send custom events from the Tauri IPC thread to the winit event loop. This pattern is documented and supported by both Tauri and winit.

---

## 7. Module Organization

### 7.1 Cargo Workspace Structure

The project uses a Cargo workspace to organize code into focused crates with clear dependency boundaries. This structure optimizes for:
- **Fast incremental compilation** (changing one crate does not recompile others)
- **Clear dependency direction** (core depends on traits, not on platform backends)
- **Independent implementation** (AI agents can work on one crate without understanding others)
- **Testability** (each crate has its own test suite)

```
luminos/
  Cargo.toml                    # Workspace root
  crates/
    luminos-core/               # Core engine: magnification pipeline, TTS coordination,
      src/                      #   configuration management. Platform-independent.
        lib.rs
        engine/
          magnification.rs      # Magnification pipeline orchestration
          tts_pipeline.rs       # TTS pipeline orchestration
          tracking.rs           # Viewport tracking (cursor, focus, smooth pan)
        config/
          mod.rs                # Configuration loading, persistence, profiles
          schema.rs             # Config schema (serde types)
        state.rs                # Shared application state (AppState, thread-safe)
        error.rs                # Top-level LuminosError enum

    luminos-platform/           # Platform abstraction: trait definitions + backends
      src/
        lib.rs
        traits.rs               # Six trait definitions
        error.rs                # Platform-specific error types
        common/                 # Shared utilities (X11 helpers for Linux + OpenBSD)
        linux_x11/              # Linux X11 backend
        linux_wayland/          # Linux Wayland backend
        macos/                  # macOS backend
        openbsd/                # OpenBSD backend (shares X11 code)
        windows/                # Windows backend
        mock/                   # Mock implementations for testing

    luminos-gpu/                # GPU rendering: wgpu setup, shaders, texture management
      src/
        lib.rs
        renderer.rs             # wgpu renderer (surface creation, render pass)
        shaders/                # WGSL shader sources
          magnify.wgsl          # Bicubic zoom shader
          color_filter.wgsl     # Color transformation shader
          cursor.wgsl           # Cursor overlay shader
        texture.rs              # GPU texture upload and management
        pipeline.rs             # Render pipeline configuration

    luminos-tts/                # TTS: espeak-ng subprocess, sherpa-onnx integration
      src/
        lib.rs
        espeak.rs               # espeak-ng subprocess management
        inference.rs            # sherpa-onnx / Kokoro inference
        models.rs               # Voice model discovery and loading
        preprocessor.rs         # Text preprocessing (sentence split, normalization)

    luminos-app/                # Application entry point: winit event loop, Tauri init
      src/
        main.rs                 # Entry point
        overlay.rs              # Magnification overlay window setup
        tauri_commands.rs       # Tauri IPC command handlers

  ui/                           # Control panel frontend (TypeScript/React)
    src/
      App.tsx
      components/
      hooks/
      types/
    package.json
    tsconfig.json

  assets/
    voices/                     # Voice model files (Kokoro, Piper)
    icons/                      # Application icons
```

### 7.2 Crate Dependency Graph

Dependencies flow in one direction: from application entry point down to platform abstractions. No circular dependencies exist.

```
luminos-app (binary)
  |
  +---> luminos-core (engine logic)
  |       |
  |       +---> luminos-platform (traits + backends)
  |       |
  |       +---> luminos-tts (TTS pipeline)
  |               |
  |               +---> luminos-platform (AudioOutput trait)
  |
  +---> luminos-gpu (rendering)
  |       |
  |       +---> luminos-platform (ScreenCapture, WindowManager traits)
  |
  +---> tauri (application framework, control panel)
```

**Key rules:**
- `luminos-platform` has no dependencies on other luminos crates. It is the foundation.
- `luminos-core` depends on `luminos-platform` (for traits) and `luminos-tts` (for TTS pipeline).
- `luminos-gpu` depends on `luminos-platform` (for `ScreenCapture` and `WindowManager` traits) and `wgpu`.
- `luminos-tts` depends on `luminos-platform` (for `AudioOutput` trait), `sherpa-rs`, and `cpal`.
- `luminos-app` is the only binary crate. Everything else is a library crate.

### 7.3 External Dependency Summary

| External Crate | Used By | Purpose |
|----------------|---------|---------|
| `winit` | `luminos-app`, `luminos-gpu` | Window creation and event loop |
| `wgpu` | `luminos-gpu` | GPU rendering |
| `xcap` | `luminos-platform` | Screen capture |
| `sherpa-rs` | `luminos-tts` | TTS neural inference |
| `cpal` | `luminos-tts`, `luminos-platform` | Audio output |
| `arboard` | `luminos-core` | Clipboard access |
| `tauri` | `luminos-app` | Application framework |
| `x11rb` | `luminos-platform` | X11 protocol (EWMH struts, XShm) |
| `atspi` | `luminos-platform` | Linux accessibility API |
| `rdev` | `luminos-platform` | Global input monitoring |
| `serde` | All crates | Serialization (config, IPC) |
| `crossbeam-channel` | `luminos-core`, `luminos-tts` | Inter-thread channels |
| `arc-swap` | `luminos-core` | Lock-free shared state (`ArcSwap<AppState>`) |

Build and distribution details (Cargo features, conditional compilation, packaging) are in [08 -- Build and Distribution](./08-build-and-distribution.md) (planned).

---

## 8. Technology Stack Summary

This table summarizes every major technology choice with its rationale. Detailed evaluation, benchmarks, and alternatives considered are in the [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md).

| Component | Technology | Version (as of 2026-03-15) | License | Rationale |
|-----------|-----------|---------------------------|---------|-----------|
| Core language | Rust (2024 edition) | 1.85+ | MIT/Apache 2.0 | Memory safety, zero-cost abstractions, compiler catches AI-generated code errors |
| Application framework | Tauri 2.0 | 2.x stable | MIT/Apache 2.0 | Lightweight (~58% less RAM than Electron), control panel only |
| Frontend UI | TypeScript + React | Latest | MIT | Largest LLM training corpus, AI-friendly UI generation |
| GPU rendering | wgpu | 28.0.0 | MIT/Apache 2.0 | Cross-platform (Vulkan/Metal/DX12/GL), 17.9M+ downloads |
| Window management | winit | 0.30.13 | Apache 2.0 | Cross-platform transparent/borderless windows, wgpu integration |
| Screen capture | xcap | 0.9.1 | Apache 2.0 | Direct X11 support, 85K monthly downloads |
| Screen capture (Windows perf) | windows-capture | Latest | MIT | WGC + DXGI Desktop Duplication for higher-performance Windows capture |
| TTS runtime | sherpa-onnx via sherpa-rs | 0.6.8 | MIT (bindings) / Apache 2.0 (runtime) | Unified runtime for Kokoro + Piper models |
| TTS model (primary) | Kokoro-82M ONNX | v1.0 | Apache 2.0 (model) | Near-commercial quality, 9 language codes (~8 unique languages) |
| TTS model (fallback) | Piper VITS | Various | MIT (model weights) | Language breadth (30+ languages) |
| Phonemizer | espeak-ng (subprocess) | Latest | GPL-3.0 (compatible with project GPLv3) | G2P conversion; subprocess for crash isolation |
| Audio output | cpal | Latest | Apache 2.0 | Cross-platform audio device access |
| Clipboard | arboard | Latest | MIT/Apache 2.0 | Cross-platform clipboard for "read selection" |
| Accessibility (Linux) | atspi | Latest | MIT/Apache 2.0 | AT-SPI2 D-Bus bindings for focus tracking |
| Accessibility (macOS) | objc2 | Latest | MIT/Apache 2.0 | AXUIElement bindings for focus tracking |
| Accessibility (Windows) | windows | Latest | MIT | UI Automation bindings for focus tracking |
| Input monitoring | rdev | Latest | MIT | Global input event capture on all platforms |
| X11 protocol | x11rb | Latest | MIT/Apache 2.0 | EWMH struts, XShm capture optimization |

**Licensing note:** Luminos is licensed under GPLv3. All direct dependencies are compatible: MIT, Apache 2.0, and GPL-3.0 (espeak-ng) are all GPLv3-compatible. The GPLv3 decision eliminates all previous license propagation concerns. See [Product Strategy](../PRODUCT_STRATEGY.md) Section 8.4 for the licensing rationale.

---

## 9. Performance Architecture

### 9.1 Performance Targets

| Metric | Target | Verification Method |
|--------|--------|---------------------|
| Frame rate | 60fps (16.67ms frame time) | Frame time histogram in CI benchmarks |
| Frame time P99 | <20ms (no visible stutter) | 99th percentile must stay below threshold |
| RAM usage | <4GB under all conditions | Memory profiler in CI |
| TTS latency | <200ms trigger-to-first-audio | End-to-end latency benchmark |
| Binary size | <50MB (excluding voice models) | CI artifact size check |
| Startup time | <2s to usable magnification | Cold start benchmark |
| GPU compatibility | Integrated GPUs (Intel UHD, Apple M-series, AMD Vega) | CI tests on integrated GPU hardware |

### 9.2 Hot Path Identification

The **only hot path** in Luminos is the magnification frame cycle (Section 5.1). Everything else operates at human interaction speed or background processing speed. Optimization effort must focus here first.

**Hot path components (ordered by typical time consumed):**
1. Screen capture (1-8ms) -- platform API call, most variable
2. GPU texture upload (0.5-2ms) -- proportional to captured region size
3. Shader execution (<1ms) -- trivially fast on any GPU
4. Viewport calculation (<0.01ms) -- pure arithmetic

**Known performance risks:**

| Risk | Impact | Mitigation | Phase |
|------|--------|------------|-------|
| xcap X11 non-SHM capture slow at low zoom | Frame drops at 1.5-3x on high-res displays | Implement x11rb XShm capture backend | Phase 1 |
| CPU-to-GPU copy per frame (xcap returns CPU buffer) | Bandwidth pressure at low zoom, high resolution | Capture only viewport source region; GPU-side texture sharing (DXGI, IOSurface) as optimization | Phase 1+ |
| Configuration reads on render thread | Lock contention | Use `ArcSwap` for lock-free reads | Phase 0 |

### 9.3 Memory Budget

| Component | Budget | Notes |
|-----------|--------|-------|
| Magnification textures | ~100MB | Source + destination GPU textures at display resolution |
| Kokoro-82M model (q8, default) | ~92MB | Loaded once; q8 is the recommended deployment variant |
| Kokoro-82M model (q4, lightweight) | ~80MB | Alternative for memory-constrained systems |
| Kokoro-82M model (fp16, quality) | ~163MB | User-selectable for higher quality |
| Kokoro-82M model (fp32, full) | ~327MB | Development/quality reference only |
| Piper models (if loaded) | ~60-75MB per model | Loaded on demand per language |
| espeak-ng subprocess | ~10-30MB | OS overhead for child process |
| TTS working memory | ~10-20MB | Phoneme buffers, audio samples, ring buffer, resampler state |
| Tauri webview (control panel) | ~30-50MB | WebkitGTK/WebView2 baseline |
| Application code + state | ~20-50MB | Rust binary, runtime allocations |
| **Total (with q8 Kokoro, default)** | **~292-392MB typical** | Well within 4GB budget |

*Model sizes from the onnx-community/Kokoro-82M-v1.0-ONNX distribution. See [04 -- TTS Pipeline](./04-tts-pipeline.md) Section 8.4 for the TTS-specific memory breakdown.*

### 9.4 Startup Sequence

The application must reach usable magnification within 2 seconds. This requires a carefully staged startup:

```
T=0ms    Process start
T=50ms   Parse config, initialize logging
T=100ms  Create winit event loop, create overlay window
T=200ms  Initialize wgpu device and rendering pipeline
T=300ms  Initialize ScreenCapture backend
T=400ms  First magnified frame rendered <-- usable magnification
T=500ms  Start input monitoring, focus tracking
T=1000ms Initialize Tauri, load control panel webview (background)
T=2000ms Load TTS model into memory (background, lazy)
```

**Key insight:** TTS model loading (~500-1000ms for Kokoro) happens after magnification is usable. The user sees a working magnifier before TTS is ready. The control panel webview also loads in the background -- it is not needed until the user opens settings.

---

## 10. Security and Privacy Architecture

### 10.1 Privacy Model

Luminos processes the user's entire screen content. This is an extraordinary level of access that demands a strict privacy architecture:

- **No network transmission of user data.** Screen content, recognized text, and usage patterns never leave the device. There is no cloud API, no analytics endpoint, no telemetry server.
- **No telemetry by default.** Optional, opt-in usage statistics (e.g., "how many hours per day is magnification active") may be added in later phases for grant reporting. This is strictly opt-in with clear user consent.
- **Local AI inference.** TTS (Kokoro, Piper), OCR (Tesseract, platform APIs), and future AI features (image description) all run on-device. No screen content is sent to external services.
- **No persistent logging of screen content.** Debug logs may temporarily contain text snippets for troubleshooting, but this is disabled by default and no logs persist screen capture data.

### 10.2 Process Isolation

The espeak-ng subprocess runs in a separate OS process. This provides:
- **Crash isolation:** An espeak-ng segfault does not crash Luminos.
- **Resource isolation:** espeak-ng memory leaks do not grow the main process's memory.
- **Privilege separation:** espeak-ng has no access to the main process's screen capture buffer or GPU textures.

The subprocess communicates via stdin/stdout pipes (text in, phonemes out). It has no network access, no file system access beyond its own data files, and no ability to interact with the display server.

### 10.3 Permission Model

Luminos requires platform-specific permissions to function. These are requested at the minimum required scope:

| Platform | Permission | Why | When Requested |
|----------|-----------|-----|----------------|
| Linux X11 | None required | X11 captures without permission | Immediate |
| Linux Wayland | Screen recording (XDG Portal) | PipeWire screen capture | First use on Wayland |
| macOS | Screen Recording | ScreenCaptureKit access | First launch |
| macOS | Accessibility | Focus tracking via AXUIElement | First use of focus tracking |
| Windows | None for capture | WGC/DXGI does not require elevation | Immediate |
| All | Audio device | cpal audio output | First TTS playback |

**Wayland permission chicken-and-egg:** On Wayland, the XDG Portal permission dialog requires user interaction, but the user may need magnification to read the dialog. This is a known platform-level UX issue. Mitigation strategies include session restoration tokens (so the dialog only appears once), clear documentation, and advocacy for OS-level accessibility of permission dialogs. See [10 -- Risk Register](./10-risk-register.md).

### 10.4 Build Integrity

- **Signed releases:** All release binaries are signed (GPG for Linux/OpenBSD, Apple codesign for macOS, Authenticode for Windows).
- **SBOM generation:** Software Bill of Materials produced for each release, enabling institutional security review.
- **Reproducible builds:** Target reproducible builds to allow independent verification of release artifacts. This is a best-effort goal (Rust reproducibility is improving but not yet guaranteed for all targets).
- **Dependency auditing:** `cargo audit` runs in CI to detect known vulnerabilities in dependencies.

---

## 11. Deployment Architecture

### 11.1 Binary Structure

Luminos ships as a single application package per platform containing:

```
luminos (or luminos.exe)        # Main binary (Rust, <50MB)
  |
  +-- Bundled Tauri webview      # Control panel (React/TS, <5MB compressed)
  |
  +-- WGSL shader files          # GPU shaders (compiled at runtime by wgpu, <100KB)
  |
  +-- espeak-ng binary           # Phonemizer subprocess (or rely on system install)
  |
  +-- espeak-ng data files       # Phoneme rules, language data (~5MB)
```

Voice model files are distributed separately (or downloaded on first use) due to their size:

| Model | Size | Distribution |
|-------|------|-------------|
| Kokoro-82M (q8) | ~92MB | Downloaded on first TTS use, or bundled in full installer |
| Kokoro-82M (q4) | ~80MB | Lightweight alternative |
| Piper voices | ~60-75MB each | Downloaded per language on demand |

### 11.2 Platform-Specific Distribution

| Platform | Package Format | Notes |
|----------|---------------|-------|
| Linux (Debian/Ubuntu) | .deb | Via apt repository |
| Linux (Fedora/RHEL) | .rpm | Via dnf repository |
| Linux (universal) | AppImage, Flatpak, snap | Self-contained distribution |
| macOS | .dmg | Signed and notarized |
| OpenBSD | Port/package | Via OpenBSD ports system |
| Windows | .msi | GPO-compatible for enterprise deployment |

Full build, packaging, and distribution details are in [08 -- Build and Distribution](./08-build-and-distribution.md) (planned).

---

## 12. Architectural Decisions Register

This section records the key architectural decisions made in this document. Each decision includes the context, the choice made, and the rationale. For technology selection decisions (crate choices, framework evaluation), see the [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md).

| # | Decision | Choice | Rationale | Status |
|---|----------|--------|-----------|--------|
| AD-01 | Window architecture | Dual-window (overlay + control panel) | Separates 60fps GPU rendering from web UI; eliminates WebkitGTK performance concerns on Linux | Decided |
| AD-02 | Overlay rendering | winit + wgpu (native Rust, no webview) | Zero web rendering overhead on hot path; direct GPU access | Decided |
| AD-03 | Control panel framework | Tauri 2.0 + React | Lightweight, AI-friendly UI generation; TypeScript has largest LLM corpus | Decided |
| AD-04 | Platform abstraction | Six Rust traits with per-platform backends | Compiler-enforced contracts, parallel development, testability via mocks | Decided |
| AD-05 | Process model | Single process, multi-threaded | Shared memory for performance; Tauri Core process hosts Rust backend + webview management | Decided |
| AD-06 | TTS process isolation | espeak-ng as subprocess | Crash isolation, resource isolation (engineering reasons, not legal) | Decided |
| AD-07 | Thread communication | Bounded channels + atomic/lock-free shared state | Prevents memory growth; lock-free reads on render hot path | Decided |
| AD-08 | Configuration access | `ArcSwap<AppState>` for lock-free reads | Render thread reads state every frame; `rcu()` updates from IPC/hotkey threads never block reads | Decided |
| AD-09 | Cargo workspace | Multi-crate workspace (core, platform, gpu, tts, app) | Fast incremental compilation, clear dependency boundaries | Decided |
| AD-10 | Startup priority | Magnification before TTS, control panel loads in background | User needs magnification immediately; TTS/settings can load lazily | Decided |
| AD-11 | Privacy architecture | No telemetry, no cloud APIs, all inference local | Screen content is maximally sensitive data; trust is non-negotiable for AT users | Decided |

---

## 13. Cross-References

| Topic | Document | Section |
|-------|----------|---------|
| Trait definitions and per-platform backends | [02 -- Platform Abstraction](./02-platform-abstraction.md) | All sections |
| GPU rendering pipeline, shaders, frame pacing | [03 -- Rendering Pipeline](./03-rendering-pipeline.md) | Pipeline design |
| TTS architecture, espeak-ng protocol, Kokoro inference | [04 -- TTS Pipeline](./04-tts-pipeline.md) | All sections |
| Tauri IPC commands, React UI architecture | [05 -- Control Panel](./05-control-panel.md) | IPC design |
| Performance budgets, security policy, licensing, accessibility, observability, error handling | [06 -- Cross-Cutting Concerns](./06-cross-cutting-concerns.md) | Sections 2-8 |
| Test architecture, CI/CD, quality gates | [07 -- Testing Strategy](./07-testing-strategy.md) (planned) | All sections |
| Cargo workspace, packaging, signing, release engineering | [08 -- Build and Distribution](./08-build-and-distribution.md) (planned) | Workspace layout, packaging |
| Phased milestones, story breakdown, delivery timeline | [09 -- Implementation Roadmap](./09-implementation-roadmap.md) (planned) | Phase 0-4 |
| Technical risks, mitigations, monitoring | [10 -- Risk Register](./10-risk-register.md) (planned) | All risks |
| Product requirements, feature roadmap, personas | [Product Strategy](../PRODUCT_STRATEGY.md) | Sections 5, 7, 8 |
| Technology selection rationale, benchmarks | [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) | All sections |

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-15 | Initial system architecture |
