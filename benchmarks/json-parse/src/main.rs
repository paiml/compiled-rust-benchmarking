//! BENCH-008: JSON Parsing
//!
//! Workload Type: CPU-bound (parsing, allocation)
//! Expected Result: 10,000 (number of parsed objects)
//! Expected Runtime: ~100ms

use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Serialize, Deserialize, Debug)]
struct Record {
    id: u32,
    name: String,
    value: f64,
}

/// Benchmark JSON parsing: serialize and deserialize 10K objects
fn benchmark_json(count: usize) -> usize {
    let records: Vec<Record> = (0..count)
        .map(|i| Record {
            id: i as u32,
            name: format!("record_{}", i),
            value: i as f64 * 1.5,
        })
        .collect();

    // Serialize
    let json = serde_json::to_string(&records).unwrap();

    // Deserialize
    let parsed: Vec<Record> = serde_json::from_str(&json).unwrap();

    parsed.len()
}

fn main() {
    let t0 = Instant::now();
    let t1 = Instant::now();
    let result = benchmark_json(10_000);
    let t2 = Instant::now();

    println!("STARTUP_TIME_US: {}", t1.duration_since(t0).as_micros());
    println!("COMPUTE_TIME_US: {}", t2.duration_since(t1).as_micros());
    println!("RESULT: {}", result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_empty() {
        assert_eq!(benchmark_json(0), 0);
    }

    #[test]
    fn test_json_single() {
        assert_eq!(benchmark_json(1), 1);
    }

    #[test]
    fn test_json_small() {
        assert_eq!(benchmark_json(10), 10);
    }

    #[test]
    fn test_json_medium() {
        assert_eq!(benchmark_json(1000), 1000);
    }

    #[test]
    fn test_json_target_workload() {
        assert_eq!(benchmark_json(10_000), 10_000);
    }
}
