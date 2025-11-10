//! Statistical analysis for benchmark results
//!
//! Provides frequentist and Bayesian statistical methods for analyzing
//! optimization benchmark data.

pub mod basic;
pub mod frequentist;

pub use basic::*;
pub use frequentist::*;
