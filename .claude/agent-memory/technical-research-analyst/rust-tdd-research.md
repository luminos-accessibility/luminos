# Rust TDD Research Notes (2026-03-21)

## Key Sources
- nexte.st/docs/design/why-process-per-test/ - nextest architecture rationale
- sunshowers.io/posts/nextest-process-per-test/ - nextest maintainer's explanation
- reddit.com/r/rust/comments/1qzjjeg/ - Community TDD discussion (Nov 2024, 116 upvotes)
- jorgeortiz.dev/posts/rust_unit_testing_* - Comprehensive multi-part testing series
- dasroot.net/posts/2026/03/rust-testing-patterns-reliable-releases/ - Testing patterns 2026
- lib.rs/crates/proptest, rstest, insta, mockall, pretty_assertions - Crate docs

## Testing Crate Recommendations for Luminos
| Crate | Version | Purpose | When to Use |
|-------|---------|---------|-------------|
| mockall | 0.13 | Trait mock generation | Interaction verification only |
| proptest | 1.x | Property-based testing | Algorithmic invariants |
| rstest | 0.26 | Fixtures + parameterized | Multi-input test cases |
| insta | 1.x | Snapshot testing | Serialization stability |
| pretty_assertions | 1.x | Better diff output | Every test module |
| assert_cmd | 2.x | Binary testing | E2E/smoke tests only |

## Test Double Taxonomy in Luminos Context
- Luminos MockScreenCapture etc. are FAKES (working simplified implementations), not mocks
- Factory closure pattern for error injection solves non-Clone error type problem
- mockall generates MOCKS (behavior verification) - use sparingly
- Prefer fakes (state verification) over mocks (behavior verification) per Martin Fowler

## cargo nextest Key Facts
- wgpu explicitly uses nextest because EGL needs one GPU context per process
- Process-per-test model is mandatory for Luminos GPU shader testing
- 35% speedup in CI benchmarks (depot.dev, warm sccache)
- Filterset DSL: test(~name), package(name), binary(name)
- Does NOT support doctests - run `cargo test --doc` separately
- nextest.toml supports per-test timeout overrides and retry profiles

## Async Testing Pitfalls
- Never nest tokio runtimes (double runtime = panic)
- Use `#[tokio::test(start_paused = true)]` for deterministic time control
- Use `tokio::time::timeout()` to prevent hanging channel tests
- `std::thread::sleep()` in async tests = bad, use `tokio::time::sleep()`

## What NOT to Test in Rust (compiler catches these)
- Type safety (passing wrong types)
- Null handling (Option forces matching)
- Data races (Send/Sync enforcement)
- Missing match arms (exhaustive matching)
- Lifetime violations

## What TO Test in Rust
- Business logic correctness (wrong arithmetic, off-by-one)
- Boundary conditions (cursor at display edge, zoom extremes)
- Error variant correctness (right error for right condition)
- Integration contracts (subsystem boundary behavior)
- State machine transitions
- Performance regressions
