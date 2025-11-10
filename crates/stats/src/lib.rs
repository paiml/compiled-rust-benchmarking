//! Statistical analysis for benchmark results
//!
//! This crate provides statistical functions for analyzing benchmark
//! performance data, including outlier detection, confidence intervals,
//! and hypothesis testing.

/// Stats crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Calculate the mean of a dataset
pub fn mean(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    data.iter().sum::<f64>() / data.len() as f64
}

/// Calculate the median of a dataset
pub fn median(data: &[f64]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    } else {
        sorted[mid]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_is_set() {
        assert_eq!(VERSION, "0.1.0");
    }

    #[test]
    fn test_mean_empty() {
        assert_eq!(mean(&[]), 0.0);
    }

    #[test]
    fn test_mean_single() {
        assert_eq!(mean(&[5.0]), 5.0);
    }

    #[test]
    fn test_mean_multiple() {
        assert_eq!(mean(&[1.0, 2.0, 3.0, 4.0, 5.0]), 3.0);
    }

    #[test]
    fn test_median_empty() {
        assert_eq!(median(&[]), 0.0);
    }

    #[test]
    fn test_median_single() {
        assert_eq!(median(&[5.0]), 5.0);
    }

    #[test]
    fn test_median_odd_length() {
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0, 5.0]), 3.0);
    }

    #[test]
    fn test_median_even_length() {
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]), 2.5);
    }
}
