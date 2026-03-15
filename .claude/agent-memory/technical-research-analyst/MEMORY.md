# Technical Research Analyst Memory

## Research Patterns
- For cross-platform desktop app research, check `screenpipe` (github.com/screenpipe/screenpipe) as architecture reference - Tauri v2 + Rust + accessibility APIs + OCR
- **CORRECTION:** `scap` crate does NOT support X11 directly — uses PipeWire (Wayland-era). Use `xcap` (v0.9.1, Apache 2.0) for X11 support.
- `xcap` (github.com/nashaofu/xcap) — cross-platform screen capture with direct X11/XCB support, 85K monthly downloads, 19 dependents
- **CAUTION:** xcap does NOT use XShm for X11 capture (xcb `shm` feature not enabled) — uses slower xcb_get_image path. Fine for small regions (high zoom) but may bottleneck at low zoom.
- `wgpu` is the go-to Rust crate for cross-platform GPU rendering (Vulkan/Metal/DX12), v28.0.0, 17.9M+ downloads
- **CORRECTION:** Piper TTS is archived (Oct 2025), moved to OHF-Voice/piper1-gpl (GPL-3.0). Kokoro TTS is the current quality leader.
- Tauri has known WebkitGTK performance issues on Linux — mitigated by using winit+wgpu for performance-critical windows

## TTS Landscape (March 2026)
- **Kokoro-82M:** Apache 2.0 (model), near-commercial quality, ~327MB fp32 ONNX / ~80MB q4 quantized, uses espeak-ng (GPL) for phonemization but misaki G2P is a viable alternative
- **Piper:** Archived Oct 2025. GPL-3.0 via espeak-ng. Still usable for language breadth (30+ languages)
- **espeak-ng GPL problem:** Affects ALL major offline TTS (Piper, Kokoro, etc.). Use subprocess isolation.
- **sherpa-onnx** (Apache 2.0, 10.8K stars): Unified C/C++/Rust runtime for Kokoro+Piper+KittenTTS+more
- **sherpa-rs** (v0.6.8, MIT): Rust bindings for sherpa-onnx, supports TTS feature flag
- **kokoroxide** (v0.1.5, MIT/Apache 2.0): Pure Rust Kokoro wrapper, BUT links espeak-ng directly (GPL contamination risk)
- **Supertonic:** Ultra-fast (0.006 RTF), 66M params, ONNX, 5 languages only, built-in phonemizer (no espeak-ng?)
- **Misaki:** Released transformer-based G2P for Kokoro (on PyPI, used by Kokoro itself). Supports EN/JA/KO/ZH. Near-term path to eliminating espeak-ng GPL dependency.
- **Supertonic:** OpenRAIL-M license for model weights (use-restriction license, NOT permissive). Code is MIT.

## Docked Window Implementation
- **Linux X11:** `_NET_WM_STRUT_PARTIAL` + `_NET_WM_WINDOW_TYPE_DOCK` (EWMH standard, works on KWin+Mutter)
- **Windows:** AppBar API (`SHAppBarMessage` + `ABM_NEW`) — reserves desktop space like taskbar
- **macOS:** No public API for third-party screen space reservation. Use NSPanel with floating level (overlay, not reservation)

## Key Crate Versions (March 2026)
- winit: v0.30.13, Apache 2.0, 34.3M total downloads — standard window management
- wgpu: v28.0.0, MIT/Apache 2.0 — GPU rendering
- ort: v2.0.0-rc.12, MIT/Apache 2.0 — ONNX Runtime (pre-release but production-recommended)
- xcap: v0.9.1, Apache 2.0 — screen capture with X11 support
- atspi: MIT/Apache 2.0 — pure Rust AT-SPI2 for Linux accessibility (Odilia project)
- windows-capture: MIT — supports both WGC and DXGI Desktop Duplication

## Framework Evaluation Insights
- GTK4: Effectively Linux-only. X11 backend deprecated 2025 for GTK5 removal.
- MAUI: No official Linux support. Avalonia backend experimental as of late 2025.
- Rust egui/iced: Poor accessibility support. Not suitable for accessibility tools.
- Avalonia: Strong .NET alternative, used by JetBrains/GitHub, accessibility support since v11.

## Accessibility Tools Competitive Landscape (March 2026)
- Primary gap: No cross-platform open-source magnifier+TTS exists
- Font re-rendering (xFont/TrueFonts) is the key commercial differentiator
- ZoomText: $905+ perpetual; Fusion: $2,309+; all Windows-only
- NVDA near-parity with JAWS (40.5% vs 37.7% primary)

## Authoritative Sources
- Screen capture APIs: Apple WWDC videos, Microsoft Learn docs, XDG Portal docs
- TTS comparison: portalzine.de quality rankings, sherpa-onnx benchmarks (k2-fsa.github.io)
- Rust GUI survey: boringcactus.com 2025 survey, areweguiyet.com
- Accessibility APIs: freedesktop.org AT-SPI2 wiki, Microsoft UIA docs
- Accessibility tool pricing: https://store.vispero.com/

## Luminos Project Files
- `specs/PRODUCT_STRATEGY.md` — Main strategy document v1.2 (tech stack alignment, 2026-03-14)
- `specs/TECH_STACK_EVALUATION.md` — Technology stack validation report (March 2026, post-audit revision)
- `specs/TECH_STACK_AUDIT_REPORT.md` — Independent audit of the tech stack evaluation

## Audit Lessons Learned
- Always verify crate versions directly against crates.io — versions in search results/lib.rs can be stale
- xcap's xcb dependency features matter: check Cargo.toml for `shm` feature enablement before claiming XShm support
- GPL subprocess isolation legal analysis: FSF FAQ says "intimate semantics" in pipes can make combined work — never claim "High" legal clarity
- misaki G2P is released (PyPI), not "emerging/unreleased" — always check PyPI and HuggingFace for current status
- Supertonic uses OpenRAIL-M (use-restriction) for model weights, not a permissive license — always check model license separately from code license
- When citing download counts, state the retrieval date or use "as of" phrasing — counts change quickly
