# E04 Story 003 — Live Magnification Integration impl audit (2026-06-05, commit ea1391a)

Verdict: **AUDIT PASS** (0 blocking; 3 non-blocking LOW). Story = loop-glue only; no engine logic changed.

## Headline 1 — DC-10 coverage honesty: DEFERRAL LEGITIMATE & BOUNDED
- Magnify SHADER pixels ARE verified in automated CI: pre-existing E02 `luminos-gpu/tests/shader_output.rs`
  (offscreen TextureView + readback: solid-color upscale, BGRA swizzle is_bgra=1.0, 1.5x/16x no-artifacts,
  bicubic edge-quality, UV coverage) runs in CI `test-gpu` job (llvmpipe, ci_platform_tests). NOT in diff.
- App-side `overlay_gpu_renderer_summary_zeroed_before_render` only builds a Renderer + asserts p99==0 before
  any frame — does NOT exercise magnify shader (that's shader_output.rs's job). `overlay_gpu_offscreen_render_clear`
  covers only the CLEAR encode path, offscreen. Subprocess tests assert wiring via LOG markers, not pixels.
- What's deferred = ONLY the full capture→render_frame→surface.present() against a live swapchain + non-zero P99.
  Genuinely unverifiable in CI: llvmpipe headless can't present a swapchain (output.present fails, OverlayGpu::new
  → NoAdapter). Deferral to story 007/real-GPU is legitimate.
- HONESTY of "DONE": SUBTASKS AC matrix marks AC-3.2 `[~]` (NOT [x]) + NFR-1 `[~]` UNMEASURABLE; AC-1.1 `[x]` but
  qualified "live present unobservable headless." HLP line 49 + IMPLEMENTATION_NOTES all flag DC-10. NOT a silent gap.

## Headline 2 — Detached-thread deviation: LEGITIMATE, hang claim REAL
- XI2 monitor thread (input.rs:229-289 run_event_loop) blocks in conn.wait_for_event() indefinitely; only checks
  channel Closed AFTER next event arrives. Its JoinHandle is DROPPED (subscribe_input_events returns Ok(rx),
  input.rs:344-353) — monitor is unconditionally detached at platform level; X11InputMonitor has NO Drop impl.
- Circular ownership: monitor holds tx (moved into thread); processor (InputProcessingTask) holds rx. Processor's
  blocking_recv→None only when tx drops → only when run_event_loop returns → only after rx drops (Closed) → only
  when processor exits. join() would deadlock if no input event after shutdown. Detach (drop both, process-exit reap)
  is correct. SIGTERM→exit-0 is test-covered (live_magnification_capture_path_wired asserts terminate_and_wait==Some(0)).
- app.rs:309-322 correctly detaches (logs input_pipeline=detached); the comment there is accurate. NON-BLOCKING:
  IMPLEMENTATION_NOTES §F lines 125-129 stale briefing describes a join() shutdown that is NOT achievable (no monitor
  handle; monitor parked in wait_for_event) — code correctly does NOT follow it.

## Other verifications
- is_bgra: capture.rs:282 hardcodes PixelFormat::Rgba8 → is_bgra=false for X11. is_bgra_format(format)=matches!(.,Bgra8),
  derived from frame.format never hardcoded. BGRA prose correction LOGGED (SUBTASKS Deviations T003 line 216, DESIGN
  body line 55 fixed). NON-BLOCKING: STORY.md line 75 Open-Question prose still says "xcap typically yields BGRA"
  (stale; FR-2 binding is correct).
- No engine changes: only non-app source change = luminos-gpu/src/lib.rs (doc-comment fix + 4 pub-use re-exports,
  zero logic). capture.rs/tracking.rs/hotkeys.rs/renderer.rs/pipeline.rs/input.rs UNTOUCHED.
- No new deps: Cargo.lock +1 line = luminos-types added to luminos-app dep list (path dep). No external dep.
- Counts EXACT: parent app=28, current app=44 (+16). 30 lib (2 GPU-ignored + 2 ci-gated x11_tests) + 14 subprocess.
  All new unit tests pass + assert real behavior.
- DC-13 seam (HLP line 298) matches handle.rs EXACTLY: frame_timings: Arc<Mutex<FrameTimingSummary>> zeroed init;
  loop set_frame_timings(gpu.frame_timing_summary()) each MainEventsCleared (app.rs:289); story 005 reads frame_timings()
  clone; fields average_ms/p99_ms/min_ms/max_ms/target_fps. Accurate for story 005.

## NON-BLOCKING LOW findings
- F-001: IMPLEMENTATION_NOTES §F shutdown recipe (join monitor then processor) is unachievable; code correctly detaches.
- F-002: STORY.md:75 stale "xcap yields BGRA" Open-Question prose (FR-2 + DESIGN + deviations all correct).
- F-003: app.rs set_frame_timings publishes on inactive/clear frames too (publishes renderer's timings regardless of
  is_active) — harmless; slot stays zeroed headless anyway.
