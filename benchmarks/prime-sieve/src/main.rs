//! BENCH-002: Prime Sieve (Sieve of Eratosthenes)
//!
//! Workload Type: Memory-bound (iterative, data cache pressure)
//! Expected Result: count_primes(1,000,000) = 78,498
//! Expected Runtime: ~100ms

use std::time::Instant;

/// Count the number of primes up to and including limit using Sieve of Eratosthenes
///
/// The Sieve of Eratosthenes is an ancient algorithm for finding all primes
/// up to a specified integer. It works by iteratively marking multiples of
/// each prime as composite.
///
/// Time Complexity: O(n log log n)
/// Space Complexity: O(n)
fn count_primes(limit: usize) -> usize {
    if limit < 2 {
        return 0;
    }

    let mut sieve = vec![true; limit + 1];
    sieve[0] = false;
    sieve[1] = false;

    for i in 2..=((limit as f64).sqrt() as usize) {
        if sieve[i] {
            for j in ((i * i)..=limit).step_by(i) {
                sieve[j] = false;
            }
        }
    }

    sieve.iter().filter(|&&x| x).count()
}

fn main() {
    let t0 = Instant::now();
    let t1 = Instant::now();
    let result = count_primes(1_000_000);
    let t2 = Instant::now();

    println!("STARTUP_TIME_US: {}", t1.duration_since(t0).as_micros());
    println!("COMPUTE_TIME_US: {}", t2.duration_since(t1).as_micros());
    println!("RESULT: {}", result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_primes_edge_cases() {
        assert_eq!(count_primes(0), 0);
        assert_eq!(count_primes(1), 0);
        assert_eq!(count_primes(2), 1); // 2 is prime
    }

    #[test]
    fn test_count_primes_small_values() {
        assert_eq!(count_primes(3), 2); // 2, 3
        assert_eq!(count_primes(5), 3); // 2, 3, 5
        assert_eq!(count_primes(10), 4); // 2, 3, 5, 7
        assert_eq!(count_primes(20), 8); // 2, 3, 5, 7, 11, 13, 17, 19
    }

    #[test]
    fn test_count_primes_medium_values() {
        assert_eq!(count_primes(100), 25);
        assert_eq!(count_primes(1000), 168);
        assert_eq!(count_primes(10_000), 1229);
    }

    #[test]
    fn test_count_primes_target_workload() {
        // This is our main benchmark workload
        assert_eq!(count_primes(1_000_000), 78_498);
    }

    #[test]
    fn test_count_primes_known_values() {
        assert_eq!(count_primes(100_000), 9592);
        assert_eq!(count_primes(500_000), 41_538);
    }
}
