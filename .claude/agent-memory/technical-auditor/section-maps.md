# Document Section Number Maps

Verified 2026-03-18. Used for cross-reference audits.

## doc-02 (Platform Abstraction)
- Section 2: ScreenCapture trait (includes per-platform impl tables in doc comments)
- Section 3: FocusTracker, TtsEngine, WindowManager, InputMonitor, AudioOutput traits
- Section 4: Error Type Architecture (LuminosError, per-subsystem errors)
- Section 5: Conditional Compilation Strategy (5.1 Module Org, 5.2 cfg Patterns, 5.3 Runtime Selection)
- Section 6: Platform Implementation Matrix (THE canonical per-platform backend reference)
- Section 7: Testing Strategy for Platform Code (mocks, naming, CI matrix)

## doc-03 (Rendering Pipeline)
- Section 2: Pipeline Architecture (stages overview)
- Section 3: Viewport Calculation (Stage 1)
- Section 4: Screen Capture Integration (Stage 2)
- Section 5: GPU Texture Management (Stage 3)
- Section 6: Shader Pipeline (Stage 4) - 6.2 magnification, 6.3 color filter, 6.4 cursor overlay
- Section 7: Zoom Mode Rendering (full-screen, lens, docked)
- Section 8: Frame Pacing and VSync (8.3 FrameTimings)
- Section 9: wgpu Initialization and Device Management
- Section 10: Performance Optimization Roadmap (10.2 XShm)
- Section 11: Font Re-Rendering (Phase 3)
- Section 12: Testing Strategy

## doc-04 (TTS Pipeline)
- Section 5: Phonemization (espeak-ng subprocess)
- Section 6: Neural Synthesis (sherpa-onnx)
- Section 7: Audio Playback (cpal)
- Section 8: Voice Model Management
- Section 9: Concurrency Model (TTS Coordinator)
- Section 10: Latency Budget
- Section 11: Word Highlighting
- Section 12: Platform-Native TTS Fallback

## doc-07 (Testing Strategy)
- Section 4: CI/CD Pipeline (4.1-4.6 stages, 4.5=Integration Tests with Xvfb)
- Section 5: Quality Gates (5.3 Release Gate)
- Section 10: Test Naming Conventions
- Section 11: Accessibility Testing
- Section 13: Phase Rollout (13.1=Phase 0, 13.2=Phase 1, 13.3=Phase 2, 13.4=Phase 3+)

## doc-08 (Build and Distribution)
- Section 2: Cargo Workspace Configuration
- Section 3: Cargo Features and Conditional Compilation
- Section 4: Build Profiles
- Section 5: Frontend Build Pipeline (5.1 Toolchain, 5.2 Build Process, 5.3 Specta)
- Section 6: espeak-ng Bundling Strategy
- Section 7: Voice Model Distribution (7.1 Strategy, 7.2 Storage, 7.3 Download Protocol)
- Section 8: Platform Packaging (8.1-8.9 per format)
- Section 9: Code Signing (9.1-9.7)
