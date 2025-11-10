//! Full pathfinder study execution with multiple iterations
//!
//! Executes all 150 pathfinder jobs (10 benchmarks × 15 profiles)
//! with multiple iterations per job to collect statistical data.

use harness::build_matrix::{BuildJob, BuildMatrix, BENCHMARKS};
use harness::config::generator::ConfigGenerator;
use harness::measurement::{JobResult, Measurement, ResultsCollection};
use harness::pathfinder::PathfinderSelector;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::time::Instant;

struct ExecutionConfig {
    iterations: usize,
    max_cv: f64, // Maximum coefficient of variation (10% = 0.1)
    min_iterations: usize,
    max_iterations: usize,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            iterations: 5,
            max_cv: 0.10, // 10% CV threshold
            min_iterations: 3,
            max_iterations: 10,
        }
    }
}

fn execute_job(
    job: &BuildJob,
    config: &ExecutionConfig,
) -> Result<JobResult, Box<dyn std::error::Error>> {
    let mut job_result = JobResult::new(job.clone());

    // Build once
    let build_output = Command::new("cargo")
        .args(["build", "-p", &job.benchmark, "--profile", &job.config_id])
        .output()?;

    if !build_output.status.success() {
        return Err(format!("Build failed for {}", job.job_id).into());
    }

    // Run iterations
    for iteration in 1..=config.max_iterations {
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
            continue; // Skip failed runs
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if let Ok(measurement) = Measurement::from_output(&stdout) {
            job_result.add_measurement(measurement);

            // Check if we have enough stable measurements
            if iteration >= config.min_iterations {
                if let Some(stats) = &job_result.stats {
                    if stats.is_stable(config.max_cv) {
                        break; // Stable enough, stop early
                    }
                }
            }
        }
    }

    Ok(job_result)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║     Full Pathfinder Study Execution with Iterations     ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let exec_config = ExecutionConfig::default();

    println!("Configuration:");
    println!("  Target iterations: {}", exec_config.iterations);
    println!("  CV threshold: {:.1}%", exec_config.max_cv * 100.0);
    println!("  Min iterations: {}", exec_config.min_iterations);
    println!("  Max iterations: {}", exec_config.max_iterations);
    println!();

    // Generate pathfinder configurations
    println!("Step 1: Generating pathfinder configurations...");
    let mut generator = ConfigGenerator::new();
    let all_configs = generator.generate_matrix();
    let selector = PathfinderSelector::balanced(15);
    let pathfinder_configs = selector.select(all_configs);

    println!(
        "  Selected {} pathfinder configurations",
        pathfinder_configs.len()
    );
    println!();

    // Create build matrix
    println!("Step 2: Creating build matrix...");
    let mut matrix = BuildMatrix::new();
    for benchmark in BENCHMARKS {
        for config in &pathfinder_configs {
            matrix.add_job(BuildJob::new(*benchmark, &config.id));
        }
    }

    let total_jobs = matrix.len();
    let estimated_measurements = total_jobs * exec_config.iterations;

    println!(
        "  Total jobs: {} ({} benchmarks × {} profiles)",
        total_jobs,
        BENCHMARKS.len(),
        pathfinder_configs.len()
    );
    println!("  Estimated measurements: ~{}", estimated_measurements);
    println!();

    // Execute all jobs
    println!("Step 3: Executing pathfinder study...\n");
    println!("┌────────────────────────────────────────────────────────┐");

    let start_time = Instant::now();
    let mut results = ResultsCollection::new();
    let mut completed = 0;
    let mut failed = 0;

    for (idx, job) in matrix.jobs().iter().enumerate() {
        let job_num = idx + 1;
        print!(
            "│ [{:3}/{:3}] {:<20} {:<18} ",
            job_num, total_jobs, job.benchmark, job.config_id
        );
        std::io::stdout().flush()?;

        match execute_job(job, &exec_config) {
            Ok(job_result) => {
                let measurements = job_result.measurement_count();
                if measurements > 0 {
                    if let Some(stats) = &job_result.stats {
                        let cv_percent = stats.coefficient_of_variation() * 100.0;
                        let stable = stats.is_stable(exec_config.max_cv);
                        let marker = if stable { "✓" } else { "~" };

                        print!(
                            "{} {:2} iter, {:>8.0}μs, CV:{:>4.1}%",
                            marker, measurements, stats.mean_compute_us, cv_percent
                        );
                    }
                    results.add_result(job_result);
                    completed += 1;
                } else {
                    print!("✗ No measurements");
                    failed += 1;
                }
            }
            Err(e) => {
                print!("✗ {}", e);
                failed += 1;
            }
        }

        // Progress and ETA
        let elapsed = start_time.elapsed().as_secs_f64();
        let rate = (idx + 1) as f64 / elapsed;
        let remaining = (total_jobs - idx - 1) as f64 / rate;
        println!(" │ ETA: {:.0}s", remaining);
    }

    println!("└────────────────────────────────────────────────────────┘\n");

    let total_time = start_time.elapsed().as_secs_f64();
    let total_measurements = results.total_measurements();

    // Summary
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║                    Execution Summary                     ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    println!("Jobs:");
    println!(
        "  ✓ Completed: {}/{} ({:.1}%)",
        completed,
        total_jobs,
        completed as f64 / total_jobs as f64 * 100.0
    );
    println!(
        "  ✗ Failed: {}/{} ({:.1}%)",
        failed,
        total_jobs,
        failed as f64 / total_jobs as f64 * 100.0
    );

    println!("\nMeasurements:");
    println!("  Total collected: {}", total_measurements);
    println!(
        "  Average per job: {:.1}",
        total_measurements as f64 / completed as f64
    );

    println!("\nPerformance:");
    println!("  Total time: {:.1}s", total_time);
    println!("  Jobs per second: {:.2}", completed as f64 / total_time);
    println!("  Average job time: {:.1}s", total_time / completed as f64);

    // Statistical quality
    let stable_jobs = results
        .results()
        .iter()
        .filter(|r| {
            r.stats
                .as_ref()
                .map(|s| s.is_stable(exec_config.max_cv))
                .unwrap_or(false)
        })
        .count();

    println!("\nStatistical Quality:");
    println!(
        "  Stable jobs (CV < {:.0}%): {}/{} ({:.1}%)",
        exec_config.max_cv * 100.0,
        stable_jobs,
        completed,
        stable_jobs as f64 / completed as f64 * 100.0
    );

    // Export to JSON
    println!("\nStep 4: Exporting results...");
    let json_output = "pathfinder_results.json";
    let json = serde_json::to_string_pretty(&results)?;
    let mut file = File::create(json_output)?;
    file.write_all(json.as_bytes())?;
    println!("  ✓ Saved to {}", json_output);

    println!("\n✅ Full pathfinder study complete!");
    println!(
        "   {} jobs executed, {} measurements collected",
        completed, total_measurements
    );

    Ok(())
}
