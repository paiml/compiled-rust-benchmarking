#!/usr/bin/env python3
"""Quick analysis of pathfinder results"""

import json
import sys

with open('pathfinder_results.json', 'r') as f:
    data = json.load(f)

results = data['results']

# Group by benchmark
by_benchmark = {}
for result in results:
    if result['stats'] is None:
        continue

    benchmark = result['job']['benchmark']
    config = result['job']['config_id']
    mean_us = result['stats']['mean_compute_us']

    if benchmark not in by_benchmark:
        by_benchmark[benchmark] = {}
    by_benchmark[benchmark][config] = mean_us

print("=== Maximum Speedups by Benchmark ===\n")
for benchmark in sorted(by_benchmark.keys()):
    configs = by_benchmark[benchmark]
    if 'baseline' not in configs:
        continue

    baseline = configs['baseline']
    fastest_config = min(configs.items(), key=lambda x: x[1])
    fastest_name, fastest_time = fastest_config
    speedup = baseline / fastest_time

    print(f"  {benchmark:20s}  {speedup:5.2f}x  ({fastest_name})")

print("\n=== Profile Rankings (avg speedup across benchmarks) ===\n")

# Calculate average speedup for each profile
profile_speedups = {}
for benchmark in by_benchmark.keys():
    configs = by_benchmark[benchmark]
    if 'baseline' not in configs:
        continue

    baseline = configs['baseline']
    for config, time in configs.items():
        if config == 'baseline':
            continue
        speedup = baseline / time
        if config not in profile_speedups:
            profile_speedups[config] = []
        profile_speedups[config].append(speedup)

# Average and sort
avg_speedups = [(profile, sum(speedups)/len(speedups))
                for profile, speedups in profile_speedups.items()]
avg_speedups.sort(key=lambda x: x[1], reverse=True)

for i, (profile, avg_speedup) in enumerate(avg_speedups[:10], 1):
    medal = {1: '🥇', 2: '🥈', 3: '🥉'}.get(i, '  ')
    print(f"{medal} {i:2d}. {profile:20s}  {avg_speedup:5.2f}x")

print(f"\n=== Statistical Quality ===\n")
stable_count = sum(1 for r in results if r['stats'] and
                   (r['stats']['stddev_compute_us'] / r['stats']['mean_compute_us']) < 0.10)
total_with_stats = sum(1 for r in results if r['stats'])

print(f"  Stable jobs (CV < 10%): {stable_count}/{total_with_stats} ({100*stable_count/total_with_stats:.1f}%)")
print(f"  Total measurements: {sum(len(r['measurements']) for r in results)}")
print(f"  Avg measurements per job: {sum(len(r['measurements']) for r in results) / len(results):.1f}")
