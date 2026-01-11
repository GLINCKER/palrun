//! NPM/Yarn/PNPM/Bun package.json scanner.
//!
//! Scans package.json files to discover npm scripts.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use super::Scanner;
use crate::core::{Command, CommandSource};

/// Scanner for package.json scripts.
pub struct NpmScanner;

impl Scanner for NpmScanner {
    fn name(&self) -> &str {
        "npm"
    }

    fn scan(&self, path: &Path) -> anyhow::Result<Vec<Command>> {
        let package_json_path = path.join("package.json");
        if !package_json_path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&package_json_path)?;
        let package: PackageJson = serde_json::from_str(&content)?;

        let package_manager = detect_package_manager(path);
        let mut commands = Vec::new();

        if let Some(scripts) = package.scripts {
            for (name, script) in scripts {
                let cmd = Command::from_npm_script(
                    &name,
                    &script,
                    &package_manager,
                    Some(path.to_path_buf()),
                );
                commands.push(cmd);
            }
        }

        // Add common package manager commands
        commands.extend(generate_common_commands(&package_manager, path));

        Ok(commands)
    }
}

/// Parsed package.json structure.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct PackageJson {
    /// Package name
    pub name: Option<String>,

    /// Package version
    pub version: Option<String>,

    /// Scripts defined in package.json
    pub scripts: Option<HashMap<String, String>>,

    /// Workspace configuration
    pub workspaces: Option<Workspaces>,

    /// Dependencies (for detecting project type)
    pub dependencies: Option<HashMap<String, String>>,

    /// Dev dependencies
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<HashMap<String, String>>,
}

/// Workspace configuration (can be array or object).
#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)]
pub enum Workspaces {
    /// Simple array of globs
    Array(Vec<String>),

    /// Object with packages field
    Object {
        /// Package globs
        packages: Vec<String>,

        /// Packages to ignore
        #[serde(default)]
        nohoist: Vec<String>,
    },
}

impl Workspaces {
    /// Get the package glob patterns.
    #[allow(dead_code)]
    pub fn patterns(&self) -> Vec<String> {
        match self {
            Self::Array(patterns) => patterns.clone(),
            Self::Object { packages, .. } => packages.clone(),
        }
    }
}

/// Detect which package manager is being used.
pub fn detect_package_manager(path: &Path) -> String {
    if path.join("bun.lockb").exists() {
        "bun".to_string()
    } else if path.join("pnpm-lock.yaml").exists() {
        "pnpm".to_string()
    } else if path.join("yarn.lock").exists() {
        "yarn".to_string()
    } else {
        "npm".to_string()
    }
}

/// Generate common package manager commands.
fn generate_common_commands(package_manager: &str, path: &Path) -> Vec<Command> {
    let mut commands = Vec::new();

    let common_ops = [
        ("install", "Install dependencies"),
        ("update", "Update dependencies"),
        ("outdated", "Check for outdated packages"),
    ];

    for (op, desc) in common_ops {
        let cmd_str = match package_manager {
            "yarn" => format!("yarn {op}"),
            "pnpm" => format!("pnpm {op}"),
            "bun" => format!("bun {op}"),
            _ => format!("npm {op}"),
        };

        let cmd = Command::new(&cmd_str, &cmd_str)
            .with_description(desc)
            .with_source(CommandSource::PackageJson(path.to_path_buf()))
            .with_working_dir(path)
            .with_tag("package-manager");

        commands.push(cmd);
    }

    commands
}

/// Parse package.json from a path.
#[allow(dead_code)]
pub fn parse_package_json(path: &Path) -> anyhow::Result<PackageJson> {
    let content = std::fs::read_to_string(path.join("package.json"))?;
    let package: PackageJson = serde_json::from_str(&content)?;
    Ok(package)
}

/// Detect if this is a monorepo with workspaces.
#[allow(dead_code)]
pub fn is_monorepo(path: &Path) -> bool {
    if let Ok(package) = parse_package_json(path) {
        return package.workspaces.is_some();
    }

    // Also check for pnpm workspace
    path.join("pnpm-workspace.yaml").exists()
}

/// Get workspace patterns from package.json or pnpm-workspace.yaml.
#[allow(dead_code)]
pub fn get_workspace_patterns(path: &Path) -> anyhow::Result<Vec<String>> {
    // Try package.json first
    if let Ok(package) = parse_package_json(path) {
        if let Some(workspaces) = package.workspaces {
            return Ok(workspaces.patterns());
        }
    }

    // Try pnpm-workspace.yaml
    let pnpm_workspace = path.join("pnpm-workspace.yaml");
    if pnpm_workspace.exists() {
        let content = std::fs::read_to_string(&pnpm_workspace)?;

        #[derive(Deserialize)]
        struct PnpmWorkspace {
            packages: Vec<String>,
        }

        let workspace: PnpmWorkspace = serde_yaml::from_str(&content)?;
        return Ok(workspace.packages);
    }

    Ok(Vec::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_json_with_scripts() {
        let json = r#"{
            "name": "test-package",
            "version": "1.0.0",
            "scripts": {
                "test": "jest",
                "build": "tsc"
            }
        }"#;

        let package: PackageJson = serde_json::from_str(json).unwrap();
        assert_eq!(package.name, Some("test-package".to_string()));

        let scripts = package.scripts.unwrap();
        assert_eq!(scripts.get("test"), Some(&"jest".to_string()));
        assert_eq!(scripts.get("build"), Some(&"tsc".to_string()));
    }

    #[test]
    fn test_parse_workspaces_array() {
        let json = r#"{
            "name": "monorepo",
            "workspaces": ["packages/*", "apps/*"]
        }"#;

        let package: PackageJson = serde_json::from_str(json).unwrap();
        let workspaces = package.workspaces.unwrap();
        let patterns = workspaces.patterns();

        assert_eq!(patterns.len(), 2);
        assert!(patterns.contains(&"packages/*".to_string()));
        assert!(patterns.contains(&"apps/*".to_string()));
    }

    #[test]
    fn test_parse_workspaces_object() {
        let json = r#"{
            "name": "monorepo",
            "workspaces": {
                "packages": ["packages/*"],
                "nohoist": ["**/react-native"]
            }
        }"#;

        let package: PackageJson = serde_json::from_str(json).unwrap();
        let workspaces = package.workspaces.unwrap();
        let patterns = workspaces.patterns();

        assert_eq!(patterns.len(), 1);
        assert!(patterns.contains(&"packages/*".to_string()));
    }

    #[test]
    fn test_npm_scanner_name() {
        let scanner = NpmScanner;
        assert_eq!(scanner.name(), "npm");
    }
}
