//! Pathfinder Study: Early hypothesis validation with configuration subset
//!
//! This module implements the Pathfinder Study phase, which selects a strategic
//! subset of configurations for early validation before running the full matrix.

use crate::config::OptimizationConfig;
use serde::{Deserialize, Serialize};

/// Strategy for selecting pathfinder configurations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathfinderStrategy {
    /// Select baseline and extreme configurations
    BaselineAndExtremes,
    /// Select configurations covering each factor variation
    SingleFactorCoverage,
    /// Select a balanced mix of configurations
    Balanced,
}

/// Pathfinder configuration selector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathfinderSelector {
    /// Selection strategy
    strategy: PathfinderStrategy,
    /// Maximum number of configurations to select
    max_configs: usize,
}

impl PathfinderSelector {
    /// Create a new selector with specified strategy
    pub fn new(strategy: PathfinderStrategy, max_configs: usize) -> Self {
        Self {
            strategy,
            max_configs: max_configs.max(1),
        }
    }

    /// Create a balanced selector (default)
    pub fn balanced(max_configs: usize) -> Self {
        Self::new(PathfinderStrategy::Balanced, max_configs)
    }

    /// Select pathfinder configurations from the full set
    pub fn select<'a>(&self, configs: &'a [OptimizationConfig]) -> Vec<&'a OptimizationConfig> {
        match self.strategy {
            PathfinderStrategy::BaselineAndExtremes => self.select_baseline_and_extremes(configs),
            PathfinderStrategy::SingleFactorCoverage => self.select_single_factor_coverage(configs),
            PathfinderStrategy::Balanced => self.select_balanced(configs),
        }
    }

    /// Select baseline and extreme configurations
    fn select_baseline_and_extremes<'a>(
        &self,
        configs: &'a [OptimizationConfig],
    ) -> Vec<&'a OptimizationConfig> {
        let mut selected = Vec::new();

        // Always include baseline and standard-release
        for config in configs {
            if config.id == "baseline" || config.id == "standard-release" {
                selected.push(config);
            }
        }

        // Add extreme configurations
        for config in configs {
            if config.id.starts_with("extreme-") || config.id.contains("max-") {
                selected.push(config);
                if selected.len() >= self.max_configs {
                    break;
                }
            }
        }

        selected.truncate(self.max_configs);
        selected
    }

    /// Select configurations covering single-factor variations
    fn select_single_factor_coverage<'a>(
        &self,
        configs: &'a [OptimizationConfig],
    ) -> Vec<&'a OptimizationConfig> {
        let mut selected = Vec::new();

        // Baseline and standard release
        for config in configs {
            if config.id == "baseline" || config.id == "standard-release" {
                selected.push(config);
            }
        }

        // Single-factor variations (one change from baseline at a time)
        for config in configs {
            if config.id.starts_with("opt-")
                || config.id.starts_with("lto-")
                || config.id.starts_with("codegen-")
                || config.id.starts_with("pgo-")
                || config.id.starts_with("cpu-")
                || config.id.starts_with("strip-")
            {
                selected.push(config);
                if selected.len() >= self.max_configs {
                    break;
                }
            }
        }

        selected.truncate(self.max_configs);
        selected
    }

    /// Select a balanced mix of configurations
    fn select_balanced<'a>(
        &self,
        configs: &'a [OptimizationConfig],
    ) -> Vec<&'a OptimizationConfig> {
        let mut selected = Vec::new();

        // Priority 1: Baseline and standard release (2 configs)
        for config in configs {
            if config.id == "baseline" || config.id == "standard-release" {
                selected.push(config);
            }
        }

        // Priority 2: Key single-factor variations (up to 6 configs)
        let single_factor_keywords = [
            "opt-3",      // Optimization level
            "lto-thin",   // Thin LTO
            "lto-fat",    // Fat LTO
            "codegen-1",  // Single codegen unit
            "cpu-native", // Native CPU
            "pgo-on",     // PGO enabled
        ];

        for keyword in &single_factor_keywords {
            if selected.len() >= self.max_configs {
                break;
            }
            for config in configs {
                if config.id.contains(keyword) {
                    selected.push(config);
                    break;
                }
            }
        }

        // Priority 3: Size optimizations (up to 2 configs)
        for config in configs {
            if selected.len() >= self.max_configs {
                break;
            }
            if config.id.contains("size-") || config.id.starts_with("opt-s") {
                selected.push(config);
            }
        }

        // Priority 4: Performance optimizations (up to 2 configs)
        for config in configs {
            if selected.len() >= self.max_configs {
                break;
            }
            if config.id.contains("perf-") || config.id.contains("max-") {
                selected.push(config);
            }
        }

        // Priority 5: Extreme cases (fill remaining slots)
        for config in configs {
            if selected.len() >= self.max_configs {
                break;
            }
            if config.id.starts_with("extreme-") {
                selected.push(config);
            }
        }

        selected.truncate(self.max_configs);
        selected
    }

    /// Get the maximum number of configurations
    pub fn max_configs(&self) -> usize {
        self.max_configs
    }

    /// Get the selection strategy
    pub fn strategy(&self) -> PathfinderStrategy {
        self.strategy
    }
}

impl Default for PathfinderSelector {
    fn default() -> Self {
        Self::balanced(15)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CodegenUnits, LtoSetting, OptLevel, PgoSetting, StripSetting, TargetCpu};

    fn create_test_configs() -> Vec<OptimizationConfig> {
        vec![
            OptimizationConfig::new(
                "baseline".to_string(),
                OptLevel::O0,
                LtoSetting::Off,
                CodegenUnits::Sixteen,
                PgoSetting::Off,
                TargetCpu::Generic,
                StripSetting::None,
            ),
            OptimizationConfig::new(
                "standard-release".to_string(),
                OptLevel::O3,
                LtoSetting::Off,
                CodegenUnits::Sixteen,
                PgoSetting::Off,
                TargetCpu::Generic,
                StripSetting::None,
            ),
            OptimizationConfig::new(
                "opt-3".to_string(),
                OptLevel::O3,
                LtoSetting::Off,
                CodegenUnits::Sixteen,
                PgoSetting::Off,
                TargetCpu::Generic,
                StripSetting::None,
            ),
            OptimizationConfig::new(
                "lto-thin".to_string(),
                OptLevel::O3,
                LtoSetting::Thin,
                CodegenUnits::Sixteen,
                PgoSetting::Off,
                TargetCpu::Generic,
                StripSetting::None,
            ),
            OptimizationConfig::new(
                "lto-fat".to_string(),
                OptLevel::O3,
                LtoSetting::Fat,
                CodegenUnits::One,
                PgoSetting::Off,
                TargetCpu::Generic,
                StripSetting::None,
            ),
            OptimizationConfig::new(
                "extreme-speed".to_string(),
                OptLevel::O3,
                LtoSetting::Fat,
                CodegenUnits::One,
                PgoSetting::On,
                TargetCpu::Native,
                StripSetting::Symbols,
            ),
        ]
    }

    #[test]
    fn test_pathfinder_strategy_variants() {
        let base = PathfinderStrategy::BaselineAndExtremes;
        let single = PathfinderStrategy::SingleFactorCoverage;
        let balanced = PathfinderStrategy::Balanced;

        assert!(matches!(base, PathfinderStrategy::BaselineAndExtremes));
        assert!(matches!(single, PathfinderStrategy::SingleFactorCoverage));
        assert!(matches!(balanced, PathfinderStrategy::Balanced));
    }

    #[test]
    fn test_pathfinder_selector_new() {
        let selector = PathfinderSelector::new(PathfinderStrategy::Balanced, 10);

        assert_eq!(selector.strategy(), PathfinderStrategy::Balanced);
        assert_eq!(selector.max_configs(), 10);
    }

    #[test]
    fn test_pathfinder_selector_balanced() {
        let selector = PathfinderSelector::balanced(15);

        assert_eq!(selector.strategy(), PathfinderStrategy::Balanced);
        assert_eq!(selector.max_configs(), 15);
    }

    #[test]
    fn test_pathfinder_selector_default() {
        let selector = PathfinderSelector::default();

        assert_eq!(selector.strategy(), PathfinderStrategy::Balanced);
        assert_eq!(selector.max_configs(), 15);
    }

    #[test]
    fn test_pathfinder_selector_zero_max() {
        let selector = PathfinderSelector::new(PathfinderStrategy::Balanced, 0);

        assert_eq!(selector.max_configs(), 1); // Minimum is 1
    }

    #[test]
    fn test_select_baseline_and_extremes() {
        let configs = create_test_configs();
        let selector = PathfinderSelector::new(PathfinderStrategy::BaselineAndExtremes, 10);

        let selected = selector.select(&configs);

        // Should include baseline, standard-release, and extreme-speed
        assert!(selected.iter().any(|c| c.id == "baseline"));
        assert!(selected.iter().any(|c| c.id == "standard-release"));
        assert!(selected.iter().any(|c| c.id == "extreme-speed"));
    }

    #[test]
    fn test_select_single_factor_coverage() {
        let configs = create_test_configs();
        let selector = PathfinderSelector::new(PathfinderStrategy::SingleFactorCoverage, 10);

        let selected = selector.select(&configs);

        // Should include baseline and standard-release
        assert!(selected.iter().any(|c| c.id == "baseline"));
        assert!(selected.iter().any(|c| c.id == "standard-release"));

        // Should include single-factor variations
        assert!(selected.iter().any(|c| c.id.starts_with("opt-")));
        assert!(selected.iter().any(|c| c.id.starts_with("lto-")));
    }

    #[test]
    fn test_select_balanced() {
        let configs = create_test_configs();
        let selector = PathfinderSelector::balanced(10);

        let selected = selector.select(&configs);

        // Should include baseline and standard-release
        assert!(selected.iter().any(|c| c.id == "baseline"));
        assert!(selected.iter().any(|c| c.id == "standard-release"));

        // Should respect max_configs
        assert!(selected.len() <= 10);
    }

    #[test]
    fn test_select_respects_max_configs() {
        let configs = create_test_configs();
        let selector = PathfinderSelector::balanced(3);

        let selected = selector.select(&configs);

        assert!(selected.len() <= 3);
    }

    #[test]
    fn test_select_with_full_config_set() {
        use crate::config::generator::ConfigGenerator;

        let mut generator = ConfigGenerator::new();
        let configs = generator.generate_matrix();

        let selector = PathfinderSelector::balanced(15);
        let selected = selector.select(configs);

        // Should select exactly 15 or fewer
        assert!(selected.len() <= 15);
        assert!(!selected.is_empty());

        // Should include baseline and standard-release
        assert!(selected.iter().any(|c| c.id == "baseline"));
        assert!(selected.iter().any(|c| c.id == "standard-release"));
    }

    #[test]
    fn test_select_all_strategies_return_valid_subset() {
        let configs = create_test_configs();

        let strategies = [
            PathfinderStrategy::BaselineAndExtremes,
            PathfinderStrategy::SingleFactorCoverage,
            PathfinderStrategy::Balanced,
        ];

        for strategy in &strategies {
            let selector = PathfinderSelector::new(*strategy, 10);
            let selected = selector.select(&configs);

            assert!(
                !selected.is_empty(),
                "Strategy {:?} returned empty",
                strategy
            );
            assert!(selected.len() <= 10, "Strategy {:?} exceeded max", strategy);
        }
    }
}
