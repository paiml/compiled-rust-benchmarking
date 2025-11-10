//! Statistical analysis of pathfinder results
//!
//! Performs comprehensive statistical analysis on the collected benchmark data,
//! including ANOVA, profile comparisons, and workload-specific insights.

use analysis::{anova_one_way, cohens_d, mean, std_dev, t_test_welch};
use harness::measurement::ResultsCollection;
use std::collections::HashMap;
use std::fs;

/// Workload classification for benchmarks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum WorkloadType {
    CpuRecursive,
    CpuIterative,
    MemoryCacheSensitive,
    MemoryRandomAccess,
    DataStructures,
    StringProcessing,
    Serialization,
    IoBound,
}

impl WorkloadType {
    fn from_benchmark(benchmark: &str) -> Self {
        match benchmark {
            "ackermann" | "fibonacci" => WorkloadType::CpuRecursive,
            "prime-sieve" => WorkloadType::CpuIterative,
            "matrix-mult" => WorkloadType::MemoryCacheSensitive,
            "quicksort" => WorkloadType::MemoryRandomAccess,
            "hashmap-ops" | "btreemap-ops" => WorkloadType::DataStructures,
            "string-parse" => WorkloadType::StringProcessing,
            "json-parse" => WorkloadType::Serialization,
            "file-io" => WorkloadType::IoBound,
            _ => WorkloadType::CpuIterative, // default
        }
    }

    fn name(&self) -> &'static str {
        match self {
            WorkloadType::CpuRecursive => "CPU-bound recursive",
            WorkloadType::CpuIterative => "CPU-bound iterative",
            WorkloadType::MemoryCacheSensitive => "Memory-bound cache-sensitive",
            WorkloadType::MemoryRandomAccess => "Memory-bound random access",
            WorkloadType::DataStructures => "Data structure operations",
            WorkloadType::StringProcessing => "String processing",
            WorkloadType::Serialization => "Serialization",
            WorkloadType::IoBound => "I/O-bound",
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║          Statistical Analysis of Pathfinder Results     ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // Load results
    println!("Loading pathfinder_results.json...");
    let json_data = fs::read_to_string("pathfinder_results.json")?;
    let results: ResultsCollection = serde_json::from_str(&json_data)?;

    println!("  ✓ Loaded {} job results\n", results.results().len());

    // Calculate speedups for each benchmark×profile
    let mut speedups: HashMap<String, HashMap<String, f64>> = HashMap::new();

    for result in results.results() {
        if let Some(stats) = &result.stats {
            let benchmark = &result.job.benchmark;
            let profile = &result.job.config_id;

            speedups
                .entry(benchmark.clone())
                .or_default()
                .insert(profile.clone(), stats.mean_compute_us);
        }
    }

    // Calculate speedup ratios (baseline = 1.0)
    let mut speedup_ratios: HashMap<String, HashMap<String, f64>> = HashMap::new();

    for (benchmark, profiles) in &speedups {
        if let Some(&baseline_time) = profiles.get("baseline") {
            for (profile, &time) in profiles {
                if profile != "baseline" {
                    let speedup = baseline_time / time;
                    speedup_ratios
                        .entry(benchmark.clone())
                        .or_default()
                        .insert(profile.clone(), speedup);
                }
            }
        }
    }

    // Analysis 1: Profile Rankings
    println!("═══════════════════════════════════════════════════════════");
    println!("1. PROFILE RANKINGS (Average Speedup)");
    println!("═══════════════════════════════════════════════════════════\n");

    let mut profile_speedups: HashMap<String, Vec<f64>> = HashMap::new();

    for ratios in speedup_ratios.values() {
        for (profile, &speedup) in ratios {
            profile_speedups
                .entry(profile.clone())
                .or_default()
                .push(speedup);
        }
    }

    let mut profile_avgs: Vec<(String, f64, f64)> = profile_speedups
        .iter()
        .filter_map(|(profile, speedups)| {
            let avg = mean(speedups)?;
            let sd = std_dev(speedups).unwrap_or(0.0);
            Some((profile.clone(), avg, sd))
        })
        .collect();

    profile_avgs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for (i, (profile, avg, sd)) in profile_avgs.iter().enumerate().take(15) {
        let medal = match i {
            0 => "🥇",
            1 => "🥈",
            2 => "🥉",
            _ => "  ",
        };
        println!(
            "{} {:2}. {:<20}  {:>5.2}x ± {:>4.2}x",
            medal,
            i + 1,
            profile,
            avg,
            sd
        );
    }

    // Analysis 2: ANOVA by Workload Type
    println!("\n═══════════════════════════════════════════════════════════");
    println!("2. ANOVA: Workload Type Comparison");
    println!("═══════════════════════════════════════════════════════════\n");

    // Group speedups by workload type (using best profile for each benchmark)
    let mut workload_speedups: HashMap<WorkloadType, Vec<f64>> = HashMap::new();

    for (benchmark, ratios) in &speedup_ratios {
        let workload = WorkloadType::from_benchmark(benchmark);
        if let Some(&max_speedup) = ratios.values().max_by(|a, b| a.partial_cmp(b).unwrap()) {
            workload_speedups
                .entry(workload)
                .or_default()
                .push(max_speedup);
        }
    }

    let mut workload_groups: Vec<(WorkloadType, Vec<f64>)> =
        workload_speedups.into_iter().collect();
    workload_groups.sort_by_key(|(wl, _)| format!("{:?}", wl));

    // Display workload type statistics
    for (workload, speedups) in &workload_groups {
        if let (Some(avg), Some(sd)) = (mean(speedups), std_dev(speedups)) {
            println!(
                "{:<30}  {:>5.2}x ± {:>4.2}x  (n={})",
                workload.name(),
                avg,
                sd,
                speedups.len()
            );
        }
    }

    // Perform ANOVA
    let groups_vec: Vec<Vec<f64>> = workload_groups.iter().map(|(_, v)| v.clone()).collect();

    if let Some(anova_result) = anova_one_way(&groups_vec) {
        println!("\nANOVA Results:");
        println!("  F-statistic: {:.2}", anova_result.f_statistic);
        println!("  df between: {}", anova_result.df_between);
        println!("  df within: {}", anova_result.df_within);
        println!("  η² (effect size): {:.3}", anova_result.eta_squared);
        println!(
            "  Significant: {}",
            if anova_result.significant {
                "✓ YES (F > 3.0)"
            } else {
                "✗ NO"
            }
        );
    }

    // Analysis 3: Pairwise Profile Comparisons
    println!("\n═══════════════════════════════════════════════════════════");
    println!("3. PAIRWISE PROFILE COMPARISONS (t-tests)");
    println!("═══════════════════════════════════════════════════════════\n");

    // Compare top 5 profiles
    let top_profiles: Vec<&String> = profile_avgs.iter().take(5).map(|(p, _, _)| p).collect();

    println!("Comparing top 5 profiles across all benchmarks:\n");

    for i in 0..top_profiles.len() {
        for j in (i + 1)..top_profiles.len() {
            let profile1 = top_profiles[i];
            let profile2 = top_profiles[j];

            // Collect speedups for both profiles across common benchmarks
            let mut speedups1 = Vec::new();
            let mut speedups2 = Vec::new();

            for ratios in speedup_ratios.values() {
                if let (Some(&s1), Some(&s2)) = (ratios.get(profile1), ratios.get(profile2)) {
                    speedups1.push(s1);
                    speedups2.push(s2);
                }
            }

            if let Some(t_result) = t_test_welch(&speedups1, &speedups2) {
                let effect = cohens_d(&speedups1, &speedups2).unwrap_or(0.0);

                println!(
                    "{:<20} vs {:<20}",
                    format!("{}:", profile1),
                    profile2
                );
                println!("  Mean diff: {:>6.2}x", t_result.mean_diff);
                println!("  t-statistic: {:>6.2}", t_result.t_statistic);
                println!("  Cohen's d: {:>6.2}", effect);
                println!(
                    "  Significant: {}",
                    if t_result.significant { "✓ YES" } else { "✗ NO" }
                );
                println!();
            }
        }
    }

    // Analysis 4: Best Profile by Workload
    println!("═══════════════════════════════════════════════════════════");
    println!("4. BEST PROFILE BY WORKLOAD TYPE");
    println!("═══════════════════════════════════════════════════════════\n");

    // For each workload, find the best average profile
    let mut workload_profile_speedups: HashMap<WorkloadType, HashMap<String, Vec<f64>>> =
        HashMap::new();

    for (benchmark, ratios) in &speedup_ratios {
        let workload = WorkloadType::from_benchmark(benchmark);
        for (profile, &speedup) in ratios {
            workload_profile_speedups
                .entry(workload)
                .or_default()
                .entry(profile.clone())
                .or_default()
                .push(speedup);
        }
    }

    for workload in [
        WorkloadType::CpuRecursive,
        WorkloadType::CpuIterative,
        WorkloadType::MemoryCacheSensitive,
        WorkloadType::MemoryRandomAccess,
        WorkloadType::DataStructures,
        WorkloadType::StringProcessing,
        WorkloadType::Serialization,
    ] {
        if let Some(profiles) = workload_profile_speedups.get(&workload) {
            let mut profile_avgs: Vec<(String, f64)> = profiles
                .iter()
                .filter_map(|(profile, speedups)| {
                    let avg = mean(speedups)?;
                    Some((profile.clone(), avg))
                })
                .collect();

            profile_avgs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            println!("{}:", workload.name());
            for (i, (profile, avg)) in profile_avgs.iter().take(3).enumerate() {
                println!("  {}. {:<20}  {:>5.2}x", i + 1, profile, avg);
            }
            println!();
        }
    }

    println!("═══════════════════════════════════════════════════════════");
    println!("✅ Analysis complete!");
    println!("═══════════════════════════════════════════════════════════\n");

    Ok(())
}
