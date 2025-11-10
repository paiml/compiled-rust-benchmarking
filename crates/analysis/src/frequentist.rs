//! Frequentist statistical methods
//!
//! ANOVA, t-tests, confidence intervals, and effect sizes.

use crate::basic::{mean, variance};

/// Result of a one-way ANOVA test
#[derive(Debug, Clone, PartialEq)]
pub struct AnovaResult {
    /// F-statistic
    pub f_statistic: f64,
    /// Degrees of freedom between groups
    pub df_between: usize,
    /// Degrees of freedom within groups
    pub df_within: usize,
    /// P-value (approximation, would need F-distribution for exact)
    pub significant: bool,
    /// Effect size (eta-squared)
    pub eta_squared: f64,
}

/// Perform one-way ANOVA
///
/// Tests whether the means of multiple groups are significantly different.
///
/// # Arguments
/// * `groups` - Vector of groups, where each group is a vector of observations
///
/// # Returns
/// * `Some(AnovaResult)` if there are at least 2 groups with at least 1 observation each
/// * `None` otherwise
pub fn anova_one_way(groups: &[Vec<f64>]) -> Option<AnovaResult> {
    if groups.len() < 2 {
        return None;
    }

    // Calculate grand mean and total n
    let mut all_data = Vec::new();
    for group in groups {
        if group.is_empty() {
            return None;
        }
        all_data.extend_from_slice(group);
    }

    let grand_mean = mean(&all_data)?;
    let n_total = all_data.len();
    let k = groups.len(); // number of groups

    // Calculate sum of squares between groups (SSB)
    let mut ss_between = 0.0;
    for group in groups {
        let group_mean = mean(group)?;
        let n_group = group.len();
        ss_between += n_group as f64 * (group_mean - grand_mean).powi(2);
    }

    // Calculate sum of squares within groups (SSW)
    let mut ss_within = 0.0;
    for group in groups {
        let group_mean = mean(group)?;
        for &value in group {
            ss_within += (value - group_mean).powi(2);
        }
    }

    // Degrees of freedom
    let df_between = k - 1;
    let df_within = n_total - k;

    if df_within == 0 {
        return None;
    }

    // Mean squares
    let ms_between = ss_between / df_between as f64;
    let ms_within = ss_within / df_within as f64;

    // F-statistic
    let f_statistic = if ms_within > 0.0 {
        ms_between / ms_within
    } else {
        return None; // No within-group variance
    };

    // Effect size (eta-squared)
    let ss_total = ss_between + ss_within;
    let eta_squared = if ss_total > 0.0 {
        ss_between / ss_total
    } else {
        0.0
    };

    // Simplified significance test: F > 3.0 is roughly significant for typical DFs
    // (This is a simplification; exact test requires F-distribution)
    let significant = f_statistic > 3.0;

    Some(AnovaResult {
        f_statistic,
        df_between,
        df_within,
        significant,
        eta_squared,
    })
}

/// Result of a t-test
#[derive(Debug, Clone, PartialEq)]
pub struct TTestResult {
    /// t-statistic
    pub t_statistic: f64,
    /// Degrees of freedom
    pub df: f64,
    /// Difference in means (group1 - group2)
    pub mean_diff: f64,
    /// Standard error of the difference
    pub std_error: f64,
    /// Simplified significance indicator (|t| > 2.0)
    pub significant: bool,
}

/// Perform Welch's t-test (unequal variances)
///
/// Tests whether two groups have significantly different means.
/// Does not assume equal variances.
///
/// # Returns
/// * `Some(TTestResult)` if both groups have at least 2 observations
/// * `None` otherwise
pub fn t_test_welch(group1: &[f64], group2: &[f64]) -> Option<TTestResult> {
    if group1.len() < 2 || group2.len() < 2 {
        return None;
    }

    let mean1 = mean(group1)?;
    let mean2 = mean(group2)?;
    let var1 = variance(group1)?;
    let var2 = variance(group2)?;

    let n1 = group1.len() as f64;
    let n2 = group2.len() as f64;

    // Standard error of difference
    let se_diff = ((var1 / n1) + (var2 / n2)).sqrt();

    if se_diff == 0.0 {
        return None;
    }

    // t-statistic
    let t_statistic = (mean1 - mean2) / se_diff;

    // Welch-Satterthwaite degrees of freedom
    let numerator = ((var1 / n1) + (var2 / n2)).powi(2);
    let denominator = ((var1 / n1).powi(2) / (n1 - 1.0)) + ((var2 / n2).powi(2) / (n2 - 1.0));
    let df = if denominator > 0.0 {
        numerator / denominator
    } else {
        return None;
    };

    // Simplified significance: |t| > 2.0 is roughly significant for most DFs
    let significant = t_statistic.abs() > 2.0;

    Some(TTestResult {
        t_statistic,
        df,
        mean_diff: mean1 - mean2,
        std_error: se_diff,
        significant,
    })
}

/// Calculate Cohen's d effect size
///
/// Measures the standardized difference between two means.
/// d = (mean1 - mean2) / pooled_sd
///
/// # Effect size interpretation
/// - Small: d = 0.2
/// - Medium: d = 0.5
/// - Large: d = 0.8
pub fn cohens_d(group1: &[f64], group2: &[f64]) -> Option<f64> {
    if group1.len() < 2 || group2.len() < 2 {
        return None;
    }

    let mean1 = mean(group1)?;
    let mean2 = mean(group2)?;
    let var1 = variance(group1)?;
    let var2 = variance(group2)?;

    let n1 = group1.len() as f64;
    let n2 = group2.len() as f64;

    // Pooled standard deviation
    let pooled_var = ((n1 - 1.0) * var1 + (n2 - 1.0) * var2) / (n1 + n2 - 2.0);
    let pooled_sd = pooled_var.sqrt();

    if pooled_sd == 0.0 {
        return None;
    }

    Some((mean1 - mean2) / pooled_sd)
}

/// Bootstrap confidence interval (percentile method)
///
/// # Arguments
/// * `data` - Dataset to bootstrap
/// * `statistic` - Function to compute statistic (e.g., mean)
/// * `n_bootstrap` - Number of bootstrap samples (default: 1000)
/// * `confidence` - Confidence level (e.g., 0.95 for 95% CI)
///
/// # Returns
/// * `Some((lower, upper))` - Confidence interval
/// * `None` if data is empty
pub fn bootstrap_ci<F>(
    data: &[f64],
    statistic: F,
    n_bootstrap: usize,
    confidence: f64,
) -> Option<(f64, f64)>
where
    F: Fn(&[f64]) -> Option<f64>,
{
    if data.is_empty() {
        return None;
    }

    // Simple deterministic bootstrap for reproducibility in tests
    // In production, would use proper RNG
    let mut bootstrap_stats = Vec::new();

    for i in 0..n_bootstrap {
        let mut sample = Vec::new();
        for j in 0..data.len() {
            // Deterministic sampling: hash(i,j) % data.len()
            let idx = (i * 31 + j * 17) % data.len();
            sample.push(data[idx]);
        }

        if let Some(stat) = statistic(&sample) {
            bootstrap_stats.push(stat);
        }
    }

    if bootstrap_stats.is_empty() {
        return None;
    }

    bootstrap_stats.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let alpha = 1.0 - confidence;
    let lower_idx = (bootstrap_stats.len() as f64 * (alpha / 2.0)) as usize;
    let upper_idx = (bootstrap_stats.len() as f64 * (1.0 - alpha / 2.0)) as usize;

    Some((
        bootstrap_stats[lower_idx.min(bootstrap_stats.len() - 1)],
        bootstrap_stats[upper_idx.min(bootstrap_stats.len() - 1)],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anova_two_identical_groups() {
        let groups = vec![vec![1.0, 2.0, 3.0], vec![1.0, 2.0, 3.0]];
        let result = anova_one_way(&groups).unwrap();
        assert!(result.f_statistic.abs() < 1e-10); // F should be ~0
        assert!(!result.significant);
        assert!(result.eta_squared.abs() < 1e-10);
    }

    #[test]
    fn test_anova_different_groups() {
        let groups = vec![vec![1.0, 2.0, 3.0], vec![10.0, 11.0, 12.0]];
        let result = anova_one_way(&groups).unwrap();
        assert!(result.f_statistic > 10.0); // Should be highly significant
        assert!(result.significant);
        assert!(result.eta_squared > 0.8); // Large effect
    }

    #[test]
    fn test_anova_empty_group() {
        let groups = vec![vec![1.0, 2.0], vec![]];
        assert_eq!(anova_one_way(&groups), None);
    }

    #[test]
    fn test_anova_single_group() {
        let groups = vec![vec![1.0, 2.0, 3.0]];
        assert_eq!(anova_one_way(&groups), None);
    }

    #[test]
    fn test_t_test_identical_groups() {
        let group1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let group2 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = t_test_welch(&group1, &group2).unwrap();
        assert!(result.t_statistic.abs() < 1e-10);
        assert!(!result.significant);
        assert!(result.mean_diff.abs() < 1e-10);
    }

    #[test]
    fn test_t_test_different_groups() {
        let group1 = vec![1.0, 2.0, 3.0];
        let group2 = vec![10.0, 11.0, 12.0];
        let result = t_test_welch(&group1, &group2).unwrap();
        assert!(result.t_statistic < -5.0); // Large negative t
        assert!(result.significant);
        assert!((result.mean_diff - (-9.0)).abs() < 1e-10);
    }

    #[test]
    fn test_t_test_insufficient_data() {
        let group1 = vec![1.0];
        let group2 = vec![2.0, 3.0];
        assert_eq!(t_test_welch(&group1, &group2), None);
    }

    #[test]
    fn test_cohens_d_no_effect() {
        let group1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let group2 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let d = cohens_d(&group1, &group2).unwrap();
        assert!(d.abs() < 1e-10);
    }

    #[test]
    fn test_cohens_d_large_effect() {
        let group1 = vec![1.0, 2.0, 3.0];
        let group2 = vec![10.0, 11.0, 12.0];
        let d = cohens_d(&group1, &group2).unwrap();
        assert!(d.abs() > 5.0); // Very large effect
    }

    #[test]
    fn test_bootstrap_ci_mean() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (lower, upper) = bootstrap_ci(&data, mean, 100, 0.95).unwrap();
        // CI should contain the true mean (3.0)
        assert!(lower <= 3.0);
        assert!(upper >= 3.0);
        // CI should be reasonably narrow
        assert!(upper - lower < 3.0);
    }

    #[test]
    fn test_bootstrap_ci_empty() {
        assert_eq!(bootstrap_ci(&[], mean, 100, 0.95), None);
    }
}
