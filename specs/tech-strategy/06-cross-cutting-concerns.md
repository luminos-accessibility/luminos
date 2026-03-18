# 06 -- Cross-Cutting Concerns

**Status:** DRAFT v1.1 (post audit review)
**Date:** 2026-03-16
**Audience:** Everyone (engineers, product managers, auditors, contributors, AI agents)
**Source Documents:** [Product Strategy](../PRODUCT_STRATEGY.md) (v1.3, Sections 8, 9, 11), [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) (FINAL), [System Architecture](./01-system-architecture.md) (Sections 9, 10), [Rendering Pipeline](./03-rendering-pipeline.md) (Section 8), [TTS Pipeline](./04-tts-pipeline.md) (Section 10), [Control Panel](./05-control-panel.md) (Section 12)

---

## 1. Overview

### 1.1 Purpose

This document defines the cross-cutting concerns that span every subsystem in Luminos: performance engineering, security and privacy, licensing compliance, accessibility, observability, error handling, and internationalization. These are the non-functional requirements and architectural policies that every implementation story must satisfy regardless of which subsystem it touches.

This document answers: **What quality attributes must every component of Luminos maintain, and how do we verify compliance?**

### 1.2 Scope

This document covers:
- Consolidated performance targets, budgets, profiling strategy, and CI enforcement
- Threat model, privacy architecture, supply chain security, and Tauri security configuration
- GPLv3 licensing compliance, dependency license audit, and contributor obligations
- WCAG 2.1 AA compliance strategy, screen reader coexistence, and overlay accessibility
- Logging architecture, structured diagnostics, metrics collection, and error reporting
- Application-wide error handling strategy and the `LuminosError` hierarchy
- Internationalization approach for the control panel UI

This document does NOT cover:
- Subsystem-specific performance optimizations (see [03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 8 for GPU-specific optimizations; [04 -- TTS Pipeline](./04-tts-pipeline.md) Section 10 for TTS latency)
- Build, packaging, or CI/CD pipeline configuration (see [08 -- Build and Distribution](./08-build-and-distribution.md) (planned))
- Feature-level testing (see [07 -- Testing Strategy](./07-testing-strategy.md) (planned))

### 1.3 Relationship to Other Documents

```
01-system-architecture.md   -- Defines performance targets (§9), security model (§10)
02-platform-abstraction.md  -- Defines trait error types per platform
03-rendering-pipeline.md    -- Defines frame pacing, GPU profiling (§8)
04-tts-pipeline.md          -- Defines TTS latency budget (§10)
05-control-panel.md         -- Defines UI accessibility (§12), diagnostics panel (§10)
    |
    v
06-cross-cutting-concerns.md (this) -- Consolidates, deepens, and adds policies
    |
    v
Implementation stories              -- Must satisfy all policies defined here
```

Cross-cutting concerns are referenced from every subsystem document. When a subsystem defines its own performance target or error type, that target must be consistent with the consolidated budgets in this document. If there is a conflict, this document is authoritative for cross-cutting policy; the subsystem document is authoritative for subsystem-specific detail.

---

## 2. Performance Engineering

### 2.1 Consolidated Performance Targets

These targets apply to all builds running on the minimum supported hardware: integrated GPUs (Intel UHD 630, Apple M-series integrated, AMD Vega) with 8GB system RAM.

| Metric | Target | Hard Limit | Verification | Source |
|--------|--------|------------|--------------|--------|
| Frame rate | 60fps (16.67ms budget) | P99 < 20ms | CI benchmark, `FrameTimings` histogram | Doc-01 §9.1 |
| Frame time variance | Jitter < 3ms (stddev) | No single frame > 33ms | Frame time distribution analysis | Doc-03 §8.3 |
| RAM usage | < 4GB total process | Never exceed 4GB under any workload | CI memory profiler (`/proc/self/status` peak RSS) | Doc-01 §9.3 |
| TTS latency | < 200ms trigger-to-first-audio | < 300ms at P99 | End-to-end latency benchmark | Doc-04 §10 |
| Binary size | < 50MB (excluding voice models) | < 60MB | CI artifact size check | Doc-01 §9.1 |
| Startup time | < 2s to usable magnification | < 3s on cold start (HDD) | Cold start benchmark (5 runs, median) | Doc-01 §9.4 |
| IPC round-trip | < 5ms command-to-response | < 10ms at P99 | IPC latency instrumentation | Doc-05 §2.1 |
| Settings persistence | < 50ms save-to-disk | < 100ms | File I/O benchmark | New |
| Model load time | < 3s for q8 Kokoro (~92MB) | < 5s | Model loading benchmark | Doc-04 §8.3 |

**Hard limits** are the thresholds that trigger CI failures. **Targets** are the engineering goals; missing a target triggers investigation but not a build break.

### 2.2 Memory Budget

The total memory budget is 4GB, shared with the user's applications on an 8GB system. The following allocation provides a more granular breakdown than the initial budget in [01 -- System Architecture](./01-system-architecture.md) Section 9.3, and supersedes it as the canonical reference for memory planning:

| Component | Budget | Typical | Peak | Notes |
|-----------|--------|---------|------|-------|
| Magnification textures | 150MB | 50-100MB | 150MB | Source + destination GPU textures at display resolution. Peak at 4K. |
| Kokoro-82M model (q8, default) | 92MB | 92MB | 92MB | Loaded once at TTS init. Fixed allocation. |
| espeak-ng subprocess | 30MB | 10-15MB | 30MB | OS process overhead + phoneme data. |
| TTS working memory | 20MB | 10MB | 20MB | Phoneme buffers, audio ring buffer, resampler state. |
| Tauri webview (control panel) | 80MB | 30-50MB | 80MB | WebkitGTK/WebView2 baseline + React app. Peak during initial load. |
| Application code + state | 50MB | 20-30MB | 50MB | Rust binary, runtime allocations, configuration. |
| Capture buffers | 40MB | 10-20MB | 40MB | Double-buffered CPU capture data. Peak at 4K low-zoom. |
| Audio output buffers | 5MB | 2MB | 5MB | cpal ring buffer + resampled audio. |
| OS + runtime overhead | 30MB | 20MB | 30MB | Thread stacks, allocator metadata, system libraries. |
| **Total (q8 Kokoro)** | **497MB** | **~264-379MB** | **497MB** | Well within 4GB budget. |
| **Headroom for user apps** | **3.5GB+** | | | On 8GB system. |

**Model variant impact on total budget:**

| Variant | Model Size | Total Peak | Fits 4GB? |
|---------|-----------|------------|-----------|
| Q4 (lightweight) | ~80MB | ~485MB | Yes |
| Q8 (default) | ~92MB | ~497MB | Yes |
| Fp16 (quality) | ~163MB | ~568MB | Yes |
| Fp32 (reference) | ~327MB | ~732MB | Yes, but with warning |

The Fp32 variant is selectable in the UI but displays a "high memory usage" warning badge (see [05 -- Control Panel](./05-control-panel.md) Section 9.5). It is intended for development and quality comparison only.

### 2.3 Hot Path Performance Budget

The magnification frame cycle is the only hot path. Its 16.67ms budget is allocated as follows:

| Stage | Budget | Typical | Notes |
|-------|--------|---------|-------|
| Viewport calculation | < 0.1ms | ~0.01ms | Pure arithmetic on `AppState` fields |
| Screen capture | < 8ms | 1-5ms | Platform-dependent. X11 XCB: ~3-5ms. XShm (Phase 1): ~1ms. |
| GPU texture upload | < 3ms | 0.5-2ms | CPU-to-GPU copy. Proportional to capture region size. |
| Shader execution | < 2ms | < 1ms | Interpolation + color filter + cursor overlay. Trivially fast on modern GPUs. |
| Present/vsync | < 3ms | ~1ms | Queue submission and surface present. |
| **Total** | **< 16.67ms** | **~3-9ms** | Headroom for spikes and OS scheduling jitter. |

The render thread reads `AppState` via `ArcSwap::load()` (lock-free, < 10ns) at the start of each frame. No locks are held during the hot path.

### 2.4 Profiling Strategy

Performance is validated through three complementary approaches:

**1. In-process instrumentation (always-on)**

The `FrameTimings` circular buffer (defined in [03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 8.3) records the last 120 frame times. It provides:
- Per-frame timing breakdown (capture, upload, shader, present)
- Rolling average, P99, min, max, and summary statistics
- Degradation detection: P99 > 20ms for 5 consecutive seconds triggers a `performance_warning` event

TTS latency is instrumented from `speak()` call to first audio sample written to cpal, recorded in a similar circular buffer in `luminos-tts`.

**2. CI benchmark suite (per-commit)**

| Benchmark | Tool | Metric | Threshold |
|-----------|------|--------|-----------|
| Frame time histogram | Custom Rust benchmark harness | P99 frame time | < 20ms (fail if exceeded) |
| Memory high-water mark | `/proc/self/status` VmPeak (Linux) | Peak RSS | < 1GB (fail if exceeded) |
| Binary size | `ls -la` on release artifact | Bytes | < 50MB (warn), < 60MB (fail) |
| Startup time | Cold start to first frame timestamp | Median of 5 runs | < 2s (warn), < 3s (fail) |
| TTS latency | Trigger-to-first-audio | P99 | < 200ms (warn), < 300ms (fail) |

CI benchmarks run on a baseline hardware profile (details in [07 -- Testing Strategy](./07-testing-strategy.md) (planned)). Results are tracked as time-series data to detect regressions before they reach users.

**3. Developer profiling tools (on-demand)**

| Tool | Platform | Use Case |
|------|----------|----------|
| `tracy` (via `profiling` crate) | All | Frame-level profiling with GPU timeline. Compile with `--features profiling`. |
| `perf` | Linux | Kernel-level CPU profiling for capture bottleneck analysis. |
| Instruments (Time Profiler) | macOS | CPU + GPU profiling for Metal backend. |
| PIX / NSight | Windows | GPU profiling for DX12 backend. |
| `heaptrack` or `dhat` | All | Allocation profiling for memory budget validation. |

Profiling instrumentation is gated behind a `profiling` Cargo feature flag. When disabled (default for release builds), profiling macros compile to no-ops with zero runtime cost.

### 2.5 Degradation Strategy

When the system cannot maintain 60fps, Luminos degrades gracefully rather than stuttering unpredictably:

| Degradation Level | Trigger | Action | User Impact |
|-------------------|---------|--------|-------------|
| **Level 0: Healthy** | P99 < 16.67ms | Normal operation | Full quality |
| **Level 1: Warning** | P99 > 20ms for 5s | Emit `performance_warning` event; control panel shows toast | User notified; no automatic changes |
| **Level 2: Auto-degrade** | P99 > 33ms for 10s | Switch from `Bicubic` to `Bilinear` interpolation; disable cursor halo | Slight quality reduction; frame rate recovers |
| **Level 3: Severe** | Average > 33ms for 30s | Reduce internal render resolution by 50%; log warning | Visible softness in magnified output; frame rate recovers |

Level 2 and Level 3 are automatic and reversible. When performance recovers (P99 < 16.67ms for 30s), the previous quality level is restored. The user can disable auto-degradation in settings (Phase 1); in that case, only Level 1 notifications are emitted.

**Phase 0 scope:** Only Level 0 and Level 1 are implemented in Phase 0. Level 2 is Phase 1. Level 3 is Phase 2.

### 2.6 Optimization Roadmap

Performance optimizations are sequenced across phases, each unlocking the next level of capability:

| Phase | Optimization | Expected Gain | Dependency |
|-------|-------------|---------------|------------|
| Phase 0 | Viewport-only capture (capture source region, not full screen) | 2-10x less data to upload | Core architecture |
| Phase 0 | `ArcSwap` lock-free state reads | Eliminates lock contention on render thread | `arc-swap` crate |
| Phase 1 | XShm shared-memory capture (Linux X11) | ~3x faster capture on X11 | `x11rb` crate with SHM |
| Phase 1 | Dirty-region detection (skip upload when screen unchanged) | 0ms upload on static frames | Capture diffing logic |
| Phase 1 | Bicubic shader optimization (LUT-based Catmull-Rom) | ~20% shader time reduction | WGSL shader work |
| Phase 2 | GPU texture sharing (DXGI on Windows, IOSurface on macOS) | Eliminates CPU-to-GPU copy | Platform-specific backends |
| Phase 2 | Adaptive capture rate (reduce capture frequency at high zoom) | Reduces CPU/bus pressure at high zoom | Heuristic based on zoom level |
| Phase 3 | Multi-GPU support (discrete GPU for rendering) | Enables 4K+ magnification at 60fps | wgpu adapter selection |

---

## 3. Security and Privacy

### 3.1 Threat Model

Luminos processes the user's entire screen content -- the most sensitive data on any personal computer. The threat model is designed around this reality.

**Assets to protect:**
1. Screen capture data (contains passwords, financial info, personal communications)
2. Recognized text (OCR/TTS input text)
3. User configuration (keybindings, profiles -- less sensitive but still personal)
4. Voice model files (intellectual property concern for custom voices)

**Threat actors:**
1. **Remote attackers** -- network-based exploitation of Luminos or its dependencies
2. **Local malicious software** -- malware on the same system attempting to access Luminos's data
3. **Supply chain attacks** -- compromised dependencies injecting malicious code
4. **Social engineering** -- tricking users into installing malicious plugins (Phase 4)

**Out of scope:** Physical access attacks, OS-level compromise, and nation-state adversaries targeting individual users. These are mitigated at the OS level, not the application level.

### 3.2 Privacy Architecture

Luminos follows a **local-first, zero-exfiltration** privacy model:

| Principle | Implementation | Verification |
|-----------|---------------|--------------|
| No network transmission of user data | Screen content, recognized text, and usage patterns never leave the device | Static analysis: grep for network API usage; CI check for no outbound connections during test |
| No telemetry by default | No analytics, no crash reporting, no usage tracking unless explicitly opted in | Code review policy: any telemetry addition requires RFC and opt-in UX |
| Local AI inference | All TTS (Kokoro, Piper), OCR, and future AI features run on-device | Architecture constraint: `luminos-tts` and `luminos-core` have no network dependencies |
| No persistent screen data | Debug logs never persist screen capture pixels; frame buffers are overwritten each frame | Log review: debug log format spec prohibits pixel data; capture buffers use double-buffering (old data overwritten) |
| Minimal configuration data | Settings stored in `~/.config/luminos/` as TOML; no cloud sync | Configuration manager writes only to user config directory |

**Network access policy:** The Luminos process makes network connections only in two strictly bounded scenarios:
1. **Voice model download** -- when the user explicitly requests a new voice model (Phase 2+). Downloads use HTTPS from a pinned domain (`models.luminos.dev` or a verified CDN). Download progress is shown in the UI.
2. **Update check** -- when the user enables auto-update checking (opt-in, Phase 3+). Checks a signed manifest from `releases.luminos.dev`.

Both scenarios require explicit user action. Neither transmits user data (screen content, settings, or usage patterns). The update checker sends only the current version and platform identifier.

### 3.3 Process Isolation

| Process | Isolation | Rationale |
|---------|-----------|-----------|
| Luminos main process | Standard OS process | Hosts Rust engine, GPU rendering, Tauri webview |
| espeak-ng subprocess | Separate OS process, stdin/stdout pipes | Crash isolation (espeak-ng segfaults do not crash Luminos); resource isolation (memory leaks confined); no access to screen capture buffers or GPU textures |
| Tauri webview | OS webview sandbox (WebkitGTK/WebView2) | Webview runs in the OS-provided sandbox; Tauri's security model restricts API access |

The espeak-ng subprocess has:
- No network access (no sockets, no DNS)
- No file system access beyond its own data files (read-only)
- No display server access
- No IPC with Luminos except the stdin/stdout pipe protocol

If espeak-ng crashes, the TTS Coordinator in `luminos-tts` detects the broken pipe, logs the event, and respawns the subprocess. The user sees a brief TTS interruption (~200ms) and an `espeak_status_changed` event in the control panel. See [04 -- TTS Pipeline](./04-tts-pipeline.md) Section 5 for the crash recovery protocol.

### 3.4 Tauri Security Configuration

Tauri 2.0 provides a capability-based permission system. Permissions not listed in a capability's `permissions` array are implicitly denied -- there is no explicit deny mechanism at the capability level. Luminos configures the control panel webview with minimal permissions:

```json
{
  "identifier": "luminos-default",
  "description": "Default permissions for the Luminos control panel",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    "core:event:default"
  ]
}
```

**Key restrictions (enforced by omission):**
- **No file system access** from the webview. `fs:default` is not listed, so all file operations (config save/load, profile import/export) must go through Tauri IPC commands with server-side validation.
- **No HTTP access** from the webview. `http:default` is not listed, so voice model downloads are initiated by Rust code, not JavaScript.
- **No shell execution** from the webview beyond `shell:allow-open` (opening documentation links in the system browser). No arbitrary command execution.
- **No notification API** -- `notification:default` is not listed; Luminos uses its own in-panel toast system.

The `core:event:default` permission allows the webview to listen to events emitted by the Rust backend. Custom events are typed (via `tauri-specta`) and validated server-side before emission.

### 3.5 Input Validation

All data crossing trust boundaries is validated:

| Boundary | Direction | Validation |
|----------|-----------|------------|
| IPC (TypeScript → Rust) | Inbound | Serde deserialization with explicit type constraints; range clamping on numeric values (e.g., zoom 1.5-20.0); enum variants reject unknown values |
| IPC (Rust → TypeScript) | Outbound | `tauri-specta` ensures type safety; TypeScript side validates complex responses with Zod schemas |
| espeak-ng subprocess (Rust → stdin) | Outbound | Text sanitized: control characters stripped, length capped at 10,000 characters, no shell metacharacters |
| espeak-ng subprocess (stdout → Rust) | Inbound | Phoneme output parsed with strict IPA/Kirshenbaum parser; malformed output triggers re-request or fallback |
| Configuration file (disk → Rust) | Inbound | TOML parsed with `serde` and validated against `AppSettings` schema; unknown fields ignored (forward compat); invalid values replaced with defaults + warning log |
| Profile import (JSON → Rust) | Inbound | JSON parsed with `serde`; validated against `ProfileDocument` schema; version field checked; all settings values range-checked |
| Voice model files (disk → sherpa-onnx) | Inbound | File integrity verified via SHA-256 checksum against manifest; ONNX model loaded in sherpa-onnx sandbox |

### 3.6 Supply Chain Security

| Measure | Implementation | Phase |
|---------|---------------|-------|
| `cargo audit` | Runs in CI on every push; fails build on known vulnerabilities with CVSS >= 7.0 | Phase 0 |
| `cargo deny` | Enforces license allowlist (MIT, Apache-2.0, BSD-2/3, ISC, Zlib, MPL-2.0, GPL-3.0, LGPL-2.1/3.0). Any license not in the allowlist is automatically denied. | Phase 0 |
| Dependency pinning | `Cargo.lock` committed to repository; exact versions in CI | Phase 0 |
| SBOM generation | Software Bill of Materials (CycloneDX format) produced for each release | Phase 1 |
| Reproducible builds | Target reproducible release builds; track Rust reproducibility improvements | Phase 2 (best-effort) |
| npm audit | Runs in CI for the `ui/` frontend; fails on high-severity vulnerabilities | Phase 0 |
| Dependabot / Renovate | Automated dependency update PRs; reviewed before merge | Phase 0 |

### 3.7 Signed Releases

All release binaries are signed to allow users and institutions to verify authenticity:

| Platform | Signing Method | Verification |
|----------|---------------|-------------|
| Linux | GPG signature on `.deb`, `.rpm`, AppImage | `gpg --verify`; APT/DNF repositories use signed Release files |
| macOS | Apple Developer ID code signing + notarization | Gatekeeper verification; `codesign --verify` |
| OpenBSD | Signify signature on package | `signify -V` |
| Windows | Authenticode code signing (EV certificate) | Windows SmartScreen; `signtool verify` |

**Key management:** During Phase 0-1 (Year 1), signing keys are held by the project founder and stored in CI secrets (GitHub Actions encrypted secrets). Release signing is automated in the CI pipeline. Private keys never exist on developer machines. When the non-profit foundation is established (Year 2+, per [Product Strategy](../PRODUCT_STRATEGY.md) Section 12.2), key custody transfers to the foundation with hardware security modules (HSMs) or cloud KMS for enhanced protection.

---

## 4. Licensing Compliance

### 4.1 Project License

Luminos is licensed under **GNU General Public License v3.0** (GPL-3.0-only). This choice was made deliberately for the following reasons (detailed in [Product Strategy](../PRODUCT_STRATEGY.md) Section 8.4):

1. **espeak-ng compatibility** -- espeak-ng is GPL-3.0. Since Luminos is also GPLv3, there is no license propagation concern regardless of how espeak-ng is integrated (subprocess or library).
2. **Community alignment** -- GPLv3 aligns with the Linux/FOSS community values where Luminos launches first.
3. **Anti-proprietary-fork protection** -- GPLv3 copyleft prevents closed-source forks from fragmenting the accessibility tool ecosystem.
4. **Precedent** -- NVDA (GPL-2.0-or-later) is widely deployed in enterprises, government agencies, and universities. GPLv3 does not prevent institutional adoption.

### 4.2 Dependency License Audit

Every direct dependency must have a license compatible with GPLv3. The following table is the canonical license audit for all direct dependencies as of 2026-03-15:

**Rust Dependencies:**

| Crate | Version | License | GPLv3 Compatible? | Notes |
|-------|---------|---------|-------------------|-------|
| `winit` | 0.30.13 | Apache-2.0 | Yes | |
| `wgpu` | 28.0.0 | MIT OR Apache-2.0 | Yes | |
| `xcap` | 0.9.1 | Apache-2.0 | Yes | |
| `sherpa-rs` | 0.6.8 | MIT | Yes | Rust bindings for sherpa-onnx |
| `cpal` | Latest | Apache-2.0 | Yes | |
| `arboard` | Latest | MIT OR Apache-2.0 | Yes | |
| `tauri` | 2.x | MIT OR Apache-2.0 | Yes | |
| `serde` | Latest | MIT OR Apache-2.0 | Yes | |
| `arc-swap` | Latest | MIT OR Apache-2.0 | Yes | |
| `crossbeam-channel` | Latest | MIT OR Apache-2.0 | Yes | |
| `parking_lot` | Latest | MIT OR Apache-2.0 | Yes | |
| `x11rb` | Latest | MIT OR Apache-2.0 | Yes | |
| `atspi` | Latest | MIT OR Apache-2.0 | Yes | |
| `rdev` | Latest | MIT | Yes | |
| `toml` | Latest | MIT OR Apache-2.0 | Yes | Configuration parsing |
| `log` | Latest | MIT OR Apache-2.0 | Yes | Logging facade |
| `env_logger` | Latest | MIT OR Apache-2.0 | Yes | Logging implementation |

**External Binaries:**

| Binary | License | GPLv3 Compatible? | Integration | Notes |
|--------|---------|-------------------|-------------|-------|
| espeak-ng | GPL-3.0 | Yes (identical copyleft) | Subprocess | Crash isolation is the engineering rationale, not legal isolation |

**TTS Model Weights:**

| Model | License | GPLv3 Compatible? | Notes |
|-------|---------|-------------------|-------|
| Kokoro-82M | Apache-2.0 | Yes | Model weights are separate from code; Apache 2.0 is GPLv3-compatible |
| Piper VITS models | MIT | Yes | Pre-fork model weights are MIT-licensed and distributed separately. Note: the Piper project has moved to `OHF-Voice/piper1-gpl` (GPL-3.0); future model weights from the post-fork project may be GPL-licensed. Luminos uses pre-fork MIT-licensed model weights. |
| espeak-ng data | GPL-3.0 | Yes | Bundled with espeak-ng |

**JavaScript Dependencies:**

| Package | License | GPLv3 Compatible? | Notes |
|---------|---------|-------------------|-------|
| `react`, `react-dom` | MIT | Yes | |
| `react-router` | MIT | Yes | |
| `zustand` | MIT | Yes | |
| `immer` | MIT | Yes | |
| `zod` | MIT | Yes | |
| `use-debounce` | MIT | Yes | |
| `@tauri-apps/api` | MIT OR Apache-2.0 | Yes | |

**sherpa-onnx runtime:**

| Component | License | GPLv3 Compatible? | Notes |
|-----------|---------|-------------------|-------|
| sherpa-onnx C++ library | Apache-2.0 | Yes | Statically linked via sherpa-rs |
| ONNX Runtime | MIT | Yes | Linked by sherpa-onnx |
| kaldi-native-fbank | Apache-2.0 | Yes | Audio feature extraction |

### 4.3 License Enforcement in CI

`cargo deny` is configured to enforce license compliance automatically:

```toml
# deny.toml
[licenses]
# In modern cargo-deny, any license NOT in the allow list is automatically denied.
# There is no separate `deny` or `copyleft` field -- those are removed.
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Zlib",
    "MPL-2.0",
    "GPL-3.0-only",
    "GPL-3.0-or-later",
    "LGPL-2.1-only",     # Compatible with GPLv3 (LGPL 2.1 §3 allows upgrade to GPL)
    "LGPL-2.1-or-later",
    "LGPL-3.0-only",     # Explicitly compatible with GPLv3
    "LGPL-3.0-or-later",
    "Unicode-3.0",
    "Unicode-DFS-2016",
]
exceptions = []
confidence-threshold = 0.8

[licenses.private]
ignore = false           # No private/proprietary dependencies allowed
```

**LGPL compatibility note:** LGPL-2.1 Section 3 allows "upgrading" to GPL-2.0+, which is GPLv3-compatible. LGPL-3.0 is explicitly compatible with GPLv3 (GPLv3 compliance automatically satisfies LGPL-3.0 requirements). For a GPLv3 project, LGPL dependencies can be statically linked without additional obligations beyond what GPLv3 already requires. The FSF compatibility matrix confirms this.

**Notable exclusions** (denied by omission from the allowlist):
- `GPL-2.0-only` -- GPLv2-only code is NOT compatible with GPLv3 (no "or later" clause)
- `AGPL-3.0-only` -- Network copyleft is overly restrictive for a desktop application
- `SSPL-1.0` -- Not OSI-approved
- `BSL-1.1` -- Not open source

Any new dependency that introduces an unlisted license triggers a CI failure and requires manual review and an update to `deny.toml` with documented rationale.

### 4.4 Contributor Obligations

All contributors to Luminos agree to license their contributions under GPLv3 via the Developer Certificate of Origin (DCO). Luminos uses DCO sign-offs (not a Contributor License Agreement) to maintain community trust:

```
Signed-off-by: Contributor Name <contributor@example.com>
```

**Why DCO over CLA:**
- CLAs require copyright assignment or broad license grants, which discourages contributions
- DCO is simpler: contributors certify they have the right to submit the code under the project's license
- Linux kernel, Git, and many major projects use DCO successfully
- GPLv3 does not require a CLA for the project to enforce its license

**Third-party code inclusion rules:**
1. All included code must be GPLv3-compatible (see §4.3 allowlist)
2. Original license and copyright notices must be preserved
3. Vendored code must be in a clearly marked directory with license files
4. AI-generated code contributions are treated as author contributions (the submitter asserts DCO)

---

## 5. Accessibility

### 5.1 Compliance Target

Luminos targets **WCAG 2.1 AA** compliance for all user-facing interfaces. This applies to:
1. The control panel (Tauri webview -- covered in detail in [05 -- Control Panel](./05-control-panel.md) Section 12)
2. The magnification overlay's visual behavior (contrast, flicker, motion)
3. All documentation and error messages
4. The installer and first-run experience

WCAG 2.2 AA is a stretch goal for Phase 3+, tracking the European Accessibility Act (EAA) requirements that become enforceable for new products from June 2025.

**Rationale:** An inaccessible accessibility tool is not merely ironic -- it is a critical defect. Low-vision users are the primary audience. Every interface element must be usable by someone who needs screen magnification to see it.

### 5.2 Control Panel Accessibility

The control panel accessibility requirements are fully specified in [05 -- Control Panel](./05-control-panel.md) Section 12. The key requirements are summarized here for completeness:

| Requirement | Standard | Implementation |
|-------------|----------|----------------|
| Full keyboard navigation | WCAG 2.1.1 (Keyboard) | All controls reachable and operable via keyboard; no keyboard traps |
| Visible focus indicators | WCAG 2.4.7 (Focus Visible) | 3px solid outline, high-contrast color, visible at all zoom levels |
| Screen reader compatibility | WCAG 4.1.2 (Name, Role, Value) | ARIA labels, roles, and live regions on all interactive elements |
| No color-only information | WCAG 1.4.1 (Use of Color) | All status indicators use color + text/icon |
| Sufficient contrast | WCAG 1.4.3 (Contrast Minimum) | 4.5:1 for normal text, 3:1 for large text |
| Responsive to OS text size | WCAG 1.4.4 (Resize Text) | All sizes in `rem`; no pixel-fixed interactive elements |
| Reduced motion support | WCAG 2.3.3 (Animation from Interactions, AAA) | `prefers-reduced-motion` media query respected. This exceeds the AA target. |
| High contrast mode | Platform integration | `forced-colors: active` and `prefers-contrast: more` CSS queries |

### 5.3 Magnification Overlay Accessibility

The magnification overlay itself has accessibility properties that matter:

**5.3.1 Flicker and photosensitivity**

- The overlay must not contain content that flashes more than three times per second (WCAG 2.3.1 "Three Flashes or Below Threshold", Level A; WCAG 2.3.2 "Three Flashes", Level AAA)
- Frame rate drops that cause visible flicker trigger the degradation strategy (Section 2.5) rather than allowing stutter
- Color filter transitions (e.g., switching from None to Invert) apply instantly in a single frame, not via animation

**5.3.2 Smooth motion**

- Viewport panning (cursor/focus tracking) uses smooth scrolling with configurable speed (see [03 -- Rendering Pipeline](./03-rendering-pipeline.md) Section 7)
- Users who experience motion sickness can disable smooth scrolling (`smoothScrolling: false` in `AppSettings`)
- When smooth scrolling is disabled, viewport jumps snap instantly to the new position

**5.3.3 Cursor visibility**

- The magnified cursor must always be visible and distinguishable from the background
- Cursor enhancement features (enlargement, crosshairs, halo, locator animation) exist specifically for this purpose
- The "Find Cursor" hotkey (`Ctrl+Alt+F`) triggers a locator animation that helps users locate a lost cursor -- this is an accessibility feature, not a convenience feature

**5.3.4 Screen reader coexistence**

Luminos must coexist with screen readers (Orca on Linux, NVDA/JAWS on Windows, VoiceOver on macOS) without conflict:

| Concern | Strategy |
|---------|----------|
| Hotkey conflicts | Luminos hotkeys are configurable; default bindings avoid known screen reader hotkeys (NVDA uses `Insert+...`, JAWS uses `Insert+...`, Orca uses `Caps Lock+...`). Conflict detection is planned for Phase 2. |
| Focus stealing | The magnification overlay window is `WS_EX_TRANSPARENT` (Windows) / `override_redirect` (X11) / equivalent. It does not appear in the accessibility tree and does not steal focus from the user's active application. |
| Audio channel sharing | Luminos TTS and screen reader TTS both output to the system audio device. cpal handles mixing at the OS level. Users can configure Luminos to use a specific output device if needed (Phase 2). |
| AT-SPI / UIA integration | Luminos uses AT-SPI2 (Linux) and UI Automation (Windows) for focus tracking. It registers as a consumer, not a provider, of accessibility information. It does not modify the accessibility tree. |

### 5.4 Documentation Accessibility

All documentation (user manuals, README, CONTRIBUTING) must be:
- Written in clear, simple language (target: 8th-grade reading level, Flesch-Kincaid)
- Available in plain text and HTML formats
- Screen reader navigable (proper heading hierarchy, alt text on images, descriptive link text)
- Available with audio descriptions for visual diagrams (Phase 3)

### 5.5 Accessibility Testing

| Method | Scope | Frequency | Phase |
|--------|-------|-----------|-------|
| `axe-core` automated checks | Control panel components | Every PR (CI) | Phase 0 |
| Manual keyboard navigation test | Control panel, all pages | Every release | Phase 0 |
| Orca screen reader testing | Control panel + overlay coexistence on Linux | Every release | Phase 0 |
| NVDA screen reader testing | Control panel + overlay coexistence on Windows | Every Windows release | Phase 4 |
| VoiceOver screen reader testing | Control panel + overlay coexistence on macOS | Every macOS release | Phase 2 |
| User testing with low-vision testers | Full application | Quarterly (when resources available) | Phase 1 |

---

## 6. Observability

### 6.1 Logging Architecture

Luminos uses the Rust `log` facade with `env_logger` as the default backend. This combination provides:
- Zero-cost logging when a level is disabled (via the `log` crate's `release_max_level_info` Cargo feature, which strips `debug!` and `trace!` calls at compile time in release builds)
- Runtime log level control via the `LUMINOS_LOG` environment variable (configured via `env_logger::Builder::from_env("LUMINOS_LOG")` at startup -- `env_logger` defaults to `RUST_LOG` but Luminos uses a custom variable to avoid conflicts with other Rust programs)
- Consistent structured logging across all crates

**Log level policy:**

| Level | Usage | Example | Default |
|-------|-------|---------|---------|
| `error` | Unrecoverable failures or conditions requiring user attention | `"Failed to initialize wgpu device: {}"` | Always on |
| `warn` | Unexpected but non-fatal conditions | `"espeak-ng subprocess exited unexpectedly, restarting"` | Always on |
| `info` | Significant state transitions visible to the user | `"Magnification started on display 'eDP-1'"`, `"Loaded Kokoro q8 model in 1.2s"` | On by default |
| `debug` | Developer-focused diagnostic detail | `"Frame 12345: capture=3.2ms upload=1.1ms shader=0.4ms total=4.7ms"` | Off by default |
| `trace` | Granular internal state (high volume) | `"ArcSwap::rcu zoom_level 5.0 -> 5.5"` | Off by default |

**Logging conventions** (from CLAUDE.md):
- Dynamic values in single quotes: `log::info!("Capturing display '{}'", display.name)`
- Multi-line messages use `concat!`
- Never log screen capture pixel data or recognized text at any level (privacy)
- Never log at `info` or above on the hot path (render loop)

**Environment variable:**
```bash
# Default: info level for luminos crates, warn for dependencies
LUMINOS_LOG=luminos=info,warn

# Full debug output for the TTS pipeline only
LUMINOS_LOG=luminos_tts=debug,luminos=info,warn

# Trace everything (development only, very verbose)
LUMINOS_LOG=trace
```

### 6.2 Structured Diagnostics

Beyond text logs, Luminos exposes structured diagnostic data through the control panel's Diagnostics page and internal instrumentation:

| Diagnostic | Source | Access | Phase |
|------------|--------|--------|-------|
| Frame timing histogram | `FrameTimings` (luminos-gpu) | IPC: `get_frame_timings` → `FrameTimingSummary` | Phase 0 |
| TTS latency histogram | `TtsTimings` (luminos-tts, proposed) | IPC: `get_tts_timings` (proposed, Phase 2+). Note: this type and command do not yet exist in doc-04 or doc-05; they will be added when TTS diagnostics are specified. | Phase 2 |
| System info | OS, GPU, RAM, version | IPC: `get_system_info` → `SystemInfo` | Phase 3 (per doc-05 §1.3) |
| espeak-ng status | Subprocess health, crash count, restart count | IPC: `check_espeak_available`, `espeak_status_changed` event | Phase 2 |
| Configuration state | Current `AppSettings` snapshot | IPC: `get_current_settings` | Phase 0 |
| Performance warnings | P99 threshold exceedance | Event: `performance_warning` | Phase 0 |

### 6.3 Error Reporting

Luminos does not phone home with crash reports. Instead, it provides tools for users to generate diagnostic bundles for bug reports:

**Diagnostic bundle (Phase 2):**

A "Copy diagnostic info" button in the Diagnostics page (available from Phase 0 as "Copy to clipboard" in `SystemInfoPanel`) collects:
- Luminos version, OS, GPU, GPU backend
- Current settings (sanitized: no keybindings or personal profile names)
- Last 30 seconds of frame timing data
- espeak-ng availability and version
- Current log output (last 100 lines, filtered for privacy)

The bundle is formatted as Markdown for direct pasting into a GitHub issue. No raw screen capture data, no recognized text, no file paths are included.

### 6.4 Metrics for Development

During development, the following internal metrics guide optimization decisions:

| Metric | Collection Point | Use |
|--------|-----------------|-----|
| Frames rendered | Render loop counter | Validate 60fps target |
| Capture time per frame | Pre/post capture timestamp | Identify platform-specific capture bottlenecks |
| GPU upload time per frame | Pre/post texture write | Identify bandwidth bottlenecks |
| Shader execution time | GPU timestamp queries (when available) | Validate shader complexity |
| TTS pipeline latency | Speak call to first audio sample | Validate <200ms target |
| Memory RSS | Periodic `/proc/self/status` read (Linux) | Track against 4GB budget |
| IPC command latency | Tauri command start to response | Validate <5ms target |
| Config save latency | ConfigManager write start to completion | Validate <50ms target |

These metrics are not exposed to end users. They are available in debug builds via the `LUMINOS_LOG=luminos=debug` log level and in CI benchmark output.

---

## 7. Error Handling

### 7.1 Error Philosophy

Luminos follows these error handling principles:
1. **Never crash the application.** The user depends on magnification to use their computer. A crash means they may be unable to navigate to restart the application. Panics in production are critical bugs.
2. **Degrade gracefully.** If a subsystem fails (TTS, control panel, capture), the remaining subsystems continue operating.
3. **Inform the user.** Errors that affect the user's experience are communicated through the control panel (toasts, banners) or system notifications. Silent failures are bugs.
4. **Log for diagnosis.** Every error is logged with sufficient context for diagnosis. The log message includes what was attempted, what failed, and what fallback (if any) was applied.

### 7.2 Error Type Hierarchy

Each platform trait defines its own error enum. The top-level `LuminosError` in `luminos-platform/src/error.rs` unifies them via `#[from]` conversions. The canonical definition is in [02 -- Platform Abstraction](./02-platform-abstraction.md) Section 4.1:

```
LuminosError (luminos-platform/src/error.rs)
  ├── Capture(CaptureError)          -- from ScreenCapture trait
  │     ├── DisplayNotFound(String)
  │     ├── PermissionDenied
  │     ├── RegionOutOfBounds { ... }
  │     ├── BackendUnavailable { reason }
  │     └── Platform { message, source }
  ├── Focus(FocusError)              -- from FocusTracker trait
  │     ├── ApiUnavailable { reason }
  │     ├── PermissionDenied { reason }
  │     └── Platform { message, source }
  ├── Tts(TtsError)                  -- from TtsEngine trait
  │     ├── VoiceNotFound(String)
  │     ├── ModelLoadFailed { path, reason }
  │     ├── InferenceFailed { reason }
  │     ├── PhonemizerUnavailable { reason }
  │     └── Platform { message, source }
  ├── Window(WindowError)            -- from WindowManager trait
  │     ├── CreationFailed { message }
  │     ├── PropertyFailed { property, message }
  │     ├── ModeUnsupported { mode }
  │     └── Platform { message, source }
  ├── Input(InputError)              -- from InputMonitor trait
  │     ├── Unavailable { reason }
  │     ├── Disconnected
  │     └── Platform { message, source }
  ├── Audio(AudioError)              -- from AudioOutput trait
  │     ├── NoDevice
  │     ├── DeviceFailed { message }
  │     ├── FormatUnsupported { ... }
  │     └── Platform { message, source }
  ├── Config { message: String }     -- configuration errors
  └── Internal { message: String }   -- unexpected internal errors
```

**Additional error types in crates outside `luminos-platform`:**

The `luminos-gpu` crate defines `RenderError` for wgpu-specific failures (device creation, surface loss, shader compilation, texture upload). The `luminos-app` crate defines `IpcError` for Tauri IPC serialization failures. These are not part of `LuminosError` directly -- they are handled at the crate boundary and converted to appropriate `LuminosError` variants (typically `Internal { message }`) when they cross into `luminos-core`.

Each error variant implements `std::fmt::Display` (via `thiserror::Error`) with a user-friendly message and `std::error::Error` for chaining. Conversion between crate error types uses `From` trait implementations so that `?` propagation works across crate boundaries.

### 7.3 Error Recovery Strategies

| Error | Impact | Recovery | User Notification |
|-------|--------|----------|-------------------|
| `CaptureError::BackendUnavailable` | Capture not functional | Retry with alternative backend; fall back to last good frame per-frame | None if transient; error banner if persistent |
| `CaptureError::PermissionDenied` | No magnification | Show permission request dialog; on Linux X11, this should never occur | Error banner in control panel |
| `RenderError::SurfaceLost` (luminos-gpu) | No rendering for 1-2 frames | Recreate wgpu surface; common on window resize | None (auto-recovered) |
| `RenderError::AdapterNotFound` (luminos-gpu) | Application cannot start | Log error; display system dialog (not webview); exit with error code | System dialog + log |
| `TtsError::PhonemizerUnavailable` | No neural TTS | Platform-native TTS fallback; espeak warning banner | Warning banner on Speech page |
| espeak-ng subprocess crash | TTS interrupted | TTS Coordinator detects broken pipe, respawns (~200ms); retry pending text | Brief TTS silence; `espeak_status_changed` event |
| `TtsError::ModelLoadFailed` | Selected voice unavailable | Fall back to platform-native TTS; log model path and reason | Error toast + TTS status indicator |
| `LuminosError::Config { message }` | Settings corrupted | Replace with compiled-in defaults; backup corrupt file as `.bak`; log warning | Warning toast: "Settings were reset to defaults" |
| `FocusError::ApiUnavailable` | No focus/caret tracking | Fall back to cursor tracking mode; log warning | Info toast: "Focus tracking unavailable; using cursor tracking" |

### 7.4 Panic Policy

**Production code must not panic.** This is enforced by:
1. **No `unwrap()` or `expect()` in production code** (CLAUDE.md rule). Use `match`, `if let`, or `.unwrap_or_else()`.
2. **Panic hook** installed at startup that logs the panic with backtrace and attempts to write a crash report file before exiting.
3. **CI lint** (`clippy::unwrap_used`, `clippy::expect_used`) fails the build if `unwrap()` or `expect()` appears outside of `#[cfg(test)]` code.

Exception: `unwrap()` is acceptable in unit tests for conciseness.

**Panic handler:**

```rust
// luminos-app/src/main.rs (simplified)
fn install_panic_handler() {
    std::panic::set_hook(Box::new(|info| {
        let backtrace = std::backtrace::Backtrace::force_capture();
        log::error!(
            concat!(
                "FATAL: Luminos panicked.\n",
                "  Info: {}\n",
                "  Backtrace:\n{}",
            ),
            info, backtrace
        );
        // Attempt to write crash report to ~/.config/luminos/crash.log
        // (best-effort; the process may be in an inconsistent state)
        let _ = write_crash_report(info, &backtrace);
    }));
}
```

---

## 8. Internationalization (i18n)

### 8.1 Scope and Timeline

| Phase | i18n Scope |
|-------|-----------|
| Phase 0-2 | English-only UI. All user-facing strings are extracted into a string table (key-value JSON) but only the English file exists. |
| Phase 3 | Community translation infrastructure. String table files for contributed languages. |
| Phase 4 | Full i18n: translated UI, locale-aware formatting, RTL support for applicable languages. |

### 8.2 String Extraction Strategy

All user-facing strings in the control panel are referenced by key, not embedded inline:

```typescript
// ui/src/i18n/en.json (Phase 0: the only file)
{
  "magnification.title": "Magnification",
  "magnification.zoom_level": "Zoom level",
  "magnification.mode.full_screen": "Full Screen",
  "magnification.mode.lens": "Lens",
  "magnification.mode.docked": "Docked",
  "speech.espeak_warning": "Text-to-speech requires espeak-ng, which was not found.",
  "speech.espeak_install_linux": "Install with: sudo apt install espeak-ng",
  ...
}
```

In Phase 0-2, the i18n layer is a simple key lookup function that returns the English string. This imposes minimal overhead while ensuring that all strings are extracted and ready for translation when Phase 3 begins.

**Rust-side strings** (log messages, error messages used in IPC responses) remain in English. Log messages are developer-facing and are not translated. IPC error strings that reach the UI are mapped to translated keys on the TypeScript side.

### 8.3 Locale-Aware Formatting

Even before full i18n, the application respects the user's locale for:
- Number formatting (decimal separator) in the control panel display
- Date formatting in profile metadata (`createdAt`)
- The `Intl` JavaScript API handles this natively in the webview

### 8.4 TTS Language vs. UI Language

The TTS language (voice selection) is independent of the UI language. A user with a French UI can use an English TTS voice and vice versa. The TTS language is determined by the selected voice, not the UI locale.

---

## 9. Cross-References

| Topic | Document | Section |
|-------|----------|---------|
| Performance targets (canonical) | [01 -- System Architecture](./01-system-architecture.md) | 9.1 |
| Memory budget (first defined) | [01 -- System Architecture](./01-system-architecture.md) | 9.3 |
| Security and privacy model (first defined) | [01 -- System Architecture](./01-system-architecture.md) | 10 |
| Hot path identification | [01 -- System Architecture](./01-system-architecture.md) | 9.2 |
| Frame pacing and vsync control | [03 -- Rendering Pipeline](./03-rendering-pipeline.md) | 8.1 |
| Frame timing instrumentation | [03 -- Rendering Pipeline](./03-rendering-pipeline.md) | 8.3 |
| Adaptive frame rate and degradation | [03 -- Rendering Pipeline](./03-rendering-pipeline.md) | 8.2 |
| TTS latency budget breakdown | [04 -- TTS Pipeline](./04-tts-pipeline.md) | 10 |
| espeak-ng crash recovery protocol | [04 -- TTS Pipeline](./04-tts-pipeline.md) | 5.5 |
| Control panel accessibility | [05 -- Control Panel](./05-control-panel.md) | 12 |
| Diagnostics panel UI | [05 -- Control Panel](./05-control-panel.md) | 10 |
| FrameTimingSummary IPC type | [05 -- Control Panel](./05-control-panel.md) | 3.4 |
| GPLv3 licensing rationale | [Product Strategy](../PRODUCT_STRATEGY.md) | 8.4 |
| Sustainability and governance | [Product Strategy](../PRODUCT_STRATEGY.md) | 12 |
| Technology license verification | [Tech Stack Evaluation](../TECH_STACK_EVALUATION.md) | 3.1 |
| Platform error types | [02 -- Platform Abstraction](./02-platform-abstraction.md) | 4 |
| Testing strategy (CI benchmarks, quality gates) | [07 -- Testing Strategy](./07-testing-strategy.md) (planned) | (TBD) |
| Build and distribution (signing, SBOM) | [08 -- Build and Distribution](./08-build-and-distribution.md) (planned) | (TBD) |

---

## 10. Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-03-16 | Initial cross-cutting concerns document |
| 1.1 | 2026-03-16 | Post-audit revision: aligned LuminosError hierarchy with canonical doc-02 definition (F-001); fixed Tauri capability JSON to remove non-existent `deny` field (F-002); removed deprecated `copyleft` and `deny` fields from cargo-deny config (F-003); corrected WCAG reduced motion reference from 2.3.1 to 2.3.3 (F-004); corrected LGPL compatibility claim -- LGPL is compatible with GPLv3, added to allowlist (F-005); fixed `get_system_info` phase from Phase 0 to Phase 3 per doc-05 (F-006); corrected flicker threshold to WCAG 2.x language (F-007); corrected NVDA license to GPL-2.0-or-later (F-008); attributed zero-cost logging to `log` crate features, not `env_logger` (F-009); documented `LUMINOS_LOG` custom env var setup requirement (F-010); added memory budget supersession note (P-001); added interim signing key management for Year 1 (P-002); added Piper post-fork GPL-3.0 license caveat (P-003); updated SPDX identifiers to current format (P-004); marked TtsTimings/get_tts_timings as proposed (P-005) |
