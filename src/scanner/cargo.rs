//! Cargo/Rust project scanner.
//!
//! Scans Cargo.toml to discover Rust project commands.

use std::path::Path;

use serde::Deserialize;

use super::Scanner;
use crate::core::{Command, CommandSource};

/// Scanner for Rust/Cargo projects.
pub struct CargoScanner;

impl Scanner for CargoScanner {
    fn name(&self) -> &str {
        "cargo"
    }

    fn scan(&self, dir: &Path) -> anyhow::Result<Vec<Command>> {
        let mut commands = Vec::new();

        // Check for Cargo.toml
        let cargo_toml_path = dir.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            return Ok(commands);
        }

        // Parse Cargo.toml
        let config = parse_cargo_toml(&cargo_toml_path)?;
        let source = CommandSource::Cargo(cargo_toml_path.clone());

        // Get package name for context
        let package_name = config
            .package
            .as_ref()
            .and_then(|p| p.name.clone())
            .unwrap_or_else(|| "project".to_string());

        // Basic cargo commands
        commands.push(
            Command::new("cargo build", "cargo build")
                .with_description(format!("Build {package_name}"))
                .with_source(source.clone())
                .with_tags(vec!["cargo".to_string(), "build".to_string()]),
        );

        commands.push(
            Command::new("cargo build --release", "cargo build --release")
                .with_description(format!("Build {package_name} (release)"))
                .with_source(source.clone())
                .with_tags(vec!["cargo".to_string(), "build".to_string(), "release".to_string()]),
        );

        commands.push(
            Command::new("cargo test", "cargo test")
                .with_description(format!("Test {package_name}"))
                .with_source(source.clone())
                .with_tags(vec!["cargo".to_string(), "test".to_string()]),
        );

        commands.push(
            Command::new("cargo run", "cargo run")
                .with_description(format!("Run {package_name}"))
                .with_source(source.clone())
                .with_tags(vec!["cargo".to_string(), "run".to_string()]),
        );

        commands.push(
            Command::new("cargo check", "cargo check")
                .with_description("Check for compilation errors")
                .with_source(source.clone())
                .with_tags(vec!["cargo".to_string(), "check".to_string()]),
        );

        commands.push(
            Command::new("cargo clippy", "cargo clippy")
                .with_description("Run Clippy lints")
                .with_source(source.clone())
                .with_tags(vec!["cargo".to_string(), "lint".to_string()]),
        );

        commands.push(
            Command::new("cargo fmt", "cargo fmt")
                .with_description("Format code")
                .with_source(source.clone())
                .with_tags(vec!["cargo".to_string(), "format".to_string()]),
        );

        commands.push(
            Command::new("cargo doc --open", "cargo doc --open")
                .with_description("Generate and open documentation")
                .with_source(source.clone())
                .with_tags(vec!["cargo".to_string(), "docs".to_string()]),
        );

        // Add binary targets
        if let Some(bins) = &config.bin {
            for bin in bins {
                if let Some(bin_name) = &bin.name {
                    commands.push(
                        Command::new(
                            format!("cargo run --bin {bin_name}"),
                            format!("cargo run --bin {bin_name}"),
                        )
                        .with_description(format!("Run {bin_name} binary"))
                        .with_source(source.clone())
                        .with_tags(vec![
                            "cargo".to_string(),
                            "run".to_string(),
                            bin_name.clone(),
                        ]),
                    );
                }
            }
        }

        // Add example targets
        if let Some(examples) = &config.example {
            for example in examples {
                if let Some(example_name) = &example.name {
                    commands.push(
                        Command::new(
                            format!("cargo run --example {example_name}"),
                            format!("cargo run --example {example_name}"),
                        )
                        .with_description(format!("Run {example_name} example"))
                        .with_source(source.clone())
                        .with_tags(vec!["cargo".to_string(), "example".to_string()]),
                    );
                }
            }
        }

        // Add benchmark if present
        if config.bench.is_some() {
            commands.push(
                Command::new("cargo bench", "cargo bench")
                    .with_description("Run benchmarks")
                    .with_source(source.clone())
                    .with_tags(vec!["cargo".to_string(), "bench".to_string()]),
            );
        }

        // Check for workspace
        if config.workspace.is_some() {
            commands.push(
                Command::new("cargo build --workspace", "cargo build --workspace")
                    .with_description("Build all workspace members")
                    .with_source(source.clone())
                    .with_tags(vec!["cargo".to_string(), "workspace".to_string()]),
            );

            commands.push(
                Command::new("cargo test --workspace", "cargo test --workspace")
                    .with_description("Test all workspace members")
                    .with_source(source.clone())
                    .with_tags(vec!["cargo".to_string(), "workspace".to_string()]),
            );

            // Scan workspace members
            if let Some(workspace) = &config.workspace {
                if let Some(members) = &workspace.members {
                    for member in members {
                        // Skip glob patterns for now
                        if member.contains('*') {
                            continue;
                        }
                        commands.push(
                            Command::new(
                                format!("cargo build -p {member}"),
                                format!("cargo build -p {member}"),
                            )
                            .with_description(format!("Build {member}"))
                            .with_source(source.clone())
                            .with_tags(vec!["cargo".to_string(), member.clone()]),
                        );
                    }
                }
            }
        }

        // Add feature-specific commands if features exist
        if let Some(features) = &config.features {
            if !features.is_empty() {
                commands.push(
                    Command::new("cargo build --all-features", "cargo build --all-features")
                        .with_description("Build with all features enabled")
                        .with_source(source.clone())
                        .with_tags(vec!["cargo".to_string(), "features".to_string()]),
                );

                commands.push(
                    Command::new("cargo test --all-features", "cargo test --all-features")
                        .with_description("Test with all features enabled")
                        .with_source(source.clone())
                        .with_tags(vec!["cargo".to_string(), "features".to_string()]),
                );

                // Add individual feature commands for non-default features
                for feature_name in features.keys() {
                    if feature_name != "default" {
                        commands.push(
                            Command::new(
                                format!("cargo build --features {feature_name}"),
                                format!("cargo build --features {feature_name}"),
                            )
                            .with_description(format!("Build with {feature_name} feature"))
                            .with_source(source.clone())
                            .with_tags(vec![
                                "cargo".to_string(),
                                "feature".to_string(),
                                feature_name.clone(),
                            ]),
                        );
                    }
                }
            }
        }

        Ok(commands)
    }
}

/// Cargo.toml configuration.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CargoConfig {
    /// Package metadata
    package: Option<Package>,
    /// Workspace configuration
    workspace: Option<Workspace>,
    /// Binary targets
    bin: Option<Vec<Target>>,
    /// Example targets
    example: Option<Vec<Target>>,
    /// Benchmark targets
    bench: Option<Vec<Target>>,
    /// Features
    features: Option<std::collections::HashMap<String, Vec<String>>>,
}

/// Package metadata.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Package {
    /// Package name
    name: Option<String>,
    /// Package version
    version: Option<String>,
}

/// Workspace configuration.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Workspace {
    /// Workspace members
    members: Option<Vec<String>>,
    /// Excluded members
    exclude: Option<Vec<String>>,
}

/// Target configuration (bin, example, bench).
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Target {
    /// Target name
    name: Option<String>,
    /// Path to the target
    path: Option<String>,
}

/// Parse Cargo.toml file.
fn parse_cargo_toml(path: &Path) -> anyhow::Result<CargoConfig> {
    let content = std::fs::read_to_string(path)?;
    let config: CargoConfig = toml::from_str(&content)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_scanner_name() {
        let scanner = CargoScanner;
        assert_eq!(scanner.name(), "cargo");
    }

    #[test]
    fn test_parse_simple_cargo_toml() {
        let toml = r#"
[package]
name = "my-project"
version = "0.1.0"

[dependencies]
serde = "1.0"
"#;

        let config: CargoConfig = toml::from_str(toml).unwrap();
        assert!(config.package.is_some());
        let package = config.package.unwrap();
        assert_eq!(package.name, Some("my-project".to_string()));
        assert_eq!(package.version, Some("0.1.0".to_string()));
    }

    #[test]
    fn test_parse_workspace_cargo_toml() {
        let toml = r#"
[workspace]
members = [
    "crates/core",
    "crates/cli",
    "crates/lib"
]
exclude = ["examples"]
"#;

        let config: CargoConfig = toml::from_str(toml).unwrap();
        assert!(config.workspace.is_some());
        let workspace = config.workspace.unwrap();
        assert!(workspace.members.is_some());
        let members = workspace.members.unwrap();
        assert_eq!(members.len(), 3);
        assert!(members.contains(&"crates/core".to_string()));
    }

    #[test]
    fn test_parse_cargo_with_bins() {
        let toml = r#"
[package]
name = "multi-bin"
version = "0.1.0"

[[bin]]
name = "server"
path = "src/bin/server.rs"

[[bin]]
name = "client"
path = "src/bin/client.rs"
"#;

        let config: CargoConfig = toml::from_str(toml).unwrap();
        assert!(config.bin.is_some());
        let bins = config.bin.unwrap();
        assert_eq!(bins.len(), 2);
        assert_eq!(bins[0].name, Some("server".to_string()));
        assert_eq!(bins[1].name, Some("client".to_string()));
    }

    #[test]
    fn test_parse_cargo_with_features() {
        let toml = r#"
[package]
name = "feature-project"
version = "0.1.0"

[features]
default = ["std"]
std = []
async = ["tokio"]
full = ["std", "async"]
"#;

        let config: CargoConfig = toml::from_str(toml).unwrap();
        assert!(config.features.is_some());
        let features = config.features.unwrap();
        assert_eq!(features.len(), 4);
        assert!(features.contains_key("default"));
        assert!(features.contains_key("async"));
    }
}
