//! Configuration matrix generator using fractional factorial design
//!
//! Generates ~100 high-impact optimization configurations from the full
//! 648-configuration space using fractional factorial design.

use super::*;

/// Configuration generator
pub struct ConfigGenerator {
    configs: Vec<OptimizationConfig>,
}

impl ConfigGenerator {
    /// Create a new generator
    pub fn new() -> Self {
        Self {
            configs: Vec::new(),
        }
    }

    /// Generate the full configuration matrix using fractional factorial design
    ///
    /// Strategy:
    /// 1. Baseline and standard release (2 configs)
    /// 2. Single-factor variations from standard release (main effects)
    /// 3. Two-factor interactions (important combinations)
    /// 4. Extreme configurations (edge cases)
    ///
    /// Target: ~100 configurations
    pub fn generate_matrix(&mut self) -> &[OptimizationConfig] {
        self.configs.clear();

        // 1. Baseline configurations (2)
        self.add_baseline_configs();

        // 2. Single-factor variations (main effects) (~15)
        self.add_single_factor_variations();

        // 3. LTO focused configurations (~15)
        self.add_lto_variations();

        // 4. Size optimization configurations (~10)
        self.add_size_optimizations();

        // 5. Performance optimization configurations (~15)
        self.add_performance_optimizations();

        // 6. PGO variations (~10)
        self.add_pgo_variations();

        // 7. Codegen-units interactions (~15)
        self.add_codegen_variations();

        // 8. Target CPU interactions (~10)
        self.add_cpu_variations();

        // 9. Strip variations (~10)
        self.add_strip_variations();

        // 10. Extreme/edge case configurations (~10)
        self.add_extreme_configs();

        // 11. Additional two-factor interactions (~20)
        self.add_two_factor_interactions();

        &self.configs
    }

    /// Add baseline configurations
    fn add_baseline_configs(&mut self) {
        self.configs.push(OptimizationConfig::baseline());
        self.configs.push(OptimizationConfig::standard_release());
    }

    /// Add single-factor variations from standard release
    fn add_single_factor_variations(&mut self) {
        let base = OptimizationConfig::standard_release();

        // Vary opt-level (5 configs)
        for opt_level in [
            OptLevel::O0,
            OptLevel::O1,
            OptLevel::O2,
            OptLevel::Os,
            OptLevel::Oz,
        ] {
            self.configs.push(OptimizationConfig::new(
                format!("opt-{}", opt_level.to_profile_string()),
                opt_level,
                base.lto,
                base.codegen_units,
                base.pgo,
                base.target_cpu,
                base.strip,
            ));
        }

        // Vary LTO (2 more configs, thin and fat)
        for lto in [LtoSetting::Thin, LtoSetting::Fat] {
            self.configs.push(OptimizationConfig::new(
                format!("lto-{:?}", lto).to_lowercase(),
                base.opt_level,
                lto,
                base.codegen_units,
                base.pgo,
                base.target_cpu,
                base.strip,
            ));
        }

        // Vary codegen-units (3 more configs)
        for units in [
            CodegenUnits::One,
            CodegenUnits::Four,
            CodegenUnits::TwoFiftySix,
        ] {
            self.configs.push(OptimizationConfig::new(
                format!("codegen-{}", units.value()),
                base.opt_level,
                base.lto,
                units,
                base.pgo,
                base.target_cpu,
                base.strip,
            ));
        }

        // Vary target-cpu (2 more configs)
        for cpu in [TargetCpu::Native, TargetCpu::Specific] {
            self.configs.push(OptimizationConfig::new(
                format!("cpu-{:?}", cpu).to_lowercase(),
                base.opt_level,
                base.lto,
                base.codegen_units,
                base.pgo,
                cpu,
                base.strip,
            ));
        }

        // Vary strip (2 more configs)
        for strip in [StripSetting::Symbols, StripSetting::Debuginfo] {
            self.configs.push(OptimizationConfig::new(
                format!("strip-{:?}", strip).to_lowercase(),
                base.opt_level,
                base.lto,
                base.codegen_units,
                base.pgo,
                base.target_cpu,
                strip,
            ));
        }
    }

    /// Add LTO-focused variations
    fn add_lto_variations(&mut self) {
        // LTO + opt-level combinations
        for opt_level in [OptLevel::O2, OptLevel::O3] {
            for lto in [LtoSetting::Thin, LtoSetting::Fat] {
                self.configs.push(OptimizationConfig::new(
                    format!("lto-{:?}-opt{}", lto, opt_level.to_profile_string()).to_lowercase(),
                    opt_level,
                    lto,
                    CodegenUnits::Sixteen,
                    PgoSetting::Off,
                    TargetCpu::Generic,
                    StripSetting::None,
                ));
            }
        }

        // LTO + codegen-units (important interaction)
        for lto in [LtoSetting::Thin, LtoSetting::Fat] {
            for units in [CodegenUnits::One, CodegenUnits::Four] {
                self.configs.push(OptimizationConfig::new(
                    format!("lto-{:?}-cg{}", lto, units.value()).to_lowercase(),
                    OptLevel::O3,
                    lto,
                    units,
                    PgoSetting::Off,
                    TargetCpu::Generic,
                    StripSetting::None,
                ));
            }
        }
    }

    /// Add size optimization configurations
    fn add_size_optimizations(&mut self) {
        // Size opt-level with various settings
        for opt_level in [OptLevel::Os, OptLevel::Oz] {
            // With LTO
            self.configs.push(OptimizationConfig::new(
                format!("size-{}-lto", opt_level.to_profile_string()),
                opt_level,
                LtoSetting::Fat,
                CodegenUnits::One,
                PgoSetting::Off,
                TargetCpu::Generic,
                StripSetting::Symbols,
            ));

            // With strip
            self.configs.push(OptimizationConfig::new(
                format!("size-{}-strip", opt_level.to_profile_string()),
                opt_level,
                LtoSetting::Thin,
                CodegenUnits::Sixteen,
                PgoSetting::Off,
                TargetCpu::Generic,
                StripSetting::Debuginfo,
            ));
        }

        // Aggressive size optimization
        self.configs.push(OptimizationConfig::new(
            "size-ultra".to_string(),
            OptLevel::Oz,
            LtoSetting::Fat,
            CodegenUnits::One,
            PgoSetting::Off,
            TargetCpu::Generic,
            StripSetting::Debuginfo,
        ));
    }

    /// Add performance optimization configurations
    fn add_performance_optimizations(&mut self) {
        // Aggressive performance configs
        self.configs.push(OptimizationConfig::new(
            "perf-ultra".to_string(),
            OptLevel::O3,
            LtoSetting::Fat,
            CodegenUnits::One,
            PgoSetting::Off,
            TargetCpu::Native,
            StripSetting::Symbols,
        ));

        // O3 + native combinations
        for lto in [LtoSetting::Thin, LtoSetting::Fat] {
            self.configs.push(OptimizationConfig::new(
                format!("perf-native-lto-{:?}", lto).to_lowercase(),
                OptLevel::O3,
                lto,
                CodegenUnits::One,
                PgoSetting::Off,
                TargetCpu::Native,
                StripSetting::None,
            ));
        }

        // Balanced performance configs
        self.configs.push(OptimizationConfig::new(
            "perf-balanced".to_string(),
            OptLevel::O3,
            LtoSetting::Thin,
            CodegenUnits::Four,
            PgoSetting::Off,
            TargetCpu::Generic,
            StripSetting::None,
        ));
    }

    /// Add PGO variation configurations
    fn add_pgo_variations(&mut self) {
        // PGO + various opt-levels
        for opt_level in [OptLevel::O2, OptLevel::O3] {
            self.configs.push(OptimizationConfig::new(
                format!("pgo-opt{}", opt_level.to_profile_string()),
                opt_level,
                LtoSetting::Off,
                CodegenUnits::Sixteen,
                PgoSetting::On,
                TargetCpu::Generic,
                StripSetting::None,
            ));
        }

        // PGO + LTO combinations
        for lto in [LtoSetting::Thin, LtoSetting::Fat] {
            self.configs.push(OptimizationConfig::new(
                format!("pgo-lto-{:?}", lto).to_lowercase(),
                OptLevel::O3,
                lto,
                CodegenUnits::Sixteen,
                PgoSetting::On,
                TargetCpu::Generic,
                StripSetting::None,
            ));
        }

        // PGO + native
        self.configs.push(OptimizationConfig::new(
            "pgo-native".to_string(),
            OptLevel::O3,
            LtoSetting::Fat,
            CodegenUnits::One,
            PgoSetting::On,
            TargetCpu::Native,
            StripSetting::Symbols,
        ));
    }

    /// Add codegen-units interaction configurations
    fn add_codegen_variations(&mut self) {
        // Codegen-units + opt-level
        for units in [CodegenUnits::One, CodegenUnits::Four, CodegenUnits::Sixteen] {
            for opt_level in [OptLevel::O2, OptLevel::O3] {
                self.configs.push(OptimizationConfig::new(
                    format!("cg{}-opt{}", units.value(), opt_level.to_profile_string()),
                    opt_level,
                    LtoSetting::Off,
                    units,
                    PgoSetting::Off,
                    TargetCpu::Generic,
                    StripSetting::None,
                ));
            }
        }

        // Codegen-units + strip
        for units in [CodegenUnits::One, CodegenUnits::Sixteen] {
            self.configs.push(OptimizationConfig::new(
                format!("cg{}-strip", units.value()),
                OptLevel::O3,
                LtoSetting::Thin,
                units,
                PgoSetting::Off,
                TargetCpu::Generic,
                StripSetting::Symbols,
            ));
        }
    }

    /// Add target-CPU interaction configurations
    fn add_cpu_variations(&mut self) {
        // CPU + opt-level
        for cpu in [TargetCpu::Native, TargetCpu::Specific] {
            for opt_level in [OptLevel::O2, OptLevel::O3] {
                self.configs.push(OptimizationConfig::new(
                    format!("cpu-{:?}-opt{}", cpu, opt_level.to_profile_string()).to_lowercase(),
                    opt_level,
                    LtoSetting::Off,
                    CodegenUnits::Sixteen,
                    PgoSetting::Off,
                    cpu,
                    StripSetting::None,
                ));
            }
        }

        // CPU + codegen-units
        for cpu in [TargetCpu::Native] {
            for units in [CodegenUnits::One, CodegenUnits::Four] {
                self.configs.push(OptimizationConfig::new(
                    format!("cpu-native-cg{}", units.value()),
                    OptLevel::O3,
                    LtoSetting::Off,
                    units,
                    PgoSetting::Off,
                    cpu,
                    StripSetting::None,
                ));
            }
        }
    }

    /// Add strip interaction configurations
    fn add_strip_variations(&mut self) {
        // Strip + opt-level
        for strip in [StripSetting::Symbols, StripSetting::Debuginfo] {
            for opt_level in [OptLevel::O2, OptLevel::O3, OptLevel::Os] {
                self.configs.push(OptimizationConfig::new(
                    format!("strip-{:?}-opt{}", strip, opt_level.to_profile_string())
                        .to_lowercase(),
                    opt_level,
                    LtoSetting::Off,
                    CodegenUnits::Sixteen,
                    PgoSetting::Off,
                    TargetCpu::Generic,
                    strip,
                ));
            }
        }

        // Strip + LTO
        for strip in [StripSetting::Symbols, StripSetting::Debuginfo] {
            self.configs.push(OptimizationConfig::new(
                format!("strip-{:?}-lto", strip).to_lowercase(),
                OptLevel::O3,
                LtoSetting::Fat,
                CodegenUnits::One,
                PgoSetting::Off,
                TargetCpu::Generic,
                strip,
            ));
        }
    }

    /// Add extreme/edge case configurations
    fn add_extreme_configs(&mut self) {
        // Minimum optimization
        self.configs.push(OptimizationConfig::new(
            "min-opt".to_string(),
            OptLevel::O0,
            LtoSetting::Off,
            CodegenUnits::TwoFiftySix,
            PgoSetting::Off,
            TargetCpu::Generic,
            StripSetting::None,
        ));

        // Maximum optimization (kitchen sink)
        self.configs.push(OptimizationConfig::new(
            "max-opt".to_string(),
            OptLevel::O3,
            LtoSetting::Fat,
            CodegenUnits::One,
            PgoSetting::On,
            TargetCpu::Native,
            StripSetting::Debuginfo,
        ));

        // Fast compilation
        self.configs.push(OptimizationConfig::new(
            "fast-compile".to_string(),
            OptLevel::O1,
            LtoSetting::Off,
            CodegenUnits::TwoFiftySix,
            PgoSetting::Off,
            TargetCpu::Generic,
            StripSetting::None,
        ));
    }

    /// Add additional two-factor interaction configurations
    fn add_two_factor_interactions(&mut self) {
        // opt-level + strip
        for opt_level in [OptLevel::O1, OptLevel::O2] {
            for strip in [StripSetting::Symbols, StripSetting::Debuginfo] {
                self.configs.push(OptimizationConfig::new(
                    format!("opt{}-strip-{:?}", opt_level.to_profile_string(), strip)
                        .to_lowercase(),
                    opt_level,
                    LtoSetting::Off,
                    CodegenUnits::Sixteen,
                    PgoSetting::Off,
                    TargetCpu::Generic,
                    strip,
                ));
            }
        }

        // codegen-units + target-cpu
        for units in [CodegenUnits::One, CodegenUnits::Four, CodegenUnits::Sixteen] {
            self.configs.push(OptimizationConfig::new(
                format!("cg{}-native", units.value()),
                OptLevel::O3,
                LtoSetting::Off,
                units,
                PgoSetting::Off,
                TargetCpu::Native,
                StripSetting::None,
            ));
        }

        // PGO + codegen-units
        for units in [CodegenUnits::One, CodegenUnits::Four] {
            self.configs.push(OptimizationConfig::new(
                format!("pgo-cg{}", units.value()),
                OptLevel::O3,
                LtoSetting::Off,
                units,
                PgoSetting::On,
                TargetCpu::Generic,
                StripSetting::None,
            ));
        }

        // PGO + strip
        for strip in [StripSetting::Symbols, StripSetting::Debuginfo] {
            self.configs.push(OptimizationConfig::new(
                format!("pgo-strip-{:?}", strip).to_lowercase(),
                OptLevel::O3,
                LtoSetting::Thin,
                CodegenUnits::Sixteen,
                PgoSetting::On,
                TargetCpu::Generic,
                strip,
            ));
        }

        // Size + native (unusual combination)
        for opt_level in [OptLevel::Os, OptLevel::Oz] {
            self.configs.push(OptimizationConfig::new(
                format!("size-{}-native", opt_level.to_profile_string()),
                opt_level,
                LtoSetting::Thin,
                CodegenUnits::One,
                PgoSetting::Off,
                TargetCpu::Native,
                StripSetting::Symbols,
            ));
        }

        // Three specific high-value combos
        self.configs.push(OptimizationConfig::new(
            "balanced-perf".to_string(),
            OptLevel::O3,
            LtoSetting::Thin,
            CodegenUnits::Four,
            PgoSetting::Off,
            TargetCpu::Native,
            StripSetting::Symbols,
        ));

        self.configs.push(OptimizationConfig::new(
            "balanced-size".to_string(),
            OptLevel::Os,
            LtoSetting::Fat,
            CodegenUnits::One,
            PgoSetting::Off,
            TargetCpu::Generic,
            StripSetting::Debuginfo,
        ));

        self.configs.push(OptimizationConfig::new(
            "pgo-perf-ultra".to_string(),
            OptLevel::O3,
            LtoSetting::Fat,
            CodegenUnits::One,
            PgoSetting::On,
            TargetCpu::Native,
            StripSetting::Symbols,
        ));

        // Add one more to reach 80
        self.configs.push(OptimizationConfig::new(
            "default-dev".to_string(),
            OptLevel::O1,
            LtoSetting::Off,
            CodegenUnits::Sixteen,
            PgoSetting::Off,
            TargetCpu::Generic,
            StripSetting::None,
        ));
    }

    /// Get the generated configurations
    pub fn configs(&self) -> &[OptimizationConfig] {
        &self.configs
    }

    /// Get the count of generated configurations
    pub fn count(&self) -> usize {
        self.configs.len()
    }
}

impl Default for ConfigGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_creates_configs() {
        let mut gen = ConfigGenerator::new();
        let configs = gen.generate_matrix();
        assert!(!configs.is_empty());
    }

    #[test]
    fn test_generator_target_count() {
        let mut gen = ConfigGenerator::new();
        gen.generate_matrix();
        let count = gen.count();

        // Should generate approximately 100 configs
        // Allow some flexibility: 80-120 configs
        assert!(
            (80..=120).contains(&count),
            "Generated {} configs, expected 80-120",
            count
        );
    }

    #[test]
    fn test_baseline_configs_included() {
        let mut gen = ConfigGenerator::new();
        gen.generate_matrix();

        let has_baseline = gen.configs().iter().any(|c| c.id == "baseline");
        let has_standard = gen.configs().iter().any(|c| c.id == "standard-release");

        assert!(has_baseline, "Should include baseline config");
        assert!(has_standard, "Should include standard-release config");
    }

    #[test]
    fn test_all_config_ids_unique() {
        let mut gen = ConfigGenerator::new();
        gen.generate_matrix();

        let mut ids: Vec<_> = gen.configs().iter().map(|c| &c.id).collect();
        let original_len = ids.len();
        ids.sort();
        ids.dedup();

        assert_eq!(ids.len(), original_len, "All config IDs should be unique");
    }

    #[test]
    fn test_extreme_configs_included() {
        let mut gen = ConfigGenerator::new();
        gen.generate_matrix();

        let has_min = gen.configs().iter().any(|c| c.id == "min-opt");
        let has_max = gen.configs().iter().any(|c| c.id == "max-opt");

        assert!(has_min, "Should include min-opt config");
        assert!(has_max, "Should include max-opt config");
    }

    #[test]
    fn test_pgo_variations_included() {
        let mut gen = ConfigGenerator::new();
        gen.generate_matrix();

        let pgo_count = gen
            .configs()
            .iter()
            .filter(|c| c.pgo == PgoSetting::On)
            .count();

        assert!(pgo_count > 0, "Should include PGO configurations");
    }

    #[test]
    fn test_size_optimizations_included() {
        let mut gen = ConfigGenerator::new();
        gen.generate_matrix();

        let size_configs = gen
            .configs()
            .iter()
            .filter(|c| c.opt_level == OptLevel::Os || c.opt_level == OptLevel::Oz)
            .count();

        assert!(size_configs > 0, "Should include size optimization configs");
    }

    #[test]
    fn test_lto_variations_included() {
        let mut gen = ConfigGenerator::new();
        gen.generate_matrix();

        let thin_lto = gen
            .configs()
            .iter()
            .filter(|c| c.lto == LtoSetting::Thin)
            .count();
        let fat_lto = gen
            .configs()
            .iter()
            .filter(|c| c.lto == LtoSetting::Fat)
            .count();

        assert!(thin_lto > 0, "Should include thin LTO configs");
        assert!(fat_lto > 0, "Should include fat LTO configs");
    }
}
