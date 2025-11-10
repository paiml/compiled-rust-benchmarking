//! BENCH-003: Matrix Multiplication
//!
//! Workload Type: Memory-bound (cache hierarchy sensitive)
//! Expected Result: c[0][0] = 256.0 (128 * 2.0)
//! Expected Runtime: ~50ms

use std::time::Instant;

type Matrix = Vec<Vec<f64>>;

/// Perform naive matrix multiplication: C = A × B
///
/// Time Complexity: O(n³)
/// Space Complexity: O(n²)
///
/// This is deliberately the naive O(n³) algorithm to stress-test
/// cache hierarchy and memory bandwidth.
#[allow(clippy::needless_range_loop)]
fn matrix_multiply(a: &Matrix, b: &Matrix) -> Matrix {
    let n = a.len();
    let mut c = vec![vec![0.0; n]; n];

    for i in 0..n {
        for j in 0..n {
            for k in 0..n {
                c[i][j] += a[i][k] * b[k][j];
            }
        }
    }

    c
}

fn main() {
    let size = 128;
    let a = vec![vec![1.0; size]; size];
    let b = vec![vec![2.0; size]; size];

    let t0 = Instant::now();
    let t1 = Instant::now();
    let c = matrix_multiply(&a, &b);
    let t2 = Instant::now();

    println!("STARTUP_TIME_US: {}", t1.duration_since(t0).as_micros());
    println!("COMPUTE_TIME_US: {}", t2.duration_since(t1).as_micros());
    println!("RESULT: {}", c[0][0]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_multiply_2x2_identity() {
        let a = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let b = vec![vec![5.0, 6.0], vec![7.0, 8.0]];
        let c = matrix_multiply(&a, &b);

        assert_eq!(c[0][0], 5.0);
        assert_eq!(c[0][1], 6.0);
        assert_eq!(c[1][0], 7.0);
        assert_eq!(c[1][1], 8.0);
    }

    #[test]
    fn test_matrix_multiply_2x2() {
        let a = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let b = vec![vec![5.0, 6.0], vec![7.0, 8.0]];
        let c = matrix_multiply(&a, &b);

        // [1 2] × [5 6] = [1×5+2×7  1×6+2×8] = [19 22]
        // [3 4]   [7 8]   [3×5+4×7  3×6+4×8]   [43 50]
        assert_eq!(c[0][0], 19.0);
        assert_eq!(c[0][1], 22.0);
        assert_eq!(c[1][0], 43.0);
        assert_eq!(c[1][1], 50.0);
    }

    #[test]
    #[allow(clippy::needless_range_loop)]
    fn test_matrix_multiply_3x3_uniform() {
        let a = vec![vec![2.0; 3]; 3];
        let b = vec![vec![3.0; 3]; 3];
        let c = matrix_multiply(&a, &b);

        // Each element should be 2.0 × 3.0 × 3 = 18.0
        for i in 0..3 {
            for j in 0..3 {
                assert_eq!(c[i][j], 18.0);
            }
        }
    }

    #[test]
    fn test_matrix_multiply_target_workload() {
        // This is our main benchmark workload: 128×128
        let size = 128;
        let a = vec![vec![1.0; size]; size];
        let b = vec![vec![2.0; size]; size];
        let c = matrix_multiply(&a, &b);

        // Each element should be 1.0 × 2.0 × 128 = 256.0
        assert_eq!(c[0][0], 256.0);
        assert_eq!(c[size - 1][size - 1], 256.0);
        assert_eq!(c[64][64], 256.0);
    }

    #[test]
    fn test_matrix_multiply_small_sizes() {
        // Test 1×1
        let a1 = vec![vec![5.0]];
        let b1 = vec![vec![7.0]];
        let c1 = matrix_multiply(&a1, &b1);
        assert_eq!(c1[0][0], 35.0);

        // Test 4×4 uniform
        let a4 = vec![vec![1.5; 4]; 4];
        let b4 = vec![vec![2.5; 4]; 4];
        let c4 = matrix_multiply(&a4, &b4);
        // Each element: 1.5 × 2.5 × 4 = 15.0
        assert_eq!(c4[0][0], 15.0);
        assert_eq!(c4[3][3], 15.0);
    }
}
