#!/usr/bin/env python3
"""Generate text-based visualizations (no dependencies required)"""

import csv

def bar_chart_horizontal(data, title, max_width=50):
    """Create horizontal bar chart"""
    print(f"\n{title}")
    print("=" * 70)

    max_val = max(val for _, val in data)

    for label, value in data:
        bar_width = int((value / max_val) * max_width)
        bar = '█' * bar_width
        print(f"{label:20s} {bar} {value:.1f}x")
    print()

def table(headers, rows, title=None):
    """Create formatted table"""
    if title:
        print(f"\n{title}")
        print("=" * 70)

    # Calculate column widths
    widths = [len(h) for h in headers]
    for row in rows:
        for i, cell in enumerate(row):
            widths[i] = max(widths[i], len(str(cell)))

    # Print header
    header_line = " | ".join(h.ljust(w) for h, w in zip(headers, widths))
    print(header_line)
    print("-" * len(header_line))

    # Print rows
    for row in rows:
        print(" | ".join(str(cell).ljust(w) for cell, w in zip(row, widths)))
    print()

# ============================================================================
# 1. Profile Rankings
# ============================================================================
print("\n" + "╔" + "═" * 68 + "╗")
print("║" + " TEXT-BASED VISUALIZATIONS ".center(68) + "║")
print("╚" + "═" * 68 + "╝")

with open('profile_rankings.csv') as f:
    reader = csv.DictReader(f)
    rankings = [(row['profile'], float(row['mean_speedup']))
                for row in list(reader)[:10]]

bar_chart_horizontal(rankings,
                    "📊 TOP 10 PROFILE RANKINGS (Average Speedup)")

# ============================================================================
# 2. Workload Comparison
# ============================================================================
with open('workload_comparison.csv') as f:
    reader = csv.DictReader(f)
    workloads = []
    for row in reader:
        workloads.append((
            row['benchmark'],
            float(row['max_speedup']),
            row['best_profile']
        ))
    workloads.sort(key=lambda x: x[1], reverse=True)

print("🏆 BEST SPEEDUP BY WORKLOAD")
print("=" * 70)
for benchmark, speedup, profile in workloads:
    bar_width = int((speedup / 52) * 40)  # Max is ~51x
    bar = '█' * bar_width
    print(f"{benchmark:15s} {bar:40s} {speedup:5.1f}x  ({profile})")
print()

# ============================================================================
# 3. Pareto Frontier Table
# ============================================================================
with open('pareto_frontier.csv') as f:
    reader = csv.DictReader(f)
    pareto_data = []
    for row in reader:
        speedup = float(row['speedup'])
        size_kb = float(row['binary_size_kb'])
        efficiency = speedup / (size_kb / 1000)  # speedup per MB
        pareto_data.append([
            row['profile'],
            f"{speedup:.2f}x",
            f"{size_kb:.0f} KB",
            f"{efficiency:.1f}"
        ])

    # Sort by speedup
    pareto_data.sort(key=lambda x: float(x[1][:-1]), reverse=True)

table(['Profile', 'Speedup', 'Binary Size', 'Efficiency'],
      pareto_data[:10],
      "⚡ PARETO FRONTIER: Speed vs Size (Top 10 by Speedup)")

# ============================================================================
# 4. Profile Statistics Summary
# ============================================================================
with open('profile_rankings.csv') as f:
    reader = csv.DictReader(f)
    stats_rows = []
    for row in list(reader)[:8]:
        mean = float(row['mean_speedup'])
        ci_low = float(row['ci_lower'])
        ci_high = float(row['ci_upper'])
        ci_width = ci_high - ci_low
        stats_rows.append([
            row['profile'],
            f"{mean:.1f}x",
            f"±{ci_width/2:.1f}",
            f"[{ci_low:.1f}, {ci_high:.1f}]"
        ])

table(['Profile', 'Mean', 'Error', '95% CI'],
      stats_rows,
      "📈 PROFILE STATISTICS (with Confidence Intervals)")

# ============================================================================
# 5. Heatmap Preview (simplified)
# ============================================================================
print("🔥 HEATMAP PREVIEW: Top 5 Benchmarks × Top 5 Profiles")
print("=" * 70)

with open('heatmap_data.csv') as f:
    reader = csv.DictReader(f)
    benchmarks_shown = ['quicksort', 'prime-sieve', 'matrix-mult', 'fibonacci', 'ackermann']
    profiles_shown = ['lto-fat', 'lto-thin', 'perf-ultra', 'opt-s', 'size-ultra']

    # Header
    print(f"{'Benchmark':<15s} " + " ".join(f"{p:>8s}" for p in profiles_shown))
    print("-" * 70)

    for row in reader:
        benchmark = row['benchmark']
        if benchmark in benchmarks_shown:
            values = []
            for profile in profiles_shown:
                val = row.get(profile, '')
                if val:
                    speedup = float(val)
                    # Visual indicator
                    if speedup >= 20:
                        indicator = '🔴'  # High
                    elif speedup >= 10:
                        indicator = '🟡'  # Medium
                    else:
                        indicator = '⚪'  # Low
                    values.append(f"{indicator}{val:>5s}")
                else:
                    values.append("     ---")
            print(f"{benchmark:<15s} " + " ".join(values))

print()

# ============================================================================
# 6. Key Insights Summary
# ============================================================================
print("\n💡 KEY INSIGHTS")
print("=" * 70)

# Best overall
with open('profile_rankings.csv') as f:
    reader = csv.DictReader(f)
    best = next(reader)
    print(f"🥇 Best Overall Profile: {best['profile']}")
    print(f"   Average Speedup: {float(best['mean_speedup']):.1f}x")
    print(f"   95% CI: [{float(best['ci_lower']):.1f}x, {float(best['ci_upper']):.1f}x]")

# Best speedup
with open('workload_comparison.csv') as f:
    reader = csv.DictReader(f)
    rows = list(reader)
    best_speedup = max(rows, key=lambda x: float(x['max_speedup']))
    print(f"\n🚀 Maximum Speedup: {float(best_speedup['max_speedup']):.1f}x")
    print(f"   Benchmark: {best_speedup['benchmark']}")
    print(f"   Profile: {best_speedup['best_profile']}")

# Best size
with open('pareto_frontier.csv') as f:
    reader = csv.DictReader(f)
    rows = list(reader)
    smallest = min(rows, key=lambda x: float(x['binary_size_kb']))
    print(f"\n📦 Smallest Binary: {float(smallest['binary_size_kb']):.0f} KB")
    print(f"   Profile: {smallest['profile']}")
    print(f"   Speedup: {float(smallest['speedup']):.2f}x (only {(1-float(smallest['speedup'])/float(best['mean_speedup']))*100:.0f}% slower)")

    baseline = next(r for r in rows if r['profile'] == 'baseline')
    reduction = (1 - float(smallest['binary_size_kb']) / float(baseline['binary_size_kb'])) * 100
    print(f"   Size Reduction: {reduction:.1f}% vs baseline")

print("\n" + "=" * 70)
print("✅ For graphical visualizations, see VISUALIZATION_GUIDE.md")
print("=" * 70 + "\n")
