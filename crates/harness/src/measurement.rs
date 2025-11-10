//! Measurement infrastructure for benchmark execution and data collection
//!
//! This module provides functionality to execute benchmarks, collect timing data,
//! and compute statistics across multiple runs.

use crate::build_matrix::BuildJob;
use serde::{Deserialize, Serialize};
use stats::{mean, median};

/// Single benchmark measurement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Measurement {
    /// Startup time in microseconds
    pub startup_us: u64,
    /// Compute time in microseconds
    pub compute_us: u64,
    /// Total time in microseconds
    pub total_us: u64,
    /// Result value (for validation)
    pub result: String,
}

impl Measurement {
    /// Create a new measurement
    pub fn new(startup_us: u64, compute_us: u64, result: String) -> Self {
        Self {
            startup_us,
            compute_us,
            total_us: startup_us + compute_us,
            result,
        }
    }

    /// Parse measurement from benchmark output
    pub fn from_output(output: &str) -> Result<Self, String> {
        let mut startup_us = None;
        let mut compute_us = None;
        let mut result = None;

        for line in output.lines() {
            if line.starts_with("STARTUP_TIME_US:") {
                startup_us = line.split(':').nth(1).and_then(|s| s.trim().parse().ok());
            } else if line.starts_with("COMPUTE_TIME_US:") {
                compute_us = line.split(':').nth(1).and_then(|s| s.trim().parse().ok());
            } else if line.starts_with("RESULT:") {
                result = line.split(':').nth(1).map(|s| s.trim().to_string());
            }
        }

        match (startup_us, compute_us, result) {
            (Some(s), Some(c), Some(r)) => Ok(Self::new(s, c, r)),
            _ => Err("Failed to parse measurement output".to_string()),
        }
    }
}

/// Statistics for multiple measurements
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurementStats {
    /// Number of measurements
    pub count: usize,
    /// Mean compute time in microseconds
    pub mean_compute_us: f64,
    /// Median compute time in microseconds
    pub median_compute_us: f64,
    /// Minimum compute time in microseconds
    pub min_compute_us: u64,
    /// Maximum compute time in microseconds
    pub max_compute_us: u64,
    /// Standard deviation of compute time
    pub stddev_compute_us: f64,
    /// Mean total time in microseconds
    pub mean_total_us: f64,
    /// Result value (should be consistent)
    pub result: String,
}

impl MeasurementStats {
    /// Compute statistics from multiple measurements
    pub fn from_measurements(measurements: &[Measurement]) -> Result<Self, String> {
        if measurements.is_empty() {
            return Err("Cannot compute stats from empty measurements".to_string());
        }

        let compute_times: Vec<f64> = measurements.iter().map(|m| m.compute_us as f64).collect();
        let total_times: Vec<f64> = measurements.iter().map(|m| m.total_us as f64).collect();

        let mean_compute = mean(&compute_times);
        let median_compute = median(&compute_times);
        let min_compute = measurements.iter().map(|m| m.compute_us).min().unwrap();
        let max_compute = measurements.iter().map(|m| m.compute_us).max().unwrap();

        // Calculate standard deviation
        let variance = compute_times
            .iter()
            .map(|&t| (t - mean_compute).powi(2))
            .sum::<f64>()
            / compute_times.len() as f64;
        let stddev = variance.sqrt();

        let mean_total = mean(&total_times);

        // All results should be the same
        let result = measurements[0].result.clone();

        Ok(Self {
            count: measurements.len(),
            mean_compute_us: mean_compute,
            median_compute_us: median_compute,
            min_compute_us: min_compute,
            max_compute_us: max_compute,
            stddev_compute_us: stddev,
            mean_total_us: mean_total,
            result,
        })
    }

    /// Get the coefficient of variation (CV = stddev / mean)
    pub fn coefficient_of_variation(&self) -> f64 {
        if self.mean_compute_us > 0.0 {
            self.stddev_compute_us / self.mean_compute_us
        } else {
            0.0
        }
    }

    /// Check if measurements are stable (CV < threshold)
    pub fn is_stable(&self, cv_threshold: f64) -> bool {
        self.coefficient_of_variation() < cv_threshold
    }
}

/// Results for a specific build job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    /// The build job
    pub job: BuildJob,
    /// All measurements
    pub measurements: Vec<Measurement>,
    /// Statistics
    pub stats: Option<MeasurementStats>,
}

impl JobResult {
    /// Create a new job result
    pub fn new(job: BuildJob) -> Self {
        Self {
            job,
            measurements: Vec::new(),
            stats: None,
        }
    }

    /// Add a measurement
    pub fn add_measurement(&mut self, measurement: Measurement) {
        self.measurements.push(measurement);
        self.update_stats();
    }

    /// Update statistics
    fn update_stats(&mut self) {
        if !self.measurements.is_empty() {
            self.stats = MeasurementStats::from_measurements(&self.measurements).ok();
        }
    }

    /// Get the number of measurements
    pub fn measurement_count(&self) -> usize {
        self.measurements.len()
    }

    /// Check if job has sufficient measurements
    pub fn has_sufficient_measurements(&self, min_count: usize) -> bool {
        self.measurements.len() >= min_count
    }
}

/// Collection of all job results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultsCollection {
    /// All job results
    results: Vec<JobResult>,
}

impl ResultsCollection {
    /// Create a new results collection
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    /// Add a job result
    pub fn add_result(&mut self, result: JobResult) {
        self.results.push(result);
    }

    /// Get all results
    pub fn results(&self) -> &[JobResult] {
        &self.results
    }

    /// Get number of results
    pub fn len(&self) -> usize {
        self.results.len()
    }

    /// Check if collection is empty
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Get results for a specific job
    pub fn get_result(&self, job_id: &str) -> Option<&JobResult> {
        self.results.iter().find(|r| r.job.job_id == job_id)
    }

    /// Get total number of measurements across all jobs
    pub fn total_measurements(&self) -> usize {
        self.results.iter().map(|r| r.measurement_count()).sum()
    }
}

impl Default for ResultsCollection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measurement_new() {
        let m = Measurement::new(100, 500, "42".to_string());

        assert_eq!(m.startup_us, 100);
        assert_eq!(m.compute_us, 500);
        assert_eq!(m.total_us, 600);
        assert_eq!(m.result, "42");
    }

    #[test]
    fn test_measurement_from_output() {
        let output = "STARTUP_TIME_US: 100\nCOMPUTE_TIME_US: 500\nRESULT: 42\n";

        let m = Measurement::from_output(output).unwrap();

        assert_eq!(m.startup_us, 100);
        assert_eq!(m.compute_us, 500);
        assert_eq!(m.total_us, 600);
        assert_eq!(m.result, "42");
    }

    #[test]
    fn test_measurement_from_output_with_extra_lines() {
        let output = "Some debug output\nSTARTUP_TIME_US: 200\nMore debug\nCOMPUTE_TIME_US: 1000\nRESULT: 123\nMore output\n";

        let m = Measurement::from_output(output).unwrap();

        assert_eq!(m.startup_us, 200);
        assert_eq!(m.compute_us, 1000);
        assert_eq!(m.total_us, 1200);
        assert_eq!(m.result, "123");
    }

    #[test]
    fn test_measurement_from_output_missing_data() {
        let output = "STARTUP_TIME_US: 100\n";

        let result = Measurement::from_output(output);

        assert!(result.is_err());
    }

    #[test]
    fn test_measurement_stats_from_measurements() {
        let measurements = vec![
            Measurement::new(10, 100, "42".to_string()),
            Measurement::new(10, 110, "42".to_string()),
            Measurement::new(10, 90, "42".to_string()),
        ];

        let stats = MeasurementStats::from_measurements(&measurements).unwrap();

        assert_eq!(stats.count, 3);
        assert_eq!(stats.mean_compute_us, 100.0);
        assert_eq!(stats.median_compute_us, 100.0);
        assert_eq!(stats.min_compute_us, 90);
        assert_eq!(stats.max_compute_us, 110);
        assert_eq!(stats.result, "42");
    }

    #[test]
    fn test_measurement_stats_empty() {
        let measurements: Vec<Measurement> = vec![];

        let result = MeasurementStats::from_measurements(&measurements);

        assert!(result.is_err());
    }

    #[test]
    fn test_measurement_stats_coefficient_of_variation() {
        let measurements = vec![
            Measurement::new(0, 100, "42".to_string()),
            Measurement::new(0, 100, "42".to_string()),
            Measurement::new(0, 100, "42".to_string()),
        ];

        let stats = MeasurementStats::from_measurements(&measurements).unwrap();

        assert_eq!(stats.coefficient_of_variation(), 0.0); // Perfect stability
    }

    #[test]
    fn test_measurement_stats_is_stable() {
        let stable_measurements = vec![
            Measurement::new(0, 100, "42".to_string()),
            Measurement::new(0, 101, "42".to_string()),
            Measurement::new(0, 99, "42".to_string()),
        ];

        let stats = MeasurementStats::from_measurements(&stable_measurements).unwrap();

        assert!(stats.is_stable(0.1)); // CV should be < 0.1
    }

    #[test]
    fn test_job_result_new() {
        let job = BuildJob::new("fibonacci", "baseline");
        let result = JobResult::new(job.clone());

        assert_eq!(result.job, job);
        assert_eq!(result.measurement_count(), 0);
        assert!(result.stats.is_none());
    }

    #[test]
    fn test_job_result_add_measurement() {
        let job = BuildJob::new("fibonacci", "baseline");
        let mut result = JobResult::new(job);

        result.add_measurement(Measurement::new(10, 100, "42".to_string()));

        assert_eq!(result.measurement_count(), 1);
        assert!(result.stats.is_some());
    }

    #[test]
    fn test_job_result_has_sufficient_measurements() {
        let job = BuildJob::new("fibonacci", "baseline");
        let mut result = JobResult::new(job);

        assert!(!result.has_sufficient_measurements(3));

        result.add_measurement(Measurement::new(10, 100, "42".to_string()));
        result.add_measurement(Measurement::new(10, 100, "42".to_string()));
        result.add_measurement(Measurement::new(10, 100, "42".to_string()));

        assert!(result.has_sufficient_measurements(3));
    }

    #[test]
    fn test_results_collection_new() {
        let collection = ResultsCollection::new();

        assert_eq!(collection.len(), 0);
        assert!(collection.is_empty());
    }

    #[test]
    fn test_results_collection_add_result() {
        let mut collection = ResultsCollection::new();
        let job = BuildJob::new("fibonacci", "baseline");
        let result = JobResult::new(job);

        collection.add_result(result);

        assert_eq!(collection.len(), 1);
        assert!(!collection.is_empty());
    }

    #[test]
    fn test_results_collection_get_result() {
        let mut collection = ResultsCollection::new();
        let job = BuildJob::new("fibonacci", "baseline");
        let result = JobResult::new(job);

        collection.add_result(result);

        let found = collection.get_result("fibonacci-baseline");
        assert!(found.is_some());
        assert_eq!(found.unwrap().job.job_id, "fibonacci-baseline");

        let not_found = collection.get_result("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_results_collection_total_measurements() {
        let mut collection = ResultsCollection::new();

        let job1 = BuildJob::new("fibonacci", "baseline");
        let mut result1 = JobResult::new(job1);
        result1.add_measurement(Measurement::new(10, 100, "42".to_string()));
        result1.add_measurement(Measurement::new(10, 100, "42".to_string()));

        let job2 = BuildJob::new("ackermann", "baseline");
        let mut result2 = JobResult::new(job2);
        result2.add_measurement(Measurement::new(10, 200, "8189".to_string()));
        result2.add_measurement(Measurement::new(10, 200, "8189".to_string()));
        result2.add_measurement(Measurement::new(10, 200, "8189".to_string()));

        collection.add_result(result1);
        collection.add_result(result2);

        assert_eq!(collection.total_measurements(), 5);
    }
}
