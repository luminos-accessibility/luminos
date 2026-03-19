# 10 -- Risk Register

**Status:** DRAFT v1.0
**Date:** 2026-03-18
**Owner:** Technical Strategy
**Review Cadence:** Phase gates + quarterly review
**Source Assessments:** Architecture, Security/Privacy/Licensing, Build/Distribution, Performance/Implementation, Product/Schedule/Resource

---

## 1. Purpose and Scope

This document is the **canonical risk register** for the Luminos project. It consolidates technical, operational, and strategic risks identified across all nine technical strategy documents (01-09), the Product Strategy (v1.3), and the Technology Stack Evaluation (FINAL).

The risk register serves three audiences:
1. **Engineering team and AI agents:** Identifies risks that affect implementation decisions and must be monitored during development.
2. **Project leadership:** Provides a prioritized view of risks requiring strategic decisions or resource allocation.
3. **Technical auditors and contributors:** Documents known risks, their mitigations, and acceptance rationale for project governance.

### 1.1 Relationship to Other Documents

Risks are cross-referenced to source documents using the notation `[doc-NN Section X.Y]`. Every risk traces to a specific architectural decision, dependency choice, or project constraint documented in the tech strategy.

```
01-system-architecture.md    -- Architecture, threading, state management
02-platform-abstraction.md   -- Trait definitions, platform backends
03-rendering-pipeline.md     -- GPU capture, shaders, frame pacing
04-tts-pipeline.md           -- TTS architecture, espeak-ng, Kokoro
05-control-panel.md          -- Tauri IPC, React UI, settings
06-cross-cutting-concerns.md -- Performance, security, licensing, a11y
07-testing-strategy.md       -- Test architecture, CI/CD, quality gates
08-build-and-distribution.md -- Cargo workspace, packaging, signing
09-implementation-roadmap.md -- Phased milestones, epic breakdown
```

### 1.2 How This Document Is Maintained

See [Section 11: Governance and Maintenance](#11-governance-and-maintenance) for the review cadence, update procedures, and lifecycle management rules.

---

## 2. Risk Scoring Methodology

### 2.1 Likelihood Scale

| Level | Label | Description |
|-------|-------|-------------|
| 1 | Low | Unlikely to occur; requires multiple unlikely preconditions |
| 2 | Medium | Could occur under realistic conditions; has precedent in similar projects |
| 3 | High | Likely to occur based on current evidence; multiple indicators present |
| 4 | Certain | Will occur; is a known constraint or confirmed limitation |

### 2.2 Impact Scale

| Level | Label | Description |
|-------|-------|-------------|
| 1 | Low | Minor inconvenience; workaround exists; no user-facing degradation |
| 2 | Medium | Noticeable degradation; affects subset of users or platforms; schedule impact < 2 weeks |
| 3 | High | Significant feature degradation or schedule impact; affects core user experience |
| 4 | Critical | Blocks a phase gate, violates a core constraint, or threatens project viability |

### 2.3 Risk Score Matrix

| | Low Impact (1) | Medium Impact (2) | High Impact (3) | Critical Impact (4) |
|---|---|---|---|---|
| **Certain (4)** | Medium (4) | High (8) | Critical (12) | Critical (16) |
| **High (3)** | Medium (3) | High (6) | Critical (9) | Critical (12) |
| **Medium (2)** | Low (2) | Medium (4) | High (6) | Critical (8) |
| **Low (1)** | Low (1) | Low (2) | Medium (3) | High (4) |

**Score ranges:** 1-3 = Accept, 4-6 = Monitor, 7-9 = Mitigate, 10-16 = Escalate

### 2.4 Risk Status Definitions

| Status | Meaning |
|--------|---------|
| **Open** | Risk identified; mitigation not yet implemented |
| **Mitigating** | Mitigation actions in progress; risk not yet fully addressed |
| **Accepted** | Risk acknowledged; accepted with documented rationale |
| **Closed** | Risk eliminated or no longer applicable |
| **Triggered** | Risk event has occurred; contingency plan activated |

---

## 3. Master Risk Summary

The following table lists all 38 consolidated risks, sorted by score (descending), then by phase (earliest first). Each risk consolidates one or more findings from the five specialist assessments (Architecture, Security, Build, Performance, Product).

| ID | Title | Category | L | I | Score | Phase | Status |
|----|-------|----------|---|---|-------|-------|--------|
| RISK-001 | Dual event loop coexistence (winit + Tauri) | Architecture | 2 | 4 | **8** | P0 | Open |
| RISK-002 | Self-capture infinite feedback loop | Architecture | 3 | 3 | **9** | P0 | Open |
| RISK-003 | Platform trait surface area inadequacy | Architecture | 3 | 2 | **6** | P1 | Open |
| RISK-004 | Render thread starvation under load | Architecture | 2 | 3 | **6** | P0 | Open |
| RISK-005 | TTS pipeline concurrency hazards | Architecture | 2 | 2 | **4** | P2 | Open |
| RISK-006 | Multi-display and HiDPI coordinate inconsistencies | Architecture | 3 | 2 | **6** | P0 | Open |
| RISK-007 | X11 capture bottleneck at low zoom on high-res displays | Performance | 3 | 3 | **9** | P0 | Open |
| RISK-008 | CPU-to-GPU texture upload bandwidth pressure | Performance | 2 | 3 | **6** | P0 | Open |
| RISK-009 | TTS 200ms latency target unreachable on budget hardware | Performance | 3 | 2 | **6** | P2 | Open |
| RISK-010 | Memory pressure on 4GB total RAM systems | Performance | 2 | 3 | **6** | P0 | Open |
| RISK-011 | Font re-rendering feasibility and performance | Performance | 3 | 4 | **12** | P1-P3 | Open |
| RISK-012 | Wayland platform integration (capture, input, permissions) | Platform | 3 | 3 | **9** | P1 | Open |
| RISK-013 | OpenBSD GPU, audio, and CI gaps | Platform | 3 | 2 | **6** | P3 | Open |
| RISK-014 | macOS permission model and annual API churn | Platform | 2 | 2 | **4** | P2 | Open |
| RISK-015 | Windows screen reader coexistence (NVDA/JAWS) | Platform | 2 | 4 | **8** | P4 | Open |
| RISK-016 | wgpu backend compatibility across platforms | Platform | 2 | 2 | **4** | P0+ | Open |
| RISK-017 | Screen content leakage via logs and GPU memory | Security | 2 | 3 | **6** | P0 | Open |
| RISK-018 | espeak-ng subprocess command injection and PATH hijacking | Security | 2 | 3 | **6** | P2 | Open |
| RISK-019 | ONNX model supply chain integrity | Security | 1 | 4 | **4** | P2 | Open |
| RISK-020 | Tauri webview and WebkitGTK attack surface | Security | 1 | 3 | **3** | P0 | Accepted |
| RISK-021 | ONNX Runtime FFI memory safety boundary | Security | 1 | 3 | **3** | P2 | Open |
| RISK-022 | GPLv3 dependency license compatibility | Licensing | 2 | 3 | **6** | P0+ | Mitigating |
| RISK-023 | Regulatory and self-accessibility compliance | Compliance | 2 | 3 | **6** | P0+ | Open |
| RISK-024 | Binary size budget with ONNX Runtime | Build | 3 | 3 | **9** | P0 | Open |
| RISK-025 | Code signing key proliferation and custody | Build | 2 | 4 | **8** | P0 | Open |
| RISK-026 | espeak-ng bundling and version skew across platforms | Build | 2 | 3 | **6** | P1 | Open |
| RISK-027 | CI pipeline performance and platform coverage gaps | Build | 3 | 2 | **6** | P0+ | Open |
| RISK-028 | Tauri build system constraints (dist profile, Flatpak/Snap) | Build | 3 | 2 | **6** | P1 | Open |
| RISK-029 | Auto-update mechanism complexity per install method | Build | 2 | 3 | **6** | P1 | Open |
| RISK-030 | wgpu/winit/Tauri major version upgrade cascade | Ecosystem | 3 | 3 | **9** | P0+ | Open |
| RISK-031 | Single-maintainer crate dependencies (xcap, sherpa-rs, atspi) | Ecosystem | 2 | 2 | **4** | P0+ | Open |
| RISK-032 | TTS ecosystem volatility (Kokoro, sherpa-onnx) | Ecosystem | 2 | 2 | **4** | P2 | Open |
| RISK-033 | Scope ambition exceeds realistic capacity | Project | 3 | 3 | **9** | All | Open |
| RISK-034 | AI-agent development model unproven at this scale | Project | 3 | 3 | **9** | All | Open |
| RISK-035 | Funding sustainability gap | Project | 3 | 3 | **9** | P0-P1 | Open |
| RISK-036 | Bus factor of one (founder key-person dependency) | Project | 2 | 4 | **8** | P0+ | Open |
| RISK-037 | Adoption barriers and contributor recruitment | Project | 3 | 2 | **6** | P1+ | Open |
| RISK-038 | i18n technical debt from Phase 4 deferral | Project | 3 | 2 | **6** | P0-P4 | Open |

**Score distribution:** 1 Escalate (10-16): RISK-011; 12 Mitigate (7-9); 23 Monitor (4-6); 2 Accept (1-3).

---

## 4. Architecture Risks

### RISK-001: Dual Event Loop Coexistence (winit + Tauri)

| Field | Value |
|-------|-------|
| **Category** | Architecture |
| **Likelihood** | Medium (2) |
| **Impact** | Critical (4) |
| **Score** | **8 -- Mitigate** |
| **Phase** | Phase 0 (must validate in E2/E4) |
| **Status** | Open |
| **Sources** | ARCH-001, BUILD-024 |

**Description:** Luminos runs both a winit event loop (main thread, magnification overlay) and Tauri's internal webview event loop within a single OS process [doc-01 Section 6.5]. On macOS, Cocoa requires the main thread for NSApplication event processing, which both winit and Tauri's WKWebView need. There is no documented, production-validated pattern for running winit and Tauri 2.0 side-by-side -- screenpipe (the closest precedent, cited in TECH_STACK_EVALUATION.md Section 4.1) does not use winit for a second window. Additionally, both `tauri` and `winit` pull platform-specific windowing libraries that may conflict in the dependency tree (e.g., different versions of `raw-window-handle`, `nix`, or `wayland-client`).

**Mitigation:**
1. Build a minimal proof-of-concept in E1/E2 that creates both a Tauri webview and a winit+wgpu overlay in the same process on Linux X11, macOS, and Windows.
2. On macOS, investigate running winit on the main thread with Tauri's webview managed via `Builder::any_thread()`. Explore `winit::platform::macos::EventLoopBuilderExtMacOS::with_activation_policy()`.
3. In E1, run `cargo tree -d` to detect duplicate transitive dependencies between Tauri and winit/wgpu. Use workspace dependency deduplication where conflicts arise.

**Contingency:** Separate the control panel into its own process. The magnification overlay process communicates with the control panel process via Unix domain sockets (Linux/macOS) or named pipes (Windows). This sacrifices shared `ArcSwap<AppState>` reads but preserves core magnification performance.

**Detection:** E2 integration testing. Measure frame time P99 with and without the Tauri webview window open. If the overlay fails to render at 60fps or the webview becomes unresponsive, this risk is materializing.

---

### RISK-002: Self-Capture Infinite Feedback Loop

| Field | Value |
|-------|-------|
| **Category** | Architecture |
| **Likelihood** | High (3) |
| **Impact** | High (3) |
| **Score** | **9 -- Mitigate** |
| **Phase** | Phase 0 (must solve in E2) |
| **Status** | Open |
| **Sources** | ARCH-009 |

**Description:** In full-screen magnification mode, the overlay covers the entire display [doc-03 Section 7.1]. If `ScreenCapture::capture_frame()` captures the overlay window itself, the result is an infinite feedback loop: the magnified view captures itself, producing exponentially zoomed recursive images that make the screen unusable. Per-platform exclusion mechanisms are documented (X11 composite pixmap, PipeWire node ID, `SCContentFilter`, DXGI auto-exclusion), but implementation gaps remain: (a) X11 "temporarily unmap/remap" creates visible flicker at 60fps, (b) X11 composite pixmap requires the composite extension and may not work on tiling WMs without compositing, (c) the `ScreenCapture` trait has no parameter for window exclusion [doc-03 Section 7.1].

**Mitigation:**
1. Add a `set_excluded_windows(&self, window_ids: &[WindowId])` method to the `ScreenCapture` trait or accept exclusion at construction time.
2. On X11, use `xcb_composite_redirect_window` to capture from the root composite buffer, which naturally excludes override-redirect windows.
3. Test self-capture prevention in E2 by rendering a known solid-color overlay, capturing a frame, and verifying the captured frame does NOT contain the overlay color.

**Contingency:** Software-based detection: render a small watermark at a known position with a known color. If captured frames contain the watermark, skip the frame and re-render the previous one.

**Detection:** Integration test on Xvfb in CI and real X11 desktops during manual testing.

---

### RISK-003: Platform Trait Surface Area Inadequacy

| Field | Value |
|-------|-------|
| **Category** | Architecture |
| **Likelihood** | High (3) |
| **Impact** | Medium (2) |
| **Score** | **6 -- Monitor** |
| **Phase** | Phase 1 (Wayland is the first stress test) |
| **Status** | Open |
| **Sources** | ARCH-002 |

**Description:** The six platform traits (`ScreenCapture`, `FocusTracker`, `TtsEngine`, `WindowManager`, `InputMonitor`, `AudioOutput`) defined in [doc-02 Section 3] abstract five fundamentally different platforms. Trait definitions were designed from research before any backend implementation. Specific concerns: (a) `WindowManager::set_overlay_mode()` handles EWMH struts, wlr-layer-shell, NSPanel, and AppBar through a single `OverlayMode` enum -- but docked mode behavior differs fundamentally per platform [doc-02 Section 3.5]; (b) `ScreenCapture::capture_frame()` is synchronous, but Wayland capture via PipeWire is inherently asynchronous; (c) `InputMonitor` on Wayland requires `rdev::grab()` which intercepts events rather than passively monitoring them [doc-02 Section 8.2].

**Mitigation:**
1. Treat trait definitions as living contracts revised during backend implementation. Track signature changes per backend.
2. Add a `capabilities()` method to `WindowManager` returning a bitflags struct indicating platform support (e.g., `DOCK_RESERVATION`, `CLICK_THROUGH`, `TRANSPARENT`).
3. For `ScreenCapture` on Wayland, manage the PipeWire stream internally on a dedicated capture thread, presenting a synchronous interface. Alternatively, introduce `capture_frame_async()` with a default implementation wrapping the sync version.

**Contingency:** Split any fundamentally incompatible trait into a base trait plus platform-specific extension traits. Use `downcast_ref` or optional extension methods with default no-op implementations.

**Detection:** Track how many trait method signatures require modification per backend. More than 2 signature changes per backend indicates inadequate abstraction.

---

### RISK-004: Render Thread Starvation Under Load

| Field | Value |
|-------|-------|
| **Category** | Architecture / Performance |
| **Likelihood** | Medium (2) |
| **Impact** | High (3) |
| **Score** | **6 -- Monitor** |
| **Phase** | Phase 0-1 |
| **Status** | Open |
| **Sources** | ARCH-003, ARCH-007, ARCH-012 |

**Description:** The render thread must complete non-vsync work in <8ms [doc-01 Section 6.3] and reads `ArcSwap<AppState>` every frame. While `ArcSwap::load()` is lock-free (<10ns per [doc-06 Section 2.3]), the render thread also calls `ScreenCapture::capture_frame()` synchronously [doc-03 Section 2.1]. On Linux X11 with xcap's non-SHM path, capture can take 3-8ms for large regions at low zoom [doc-01 Section 9.2]. Combined with `Queue::write_texture()` (2-3ms for large regions) and OS scheduling jitter, the pipeline can exceed the 16.67ms budget. Additionally, IPC-originated setting changes (via `EventLoopProxy::send_event()`) are asynchronous and may lag by up to one frame, causing configuration-to-render latency of 16-33ms [doc-05 Section 2.1].

**Mitigation:**
1. Implement XShm capture backend (planned E6, Phase 1) reducing capture to 0.5-3ms.
2. Implement dirty-region detection [doc-03 Section 10.3]: skip capture and upload for unchanged content.
3. Set render thread to high priority (`pthread_setschedparam` on Linux, `SetThreadPriority` on Windows).
4. Group related IPC settings into single `rcu()` calls. Accept single-frame glitches as imperceptible at 60fps.

**Contingency:** Auto-degrade per [doc-06 Section 2.5]: reduce internal render resolution by 50% when P99 > 33ms for 10 seconds.

**Detection:** `FrameTimings` ring buffer [doc-03 Section 8.3] with 20ms P99 threshold. CI benchmarks at 2x zoom on 1920x1080 Xvfb.

---

### RISK-005: TTS Pipeline Concurrency Hazards

| Field | Value |
|-------|-------|
| **Category** | Architecture |
| **Likelihood** | Medium (2) |
| **Impact** | Medium (2) |
| **Score** | **4 -- Monitor** |
| **Phase** | Phase 2 |
| **Status** | Open |
| **Sources** | ARCH-005, ARCH-006, ARCH-014 |

**Description:** The TTS pipeline uses sentence-level pipelining [doc-04 Section 9.3] with multiple concurrent hazards: (a) Interrupt during pipelining: when `interrupt: true` arrives, up to 150ms passes before the inference thread checks the `AtomicBool` flag, during which stale audio continues playing [doc-04 Section 6.4]; (b) Ring buffer overflow: if the cpal audio callback is delayed, the inference thread blocks indefinitely [doc-04 Section 7.2]; (c) Resampling in the audio callback path has hard real-time deadlines [doc-04 Section 7.2]; (d) espeak-ng subprocess pipe deadlock: if both processes block on their respective pipes simultaneously [doc-04 Section 5.2]; (e) Audio device hot-swap during speech causes silent TTS failure [doc-02 Section 3.7].

**Mitigation:**
1. Pre-resample in the inference thread, not the audio callback. The callback becomes a simple memcpy from ring buffer.
2. Use bounded timeout (500ms) on inference thread ring buffer writes. Drop samples on timeout rather than blocking indefinitely.
3. Implement non-blocking I/O on espeak-ng subprocess pipes with a dedicated reader thread.
4. Detect audio device changes via platform APIs and auto-recreate the cpal stream.

**Contingency:** Fall back to a sequential TTS model (phonemize all, then synthesize, then play). This increases first-sentence latency by ~10-50ms but eliminates inter-stage race conditions.

**Detection:** Integration test `tts_pipeline_integration_text_to_audio` with multi-sentence input. Ring buffer underrun counter in `TtsStatus` diagnostics.

---

### RISK-006: Multi-Display and HiDPI Coordinate Inconsistencies

| Field | Value |
|-------|-------|
| **Category** | Architecture |
| **Likelihood** | High (3) |
| **Impact** | Medium (2) |
| **Score** | **6 -- Monitor** |
| **Phase** | Phase 0 (HiDPI), Phase 2 (macOS) |
| **Status** | Open |
| **Sources** | ARCH-013 |

**Description:** The architecture uses `ScreenRect` and `ScreenPoint` types with pixel coordinates [doc-02 Section 3.1], but the coordinate space differs by platform and API: `ScreenCapture` operates in physical pixels, `WindowManager` may operate in logical points (winit defaults on macOS), `InputMonitor` reports vary by platform, and `FocusTracker` uses screen coordinates (physical on X11, logical on macOS). Multi-monitor setups with different scale factors (e.g., 4K at 2x + 1080p at 1x) create coordinate space mismatches that cause the magnification viewport to be offset from the cursor position.

**Mitigation:**
1. Establish a project-wide convention: all `ScreenRect` and `ScreenPoint` values are in **physical pixels**. Document this in trait definitions.
2. Each platform backend converts from native coordinate system to physical pixels before returning values.
3. Add `ScreenPoint::to_physical(scale: f64)` and `ScreenPoint::to_logical(scale: f64)` conversions.
4. Add guards in the viewport calculator [doc-03 Section 3.1] to catch coordinate space mismatches.

**Contingency:** Introduce a `CoordinateSpace` enum (`Physical`, `Logical`) attached to every `ScreenRect` and `ScreenPoint`, forcing explicit conversion at the type level.

**Detection:** Integration test on HiDPI Xvfb (3840x2160 with `GDK_SCALE=2`). Verify viewport center aligns with mouse cursor within 2 physical pixels.

---

## 5. Performance Risks

### RISK-007: X11 Capture Bottleneck at Low Zoom on High-Resolution Displays

| Field | Value |
|-------|-------|
| **Category** | Performance |
| **Likelihood** | High (3) |
| **Impact** | High (3) |
| **Score** | **9 -- Mitigate** |
| **Phase** | Phase 0 (exposed), Phase 1 (mitigated) |
| **Status** | Open |
| **Sources** | PERF-001 |

**Description:** xcap (v0.9.1) uses `xcb_get_image` for X11 screen capture, performing a full X server round-trip per call. Capture time scales linearly with region size. At 1.5x zoom on 1080p, the source region is 1280x720 = 3.5MB BGRA. At 1.5x zoom on 4K, the source region is 2560x1440 = 14.7MB BGRA [doc-03 Section 3.2]. xcap's XCB dependency does not enable the `shm` feature [TECH_STACK_EVALUATION.md Section 4.3]. On 4K at 1.5x zoom, `xcb_get_image` must copy 14.7MB through the X server socket per frame, likely reaching 15-25ms -- well beyond the 8ms capture budget [doc-06 Section 2.3].

**Mitigation:**
1. Implement `x11rb`-based XShm capture backend as E6 (Phase 1) [doc-09 Section 3.2]. XShm eliminates the socket copy, reducing capture to 0.5-3ms. OBS validates this approach at 60fps+.
2. Viewport-only capture in Phase 0 (already planned) [doc-06 Section 2.6].

**Contingency:** Enforce minimum zoom floor of 3x on X11 at resolutions above 1080p, or enable automatic frame rate reduction (30fps) at low zoom with adaptive capture rate.

**Detection:** CI frame time benchmark (P99 < 20ms). Instrument capture stage timing via `FrameTimings` per-stage breakdown [doc-06 Section 6.4]. Test at 1.5x zoom on 1080p and 4K in CI.

---

### RISK-008: CPU-to-GPU Texture Upload Bandwidth Pressure

| Field | Value |
|-------|-------|
| **Category** | Performance |
| **Likelihood** | Medium (2) |
| **Impact** | High (3) |
| **Score** | **6 -- Monitor** |
| **Phase** | Phase 0 (exposed), Phase 1-2 (mitigated) |
| **Status** | Open |
| **Sources** | PERF-002 |

**Description:** Every frame, `wgpu::Queue::write_texture()` uploads the CPU-side capture buffer to GPU memory [doc-03 Section 5.2]. At 1.5x zoom on 4K, this is 14.7MB at 60fps = 882MB/s through the system memory bus. On integrated GPUs sharing system memory with DDR4-2400 (~20GB/s practical), the upload takes ~1.8-2.9ms against a 3ms budget [doc-06 Section 2.3]. Combined with an already-tight capture stage, total capture+upload at low zoom on 4K could reach 15-28ms.

**Mitigation:**
1. Dirty-region tracking in Phase 1+ [doc-03 Section 10.3] -- only upload changed pixels, saving 0.5-2ms on static screens.
2. GPU texture sharing in Phase 2+ [doc-03 Section 10.4]: DXGI (Windows), IOSurface (macOS), DMA-BUF (Wayland) eliminate the CPU-GPU copy entirely.

**Contingency:** Adaptive internal render resolution reduction per [doc-06 Section 2.5 Level 3 degradation]: reduce resolution by 50%, halving upload size. Alternatively, double-buffered async upload.

**Detection:** Per-frame upload timing in `FrameTimings`. CI benchmark at 1.5x zoom on 4K when hardware available. Monitor upload-time-to-budget ratio.

---

### RISK-009: TTS 200ms Latency Target Unreachable on Budget Hardware

| Field | Value |
|-------|-------|
| **Category** | Performance |
| **Likelihood** | High (3) |
| **Impact** | Medium (2) |
| **Score** | **6 -- Monitor** |
| **Phase** | Phase 2 |
| **Status** | Open |
| **Sources** | PERF-003, PROJ-013 |

**Description:** The TTS worst-case first-sentence path is 221ms [doc-04 Section 10.2]: text preprocessing (<1ms) + espeak-ng phonemization (50ms) + Kokoro inference first chunk (150ms) + ring buffer + cpal callback (20ms). This already exceeds the 200ms target [doc-01 Section 9.1]. On budget hardware matching the "Amara" persona (4GB RAM, low-spec laptop), Kokoro inference could reach 300-500ms. Kokoro's RTF on Raspberry Pi 4 is 0.25-0.4 [doc-04 Section 6.2].

**Mitigation:**
1. Offer Kokoro q4 model (~80MB) as a lightweight alternative with faster inference.
2. Keep espeak-ng subprocess warm to eliminate ~100-200ms spawn overhead [doc-04 Section 10.4].
3. Sentence pipelining hides phonemization latency for sentences 2+ [doc-04 Section 9.3].
4. Memory-aware model selection: auto-select q4 on systems with <6GB RAM.

**Contingency:** Platform-native TTS fallback [doc-04 Section 12]: AVSpeech (macOS), SAPI (Windows), speech-dispatcher (Linux). Accept lower voice quality in exchange for meeting latency target. Document the 200ms target as applicable to "modern desktop CPUs" and provide a "low-spec mode."

**Detection:** TTS latency benchmark in CI (P99 < 300ms hard limit per [doc-06 Section 2.1]). Benchmark on representative lower-spec hardware.

---

### RISK-010: Memory Pressure on 4GB Total RAM Systems

| Field | Value |
|-------|-------|
| **Category** | Performance |
| **Likelihood** | Medium (2) |
| **Impact** | High (3) |
| **Score** | **6 -- Monitor** |
| **Phase** | Phase 0 (magnification), Phase 2 (TTS compounds) |
| **Status** | Open |
| **Sources** | PERF-004 |

**Description:** Peak memory budget is 497MB (q8 Kokoro) [doc-06 Section 2.2]. On a 4GB system (Amara persona, Product Strategy Section 6): OS + desktop (~1GB) + Luminos (497MB) + browser (500MB-1.5GB) = 3-4GB, triggering swap. A single page fault on the render thread causes a 5-50ms stall -- far exceeding the 16.67ms frame budget. The memory budget does not include page cache, GPU driver allocations, or the X server itself (~50-150MB).

**Mitigation:**
1. Default to q4 Kokoro on systems with <8GB RAM.
2. Defer TTS model loading until user activates TTS (lazy loading at T=2000ms per [doc-01 Section 9.4]).
3. Detect available RAM at startup, auto-select model, show warning if RAM < 4GB.
4. Use `mlock()` on render thread critical allocations to prevent swap-out.

**Contingency:** Unload TTS model when not in use (free ~80-92MB). Reduce GPU texture over-allocation from 1.5x to 1.0x [doc-03 Section 5.3]. Document 8GB as recommended minimum, 4GB as "basic magnification only."

**Detection:** CI memory high-water mark test (peak RSS < 1GB per [doc-06 Section 2.4]). Memory profiling on simulated 4GB system (cgroup memory limit).

---

### RISK-011: Font Re-Rendering Feasibility and Performance

| Field | Value |
|-------|-------|
| **Category** | Performance / Feasibility |
| **Likelihood** | High (3) |
| **Impact** | Critical (4) |
| **Score** | **12 -- Escalate** |
| **Phase** | Phase 1 (research), Phase 3 (implementation) |
| **Status** | Open |
| **Sources** | PERF-005, PROJ-017 |

**Description:** Font re-rendering is "the key commercial differentiator" [Product Strategy Section 4.1] and the feature separating Luminos from free built-in OS magnifiers at zoom > 4x. ZoomText's xFont and SuperNova's TrueFonts represent decades of proprietary R&D. No open-source project has implemented this. Four major open questions remain [doc-03 Section 11.3]: (a) font metadata extraction reliability across platforms and applications, (b) custom/embedded fonts in web apps and PDF viewers, (c) text in non-accessible contexts (images, canvas, games, terminals), (d) per-frame performance of font matching, glyph rasterization, and compositing. The 5-week epic estimate [doc-09 E13] is likely off by 3-10x for production quality. ZoomText has refined xFont over 30+ years.

**This is the highest-scored risk in the entire register.**

**Mitigation:**
1. Begin research spike in Phase 1 (2 weeks, time-boxed) on accessibility API font data availability. Test AXFont (macOS) and UIA font properties (Windows).
2. Implement incrementally: start with highest-coverage scenario (native OS toolkit text with full accessibility tree) before custom fonts.
3. Make font re-rendering a per-app setting, disabled where it produces artifacts.
4. Invest in bicubic interpolation quality (Phase 1 shader upgrade) as the baseline for apps where font re-rendering does not work.
5. Decouple competitive positioning from font re-rendering -- "the only cross-platform magnification + TTS tool" is independently strong.

**Contingency:** If the 2-week research spike does not produce a working prototype extracting text from at least one application on Linux, defer the full epic to v2.0+. Apply freed time to Windows platform work or OCR. Reframe value proposition around integrated mag+TTS rather than font quality.

**Detection:** Phase 1 spike deliverable: percentage of on-screen text with successful font metadata extraction, visual quality comparison at 10x zoom, per-frame performance cost. Go/no-go decision at spike end.

---

## 6. Platform Risks

### RISK-012: Wayland Platform Integration (Capture, Input, Permissions)

| Field | Value |
|-------|-------|
| **Category** | Platform |
| **Likelihood** | High (3) |
| **Impact** | High (3) |
| **Score** | **9 -- Mitigate** |
| **Phase** | Phase 1 (E8) |
| **Status** | Open |
| **Sources** | PERF-006, PERF-009, ARCH-011, SEC-011, SEC-016, PROJ-009, PROJ-015 |

**Description:** Wayland is the single riskiest platform, with interconnected risks across capture, input, permissions, and compositor fragmentation:

1. **Permission chicken-and-egg (Certain likelihood):** XDG Desktop Portal requires a user consent dialog to grant screen capture. A low-vision user who needs magnification to read the dialog cannot magnify it because capture permission hasn't been granted [doc-02 Section 8.2]. Session restore tokens mitigate after first grant, but the first-run experience is broken.

2. **Input monitoring (Certain):** `rdev::listen()` does NOT work on Wayland (X11-only APIs). The only path is `rdev::grab()` (requires `unstable_grab` feature, `input` group membership), which **intercepts** events rather than passively monitoring them [doc-02 Section 8.2]. This is semantically wrong for a magnification tool and grants keylogger-level access to all input devices.

3. **Compositor fragmentation:** GNOME Mutter does NOT support `wlr-layer-shell` -- the protocol needed for docked overlay mode. On GNOME Wayland, docked mode falls back to a floating window [doc-02 Section 8.2]. The testing matrix is {GNOME, KDE, Sway, Hyprland} x {3 modes} x {filters} = ~192 scenarios.

4. **Capture latency:** PipeWire adds D-Bus round-trip (~0.5-2ms) and stream scheduling overhead. Jitter is higher than X11 direct capture.

**Mitigation:**
1. Persist session restore tokens immediately after first grant. Display a large-font, high-contrast pre-magnification dialog (using GTK directly) explaining the permission flow.
2. Investigate direct `libinput` integration via the `input` crate for passive event monitoring without interception. Evaluate `xdg-global-shortcuts` portal for hotkeys on GNOME.
3. Tier compositor support: **Tier 1** (tested in CI): GNOME + KDE. **Tier 2** (best-effort): wlroots compositors.
4. Accept floating-overlay fallback on GNOME Wayland for docked mode.
5. Buffer 2-3 frames of PipeWire output to absorb capture jitter.
6. File upstream accessibility bugs with GNOME, KDE, and freedesktop.org.

**Contingency:** On first launch under Wayland without permission, offer to switch to XWayland mode (if available) where no permission dialog is needed. For input monitoring, implement `rdev::grab()` with immediate event passthrough (<0.1ms interception delay) as the initial implementation while pursuing libinput long-term.

**Detection:** Phase 1 integration testing on GNOME Wayland, KDE Plasma Wayland, and Sway. First-run UX testing with actual low-vision users. Begin compositor research during Phase 0 (Month 2) -- if no viable GNOME overlay approach is identified by Phase 0 gate, E8 scope must be revised.

---

### RISK-013: OpenBSD GPU, Audio, and CI Gaps

| Field | Value |
|-------|-------|
| **Category** | Platform |
| **Likelihood** | High (3) |
| **Impact** | Medium (2) |
| **Score** | **6 -- Monitor** |
| **Phase** | Phase 3 (E15) |
| **Status** | Open |
| **Sources** | PERF-007, PERF-008, PERF-011, BUILD-003 |

**Description:** OpenBSD presents three compounding challenges:

1. **GPU:** Vulkan support via Mesa is limited [doc-02 Section 8.4]. wgpu's `Backends::GL` OpenGL ES fallback is needed but is less mature -- higher CPU overhead per draw call, potential sRGB texture handling issues, and pre-multiplied alpha compositing differences.

2. **Audio:** cpal's sndio backend has a pending upstream PR (#493) open since 2020 [doc-04 Section 7.4]. Without sndio, TTS audio output is completely blocked. Four workaround options exist: contribute upstream, maintain a patched fork, use `sndio-sys` directly, or require PulseAudio.

3. **CI:** GitHub Actions has no OpenBSD runners [doc-07 Section 4.7]. For 9+ months (until Phase 3), OpenBSD compilation is unverified. Changes to shared X11 code could silently break OpenBSD.

**Mitigation:**
1. Request `wgpu::Backends::all()` for automatic Vulkan-to-GL fallback. Default to 30fps performance mode on GL backend. Use bilinear interpolation (skip bicubic) on OpenBSD to reduce shader cost.
2. Engage cpal maintainers on PR #493 in Phase 2. Prepare `sndio-sys` direct integration as a parallel option.
3. Extract shared X11 code into `common::x11_common` compiled on both `target_os = "linux"` and `target_os = "openbsd"`. Add periodic cross-compilation checks before Phase 3.

**Contingency:** Ship OpenBSD as "magnification at reduced quality" without TTS initially. Accept bilinear-only interpolation and 30fps. Magnification alone is still valuable on a platform with zero accessibility tools.

**Detection:** Build validation on OpenBSD hardware during Phase 3. Frame timing benchmarks per GPU backend. cpal PR #493 status monitoring.

---

### RISK-014: macOS Permission Model and Annual API Churn

| Field | Value |
|-------|-------|
| **Category** | Platform |
| **Likelihood** | Medium (2) |
| **Impact** | Medium (2) |
| **Score** | **4 -- Monitor** |
| **Phase** | Phase 2 (E12) |
| **Status** | Open |
| **Sources** | SEC-012, BUILD-004, PROJ-016 |

**Description:** macOS introduces two ongoing maintenance challenges:

1. **Permission revocation:** Screen Recording permission can be revoked at any time or reset during macOS upgrades [doc-02 Section 8.3]. If revoked while running, `capture_frame()` returns `CaptureError::PermissionDenied`. The stale-frame mechanism displays the last frame for 1 second, then warns [doc-03 Section 4.4]. macOS also requires Accessibility permission for `FocusTracker`.

2. **Annual deprecation cycles:** Apple deprecates APIs with 1-2 year notice. ScreenCaptureKit replaced `CGWindowListCreateImage` in macOS 15. Future releases may change ScreenCaptureKit, AXUIElement, or Metal shader compilation. The xcap crate must track these changes.

3. **CI limitation:** GitHub Actions macOS runners do NOT grant Screen Recording permission (actions/runner-images#8951). macOS capture integration tests are manual-only until a self-hosted runner is provisioned.

**Mitigation:**
1. Detect permission status at startup. Display user-friendly dialog with a button opening System Settings if denied.
2. Monitor for `CaptureError::PermissionDenied` during runtime; switch to "permission lost" UI state.
3. Test on macOS beta builds during Apple's developer beta period (June-September annually). Budget 1-2 weeks for annual compatibility work.
4. Provision a self-hosted Mac mini M2 (~$600) for Phase 2 with pre-granted permissions.

**Contingency:** Ship unsigned builds when certificates are unavailable (Gatekeeper warnings can be bypassed via right-click -> Open). For API breakage, remain on the last working macOS version and document the limitation until patched.

**Detection:** Automated test on self-hosted macOS runner (Phase 2+). Calendar reminders for annual macOS beta testing.

---

### RISK-015: Windows Screen Reader Coexistence (NVDA/JAWS)

| Field | Value |
|-------|-------|
| **Category** | Platform |
| **Likelihood** | Medium (2) |
| **Impact** | Critical (4) |
| **Score** | **8 -- Mitigate** |
| **Phase** | Phase 4 (E17-E18) |
| **Status** | Open |
| **Sources** | PROJ-018 |

**Description:** The roadmap itself identifies this as "the highest-risk item in the entire project" [doc-09 E18]. Luminos must not interfere with NVDA or JAWS operation. Challenges: (a) NVDA/JAWS interact with all Windows screen content via UI Automation and MSAA -- Luminos's overlay window will appear in the accessibility tree and produce nonsensical output if read; (b) DXGI Desktop Duplication may conflict with screen reader capture; (c) keyboard hook conflicts between Luminos hotkeys and NVDA/JAWS modifier combinations; (d) testing requires licensed JAWS ($926/year subscription).

**Mitigation:**
1. Begin coexistence research in Phase 2 (not Phase 4) to understand the Windows accessibility landscape early.
2. Contact NV Access (NVDA developers) to discuss interoperability.
3. Mark overlay with `UIA_IsContentElementPropertyId = false` and `UIA_IsControlElementPropertyId = false` so screen readers ignore it.
4. Budget for JAWS license ($926/year).
5. Define specific coexistence test scenarios: "user runs NVDA, activates Luminos, tabs through a web form -- NVDA reads correct content, not the overlay."

**Contingency:** Ship Windows with documented limitation: "pause Luminos when using a screen reader." The long-term fix may require an NVDA add-on for deeper integration.

**Detection:** Manual coexistence testing from the first week of E17. If conflicts emerge early, they reshape the Windows backend architecture.

---

### RISK-016: wgpu Backend Compatibility Across Platforms

| Field | Value |
|-------|-------|
| **Category** | Platform |
| **Likelihood** | Medium (2) |
| **Impact** | Medium (2) |
| **Score** | **4 -- Monitor** |
| **Phase** | Phase 0+ (each platform introduces new surface combinations) |
| **Status** | Open |
| **Sources** | ARCH-004, PERF-010 |

**Description:** wgpu translates WebGPU to Vulkan (Linux), Metal (macOS), DX12 (Windows), and GL ES (fallback). Platform-specific gaps include: (a) `PresentMode::Mailbox` unavailable on macOS Metal, AMD/Intel X11 Vulkan, and OpenBSD [doc-03 Section 8.1]; (b) `CompositeAlphaMode::PreMultiplied` not supported on all drivers, breaking lens mode transparency [doc-03 Section 9.2]; (c) sRGB surface format fallback may produce incorrect color [doc-03 Section 9.2]; (d) on very old integrated GPUs (Intel HD 4000-era), the 16-tap bicubic shader may exceed texture sampling throughput.

**Mitigation:**
1. Query `surface.get_capabilities()` and disable unsupported modes in the UI.
2. Fall back to `CompositeAlphaMode::PostMultiplied` or `Opaque` with a warning log when `PreMultiplied` is unavailable.
3. Retain bilinear interpolation as "performance mode" fallback. Auto-degrade per [doc-06 Section 2.5] when P99 > 33ms.
4. Use `downlevel_webgl2_defaults` [doc-03 Section 9.1] for widest hardware compatibility.

**Contingency:** Display a system dialog explaining GPU incompatibility and suggesting driver updates if no adapter is found (`RenderError::NoAdapter`). Exit gracefully.

**Detection:** GPU compatibility test suite querying surface capabilities on each platform. CI tests with `WGPU_BACKEND=gl` on Mesa llvmpipe. Visual comparison tests between Vulkan and GL output.

---

## 7. Security and Privacy Risks

### RISK-017: Screen Content and TTS Text Leakage via Logs and GPU Memory

| Field | Value |
|-------|-------|
| **Category** | Privacy |
| **Likelihood** | Medium (2) |
| **Impact** | High (3) |
| **Score** | **6 -- Monitor** |
| **Phase** | Phase 0 (capture), Phase 2 (TTS text) |
| **Status** | Open |
| **Sources** | SEC-001, SEC-002, SEC-015, SEC-019 |

**Description:** Multiple data channels carry sensitive user content:

1. **Screen capture pixels:** `CaptureFrame` holds raw screen pixels (passwords, banking, medical records). The struct has `#[derive(Debug, Clone)]` [doc-02 Section 3.2], meaning `{:?}` formatting dumps megabytes of pixel data. AI-agent-driven development increases the probability of boilerplate debug logging.

2. **TTS text:** `SpeechRequest.text` contains text from accessibility APIs and clipboard, flowing through `TextPreprocessor`, `EspeakSubprocess`, `SherpaInference`, and `HighlightEvent` -- each a potential logging point.

3. **GPU texture residuals:** Four GPU textures contain screen content. On exit, wgpu drops textures, but GPU drivers may not zero memory. On integrated GPUs, GPU memory is shared with system RAM.

4. **Accessibility API text:** `FocusChangedEvent.label` contains accessible names from any running application [doc-02 Section 3.3].

[doc-06 Section 6.1] prohibits logging screen capture data or recognized text, but enforcement is by convention only.

**Mitigation:**
1. Implement custom `Debug` for `CaptureFrame` that prints metadata only (width, height, stride, format), omitting the `data` field.
2. Implement custom `Debug` for `SpeechRequest`, `Sentence`, `HighlightEvent` that redacts the `text` field: `text: "[REDACTED len=42]"`.
3. Add CI grep scan for `{:?}` formatting of capture/text types in log macro invocations.
4. On shutdown, write zeroes to all GPU textures via `Queue::write_texture` before dropping.
5. Clear accessibility text from memory after TTS processing completes.
6. ESLint/clippy rules preventing text content in diagnostic bundles [doc-06 Section 6.3].

**Contingency:** If pixel/text data is found in logs, immediately rotate and securely delete log files. Audit all log statements touching sensitive types.

**Detection:** CI grep scan for sensitive types in log macros. Periodic manual review of log output at all levels. Memory analysis during development.

---

### RISK-018: espeak-ng Subprocess Security (Injection and PATH Hijacking)

| Field | Value |
|-------|-------|
| **Category** | Security |
| **Likelihood** | Medium (2) |
| **Impact** | High (3) |
| **Score** | **6 -- Monitor** |
| **Phase** | Phase 2 |
| **Status** | Open |
| **Sources** | SEC-003, SEC-004 |

**Description:** The espeak-ng subprocess receives text via stdin [doc-04 Section 5.2] from three sources: accessibility APIs, clipboard, and OCR (Phase 3). Two attack vectors exist:

1. **Command injection:** espeak-ng's `--stdin` mode processes IPA control codes and SSML-like markup. A malicious application could place crafted text in a UI element label or clipboard. espeak-ng has historical CVEs for buffer overflows (CVE-2023-49990 through CVE-2023-49994). While the subprocess is isolated, it runs as the same OS user and could access user files.

2. **PATH hijacking:** On Linux .deb/.rpm installations where espeak-ng is a system dependency, the binary is resolved from PATH. An attacker with write access to a directory earlier in PATH could substitute a malicious binary that receives all TTS text.

**Mitigation:**
1. Implement text sanitization per [doc-06 Section 3.5]: strip ASCII control characters (0x00-0x1F except \n), reject NUL bytes, cap at 10,000 characters, strip SSML tags.
2. On Linux, run espeak-ng under a seccomp filter (via `Command::pre_exec`) restricting syscalls to read/write/mmap/exit.
3. For bundled distributions, always use the bundled binary path. For system installs, resolve at absolute path (`/usr/bin/espeak-ng`), never via PATH.
4. Log the resolved espeak-ng path at `info` level on first use.
5. Pin espeak-ng version and monitor for new CVEs.

**Contingency:** If an espeak-ng vulnerability is disclosed, release a patch restricting the vulnerable input pattern. Fall back to platform-native TTS [doc-04 Section 12] if espeak-ng must be disabled. If binary substitution is detected, disable TTS and notify the user.

**Detection:** Fuzz testing of `EspeakSubprocess::phonemize()` with malformed UTF-8, overlapping byte sequences, and known CVE reproduction inputs. `check_espeak_available` IPC command returns path and version for user audit.

---

### RISK-019: ONNX Model Supply Chain Integrity

| Field | Value |
|-------|-------|
| **Category** | Supply Chain |
| **Likelihood** | Low (1) |
| **Impact** | Critical (4) |
| **Score** | **4 -- Monitor** |
| **Phase** | Phase 2 |
| **Status** | Open |
| **Sources** | SEC-006, SEC-007, BUILD-023 |

**Description:** Voice models are ONNX files loaded by sherpa-onnx via ONNX Runtime in-process [doc-04 Section 6]. ONNX models can contain custom operators invoking native code. The model download protocol [doc-08 Section 7.3] uses SHA-256 checksums against a manifest, but the manifest itself is served over HTTPS without independent cryptographic signing. If the manifest server is compromised, an attacker can supply both a malicious model and a matching checksum. Unlike espeak-ng (subprocess-isolated), ONNX inference runs in-process -- a malicious model gains access to screen capture buffers, GPU textures, and all application state.

**Mitigation:**
1. Hard-code SHA-256 checksums for officially supported models in the application binary (not just the downloadable manifest). The manifest provides download URLs; the binary provides trusted checksums.
2. Sign the model manifest with the same Ed25519 key used for Tauri updater signing [doc-08 Section 9.6]. Verify the signature before trusting URLs or checksums.
3. Disable ONNX Runtime custom operator loading if sherpa-onnx exposes this configuration.
4. Re-verify model file integrity at every load (not just after download).
5. Pin model manifest URL to a project-controlled domain. Consider certificate pinning.

**Contingency:** If a malicious model is distributed, revoke its checksum in a signed manifest update and issue a security advisory. Release a patch rejecting the compromised model file.

**Detection:** File integrity monitoring: re-verify SHA-256 at load time. Log model hash at `info` level on every load. Community verification (open-source: anyone can verify checksums).

---

### RISK-020: Tauri Webview and WebkitGTK Attack Surface

| Field | Value |
|-------|-------|
| **Category** | Security |
| **Likelihood** | Low (1) |
| **Impact** | High (3) |
| **Score** | **3 -- Accept** |
| **Phase** | Phase 0+ |
| **Status** | Accepted |
| **Sources** | SEC-005, SEC-017 |

**Description:** The control panel uses a Tauri 2.0 webview. On Linux, this means WebkitGTK -- a large C++ codebase with multiple annual CVEs. Two vectors: (a) XSS via IPC response injection if user-controlled text (profile names, voice names, error messages) is rendered as HTML without escaping; (b) WebkitGTK memory corruption if the webview processes malicious content.

**Rationale for Acceptance:** (a) React's default JSX rendering escapes HTML by default, and the control panel loads no external content -- all assets are bundled. (b) The Tauri capability system [doc-06 Section 3.4] restricts webview access. (c) CSP headers (`default-src 'self'; script-src 'self'`) block inline scripts and external resources. (d) `shell:allow-open` opens links in the system browser, not the webview. The residual risk is low given these layered defenses.

**Mitigation:**
1. ESLint rule banning `dangerouslySetInnerHTML` in the control panel codebase.
2. CSP configured in `tauri.conf.json`.
3. WebkitGTK declared as a system dependency in packages, ensuring users receive security updates via their OS package manager.
4. Validate imported profile JSON against `ProfileDocument` Zod schema server-side in Rust.

**Detection:** ESLint rule enforcement in CI. Monitor WebkitGTK security advisories.

---

### RISK-021: ONNX Runtime FFI Memory Safety Boundary

| Field | Value |
|-------|-------|
| **Category** | Security |
| **Likelihood** | Low (1) |
| **Impact** | High (3) |
| **Score** | **3 -- Accept** |
| **Phase** | Phase 2 |
| **Status** | Open |
| **Sources** | SEC-020 |

**Description:** The sherpa-onnx runtime is a C++ library accessed via FFI through `sherpa-rs` [doc-04 Section 6.1]. The FFI boundary bypasses Rust's memory safety guarantees. Bugs in ONNX Runtime, sherpa-onnx, or `sherpa-rs` could lead to memory corruption, use-after-free, or buffer overflows in the Luminos process. Unlike espeak-ng (subprocess), ONNX inference is in-process.

**Mitigation:**
1. Monitor sherpa-onnx and ONNX Runtime for security advisories. Update promptly.
2. Run ONNX inference on a dedicated thread [doc-04 Section 6.4] with a `Mutex` around the `OfflineTts` instance.
3. Use AddressSanitizer (ASAN) during development and CI integration testing.
4. Consider process-level isolation for ONNX inference in a future phase (similar to espeak-ng subprocess).

**Contingency:** If a memory safety bug is found, disable neural TTS and fall back to platform-native TTS until patched.

**Detection:** ASAN in CI integration tests. Monitor ONNX Runtime GitHub for security issues. Investigate segfaults or memory corruption reports for FFI-related causes.

---

## 8. Licensing and Compliance Risks

### RISK-022: GPLv3 Dependency License Compatibility

| Field | Value |
|-------|-------|
| **Category** | Licensing |
| **Likelihood** | Medium (2) |
| **Impact** | High (3) |
| **Score** | **6 -- Monitor** |
| **Phase** | Phase 0+ (ongoing) |
| **Status** | Mitigating |
| **Sources** | SEC-008, SEC-009 |

**Description:** Luminos is GPLv3-only [doc-06 Section 4.1]. All current dependencies are confirmed GPLv3-compatible [doc-06 Section 4.2]. Two ongoing risks:

1. **New incompatible dependencies:** Future dependency additions could introduce `GPL-2.0-only` (NOT compatible with GPLv3, no "or later" clause), proprietary, or SSPL-licensed transitive dependencies. AI agents adding dependencies may not check licenses.

2. **Piper model license drift:** The Piper project has moved to `OHF-Voice/piper1-gpl` (GPL-3.0) [doc-06 Section 4.2]. Post-fork model weights may be GPL-licensed rather than MIT. Using GPL-3.0 models is compatible with GPLv3 Luminos, but changes redistribution terms and constrains future license flexibility.

**Mitigation:**
1. `cargo deny check licenses` runs in CI on every push [doc-06 Section 4.3]. Restrictive allowlist; any unlisted license is automatically denied. **This is already operational.**
2. Prohibit adding dependencies without a green `cargo deny` CI check.
3. Track Piper model provenance with explicit `license` field in `ModelManifest` [doc-08 Section 7.3].
4. Periodically audit transitive dependencies (`cargo tree --format '{p} {l}'`).
5. Add `GPL-2.0-only` to `deny.toml` with a comment explaining why it is NOT in the allowlist.

**Contingency:** If an incompatible dependency is discovered post-release, issue a patch removing or replacing it. If deep in the tree, vendor the last compatible version while finding an alternative.

**Detection:** `cargo deny` (automated, every push). Quarterly manual license audit of the full dependency tree.

---

### RISK-023: Regulatory and Self-Accessibility Compliance

| Field | Value |
|-------|-------|
| **Category** | Compliance |
| **Likelihood** | Medium (2) |
| **Impact** | High (3) |
| **Score** | **6 -- Monitor** |
| **Phase** | Phase 0+ |
| **Status** | Open |
| **Sources** | PROJ-010 |

**Description:** The European Accessibility Act (EAA, effective June 2025) and WCAG 2.1 AA requirements affect institutional deployment. If Luminos's own control panel is not WCAG 2.1 AA compliant, institutional buyers (Robert persona) cannot deploy without a compliance gap. The Product Strategy commits to "WCAG 2.2 AA compliance (own UI): 100%" for v1.0 [Product Strategy Section 10.3]. However: (a) accessibility testing is largely manual ("keyboard navigation + Orca screen reader tests") [doc-07]; (b) no automated WCAG audit tool applies to the native winit/wgpu overlay (not a web page); (c) the Wayland consent dialog chicken-and-egg is a structural accessibility barrier in the product's first-run experience.

**Mitigation:**
1. Add automated axe-core testing to every control panel component from Phase 0 (E4). Treat any violation as a CI failure.
2. Define limited WCAG scope for the overlay: it is a visual rendering surface without interactive elements; standard WCAG criteria (focus management, form labels) do not apply. WCAG compliance focuses on the control panel (web UI, fully auditable by axe-core).
3. Add a minimal first-run onboarding to Phase 0/1: a high-contrast, large-text welcome screen explaining basic keybindings.
4. For institutional deployments requiring formal certification, budget for a third-party accessibility audit ($5K-$15K) before Phase 4.

**Contingency:** If axe-core violation count increases across releases, accessibility is regressing -- halt feature work until resolved. If formal WCAG certification is required, fund it from early support contract revenue.

**Detection:** axe-core violation count in CI (zero tolerance). Monitor institutional pilot feedback for compliance concerns.

---

## 9. Build and Distribution Risks

### RISK-024: Binary Size Budget with ONNX Runtime

| Field | Value |
|-------|-------|
| **Category** | Build |
| **Likelihood** | High (3) |
| **Impact** | High (3) |
| **Score** | **9 -- Mitigate** |
| **Phase** | Phase 0 (validation), Phase 2 (materialized) |
| **Status** | Open |
| **Sources** | BUILD-002, BUILD-013 |

**Description:** The ONNX Runtime (bundled via `sherpa-rs`) is a large C++ library. On Linux x86_64, `libonnxruntime.so` alone is typically 40-80MB depending on build configuration and execution providers. The 50MB binary budget [doc-06 Section 2.1, doc-08 Section 4.5] leaves ~15-20MB for Rust application logic + Tauri webview assets + wgpu. With ONNX Runtime statically linked (`lto = "fat"` in `dist` profile), the binary could reach 60-80MB, exceeding the 60MB binary CI hard failure threshold [doc-08 Section 4.5]. Additionally, AppImage bundles all shared libraries including WebkitGTK (~40-60MB), making AppImages 100-130MB total [doc-08 Section 8.4].

**Mitigation:**
1. Investigate `sherpa-rs` build options to disable unnecessary ONNX Runtime execution providers (CUDA, TensorRT, NCCL) -- only CPU provider is needed.
2. Benchmark stripped binary sizes in E1 with a minimal sherpa-rs integration to establish a baseline.
3. Consider dynamic linking of `libonnxruntime.so` (shipped as sidecar) to exempt it from the binary target.
4. Establish a separate AppImage size budget (~150MB) distinct from the binary budget.
5. Use `cargo bloat --profile dist -p luminos-app` to identify largest contributing crates.

**Contingency:** Ship ONNX Runtime as a dynamic library sidecar. Adjust the 50MB target to apply to the Luminos binary alone with a separate ~30MB budget for ONNX Runtime. Total package budget becomes ~80MB excluding voice models. Update [doc-06 Section 2.1] and [doc-08 Section 4.5].

**Detection:** CI Stage 5 binary size check. AppImage size checked separately (warn at 120MB, fail at 200MB).

---

### RISK-025: Code Signing Key Proliferation and Custody

| Field | Value |
|-------|-------|
| **Category** | Build / Infrastructure |
| **Likelihood** | Medium (2) |
| **Impact** | Critical (4) |
| **Score** | **8 -- Mitigate** |
| **Phase** | Phase 0 (immediate) |
| **Status** | Open |
| **Sources** | BUILD-006, BUILD-021, SEC-013 |

**Description:** Luminos requires at least 6 distinct signing keys [doc-08 Section 9]: GPG (Linux packages), Apple Developer ID (macOS), OpenBSD signify, Windows Authenticode, Tauri Ed25519 (auto-updater), and GPG for SHA256SUMS. In Phase 0-1, all keys are in GitHub Actions encrypted secrets managed by the project founder [doc-08 Section 9.7] -- a single point of failure. If the founder's account is compromised, all keys are exposed. The Ed25519 updater public key is compiled into the binary [doc-08 Section 9.6], meaning key rotation requires users to manually download a new binary (the auto-updater cannot deliver its own key rotation). Additionally, the Apple Developer account ($99/year) and Authenticode certificate ($200-400/year) are ongoing costs for an unfunded project.

**Mitigation:**
1. Implement a two-person key generation ceremony with Shamir's Secret Sharing for the GPG master key and Ed25519 updater key. Distribute shares to 2-3 trusted individuals.
2. Store only signing subkeys (not master keys) in GitHub Actions secrets.
3. Restrict access using GitHub environment protection rules requiring manual approval for the release environment.
4. Establish the Apple Developer account as an organization account (not individual) from day one.
5. Document key rotation and revocation procedures before the first release.
6. Phase 2+: migrate to Cloud KMS/HSM with non-extractable private keys [doc-08 Section 9.7].

**Contingency:** If a key is compromised: revoke via the respective mechanism, generate new keys, publish a signed security advisory using a different key, release a new binary for Ed25519 rotation. For account cost issues, ship unsigned builds (Gatekeeper/SmartScreen warnings can be bypassed).

**Detection:** CI verifies all release artifacts have valid signatures. Monitor GitHub audit logs for unauthorized secret access. Calendar reminders for account/certificate renewal.

---

### RISK-026: espeak-ng Bundling and Version Skew Across Platforms

| Field | Value |
|-------|-------|
| **Category** | Distribution |
| **Likelihood** | Medium (2) |
| **Impact** | High (3) |
| **Score** | **6 -- Monitor** |
| **Phase** | Phase 1 (E10) |
| **Status** | Open |
| **Sources** | BUILD-009, BUILD-019 |

**Description:** Different platforms use different espeak-ng bundling strategies [doc-08 Section 6.1]: system dependency for .deb/.rpm/OpenBSD, bundled binary for AppImage/Flatpak/macOS/Windows. This means Luminos may interact with different espeak-ng versions depending on installation method. espeak-ng phoneme output varies between versions -- version-dependent differences cause voice quality variation that is difficult to diagnose. On Windows, espeak-ng is obtained from an MSI on GitHub releases [doc-07 Section 4.7] -- the espeak-ng project has inconsistent release practices (1.51 was tagged in 2020), and the MSI may not be updated for future versions.

**Mitigation:**
1. Pin expected espeak-ng version and log a warning if the system version differs.
2. For bundled platforms, use a consistent version across all platforms.
3. Add espeak-ng phoneme regression tests in CI: run 50 reference sentences through `espeak-ng --ipa` and assert output matches known-good reference.
4. For Windows, mirror the espeak-ng MSI on Luminos's GitHub Releases with SHA-256 verification. Consider building from source as an alternative.

**Contingency:** Accelerate misaki G2P evaluation to reduce dependency on espeak-ng. For non-English, espeak-ng remains necessary until misaki supports those languages. As a last resort, build espeak-ng from source in CI for all platforms.

**Detection:** CI phoneme regression test. espeak-ng MSI download CI step validates SHA-256 checksum.

---

### RISK-027: CI Pipeline Performance and Platform Coverage Gaps

| Field | Value |
|-------|-------|
| **Category** | CI/CD |
| **Likelihood** | High (3) |
| **Impact** | Medium (2) |
| **Score** | **6 -- Monitor** |
| **Phase** | Phase 0+ |
| **Status** | Open |
| **Sources** | BUILD-003, BUILD-004, BUILD-010, BUILD-014, BUILD-018 |

**Description:** Multiple CI limitations compound:

1. **Wall-clock time:** Full pipeline estimated at ~62 minutes on a single runner [doc-07 Section 4.1]. Release builds take 80+ minutes across platforms.
2. **No OpenBSD runners:** GitHub Actions has no OpenBSD images. OpenBSD compilation is unverified for 9+ months until Phase 3.
3. **macOS limitations:** Runners do NOT grant Screen Recording permission. macOS capture tests are manual-only.
4. **macOS cost:** macOS runners are 10x multiplier on GitHub Actions minutes. ~20 PRs/week could exhaust free-tier macOS minutes within the first week of active Phase 2 development.
5. **Self-hosted runners:** Benchmark runners require hardware procurement (~$500-800), provisioning, and ongoing maintenance.

**Mitigation:**
1. Maximize parallelization: Stages 1-3 run concurrently. Use Rust build caching (`sccache`) aggressively.
2. Path-based filtering: skip macOS CI for non-platform-specific changes. Skip frontend stages for Rust-only PRs.
3. Run macOS CI only on PRs targeting `main`. Apply for GitHub's open-source CI sponsorship.
4. For OpenBSD, extract shared X11 code with `#[cfg(any(target_os = "linux", target_os = "openbsd"))]` and add periodic manual cross-compilation checks.
5. Use a cloud VM (Hetzner, ~$45/month) for benchmark runner instead of physical hardware.

**Contingency:** If CI time exceeds 45 minutes for PRs, move to two-tier: fast checks (lint, unit tests, ~5 min) block merge; full tests run post-merge. If macOS minutes are exhausted, disable macOS CI and gate on self-hosted runner or manual testing.

**Detection:** Track CI duration per workflow. Set 45-minute warning for PR workflows, 120-minute for release. Monitor GitHub Actions usage dashboard.

---

### RISK-028: Tauri Build System Constraints

| Field | Value |
|-------|-------|
| **Category** | Build |
| **Likelihood** | High (3) |
| **Impact** | Medium (2) |
| **Score** | **6 -- Monitor** |
| **Phase** | Phase 1 (E9) |
| **Status** | Open |
| **Sources** | BUILD-001, BUILD-005, BUILD-015 |

**Description:** Three Tauri build limitations:

1. **dist profile:** `cargo tauri build` always uses the `release` profile internally [doc-08 Section 8.1]. The two-step workflow (build with `--profile dist`, then let Tauri bundle) relies on undocumented behavior -- Tauri may rebuild with its own `release` profile, defeating `dist` optimizations (`opt-level = "z"`, `lto = "fat"`, `codegen-units = 1`).

2. **Flatpak/Snap:** Tauri's native bundler does not produce Flatpak or Snap packages [doc-08 Section 8.1]. Both require separate build infrastructure (flatpak-builder YAML, snapcraft YAML), separate CI jobs, and separate submission processes.

3. **specta bindings:** `tauri-specta` generates `ui/src/ipc/bindings.ts` during Rust compilation. The committed file can accumulate merge conflicts if two PRs modify IPC types simultaneously [doc-08 Section 5.3].

**Mitigation:**
1. Validate the two-step dist build workflow empirically in E1. If Tauri overrides, copy the `dist` binary to `target/release/` or override `[profile.release]` in CI with dist settings.
2. Defer Flatpak/Snap to a dedicated follow-up after E9 with 2-week estimate. Prioritize Flatpak over Snap.
3. Add a pre-commit hook for binding regeneration. Consider CI-only generation instead of committed bindings.

**Contingency:** For dist profile: set `[profile.release]` identical to `[profile.dist]` during release CI runs. For Flatpak/Snap: focus on .deb, .rpm, and AppImage if maintenance cost is too high.

**Detection:** CI Stage 8 checks bundled binary size. If >10% larger than standalone `--profile dist` build, the override is not working. CI bindings check per [doc-08 Section 5.3].

---

### RISK-029: Auto-Update Mechanism Complexity per Install Method

| Field | Value |
|-------|-------|
| **Category** | Distribution |
| **Likelihood** | Medium (2) |
| **Impact** | High (3) |
| **Score** | **6 -- Monitor** |
| **Phase** | Phase 1 (E9) |
| **Status** | Open |
| **Sources** | BUILD-017, PROJ-019 |

**Description:** The Tauri auto-updater must be disabled for .deb/.rpm/Flatpak/Snap (updated via package managers) and enabled only for AppImage/macOS/.dmg/Windows installers [doc-08 Section 10.2]. Runtime detection of the installation method is heuristic (checking for package metadata files, environment variables, AppImage mount points) and may fail in edge cases. A false positive (updater enabled on a package-managed install) could overwrite the package-managed binary, breaking future `apt upgrade`. Additionally, Phase 1's first external release (v0.1.0-alpha) only provides auto-update for AppImage -- .deb/.rpm users receive no update notifications.

**Mitigation:**
1. Conservative detection: only enable the updater when a positive indicator is found (`APPIMAGE` env var, `.app` bundle). Default to disabled.
2. Add a user-visible setting in the control panel to override auto-update behavior.
3. For .deb/.rpm, set a `LUMINOS_UPDATE_CHANNEL=system` marker file that the application reads at startup.
4. Include a "Check for Updates" button from Phase 1.
5. Prioritize APT/DNF repository hosting in Phase 1 for system update mechanisms.

**Contingency:** Disable in-app updater for Linux entirely. Rely on package manager updates plus manual download for AppImage. Keep in-app updater only for macOS and Windows.

**Detection:** Integration test: install via each package format, verify updater status matches expected state.

---

## 10. Ecosystem, Dependency, and Project Risks

### RISK-030: wgpu/winit/Tauri Major Version Upgrade Cascade

| Field | Value |
|-------|-------|
| **Category** | Ecosystem |
| **Likelihood** | High (3) |
| **Impact** | High (3) |
| **Score** | **9 -- Mitigate** |
| **Phase** | Phase 0+ (continuous) |
| **Status** | Open |
| **Sources** | ARCH-008, ARCH-010, BUILD-011 |

**Description:** Luminos depends on three rapidly-evolving, pre-1.0 ecosystems: wgpu (v28.0.0, major releases every 3-6 months), winit (v0.30.13, breaking changes between minors), and Tauri (v2.x, plugin ecosystem changes). They share a critical interface: the `raw-window-handle` crate. Historically, wgpu and winit must be updated in lockstep when `raw-window-handle` changes version (this happened between winit 0.28 and wgpu 0.18, causing weeks of ecosystem churn). Over the 20-month project timeline, at least one major version bump is virtually certain for each. Additionally, Tauri's internal `tao` (their winit fork) may conflict with mainline winit in the dependency tree.

**Mitigation:**
1. Pin exact versions in `[workspace.dependencies]` [doc-08 Section 2.2]. Upgrade deliberately, not automatically.
2. Isolate winit and wgpu usage to specific files (`renderer.rs`, `pipeline.rs`, `texture.rs`, `overlay.rs`, `main.rs`). No other crate imports winit or wgpu directly.
3. Do NOT adopt new major versions during active epic development. Schedule updates as explicit maintenance windows between phases.
4. When upgrading, create a dedicated branch and run the full CI suite before merging. Upgrade winit and wgpu simultaneously.
5. Monitor changelogs and subscribe to Matrix/Discord channels for pre-release announcements.
6. Maintain a compatibility matrix documenting known-good version combinations.

**Contingency:** Remain on pinned versions indefinitely if upgrades introduce regressions. Pre-1.0 crates lack backward-compatibility guarantees, but specific versions continue working. Worst case: miss new features or security fixes that can be backported.

**Detection:** `cargo update --dry-run` periodically. CI compilation failure signals incompatible upgrade. Weekly `cargo outdated` report (non-blocking). Dependabot alerts for security advisories.

---

### RISK-031: Single-Maintainer Crate Dependencies

| Field | Value |
|-------|-------|
| **Category** | Ecosystem |
| **Likelihood** | Medium (2) |
| **Impact** | Medium (2) |
| **Score** | **4 -- Monitor** |
| **Phase** | Phase 0+ |
| **Status** | Open |
| **Sources** | PROJ-020, SEC-010 |

**Description:** Critical-path crates are maintained by small teams or individuals:

- **xcap** (v0.9.1, nashaofu): 85K downloads/month, primary capture library for 3 of 5 platforms. Single-maintainer.
- **sherpa-rs** (v0.6.8, thewh1teagle): Rust bindings for sherpa-onnx. Pre-1.0, potentially breaking API changes.
- **atspi** (Odilia project): Linux AT-SPI2 bindings. Maintained by a small OSS team.
- **rdev** (v0.5): Global input monitoring. Known Wayland issues. Small team.

A compromise of any of these (account takeover, malicious release) would introduce malicious code into the Luminos binary. `sherpa-rs` is particularly sensitive: its `build.rs` executes at build time and the FFI layer bypasses Rust safety at runtime.

**Mitigation:**
1. Trait boundaries enable localized replacement of any dependency.
2. `cargo audit` runs in CI on every push [doc-06 Section 3.6]. `Cargo.lock` pinning with reviewed update PRs.
3. Contribute upstream to build relationships and ensure crates evolve compatibly.
4. For xcap: the planned `x11rb` XShm backend (E6) also serves as an xcap replacement for X11.
5. For sherpa-rs: the documented fallback is raw sherpa-onnx C FFI bindings [TECH_STACK_EVALUATION.md Section 6.1].
6. Consider `cargo-vet` for tracking dependency audit status.
7. Monitor repository activity quarterly. If a critical dependency has zero commits and >10 open unresponded issues in 3 months, escalate.

**Contingency:** Fork under the Luminos organization and maintain the minimum viable subset. Forking cost: xcap (moderate), sherpa-rs (low -- thin wrapper), atspi (moderate), rdev (moderate).

**Detection:** `cargo audit` (automated). GitHub notifications for security advisories. Monthly automated repository activity check.

---

### RISK-032: TTS Ecosystem Volatility (Kokoro, sherpa-onnx)

| Field | Value |
|-------|-------|
| **Category** | Ecosystem |
| **Likelihood** | Medium (2) |
| **Impact** | Medium (2) |
| **Score** | **4 -- Monitor** |
| **Phase** | Phase 2 |
| **Status** | Open |
| **Sources** | PROJ-008, PERF-012, PROJ-007 |

**Description:** The TTS stack is a three-layer dependency: Kokoro-82M (model) delivered via sherpa-onnx (runtime) with sherpa-rs (Rust bindings). Each layer evolves independently. Risks: (a) the TTS model landscape evolves rapidly -- a significantly better model requiring a different runtime would require replacing the entire inference layer; (b) Kokoro's 8-language support is limited versus the 30+ language target; (c) espeak-ng upstream has irregular maintenance history -- critical bugs propagate with no recourse except forking; (d) model loading takes 500-1000ms [doc-04 Section 8.3], and loading failures fall back to platform-native TTS with 2-3 second delay.

**Mitigation:**
1. The `TtsEngine` trait [doc-02 Section 3] abstracts TTS backends. New engines are localized additions.
2. Evaluate the `ort` crate (v2.0.0-rc.12, ONNX Runtime bindings) as a more mature alternative to sherpa-rs.
3. Monitor TTS landscape quarterly (Hugging Face leaderboards, r/LocalLLaMA).
4. For espeak-ng maintenance risk: evaluate misaki G2P during Phase 2 as a replacement for English phonemization.
5. Display "Loading voice..." indicator during model loading. Pre-validate model integrity before loading.
6. Keep a warm espeak-ng subprocess to eliminate spawn overhead.

**Contingency:** Replace sherpa-rs with direct FFI bindings to sherpa-onnx C API, or replace the inference layer with the `ort` crate. If espeak-ng becomes unmaintained, fork and maintain only the G2P subsystem. Platform-native TTS provides degraded-but-functional fallback.

**Detection:** Track sherpa-rs release cadence. Monitor espeak-ng GitHub activity. If either has no release in 6 months, begin evaluating alternatives.

---

### RISK-033: Scope Ambition Exceeds Realistic Capacity

| Field | Value |
|-------|-------|
| **Category** | Project |
| **Likelihood** | High (3) |
| **Impact** | High (3) |
| **Score** | **9 -- Mitigate** |
| **Phase** | All phases |
| **Status** | Open |
| **Sources** | PROJ-001, PROJ-012 |

**Description:** The roadmap defines 20 epics across 5 phases spanning 20 months, targeting 4 platforms, 3 magnification modes, neural TTS in 8+ languages, font re-rendering, OCR, AI image description, plugin architecture, enterprise GPO/MDM, and i18n in 10+ languages [doc-09]. The claimed 10 months of schedule margin is illusory: (a) the staffing model (2-3 AI agents + 1 human tech lead) is aspirational, not resourced; (b) epic estimates systematically undercount integration work (e.g., E12 macOS in 5 weeks, E13 font re-rendering in 5 weeks); (c) phase gate criteria require ALL epic success criteria met, ALL CI green, zero P0 bugs -- a single P0 on any platform blocks the gate. Phase 3-4 collectively contain 8 epics (E13-E20) delivering features that would individually challenge a well-funded team of 10+.

**Mitigation:**
1. Define a **Minimum Viable v1.0** scope: Phases 0-2 (Linux + macOS magnification + TTS) are the core product. Phases 3-4 are aspirational, not committed.
2. Redefine v1.0 as Phase 2 completion, not Phase 4.
3. Insert explicit "scope down" decision points at each phase gate: if a phase took >150% of estimate, reduce next phase scope.
4. Apply strict MoSCoW to Phase 3-4: **Must:** Windows (E17-E18), basic i18n. **Should:** OCR (E14), OpenBSD (E15). **Could:** Font re-rendering (E13, only if research succeeds). **Won't (v1.0):** Plugins (E19), enterprise (E20).
5. Time-box E13 research at 2 weeks with go/no-go criteria.

**Contingency:** Ship v1.0 as Linux + macOS + Windows with magnification + TTS + color filters + cursor enhancement. Windows becomes v2.0 if delayed. Plugin architecture, enterprise features, i18n are v3.0+.

**Detection:** Track epic actual-vs-estimated duration. If any epic exceeds 150%, trigger roadmap review. Phase 0 completion date is the leading indicator.

---

### RISK-034: AI-Agent Development Model Unproven at This Scale

| Field | Value |
|-------|-------|
| **Category** | Project |
| **Likelihood** | High (3) |
| **Impact** | High (3) |
| **Score** | **9 -- Mitigate** |
| **Phase** | All phases |
| **Status** | Open |
| **Sources** | PROJ-002 |

**Description:** The project assumes AI coding agents as primary developers, with the Rust compiler as automated reviewer and a single human tech lead [Product Strategy Section 9.1]. This is unproven for this complexity: (a) deep platform-specific expertise (X11, Wayland, ScreenCaptureKit, AXUIElement, Win32, UI Automation, AT-SPI2) has thin LLM training data; (b) GPU shader development (WGSL for wgpu) has very limited training corpus; (c) accessibility API integration has subtle correctness requirements the compiler cannot catch (race conditions, runloop integration); (d) the "100K lines in 4 weeks" reference was a well-defined port, not greenfield system integration; (e) AI agents struggle with cross-subsystem integration, which dominates Phases 2-4.

**Mitigation:**
1. **Validate in Phase 0** -- it is the cheapest test. If E1-E4 take 2x estimate (6+ months instead of 3), the development model needs revision.
2. Budget for human domain expert contractors for three hardest subsystems: Wayland compositor integration, macOS accessibility APIs, Windows screen reader coexistence.
3. Build comprehensive integration tests early (E1) to catch semantic bugs the compiler misses.
4. Ensure the SDD methodology (STORY.md, DESIGN.md, SUBTASKS.md) provides enough context for AI agents -- validate with a real agent run in Phase 0.

**Contingency:** Shift from AI-primary to AI-assisted development. Recruit 2-3 experienced Rust developers for platform backends. AI handles TypeScript UI, test generation, and well-documented Rust modules.

**Detection:** Measure AI agent story completion rate during Phase 0. If agents require >50% human revision, the model is not working.

---

### RISK-035: Funding Sustainability Gap

| Field | Value |
|-------|-------|
| **Category** | Project |
| **Likelihood** | High (3) |
| **Impact** | High (3) |
| **Score** | **9 -- Mitigate** |
| **Phase** | Phase 0-1 (pre-revenue) |
| **Status** | Open |
| **Sources** | PROJ-006 |

**Description:** Year 1 revenue target ($50K-$150K from grants + donations) is optimistic [Product Strategy Section 12.1]. NVDA generates ~AUD $1.5-2M/year, but only after 18+ years and 250K+ users. Luminos in Year 1 has zero users and no non-profit entity. Grant bodies rarely fund pre-launch projects. Institutional contracts ($2K-$15K/year) require Windows support (Phase 4, Month 15-20). The non-profit foundation (required for grants) takes 3-12 months to establish, competing for the founder's time during critical Phase 1-2 development.

**Mitigation:**
1. Apply for seed funding before writing code: NLnet Foundation (EUR 5K-50K, milestone-based), Microsoft AI for Accessibility ($25K).
2. Use a fiscal sponsor (Open Source Collective via Open Collective) in Year 1 to receive donations without full non-profit infrastructure.
3. Reduce Year 1 revenue target to $20K-$50K (grants only).
4. Plan for founder self-funding or part-time work during Year 1.
5. Prioritize the Windows port as the unlock for institutional revenue.

**Contingency:** If no funding by Phase 2 (Month 9): continue as volunteer/spare-time project (slower timeline), or seek corporate sponsor (Platinum sponsorship, not code ownership -- GPLv3 prevents proprietary forks).

**Detection:** Track grant submissions (target: 3+ by Month 3), decisions, Open Collective donation inflow, GitHub Sponsors sign-ups.

---

### RISK-036: Bus Factor of One (Founder Key-Person Dependency)

| Field | Value |
|-------|-------|
| **Category** | Project |
| **Likelihood** | Medium (2) |
| **Impact** | Critical (4) |
| **Score** | **8 -- Mitigate** |
| **Phase** | Phase 0+ |
| **Status** | Open |
| **Sources** | PROJ-011 |

**Description:** All strategic decisions, signing keys, code review authority, grant applications, and institutional relationships depend on a single founder (BDFL model, [Product Strategy Section 12.2]). Single points of failure: (a) GPG signing keys, (b) Apple Developer account, (c) story/design document authorship, (d) grant body relationships, (e) non-profit formation legal filings.

**Mitigation:**
1. Identify and cultivate 2-3 core maintainers by Phase 1 gate (Month 6). These individuals should have commit rights, release authority, and architectural understanding.
2. Document all key management procedures: signing key backup, Apple account recovery, GitHub admin access, domain registration. Store in encrypted location accessible to trusted backups.
3. Use organizational signing keys (not personal) from the start.
4. Write an "Emergency Succession Plan" describing how the project continues if the founder is unavailable for 3+ months.

**Contingency:** GPLv3 licensing ensures the codebase cannot be locked away -- the community can fork. But forking loses signing keys, domain/trademark, institutional relationships, and grant agreements. The succession plan must cover these assets.

**Detection:** Track the number of individuals with: GitHub admin access, signing key access, Apple/Windows certificate access, domain registration access. If any is 1, the bus factor is unacceptably low.

---

### RISK-037: Adoption Barriers and Contributor Recruitment

| Field | Value |
|-------|-------|
| **Category** | Project |
| **Likelihood** | High (3) |
| **Impact** | Medium (2) |
| **Score** | **6 -- Monitor** |
| **Phase** | Phase 1+ |
| **Status** | Open |
| **Sources** | PROJ-004, PROJ-005 |

**Description:** Two interconnected adoption challenges:

1. **User adoption:** Low-vision users are conservative technology adopters with extremely high switching costs (the tool mediates their entire computer interaction). Clinical referral is the primary channel, but Luminos has zero clinical track record. The Year 1 target of 50,000 downloads and 5,000 MAU is ambitious. Linux-first constrains initial reach (~86% of screen reader respondents use Windows per WebAIM Screen Reader Survey #10 (2024)). The MAU metric requires opt-in telemetry -- but the product philosophy is "no telemetry by default" [Product Strategy Section 5.3].

2. **Contributor recruitment:** The required talent intersection (Rust + accessibility + GPU + platform systems) is extremely narrow. GPLv3 may deter corporate contributors. The SDD methodology creates a high barrier for casual contributors.

**Mitigation:**
1. Redefine Year 1 metrics: 5K-10K downloads, 500-1K MAU, 3-5 institutional pilots.
2. Prioritize 5-10 AT specialist beta testers from Day 1. Iterate based on their feedback.
3. Resolve telemetry contradiction: use proxy metrics (downloads, GitHub stars, forum activity) or opt-in anonymous pings.
4. Lower contribution barrier: "good first issues" without full SDD process (docs, translations, CI improvements).
5. Target Rust accessibility community (Odilia, wgpu, Tauri plugin developers).
6. Create comprehensive CONTRIBUTING.md, architecture walkthrough, and "your first Luminos PR" guide.

**Contingency:** If Year 1 downloads < 1,000, pivot to direct institutional partnerships (3-5 universities or government agencies) as primary channel.

**Detection:** Track downloads per release, GitHub stars, first-time contributor PRs per quarter. If no external contributor merges a PR in 6 months, the process is too heavy.

---

### RISK-038: i18n Technical Debt from Phase 4 Deferral

| Field | Value |
|-------|-------|
| **Category** | Project |
| **Likelihood** | High (3) |
| **Impact** | Medium (2) |
| **Score** | **6 -- Monitor** |
| **Phase** | Phase 0 (infrastructure), Phase 4 (translations) |
| **Status** | Open |
| **Sources** | PROJ-014 |

**Description:** i18n infrastructure is not established until Phase 4 (Month 18-20) [doc-09 E20]. All UI code in Phases 0-3 (18+ months of React development) will use hardcoded English strings. Retrofitting i18n requires: extracting every string into message files, replacing hardcoded strings with i18n function calls, testing with variable-length strings (German is ~30% longer), testing RTL layout (Arabic -- Dr. Fatima persona), and handling pluralization/number/date formatting. This is significantly more expensive than building i18n-ready from Day 1.

**Mitigation:**
1. **Establish i18n infrastructure in Phase 0 (E4):** install `react-intl` or `i18next`, create message file format, wrap all strings in `FormattedMessage` from Day 1. Cost: ~1-2 days setup.
2. English is the only translation in Phases 0-3, but infrastructure ensures every new string is i18n-ready.
3. Add a CI lint rule that detects hardcoded strings in React components not wrapped in i18n functions.
4. Defer actual translations to Phase 4 -- translations become a content task, not a code task.

**Contingency:** If not established in Phase 0, budget 3-4 weeks for a dedicated retrofit epic in Phase 4 (separate from the 4-week E20 estimate).

**Detection:** CI lint rule tracking hardcoded string count. Track count per release -- increasing count indicates growing debt.

---

## 11. Governance and Maintenance

### 11.1 Review Cadence

The risk register is a **living document** reviewed at multiple intervals:

| Trigger | Scope | Participants | Output |
|---------|-------|--------------|--------|
| **Phase gate** | Full register review | Tech lead + core maintainers | Status update for all risks; new risks from completed phase; score adjustments based on evidence |
| **Quarterly** | Score recalibration | Tech lead | Likelihood/impact reassessment based on project progress, ecosystem changes, and new information |
| **Epic completion** | Affected risks only | Epic implementer | Update status, add completion notes, close mitigated risks |
| **Dependency update** | Ecosystem/build risks | Engineer performing update | Verify dependency-related risks are still accurate; flag new risks from breaking changes |
| **Security advisory** | Security risks only | Tech lead | Assess whether advisory affects any documented risk; add new risks if needed |
| **Ad hoc** | Any risk | Any contributor | New risks discovered during implementation; triggered risks needing contingency activation |

### 11.2 How to Add a New Risk

New risks follow this process:

1. **Identify:** Document the risk with a description, affected components, and phase.
2. **Score:** Apply the Likelihood x Impact matrix from Section 2.
3. **Assign ID:** Use the next available `RISK-NNN` sequential ID.
4. **Cross-reference:** Link to the source document section(s) where the risk originates.
5. **Mitigate:** Define at least one mitigation strategy and one contingency plan.
6. **Detect:** Define how the risk will be detected if it materializes.
7. **Submit:** Add the risk to the appropriate category section and update the master summary table (Section 3).

### 11.3 Risk Lifecycle

```
Open --> Mitigating --> Closed
  |         |
  |         +--> Triggered --> (Contingency) --> Closed
  |
  +--> Accepted (with documented rationale)
```

- **Open -> Mitigating:** When the first mitigation action is implemented.
- **Mitigating -> Closed:** When all mitigation actions are verified effective (risk reduced to acceptable level or eliminated).
- **Open/Mitigating -> Triggered:** When the risk event occurs. Document the date, impact, and contingency actions taken.
- **Open -> Accepted:** When the risk is acknowledged but no further mitigation is cost-effective. Document the acceptance rationale and the residual risk level.
- **Any -> Closed:** When the risk is no longer applicable (e.g., the affected component was removed, the dependency was replaced, the phase was descoped).

### 11.4 Completion Notes Convention

When updating a risk's status, add a dated note in the following format appended after the risk's Detection section:

```markdown
**Updates:**
- [2026-MM-DD] Status changed from Open to Mitigating. Implemented custom Debug for CaptureFrame in PR #42.
- [2026-MM-DD] Status changed to Closed. All mitigation actions verified in CI. Residual risk: Low.
```

### 11.5 Integration with Story Development

When creating implementation stories (STORY.md / DESIGN.md / SUBTASKS.md), check this risk register for risks affecting the epic's components:

- **DESIGN.md** must reference relevant risks and document how the design addresses them.
- **SUBTASKS.md** should include specific tasks for implementing risk mitigations where applicable.
- If a story reveals a new risk not in this register, add it during story review.

---

## 12. Phase Risk Heatmap

The following table shows risk concentration by phase, highlighting where the highest-scored risks cluster.

### 12.1 Risks by Phase

**Binning rule:** Each risk is assigned to the phase where its primary impact first materializes. Risks with "P0+" designations appear in the earliest affected phase unless their primary mitigation window is later.

| Phase | Escalate (10-16) | Mitigate (7-9) | Monitor (4-6) | Accept (1-3) | Total |
|-------|-----------------|----------------|---------------|--------------|-------|
| **Phase 0** | -- | RISK-001 (8), RISK-002 (9), RISK-007 (9), RISK-024 (9), RISK-025 (8) | RISK-004 (6), RISK-006 (6), RISK-008 (6), RISK-010 (6), RISK-017 (6), RISK-022 (6), RISK-023 (6), RISK-027 (6), RISK-016 (4) | RISK-020 (3) | 15 |
| **Phase 1** | RISK-011 (12) | RISK-012 (9), RISK-030 (9) | RISK-003 (6), RISK-026 (6), RISK-028 (6), RISK-029 (6), RISK-037 (6), RISK-038 (6) | -- | 9 |
| **Phase 2** | -- | -- | RISK-005 (4), RISK-009 (6), RISK-018 (6), RISK-014 (4), RISK-019 (4), RISK-032 (4) | RISK-021 (3) | 7 |
| **Phase 3** | -- | -- | RISK-013 (6) | -- | 1 |
| **Phase 4** | -- | RISK-015 (8) | -- | -- | 1 |
| **All phases** | -- | RISK-033 (9), RISK-034 (9), RISK-035 (9), RISK-036 (8) | RISK-031 (4) | -- | 5 |

### 12.2 Key Observations

1. **Phase 0 carries the highest risk density** with 5 Mitigate-level risks, 9 Monitor risks, and 1 Accepted risk (15 total). The dual event loop (RISK-001), self-capture (RISK-002), X11 capture (RISK-007), binary size (RISK-024), and key management (RISK-025) all require validation or resolution before Phase 1. Phase 0 is the cheapest phase to fail in -- validating these risks early prevents compounding failure later.

2. **Phase 1 contains the highest-scored single risk** -- font re-rendering feasibility (RISK-011, score 12). The research spike decision in Phase 1 determines whether this risk remains or is descoped. Wayland integration (RISK-012, score 9) is the other Phase 1 critical risk, representing the systemic challenge of Wayland's security model versus accessibility tool requirements.

3. **Structural project risks (RISK-033 through RISK-036) span all phases** and represent the highest aggregate risk to the project. Scope ambition, development model, funding, and bus factor are not technical problems with technical solutions -- they require strategic decisions and process changes.

4. **Phase 2 (macOS + TTS) has moderate risk density** with no Critical risks but 7 Monitor-level risks. This is appropriate for a phase building on a validated Phase 0-1 foundation.

5. **Phase 3-4 risk density is low in the register** because most platform risks are identified early (Phase 0-1) and their mitigations carry forward. The exception is Windows screen reader coexistence (RISK-015, score 8), which is deferred but should have research beginning in Phase 2.

### 12.3 Critical Path Risk Chain

The following risks form a dependency chain on the critical path. Failure of any risk in the chain blocks downstream phases:

```
Phase 0: RISK-001 (dual event loop) --> RISK-002 (self-capture) --> RISK-007 (X11 capture)
    |                                                                    |
    +--> RISK-034 (AI agent model validation)        RISK-024 (binary size -- validate in P0)
    |
    v
Phase 1: RISK-012 (Wayland) --> RISK-011 (font re-rendering go/no-go)
    |
    v
Phase 2: RISK-009 (TTS latency) + RISK-024 (binary size -- materializes with ONNX bundling)
    |
    v
Phase 4: RISK-015 (Windows screen reader coexistence)
```

Note: RISK-024 appears in two phases because binary size is validated in Phase 0 (baseline measurement) but fully materializes in Phase 2 when ONNX Runtime is bundled. RISK-009 and RISK-024 are parallel concerns, not sequential dependencies.

If Phase 0 takes >150% of estimate, the entire downstream timeline must be replanned per RISK-033.

---

## 13. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-18 | Initial risk register (38 risks consolidated from 5 specialist assessments) |
