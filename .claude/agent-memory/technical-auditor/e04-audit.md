---
name: e04-audit
description: E04 HIGH_LEVEL_PLAN.md audit facts -- Tauri control panel, RISK-001 dual event loop, MainEventsCleared nuance
metadata:
  type: project
---

# E04 HIGH_LEVEL_PLAN.md Audit (2026-06-04)

## Verified code reconciliation (all CORRECT in plan's Integration Points table)
- AppState.settings.magnification.zoom_level: CONFIRMED (state.rs nested via AppSettings; schema.rs MagnificationSettings.zoom_level f32, .mode MagnificationMode)
- StateManager methods update_zoom_level/toggle_magnification/reset_zoom: CONFIRMED (state_manager.rs); clamps [1.5,20] via MIN_ZOOM/MAX_ZOOM consts
- LuminosEvent only StateChanged/RequestExit: CONFIRMED (event.rs)
- EventNotifier trait in pipeline.rs: CONFIRMED; impl for winit::EventLoopProxy<LuminosEvent>
- Renderer::new(device,queue,surface_format,vw,vh,method): CONFIRMED. render_frame(&mut self,&Surface,&CaptureFrame,is_bgra:bool). frame_timings()->&FrameTimings. (renderer.rs)
- InputProcessingTask::spawn(receiver:mpsc::Receiver<InputEvent>,state_manager,hotkey_matcher,notifier): CONFIRMED (pipeline.rs)
- FrameTimingSummary{average_ms,p99_ms,min_ms,max_ms,target_fps}: CONFIRMED (frame_timings.rs); plan correctly notes snake_case real / camelCase IPC
- main.rs is empty `fn main() {}`: CONFIRMED (DC-1 valid)
- X11WindowManager uses winit EventLoop + with_override_redirect + WindowAttributesExtX11: CONFIRMED (linux_x11/window.rs); creates ephemeral EventLoop

## Cargo.toml pins (CONFIRMED 2026-06-04)
- wgpu =29.0.3, winit =0.30.13, raw-window-handle =0.6.2, tauri =2.11.2 (tray-icon feat), tauri-build =2.6.2
- tauri-specta =2.0.0-rc.25, specta-typescript =0.0.12
- All rwh 0.6 compatible. Plan's version claims CORRECT.

## Tauri facts verified (web)
- core:default, core:event:default, shell:allow-open: ALL VALID Tauri 2.x permission identifiers
- Tauri Window/WebviewWindow implement HasWindowHandle+HasDisplayHandle since 2.0.0-beta.13: CONFIRMED
- RunEvent::MainEventsCleared EXISTS in Tauri 2.x

## KEY NUANCE / FINDING (MainEventsCleared is NOT a free per-frame tick)
- MainEventsCleared is NOT emitted continuously by default. Needs ControlFlow::Poll AND/OR request_redraw.
- Upstream guidance: render in RedrawRequested, not MainEventsCleared, for redraw-driven apps.
- tao GTK3 Linux backend has a DIFFERENT event loop (DrawQueue state, glib channel); ControlFlow::Poll support is historically fragile (tao #635). The plan's "render in MainEventsCleared each tick" is an oversimplification borrowed from winit semantics; on tao/GTK3 it needs validation. Plan does flag overlay mechanics as MEDIUM-HIGH confidence + spike, partially covering this.
- AD-2 mentions AppHandle::run_on_main_thread / channel drained in MainEventsCleared as wake -- reasonable but per-frame drive cadence still needs Poll or self-rescheduled redraw.

## RISK-001 / AD-1 assessment
- "Cannot run 2nd winit EventLoop in Tauri process (macOS NSApplication principal class)": CORRECT, matches research file + winit #3772.
- Two-window approach avoiding tauri #9220 single-window flicker: SOUND.
- Overstatement risk: AD-1 confidence framing is honest (HIGH on one-loop, MED-HIGH on overlay mechanics).

## Dependency graph
- 001->{002,003}; 004<-{001,003}; 005<-004; 006<-{002,004,005}: ACYCLIC, sensible.
- POTENTIAL MISSING EDGE: 004 get_frame_timings reads live FrameTimingSummary which is only populated by 002's running render loop. Plan says timings "reachable for get_frame_timings (story 004)" as a 002 deliverable. For COMPILE/wiring 004 needs only the type (from luminos-gpu, exists). For MEANINGFUL data 004 depends on 002 at runtime. Not a hard build edge; 006 (which asserts D4 end-to-end) correctly depends on both 002 and 004. Acceptable but worth noting.

## Deliverable coverage D1-D8 (roadmap Section 4.4 lines 387-396)
- D1 webview+overlay -> 001 OK
- D2 zoom slider realtime -> 005 (UI) + 002/004 (round-trip) OK
- D3 mode selector -> 005 OK
- D4 frame timing readout -> 005 (UI) + 002 (timings) OK
- D5 settings persist -> 003 OK
- D6 system tray -> 006 OK
- D7 tauri-specta bindings -> 004 OK
- D8 axe-core -> 005 OK
- All 8 covered; every story maps to >=1 deliverable. COMPLETE.

## Scope (DC-1) -- event loop absorption
- CORRECT that no prior epic built a persistent loop (E2 GPU tests construct throwaway contexts; E3 X11WindowManager drops ephemeral EventLoop). Absorbing unified runtime into E04 is defensible per roadmap "dual-window architecture" deliverable. Worth flagging as roadmap under-specification but not a plan error.

---

# E04 Stories 002-007 Spec Audit (2026-06-04) — re-numbered 7-story layout

NOTE: story numbering changed from the HLP-audit above (001-007 now: 001 app shell, 002 overlay WM, 003 live mag, 004 config, 005 IPC, 006 UI, 007 tray+e2e).

## CRITICAL — specta::Type missing (story 005)
- `specta` is NOT a dep of luminos-types/core/gpu; NO workspace type derives `specta::Type`.
- `#[specta::specta]` commands returning AppSettings / FrameTimingSummary / MagnificationMode REQUIRE specta::Type on those types. 005 DESIGN does not add specta to the 3 engine crates. Compile blocker.
- `FrameTimingSummary` (luminos-gpu/src/frame_timings.rs) derives ONLY Debug,Clone,PartialEq — NO serde, NO specta. HLP "serde rename to camelCase" is false for current code.

## MAJOR — schema_version (story 004)
- AppSettings ALREADY has Default (all sub-structs too). Adding `schema_version: u32` inside AppSettings breaks existing struct-literal tests in schema.rs (compile) + deser of field-less configs unless `#[serde(default)]`. DESIGN omits both. Wrapper struct is better placement.

## MAJOR — self-capture backwards (story 002)
- Real XcbCapture (linux_x11/capture.rs) ALREADY does self-capture via UNMAP/REMAP through `set_excluded_windows(&[u64])` (x11rb-based). 002 DESIGN ranks unmap/remap LAST and prefers unvalidated "window-id exclusion" — contradicts shipped code. Should reuse set_excluded_windows seam.

## MINOR — crate-root path imprecision (recurring)
- luminos-gpu & luminos-platform lib.rs have NO pub use (only pub mod). `luminos_gpu::Renderer/InterpolationMethod/FrameTimingSummary`, `luminos_platform::ScreenCapture` don't resolve as written — need module segment. luminos-core DOES re-export (paths OK there).

## MINOR — event Deserialize (story 005)
- tauri-specta requires event structs derive Serialize+Deserialize+Type+Event. 005 + HLP omit Deserialize.

## Confirmed OK
- 002(a) luminos-platform no tauri dep, x11rb already dep, winit only in window.rs. 002(b) gtk_window() real.
- 003 all reuse signatures match. 003(b) wgpu 29.0.3 Device: Clone = YES. 003(c) InterpolationMethod(gpu) vs InterpolationMode(types) both exist, design uses gpu one correctly.
- 004(a) AppSettings has Default. 004(d) toml 1.1.2 to_string/from_str/ser::Error all exist.
- 005(a-e) tauri-specta rc API names all verified. StateManager add set_magnification_mode/replace_settings reasonable; From<&LuminosHandle> is a cross-crate impl (legal, handle is local).
- 006(a) generated commands/events objects + .listen(e=>e.payload) correct. 006(c) @tauri-apps/api/mocks real (event mock needs api>=2.7.0). 006(d) import.meta.env.DEV valid.
- 007 TrayIconBuilder menu/on_menu_event/on_tray_icon_event/build real; CloseRequested api.prevent_close() on CloseRequestApi correct; tauri-driver tauri:options.application real (pkg @crabnebula/tauri-driver).

---

# E04 Round-3 Audit Delta (2026-06-04) — what changed since round-2

Verdict: PASS-WITH-FIXES. Round-3 re-verified all real-code couplings against crates/ (all HOLD). Two prior MAJORs were PARTIALLY revised, leaving self-contradicting docs:

- F-001 (was round-2 "self-capture backwards"): 002/DESIGN was FIXED only partially. New Primary table (lines 52-63) now correctly ranks unmap/remap (`set_excluded_windows`) as the shipped primary. BUT lines 108 ("self-capture A->C->B") and 150 (Alternatives #4: "unmap/remap ... Deferred to fallback candidate B; try window-id exclusion (A) first") STILL carry the old backwards model and contradict the corrected table + real code. THE single highest-value remaining fix. Verified: capture.rs:166-217,245 = unmap/remap; no "window-id exclusion" path exists.
- F-002 (NEW emphasis): stale winit `Poll`/`request_redraw`/`run_on_main_thread` still in HLP summary lines 47,85,91,254 and 001/DESIGN diagram 39-42, contradicting verified Tauri-2.11 (no request_redraw) and AD-2's own correct dirty-flag body. HLP:254 even cites AD-2 while saying request_redraw.
- F-003: `@tauri-apps/api >= 2.7.0` (006/SUBTASKS T001) — round-2 treated as benign min-version; it is a RANGE pin = violates exact-pin rule (CLAUDE.md global + project). Must pin exact.
- specta::Type + FrameTimingSummary serde gaps: NOW correctly captured in 005 (T001/T002) and HLP DC-5. Resolved since round-2.
- schema_version: NOW correctly placed in a ConfigFile wrapper (004/DESIGN:23-24,54-59); AppSettings untouched. Resolved.
- crate-root re-exports: NOW correctly flagged (DC-5, 005/DESIGN:21). Resolved.
- event Deserialize: NOW present (HLP:205-208, 005/DESIGN:98-101). Resolved.

Minor new: F-005 HLP:199 save(&self) vs 004/DESIGN:78 save(&mut self) drift (DESIGN correct). F-006 005/DESIGN StateManager::from(&h) sugar has no tasked From impl (use StateManager::new(h.app_state.clone())). F-007/F-008 capability/permission timing: window ops + set_ignore_cursor_events used in 001/007 but capability file authored in 005 — confirm core:default suffices for the 001 spike.

Governance round-3: 7 stories; all stories <=5 ACs; subtask totals all arithmetically correct (001:14,002:11,003:10,004:8,005:9,006:10,007:8) and within 5-15; dep graph 001->{002,004};003<-002;005<-{001,004};006<-005;007<-{003,005,006} acyclic. CLEAN.

Lesson: when a prior round flags a doc contradiction, re-grep the WHOLE doc next round — fixes here were applied to one section (the table) but not the contradicting prose/alternatives, producing a self-inconsistent document that reads "fixed" if you only check the table.
