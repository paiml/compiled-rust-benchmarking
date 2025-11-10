//! Pathfinder study runner
//!
//! Executes the pathfinder study with a strategic subset of configurations
//! to validate hypotheses early before running the full 800-job matrix.

use harness::build_matrix::{BuildMatrix, BENCHMARKS};
use harness::config::generator::ConfigGenerator;
use harness::measurement::{JobResult, Measurement, ResultsCollection};
use harness::pathfinder::PathfinderSelector;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Pathfinder Study ===");
    println!("Early hypothesis validation with configuration subset\n");

    // Generate all configurations
    let mut generator = ConfigGenerator::new();
    let all_configs = generator.generate_matrix();
    println!("Total configurations available: {}", all_configs.len());

    // Select pathfinder subset
    let selector = PathfinderSelector::balanced(15);
    let selected_configs = selector.select(all_configs);
    println!(
        "Selected {} pathfinder configurations",
        selected_configs.len()
    );

    // Display selected configurations
    println!("\nSelected configurations:");
    for (i, config) in selected_configs.iter().enumerate() {
        println!(
            "  {}. {} (opt={:?}, lto={:?}, codegen={}, pgo={:?})",
            i + 1,
            config.id,
            config.opt_level,
            config.lto,
            config.codegen_units.value(),
            config.pgo
        );
    }

    // Create build matrix for pathfinder subset
    let mut pathfinder_matrix = BuildMatrix::new();
    for benchmark in BENCHMARKS {
        for config in &selected_configs {
            let job = harness::build_matrix::BuildJob::new(*benchmark, &config.id);
            pathfinder_matrix.add_job(job);
        }
    }

    let total_jobs = pathfinder_matrix.len();
    println!(
        "\nTotal pathfinder jobs: {} ({}×{})",
        total_jobs,
        BENCHMARKS.len(),
        selected_configs.len()
    );
    println!(
        "This is {}% of the full matrix (800 jobs)",
        (total_jobs as f64 / 800.0 * 100.0) as u32
    );

    // Create results collection
    let mut results = ResultsCollection::new();

    println!("\n=== Running Sample Benchmarks ===");
    println!("Running a few sample jobs to demonstrate measurement infrastructure...\n");

    // Run a sample of jobs (just baseline and standard-release for fibonacci)
    let sample_jobs: Vec<_> = pathfinder_matrix
        .jobs()
        .iter()
        .filter(|j| {
            j.benchmark == "fibonacci"
                && (j.config_id == "baseline" || j.config_id == "standard-release")
        })
        .take(2)
        .cloned()
        .collect();

    for job in sample_jobs {
        println!("Running: {}", job.job_id);
        let mut job_result = JobResult::new(job.clone());

        // Run 3 iterations
        for iteration in 1..=3 {
            print!("  Iteration {}... ", iteration);

            // Execute benchmark
            let output = Command::new("cargo")
                .args(["run", "-p", &job.benchmark, "--profile", &job.config_id])
                .output()?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                match Measurement::from_output(&stdout) {
                    Ok(measurement) => {
                        println!(
                            "✓ compute={}μs, total={}μs",
                            measurement.compute_us, measurement.total_us
                        );
                        job_result.add_measurement(measurement);
                    }
                    Err(e) => {
                        println!("✗ Failed to parse: {}", e);
                    }
                }
            } else {
                println!("✗ Execution failed");
            }
        }

        // Display statistics
        if let Some(stats) = &job_result.stats {
            println!(
                "  Stats: mean={:.0}μs, median={:.0}μs, stddev={:.1}μs, CV={:.2}%",
                stats.mean_compute_us,
                stats.median_compute_us,
                stats.stddev_compute_us,
                stats.coefficient_of_variation() * 100.0
            );
        }

        results.add_result(job_result);
        println!();
    }

    println!("=== Pathfinder Study Summary ===");
    println!("Configurations selected: {}", selected_configs.len());
    println!("Total jobs in pathfinder: {}", total_jobs);
    println!("Sample jobs executed: {}", results.len());
    println!(
        "Total measurements collected: {}",
        results.total_measurements()
    );

    println!("\n=== Next Steps ===");
    println!("1. Build all pathfinder configurations");
    println!("2. Execute all {} pathfinder jobs", total_jobs);
    println!("3. Collect 5-10 measurements per job");
    println!("4. Analyze results to validate hypotheses");
    println!("5. Use insights to guide full matrix execution");

    Ok(())
}
