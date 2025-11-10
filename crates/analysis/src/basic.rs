//! Basic statistical functions
//!
//! Core statistical calculations used throughout the analysis.

/// Calculate the mean (average) of a dataset
///
/// Returns None if the dataset is empty.
pub fn mean(data: &[f64]) -> Option<f64> {
    if data.is_empty() {
        return None;
    }
    Some(data.iter().sum::<f64>() / data.len() as f64)
}

/// Calculate the median of a dataset
///
/// Returns None if the dataset is empty.
/// For even-length datasets, returns the average of the two middle values.
pub fn median(data: &[f64]) -> Option<f64> {
    if data.is_empty() {
        return None;
    }

    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        Some((sorted[mid - 1] + sorted[mid]) / 2.0)
    } else {
        Some(sorted[mid])
    }
}

/// Calculate the variance of a dataset
///
/// Uses the sample variance formula (dividing by n-1).
/// Returns None if the dataset has fewer than 2 elements.
pub fn variance(data: &[f64]) -> Option<f64> {
    if data.len() < 2 {
        return None;
    }

    let m = mean(data)?;
    let sum_sq_diff = data.iter().map(|x| (x - m).powi(2)).sum::<f64>();
    Some(sum_sq_diff / (data.len() - 1) as f64)
}

/// Calculate the standard deviation of a dataset
///
/// Returns the square root of the sample variance.
/// Returns None if the dataset has fewer than 2 elements.
pub fn std_dev(data: &[f64]) -> Option<f64> {
    variance(data).map(|v| v.sqrt())
}

/// Calculate the standard error of the mean
///
/// SE = stddev / sqrt(n)
/// Returns None if the dataset has fewer than 2 elements.
pub fn std_error(data: &[f64]) -> Option<f64> {
    if data.len() < 2 {
        return None;
    }
    let sd = std_dev(data)?;
    Some(sd / (data.len() as f64).sqrt())
}

/// Calculate Pearson correlation coefficient between two datasets
///
/// Returns None if:
/// - Datasets have different lengths
/// - Datasets have fewer than 2 elements
/// - Either dataset has zero variance
pub fn correlation(x: &[f64], y: &[f64]) -> Option<f64> {
    if x.len() != y.len() || x.len() < 2 {
        return None;
    }

    let mean_x = mean(x)?;
    let mean_y = mean(y)?;

    let mut sum_xy = 0.0;
    let mut sum_xx = 0.0;
    let mut sum_yy = 0.0;

    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        sum_xy += dx * dy;
        sum_xx += dx * dx;
        sum_yy += dy * dy;
    }

    if sum_xx == 0.0 || sum_yy == 0.0 {
        return None; // Zero variance
    }

    Some(sum_xy / (sum_xx * sum_yy).sqrt())
}

/// Calculate the coefficient of variation (CV)
///
/// CV = (stddev / mean) * 100
/// Returns None if mean is zero or dataset has fewer than 2 elements.
pub fn coefficient_of_variation(data: &[f64]) -> Option<f64> {
    let m = mean(data)?;
    if m == 0.0 {
        return None;
    }
    let sd = std_dev(data)?;
    Some((sd / m) * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean_empty() {
        assert_eq!(mean(&[]), None);
    }

    #[test]
    fn test_mean_single() {
        assert_eq!(mean(&[5.0]), Some(5.0));
    }

    #[test]
    fn test_mean_multiple() {
        assert_eq!(mean(&[1.0, 2.0, 3.0, 4.0, 5.0]), Some(3.0));
    }

    #[test]
    fn test_mean_negative() {
        assert_eq!(mean(&[-2.0, -1.0, 0.0, 1.0, 2.0]), Some(0.0));
    }

    #[test]
    fn test_median_empty() {
        assert_eq!(median(&[]), None);
    }

    #[test]
    fn test_median_single() {
        assert_eq!(median(&[5.0]), Some(5.0));
    }

    #[test]
    fn test_median_odd() {
        assert_eq!(median(&[1.0, 3.0, 5.0]), Some(3.0));
    }

    #[test]
    fn test_median_even() {
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]), Some(2.5));
    }

    #[test]
    fn test_median_unsorted() {
        assert_eq!(median(&[5.0, 1.0, 3.0]), Some(3.0));
    }

    #[test]
    fn test_variance_empty() {
        assert_eq!(variance(&[]), None);
    }

    #[test]
    fn test_variance_single() {
        assert_eq!(variance(&[5.0]), None); // Need at least 2 for sample variance
    }

    #[test]
    fn test_variance_identical() {
        assert_eq!(variance(&[5.0, 5.0, 5.0]), Some(0.0));
    }

    #[test]
    fn test_variance_simple() {
        // Variance of [1, 2, 3] = ((1-2)^2 + (2-2)^2 + (3-2)^2) / 2 = 2/2 = 1
        let result = variance(&[1.0, 2.0, 3.0]).unwrap();
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_std_dev_simple() {
        let result = std_dev(&[1.0, 2.0, 3.0]).unwrap();
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_std_error() {
        // stddev([1,2,3]) = 1.0, n=3, SE = 1.0 / sqrt(3) ≈ 0.577
        let result = std_error(&[1.0, 2.0, 3.0]).unwrap();
        let expected = 1.0 / 3.0_f64.sqrt();
        assert!((result - expected).abs() < 1e-10);
    }

    #[test]
    fn test_correlation_perfect_positive() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let result = correlation(&x, &y).unwrap();
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_correlation_perfect_negative() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![10.0, 8.0, 6.0, 4.0, 2.0];
        let result = correlation(&x, &y).unwrap();
        assert!((result + 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_correlation_zero() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![1.0, 1.0, 1.0, 1.0, 1.0]; // No variance in y
        assert_eq!(correlation(&x, &y), None);
    }

    #[test]
    fn test_correlation_different_lengths() {
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![1.0, 2.0];
        assert_eq!(correlation(&x, &y), None);
    }

    #[test]
    fn test_coefficient_of_variation() {
        // CV of [10, 11, 12] = (stddev/mean) * 100
        // mean = 11, stddev = 1
        // CV = (1/11) * 100 ≈ 9.09%
        let result = coefficient_of_variation(&[10.0, 11.0, 12.0]).unwrap();
        let expected = (1.0 / 11.0) * 100.0;
        assert!((result - expected).abs() < 0.01);
    }

    #[test]
    fn test_coefficient_of_variation_zero_mean() {
        assert_eq!(coefficient_of_variation(&[-1.0, 0.0, 1.0]), None);
    }
}
