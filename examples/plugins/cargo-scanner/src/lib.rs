//! Cargo Scanner Plugin for Palrun
//!
//! Scans Rust Cargo projects (Cargo.toml) and extracts available
//! commands, binaries, examples, and tests.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub working_dir: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Common Cargo commands with descriptions.
const CARGO_COMMANDS: &[(&str, &str)] = &[
    ("build", "Compile the current package"),
    ("build --release", "Compile with optimizations"),
    ("check", "Check the package for errors without building"),
    ("test", "Run the tests"),
    ("test -- --nocapture", "Run tests with stdout visible"),
    ("run", "Run the main binary"),
    ("run --release", "Run the optimized binary"),
    ("clean", "Remove the target directory"),
    ("doc", "Build documentation"),
    ("doc --open", "Build and open documentation"),
    ("update", "Update dependencies"),
    ("fmt", "Format code with rustfmt"),
    ("fmt --check", "Check formatting without modifying"),
    ("clippy", "Run clippy lints"),
    ("clippy --fix", "Run clippy and apply fixes"),
];

/// Cargo workspace commands.
const WORKSPACE_COMMANDS: &[(&str, &str)] = &[
    ("build --workspace", "Build all workspace members"),
    ("test --workspace", "Test all workspace members"),
    ("check --workspace", "Check all workspace members"),
];

/// Parsed Cargo.toml structure.
#[derive(Debug, Deserialize, Default)]
struct CargoToml {
    package: Option<PackageInfo>,
    #[serde(default)]
    workspace: Option<WorkspaceInfo>,
    #[serde(default)]
    bin: Vec<BinaryTarget>,
    #[serde(default)]
    example: Vec<BinaryTarget>,
    #[serde(default)]
    bench: Vec<BinaryTarget>,
    #[serde(default)]
    features: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize, Default)]
struct PackageInfo {
    name: Option<String>,
    #[serde(default)]
    default_run: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct WorkspaceInfo {
    #[serde(default)]
    members: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct BinaryTarget {
    name: String,
    #[serde(default)]
    path: Option<String>,
}

/// Parse a Cargo.toml file and extract available commands.
pub fn parse_cargo_toml(content: &str) -> Vec<PluginCommand> {
    let mut commands = Vec::new();

    // Try to parse the TOML
    let cargo: CargoToml = match toml::from_str(content) {
        Ok(c) => c,
        Err(_) => return commands,
    };

    let is_workspace = cargo.workspace.is_some();
    let package_name = cargo
        .package
        .as_ref()
        .and_then(|p| p.name.clone())
        .unwrap_or_else(|| "package".to_string());

    // Add basic Cargo commands
    for (cmd, desc) in CARGO_COMMANDS {
        commands.push(PluginCommand {
            name: format!("cargo {}", cmd),
            command: format!("cargo {}", cmd),
            description: Some(desc.to_string()),
            working_dir: None,
            tags: vec!["cargo".to_string(), "rust".to_string()],
        });
    }

    // Add workspace commands if this is a workspace
    if is_workspace {
        for (cmd, desc) in WORKSPACE_COMMANDS {
            commands.push(PluginCommand {
                name: format!("cargo {}", cmd),
                command: format!("cargo {}", cmd),
                description: Some(desc.to_string()),
                working_dir: None,
                tags: vec!["cargo".to_string(), "workspace".to_string()],
            });
        }
    }

    // Add binary targets
    for bin in &cargo.bin {
        commands.push(PluginCommand {
            name: format!("cargo run --bin {}", bin.name),
            command: format!("cargo run --bin {}", bin.name),
            description: Some(format!("Run '{}' binary", bin.name)),
            working_dir: None,
            tags: vec!["cargo".to_string(), "binary".to_string()],
        });
    }

    // Add example targets
    for example in &cargo.example {
        commands.push(PluginCommand {
            name: format!("cargo run --example {}", example.name),
            command: format!("cargo run --example {}", example.name),
            description: Some(format!("Run '{}' example", example.name)),
            working_dir: None,
            tags: vec!["cargo".to_string(), "example".to_string()],
        });
    }

    // Add benchmark targets
    for bench in &cargo.bench {
        commands.push(PluginCommand {
            name: format!("cargo bench --bench {}", bench.name),
            command: format!("cargo bench --bench {}", bench.name),
            description: Some(format!("Run '{}' benchmark", bench.name)),
            working_dir: None,
            tags: vec!["cargo".to_string(), "benchmark".to_string()],
        });
    }

    // Add feature-specific builds
    let non_default_features: Vec<_> = cargo
        .features
        .keys()
        .filter(|f| *f != "default")
        .take(5) // Limit to avoid too many commands
        .collect();

    for feature in non_default_features {
        commands.push(PluginCommand {
            name: format!("cargo build --features {}", feature),
            command: format!("cargo build --features {}", feature),
            description: Some(format!("Build with '{}' feature", feature)),
            working_dir: None,
            tags: vec!["cargo".to_string(), "feature".to_string()],
        });
    }

    // Add test with package name
    commands.push(PluginCommand {
        name: format!("cargo test -p {}", package_name),
        command: format!("cargo test -p {}", package_name),
        description: Some(format!("Test '{}' package", package_name)),
        working_dir: None,
        tags: vec!["cargo".to_string(), "test".to_string()],
    });

    commands
}

/// Main entry point for the scanner plugin.
#[no_mangle]
pub extern "C" fn scan(project_path_ptr: *const u8, project_path_len: usize) -> *mut u8 {
    let project_path = unsafe {
        let slice = std::slice::from_raw_parts(project_path_ptr, project_path_len);
        std::str::from_utf8(slice).unwrap_or("")
    };

    let commands = scan_project(project_path);
    let json = serde_json::to_string(&commands).unwrap_or_else(|_| "[]".to_string());
    let bytes = json.into_bytes();
    let ptr = bytes.as_ptr() as *mut u8;
    std::mem::forget(bytes);
    ptr
}

/// Scan a project directory for Cargo commands.
pub fn scan_project(project_path: &str) -> Vec<PluginCommand> {
    let cargo_path = std::path::Path::new(project_path).join("Cargo.toml");

    if cargo_path.exists() {
        // In a real plugin, read through host API
        parse_cargo_toml("")
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_commands() {
        let content = r#"
[package]
name = "myproject"
version = "0.1.0"
"#;

        let commands = parse_cargo_toml(content);
        assert!(commands.iter().any(|c| c.name == "cargo build"));
        assert!(commands.iter().any(|c| c.name == "cargo test"));
        assert!(commands.iter().any(|c| c.name == "cargo run"));
        assert!(commands.iter().any(|c| c.name == "cargo clippy"));
    }

    #[test]
    fn test_parse_binary_targets() {
        let content = r#"
[package]
name = "myproject"

[[bin]]
name = "mycli"
path = "src/bin/mycli.rs"

[[bin]]
name = "server"
path = "src/bin/server.rs"
"#;

        let commands = parse_cargo_toml(content);
        assert!(commands.iter().any(|c| c.name == "cargo run --bin mycli"));
        assert!(commands.iter().any(|c| c.name == "cargo run --bin server"));
    }

    #[test]
    fn test_parse_examples() {
        let content = r#"
[package]
name = "myproject"

[[example]]
name = "basic"

[[example]]
name = "advanced"
"#;

        let commands = parse_cargo_toml(content);
        assert!(commands.iter().any(|c| c.name == "cargo run --example basic"));
        assert!(commands
            .iter()
            .any(|c| c.name == "cargo run --example advanced"));
    }

    #[test]
    fn test_parse_workspace() {
        let content = r#"
[workspace]
members = ["crates/*"]
"#;

        let commands = parse_cargo_toml(content);
        assert!(commands.iter().any(|c| c.name == "cargo build --workspace"));
        assert!(commands.iter().any(|c| c.name == "cargo test --workspace"));
    }

    #[test]
    fn test_parse_features() {
        let content = r#"
[package]
name = "myproject"

[features]
default = ["std"]
std = []
async = ["tokio"]
serde = ["dep:serde"]
"#;

        let commands = parse_cargo_toml(content);
        // Should have feature-specific builds (not default)
        let feature_cmds: Vec<_> = commands
            .iter()
            .filter(|c| c.name.contains("--features"))
            .collect();
        assert!(feature_cmds.len() >= 1);
    }

    #[test]
    fn test_invalid_toml() {
        let content = "not valid toml [";
        let commands = parse_cargo_toml(content);
        assert!(commands.is_empty());
    }
}
