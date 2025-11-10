//! Configuration generation for optimization matrix
//!
//! This module provides types and functions for generating and validating
//! optimization configurations for the benchmark harness.

pub mod generator;

use serde::{Deserialize, Serialize};

/// Optimization level setting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptLevel {
    /// No optimization (debug)
    O0,
    /// Basic optimization
    O1,
    /// Moderate optimization
    O2,
    /// Aggressive optimization (production)
    O3,
    /// Size optimization
    Os,
    /// Aggressive size optimization
    Oz,
}

impl OptLevel {
    /// Convert to Cargo profile string
    pub fn to_profile_string(&self) -> &'static str {
        match self {
            OptLevel::O0 => "0",
            OptLevel::O1 => "1",
            OptLevel::O2 => "2",
            OptLevel::O3 => "3",
            OptLevel::Os => "s",
            OptLevel::Oz => "z",
        }
    }
}

/// Link-Time Optimization setting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LtoSetting {
    /// No LTO
    Off,
    /// Thin LTO (fast, good balance)
    Thin,
    /// Fat LTO (slow build, maximum optimization)
    Fat,
}

impl LtoSetting {
    /// Convert to Cargo profile string
    pub fn to_profile_string(&self) -> &'static str {
        match self {
            LtoSetting::Off => "false",
            LtoSetting::Thin => "\"thin\"",
            LtoSetting::Fat => "\"fat\"",
        }
    }
}

/// Codegen units setting (parallelization vs optimization)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodegenUnits {
    /// Maximum optimization, no parallelization
    One = 1,
    /// Balanced
    Four = 4,
    /// Default (fast compilation)
    Sixteen = 16,
    /// Maximum parallelization (fastest compilation)
    TwoFiftySix = 256,
}

impl CodegenUnits {
    /// Get the numeric value
    pub fn value(&self) -> u32 {
        *self as u32
    }
}

/// Profile-Guided Optimization setting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PgoSetting {
    /// No PGO
    Off,
    /// PGO enabled
    On,
}

/// Target CPU setting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetCpu {
    /// Generic (portable)
    Generic,
    /// Native (current CPU)
    Native,
    /// Specific microarchitecture (e.g., haswell, skylake)
    Specific,
}

impl TargetCpu {
    /// Convert to rustc flag value
    pub fn to_rustc_flag(&self) -> &'static str {
        match self {
            TargetCpu::Generic => "generic",
            TargetCpu::Native => "native",
            TargetCpu::Specific => "haswell", // Default specific target
        }
    }
}

/// Binary stripping setting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StripSetting {
    /// No stripping
    None,
    /// Strip symbols only
    Symbols,
    /// Strip all debug info
    Debuginfo,
}

impl StripSetting {
    /// Convert to Cargo profile string
    pub fn to_profile_string(&self) -> &'static str {
        match self {
            StripSetting::None => "false",
            StripSetting::Symbols => "\"symbols\"",
            StripSetting::Debuginfo => "\"debuginfo\"",
        }
    }
}

/// Complete optimization configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Configuration ID
    pub id: String,
    /// Optimization level
    pub opt_level: OptLevel,
    /// LTO setting
    pub lto: LtoSetting,
    /// Codegen units
    pub codegen_units: CodegenUnits,
    /// PGO enabled
    pub pgo: PgoSetting,
    /// Target CPU
    pub target_cpu: TargetCpu,
    /// Strip setting
    pub strip: StripSetting,
}

impl OptimizationConfig {
    /// Create a new configuration
    pub fn new(
        id: String,
        opt_level: OptLevel,
        lto: LtoSetting,
        codegen_units: CodegenUnits,
        pgo: PgoSetting,
        target_cpu: TargetCpu,
        strip: StripSetting,
    ) -> Self {
        Self {
            id,
            opt_level,
            lto,
            codegen_units,
            pgo,
            target_cpu,
            strip,
        }
    }

    /// Create baseline (debug) configuration
    pub fn baseline() -> Self {
        Self {
            id: "baseline".to_string(),
            opt_level: OptLevel::O0,
            lto: LtoSetting::Off,
            codegen_units: CodegenUnits::Sixteen,
            pgo: PgoSetting::Off,
            target_cpu: TargetCpu::Generic,
            strip: StripSetting::None,
        }
    }

    /// Create standard release configuration
    pub fn standard_release() -> Self {
        Self {
            id: "standard-release".to_string(),
            opt_level: OptLevel::O3,
            lto: LtoSetting::Off,
            codegen_units: CodegenUnits::Sixteen,
            pgo: PgoSetting::Off,
            target_cpu: TargetCpu::Generic,
            strip: StripSetting::None,
        }
    }

    /// Generate Cargo.toml profile section
    pub fn to_cargo_profile(&self, profile_name: &str) -> String {
        format!(
            r#"[profile.{}]
opt-level = {}
lto = {}
codegen-units = {}
strip = {}
"#,
            profile_name,
            self.opt_level.to_profile_string(),
            self.lto.to_profile_string(),
            self.codegen_units.value(),
            self.strip.to_profile_string()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opt_level_to_profile_string() {
        assert_eq!(OptLevel::O0.to_profile_string(), "0");
        assert_eq!(OptLevel::O1.to_profile_string(), "1");
        assert_eq!(OptLevel::O2.to_profile_string(), "2");
        assert_eq!(OptLevel::O3.to_profile_string(), "3");
        assert_eq!(OptLevel::Os.to_profile_string(), "s");
        assert_eq!(OptLevel::Oz.to_profile_string(), "z");
    }

    #[test]
    fn test_lto_to_profile_string() {
        assert_eq!(LtoSetting::Off.to_profile_string(), "false");
        assert_eq!(LtoSetting::Thin.to_profile_string(), "\"thin\"");
        assert_eq!(LtoSetting::Fat.to_profile_string(), "\"fat\"");
    }

    #[test]
    fn test_codegen_units_value() {
        assert_eq!(CodegenUnits::One.value(), 1);
        assert_eq!(CodegenUnits::Four.value(), 4);
        assert_eq!(CodegenUnits::Sixteen.value(), 16);
        assert_eq!(CodegenUnits::TwoFiftySix.value(), 256);
    }

    #[test]
    fn test_target_cpu_to_rustc_flag() {
        assert_eq!(TargetCpu::Generic.to_rustc_flag(), "generic");
        assert_eq!(TargetCpu::Native.to_rustc_flag(), "native");
        assert_eq!(TargetCpu::Specific.to_rustc_flag(), "haswell");
    }

    #[test]
    fn test_strip_to_profile_string() {
        assert_eq!(StripSetting::None.to_profile_string(), "false");
        assert_eq!(StripSetting::Symbols.to_profile_string(), "\"symbols\"");
        assert_eq!(StripSetting::Debuginfo.to_profile_string(), "\"debuginfo\"");
    }

    #[test]
    fn test_baseline_config() {
        let config = OptimizationConfig::baseline();
        assert_eq!(config.id, "baseline");
        assert_eq!(config.opt_level, OptLevel::O0);
        assert_eq!(config.lto, LtoSetting::Off);
        assert_eq!(config.pgo, PgoSetting::Off);
    }

    #[test]
    fn test_standard_release_config() {
        let config = OptimizationConfig::standard_release();
        assert_eq!(config.id, "standard-release");
        assert_eq!(config.opt_level, OptLevel::O3);
        assert_eq!(config.lto, LtoSetting::Off);
        assert_eq!(config.codegen_units, CodegenUnits::Sixteen);
    }

    #[test]
    fn test_to_cargo_profile() {
        let config = OptimizationConfig::standard_release();
        let profile = config.to_cargo_profile("test-profile");

        assert!(profile.contains("[profile.test-profile]"));
        assert!(profile.contains("opt-level = 3"));
        assert!(profile.contains("lto = false"));
        assert!(profile.contains("codegen-units = 16"));
    }

    #[test]
    fn test_new_config() {
        let config = OptimizationConfig::new(
            "custom".to_string(),
            OptLevel::O3,
            LtoSetting::Fat,
            CodegenUnits::One,
            PgoSetting::On,
            TargetCpu::Native,
            StripSetting::Symbols,
        );

        assert_eq!(config.id, "custom");
        assert_eq!(config.opt_level, OptLevel::O3);
        assert_eq!(config.lto, LtoSetting::Fat);
        assert_eq!(config.codegen_units, CodegenUnits::One);
        assert_eq!(config.pgo, PgoSetting::On);
        assert_eq!(config.target_cpu, TargetCpu::Native);
        assert_eq!(config.strip, StripSetting::Symbols);
    }
}
