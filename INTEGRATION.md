# Integration Guide: certeza, trueno, and ruchy-docker

This document explains how this project integrates ideas from three key PAIML projects to create a comprehensive Rust optimization benchmarking framework with extreme testing rigor.

![Optimization Results](docs/images/optimization-speedup.png)

**Figure**: Comprehensive optimization results showing 15× average speedup, SIMD acceleration potential, and proven strategies validated across 580 measurements.

## Overview

This project combines:

1. **certeza** - 3-tiered testing framework for sustainable quality
2. **trueno** - SIMD-accelerated tensor operations (optional benchmark)
3. **ruchy-docker** - Instrumented measurement for compute vs startup time

## 1. Certeza Integration: 3-Tiered Testing

From [../certeza](../certeza), we've adopted the **sustainable testing workflow** that prevents developer burnout while maintaining extreme quality standards.

### Tier 1: ON-SAVE (Sub-second feedback)
```bash
make tier1
```

**What it does:**
- Quick build check (`cargo check`)
- Format verification (`cargo fmt --check`)
- Quick clippy checks (correctness & suspicious only)
- Unit tests only (`cargo test --lib`)

**Time target:** < 10 seconds
**Use case:** Rapid iteration during active development (flow state)

### Tier 2: ON-COMMIT (1-5 minutes)
```bash
make tier2
```

**What it does:**
- Full test suite (unit + integration + doc tests)
- Strict clippy (`-D warnings`)
- SATD violation check (no TODO/FIXME/HACK)
- Quick coverage analysis
- Property-based tests (100 cases per test)

**Time target:** 1-5 minutes
**Use case:** Pre-commit validation, ensures code is ready for team review

### Tier 3: ON-MERGE/NIGHTLY (Hours)
```bash
make tier3
```

**What it does:**
- Comprehensive mutation testing (≥85% mutation score)
- Full coverage analysis with HTML reports
- Complete benchmark suite
- Performance regression detection

**Time target:** Hours (suitable for CI/CD or nightly runs)
**Use case:** Final validation before merging to main, comprehensive quality assurance

### Why Tiered Testing?

From certeza's philosophy:

> **Critical Principle**: Different verification techniques operate at different time scales. Fast feedback enables flow; slow feedback causes context switching waste. Never run mutation testing or formal verification in the inner development loop.

This approach prevents:
- ❌ **Muri (overburden)**: Developers waiting minutes for simple checks
- ❌ **Muda (waste)**: Context switching during long test runs
- ❌ **Mura (irregularity)**: Inconsistent quality practices

And enables:
- ✅ **Sustainable quality**: Fast feedback for frequent checks
- ✅ **Flow state**: Sub-second tier1 doesn't break concentration
- ✅ **Comprehensive validation**: Tier 3 catches everything before merge

## 2. Trueno Integration: SIMD Benchmarking (Optional)

From [../trueno](../trueno), we've created an optional benchmark demonstrating SIMD-accelerated matrix operations.

### Why Optional?

Trueno v0.4.1 uses unstable AVX-512 intrinsics that require **nightly Rust**. Since the main project uses stable Rust 1.83.0, the trueno benchmark is excluded from the default workspace but available for advanced users.

### Building the Trueno Benchmark

```bash
# Switch to nightly
rustup default nightly

# Build trueno-simd benchmark
cargo build --release --manifest-path benchmarks/trueno-simd/Cargo.toml

# Run it
./target/release/trueno-simd

# Switch back to stable
rustup default stable
```

### What It Demonstrates

The `benchmarks/trueno-simd` benchmark showcases:

1. **SIMD Performance**
   - Matrix multiplication with AVX-512/AVX2/SSE2 acceleration
   - Vector dot product with SIMD
   - Element-wise operations (add, mul, sum)

2. **Instrumented Measurement** (ruchy-docker style)
   - Separate startup time vs compute time
   - Multiple benchmark phases with individual timing
   - SIMD backend detection (AVX-512/AVX2/SSE2/Scalar)

3. **Comprehensive Testing** (certeza style)
   - **Tier 1**: Unit tests for correctness
     - Identity matrix properties
     - Orthogonal/parallel vector tests
     - Commutative/associative properties
   - **Tier 2**: Property-based tests with proptest
     - Dot product distributive property
     - Matrix transpose involution
     - Identity multiplication invariants
     - Sum reduction correctness

### Example Output

```
STARTUP_TIME_US: 42
MATMUL_TIME_US: 1234
DOT_TIME_US: 89
VECOPS_TIME_US: 156
TOTAL_COMPUTE_US: 1479
RESULT_MATMUL: 42.000000
RESULT_DOT: 333300000.000000
RESULT_SUM: 299990000.000000
SIMD_BACKEND: AVX2
```

### Performance Expectations

From trueno's benchmarks:

| Operation | Scalar | SSE2 | AVX2 | AVX-512 |
|-----------|--------|------|------|---------|
| Dot Product | baseline | 3.4x | 6.2x | 11.9x |
| Matrix Multiply (128×128) | baseline | 2x | 4x | 7x |
| Vector Sum | baseline | 3.1x | 5x | 8x |

## 3. Ruchy-Docker Integration: Instrumented Measurement

From [../ruchy-docker](../ruchy-docker), we've adopted the **instrumented measurement approach** that separates startup overhead from pure compute time.

### Why Instrumented Measurement?

Docker benchmarking (and binary benchmarking in general) must distinguish:
- **Startup time**: Binary loading, initialization, memory allocation
- **Compute time**: The actual algorithm execution

Without this separation, you're measuring:
```
Total Time = Startup + Compute + Teardown
```

With instrumented measurement:
```rust
let t0 = Instant::now();  // Before any work
// ... initialization code ...
let t1 = Instant::now();  // After initialization, before compute
// ... actual computation ...
let t2 = Instant::now();  // After computation

println!("STARTUP_TIME_US: {}", t1.duration_since(t0).as_micros());
println!("COMPUTE_TIME_US: {}", t2.duration_since(t1).as_micros());
```

### Example: Matrix Multiplication

Our `benchmarks/matrix-mult/src/main.rs` uses this pattern:

```rust
let t0 = Instant::now();
let t1 = Instant::now();
let c = matrix_multiply(&a, &b);  // The actual benchmark
let t2 = Instant::now();

println!("STARTUP_TIME_US: {}", t1.duration_since(t0).as_micros());
println!("COMPUTE_TIME_US: {}", t2.duration_since(t1).as_micros());
println!("RESULT: {}", c[0][0]);  // For verification
```

### Benefits

1. **Isolates optimization impact**: See if optimizations affect startup or compute
2. **Cache effects visible**: First run vs subsequent runs
3. **Verification built-in**: `RESULT` output allows correctness checking
4. **Statistical rigor**: Multiple measurements of compute time only

## Installation & Quick Start

### 1. Install Required Tools

```bash
make install-tools
```

This installs:
- `cargo-llvm-cov` - Coverage analysis
- `cargo-nextest` - Fast test runner
- `cargo-mutants` - Mutation testing
- `cargo-watch` - Watch mode for development

### 2. Run Tiered Tests

```bash
# During development (fast)
make tier1

# Before committing (comprehensive)
make tier2

# Before merging (exhaustive) - only in CI/CD
make tier3
```

### 3. View Available Commands

```bash
make help
```

## Quality Standards

From certeza's quality gates configuration (`.pmat-gates.toml`):

- **Test Coverage**: ≥85% (target: 95%)
- **Mutation Score**: ≥85% (critical paths: 95%)
- **Cyclomatic Complexity**: ≤15
- **Cognitive Complexity**: ≤20
- **SATD Violations**: 0 (zero tolerance for TODO/FIXME/HACK)
- **Dead Code**: 0%
- **Clippy Warnings**: 0
- **TDG Grade**: A minimum (target: A+)

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                   Rust Optimization Benchmarking            │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │   certeza    │  │    trueno    │  │ ruchy-docker │    │
│  │ (3-tier TDD) │  │(SIMD accel)  │  │ (instrumented│    │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘    │
│         │                  │                  │             │
│         v                  v                  v             │
│  ┌─────────────────────────────────────────────────────┐  │
│  │            Makefile (tier1/tier2/tier3)             │  │
│  └─────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌───────────────────────────────────────────────────┐    │
│  │  Benchmarks (10 workloads + trueno-simd)          │    │
│  │  - Fibonacci, Matrix Mult, Quicksort, etc.        │    │
│  │  - All use instrumented measurement               │    │
│  │  - Optional trueno SIMD acceleration              │    │
│  └───────────────────────────────────────────────────┘    │
│                                                             │
│  ┌───────────────────────────────────────────────────┐    │
│  │  Quality Infrastructure                            │    │
│  │  - Property-based testing (proptest)              │    │
│  │  - Mutation testing (cargo-mutants)               │    │
│  │  - Coverage (cargo-llvm-cov)                      │    │
│  │  - Statistical analysis                           │    │
│  └───────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## File Structure

```
compiled-rust-benchmarking/
├── Makefile                   # Tiered testing targets (certeza-style)
├── .pmat-gates.toml          # Quality gates configuration
├── INTEGRATION.md            # This file
│
├── benchmarks/
│   ├── matrix-mult/          # Instrumented measurement (ruchy-docker)
│   ├── fibonacci/            # All benchmarks use instrumented timing
│   ├── quicksort/
│   └── trueno-simd/          # Optional SIMD benchmark (requires nightly)
│       ├── Cargo.toml        # Excluded from workspace
│       └── src/main.rs       # Comprehensive tests + instrumentation
│
├── crates/
│   ├── harness/              # Benchmark execution framework
│   ├── stats/                # Statistical analysis
│   └── analysis/             # Result analysis
│
└── target/
    ├── coverage/             # Generated by tier2
    └── mutants.txt           # Generated by tier3
```

## Development Workflow

### Daily Development

```bash
# 1. Make changes to code
# 2. Run tier1 frequently (sub-second)
make tier1

# 3. When ready to commit
make tier2
git add .
git commit -m "Your message"
```

### Before Creating PR

```bash
# Run tier2 one final time
make tier2

# Optional: Run specific benchmark
make bench-trueno  # If using nightly
```

### CI/CD (Automated)

```bash
# On merge to main
make tier3

# This runs:
# - Mutation testing (≥85% score required)
# - Full coverage (≥85% required)
# - All benchmarks
# - Performance regression detection
```

## References

- [certeza](../certeza) - Testing rigor framework
- [trueno](../trueno) - SIMD tensor operations
- [ruchy-docker](../ruchy-docker) - Docker runtime benchmarking

## FAQ

### Q: Why is trueno-simd excluded from the workspace?

A: Trueno v0.4.1 uses unstable AVX-512 intrinsics (`stdarch_x86_avx512`) that require nightly Rust. The main project uses stable Rust 1.83.0 for reliability. Advanced users can build the trueno benchmark separately with nightly.

### Q: How long does each tier take?

- **Tier 1**: ~7-10 seconds (fast enough for rapid iteration)
- **Tier 2**: ~1-5 minutes (acceptable pre-commit delay)
- **Tier 3**: ~1-3 hours (only for CI/CD or nightly runs)

### Q: Can I skip tiers during development?

Yes! The tiered approach is designed for flexibility:
- **Flow state coding**: Use tier1 frequently
- **Taking a break**: Run tier2 while grabbing coffee
- **Final validation**: Let CI/CD handle tier3

### Q: Do all benchmarks use instrumented measurement?

Yes! Following ruchy-docker's pattern, all benchmarks output:
- `STARTUP_TIME_US`: Initialization overhead
- `COMPUTE_TIME_US`: Pure algorithm execution
- `RESULT`: For verification

This allows isolating optimization impact on startup vs compute.

## Contributing

When adding new benchmarks or features:

1. ✅ Follow instrumented measurement pattern (ruchy-docker)
2. ✅ Add unit tests (tier1)
3. ✅ Add property-based tests (tier2) if applicable
4. ✅ Run `make tier2` before committing
5. ✅ Document SIMD expectations if using trueno patterns

## License

MIT - Same as parent project.
