//! Performance Tests for Palrun
//!
//! These tests verify that performance targets are met.
//! Run with: `cargo test --test performance_tests`

use std::process::Command;
use std::time::{Duration, Instant};

/// Test that startup time is acceptable (< 100ms).
#[test]
fn test_startup_time_help() {
    let binary = std::env::current_dir()
        .unwrap()
        .join("target")
        .join(if cfg!(debug_assertions) { "debug" } else { "release" })
        .join("palrun");

    // Skip if binary doesn't exist
    if !binary.exists() {
        eprintln!("Binary not found at {:?}, skipping test", binary);
        return;
    }

    // Warm up run
    let _ = Command::new(&binary).arg("--help").output();

    // Measure 5 runs
    let mut times = Vec::with_capacity(5);
    for _ in 0..5 {
        let start = Instant::now();
        let output = Command::new(&binary).arg("--help").output().expect("Failed to execute");
        let elapsed = start.elapsed();
        assert!(output.status.success(), "Help command should succeed");
        times.push(elapsed);
    }

    let avg = times.iter().sum::<Duration>() / times.len() as u32;
    let max_acceptable = Duration::from_millis(100);

    assert!(
        avg < max_acceptable,
        "Average startup time {:?} exceeds acceptable {:?}",
        avg,
        max_acceptable
    );

    println!("Average startup time: {:?}", avg);
}

/// Test that list command completes in reasonable time.
#[test]
fn test_list_performance() {
    let binary = std::env::current_dir()
        .unwrap()
        .join("target")
        .join(if cfg!(debug_assertions) { "debug" } else { "release" })
        .join("palrun");

    if !binary.exists() {
        eprintln!("Binary not found, skipping test");
        return;
    }

    let start = Instant::now();
    let output = Command::new(&binary).arg("list").output().expect("Failed to execute");
    let elapsed = start.elapsed();

    assert!(output.status.success(), "List command should succeed");

    // List should complete in under 500ms
    let max_acceptable = Duration::from_millis(500);
    assert!(
        elapsed < max_acceptable,
        "List command took {:?}, exceeds acceptable {:?}",
        elapsed,
        max_acceptable
    );

    println!("List command time: {:?}", elapsed);
}

/// Test that version command is fast.
#[test]
fn test_version_fast() {
    let binary = std::env::current_dir()
        .unwrap()
        .join("target")
        .join(if cfg!(debug_assertions) { "debug" } else { "release" })
        .join("palrun");

    if !binary.exists() {
        eprintln!("Binary not found, skipping test");
        return;
    }

    // Warm up
    let _ = Command::new(&binary).arg("--version").output();

    let start = Instant::now();
    let output = Command::new(&binary).arg("--version").output().expect("Failed to execute");
    let elapsed = start.elapsed();

    assert!(output.status.success());

    // Version should be fast - allow more time for debug builds
    let max_acceptable = if cfg!(debug_assertions) {
        Duration::from_millis(500) // Debug builds are slower
    } else {
        Duration::from_millis(50) // Release should be instant
    };
    assert!(
        elapsed < max_acceptable,
        "Version command took {:?}, exceeds {:?}",
        elapsed,
        max_acceptable
    );

    println!("Version command time: {:?}", elapsed);
}

/// Test JSON output is well-formed and efficient.
#[test]
fn test_json_output_efficiency() {
    let binary = std::env::current_dir()
        .unwrap()
        .join("target")
        .join(if cfg!(debug_assertions) { "debug" } else { "release" })
        .join("palrun");

    if !binary.exists() {
        eprintln!("Binary not found, skipping test");
        return;
    }

    let output = Command::new(&binary)
        .args(["list", "--format", "json"])
        .output()
        .expect("Failed to execute");

    assert!(output.status.success());

    // Verify JSON is valid
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("Output should be valid JSON");

    // Should be an array
    assert!(json.is_array(), "JSON output should be an array");

    // Output should not be excessively large (< 1MB for reasonable projects)
    let size = output.stdout.len();
    let max_size = 1024 * 1024; // 1MB
    assert!(size < max_size, "JSON output size {} exceeds max {}", size, max_size);

    println!("JSON output size: {} bytes", size);
}

/// Test binary size is reasonable.
#[test]
fn test_binary_size() {
    let binary = std::env::current_dir().unwrap().join("target").join("release").join("palrun");

    if !binary.exists() {
        eprintln!("Release binary not found, skipping test");
        return;
    }

    let metadata = std::fs::metadata(&binary).expect("Failed to get metadata");
    let size = metadata.len();

    // Binary should be < 20MB (acceptable), ideally < 10MB (target)
    let max_acceptable = 20 * 1024 * 1024; // 20MB
    assert!(
        size < max_acceptable,
        "Binary size {} exceeds max acceptable {}",
        size,
        max_acceptable
    );

    println!("Binary size: {} bytes ({:.1} MB)", size, size as f64 / 1024.0 / 1024.0);
}
