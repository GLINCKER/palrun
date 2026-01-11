//! CLI Integration Tests
//!
//! Tests the command-line interface end-to-end.

use assert_cmd::Command;
use assert_fs::prelude::*;
use predicates::prelude::*;

/// Get the binary to test.
fn palrun() -> Command {
    Command::cargo_bin("palrun").unwrap()
}

// ============================================================================
// Help & Version Tests
// ============================================================================

#[test]
fn test_help_flag() {
    palrun()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("AI command palette"));
}

#[test]
fn test_short_help_flag() {
    palrun().arg("-h").assert().success().stdout(predicate::str::contains("Usage:"));
}

#[test]
fn test_version_flag() {
    palrun()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_short_version_flag() {
    palrun().arg("-V").assert().success().stdout(predicate::str::contains("palrun"));
}

// ============================================================================
// List Command Tests
// ============================================================================

#[test]
fn test_list_command_help() {
    palrun()
        .args(["list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("List all available commands"));
}

#[test]
fn test_list_in_current_dir() {
    // Should work in any directory (may find Cargo commands here)
    palrun().arg("list").assert().success();
}

#[test]
fn test_list_with_json_output() {
    palrun()
        .args(["list", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["));
}

// ============================================================================
// Scan Command Tests
// ============================================================================

#[test]
fn test_scan_command_help() {
    palrun().args(["scan", "--help"]).assert().success().stdout(predicate::str::contains("Scan"));
}

#[test]
fn test_scan_current_directory() {
    // Scanning the palrun project should find Cargo commands
    palrun()
        .arg("scan")
        .assert()
        .success()
        .stdout(predicate::str::contains("cargo").or(predicate::str::contains("Discovered")));
}

// ============================================================================
// Project Detection Tests
// ============================================================================

#[test]
fn test_detects_cargo_project() {
    // Running in the palrun project should detect Cargo.toml
    palrun().arg("list").assert().success().stdout(predicate::str::contains("cargo"));
}

#[test]
fn test_list_with_filter() {
    // Filter by source type
    palrun().args(["list", "--source", "cargo"]).assert().success();
}

// ============================================================================
// Fixture-Based Tests
// ============================================================================

#[test]
fn test_scan_npm_project() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create a minimal package.json
    temp.child("package.json")
        .write_str(r#"{"name": "test", "scripts": {"build": "echo build", "test": "echo test"}}"#)
        .unwrap();

    palrun()
        .arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("npm run build").or(predicate::str::contains("build")));

    temp.close().unwrap();
}

#[test]
fn test_scan_makefile_project() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create a minimal Makefile
    temp.child("Makefile")
        .write_str(".PHONY: build test\n\nbuild:\n\techo building\n\ntest:\n\techo testing\n")
        .unwrap();

    palrun()
        .arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("make build").or(predicate::str::contains("build")));

    temp.close().unwrap();
}

#[test]
fn test_scan_taskfile_project() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create a minimal Taskfile.yml
    temp.child("Taskfile.yml")
        .write_str("version: '3'\ntasks:\n  build:\n    desc: Build the project\n    cmds:\n      - echo build\n")
        .unwrap();

    palrun()
        .arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("task build").or(predicate::str::contains("build")));

    temp.close().unwrap();
}

#[test]
fn test_scan_docker_compose_project() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create a minimal docker-compose.yml
    temp.child("docker-compose.yml")
        .write_str(
            "version: '3'\nservices:\n  web:\n    image: nginx\n  db:\n    image: postgres\n",
        )
        .unwrap();

    palrun()
        .arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("docker").or(predicate::str::contains("compose")));

    temp.close().unwrap();
}

#[test]
fn test_scan_go_project() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create a minimal go.mod
    temp.child("go.mod").write_str("module example.com/test\n\ngo 1.21\n").unwrap();

    palrun()
        .arg("list")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("go").or(predicate::str::is_empty().not()));

    temp.close().unwrap();
}

#[test]
fn test_scan_python_project() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create a minimal pyproject.toml with Poetry
    temp.child("pyproject.toml")
        .write_str("[tool.poetry]\nname = \"test\"\nversion = \"0.1.0\"\n\n[tool.poetry.scripts]\nbuild = \"build:main\"\n")
        .unwrap();

    palrun().arg("list").current_dir(temp.path()).assert().success();

    temp.close().unwrap();
}

// ============================================================================
// Empty Project Tests
// ============================================================================

#[test]
fn test_scan_empty_directory() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Empty directory should return success but no commands
    palrun().arg("list").current_dir(temp.path()).assert().success();

    temp.close().unwrap();
}

// ============================================================================
// Multi-Scanner Tests
// ============================================================================

#[test]
fn test_scan_multi_config_project() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create both package.json and Makefile
    temp.child("package.json")
        .write_str(r#"{"name": "test", "scripts": {"dev": "vite"}}"#)
        .unwrap();

    temp.child("Makefile").write_str(".PHONY: deploy\n\ndeploy:\n\techo deploying\n").unwrap();

    let output = palrun().arg("list").current_dir(temp.path()).assert().success();

    // Should find commands from both sources
    let stdout = String::from_utf8_lossy(&output.get_output().stdout);
    // Either npm or make commands should be present
    assert!(
        stdout.contains("npm")
            || stdout.contains("make")
            || stdout.contains("dev")
            || stdout.contains("deploy"),
        "Expected to find npm or make commands, got: {}",
        stdout
    );

    temp.close().unwrap();
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_invalid_subcommand() {
    palrun().arg("invalid-command-that-does-not-exist").assert().failure();
}

#[test]
fn test_invalid_flag() {
    palrun().arg("--invalid-flag-xyz").assert().failure();
}
