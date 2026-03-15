# Technical Auditor Memory

## Project: Luminos
- Product strategy doc at `/Users/oliveren/Development/luminos/specs/PRODUCT_STRATEGY.md`
- Tech stack eval at `/Users/oliveren/Development/luminos/specs/TECH_STACK_EVALUATION.md`
- SDD guide at `/Users/oliveren/Development/luminos/specs/README.md`
- README audit at `/Users/oliveren/Development/luminos/README_AUDIT_REPORT.md`
- NOTE: CLAUDE.md Repository Structure section was updated 2026-03-14 to correctly reference `specs/` directory

## Key Verified Facts (Accessibility Market)
- WHO: 2.2 billion people with vision impairment globally (fact sheet, current as of Feb 2026 update)
- IAPB 2020: 1.1B total (43M blind, 295M MSVI, 258M mild, 510M near vision)
- IAPB 2050 projection: ~1.7B (not 1.8B as some docs claim)
- 90% of visually impaired in low/middle-income countries
- $411B annual productivity loss (WHO, purchasing power parity)
- CDC: ~7 million visually impaired in US (not 7.6M as sometimes cited)
- WebAIM SR Survey #10 (2024): JAWS=40.5% primary, NVDA=37.7% primary; NVDA=65.6% commonly used, JAWS=60.5%

## Pricing Facts (as of 2025-2026, verified via store.vispero.com)
- ZoomText Magnifier perpetual: $905
- ZoomText Mag/Reader perpetual: $1,259
- JAWS Professional perpetual: $2,316.50
- Fusion Home perpetual: $2,309
- Fusion Professional perpetual: $3,262
- SuperNova Magnifier (business): $595; Mag&Reader (business): $1,700 (Boundless AT)
- eStore URLs now redirect: store.freedomscientific.com -> store.vispero.com

## Common Errors Found in This Domain
- Piper TTS is effectively **GPL** (not MIT) due to espeak-ng dependency; development moved to OHF-Voice/piper1-gpl
- ZoomText max zoom is **36x** on current versions; 60x was only on Windows 8 (ZoomText 10.1)
- Section 508 refresh references **WCAG 2.0** AA, not 2.1
- WebAIM has TWO low vision surveys (2013 and 2018) with different stats; easy to conflate
- SuperNova USB mode is being discontinued (won't work on Windows 11)
- **Kokoro supports ~9 lang codes / ~8 languages, NOT 30+.** The 30+ figure is Piper's. Easy to confuse when migrating from Piper to Kokoro.
- macOS Tahoe = macOS 26 (released Sep 2025); succeeded Sequoia (macOS 15). Version jump 15->26 intentional.

## Technical Facts
- scap crate: v0.1.0-beta.1 on crates.io, MIT; docs.rs build failed
- leptess crate: v0.14.0, last updated ~3 years ago; possibly unmaintained
- Piper TTS: archived Oct 6 2025, moved to OHF-Voice/piper1-gpl; supports ~30+ languages
- screenpipe: ~17K stars (17,043 as of Mar 2026), uses Tauri + Rust + TypeScript
- macOS ScreenCaptureKit mandatory from macOS 15 (Sequoia); CGWindowListCreateImage deprecated
- Tauri WebkitGTK rendering issues on Linux are well-documented (especially Nvidia)
- wgpu transparent overlay windows possible via winit (confirmed via StackOverflow)

## Crate Version Facts (verified 2026-03-13)
- wgpu: v28.0.0 (latest), 17.9M total downloads, MIT/Apache 2.0
- winit: v0.30.13 (latest), 34.3M total downloads, Apache 2.0
- xcap: v0.9.1, Apache 2.0; 19 reverse deps (not 45); primary for ALL platforms, windows-capture is Windows fallback
- xcap X11: uses `xcb` crate with `randr` feature only -- NOT `shm`; likely uses non-SHM capture
- sherpa-rs: v0.6.8, MIT, 52K total downloads
- sherpa-onnx: 10.8K GitHub stars, Apache 2.0
- ort: v2.0.0-rc.12 (latest, recommended for new projects); NOT "1.16+"
- cpal: v0.17.3, Apache 2.0, 11.2M total downloads
- arboard: v3.6.1, MIT/Apache 2.0, 23.4M total downloads (by 1Password)
- atspi: v0.29.0, MIT/Apache 2.0, 6.5M total downloads (by Odilia project)
- rdev: v0.5.3, MIT, 340K total downloads, last updated Jun 2023
- kokoroxide: v0.1.5, MIT/Apache 2.0, 1.5K total downloads
- x11rb: v0.13.2, MIT/Apache 2.0, 32.3M total downloads

## TTS Model Facts (verified 2026-03-13)
- Kokoro-82M: Apache 2.0 (model weights), uses misaki for G2P, espeak-ng as fallback
- Kokoro ONNX repo (onnx-community/Kokoro-82M-v1.0-ONNX): 1.45GB total (multiple quantizations)
- Kokoro PyTorch model: ~327MB; fp32 ONNX likely similar; "~80MB" only possible for q4/q8
- **Kokoro languages (as of current GitHub README): 9 lang codes (a/b/e/f/h/i/j/p/z); ~8 unique languages**
  - Korean (k) was previously listed but is NO LONGER on the official GitHub README
  - TECH_STACK_EVALUATION.md lists 10 codes including Korean -- may be stale
- misaki G2P: RELEASED on PyPI, active GitHub repo; NOT "unreleased"
- Supertonic: 66M params, 5 languages, MIT code + OpenRAIL-M model weights (NOT MIT/Apache)
- Supertonic RTF benchmarks are from M4 Pro and RTX 4090, NOT Raspberry Pi 4

## GPL Subprocess Isolation
- FSF FAQ: pipes/sockets are "normally" separate programs, BUT "intimate semantics" can make it combined
- "This is a legal question, which ultimately judges will decide" -- FSF's own words
- Simple text-in/phonemes-out is likely safe; complex structured exchange is riskier
- Report should rate legal clarity as "Medium" not "High"

## SDD Methodology Facts (verified 2026-03-13)
- Three artifacts per story: STORY.md, DESIGN.md, SUBTASKS.md
- Workflow: Specify -> Design -> Implement (TDD) -> Review & Close
- GitHub spec-kit & Kiro have similar but not identical artifact naming
- ThoughtWorks Tech Radar Vol 33: SDD is a key 2025 practice
- NOTE: `specs/SDD_AUDIT_REPORT.md` referenced in agent memory but file does NOT exist in repo

## Audit Patterns
- Market size reports vary wildly across research firms; always cross-reference
- Regulatory claims need precise version numbers (WCAG 2.0 vs 2.1 matters)
- Open-source license claims must be verified against actual dependency trees
- "Mature" for Rust crates needs distinction between stable-but-unmaintained vs actively-maintained
- Survey statistics must be traced to exact survey edition, not just the series name
- Crate feature flags matter: dependency on a crate does NOT mean all features are enabled
- TTS benchmarks must specify hardware; RPi4 vs desktop vs GPU numbers are not comparable
- HuggingFace model sizes depend on quantization; always specify which variant
- **When TTS engine changes (Piper->Kokoro), language count claims often carry over incorrectly**
- **Project structure in READMEs must be verified against actual filesystem, not just source docs**
- **CLAUDE.md file paths can drift from actual locations when files are reorganized into subdirs**
- README github links: `../../issues` is the correct relative path pattern for root-level README on GitHub
