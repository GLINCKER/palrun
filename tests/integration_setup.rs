//! Integration tests for `palrun setup` command.

use std::fs;

use palrun::init::{setup_project, ProjectDetector, ProjectType, SetupOptions};
use tempfile::TempDir;

#[test]
fn test_detect_nextjs_project() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // Create Next.js project files
    fs::write(path.join("package.json"), r#"{"name": "test"}"#).unwrap();
    fs::write(path.join("next.config.js"), "module.exports = {}").unwrap();

    let detector = ProjectDetector::new(path);
    let project_type = detector.detect().unwrap();

    assert_eq!(project_type, ProjectType::NextJs);
}

#[test]
fn test_detect_rust_project() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // Create Rust project files
    fs::write(path.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

    let detector = ProjectDetector::new(path);
    let project_type = detector.detect().unwrap();

    assert_eq!(project_type, ProjectType::Rust);
}

#[test]
fn test_detect_python_project() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // Create Python project files
    fs::write(path.join("pyproject.toml"), "[tool.poetry]\nname = \"test\"").unwrap();

    let detector = ProjectDetector::new(path);
    let project_type = detector.detect().unwrap();

    assert_eq!(project_type, ProjectType::Python);
}

#[test]
fn test_detect_go_project() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // Create Go project files
    fs::write(path.join("go.mod"), "module test").unwrap();

    let detector = ProjectDetector::new(path);
    let project_type = detector.detect().unwrap();

    assert_eq!(project_type, ProjectType::Go);
}

#[test]
fn test_detect_nx_monorepo() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // Create Nx monorepo files
    fs::write(path.join("nx.json"), r#"{"extends": "nx/presets/npm.json"}"#).unwrap();
    fs::write(path.join("package.json"), r#"{"name": "monorepo"}"#).unwrap();

    let detector = ProjectDetector::new(path);
    let project_type = detector.detect().unwrap();

    assert_eq!(project_type, ProjectType::NxMonorepo);
}

#[test]
fn test_detect_turborepo() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // Create Turborepo files
    fs::write(path.join("turbo.json"), r#"{"pipeline": {}}"#).unwrap();
    fs::write(path.join("package.json"), r#"{"name": "monorepo"}"#).unwrap();

    let detector = ProjectDetector::new(path);
    let project_type = detector.detect().unwrap();

    assert_eq!(project_type, ProjectType::Turborepo);
}

#[test]
fn test_setup_creates_config() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // Create a Rust project
    fs::write(path.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

    // Run setup
    let options = SetupOptions { force: true, dry_run: false, non_interactive: true };
    setup_project(path, options).unwrap();

    // Verify .palrun.toml was created
    assert!(path.join(".palrun.toml").exists());

    // Verify content
    let content = fs::read_to_string(path.join(".palrun.toml")).unwrap();
    assert!(content.contains("Palrun Configuration for Rust Project"));
    assert!(content.contains("cargo"));
}

#[test]
fn test_setup_creates_runbooks() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // Create a Next.js project
    fs::write(path.join("package.json"), r#"{"name": "test"}"#).unwrap();
    fs::write(path.join("next.config.js"), "module.exports = {}").unwrap();

    // Run setup
    let options = SetupOptions { force: true, dry_run: false, non_interactive: true };
    setup_project(path, options).unwrap();

    // Verify runbooks directory was created
    let runbooks_dir = path.join(".palrun").join("runbooks");
    assert!(runbooks_dir.exists());

    // Verify sample runbooks were created
    assert!(runbooks_dir.join("deploy.yml").exists());
    assert!(runbooks_dir.join("dev-setup.yml").exists());
}

#[test]
fn test_setup_dry_run() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path();

    // Create a Python project
    fs::write(path.join("pyproject.toml"), "[tool.poetry]\nname = \"test\"").unwrap();

    // Run setup with dry-run
    let options = SetupOptions { force: false, dry_run: true, non_interactive: true };
    setup_project(path, options).unwrap();

    // Verify nothing was created
    assert!(!path.join(".palrun.toml").exists());
    assert!(!path.join(".palrun").exists());
}
