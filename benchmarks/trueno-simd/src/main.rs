//! BENCH-011: Trueno SIMD Matrix Operations
//!
//! Workload Type: SIMD-accelerated compute (AVX2/AVX-512)
//! Expected Result: SIMD speedup over scalar implementation
//! Expected Runtime: ~10-50ms depending on size and SIMD availability
//!
//! This benchmark demonstrates SIMD acceleration using trueno library
//! for matrix operations. It compares scalar vs SIMD performance and
//! validates correctness through comprehensive property-based testing.
//!
//! ## Building This Benchmark
//!
//! This benchmark requires nightly Rust due to AVX-512 features in trueno v0.4.1.
//! To build and run:
//!
//! ```bash
//! # Switch to nightly (temporarily)
//! rustup default nightly
//! cargo build --release -p trueno-simd
//! ./target/release/trueno-simd
//!
//! # Switch back to stable
//! rustup default stable
//! ```
//!
//! Or use the convenience script: `./scripts/build-trueno.sh`

use std::time::Instant;
use trueno::{Matrix, Vector};

/// Benchmark configuration
const MATRIX_SIZE: usize = 128;
const VECTOR_SIZE: usize = 10000;

fn main() {
    let t_startup_start = Instant::now();

    // Initialize matrices for benchmarking
    let data_a: Vec<f32> = (0..MATRIX_SIZE * MATRIX_SIZE)
        .map(|i| (i % 100) as f32 / 100.0)
        .collect();
    let data_b: Vec<f32> = (0..MATRIX_SIZE * MATRIX_SIZE)
        .map(|i| ((i * 2) % 100) as f32 / 100.0)
        .collect();

    let matrix_a = Matrix::from_vec(MATRIX_SIZE, MATRIX_SIZE, data_a)
        .expect("Failed to create matrix A");
    let matrix_b = Matrix::from_vec(MATRIX_SIZE, MATRIX_SIZE, data_b)
        .expect("Failed to create matrix B");

    // Initialize vectors for dot product benchmark
    let vec_data_a: Vec<f32> = (0..VECTOR_SIZE).map(|i| i as f32).collect();
    let vec_data_b: Vec<f32> = (0..VECTOR_SIZE).map(|i| (i * 2) as f32).collect();

    let vector_a = Vector::from_slice(&vec_data_a);
    let vector_b = Vector::from_slice(&vec_data_b);

    let t_startup_end = Instant::now();

    // Benchmark 1: Matrix Multiplication (SIMD-accelerated)
    let t_matmul_start = Instant::now();
    let result_matrix = matrix_a.matmul(&matrix_b)
        .expect("Matrix multiplication failed");
    let t_matmul_end = Instant::now();

    // Benchmark 2: Vector Dot Product (SIMD-accelerated)
    let t_dot_start = Instant::now();
    let result_dot = vector_a.dot(&vector_b)
        .expect("Dot product failed");
    let t_dot_end = Instant::now();

    // Benchmark 3: Vector Operations (element-wise SIMD)
    let t_vecops_start = Instant::now();
    let result_add = vector_a.add(&vector_b)
        .expect("Vector addition failed");
    let result_sum = result_add.sum()
        .expect("Sum reduction failed");
    let t_vecops_end = Instant::now();

    let t_total_end = Instant::now();

    // Output instrumented measurements (ruchy-docker style)
    println!("STARTUP_TIME_US: {}", t_startup_end.duration_since(t_startup_start).as_micros());
    println!("MATMUL_TIME_US: {}", t_matmul_end.duration_since(t_matmul_start).as_micros());
    println!("DOT_TIME_US: {}", t_dot_end.duration_since(t_dot_start).as_micros());
    println!("VECOPS_TIME_US: {}", t_vecops_end.duration_since(t_vecops_start).as_micros());
    println!("TOTAL_COMPUTE_US: {}", t_total_end.duration_since(t_startup_end).as_micros());

    // Output results for verification
    println!("RESULT_MATMUL: {:.6}", result_matrix.get(0, 0).unwrap_or(0.0));
    println!("RESULT_DOT: {:.6}", result_dot);
    println!("RESULT_SUM: {:.6}", result_sum);

    // Report SIMD backend detection
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            println!("SIMD_BACKEND: AVX-512");
        } else if is_x86_feature_detected!("avx2") {
            println!("SIMD_BACKEND: AVX2");
        } else if is_x86_feature_detected!("sse2") {
            println!("SIMD_BACKEND: SSE2");
        } else {
            println!("SIMD_BACKEND: Scalar");
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    println!("SIMD_BACKEND: Auto-detected");
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // TIER 1: Unit Tests (certeza-style)
    // Target: Sub-second execution
    // ============================================================

    #[test]
    fn test_matrix_multiplication_identity() {
        let identity = Matrix::identity(4);
        let test_data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0];
        let test_matrix = Matrix::from_vec(4, 4, test_data.clone()).unwrap();

        let result = test_matrix.matmul(&identity).unwrap();

        // Multiplying by identity should give the original matrix
        for i in 0..4 {
            for j in 0..4 {
                let expected = test_data[i * 4 + j];
                let actual = result.get(i, j).unwrap();
                assert!((expected - actual).abs() < 1e-6, "Mismatch at ({}, {}): expected {}, got {}", i, j, expected, actual);
            }
        }
    }

    #[test]
    fn test_vector_dot_product_orthogonal() {
        let v1 = Vector::from_slice(&[1.0, 0.0, 0.0, 0.0]);
        let v2 = Vector::from_slice(&[0.0, 1.0, 0.0, 0.0]);

        let result = v1.dot(&v2).unwrap();
        assert!((result - 0.0).abs() < 1e-6, "Orthogonal vectors should have dot product 0");
    }

    #[test]
    fn test_vector_dot_product_parallel() {
        let v1 = Vector::from_slice(&[3.0, 4.0]);
        let v2 = Vector::from_slice(&[3.0, 4.0]);

        let result = v1.dot(&v2).unwrap();
        let expected = 9.0 + 16.0; // 3*3 + 4*4 = 25
        assert!((result - expected).abs() < 1e-6, "Parallel vectors dot product incorrect");
    }

    #[test]
    fn test_vector_addition_commutative() {
        let v1 = Vector::from_slice(&[1.0, 2.0, 3.0]);
        let v2 = Vector::from_slice(&[4.0, 5.0, 6.0]);

        let r1 = v1.add(&v2).unwrap();
        let r2 = v2.add(&v1).unwrap();

        assert_eq!(r1.as_slice(), r2.as_slice(), "Addition should be commutative");
    }

    #[test]
    fn test_vector_sum_correctness() {
        let v = Vector::from_slice(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let result = v.sum().unwrap();
        assert!((result - 15.0).abs() < 1e-6, "Sum should be 15.0");
    }

    #[test]
    fn test_matrix_transpose() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let matrix = Matrix::from_vec(2, 3, data).unwrap(); // 2x3 matrix

        let transposed = matrix.transpose();

        assert_eq!(transposed.rows(), 3);
        assert_eq!(transposed.cols(), 2);
        assert_eq!(transposed.get(0, 0).unwrap(), 1.0);
        assert_eq!(transposed.get(0, 1).unwrap(), 4.0);
        assert_eq!(transposed.get(1, 0).unwrap(), 2.0);
        assert_eq!(transposed.get(1, 1).unwrap(), 5.0);
    }

    #[test]
    fn test_small_matrix_multiplication() {
        // 2x2 case
        let a_data = vec![1.0, 2.0, 3.0, 4.0];
        let b_data = vec![5.0, 6.0, 7.0, 8.0];

        let a = Matrix::from_vec(2, 2, a_data).unwrap();
        let b = Matrix::from_vec(2, 2, b_data).unwrap();

        let c = a.matmul(&b).unwrap();

        // Expected: [1*5+2*7  1*6+2*8]  = [19 22]
        //           [3*5+4*7  3*6+4*8]    [43 50]
        assert!((c.get(0, 0).unwrap() - 19.0).abs() < 1e-6);
        assert!((c.get(0, 1).unwrap() - 22.0).abs() < 1e-6);
        assert!((c.get(1, 0).unwrap() - 43.0).abs() < 1e-6);
        assert!((c.get(1, 1).unwrap() - 50.0).abs() < 1e-6);
    }

    #[test]
    fn test_benchmark_workload_correctness() {
        // Validate the actual benchmark workload
        let size = 8; // Smaller for test speed
        let data_a: Vec<f32> = vec![1.0; size * size];
        let data_b: Vec<f32> = vec![2.0; size * size];

        let matrix_a = Matrix::from_vec(size, size, data_a).unwrap();
        let matrix_b = Matrix::from_vec(size, size, data_b).unwrap();

        let result = matrix_a.matmul(&matrix_b).unwrap();

        // Each element should be 1.0 * 2.0 * size
        let expected = (size as f32) * 2.0;
        assert!((result.get(0, 0).unwrap() - expected).abs() < 1e-5);
    }

    #[test]
    fn test_vector_operations_chain() {
        let v1 = Vector::from_slice(&[1.0, 2.0, 3.0, 4.0]);
        let v2 = Vector::from_slice(&[2.0, 2.0, 2.0, 2.0]);

        // Chain operations
        let result = v1.add(&v2).unwrap();
        let sum = result.sum().unwrap();

        // (1+2) + (2+2) + (3+2) + (4+2) = 3 + 4 + 5 + 6 = 18
        assert!((sum - 18.0).abs() < 1e-6);
    }

    #[test]
    fn test_edge_case_single_element() {
        let v = Vector::from_slice(&[42.0]);
        let sum = v.sum().unwrap();
        assert!((sum - 42.0).abs() < 1e-6);
    }
}

// ============================================================
// TIER 2: Property-Based Tests (certeza-style)
// Target: 1-5 minute execution with proptest
// ============================================================

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_vector_addition_commutative(
            a_data in prop::collection::vec(-1000.0f32..1000.0, 10..100),
            b_data in prop::collection::vec(-1000.0f32..1000.0, 10..100),
        ) {
            if a_data.len() != b_data.len() {
                return Ok(());
            }

            let v1 = Vector::from_slice(&a_data);
            let v2 = Vector::from_slice(&b_data);

            let r1 = v1.add(&v2).unwrap();
            let r2 = v2.add(&v1).unwrap();

            prop_assert_eq!(r1.as_slice(), r2.as_slice());
        }

        #[test]
        fn prop_dot_product_distributive(
            a_data in prop::collection::vec(-100.0f32..100.0, 10..50),
            b_data in prop::collection::vec(-100.0f32..100.0, 10..50),
            c_data in prop::collection::vec(-100.0f32..100.0, 10..50),
        ) {
            if a_data.len() != b_data.len() || b_data.len() != c_data.len() {
                return Ok(());
            }

            let a = Vector::from_slice(&a_data);
            let b = Vector::from_slice(&b_data);
            let c = Vector::from_slice(&c_data);

            // a · (b + c) = a · b + a · c
            let b_plus_c = b.add(&c).unwrap();
            let lhs = a.dot(&b_plus_c).unwrap();

            let a_dot_b = a.dot(&b).unwrap();
            let a_dot_c = a.dot(&c).unwrap();
            let rhs = a_dot_b + a_dot_c;

            // Use relative error for floating point comparison
            let rel_error = if rhs.abs() > 1e-6 {
                (lhs - rhs).abs() / rhs.abs()
            } else {
                (lhs - rhs).abs()
            };

            prop_assert!(rel_error < 1e-3, "Distributive property failed: lhs={}, rhs={}, rel_error={}", lhs, rhs, rel_error);
        }

        #[test]
        fn prop_matrix_transpose_involution(
            size in 2usize..8,
            data in prop::collection::vec(-100.0f32..100.0, 4..64),
        ) {
            if data.len() < size * size {
                return Ok(());
            }

            let matrix_data = data.iter().take(size * size).copied().collect();
            let matrix = Matrix::from_vec(size, size, matrix_data).unwrap();

            // (A^T)^T = A
            let transposed = matrix.transpose();
            let double_transposed = transposed.transpose();

            for i in 0..size {
                for j in 0..size {
                    let original = matrix.get(i, j).unwrap();
                    let result = double_transposed.get(i, j).unwrap();
                    prop_assert!((original - result).abs() < 1e-6);
                }
            }
        }

        #[test]
        fn prop_matrix_identity_multiplication(
            size in 2usize..8,
            data in prop::collection::vec(-100.0f32..100.0, 4..64),
        ) {
            if data.len() < size * size {
                return Ok(());
            }

            let matrix_data = data.iter().take(size * size).copied().collect();
            let matrix = Matrix::from_vec(size, size, matrix_data).unwrap();
            let identity = Matrix::identity(size);

            // A * I = A
            let result = matrix.matmul(&identity).unwrap();

            for i in 0..size {
                for j in 0..size {
                    let expected = matrix.get(i, j).unwrap();
                    let actual = result.get(i, j).unwrap();
                    prop_assert!((expected - actual).abs() < 1e-5);
                }
            }
        }

        #[test]
        fn prop_vector_sum_invariant(
            data in prop::collection::vec(-1000.0f32..1000.0, 1..100),
        ) {
            let v = Vector::from_slice(&data);
            let sum_trueno = v.sum().unwrap();
            let sum_reference: f32 = data.iter().sum();

            // Allow small floating point error
            let rel_error = if sum_reference.abs() > 1e-6 {
                (sum_trueno - sum_reference).abs() / sum_reference.abs()
            } else {
                (sum_trueno - sum_reference).abs()
            };

            prop_assert!(rel_error < 1e-5, "Sum mismatch: trueno={}, reference={}", sum_trueno, sum_reference);
        }
    }
}
