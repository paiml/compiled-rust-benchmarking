#!/usr/bin/env python3
"""Generate publication-ready visualizations from benchmark data"""

import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import numpy as np
from pathlib import Path

# Set publication-quality defaults
plt.rcParams['figure.dpi'] = 300
plt.rcParams['savefig.dpi'] = 300
plt.rcParams['font.size'] = 10
plt.rcParams['font.family'] = 'sans-serif'
plt.rcParams['axes.labelsize'] = 11
plt.rcParams['axes.titlesize'] = 12
plt.rcParams['xtick.labelsize'] = 9
plt.rcParams['ytick.labelsize'] = 9
plt.rcParams['legend.fontsize'] = 9

# Create output directory
Path('visualizations').mkdir(exist_ok=True)

print("Generating visualizations...\n")

# ============================================================================
# 1. PARETO FRONTIER: Speed vs Size
# ============================================================================
print("1. Creating Pareto frontier (speed vs size)...")

df_pareto = pd.read_csv('pareto_frontier.csv')

fig, ax = plt.subplots(figsize=(8, 6))

# Plot all points
scatter = ax.scatter(df_pareto['binary_size_kb'],
                     df_pareto['speedup'],
                     s=100,
                     alpha=0.6,
                     c=df_pareto['speedup'],
                     cmap='viridis',
                     edgecolors='black',
                     linewidth=0.5)

# Annotate key points
key_profiles = ['lto-fat', 'size-ultra', 'baseline', 'perf-ultra']
for _, row in df_pareto.iterrows():
    if row['profile'] in key_profiles:
        ax.annotate(row['profile'],
                   (row['binary_size_kb'], row['speedup']),
                   xytext=(5, 5),
                   textcoords='offset points',
                   fontsize=8,
                   bbox=dict(boxstyle='round,pad=0.3',
                            facecolor='yellow',
                            alpha=0.5))

# Identify Pareto frontier
df_sorted = df_pareto.sort_values('binary_size_kb')
pareto_x = []
pareto_y = []
max_speedup = 0
for _, row in df_sorted.iterrows():
    if row['speedup'] > max_speedup:
        pareto_x.append(row['binary_size_kb'])
        pareto_y.append(row['speedup'])
        max_speedup = row['speedup']

# Draw Pareto frontier line
ax.plot(pareto_x, pareto_y, 'r--', alpha=0.5, linewidth=2, label='Pareto frontier')

ax.set_xlabel('Binary Size (KB)')
ax.set_ylabel('Speedup vs Baseline')
ax.set_title('Pareto Frontier: Speed vs Size Tradeoff\n(Fibonacci Benchmark)')
ax.grid(True, alpha=0.3)
ax.legend()

plt.colorbar(scatter, ax=ax, label='Speedup')
plt.tight_layout()
plt.savefig('visualizations/pareto_frontier.png', bbox_inches='tight')
plt.close()

print("  ✓ visualizations/pareto_frontier.png")

# ============================================================================
# 2. HEATMAP: Benchmark × Profile Performance
# ============================================================================
print("2. Creating heatmap (benchmark × profile performance)...")

df_heatmap = pd.read_csv('heatmap_data.csv', index_col=0)

# Convert to numeric, replace empty strings with NaN
df_heatmap = df_heatmap.apply(pd.to_numeric, errors='coerce')

fig, ax = plt.subplots(figsize=(12, 6))

# Create heatmap
im = ax.imshow(df_heatmap.values, cmap='YlOrRd', aspect='auto')

# Set ticks
ax.set_xticks(np.arange(len(df_heatmap.columns)))
ax.set_yticks(np.arange(len(df_heatmap.index)))

# Label ticks
ax.set_xticklabels(df_heatmap.columns, rotation=45, ha='right')
ax.set_yticklabels(df_heatmap.index)

# Add colorbar
cbar = plt.colorbar(im, ax=ax)
cbar.set_label('Speedup vs Baseline', rotation=270, labelpad=20)

# Add text annotations
for i in range(len(df_heatmap.index)):
    for j in range(len(df_heatmap.columns)):
        value = df_heatmap.values[i, j]
        if not np.isnan(value):
            text = ax.text(j, i, f'{value:.1f}x',
                          ha="center", va="center",
                          color="black" if value < 15 else "white",
                          fontsize=7)

ax.set_title('Heatmap: Speedup by Benchmark and Profile')
plt.tight_layout()
plt.savefig('visualizations/heatmap.png', bbox_inches='tight')
plt.close()

print("  ✓ visualizations/heatmap.png")

# ============================================================================
# 3. PROFILE RANKINGS: Bar chart with error bars
# ============================================================================
print("3. Creating profile rankings with confidence intervals...")

df_rankings = pd.read_csv('profile_rankings.csv')

# Take top 10
df_top = df_rankings.head(10)

fig, ax = plt.subplots(figsize=(10, 6))

# Calculate error bar sizes
errors_lower = df_top['mean_speedup'] - df_top['ci_lower']
errors_upper = df_top['ci_upper'] - df_top['mean_speedup']
errors = [errors_lower, errors_upper]

# Create bar chart
bars = ax.barh(range(len(df_top)),
               df_top['mean_speedup'],
               xerr=errors,
               capsize=5,
               color='steelblue',
               edgecolor='black',
               alpha=0.7)

# Color top 3 differently
colors = ['gold', 'silver', '#CD7F32'] + ['steelblue'] * 7
for bar, color in zip(bars, colors):
    bar.set_color(color)
    bar.set_edgecolor('black')

ax.set_yticks(range(len(df_top)))
ax.set_yticklabels(df_top['profile'])
ax.invert_yaxis()

ax.set_xlabel('Average Speedup (with 95% CI)')
ax.set_title('Top 10 Profile Rankings\n(Average across all benchmarks)')
ax.grid(True, axis='x', alpha=0.3)

# Add value labels
for i, (mean, upper) in enumerate(zip(df_top['mean_speedup'], df_top['ci_upper'])):
    ax.text(upper + 1, i, f'{mean:.1f}x',
           va='center', fontsize=9, fontweight='bold')

plt.tight_layout()
plt.savefig('visualizations/profile_rankings.png', bbox_inches='tight')
plt.close()

print("  ✓ visualizations/profile_rankings.png")

# ============================================================================
# 4. WORKLOAD COMPARISON: Grouped bar chart
# ============================================================================
print("4. Creating workload comparison...")

df_workload = pd.read_csv('workload_comparison.csv')

# Group by workload type
workload_order = [
    'Memory-random',
    'CPU-iterative',
    'Memory-cache',
    'Data-structures',
    'Serialization',
    'String-processing',
    'CPU-recursive',
    'IO-bound'
]

# Map and sort
df_workload['workload_order'] = df_workload['workload_type'].map(
    {wl: i for i, wl in enumerate(workload_order)}
)
df_workload = df_workload.sort_values('workload_order')

fig, ax = plt.subplots(figsize=(10, 6))

# Create bars
bars = ax.barh(range(len(df_workload)),
               df_workload['max_speedup'],
               color='coral',
               edgecolor='black',
               alpha=0.7)

# Color by magnitude
colors_map = plt.cm.RdYlGn(df_workload['max_speedup'] / df_workload['max_speedup'].max())
for bar, color in zip(bars, colors_map):
    bar.set_color(color)

ax.set_yticks(range(len(df_workload)))
ax.set_yticklabels([f"{row['benchmark']}\n({row['workload_type']})"
                    for _, row in df_workload.iterrows()])
ax.invert_yaxis()

ax.set_xlabel('Maximum Speedup Achieved')
ax.set_title('Best Speedup by Workload Type\n(with optimal profile)')
ax.grid(True, axis='x', alpha=0.3)

# Add value labels with best profile
for i, row in enumerate(df_workload.itertuples()):
    ax.text(row.max_speedup + 1, i,
           f'{row.max_speedup:.1f}x ({row.best_profile})',
           va='center', fontsize=8)

plt.tight_layout()
plt.savefig('visualizations/workload_comparison.png', bbox_inches='tight')
plt.close()

print("  ✓ visualizations/workload_comparison.png")

# ============================================================================
# 5. COMBINED FIGURE: 2x2 grid
# ============================================================================
print("5. Creating combined visualization (all 4 plots)...")

fig = plt.figure(figsize=(16, 12))
gs = fig.add_gridspec(2, 2, hspace=0.3, wspace=0.3)

# --- Subplot 1: Pareto Frontier ---
ax1 = fig.add_subplot(gs[0, 0])
df_pareto = pd.read_csv('pareto_frontier.csv')
scatter = ax1.scatter(df_pareto['binary_size_kb'],
                     df_pareto['speedup'],
                     s=80,
                     alpha=0.6,
                     c=df_pareto['speedup'],
                     cmap='viridis',
                     edgecolors='black',
                     linewidth=0.5)
for profile in ['lto-fat', 'size-ultra', 'baseline']:
    row = df_pareto[df_pareto['profile'] == profile].iloc[0]
    ax1.annotate(profile,
               (row['binary_size_kb'], row['speedup']),
               xytext=(3, 3),
               textcoords='offset points',
               fontsize=7)
ax1.set_xlabel('Binary Size (KB)', fontsize=10)
ax1.set_ylabel('Speedup', fontsize=10)
ax1.set_title('(A) Pareto Frontier: Speed vs Size', fontsize=11, fontweight='bold')
ax1.grid(True, alpha=0.3)

# --- Subplot 2: Profile Rankings ---
ax2 = fig.add_subplot(gs[0, 1])
df_rankings = pd.read_csv('profile_rankings.csv')
df_top = df_rankings.head(8)
errors_lower = df_top['mean_speedup'] - df_top['ci_lower']
errors_upper = df_top['ci_upper'] - df_top['mean_speedup']
bars = ax2.barh(range(len(df_top)),
               df_top['mean_speedup'],
               xerr=[errors_lower, errors_upper],
               capsize=3,
               color='steelblue',
               alpha=0.7)
colors = ['gold', 'silver', '#CD7F32'] + ['steelblue'] * 5
for bar, color in zip(bars, colors):
    bar.set_color(color)
ax2.set_yticks(range(len(df_top)))
ax2.set_yticklabels(df_top['profile'], fontsize=9)
ax2.invert_yaxis()
ax2.set_xlabel('Average Speedup (95% CI)', fontsize=10)
ax2.set_title('(B) Top Profile Rankings', fontsize=11, fontweight='bold')
ax2.grid(True, axis='x', alpha=0.3)

# --- Subplot 3: Workload Comparison ---
ax3 = fig.add_subplot(gs[1, 0])
df_workload = pd.read_csv('workload_comparison.csv')
df_workload = df_workload.sort_values('max_speedup', ascending=True)
bars = ax3.barh(range(len(df_workload)),
               df_workload['max_speedup'],
               color='coral',
               alpha=0.7)
colors_map = plt.cm.RdYlGn(df_workload['max_speedup'] / df_workload['max_speedup'].max())
for bar, color in zip(bars, colors_map):
    bar.set_color(color)
ax3.set_yticks(range(len(df_workload)))
ax3.set_yticklabels(df_workload['benchmark'], fontsize=9)
ax3.set_xlabel('Maximum Speedup', fontsize=10)
ax3.set_title('(C) Best Speedup by Benchmark', fontsize=11, fontweight='bold')
ax3.grid(True, axis='x', alpha=0.3)

# --- Subplot 4: Heatmap (simplified) ---
ax4 = fig.add_subplot(gs[1, 1])
df_heatmap = pd.read_csv('heatmap_data.csv', index_col=0)
df_heatmap = df_heatmap.apply(pd.to_numeric, errors='coerce')
# Show subset
cols_to_show = ['lto-fat', 'lto-thin', 'standard-release', 'perf-ultra', 'size-ultra']
df_subset = df_heatmap[cols_to_show]
im = ax4.imshow(df_subset.values, cmap='YlOrRd', aspect='auto')
ax4.set_xticks(np.arange(len(df_subset.columns)))
ax4.set_yticks(np.arange(len(df_subset.index)))
ax4.set_xticklabels(df_subset.columns, rotation=45, ha='right', fontsize=8)
ax4.set_yticklabels(df_subset.index, fontsize=9)
ax4.set_title('(D) Heatmap: Speedup Matrix (subset)', fontsize=11, fontweight='bold')
for i in range(len(df_subset.index)):
    for j in range(len(df_subset.columns)):
        value = df_subset.values[i, j]
        if not np.isnan(value):
            ax4.text(j, i, f'{value:.0f}',
                    ha="center", va="center",
                    color="black" if value < 15 else "white",
                    fontsize=7)

plt.suptitle('Rust Optimization Benchmark Results: Comprehensive Analysis',
            fontsize=14, fontweight='bold', y=0.995)

plt.savefig('visualizations/combined_analysis.png', bbox_inches='tight')
plt.close()

print("  ✓ visualizations/combined_analysis.png")

# ============================================================================
# Summary
# ============================================================================
print("\n" + "="*60)
print("✅ All visualizations generated successfully!")
print("="*60)
print("\nFiles created in visualizations/:")
print("  1. pareto_frontier.png       - Speed vs size tradeoff")
print("  2. heatmap.png               - Benchmark × profile matrix")
print("  3. profile_rankings.png      - Top profiles with CIs")
print("  4. workload_comparison.png   - Best speedup by workload")
print("  5. combined_analysis.png     - All 4 plots in one figure")
print("\nAll figures are publication-ready (300 DPI)")
