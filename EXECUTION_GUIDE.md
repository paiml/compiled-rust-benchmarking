# Pathfinder Study Execution Guide

**Purpose**: Guide for executing the full pathfinder study with statistical rigor

## Overview

The pathfinder study executes **150 jobs** (10 benchmarks × 15 profiles) with multiple iterations per job to collect statistically significant performance data.

## Execution Tool

### Full Pathfinder Execution

```bash
cargo run --bin full-pathfinder-execution
```

**What it does**:
- Builds all 150 benchmark×profile combinations
- Executes each job 3-10 times (adaptive based on stability)
- Collects measurements until CV < 10% or max iterations reached
- Exports results to `pathfinder_results.json`
- Shows real-time progress and ETA

**Expected Duration**: 1-3 hours (depending on hardware)

**Output Format**:
```
┌────────────────────────────────────────────────────────┐
│ [  1/150] fibonacci          baseline            ✓  5 iter,   465123μs, CV: 1.2% │ ETA: 3600s
│ [  2/150] fibonacci          lto-fat              ✓  3 iter,   210337μs, CV: 0.9% │ ETA: 3550s
│ [  3/150] fibonacci          opt-s                ~  7 iter,   184750μs, CV:12.1% │ ETA: 3500s
```

**Status Indicators**:
- `✓` - Stable (CV < 10%)
- `~` - Unstable (CV > 10% after max iterations)
- `✗` - Failed

### Quick Multi-Benchmark Study

```bash
cargo run --bin multi-benchmark-study
```

**What it does**:
- Tests 5 representative benchmarks × 6 key profiles = 30 jobs
- Single iteration per job (fast validation)
- Displays comparative analysis
- Useful for quick hypothesis testing

**Expected Duration**: 5-10 minutes

## Configuration Parameters

### ExecutionConfig (in full-pathfinder-execution)

```rust
ExecutionConfig {
    iterations: 5,           // Target iterations per job
    max_cv: 0.10,           // 10% coefficient of variation threshold
    min_iterations: 3,       // Minimum iterations before stability check
    max_iterations: 10,      // Maximum iterations (prevent infinite loops)
}
```

**Coefficient of Variation (CV)**:
- CV = (std_dev / mean) × 100%
- CV < 5%: Excellent stability
- CV < 10%: Acceptable stability
- CV > 10%: Unstable (may indicate system noise)

### Iteration Strategy

**Adaptive stopping**:
1. Run minimum 3 iterations
2. After each iteration, calculate CV
3. If CV < 10% and iterations ≥ 3: STOP (stable)
4. If iterations ≥ 10: STOP (max reached)
5. Otherwise: continue

**Benefits**:
- Fast jobs finish quickly (3 iterations if stable)
- Noisy jobs get more data (up to 10 iterations)
- Automatic quality control

## Output Format

### JSON Structure

```json
{
  "results": [
    {
      "job": {
        "benchmark": "fibonacci",
        "config_id": "lto-fat",
        "job_id": "fibonacci-lto-fat"
      },
      "measurements": [
        {
          "startup_us": 0,
          "compute_us": 210337,
          "total_us": 210337,
          "result": "102334155"
        },
        ...
      ],
      "stats": {
        "count": 5,
        "mean_compute_us": 210156.2,
        "median_compute_us": 210337.0,
        "min_compute_us": 208789,
        "max_compute_us": 212559,
        "stddev_compute_us": 1523.7,
        "mean_total_us": 210156.2,
        "result": "102334155"
      }
    },
    ...
  ]
}
```

## Measurement Variance

### Expected Variance by Workload

**CPU-bound benchmarks** (fibonacci, ackermann):
- Expected CV: 1-3%
- Cause: Minimal system interference
- Action: Should stabilize in 3-5 iterations

**Memory-bound benchmarks** (matrix-mult, quicksort):
- Expected CV: 2-5%
- Cause: Cache effects, memory allocation
- Action: May need 5-7 iterations

**I/O-bound benchmarks** (file-io):
- Expected CV: 5-15%
- Cause: Filesystem caching, system I/O
- Action: Often needs 7-10 iterations, may stay unstable

### Demo: Fibonacci Variance

From quick demo runs:

**Baseline profile** (O0):
```
Iteration 1: 460,318μs
Iteration 2: 469,346μs
Iteration 3: 469,421μs
Mean: 466,362μs
Std Dev: 5,204μs
CV: 1.12% ✓ STABLE
```

**LTO Fat profile**:
```
Iteration 1: 212,559μs
Iteration 2: 208,789μs
Iteration 3: 210,663μs
Mean: 210,670μs
Std Dev: 1,885μs
CV: 0.89% ✓ STABLE
```

## Execution Best Practices

### System Preparation

**Before running**:
```bash
# Close unnecessary applications
# Disable background updates
# Run on AC power (not battery)
# Consider setting CPU governor to performance
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Verify no thermal throttling
sensors  # Check CPU temperature
```

### Parallel Execution

**Single-threaded** (current implementation):
```bash
cargo run --bin full-pathfinder-execution
```
- Sequential job execution
- Predictable, reliable
- Takes ~1-3 hours for 150 jobs

**Future: Parallel execution** (not yet implemented):
```bash
# Would run multiple jobs in parallel
# Reduces total time to ~15-30 minutes
# Risk: cross-job interference increases variance
```

### Incremental Execution

**Resume from partial results**:
```bash
# Not yet implemented, but planned:
# cargo run --bin full-pathfinder-execution --resume pathfinder_results.json
```

## Quality Validation

### Statistical Quality Metrics

**Per-job quality**:
- ✓ CV < 5%: Excellent
- ✓ CV < 10%: Good
- ~ CV < 15%: Acceptable
- ✗ CV ≥ 15%: Poor (investigate)

**Overall study quality**:
- Target: ≥80% jobs with CV < 10%
- Acceptable: ≥70% jobs with CV < 15%
- If < 70%: System noise too high, investigate

### Outlier Detection

**Individual measurements** (not yet implemented):
- Flag measurements >3 std devs from mean
- Consider removing outliers if CV improves significantly
- Document outlier removal in methodology

**Jobs with outliers**:
- Review console output during execution
- Check for thermal throttling (CPU temp spikes)
- Check for background processes (system load)

## Analysis Workflow

### Step 1: Collect Data

```bash
cargo run --bin full-pathfinder-execution
# Produces: pathfinder_results.json
```

### Step 2: Quick Analysis

```bash
# Extract key metrics (example jq queries)
# Top 10 fastest configurations
jq -r '.results[] | select(.stats != null) |
    "\(.stats.mean_compute_us) \(.job.benchmark) \(.job.config_id)"' \
    pathfinder_results.json | sort -n | head -10

# Jobs with high variance (CV > 10%)
jq -r '.results[] | select(.stats != null) |
    select((.stats.stddev_compute_us / .stats.mean_compute_us) > 0.1) |
    "\(.job.job_id) CV: \((.stats.stddev_compute_us / .stats.mean_compute_us * 100) | floor)%"' \
    pathfinder_results.json
```

### Step 3: Full Analysis (Phase 4)

- Statistical significance testing
- Pareto frontier generation
- Workload-specific recommendations
- Final report generation

## Troubleshooting

### Build Failures

**Symptom**: Some jobs fail to build
```
✗ Build failed for fibonacci-cpu-native
```

**Cause**: Profile uses target-specific features
**Solution**: Verify profile is valid for your CPU

### Execution Failures

**Symptom**: Jobs build but execution fails
```
✓ Build succeeded
✗ No measurements
```

**Cause**: Benchmark panics or times out
**Solution**:
1. Run manually to see error: `cargo run -p <benchmark> --profile <profile>`
2. Check for stack overflow (increase stack size if needed)
3. Check for out-of-memory (reduce problem size if needed)

### High Variance

**Symptom**: Many jobs with CV > 15%
```
~ 10 iter, 150000μs, CV:18.5%
```

**Causes**:
1. Background processes consuming CPU
2. Thermal throttling
3. Power management (battery saver mode)
4. Filesystem cache effects (I/O benchmarks)

**Solutions**:
1. Close all applications
2. Monitor CPU temperature
3. Use AC power, set performance mode
4. For I/O: Consider dropping filesystem caches between runs

### Slow Execution

**Symptom**: Taking >4 hours for 150 jobs

**Causes**:
1. LTO Fat builds are slow (10x slower than O3)
2. System under load
3. Debug profile builds accidentally

**Solutions**:
1. Normal for LTO Fat (be patient)
2. Check system load: `top` or `htop`
3. Verify using release profiles, not debug

## Next Steps After Execution

### Immediate (Manual Analysis)

1. Review `pathfinder_results.json`
2. Identify top performers per benchmark
3. Check statistical quality (% stable jobs)
4. Document any anomalies

### Phase 4 (Statistical Analysis)

1. Implement automated analysis tools
2. Bayesian inference for factor importance
3. ANOVA across workload types
4. Pareto frontier visualization
5. Generate final recommendations

## Example Output

### Successful Run

```
╔══════════════════════════════════════════════════════════╗
║     Full Pathfinder Study Execution with Iterations     ║
╚══════════════════════════════════════════════════════════╝

Configuration:
  Target iterations: 5
  CV threshold: 10.0%
  Min iterations: 3
  Max iterations: 10

Step 1: Generating pathfinder configurations...
  Selected 15 pathfinder configurations

Step 2: Creating build matrix...
  Total jobs: 150 (10 benchmarks × 15 profiles)
  Estimated measurements: ~750

Step 3: Executing pathfinder study...

┌────────────────────────────────────────────────────────┐
│ [  1/150] ackermann          baseline            ✓  3 iter,   128324μs, CV: 0.5% │ ETA: 3600s
│ [  2/150] ackermann          standard-release    ✓  3 iter,    24022μs, CV: 1.2% │ ETA: 3550s
│ [  3/150] ackermann          lto-fat              ✓  4 iter,    24667μs, CV: 2.1% │ ETA: 3500s
│ ...
│ [150/150] btreemap-ops       size-ultra           ✓  5 iter,   291541μs, CV: 3.8% │ ETA: 0s
└────────────────────────────────────────────────────────┘

╔══════════════════════════════════════════════════════════╗
║                    Execution Summary                     ║
╚══════════════════════════════════════════════════════════╝

Jobs:
  ✓ Completed: 148/150 (98.7%)
  ✗ Failed: 2/150 (1.3%)

Measurements:
  Total collected: 623
  Average per job: 4.2

Performance:
  Total time: 2847.3s
  Jobs per second: 0.05
  Average job time: 19.2s

Statistical Quality:
  Stable jobs (CV < 10%): 135/148 (91.2%)

Step 4: Exporting results...
  ✓ Saved to pathfinder_results.json

✅ Full pathfinder study complete!
   148 jobs executed, 623 measurements collected
```

## Command Reference

```bash
# Full pathfinder study (150 jobs, ~2 hours)
cargo run --bin full-pathfinder-execution

# Quick multi-benchmark comparison (30 jobs, ~10 minutes)
cargo run --bin multi-benchmark-study

# Show build matrix
cargo run --bin show-build-matrix

# Generate configuration files
cargo run --bin generate-configs
```

---

**Status**: Tool implemented and ready for execution. Run `cargo run --bin full-pathfinder-execution` to collect comprehensive performance data for all 150 pathfinder jobs.
