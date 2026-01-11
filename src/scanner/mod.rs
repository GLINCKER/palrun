//! Project scanners for discovering commands.
//!
//! This module contains scanners that detect and parse various project
//! configuration files to discover available commands.

mod cargo;
mod docker;
mod git;
mod go_lang;
mod makefile;
mod npm;
mod nx;
mod python;
mod taskfile;
mod turbo;

pub use cargo::CargoScanner;
pub use docker::DockerScanner;
pub use git::GitScanner;
pub use go_lang::GoScanner;
pub use makefile::MakefileScanner;
pub use npm::NpmScanner;
pub use nx::NxScanner;
pub use python::PythonScanner;
pub use taskfile::TaskfileScanner;
pub use turbo::TurboScanner;

use std::path::Path;

use crate::core::Command;

/// Trait for project scanners.
pub trait Scanner: Send + Sync {
    /// Get the name of this scanner.
    fn name(&self) -> &str;

    /// Scan the directory and return discovered commands.
    fn scan(&self, path: &Path) -> anyhow::Result<Vec<Command>>;
}

/// Main project scanner that aggregates all individual scanners.
pub struct ProjectScanner {
    /// Root directory to scan
    root: std::path::PathBuf,

    /// Enabled scanners
    scanners: Vec<Box<dyn Scanner>>,
}

impl ProjectScanner {
    /// Create a new project scanner for the given directory.
    pub fn new(root: &Path) -> Self {
        let scanners: Vec<Box<dyn Scanner>> = vec![
            Box::new(NpmScanner),
            Box::new(MakefileScanner),
            Box::new(NxScanner),
            Box::new(TurboScanner),
            Box::new(CargoScanner),
            Box::new(TaskfileScanner),
            Box::new(DockerScanner),
            Box::new(GoScanner),
            Box::new(PythonScanner),
            Box::new(GitScanner),
        ];

        Self { root: root.to_path_buf(), scanners }
    }

    /// Scan the project and return all discovered commands.
    pub fn scan(&self) -> anyhow::Result<Vec<Command>> {
        let mut all_commands = Vec::new();

        for scanner in &self.scanners {
            match scanner.scan(&self.root) {
                Ok(commands) => {
                    if !commands.is_empty() {
                        tracing::debug!(
                            scanner = scanner.name(),
                            count = commands.len(),
                            "Discovered commands"
                        );
                        all_commands.extend(commands);
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        scanner = scanner.name(),
                        error = %e,
                        "Scanner failed"
                    );
                }
            }
        }

        // Sort commands by name for consistent ordering
        all_commands.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(all_commands)
    }

    /// Scan with recursive workspace detection.
    pub fn scan_recursive(&self, max_depth: usize) -> anyhow::Result<Vec<Command>> {
        let mut all_commands = self.scan()?;

        if max_depth > 0 {
            // Look for workspace subdirectories
            if let Ok(entries) = std::fs::read_dir(&self.root) {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if path.is_dir() {
                        // Skip common non-project directories
                        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if should_skip_dir(name) {
                            continue;
                        }

                        let sub_scanner = ProjectScanner::new(&path);
                        if let Ok(sub_commands) = sub_scanner.scan_recursive(max_depth - 1) {
                            all_commands.extend(sub_commands);
                        }
                    }
                }
            }
        }

        Ok(all_commands)
    }

    /// Get the number of scanners.
    pub fn scanner_count(&self) -> usize {
        self.scanners.len()
    }
}

/// Check if a directory should be skipped during scanning.
fn should_skip_dir(name: &str) -> bool {
    matches!(
        name,
        "node_modules"
            | ".git"
            | "target"
            | "dist"
            | "build"
            | ".next"
            | ".nuxt"
            | ".output"
            | "coverage"
            | ".cache"
            | ".turbo"
            | ".nx"
            | ".pnpm"
            | "vendor"
            | "__pycache__"
            | ".venv"
            | "venv"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_skip_dir() {
        assert!(should_skip_dir("node_modules"));
        assert!(should_skip_dir(".git"));
        assert!(should_skip_dir("target"));
        assert!(should_skip_dir("vendor"));

        assert!(!should_skip_dir("src"));
        assert!(!should_skip_dir("packages"));
        assert!(!should_skip_dir("apps"));
    }

    #[test]
    fn test_project_scanner_creation() {
        let scanner = ProjectScanner::new(Path::new("."));
        assert_eq!(scanner.scanner_count(), 10);
    }

    #[test]
    fn test_all_scanners_have_names() {
        let scanner = ProjectScanner::new(Path::new("."));
        for s in &scanner.scanners {
            assert!(!s.name().is_empty());
        }
    }
}
