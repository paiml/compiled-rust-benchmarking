//! BENCH-010: Ackermann Function
//!
//! Workload Type: CPU-bound (recursive, stack pressure)
//! Expected Result: ackermann(3, 10) = 8189
//! Expected Runtime: ~10ms

use std::time::Instant;

/// Compute Ackermann function A(m, n)
///
/// The Ackermann function is a classic example of a computable function
/// that is not primitive recursive. It grows extremely rapidly.
///
/// Definition:
/// - A(0, n) = n + 1
/// - A(m, 0) = A(m - 1, 1) for m > 0
/// - A(m, n) = A(m - 1, A(m, n - 1)) for m, n > 0
fn ackermann(m: u32, n: u32) -> u32 {
    match (m, n) {
        (0, n) => n + 1,
        (m, 0) => ackermann(m - 1, 1),
        (m, n) => ackermann(m - 1, ackermann(m, n - 1)),
    }
}

fn main() {
    let t0 = Instant::now();
    let t1 = Instant::now();
    let result = ackermann(3, 10);
    let t2 = Instant::now();

    println!("STARTUP_TIME_US: {}", t1.duration_since(t0).as_micros());
    println!("COMPUTE_TIME_US: {}", t2.duration_since(t1).as_micros());
    println!("RESULT: {}", result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ackermann_base_case_m0() {
        assert_eq!(ackermann(0, 0), 1);
        assert_eq!(ackermann(0, 1), 2);
        assert_eq!(ackermann(0, 5), 6);
        assert_eq!(ackermann(0, 100), 101);
    }

    #[test]
    fn test_ackermann_base_case_n0() {
        assert_eq!(ackermann(1, 0), 2);
        assert_eq!(ackermann(2, 0), 3);
        assert_eq!(ackermann(3, 0), 5);
    }

    #[test]
    fn test_ackermann_small_values() {
        assert_eq!(ackermann(1, 1), 3);
        assert_eq!(ackermann(1, 2), 4);
        assert_eq!(ackermann(2, 2), 7);
        assert_eq!(ackermann(2, 3), 9);
    }

    #[test]
    fn test_ackermann_target_workload() {
        // This is our main benchmark workload
        assert_eq!(ackermann(3, 10), 8189);
    }

    #[test]
    fn test_ackermann_additional_m3_values() {
        assert_eq!(ackermann(3, 0), 5);
        assert_eq!(ackermann(3, 1), 13);
        assert_eq!(ackermann(3, 2), 29);
        assert_eq!(ackermann(3, 3), 61);
    }
}
