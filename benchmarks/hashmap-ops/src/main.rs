//! BENCH-006: HashMap Operations
//!
//! Workload Type: Memory-bound (hash computation, random access)
//! Expected Result: 999,999,000,000 (sum of 0*2 to 999,999*2)
//! Expected Runtime: ~100ms

use std::collections::HashMap;
use std::time::Instant;

/// Benchmark HashMap insert and lookup operations
fn benchmark_hashmap(size: usize) -> i64 {
    let mut map = HashMap::new();

    // Insert phase
    for i in 0..size {
        map.insert(i, i * 2);
    }

    // Lookup phase
    let mut sum: i64 = 0;
    for i in 0..size {
        if let Some(&val) = map.get(&i) {
            sum += val as i64;
        }
    }

    sum
}

fn main() {
    let t0 = Instant::now();
    let t1 = Instant::now();
    let result = benchmark_hashmap(1_000_000);
    let t2 = Instant::now();

    println!("STARTUP_TIME_US: {}", t1.duration_since(t0).as_micros());
    println!("COMPUTE_TIME_US: {}", t2.duration_since(t1).as_micros());
    println!("RESULT: {}", result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashmap_empty() {
        assert_eq!(benchmark_hashmap(0), 0);
    }

    #[test]
    fn test_hashmap_small() {
        // 0*2 + 1*2 + 2*2 + 3*2 + 4*2 = 0 + 2 + 4 + 6 + 8 = 20
        assert_eq!(benchmark_hashmap(5), 20);
    }

    #[test]
    fn test_hashmap_medium() {
        // Sum of 2*i for i from 0 to 99 = 2 * (0+1+...+99) = 2 * 4950 = 9900
        assert_eq!(benchmark_hashmap(100), 9900);
    }

    #[test]
    fn test_hashmap_target_workload() {
        // Sum of 2*i for i from 0 to 999,999
        // = 2 * (0+1+...+999999) = 2 * n(n-1)/2 = n(n-1)
        // = 1000000 * 999999 = 999,999,000,000
        assert_eq!(benchmark_hashmap(1_000_000), 999_999_000_000);
    }
}
