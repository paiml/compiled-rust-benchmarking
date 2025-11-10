#!/usr/bin/env python3
"""Export visualization data as CSV for plotting"""

import json
import csv
from collections import defaultdict

def export_pareto_frontier():
    """Export speed vs size data for Pareto frontier"""
    # Load results and binary sizes
    with open('pathfinder_results.json') as f:
        results = json.load(f)

    with open('binary_sizes_fibonacci.json') as f:
        sizes = json.load(f)

    # Filter for fibonacci benchmark only (where we have size data)
    with open('pareto_frontier.csv', 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(['profile', 'speedup', 'binary_size_kb'])

        for result in results['results']:
            if result['job']['benchmark'] != 'fibonacci':
                continue

            profile = result['job']['config_id']
            if not result['stats']:
                continue

            # Get baseline time for fibonacci
            baseline_time = next(
                (r['stats']['mean_compute_us']
                 for r in results['results']
                 if r['job']['benchmark'] == 'fibonacci'
                 and r['job']['config_id'] == 'baseline'
                 and r['stats']),
                None
            )

            if not baseline_time:
                continue

            speedup = baseline_time / result['stats']['mean_compute_us']

            if profile in sizes['sizes']:
                size_kb = sizes['sizes'][profile]['kb']
                writer.writerow([profile, f"{speedup:.2f}", f"{size_kb:.2f}"])

    print("✓ pareto_frontier.csv")


def export_heatmap():
    """Export benchmark × profile heatmap data"""
    with open('pathfinder_results.json') as f:
        results = json.load(f)

    # Build matrix
    benchmarks = set()
    profiles = set()
    speedups = defaultdict(dict)

    for result in results['results']:
        benchmark = result['job']['benchmark']
        profile = result['job']['config_id']

        if not result['stats']:
            continue

        benchmarks.add(benchmark)
        profiles.add(profile)

        # Get baseline for this benchmark
        baseline_time = next(
            (r['stats']['mean_compute_us']
             for r in results['results']
             if r['job']['benchmark'] == benchmark
             and r['job']['config_id'] == 'baseline'
             and r['stats']),
            None
        )

        if baseline_time and profile != 'baseline':
            speedup = baseline_time / result['stats']['mean_compute_us']
            speedups[benchmark][profile] = speedup

    # Write CSV
    benchmarks = sorted(benchmarks)
    profiles = sorted([p for p in profiles if p != 'baseline'])

    with open('heatmap_data.csv', 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(['benchmark'] + profiles)

        for benchmark in benchmarks:
            row = [benchmark]
            for profile in profiles:
                speedup = speedups[benchmark].get(profile, '')
                row.append(f"{speedup:.2f}" if speedup else '')
            writer.writerow(row)

    print("✓ heatmap_data.csv")


def export_profile_rankings():
    """Export profile rankings with confidence intervals"""
    with open('pathfinder_results.json') as f:
        results = json.load(f)

    # Calculate average speedup per profile
    profile_speedups = defaultdict(list)

    for result in results['results']:
        benchmark = result['job']['benchmark']
        profile = result['job']['config_id']

        if not result['stats'] or profile == 'baseline':
            continue

        # Get baseline for this benchmark
        baseline_time = next(
            (r['stats']['mean_compute_us']
             for r in results['results']
             if r['job']['benchmark'] == benchmark
             and r['job']['config_id'] == 'baseline'
             and r['stats']),
            None
        )

        if baseline_time:
            speedup = baseline_time / result['stats']['mean_compute_us']
            profile_speedups[profile].append(speedup)

    # Calculate stats
    rankings = []
    for profile, speedups_list in profile_speedups.items():
        mean_speedup = sum(speedups_list) / len(speedups_list)
        # Simple CI: mean ± 1.96 * (stddev / sqrt(n))
        stddev = (sum((x - mean_speedup)**2 for x in speedups_list) / (len(speedups_list) - 1))**0.5
        se = stddev / (len(speedups_list)**0.5)
        ci_lower = mean_speedup - 1.96 * se
        ci_upper = mean_speedup + 1.96 * se

        rankings.append((profile, mean_speedup, ci_lower, ci_upper, stddev))

    rankings.sort(key=lambda x: x[1], reverse=True)

    with open('profile_rankings.csv', 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(['profile', 'mean_speedup', 'ci_lower', 'ci_upper', 'stddev'])

        for profile, mean, ci_low, ci_high, sd in rankings:
            writer.writerow([
                profile,
                f"{mean:.2f}",
                f"{ci_low:.2f}",
                f"{ci_high:.2f}",
                f"{sd:.2f}"
            ])

    print("✓ profile_rankings.csv")


def export_workload_comparison():
    """Export workload type comparison data"""
    with open('pathfinder_results.json') as f:
        results = json.load(f)

    workload_map = {
        'ackermann': 'CPU-recursive',
        'fibonacci': 'CPU-recursive',
        'prime-sieve': 'CPU-iterative',
        'matrix-mult': 'Memory-cache',
        'quicksort': 'Memory-random',
        'hashmap-ops': 'Data-structures',
        'btreemap-ops': 'Data-structures',
        'string-parse': 'String-processing',
        'json-parse': 'Serialization',
        'file-io': 'IO-bound'
    }

    # Get max speedup for each benchmark
    max_speedups = {}

    for result in results['results']:
        benchmark = result['job']['benchmark']
        profile = result['job']['config_id']

        if not result['stats'] or profile == 'baseline':
            continue

        baseline_time = next(
            (r['stats']['mean_compute_us']
             for r in results['results']
             if r['job']['benchmark'] == benchmark
             and r['job']['config_id'] == 'baseline'
             and r['stats']),
            None
        )

        if baseline_time:
            speedup = baseline_time / result['stats']['mean_compute_us']
            if benchmark not in max_speedups or speedup > max_speedups[benchmark][0]:
                max_speedups[benchmark] = (speedup, profile)

    with open('workload_comparison.csv', 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(['benchmark', 'workload_type', 'max_speedup', 'best_profile'])

        for benchmark, (speedup, profile) in sorted(max_speedups.items()):
            workload = workload_map.get(benchmark, 'Unknown')
            writer.writerow([benchmark, workload, f"{speedup:.2f}", profile])

    print("✓ workload_comparison.csv")


def main():
    print("Exporting visualization data to CSV...\n")

    export_pareto_frontier()
    export_heatmap()
    export_profile_rankings()
    export_workload_comparison()

    print("\n✅ All CSV files exported")
    print("\nFiles created:")
    print("  - pareto_frontier.csv       (speed vs size tradeoff)")
    print("  - heatmap_data.csv          (benchmark × profile matrix)")
    print("  - profile_rankings.csv      (profile performance with CIs)")
    print("  - workload_comparison.csv   (workload types)")


if __name__ == '__main__':
    main()
