---
name: Versioning & 1.0.0 Milestone
description: Luminos uses lockstep SemVer; 1.0.0 = production-ready X11 magnification at end of Phase 1
type: project
---

Luminos 1.0.0 will be cut when screen magnification is fully stable, working, and production-ready with all features on Linux X11 systems. This maps to end of Phase 1 (epics E1-E7 complete and stable). Wayland support (E8) may ship alongside 1.0.0 but is NOT a gate -- X11 stability is the only hard requirement.

**Why:** The user defined 1.0.0 as the point where the core product (magnification) is usable for daily use by low-vision users on the primary platform (Linux X11). TTS, macOS, and other platforms are post-1.0 features delivered as 1.x.y minor releases. Wayland can be included if ready but doesn't block the release.

**How to apply:** When discussing releases, versioning, or milestone planning, 1.0.0 is not about TTS or cross-platform -- it's about X11 magnification maturity. Phase 2+ features (TTS, macOS, Windows) go into 1.x.y minor bumps. Wayland is a special case: it's a Phase 1 epic that may or may not be stable at 1.0.0 -- it does not gate the release. Pre-1.0 versions (0.1.x through 0.9.x) cover Phases 0-1 only.
