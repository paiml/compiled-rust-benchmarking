//! Build matrix display binary
//!
//! Generates and displays the complete build matrix showing all
//! benchmark × configuration combinations.

use harness::build_matrix::BuildMatrix;
use harness::config::generator::ConfigGenerator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating build matrix...\n");

    // Generate configurations
    let mut generator = ConfigGenerator::new();
    let configs = generator.generate_matrix();

    println!("Configurations: {}", configs.len());

    // Generate build matrix
    let matrix = BuildMatrix::generate(configs);

    println!("Build matrix statistics:");
    println!("  Total jobs: {}", matrix.len());
    println!("  Benchmarks: {}", matrix.benchmarks().len());
    println!("  Configurations: {}", matrix.config_ids().len());

    let expected_jobs = matrix.benchmarks().len() * matrix.config_ids().len();
    println!(
        "  Expected jobs ({}×{}): {}",
        matrix.benchmarks().len(),
        matrix.config_ids().len(),
        expected_jobs
    );

    if matrix.len() == expected_jobs {
        println!("  ✓ Matrix is complete!");
    } else {
        println!("  ⚠ Matrix is incomplete!");
    }

    println!("\nBenchmark breakdown:");
    for benchmark in matrix.benchmarks() {
        let jobs = matrix.jobs_for_benchmark(&benchmark);
        println!("  {}: {} jobs", benchmark, jobs.len());
    }

    println!("\nSample build commands:");
    for (i, job) in matrix.jobs().iter().take(5).enumerate() {
        println!("  {}. {}", i + 1, job.build_command());
    }
    println!("  ...");
    println!("  (showing 5 of {} total jobs)", matrix.len());

    println!("\nSample run commands:");
    for (i, job) in matrix.jobs().iter().take(5).enumerate() {
        println!("  {}. {}", i + 1, job.run_command());
    }
    println!("  ...");

    println!("\nSample binary paths:");
    for (i, job) in matrix.jobs().iter().take(5).enumerate() {
        println!("  {}. {}", i + 1, job.binary_path());
    }
    println!("  ...");

    println!(
        "\n✅ Successfully generated build matrix with {} jobs",
        matrix.len()
    );

    Ok(())
}
