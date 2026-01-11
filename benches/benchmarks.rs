//! Performance benchmarks for Palrun.
//!
//! This module contains benchmarks for:
//! - Scanner performance (package.json, Cargo.toml)
//! - Fuzzy search performance with large command sets
//! - Command execution startup time
//!
//! Run with: `cargo bench`

use std::path::PathBuf;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use palrun::core::{Command, CommandRegistry, CommandSource};
use palrun::scanner::{CargoScanner, NpmScanner, Scanner};

// ============================================================================
// Mock Data Fixtures
// ============================================================================

mod fixtures {
    use super::*;

    /// Generate a realistic package.json content with scripts.
    pub fn generate_package_json(num_scripts: usize) -> String {
        let mut scripts = Vec::with_capacity(num_scripts);

        // Common npm script names
        let common_scripts = [
            ("dev", "vite dev"),
            ("build", "vite build"),
            ("test", "vitest run"),
            ("test:watch", "vitest"),
            ("test:coverage", "vitest run --coverage"),
            ("lint", "eslint . --ext .ts,.tsx"),
            ("lint:fix", "eslint . --ext .ts,.tsx --fix"),
            ("format", "prettier --write ."),
            ("format:check", "prettier --check ."),
            ("typecheck", "tsc --noEmit"),
            ("start", "node dist/index.js"),
            ("preview", "vite preview"),
            ("clean", "rm -rf dist node_modules/.cache"),
            ("prepare", "husky install"),
            ("precommit", "lint-staged"),
            ("release", "semantic-release"),
            ("docs", "typedoc --out docs src/index.ts"),
            ("docs:serve", "serve docs"),
            ("ci", "npm run lint && npm run test && npm run build"),
            ("deploy", "npm run build && npm run deploy:prod"),
        ];

        // Add common scripts first
        for (i, (name, cmd)) in common_scripts.iter().enumerate() {
            if i >= num_scripts {
                break;
            }
            scripts.push(format!(r#"    "{}": "{}""#, name, cmd));
        }

        // Add generated scripts if we need more
        for i in common_scripts.len()..num_scripts {
            scripts.push(format!(r#"    "script-{}": "echo running script {}""#, i, i));
        }

        format!(
            r#"{{
  "name": "benchmark-project",
  "version": "1.0.0",
  "description": "A benchmark test project",
  "scripts": {{
{}
  }},
  "dependencies": {{
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  }},
  "devDependencies": {{
    "typescript": "^5.0.0",
    "vite": "^5.0.0",
    "vitest": "^1.0.0",
    "eslint": "^8.0.0"
  }}
}}"#,
            scripts.join(",\n")
        )
    }

    /// Generate a realistic Cargo.toml content.
    pub fn generate_cargo_toml(num_features: usize, num_bins: usize) -> String {
        let mut features = Vec::with_capacity(num_features);
        let mut bins = Vec::with_capacity(num_bins);

        // Generate features
        features.push("default = [\"std\"]".to_string());
        features.push("std = []".to_string());
        features.push("full = [\"std\", \"async\", \"serde\"]".to_string());
        features.push("async = [\"tokio\"]".to_string());
        features.push("serde = [\"dep:serde\"]".to_string());

        for i in 5..num_features {
            features.push(format!("feature-{} = []", i));
        }

        // Generate bins
        for i in 0..num_bins {
            bins.push(format!(
                r#"[[bin]]
name = "bin-{}"
path = "src/bin/bin{}.rs""#,
                i, i
            ));
        }

        format!(
            r#"[package]
name = "benchmark-crate"
version = "0.1.0"
edition = "2021"
description = "A benchmark test crate"

[dependencies]
tokio = {{ version = "1", optional = true }}
serde = {{ version = "1", optional = true }}
anyhow = "1"

[features]
{}

{}
"#,
            features.join("\n"),
            bins.join("\n\n")
        )
    }

    /// Generate a large set of realistic commands for fuzzy search benchmarks.
    pub fn generate_commands(count: usize) -> Vec<Command> {
        let mut commands = Vec::with_capacity(count);

        // Common command patterns
        let patterns = [
            ("npm run", "build", CommandSource::PackageJson(PathBuf::from("."))),
            ("npm run", "test", CommandSource::PackageJson(PathBuf::from("."))),
            ("npm run", "lint", CommandSource::PackageJson(PathBuf::from("."))),
            ("npm run", "dev", CommandSource::PackageJson(PathBuf::from("."))),
            ("yarn", "build", CommandSource::PackageJson(PathBuf::from("."))),
            ("pnpm", "test", CommandSource::PackageJson(PathBuf::from("."))),
            ("cargo", "build", CommandSource::Cargo(PathBuf::from("."))),
            ("cargo", "test", CommandSource::Cargo(PathBuf::from("."))),
            ("cargo", "run", CommandSource::Cargo(PathBuf::from("."))),
            ("cargo", "clippy", CommandSource::Cargo(PathBuf::from("."))),
            ("make", "all", CommandSource::Makefile(PathBuf::from("."))),
            ("make", "clean", CommandSource::Makefile(PathBuf::from("."))),
            ("docker", "build", CommandSource::DockerCompose(PathBuf::from("."))),
            ("docker-compose", "up", CommandSource::DockerCompose(PathBuf::from("."))),
            ("go", "build", CommandSource::GoMod(PathBuf::from("."))),
            ("go", "test", CommandSource::GoMod(PathBuf::from("."))),
        ];

        let workspaces = [
            "frontend", "backend", "api", "core", "lib", "cli", "web", "mobile", "shared", "utils",
        ];

        let tags =
            ["build", "test", "lint", "deploy", "dev", "ci", "format", "docs", "release", "clean"];

        for i in 0..count {
            let (prefix, suffix, source) = &patterns[i % patterns.len()];
            let workspace = workspaces[i % workspaces.len()];
            let tag = tags[i % tags.len()];

            let name = if i < patterns.len() * 2 {
                format!("{} {}", prefix, suffix)
            } else {
                format!("{} {}:{}", prefix, suffix, i)
            };

            let cmd = format!("{} {}", prefix, suffix);
            let description = format!("{}s the {} project (variant {})", suffix, workspace, i);

            let command = Command::new(&name, &cmd)
                .with_description(description)
                .with_source(source.clone())
                .with_workspace(workspace)
                .with_tag(tag)
                .with_tag(workspace);

            commands.push(command);
        }

        commands
    }

    /// Search patterns of varying complexity for benchmarking.
    pub fn search_patterns() -> Vec<&'static str> {
        vec![
            "",                  // Empty (return all)
            "b",                 // Single char
            "bu",                // Two chars
            "bui",               // Three chars
            "buil",              // Four chars
            "build",             // Exact word
            "npm build",         // Two words
            "nrb",               // Fuzzy pattern (npm run build)
            "cgt",               // Fuzzy pattern (cargo test)
            "frontend",          // Workspace name
            "test lint",         // Multiple tags
            "api backend build", // Complex query
        ]
    }
}

// ============================================================================
// Scanner Benchmarks
// ============================================================================

fn bench_npm_scanner(c: &mut Criterion) {
    let mut group = c.benchmark_group("scanner/npm");

    // Create temp directories with package.json files
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    for num_scripts in [10, 50, 100, 200].iter() {
        let project_dir = temp_dir.path().join(format!("project_{}", num_scripts));
        std::fs::create_dir_all(&project_dir).expect("Failed to create project dir");

        let package_json = fixtures::generate_package_json(*num_scripts);
        std::fs::write(project_dir.join("package.json"), &package_json)
            .expect("Failed to write package.json");

        group.throughput(Throughput::Elements(*num_scripts as u64));
        group.bench_with_input(
            BenchmarkId::new("scan_package_json", num_scripts),
            num_scripts,
            |b, _| {
                let scanner = NpmScanner;
                b.iter(|| {
                    let result = scanner.scan(black_box(&project_dir));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

fn bench_cargo_scanner(c: &mut Criterion) {
    let mut group = c.benchmark_group("scanner/cargo");

    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    // Benchmark with different numbers of features and binaries
    let configs = [
        (5, 2),   // Small project
        (10, 5),  // Medium project
        (20, 10), // Large project with many features
        (30, 15), // Very large project
    ];

    for (num_features, num_bins) in configs.iter() {
        let project_dir = temp_dir.path().join(format!("crate_{}_{}", num_features, num_bins));
        std::fs::create_dir_all(&project_dir).expect("Failed to create project dir");

        let cargo_toml = fixtures::generate_cargo_toml(*num_features, *num_bins);
        std::fs::write(project_dir.join("Cargo.toml"), &cargo_toml)
            .expect("Failed to write Cargo.toml");

        let label = format!("{}features_{}bins", num_features, num_bins);
        group.bench_with_input(
            BenchmarkId::new("scan_cargo_toml", &label),
            &(num_features, num_bins),
            |b, _| {
                let scanner = CargoScanner;
                b.iter(|| {
                    let result = scanner.scan(black_box(&project_dir));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Fuzzy Search Benchmarks
// ============================================================================

fn bench_fuzzy_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("fuzzy_search");

    // Test with different command counts
    for cmd_count in [100, 500, 1000, 2000, 5000].iter() {
        let commands = fixtures::generate_commands(*cmd_count);
        let mut registry = CommandRegistry::new();
        for cmd in commands {
            registry.add(cmd);
        }

        group.throughput(Throughput::Elements(*cmd_count as u64));

        // Benchmark each search pattern
        for pattern in fixtures::search_patterns() {
            let pattern_name = if pattern.is_empty() {
                "empty"
            } else if pattern.len() <= 3 {
                "short"
            } else if pattern.contains(' ') {
                "multi_word"
            } else {
                "single_word"
            };

            let bench_name = format!("{}cmds_{}", cmd_count, pattern_name);
            group.bench_with_input(
                BenchmarkId::new("search", &bench_name),
                &pattern,
                |b, &pattern| {
                    b.iter(|| {
                        let results = registry.search(black_box(pattern));
                        black_box(results)
                    });
                },
            );
        }
    }

    group.finish();
}

fn bench_fuzzy_search_with_context(c: &mut Criterion) {
    let mut group = c.benchmark_group("fuzzy_search_context");

    let commands = fixtures::generate_commands(1000);
    let mut registry = CommandRegistry::new();
    for cmd in commands {
        registry.add(cmd);
    }

    // Create a context for context-aware search
    let cwd = PathBuf::from("/projects/frontend");
    let project_root = PathBuf::from("/projects");
    let context = palrun::core::CommandContext::new(&cwd, &project_root);

    for pattern in fixtures::search_patterns() {
        let pattern_name = if pattern.is_empty() {
            "empty"
        } else if pattern.len() <= 3 {
            "short"
        } else {
            "complex"
        };

        group.bench_with_input(
            BenchmarkId::new("search_with_context", pattern_name),
            &pattern,
            |b, &pattern| {
                b.iter(|| {
                    let results = registry.search_with_context(black_box(pattern), &context);
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

fn bench_registry_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry");

    // Benchmark adding commands
    group.bench_function("add_1000_commands", |b| {
        b.iter(|| {
            let mut registry = CommandRegistry::new();
            for cmd in fixtures::generate_commands(1000) {
                registry.add(black_box(cmd));
            }
            black_box(registry)
        });
    });

    // Benchmark clearing registry
    let commands = fixtures::generate_commands(1000);
    group.bench_function("clear_1000_commands", |b| {
        b.iter_batched(
            || {
                let mut registry = CommandRegistry::new();
                for cmd in commands.clone() {
                    registry.add(cmd);
                }
                registry
            },
            |mut registry| {
                registry.clear();
                black_box(registry)
            },
            criterion::BatchSize::SmallInput,
        );
    });

    // Benchmark get_by_index
    let mut registry = CommandRegistry::new();
    for cmd in fixtures::generate_commands(1000) {
        registry.add(cmd);
    }

    group.bench_function("get_by_index", |b| {
        let indices: Vec<usize> = (0..100).collect();
        b.iter(|| {
            for &idx in &indices {
                let _ = black_box(registry.get_by_index(black_box(idx)));
            }
        });
    });

    group.finish();
}

// ============================================================================
// Command Execution Benchmarks
// ============================================================================

fn bench_command_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("command");

    // Benchmark command creation
    group.bench_function("create_simple", |b| {
        b.iter(|| {
            let cmd = Command::new(black_box("npm run build"), black_box("npm run build"));
            black_box(cmd)
        });
    });

    group.bench_function("create_with_metadata", |b| {
        b.iter(|| {
            let cmd = Command::new(black_box("npm run build"), black_box("npm run build"))
                .with_description("Build the project")
                .with_source(CommandSource::PackageJson(PathBuf::from(".")))
                .with_workspace("frontend")
                .with_tag("build")
                .with_tag("npm")
                .with_env("NODE_ENV", "production");
            black_box(cmd)
        });
    });

    // Benchmark match_text generation
    let cmd = Command::new("npm run build", "npm run build")
        .with_description("Build the project for production")
        .with_tag("build")
        .with_tag("npm")
        .with_tag("frontend");

    group.bench_function("match_text", |b| {
        b.iter(|| {
            let text = cmd.match_text();
            black_box(text)
        });
    });

    group.finish();
}

fn bench_executor_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("executor");

    // Benchmark executor creation
    group.bench_function("create_executor", |b| {
        b.iter(|| {
            let executor = palrun::core::Executor::new();
            black_box(executor)
        });
    });

    // Benchmark executing a trivial command (echo)
    // This measures the startup overhead
    group.bench_function("execute_echo", |b| {
        let executor = palrun::core::Executor::new().capture(true);
        let cmd = Command::new("echo", "echo hello");

        b.iter(|| {
            let result = executor.execute(black_box(&cmd));
            black_box(result)
        });
    });

    // Benchmark executing true (minimal shell command)
    group.bench_function("execute_true", |b| {
        let executor = palrun::core::Executor::new().capture(true);
        let cmd = Command::new("true", "true");

        b.iter(|| {
            let result = executor.execute(black_box(&cmd));
            black_box(result)
        });
    });

    group.finish();
}

// ============================================================================
// Parser Benchmarks
// ============================================================================

fn bench_package_json_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing/package_json");

    for num_scripts in [10, 50, 100, 200].iter() {
        let json = fixtures::generate_package_json(*num_scripts);

        group.throughput(Throughput::Bytes(json.len() as u64));
        group.bench_with_input(BenchmarkId::new("parse", num_scripts), &json, |b, json| {
            b.iter(|| {
                let parsed: serde_json::Value = serde_json::from_str(black_box(json)).unwrap();
                black_box(parsed)
            });
        });
    }

    group.finish();
}

fn bench_cargo_toml_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing/cargo_toml");

    let configs = [(5, 2), (10, 5), (20, 10), (30, 15)];

    for (num_features, num_bins) in configs.iter() {
        let toml_content = fixtures::generate_cargo_toml(*num_features, *num_bins);
        let label = format!("{}f_{}b", num_features, num_bins);

        group.throughput(Throughput::Bytes(toml_content.len() as u64));
        group.bench_with_input(BenchmarkId::new("parse", &label), &toml_content, |b, content| {
            b.iter(|| {
                let parsed: toml::Value = toml::from_str(black_box(content)).unwrap();
                black_box(parsed)
            });
        });
    }

    group.finish();
}

// ============================================================================
// Branch Matching Benchmarks
// ============================================================================

fn bench_branch_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("branch_matching");

    // Command with no branch restrictions
    let cmd_all = Command::new("test", "npm test");

    // Command with exact branch
    let cmd_exact = Command::new("deploy", "npm run deploy").with_branch_pattern("main");

    // Command with wildcard pattern
    let cmd_wildcard = Command::new("feature", "npm run feature").with_branch_pattern("feature/*");

    // Command with multiple patterns
    let cmd_multi = Command::new("release", "npm run release").with_branch_patterns(vec![
        "main".to_string(),
        "release/*".to_string(),
        "hotfix/*".to_string(),
    ]);

    let branches = [
        Some("main"),
        Some("develop"),
        Some("feature/new-feature"),
        Some("release/v1.0.0"),
        Some("hotfix/urgent-fix"),
        None,
    ];

    group.bench_function("no_restrictions", |b| {
        b.iter(|| {
            for branch in &branches {
                let _ = black_box(cmd_all.matches_branch(*branch));
            }
        });
    });

    group.bench_function("exact_match", |b| {
        b.iter(|| {
            for branch in &branches {
                let _ = black_box(cmd_exact.matches_branch(*branch));
            }
        });
    });

    group.bench_function("wildcard_match", |b| {
        b.iter(|| {
            for branch in &branches {
                let _ = black_box(cmd_wildcard.matches_branch(*branch));
            }
        });
    });

    group.bench_function("multiple_patterns", |b| {
        b.iter(|| {
            for branch in &branches {
                let _ = black_box(cmd_multi.matches_branch(*branch));
            }
        });
    });

    group.finish();
}

// ============================================================================
// Criterion Groups and Main
// ============================================================================

criterion_group!(scanner_benches, bench_npm_scanner, bench_cargo_scanner,);

criterion_group!(
    search_benches,
    bench_fuzzy_search,
    bench_fuzzy_search_with_context,
    bench_registry_operations,
);

criterion_group!(command_benches, bench_command_creation, bench_executor_startup,);

criterion_group!(parsing_benches, bench_package_json_parsing, bench_cargo_toml_parsing,);

criterion_group!(misc_benches, bench_branch_matching,);

criterion_main!(scanner_benches, search_benches, command_benches, parsing_benches, misc_benches,);
