# Technical Auditor Memory

## Project: Luminos
- Product strategy doc at `/Users/oliveren/Development/luminos/specs/PRODUCT_STRATEGY.md`
- Tech stack eval at `/Users/oliveren/Development/luminos/specs/TECH_STACK_EVALUATION.md`
- SDD guide at `/Users/oliveren/Development/luminos/specs/README.md`
- Tech strategy overview at `/Users/oliveren/Development/luminos/specs/tech-strategy/README.md`
- README at `/Users/oliveren/Development/luminos/README.md`
- CLAUDE.md at `/Users/oliveren/Development/luminos/CLAUDE.md`
- NOTE: CLAUDE.md Repository Structure updated 2026-03-15 to reference `specs/` dir, `tech-strategy/` subdir, and v1.3
- NOTE: Product strategy is v1.3 as of 2026-03-15 (GPLv3 + Linux-first pivot)

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

## Common Errors Found in This Domain
- Piper TTS is effectively **GPL** (not MIT) due to espeak-ng dependency; moved to OHF-Voice/piper1-gpl
- ZoomText max zoom is **36x** on current versions; 60x was only on Windows 8 (ZoomText 10.1)
- Section 508 refresh references **WCAG 2.0** AA, not 2.1
- WebAIM has TWO low vision surveys (2013 and 2018) with different stats; easy to conflate
- SuperNova USB mode is being discontinued (won't work on Windows 11)
- **Kokoro supports ~9 lang codes / ~8 languages, NOT 30+.** The 30+ figure is Piper's.
- macOS Tahoe = macOS 26 (released Sep 2025); succeeded Sequoia (macOS 15).

## Crate Version Facts (verified 2026-03-13)
- wgpu: v28.0.0, 17.9M total downloads, MIT/Apache 2.0
- winit: v0.30.13, 34.3M total downloads, Apache 2.0
- xcap: v0.9.1, Apache 2.0; 19 reverse deps; non-SHM X11 capture
- sherpa-rs: v0.6.8, MIT, 52K total downloads
- sherpa-onnx: 10.8K GitHub stars, Apache 2.0
- cpal: v0.17.3, Apache 2.0, 11.2M total downloads
- arboard: v3.6.1, MIT/Apache 2.0, 23.4M total downloads
- atspi: v0.29.0, MIT/Apache 2.0, 6.5M total downloads

## TTS Model Facts (verified 2026-03-13)
- Kokoro-82M: Apache 2.0 (model weights); PyTorch ~327MB; q4/q8 ~80MB
- **Kokoro languages: 9 lang codes (a/b/e/f/h/i/j/p/z); ~8 unique languages**
  - Korean (k) was previously listed but removed from official GitHub README
  - TECH_STACK_EVALUATION.md lists 10 codes including Korean -- may be stale
- misaki G2P: RELEASED on PyPI, active GitHub repo
- Supertonic: 66M params, 5 languages, MIT code + OpenRAIL-M model weights (NOT MIT/Apache)

## GPL Subprocess Isolation
- FSF FAQ: pipes/sockets "normally" separate programs, BUT "intimate semantics" can make combined
- With GPLv3 project license, subprocess isolation is engineering choice, not legal necessity

## Cross-Document Consistency Audit (2026-03-15)
- v1.2 -> v1.3 version refs were stale in CLAUDE.md and README.md (FIXED)
- tech-strategy/ directory missing from CLAUDE.md and README.md project structure (FIXED)
- "GPL isolation" label stale in tech-strategy/README.md doc 04 scope (FIXED to "subprocess isolation")
- Anti-metric "Revenue (Year 1-2)" mildly tensions with Section 12.1.2 Year 1 revenue targets (FLAGGED)
- All 6 docs consistent on: GPLv3 license, Linux-first platform order, phase structure, tech stack, perf targets

## Audit Patterns
- Market size reports vary wildly across research firms; always cross-reference
- Regulatory claims need precise version numbers (WCAG 2.0 vs 2.1 matters)
- Open-source license claims must be verified against actual dependency trees
- Survey statistics must be traced to exact survey edition, not just the series name
- TTS benchmarks must specify hardware; RPi4 vs desktop vs GPU numbers are not comparable
- HuggingFace model sizes depend on quantization; always specify which variant
- **When TTS engine changes (Piper->Kokoro), language count claims often carry over incorrectly**
- **Project structure in READMEs must be verified against actual filesystem**
- **CLAUDE.md file paths can drift from actual locations when files are reorganized**
- **After strategic pivots, version references commonly missed in CLAUDE.md and README project structure**
- **Document scope labels (e.g., "GPL isolation") must be updated when licensing strategy changes**
- README github links: `../../issues` is the correct relative path pattern for root-level README on GitHub
