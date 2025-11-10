//! Benchmark harness for compiled Rust optimization research
//!
//! This crate provides the test harness infrastructure for running
//! benchmarks across multiple optimization configurations.

pub mod build_matrix;
pub mod config;
pub mod measurement;
pub mod pathfinder;
pub mod scheduler;

/// Harness version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_is_set() {
        assert_eq!(VERSION, "0.1.0");
    }
}
