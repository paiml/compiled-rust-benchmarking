//! Pathfinder execution demo
//!
//! Demonstrates the full execution infrastructure with a small subset of jobs.
//! Runs 2 benchmarks × 3 profiles = 6 jobs for quick validation.

use harness::build_matrix::BuildJob;
use harness::measurement::{JobResult, Measurement, ResultsCollection};
use std::io::Write;
use std::process::Command;
use std::time::Instant;

fn execute_job_demo(
    job: &BuildJob,
    iterations: usize,
) -> Result<JobResult, Box<dyn std::error::Error>> {
    let mut job_result = JobResult::new(job.clone());

    // Build once
    print!("    Building... ");
    std::io::stdout().flush()?;

    let build_output = Command::new("cargo")
        .args([
            "build",
            "-p",
            &job.benchmark,
            "--profile",
            &job.config_id,
            "--quiet",
        ])
        .output()?;

    if !build_output.status.success() {
        return Err("Build failed".into());
    }
    println!("✓");

    // Run iterations
    for iteration in 1..=iterations {
        print!("    Iteration {}/{}... ", iteration, iterations);
        std::io::stdout().flush()?;

        let output = Command::new("cargo")
            .args([
                "run",
                "-p",
                &job.benchmark,
                "--profile",
                &job.config_id,
                "--quiet",
            ])
            .output()?;

        if !output.status.success() {
            println!("✗ Failed");
            continue;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(measurement) = Measurement::from_output(&stdout) {
            println!("✓ {}μs", measurement.compute_us);
            job_result.add_measurement(measurement);
        } else {
            println!("✗ Parse error");
        }
    }

    Ok(job_result)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║          Pathfinder Execution Demo (6 Jobs)             ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // Demo configuration: 2 benchmarks × 3 profiles = 6 jobs
    let benchmarks = vec!["fibonacci", "prime-sieve"];
    let profiles = vec!["baseline", "standard-release", "lto-fat"];
    let iterations = 3;

    println!("Configuration:");
    println!(
        "  Benchmarks: {} (fibonacci, prime-sieve)",
        benchmarks.len()
    );
    println!(
        "  Profiles: {} (baseline, standard-release, lto-fat)",
        profiles.len()
    );
    println!("  Iterations per job: {}", iterations);
    println!("  Total jobs: {}\n", benchmarks.len() * profiles.len());

    let start_time = Instant::now();
    let mut results = ResultsCollection::new();
    let mut completed = 0;
    let mut failed = 0;

    let total_jobs = benchmarks.len() * profiles.len();
    let mut job_num = 0;

    for benchmark in &benchmarks {
        for profile in &profiles {
            job_num += 1;
            println!("┌─────────────────────────────────────────────────────────┐");
            println!(
                "│ Job {}/{}: {} × {}",
                job_num, total_jobs, benchmark, profile
            );
            println!("└─────────────────────────────────────────────────────────┘");

            let job = BuildJob::new(*benchmark, *profile);

            match execute_job_demo(&job, iterations) {
                Ok(job_result) => {
                    let measurements = job_result.measurement_count();
                    println!("  ✓ Collected {} measurements", measurements);

                    if let Some(stats) = &job_result.stats {
                        println!("  📊 Statistics:");
                        println!("      Mean:   {:.0}μs", stats.mean_compute_us);
                        println!("      Median: {:.0}μs", stats.median_compute_us);
                        println!("      StdDev: {:.1}μs", stats.stddev_compute_us);
                        println!(
                            "      CV:     {:.2}%",
                            stats.coefficient_of_variation() * 100.0
                        );

                        let stable = stats.is_stable(0.10);
                        if stable {
                            println!("      Status: ✓ STABLE (CV < 10%)");
                        } else {
                            println!("      Status: ~ UNSTABLE (CV > 10%)");
                        }
                    }

                    results.add_result(job_result);
                    completed += 1;
                }
                Err(e) => {
                    println!("  ✗ Failed: {}", e);
                    failed += 1;
                }
            }
            println!();
        }
    }

    let total_time = start_time.elapsed().as_secs_f64();

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║                    Demo Summary                          ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    println!("Jobs:");
    println!("  ✓ Completed: {}/{}", completed, total_jobs);
    println!("  ✗ Failed: {}/{}", failed, total_jobs);

    println!("\nMeasurements:");
    println!("  Total collected: {}", results.total_measurements());
    println!(
        "  Average per job: {:.1}",
        results.total_measurements() as f64 / completed as f64
    );

    println!("\nPerformance:");
    println!("  Total time: {:.1}s", total_time);
    println!("  Average job time: {:.1}s", total_time / completed as f64);

    // Calculate speedups for fibonacci
    println!("\n📈 Speedup Analysis (Fibonacci):");

    let fib_baseline = results
        .results()
        .iter()
        .find(|r| r.job.benchmark == "fibonacci" && r.job.config_id == "baseline")
        .and_then(|r| r.stats.as_ref())
        .map(|s| s.mean_compute_us);

    if let Some(baseline_time) = fib_baseline {
        for profile in &profiles {
            if let Some(job_result) = results
                .results()
                .iter()
                .find(|r| r.job.benchmark == "fibonacci" && r.job.config_id == *profile)
            {
                if let Some(stats) = &job_result.stats {
                    let speedup = baseline_time / stats.mean_compute_us;
                    println!("  {:<18} {:.2}x speedup", profile, speedup);
                }
            }
        }
    }

    println!("\n✅ Demo complete! Full execution would run 150 jobs (2-3 hours)");
    println!("   Run: cargo run --bin full-pathfinder-execution");

    Ok(())
}
