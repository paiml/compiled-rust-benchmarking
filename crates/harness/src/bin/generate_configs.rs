//! Configuration generator binary
//!
//! Generates all optimization configurations and exports them to TOML files.

use harness::config::generator::ConfigGenerator;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating optimization configurations...");

    let mut generator = ConfigGenerator::new();
    let configs = generator.generate_matrix();

    println!("Generated {} configurations", configs.len());

    // Create configs directory if it doesn't exist
    let configs_dir = Path::new("configs");
    fs::create_dir_all(configs_dir)?;

    // Export each configuration to a TOML file
    for config in configs {
        let filename = format!("{}.toml", config.id);
        let filepath = configs_dir.join(&filename);

        let toml_content = config.to_cargo_profile(&config.id);

        fs::write(&filepath, toml_content)?;
        println!("  ✓ {}", filename);
    }

    println!(
        "\n✅ Successfully generated {} configuration files in configs/",
        configs.len()
    );
    println!("\nConfiguration breakdown:");
    println!("  - Baseline configurations: 2");
    println!("  - Single-factor variations: ~15");
    println!("  - LTO optimizations: ~15");
    println!("  - Size optimizations: ~10");
    println!("  - Performance optimizations: ~15");
    println!("  - PGO variations: ~10");
    println!("  - Two-factor interactions: ~20");
    println!("  - Extreme cases: 3");

    Ok(())
}
