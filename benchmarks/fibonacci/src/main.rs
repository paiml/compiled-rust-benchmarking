//! BENCH-001: Recursive Fibonacci
//!
//! Workload Type: CPU-bound (recursive, instruction cache pressure)
//! Expected Result: fibonacci(40) = 102,334,155
//! Expected Runtime: ~500ms

use std::time::Instant;

/// Compute Fibonacci number using naive recursive algorithm
///
/// This is deliberately inefficient (O(2^n) time complexity) to stress-test
/// compiler optimizations around recursion and function call overhead.
///
/// Definition:
/// - F(0) = 0
/// - F(1) = 1
/// - F(n) = F(n-1) + F(n-2) for n > 1
fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

fn main() {
    let t0 = Instant::now();
    let t1 = Instant::now();
    let result = fibonacci(40);
    let t2 = Instant::now();

    println!("STARTUP_TIME_US: {}", t1.duration_since(t0).as_micros());
    println!("COMPUTE_TIME_US: {}", t2.duration_since(t1).as_micros());
    println!("RESULT: {}", result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci_base_cases() {
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(1), 1);
    }

    #[test]
    fn test_fibonacci_small_values() {
        assert_eq!(fibonacci(2), 1);
        assert_eq!(fibonacci(3), 2);
        assert_eq!(fibonacci(4), 3);
        assert_eq!(fibonacci(5), 5);
        assert_eq!(fibonacci(6), 8);
        assert_eq!(fibonacci(7), 13);
        assert_eq!(fibonacci(8), 21);
    }

    #[test]
    fn test_fibonacci_medium_values() {
        assert_eq!(fibonacci(10), 55);
        assert_eq!(fibonacci(15), 610);
        assert_eq!(fibonacci(20), 6765);
    }

    #[test]
    fn test_fibonacci_target_workload() {
        // This is our main benchmark workload
        assert_eq!(fibonacci(40), 102_334_155);
    }

    #[test]
    fn test_fibonacci_additional_values() {
        assert_eq!(fibonacci(25), 75025);
        assert_eq!(fibonacci(30), 832040);
        assert_eq!(fibonacci(35), 9227465);
    }
}
