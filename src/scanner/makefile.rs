//! Makefile scanner.
//!
//! Scans Makefiles to discover make targets.

use std::path::Path;

use regex::Regex;

use super::Scanner;
use crate::core::Command;

/// Scanner for Makefile targets.
pub struct MakefileScanner;

impl Scanner for MakefileScanner {
    fn name(&self) -> &str {
        "make"
    }

    fn scan(&self, path: &Path) -> anyhow::Result<Vec<Command>> {
        let makefile_path = match find_makefile(path) {
            Ok(p) => p,
            Err(_) => return Ok(Vec::new()),
        };
        let content = std::fs::read_to_string(&makefile_path)?;

        let targets = parse_makefile_targets(&content);
        let mut commands = Vec::new();

        for target in targets {
            // Skip internal targets (starting with .)
            if target.starts_with('.') {
                continue;
            }

            // Skip special targets
            if is_special_target(&target) {
                continue;
            }

            let cmd = Command::from_make_target(&target, Some(path.to_path_buf()));
            commands.push(cmd);
        }

        Ok(commands)
    }
}

/// Find the Makefile in the given directory.
fn find_makefile(path: &Path) -> anyhow::Result<std::path::PathBuf> {
    for name in &["GNUmakefile", "Makefile", "makefile"] {
        let makefile_path = path.join(name);
        if makefile_path.exists() {
            return Ok(makefile_path);
        }
    }

    anyhow::bail!("No Makefile found in {:?}", path)
}

/// Parse Makefile content and extract target names.
fn parse_makefile_targets(content: &str) -> Vec<String> {
    let mut targets = Vec::new();

    // Match lines like "target: deps" or "target:"
    // But not variable assignments like "VAR = value"
    let target_re = Regex::new(r"^([a-zA-Z_][a-zA-Z0-9_.-]*):\s*").unwrap();

    // Track .PHONY targets for prioritization
    let mut phony_targets: Vec<String> = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Check for .PHONY declaration
        if line.starts_with(".PHONY:") {
            let phony_str = line.strip_prefix(".PHONY:").unwrap_or("").trim();
            phony_targets.extend(phony_str.split_whitespace().map(String::from));
            continue;
        }

        // Match regular targets
        if let Some(captures) = target_re.captures(line) {
            let target = captures.get(1).unwrap().as_str().to_string();

            // Avoid duplicates
            if !targets.contains(&target) {
                targets.push(target);
            }
        }
    }

    // Sort: phony targets first (they're typically the user-facing ones)
    targets.sort_by(|a, b| {
        let a_phony = phony_targets.contains(a);
        let b_phony = phony_targets.contains(b);

        match (a_phony, b_phony) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.cmp(b),
        }
    });

    targets
}

/// Check if a target is a special Make target.
fn is_special_target(target: &str) -> bool {
    matches!(
        target,
        "FORCE"
            | "MAKEFLAGS"
            | "SHELL"
            | "MAKEFILE_LIST"
            | ".DEFAULT"
            | ".DELETE_ON_ERROR"
            | ".EXPORT_ALL_VARIABLES"
            | ".IGNORE"
            | ".INTERMEDIATE"
            | ".LOW_RESOLUTION_TIME"
            | ".NOTPARALLEL"
            | ".ONESHELL"
            | ".PHONY"
            | ".POSIX"
            | ".PRECIOUS"
            | ".SECONDARY"
            | ".SECONDEXPANSION"
            | ".SILENT"
            | ".SUFFIXES"
    )
}

/// Extract description from comment above target (if any).
#[allow(dead_code)]
fn extract_target_description(content: &str, target: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with(&format!("{target}:")) {
            // Check if previous line is a comment
            if i > 0 {
                let prev_line = lines[i - 1].trim();
                if prev_line.starts_with('#') {
                    let description = prev_line.trim_start_matches('#').trim();
                    if !description.is_empty() {
                        return Some(description.to_string());
                    }
                }
            }
            break;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_makefile() {
        let content = r#"
.PHONY: build test clean

build:
	cargo build

test:
	cargo test

clean:
	rm -rf target
"#;

        let targets = parse_makefile_targets(content);

        assert!(targets.contains(&"build".to_string()));
        assert!(targets.contains(&"test".to_string()));
        assert!(targets.contains(&"clean".to_string()));
    }

    #[test]
    fn test_skip_internal_targets() {
        let content = r#"
.PHONY: all

all: .internal
	echo "done"

.internal:
	echo "internal"
"#;

        let targets = parse_makefile_targets(content);

        assert!(targets.contains(&"all".to_string()));
        // .internal should not be in targets (filtered in scan())
    }

    #[test]
    fn test_skip_variable_assignments() {
        let content = r#"
CC = gcc
CFLAGS = -Wall

build:
	$(CC) $(CFLAGS) -o app main.c
"#;

        let targets = parse_makefile_targets(content);

        assert!(targets.contains(&"build".to_string()));
        assert!(!targets.contains(&"CC".to_string()));
        assert!(!targets.contains(&"CFLAGS".to_string()));
    }

    #[test]
    fn test_phony_targets_first() {
        let content = r#"
.PHONY: build test

internal-build:
	echo "internal"

build:
	cargo build

test:
	cargo test

helper:
	echo "helper"
"#;

        let targets = parse_makefile_targets(content);

        // Phony targets should come first
        let build_pos = targets.iter().position(|t| t == "build").unwrap();
        let test_pos = targets.iter().position(|t| t == "test").unwrap();
        let internal_pos = targets.iter().position(|t| t == "internal-build").unwrap();

        assert!(build_pos < internal_pos);
        assert!(test_pos < internal_pos);
    }

    #[test]
    fn test_is_special_target() {
        assert!(is_special_target("FORCE"));
        assert!(is_special_target(".PHONY"));
        assert!(is_special_target("SHELL"));

        assert!(!is_special_target("build"));
        assert!(!is_special_target("test"));
        assert!(!is_special_target("clean"));
    }

    #[test]
    fn test_makefile_scanner_name() {
        let scanner = MakefileScanner;
        assert_eq!(scanner.name(), "make");
    }

    #[test]
    fn test_extract_description() {
        let content = r#"
# Build the project
build:
	cargo build

test:
	cargo test
"#;

        let desc = extract_target_description(content, "build");
        assert_eq!(desc, Some("Build the project".to_string()));

        let desc = extract_target_description(content, "test");
        assert!(desc.is_none());
    }
}
