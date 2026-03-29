---
name: E02/003-004 Implementation Decisions
description: Key decisions from Stories E02/003 (GPU Texture Pipeline) and E02/004 (Magnification Shaders & Viewport) — wgpu v28 API adaptations, bytemuck, test patterns
type: project
---

E02/003 and E02/004 completed 2026-03-28 with the following decisions:

- **SourceTextureManager API matches DESIGN.md exactly.** Constructor takes `wgpu::Device` by value (Clone/Arc-backed). `upload()` is infallible (wgpu reports errors asynchronously). `record_capture_failure()` warns at 60-frame threshold.

- **Import paths use `luminos_types` directly** instead of DESIGN.md's `luminos_platform::traits::types` re-export path. This is correct — `luminos_types` is the canonical source.

- **`over_allocate()` uses f64 promotion** instead of DESIGN.md's f32. Slightly more precise, identical results for practical display dimensions.

- **6 wgpu v28 API adaptations from DESIGN.md:** `PipelineLayoutDescriptor` uses `immediate_size: 0` (not `push_constant_ranges`), `RenderPipelineDescriptor` uses `multiview_mask` (not `multiview`), `SamplerDescriptor.mipmap_filter` uses `MipmapFilterMode` (not `FilterMode`), `device.poll()` uses `PollType::Wait { submission_index: None, timeout: None }` (not `Maintain::Wait`), `RenderPassDescriptor` requires `multiview_mask` field, `RenderPassColorAttachment` requires `depth_slice` field.

- **bytemuck v1 with derive feature** added as workspace dependency for `MagnifyUniforms` Pod/Zeroable derives.

- **GPU integration test pattern:** Tests create wgpu Instance with GL backend, request adapter with `force_fallback_adapter: true` for Mesa llvmpipe. Graceful skip when no adapter available. Shader output tests render to texture and read back via staging buffer + `map_async`.

- **`create_magnify_pipeline` returns `Result` but always `Ok`:** In wgpu v28, shader validation errors are reported asynchronously via device error callback. The `Result` return type is forward-compatible for future wgpu versions. Technical auditor flagged as MEDIUM (doc-only issue).

**Quality gates:** Code review APPROVED (4 clippy fixes applied), QA APPROVED (243 tests, all 30 ACs verified), Technical audit APPROVED (0 critical, 0 high, 1 medium, 3 low)

**Why:** These decisions affect Story 005 (Render Loop & Frame Pacing) which assembles all components. The wgpu v28 API notes are critical for 005's render pass code.

**How to apply:** Story 005 agents should use the wgpu v28 API patterns from 003/004 (not DESIGN.md code samples which target an older API). Use `luminos_types` imports directly. Follow the GPU integration test pattern for render loop tests.
