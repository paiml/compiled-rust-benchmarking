//! Multi-benchmark pathfinder study
//!
//! Tests key optimization profiles across all benchmark types to identify
//! which optimizations work best for different workload patterns.

use harness::measurement::Measurement;
use std::collections::HashMap;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Multi-Benchmark Pathfinder Study ===\n");

    // Key profiles to test
    let profiles = vec![
        "baseline",
        "standard-release",
        "lto-thin",
        "lto-fat",
        "opt-s",
        "size-ultra",
    ];

    // Select representative benchmarks (one from each category)
    let test_benchmarks = vec![
        ("ackermann", "CPU-bound recursive"),
        ("prime-sieve", "CPU-bound iterative"),
        ("matrix-mult", "Memory-bound cache-sensitive"),
        ("quicksort", "Memory-bound random access"),
        ("hashmap-ops", "Data structure operations"),
    ];

    println!(
        "Testing {} profiles × {} benchmarks = {} combinations\n",
        profiles.len(),
        test_benchmarks.len(),
        profiles.len() * test_benchmarks.len()
    );

    let mut results: HashMap<String, HashMap<String, Measurement>> = HashMap::new();

    for (benchmark, category) in &test_benchmarks {
        println!("=== {} ({}) ===", benchmark, category);

        for profile in &profiles {
            print!("  Testing {}... ", profile);

            // Build if needed
            let build = Command::new("cargo")
                .args(["build", "-p", benchmark, "--profile", profile, "--quiet"])
                .status()?;

            if !build.success() {
                println!("✗ Build failed");
                continue;
            }

            // Run benchmark
            let output = Command::new("cargo")
                .args(["run", "-p", benchmark, "--profile", profile, "--quiet"])
                .output()?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                match Measurement::from_output(&stdout) {
                    Ok(measurement) => {
                        println!("✓ {}μs", measurement.compute_us);

                        results
                            .entry(benchmark.to_string())
                            .or_default()
                            .insert(profile.to_string(), measurement);
                    }
                    Err(e) => {
                        println!("✗ Parse error: {}", e);
                    }
                }
            } else {
                println!("✗ Execution failed");
            }
        }
        println!();
    }

    // Analysis
    println!("\n=== Performance Summary ===\n");

    for (benchmark, category) in &test_benchmarks {
        if let Some(bench_results) = results.get(*benchmark) {
            println!("{} ({}):", benchmark, category);

            if let Some(baseline) = bench_results.get("baseline") {
                let baseline_time = baseline.compute_us as f64;

                // Sort by performance
                let mut sorted: Vec<_> = bench_results.iter().collect();
                sorted.sort_by_key(|(_, m)| m.compute_us);

                for (profile, measurement) in sorted {
                    let speedup = baseline_time / measurement.compute_us as f64;
                    println!(
                        "  {:<18} {:>10}μs  {:>5.2}x",
                        profile, measurement.compute_us, speedup
                    );
                }
            }
            println!();
        }
    }

    // Find best overall profile
    println!("\n=== Best Profile by Benchmark ===\n");
    for (benchmark, category) in &test_benchmarks {
        if let Some(bench_results) = results.get(*benchmark) {
            let mut sorted: Vec<_> = bench_results
                .iter()
                .filter(|(p, _)| *p != "baseline")
                .collect();
            sorted.sort_by_key(|(_, m)| m.compute_us);

            if let Some((best_profile, best_measurement)) = sorted.first() {
                if let Some(baseline) = bench_results.get("baseline") {
                    let speedup = baseline.compute_us as f64 / best_measurement.compute_us as f64;
                    println!(
                        "{:<15} ({:<30}): {} ({:.2}x speedup)",
                        benchmark, category, best_profile, speedup
                    );
                }
            }
        }
    }

    // Profile effectiveness ranking
    println!("\n=== Profile Effectiveness (average speedup across all benchmarks) ===\n");

    let mut profile_avg_speedups: HashMap<String, Vec<f64>> = HashMap::new();

    for (benchmark, _) in &test_benchmarks {
        if let Some(bench_results) = results.get(*benchmark) {
            if let Some(baseline) = bench_results.get("baseline") {
                let baseline_time = baseline.compute_us as f64;

                for (profile, measurement) in bench_results {
                    if profile != "baseline" {
                        let speedup = baseline_time / measurement.compute_us as f64;
                        profile_avg_speedups
                            .entry(profile.clone())
                            .or_default()
                            .push(speedup);
                    }
                }
            }
        }
    }

    let mut profile_averages: Vec<_> = profile_avg_speedups
        .iter()
        .map(|(profile, speedups)| {
            let avg = speedups.iter().sum::<f64>() / speedups.len() as f64;
            (profile.clone(), avg)
        })
        .collect();

    profile_averages.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for (i, (profile, avg_speedup)) in profile_averages.iter().enumerate() {
        let marker = if i == 0 { "⭐" } else { "  " };
        println!(
            "{}  {:<18} {:.2}x average speedup",
            marker, profile, avg_speedup
        );
    }

    println!("\n✅ Multi-benchmark pathfinder study complete");
    println!(
        "Tested {} benchmarks across {} profiles",
        test_benchmarks.len(),
        profiles.len()
    );

    Ok(())
}
