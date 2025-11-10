//! Parallel build scheduler for executing build matrix jobs
//!
//! This module provides functionality to schedule and execute multiple
//! build jobs in parallel, respecting system resource limits.

use crate::build_matrix::{BuildJob, BuildMatrix};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of a build job execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildResult {
    /// Build succeeded
    Success,
    /// Build failed with error message
    Failure(String),
    /// Build not yet started
    Pending,
    /// Build currently running
    Running,
}

/// Status of a build job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatus {
    /// The build job
    pub job: BuildJob,
    /// Result of the build
    pub result: BuildResult,
    /// Duration in milliseconds (if completed)
    pub duration_ms: Option<u64>,
}

impl JobStatus {
    /// Create a new pending job status
    pub fn new(job: BuildJob) -> Self {
        Self {
            job,
            result: BuildResult::Pending,
            duration_ms: None,
        }
    }

    /// Mark job as running
    pub fn mark_running(&mut self) {
        self.result = BuildResult::Running;
    }

    /// Mark job as successful
    pub fn mark_success(&mut self, duration_ms: u64) {
        self.result = BuildResult::Success;
        self.duration_ms = Some(duration_ms);
    }

    /// Mark job as failed
    pub fn mark_failure(&mut self, error: String) {
        self.result = BuildResult::Failure(error);
    }

    /// Check if job is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.result, BuildResult::Success | BuildResult::Failure(_))
    }

    /// Check if job succeeded
    pub fn is_success(&self) -> bool {
        matches!(self.result, BuildResult::Success)
    }

    /// Check if job failed
    pub fn is_failure(&self) -> bool {
        matches!(self.result, BuildResult::Failure(_))
    }
}

/// Parallel build scheduler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildScheduler {
    /// All job statuses
    statuses: HashMap<String, JobStatus>,
    /// Maximum parallel jobs
    max_parallel: usize,
}

impl BuildScheduler {
    /// Create a new scheduler with default parallelism (number of CPUs)
    pub fn new() -> Self {
        Self::with_parallelism(num_cpus())
    }

    /// Create a new scheduler with specified parallelism
    pub fn with_parallelism(max_parallel: usize) -> Self {
        Self {
            statuses: HashMap::new(),
            max_parallel: max_parallel.max(1),
        }
    }

    /// Initialize scheduler with a build matrix
    pub fn initialize(&mut self, matrix: &BuildMatrix) {
        self.statuses.clear();
        for job in matrix.jobs() {
            let status = JobStatus::new(job.clone());
            self.statuses.insert(job.job_id.clone(), status);
        }
    }

    /// Get the maximum parallelism level
    pub fn max_parallelism(&self) -> usize {
        self.max_parallel
    }

    /// Get total number of jobs
    pub fn total_jobs(&self) -> usize {
        self.statuses.len()
    }

    /// Get number of pending jobs
    pub fn pending_count(&self) -> usize {
        self.statuses
            .values()
            .filter(|s| matches!(s.result, BuildResult::Pending))
            .count()
    }

    /// Get number of running jobs
    pub fn running_count(&self) -> usize {
        self.statuses
            .values()
            .filter(|s| matches!(s.result, BuildResult::Running))
            .count()
    }

    /// Get number of completed jobs
    pub fn completed_count(&self) -> usize {
        self.statuses.values().filter(|s| s.is_complete()).count()
    }

    /// Get number of successful jobs
    pub fn success_count(&self) -> usize {
        self.statuses.values().filter(|s| s.is_success()).count()
    }

    /// Get number of failed jobs
    pub fn failure_count(&self) -> usize {
        self.statuses.values().filter(|s| s.is_failure()).count()
    }

    /// Get status of a specific job
    pub fn get_status(&self, job_id: &str) -> Option<&JobStatus> {
        self.statuses.get(job_id)
    }

    /// Get mutable status of a specific job
    pub fn get_status_mut(&mut self, job_id: &str) -> Option<&mut JobStatus> {
        self.statuses.get_mut(job_id)
    }

    /// Get all job statuses
    pub fn all_statuses(&self) -> Vec<&JobStatus> {
        self.statuses.values().collect()
    }

    /// Get all failed jobs
    pub fn failed_jobs(&self) -> Vec<&JobStatus> {
        self.statuses.values().filter(|s| s.is_failure()).collect()
    }

    /// Get all successful jobs
    pub fn successful_jobs(&self) -> Vec<&JobStatus> {
        self.statuses.values().filter(|s| s.is_success()).collect()
    }

    /// Check if all jobs are complete
    pub fn is_complete(&self) -> bool {
        self.statuses.values().all(|s| s.is_complete())
    }

    /// Get completion percentage
    pub fn completion_percentage(&self) -> f64 {
        if self.total_jobs() == 0 {
            return 100.0;
        }
        (self.completed_count() as f64 / self.total_jobs() as f64) * 100.0
    }

    /// Get average build duration for successful builds
    pub fn average_duration_ms(&self) -> Option<f64> {
        let durations: Vec<u64> = self
            .statuses
            .values()
            .filter_map(|s| s.duration_ms)
            .collect();

        if durations.is_empty() {
            None
        } else {
            let sum: u64 = durations.iter().sum();
            Some(sum as f64 / durations.len() as f64)
        }
    }
}

impl Default for BuildScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Get number of CPU cores
fn num_cpus() -> usize {
    // For now, use a conservative default
    // In a real implementation, we'd use a crate like num_cpus
    4
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        CodegenUnits, LtoSetting, OptLevel, OptimizationConfig, PgoSetting, StripSetting, TargetCpu,
    };

    #[test]
    fn test_build_result_variants() {
        let success = BuildResult::Success;
        let failure = BuildResult::Failure("error".to_string());
        let pending = BuildResult::Pending;
        let running = BuildResult::Running;

        assert!(matches!(success, BuildResult::Success));
        assert!(matches!(failure, BuildResult::Failure(_)));
        assert!(matches!(pending, BuildResult::Pending));
        assert!(matches!(running, BuildResult::Running));
    }

    #[test]
    fn test_job_status_new() {
        let job = BuildJob::new("fibonacci", "baseline");
        let status = JobStatus::new(job.clone());

        assert_eq!(status.job, job);
        assert!(matches!(status.result, BuildResult::Pending));
        assert_eq!(status.duration_ms, None);
    }

    #[test]
    fn test_job_status_mark_running() {
        let job = BuildJob::new("fibonacci", "baseline");
        let mut status = JobStatus::new(job);

        status.mark_running();

        assert!(matches!(status.result, BuildResult::Running));
    }

    #[test]
    fn test_job_status_mark_success() {
        let job = BuildJob::new("fibonacci", "baseline");
        let mut status = JobStatus::new(job);

        status.mark_success(1234);

        assert!(status.is_success());
        assert_eq!(status.duration_ms, Some(1234));
    }

    #[test]
    fn test_job_status_mark_failure() {
        let job = BuildJob::new("fibonacci", "baseline");
        let mut status = JobStatus::new(job);

        status.mark_failure("compile error".to_string());

        assert!(status.is_failure());
        if let BuildResult::Failure(msg) = &status.result {
            assert_eq!(msg, "compile error");
        }
    }

    #[test]
    fn test_job_status_is_complete() {
        let job = BuildJob::new("fibonacci", "baseline");

        let mut status = JobStatus::new(job.clone());
        assert!(!status.is_complete());

        status.mark_running();
        assert!(!status.is_complete());

        status.mark_success(100);
        assert!(status.is_complete());

        let mut status2 = JobStatus::new(job);
        status2.mark_failure("error".to_string());
        assert!(status2.is_complete());
    }

    #[test]
    fn test_build_scheduler_new() {
        let scheduler = BuildScheduler::new();

        assert_eq!(scheduler.max_parallelism(), 4); // Default num_cpus()
        assert_eq!(scheduler.total_jobs(), 0);
    }

    #[test]
    fn test_build_scheduler_with_parallelism() {
        let scheduler = BuildScheduler::with_parallelism(8);

        assert_eq!(scheduler.max_parallelism(), 8);
    }

    #[test]
    fn test_build_scheduler_with_zero_parallelism() {
        let scheduler = BuildScheduler::with_parallelism(0);

        assert_eq!(scheduler.max_parallelism(), 1); // Minimum is 1
    }

    #[test]
    fn test_build_scheduler_initialize() {
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
        let mut scheduler = BuildScheduler::new();

        scheduler.initialize(&matrix);

        assert_eq!(scheduler.total_jobs(), 20); // 10 benchmarks × 2 configs
        assert_eq!(scheduler.pending_count(), 20);
        assert_eq!(scheduler.running_count(), 0);
        assert_eq!(scheduler.completed_count(), 0);
    }

    #[test]
    fn test_build_scheduler_counts() {
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
        let mut scheduler = BuildScheduler::new();
        scheduler.initialize(&matrix);

        // Initially all pending
        assert_eq!(scheduler.pending_count(), 10);
        assert_eq!(scheduler.running_count(), 0);
        assert_eq!(scheduler.success_count(), 0);
        assert_eq!(scheduler.failure_count(), 0);

        // Mark one as running
        if let Some(status) = scheduler.get_status_mut("fibonacci-baseline") {
            status.mark_running();
        }
        assert_eq!(scheduler.pending_count(), 9);
        assert_eq!(scheduler.running_count(), 1);

        // Mark one as success
        if let Some(status) = scheduler.get_status_mut("fibonacci-baseline") {
            status.mark_success(100);
        }
        assert_eq!(scheduler.success_count(), 1);
        assert_eq!(scheduler.running_count(), 0);
        assert_eq!(scheduler.completed_count(), 1);

        // Mark one as failure
        if let Some(status) = scheduler.get_status_mut("ackermann-baseline") {
            status.mark_failure("error".to_string());
        }
        assert_eq!(scheduler.failure_count(), 1);
        assert_eq!(scheduler.completed_count(), 2);
    }

    #[test]
    fn test_build_scheduler_get_status() {
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
        let mut scheduler = BuildScheduler::new();
        scheduler.initialize(&matrix);

        let status = scheduler.get_status("fibonacci-baseline");
        assert!(status.is_some());
        assert_eq!(status.unwrap().job.benchmark, "fibonacci");
        assert_eq!(status.unwrap().job.config_id, "baseline");

        let missing = scheduler.get_status("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_build_scheduler_is_complete() {
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
        let mut scheduler = BuildScheduler::new();
        scheduler.initialize(&matrix);

        assert!(!scheduler.is_complete());

        // Mark all as complete
        for job_id in matrix.jobs().iter().map(|j| j.job_id.clone()) {
            if let Some(status) = scheduler.get_status_mut(&job_id) {
                status.mark_success(100);
            }
        }

        assert!(scheduler.is_complete());
    }

    #[test]
    fn test_build_scheduler_completion_percentage() {
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
        let mut scheduler = BuildScheduler::new();
        scheduler.initialize(&matrix);

        assert_eq!(scheduler.completion_percentage(), 0.0);

        // Complete 5 out of 10
        let jobs: Vec<_> = matrix.jobs().iter().take(5).cloned().collect();
        for job in jobs {
            if let Some(status) = scheduler.get_status_mut(&job.job_id) {
                status.mark_success(100);
            }
        }

        assert_eq!(scheduler.completion_percentage(), 50.0);
    }

    #[test]
    fn test_build_scheduler_average_duration() {
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
        let mut scheduler = BuildScheduler::new();
        scheduler.initialize(&matrix);

        // No completions yet
        assert_eq!(scheduler.average_duration_ms(), None);

        // Complete a few with different durations
        let jobs: Vec<_> = matrix.jobs().iter().take(3).cloned().collect();
        if let Some(status) = scheduler.get_status_mut(&jobs[0].job_id) {
            status.mark_success(100);
        }
        if let Some(status) = scheduler.get_status_mut(&jobs[1].job_id) {
            status.mark_success(200);
        }
        if let Some(status) = scheduler.get_status_mut(&jobs[2].job_id) {
            status.mark_success(300);
        }

        // Average should be (100 + 200 + 300) / 3 = 200
        assert_eq!(scheduler.average_duration_ms(), Some(200.0));
    }

    #[test]
    fn test_build_scheduler_failed_jobs() {
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
        let mut scheduler = BuildScheduler::new();
        scheduler.initialize(&matrix);

        assert_eq!(scheduler.failed_jobs().len(), 0);

        // Fail a few jobs
        if let Some(status) = scheduler.get_status_mut("fibonacci-baseline") {
            status.mark_failure("error1".to_string());
        }
        if let Some(status) = scheduler.get_status_mut("ackermann-baseline") {
            status.mark_failure("error2".to_string());
        }

        let failed = scheduler.failed_jobs();
        assert_eq!(failed.len(), 2);
    }

    #[test]
    fn test_build_scheduler_successful_jobs() {
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
        let mut scheduler = BuildScheduler::new();
        scheduler.initialize(&matrix);

        assert_eq!(scheduler.successful_jobs().len(), 0);

        // Succeed a few jobs
        if let Some(status) = scheduler.get_status_mut("fibonacci-baseline") {
            status.mark_success(100);
        }
        if let Some(status) = scheduler.get_status_mut("ackermann-baseline") {
            status.mark_success(200);
        }

        let successful = scheduler.successful_jobs();
        assert_eq!(successful.len(), 2);
    }
}
