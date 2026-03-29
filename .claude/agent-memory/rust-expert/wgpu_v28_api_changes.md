---
name: wgpu v28 API changes vs DESIGN.md
description: Actual wgpu v28 API differs from DESIGN.md assumptions -- request_adapter returns Result, Instance::new takes reference, request_device has no trace_path
type: project
---

wgpu v28.0.0 API differences from the E02 DESIGN.md documents (which were written based on earlier wgpu versions):

1. `Instance::new(&InstanceDescriptor)` -- takes a reference, not owned value
2. `request_adapter()` returns `Result<Adapter, RequestAdapterError>` -- NOT `Option<Adapter>`. Use `map_err` not `ok_or`.
3. `request_device(&DeviceDescriptor)` takes only 1 arg -- NO `trace_path: Option<&Path>` second parameter
4. `DeviceDescriptor` has 6 fields: `label`, `required_features`, `required_limits`, `experimental_features`, `memory_hints`, `trace`. Use `..Default::default()` for the trailing fields.
5. `PipelineLayoutDescriptor` uses `immediate_size: 0` -- NOT `push_constant_ranges: &[]`
6. `RenderPipelineDescriptor` uses `multiview_mask: None` -- NOT `multiview: None`
7. `SamplerDescriptor.mipmap_filter` is `wgpu::MipmapFilterMode::Nearest` -- NOT `wgpu::FilterMode::Nearest`
8. `device.poll()` takes `PollType::Wait { submission_index: None, timeout: None }` -- NOT `Maintain::Wait`
9. `RenderPassDescriptor` requires `multiview_mask: None` field
10. `RenderPassColorAttachment` requires `depth_slice: None` field

**Why:** The DESIGN.md documents were authored before verifying against the actual v28 API. These are breaking changes from earlier wgpu versions.

**How to apply:** When implementing any future code that touches wgpu pipelines, render passes, or device polling (Story 005+), use these correct signatures. Future DESIGN.md documents should be validated against `cargo doc -p wgpu` output.
