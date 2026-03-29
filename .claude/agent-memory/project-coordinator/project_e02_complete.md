---
name: Epic E02 Complete
description: Epic E02 (X11 Screen Capture & GPU Magnification) completed 2026-03-28 — 5 stories, 275 tests, full capture-to-present pipeline working
type: project
---

Epic E02 completed 2026-03-28. All 5 stories DONE:

- **001** — X11 Screen Capture Backend (XcbCapture, unmap/remap self-capture prevention)
- **002** — X11 Overlay Window & GPU Surface (X11WindowManager, wgpu device/surface, luminos-types crate)
- **003** — GPU Texture Pipeline (SourceTextureManager, 1.5x over-allocation, stale frame tracking)
- **004** — Magnification Shaders & Viewport (bilinear + bicubic WGSL shaders, viewport calculation)
- **005** — Render Loop, Frame Pacing & CI (Renderer, FrameTimings, CI test-platform/test-gpu jobs)

**Key metrics:** 275 tests passing, 6 crates, clippy pedantic clean across all crates.

**Key architectural decisions carried forward:**
- `RenderError` lives in `error.rs` (single source of truth) — Story 005 reused it instead of creating a duplicate
- All shared types in `luminos_types` crate (zero workspace deps, only serde)
- wgpu v28 API: `multiview_mask`, `depth_slice`, `immediate_size`, `PollType::Wait`, `MipmapFilterMode`
- CI: `test-platform` (Xvfb + picom for X11) and `test-gpu` (Mesa llvmpipe + `--features ci_platform_tests`)
- Integration tests feature-gated behind `ci_platform_tests` for tests requiring X11/Xvfb

**Why:** E02 is the foundation for all subsequent rendering work. E03 (Focus Tracking + Input Monitoring) and E04 (Control Panel Foundation) are next in the roadmap.

**How to apply:** Future epics build on the Renderer, SourceTextureManager, MagnifyPipeline, FrameTimings components. The CI test-platform and test-gpu jobs validate all GPU and X11 code automatically.
