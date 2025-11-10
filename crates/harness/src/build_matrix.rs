//! Build matrix generation for benchmark execution
//!
//! This module generates the build matrix combining benchmarks with
//! optimization configurations.

use crate::config::OptimizationConfig;
use serde::{Deserialize, Serialize};

/// List of all benchmark names
pub const BENCHMARKS: &[&str] = &[
    "ackermann",
    "fibonacci",
    "prime-sieve",
    "matrix-mult",
    "quicksort",
    "string-parse",
    "hashmap-ops",
    "file-io",
    "json-parse",
    "btreemap-ops",
];

/// A single build job combining a benchmark with a configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildJob {
    /// Benchmark name
    pub benchmark: String,
    /// Configuration ID
    pub config_id: String,
    /// Full job ID (benchmark-config)
    pub job_id: String,
}

impl BuildJob {
    /// Create a new build job
    pub fn new(benchmark: impl Into<String>, config_id: impl Into<String>) -> Self {
        let benchmark = benchmark.into();
        let config_id = config_id.into();
        let job_id = format!("{}-{}", benchmark, config_id);
        Self {
            benchmark,
            config_id,
            job_id,
        }
    }

    /// Generate the cargo build command for this job
    pub fn build_command(&self) -> String {
        format!(
            "cargo build -p {} --profile {}",
            self.benchmark, self.config_id
        )
    }

    /// Generate the cargo run command for this job
    pub fn run_command(&self) -> String {
        format!(
            "cargo run -p {} --profile {}",
            self.benchmark, self.config_id
        )
    }

    /// Get the expected binary path for this job
    pub fn binary_path(&self) -> String {
        format!("target/{}/{}", self.config_id, self.benchmark)
    }
}

/// Build matrix combining all benchmarks with all configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMatrix {
    /// All build jobs
    jobs: Vec<BuildJob>,
}

impl BuildMatrix {
    /// Create a new empty build matrix
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    /// Generate the full build matrix from benchmarks and configurations
    pub fn generate(configs: &[OptimizationConfig]) -> Self {
        let mut matrix = Self::new();

        for benchmark in BENCHMARKS {
            for config in configs {
                matrix.add_job(BuildJob::new(*benchmark, &config.id));
            }
        }

        matrix
    }

    /// Add a job to the matrix
    pub fn add_job(&mut self, job: BuildJob) {
        self.jobs.push(job);
    }

    /// Get all jobs
    pub fn jobs(&self) -> &[BuildJob] {
        &self.jobs
    }

    /// Get total number of jobs
    pub fn len(&self) -> usize {
        self.jobs.len()
    }

    /// Check if matrix is empty
    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }

    /// Get jobs for a specific benchmark
    pub fn jobs_for_benchmark(&self, benchmark: &str) -> Vec<&BuildJob> {
        self.jobs
            .iter()
            .filter(|job| job.benchmark == benchmark)
            .collect()
    }

    /// Get jobs for a specific configuration
    pub fn jobs_for_config(&self, config_id: &str) -> Vec<&BuildJob> {
        self.jobs
            .iter()
            .filter(|job| job.config_id == config_id)
            .collect()
    }

    /// Get all unique benchmark names
    pub fn benchmarks(&self) -> Vec<String> {
        let mut benchmarks: Vec<String> = self
            .jobs
            .iter()
            .map(|job| job.benchmark.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        benchmarks.sort();
        benchmarks
    }

    /// Get all unique configuration IDs
    pub fn config_ids(&self) -> Vec<String> {
        let mut config_ids: Vec<String> = self
            .jobs
            .iter()
            .map(|job| job.config_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        config_ids.sort();
        config_ids
    }
}

impl Default for BuildMatrix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CodegenUnits, LtoSetting, OptLevel, PgoSetting, StripSetting, TargetCpu};

    #[test]
    fn test_benchmarks_constant_has_ten_items() {
        assert_eq!(BENCHMARKS.len(), 10, "Should have exactly 10 benchmarks");
    }

    #[test]
    fn test_benchmarks_constant_contains_expected_benchmarks() {
        assert!(BENCHMARKS.contains(&"ackermann"));
        assert!(BENCHMARKS.contains(&"fibonacci"));
        assert!(BENCHMARKS.contains(&"prime-sieve"));
        assert!(BENCHMARKS.contains(&"matrix-mult"));
        assert!(BENCHMARKS.contains(&"quicksort"));
        assert!(BENCHMARKS.contains(&"string-parse"));
        assert!(BENCHMARKS.contains(&"hashmap-ops"));
        assert!(BENCHMARKS.contains(&"file-io"));
        assert!(BENCHMARKS.contains(&"json-parse"));
        assert!(BENCHMARKS.contains(&"btreemap-ops"));
    }

    #[test]
    fn test_build_job_new() {
        let job = BuildJob::new("fibonacci", "baseline");

        assert_eq!(job.benchmark, "fibonacci");
        assert_eq!(job.config_id, "baseline");
        assert_eq!(job.job_id, "fibonacci-baseline");
    }

    #[test]
    fn test_build_job_build_command() {
        let job = BuildJob::new("fibonacci", "opt-o3");

        assert_eq!(
            job.build_command(),
            "cargo build -p fibonacci --profile opt-o3"
        );
    }

    #[test]
    fn test_build_job_run_command() {
        let job = BuildJob::new("fibonacci", "opt-o3");

        assert_eq!(job.run_command(), "cargo run -p fibonacci --profile opt-o3");
    }

    #[test]
    fn test_build_job_binary_path() {
        let job = BuildJob::new("fibonacci", "opt-o3");

        assert_eq!(job.binary_path(), "target/opt-o3/fibonacci");
    }

    #[test]
    fn test_build_matrix_new() {
        let matrix = BuildMatrix::new();

        assert_eq!(matrix.len(), 0);
        assert!(matrix.is_empty());
    }

    #[test]
    fn test_build_matrix_add_job() {
        let mut matrix = BuildMatrix::new();
        let job = BuildJob::new("fibonacci", "baseline");

        matrix.add_job(job.clone());

        assert_eq!(matrix.len(), 1);
        assert!(!matrix.is_empty());
        assert_eq!(matrix.jobs()[0], job);
    }

    #[test]
    fn test_build_matrix_generate() {
        // Create a small set of test configurations
        let configs = vec![
            OptimizationConfig::new(
                "baseline".to_string(),
                OptLevel::O0,
                LtoSetting::Off,
                CodegenUnits::Sixteen,
                PgoSetting::Off,
                TargetCpu::Generic,
                StripSetting::None,
            ),
            OptimizationConfig::new(
                "opt-o3".to_string(),
                OptLevel::O3,
                LtoSetting::Off,
                CodegenUnits::Sixteen,
                PgoSetting::Off,
                TargetCpu::Generic,
                StripSetting::None,
            ),
        ];

        let matrix = BuildMatrix::generate(&configs);

        // Should have 10 benchmarks × 2 configs = 20 jobs
        assert_eq!(matrix.len(), 20);
        assert!(!matrix.is_empty());
    }

    #[test]
    fn test_build_matrix_jobs_for_benchmark() {
        let configs = vec![
            OptimizationConfig::new(
                "baseline".to_string(),
                OptLevel::O0,
                LtoSetting::Off,
                CodegenUnits::Sixteen,
                PgoSetting::Off,
                TargetCpu::Generic,
                StripSetting::None,
            ),
            OptimizationConfig::new(
                "opt-o3".to_string(),
                OptLevel::O3,
                LtoSetting::Off,
                CodegenUnits::Sixteen,
                PgoSetting::Off,
                TargetCpu::Generic,
                StripSetting::None,
            ),
        ];

        let matrix = BuildMatrix::generate(&configs);
        let fib_jobs = matrix.jobs_for_benchmark("fibonacci");

        assert_eq!(fib_jobs.len(), 2);
        assert!(fib_jobs.iter().all(|job| job.benchmark == "fibonacci"));
    }

    #[test]
    fn test_build_matrix_jobs_for_config() {
        let configs = vec![
            OptimizationConfig::new(
                "baseline".to_string(),
                OptLevel::O0,
                LtoSetting::Off,
                CodegenUnits::Sixteen,
                PgoSetting::Off,
                TargetCpu::Generic,
                StripSetting::None,
            ),
            OptimizationConfig::new(
                "opt-o3".to_string(),
                OptLevel::O3,
                LtoSetting::Off,
                CodegenUnits::Sixteen,
                PgoSetting::Off,
                TargetCpu::Generic,
                StripSetting::None,
            ),
        ];

        let matrix = BuildMatrix::generate(&configs);
        let baseline_jobs = matrix.jobs_for_config("baseline");

        assert_eq!(baseline_jobs.len(), 10);
        assert!(baseline_jobs.iter().all(|job| job.config_id == "baseline"));
    }

    #[test]
    fn test_build_matrix_benchmarks() {
        let configs = vec![OptimizationConfig::new(
            "baseline".to_string(),
            OptLevel::O0,
            LtoSetting::Off,
            CodegenUnits::Sixteen,
            PgoSetting::Off,
            TargetCpu::Generic,
            StripSetting::None,
        )];

        let matrix = BuildMatrix::generate(&configs);
        let benchmarks = matrix.benchmarks();

        assert_eq!(benchmarks.len(), 10);
        assert!(benchmarks.contains(&"fibonacci".to_string()));
        assert!(benchmarks.contains(&"ackermann".to_string()));
    }

    #[test]
    fn test_build_matrix_config_ids() {
        let configs = vec![
            OptimizationConfig::new(
                "baseline".to_string(),
                OptLevel::O0,
                LtoSetting::Off,
                CodegenUnits::Sixteen,
                PgoSetting::Off,
                TargetCpu::Generic,
                StripSetting::None,
            ),
            OptimizationConfig::new(
                "opt-o3".to_string(),
                OptLevel::O3,
                LtoSetting::Off,
                CodegenUnits::Sixteen,
                PgoSetting::Off,
                TargetCpu::Generic,
                StripSetting::None,
            ),
        ];

        let matrix = BuildMatrix::generate(&configs);
        let config_ids = matrix.config_ids();

        assert_eq!(config_ids.len(), 2);
        assert!(config_ids.contains(&"baseline".to_string()));
        assert!(config_ids.contains(&"opt-o3".to_string()));
    }

    #[test]
    fn test_build_matrix_full_generation() {
        // Test with a realistic number of configs (similar to our 80)
        use crate::config::generator::ConfigGenerator;

        let mut generator = ConfigGenerator::new();
        let configs = generator.generate_matrix();

        let matrix = BuildMatrix::generate(configs);

        // Should have 10 benchmarks × ~80 configs = ~800 jobs
        assert!(
            (700..=900).contains(&matrix.len()),
            "Expected ~800 jobs, got {}",
            matrix.len()
        );

        // Verify all benchmarks are present
        let benchmarks = matrix.benchmarks();
        assert_eq!(benchmarks.len(), 10);

        // Verify each benchmark has the same number of jobs
        for benchmark in &benchmarks {
            let jobs = matrix.jobs_for_benchmark(benchmark);
            assert!(
                jobs.len() >= 70,
                "Benchmark {} should have at least 70 jobs",
                benchmark
            );
        }
    }

    #[test]
    fn test_build_job_id_uniqueness() {
        let configs = vec![
            OptimizationConfig::new(
                "baseline".to_string(),
                OptLevel::O0,
                LtoSetting::Off,
                CodegenUnits::Sixteen,
                PgoSetting::Off,
                TargetCpu::Generic,
                StripSetting::None,
            ),
            OptimizationConfig::new(
                "opt-o3".to_string(),
                OptLevel::O3,
                LtoSetting::Off,
                CodegenUnits::Sixteen,
                PgoSetting::Off,
                TargetCpu::Generic,
                StripSetting::None,
            ),
        ];

        let matrix = BuildMatrix::generate(&configs);

        // Collect all job IDs
        let job_ids: std::collections::HashSet<_> =
            matrix.jobs().iter().map(|job| &job.job_id).collect();

        // All job IDs should be unique
        assert_eq!(job_ids.len(), matrix.len());
    }
}
