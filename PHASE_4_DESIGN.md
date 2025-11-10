# Phase 4: Statistical Analysis - Design Document

**Phase**: 4
**Status**: In Progress
**Goal**: Extract actionable insights from 580 measurements across 150 jobs

---

## Objectives

1. **Quantify factor importance**: Which optimization parameters matter most?
2. **Compare workload types**: Statistical differences between CPU/Memory/I/O workloads
3. **Generate recommendations**: Evidence-based guidance for practitioners
4. **Visualize tradeoffs**: Speed vs size Pareto frontiers
5. **Validate significance**: Ensure speedups are statistically meaningful

---

## Architecture

### Module Structure

```
crates/
  analysis/              # New crate for statistical analysis
    src/
      lib.rs            # Public API
      bayesian.rs       # Bayesian inference
      frequentist.rs    # ANOVA, t-tests, confidence intervals
      visualization.rs  # Data preparation for plotting
      recommendations.rs # Generate practical guidance
    Cargo.toml

  harness/
    src/
      bin/
        analyze_results.rs     # Main analysis driver
        generate_report.rs     # Generate markdown report
```

### Dependencies

- **Statistics**: No external stats crates (implement ourselves for TDD)
- **Visualization prep**: Generate CSV/JSON for external plotting (matplotlib, R)
- **Keep it simple**: Focus on core statistical methods

---

## Statistical Methods

### 1. Bayesian Analysis

**Goal**: Estimate factor importance and interaction effects

**Approach**:
- Model: speedup ~ opt_level + lto + codegen_units + ...
- Use conjugate priors for tractability
- Compute posterior distributions for each factor
- Rank factors by effect size

**Output**:
- Factor importance rankings
- Posterior mean and credible intervals
- Interaction effects (e.g., LTO × codegen-units)

**Implementation**:
- Simple Bayesian linear regression
- Normal-inverse-gamma conjugate prior
- Analytical posterior (no MCMC needed for simplicity)

### 2. Frequentist Methods

#### ANOVA (Analysis of Variance)

**Goal**: Test if workload types have significantly different speedups

**Approach**:
- Group benchmarks by workload type (CPU recursive, CPU iterative, Memory, I/O)
- One-way ANOVA: F-test for group differences
- Post-hoc pairwise comparisons (Tukey HSD)

**Output**:
- F-statistic and p-value
- Which workload types differ significantly
- Effect sizes (η²)

#### T-Tests

**Goal**: Compare specific profile pairs

**Approach**:
- Paired t-tests (same benchmarks, different profiles)
- Welch's t-test (unequal variances)
- Multiple comparison correction (Bonferroni)

**Output**:
- Which profiles are significantly different
- Confidence intervals for speedup differences
- Effect sizes (Cohen's d)

#### Confidence Intervals

**Goal**: Quantify uncertainty in speedup estimates

**Approach**:
- Bootstrap confidence intervals (percentile method)
- 95% CI for each profile's average speedup
- 95% CI for pairwise differences

**Output**:
- CI for each profile ranking
- Overlapping CIs indicate non-significant differences

### 3. Correlation Analysis

**Goal**: Identify optimization parameter synergies

**Approach**:
- Pearson correlation between parameters and speedup
- Partial correlations (control for other factors)
- Correlation matrix visualization prep

**Output**:
- Which parameters correlate with speedup
- Which parameters interact (correlation of products)

---

## Visualization Data Preparation

### 1. Pareto Frontiers (Speed vs Size)

**Goal**: Show optimal speed/size tradeoffs

**Data needed**:
- Measure binary sizes for all 150 jobs
- Plot speedup vs binary size
- Identify Pareto-optimal configurations

**Output**: CSV for plotting
```csv
benchmark,profile,speedup,binary_size_kb
fibonacci,lto-fat,2.68,1834
...
```

### 2. Heatmaps (Benchmark × Profile)

**Goal**: Visual comparison of all combinations

**Data format**: Matrix CSV
```csv
,baseline,standard-release,lto-thin,lto-fat,...
ackermann,1.0,5.95,5.97,5.97,...
fibonacci,1.0,2.25,2.25,2.25,...
...
```

### 3. Box Plots (Variance Analysis)

**Goal**: Show measurement distributions

**Data format**: Long-form CSV
```csv
benchmark,profile,iteration,compute_us
ackermann,baseline,1,144628
ackermann,baseline,2,146723
...
```

### 4. Bar Charts (Profile Rankings)

**Goal**: Compare average speedups with error bars

**Data format**: Summary CSV with confidence intervals
```csv
profile,mean_speedup,ci_lower,ci_upper
lto-fat,15.06,12.34,17.78
...
```

---

## Recommendations Engine

### Workload-Specific Recommendations

**Input**: Benchmark workload type
**Output**: Top 3 profiles with expected speedup and confidence

Example:
```
For CPU-bound iterative workloads:
1. perf-ultra: 20-30x speedup (95% CI: 18-32x)
2. lto-fat: 18-28x speedup (95% CI: 16-30x)
3. lto-thin: 17-27x speedup (95% CI: 15-29x)

Build time consideration:
- perf-ultra: ~30s build
- lto-fat: ~25s build
- lto-thin: ~15s build
```

### General-Purpose Recommendation

**Input**: None (universal)
**Output**: Best overall profile with tradeoff analysis

Example:
```
Best overall profile: lto-fat
- Average speedup: 15.06x (95% CI: 12-18x)
- Works well across all workload types
- Binary size: -51% vs baseline
- Build time: +3x vs standard-release

Alternative if build time matters: lto-thin
- Average speedup: 14.47x (only 4% slower)
- Build time: +2x vs standard-release (33% faster than lto-fat)
```

### Size-Constrained Recommendation

**Input**: Maximum binary size
**Output**: Best profile within size constraint

Example:
```
For binary size < 500KB:
1. size-ultra: 7.34x speedup, 331KB
2. size-z-lto: 9.12x speedup, 445KB
3. opt-s: 12.11x speedup, 892KB (exceeds constraint)
```

---

## Implementation Plan

### Step 1: Create Analysis Crate (TDD)

- [ ] New `analysis` crate
- [ ] Basic statistical functions (mean, stddev, etc.)
- [ ] Comprehensive tests (property-based where applicable)

### Step 2: Implement Frequentist Methods (TDD)

- [ ] ANOVA implementation with tests
- [ ] T-test implementation with tests
- [ ] Confidence intervals (bootstrap) with tests
- [ ] Effect size calculations with tests

### Step 3: Implement Bayesian Analysis (TDD)

- [ ] Bayesian linear regression
- [ ] Conjugate prior/posterior
- [ ] Factor importance ranking
- [ ] Tests with known datasets

### Step 4: Binary Size Measurement

- [ ] Script to measure all 150 binary sizes
- [ ] Add to results JSON
- [ ] Calculate size reduction percentages

### Step 5: Visualization Data Prep

- [ ] Export Pareto frontier CSV
- [ ] Export heatmap matrix CSV
- [ ] Export box plot data CSV
- [ ] Export bar chart data CSV

### Step 6: Recommendations Engine (TDD)

- [ ] Workload classifier
- [ ] Profile ranker by workload
- [ ] Confidence interval incorporation
- [ ] Tradeoff calculator (speed vs size vs build time)

### Step 7: Report Generator

- [ ] Main analysis driver binary
- [ ] Markdown report generator
- [ ] Integrate all analyses
- [ ] Executive summary

---

## Test Strategy

### Unit Tests

- All statistical functions tested individually
- Known dataset validation (e.g., textbook ANOVA examples)
- Edge cases (zero variance, single sample, etc.)

### Property-Based Tests

- ANOVA F-statistic properties
- Confidence interval coverage (should contain true mean 95% of time)
- Correlation invariance to scaling

### Integration Tests

- End-to-end on synthetic data
- Verify report generation
- Validate CSV export formats

### Quality Targets

- Coverage: ≥85%
- Mutation score: ≥85%
- TDG Grade: A
- Zero clippy warnings

---

## Deliverables

### Code
1. `crates/analysis/` - Statistical analysis crate (~800 lines, ~40 tests)
2. `analyze_results` binary - Main driver
3. `generate_report` binary - Report generator

### Data
1. `binary_sizes.json` - All binary sizes measured
2. `pareto_frontier.csv` - Speed vs size data
3. `heatmap_data.csv` - Benchmark × profile matrix
4. `variance_data.csv` - Box plot data
5. `ranking_data.csv` - Profile rankings with CIs

### Documentation
1. `STATISTICAL_ANALYSIS_REPORT.md` - Full analysis report
2. `RECOMMENDATIONS.md` - Practical guidance
3. `VISUALIZATION_GUIDE.md` - How to plot the CSVs

### Expected Insights

1. **Factor importance**: LTO > opt-level > codegen-units > target-cpu
2. **Workload differences**: Memory-bound benefits 3x more than CPU-bound
3. **Interactions**: LTO × codegen-units shows significant interaction
4. **Recommendations**: Clear guidance for each workload type
5. **Pareto frontier**: Identify optimal speed/size configurations

---

## Timeline Estimate

- Step 1 (crate setup): ~30 min
- Step 2 (frequentist): ~2 hours
- Step 3 (Bayesian): ~2 hours
- Step 4 (binary sizes): ~30 min
- Step 5 (viz prep): ~1 hour
- Step 6 (recommendations): ~1.5 hours
- Step 7 (report gen): ~1 hour

**Total**: ~8.5 hours of focused work

---

## Success Criteria

✅ All statistical methods implemented with tests
✅ 85%+ test coverage maintained
✅ Binary sizes measured for all configurations
✅ CSV exports ready for plotting
✅ Comprehensive report generated
✅ Actionable recommendations produced
✅ TDG Grade A maintained

---

**Status**: Design complete, ready for implementation
**Next**: Create analysis crate and implement basic statistical functions
