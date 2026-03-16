# Principal Product Manager - Agent Memory

## Project: Luminos
- GPLv3 cross-platform screen magnification + TTS accessibility suite
- Pre-development research phase (no app code yet)
- Canonical product definition: `specs/PRODUCT_STRATEGY.md` v1.3
- Tech strategy: `specs/tech-strategy/01-05` (architecture, platform, rendering, TTS, control panel)
- Docs 06-10 referenced but DO NOT EXIST yet (cross-cutting, testing, build, roadmap, risk)

## Key Product-Tech Alignment Findings (2026-03-16 Review)
- See detailed review in conversation; summary in `product-tech-alignment.md`
- Phase 0-2 coverage: Excellent. All P0/P1 features fully specified.
- Phase 3-4 gaps: Plugin architecture (P0, zero spec), font re-rendering (P0, research only), OCR (P0, no trait/spec), AI image desc (P1, zero spec)
- CI/CD is Phase 0 P0 but no doc exists (needs docs 07+08)
- First-run experience, meta-accessibility (WCAG 2.2 AA for own UI), update mechanism: unspecified
- TTS latency budget worst-case 221ms exceeds 200ms target - needs resolution
- Adoption metrics (MAU, geographic) require opt-in telemetry not yet designed
- i18n/RTL not in tech strategy; recommend i18n-ready structure from Phase 0

## Platform Priority Order
Linux X11 -> Linux Wayland -> macOS -> OpenBSD -> Windows

## User Personas (Section 6)
1. Margaret (62, AMD, Windows, zero-config need)
2. David (34, RP/tunnel vision, Ubuntu+Windows, cross-platform power user)
3. Amara (19, albinism/photophobia, low-spec 4GB RAM, cost-sensitive)
4. Robert (45, IT admin, enterprise deployment, config-as-code)
5. Dr. Fatima (38, AT specialist, Arabic UI/RTL, portable profiles)

## Performance Targets
- 60fps (16ms frame time) on integrated GPUs
- <4GB RAM, <200ms TTS latency, <50MB binary, <2s startup
