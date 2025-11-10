//! BENCH-005: String Parsing
//!
//! Workload Type: CPU-bound (instruction cache, branch prediction)
//! Expected Result: 500,000,500,000 (sum of 1 to 1,000,000)
//! Expected Runtime: ~150ms

use std::time::Instant;

/// Parse and sum integers from a multi-line string
fn parse_and_sum(input: &str) -> i64 {
    input
        .lines()
        .filter_map(|line| line.parse::<i64>().ok())
        .sum()
}

fn main() {
    let input = (1..=1_000_000)
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join("\n");

    let t0 = Instant::now();
    let t1 = Instant::now();
    let result = parse_and_sum(&input);
    let t2 = Instant::now();

    println!("STARTUP_TIME_US: {}", t1.duration_since(t0).as_micros());
    println!("COMPUTE_TIME_US: {}", t2.duration_since(t1).as_micros());
    println!("RESULT: {}", result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_sum_empty() {
        assert_eq!(parse_and_sum(""), 0);
    }

    #[test]
    fn test_parse_and_sum_single() {
        assert_eq!(parse_and_sum("42"), 42);
    }

    #[test]
    fn test_parse_and_sum_multiple() {
        assert_eq!(parse_and_sum("1\n2\n3\n4\n5"), 15);
    }

    #[test]
    fn test_parse_and_sum_with_invalid() {
        assert_eq!(parse_and_sum("1\ninvalid\n3\n4"), 8);
    }

    #[test]
    fn test_parse_and_sum_target_workload() {
        // Sum of 1 to 1,000,000 = n(n+1)/2 = 500,000,500,000
        let input = (1..=1_000_000)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(parse_and_sum(&input), 500_000_500_000);
    }
}
