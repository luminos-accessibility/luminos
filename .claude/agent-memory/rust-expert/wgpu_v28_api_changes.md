---
name: wgpu v28 API changes vs DESIGN.md
description: Actual wgpu v28 API differs from DESIGN.md assumptions -- request_adapter returns Result, Instance::new takes reference, request_device has no trace_path
type: project
---

wgpu v28.0.0 API differences from the E02/002 DESIGN.md (which was written based on earlier wgpu versions):

1. `Instance::new(&InstanceDescriptor)` -- takes a reference, not owned value
2. `request_adapter()` returns `Result<Adapter, RequestAdapterError>` -- NOT `Option<Adapter>`. Use `map_err` not `ok_or`.
3. `request_device(&DeviceDescriptor)` takes only 1 arg -- NO `trace_path: Option<&Path>` second parameter
4. `DeviceDescriptor` has 6 fields: `label`, `required_features`, `required_limits`, `experimental_features`, `memory_hints`, `trace`. Use `..Default::default()` for the trailing fields.

**Why:** The DESIGN.md was authored before verifying against the actual v28 API. These are breaking changes from earlier wgpu versions.

**How to apply:** When implementing any future code that touches wgpu device/instance creation (Stories 003-005), use these correct signatures. Future DESIGN.md documents should be validated against `cargo doc -p wgpu` output.
