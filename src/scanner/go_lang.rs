//! Go project scanner.
//!
//! Scans go.mod to discover Go project commands.

use std::path::Path;

use super::Scanner;
use crate::core::{Command, CommandSource};

/// Scanner for Go projects.
pub struct GoScanner;

impl Scanner for GoScanner {
    fn name(&self) -> &str {
        "go"
    }

    fn scan(&self, dir: &Path) -> anyhow::Result<Vec<Command>> {
        let mut commands = Vec::new();

        // Check for go.mod
        let go_mod_path = dir.join("go.mod");
        if !go_mod_path.exists() {
            return Ok(commands);
        }

        // Parse go.mod to extract module name
        let module_name = parse_go_mod(&go_mod_path)?;
        let source = CommandSource::GoMod(go_mod_path.clone());

        // Basic Go commands
        commands.push(
            Command::new("go build", "go build")
                .with_description(format!("Build {module_name}"))
                .with_source(source.clone())
                .with_tags(vec!["go".to_string(), "build".to_string()]),
        );

        commands.push(
            Command::new("go build ./...", "go build ./...")
                .with_description("Build all packages")
                .with_source(source.clone())
                .with_tags(vec!["go".to_string(), "build".to_string()]),
        );

        commands.push(
            Command::new("go test ./...", "go test ./...")
                .with_description("Run all tests")
                .with_source(source.clone())
                .with_tags(vec!["go".to_string(), "test".to_string()]),
        );

        commands.push(
            Command::new("go run .", "go run .")
                .with_description(format!("Run {module_name}"))
                .with_source(source.clone())
                .with_tags(vec!["go".to_string(), "run".to_string()]),
        );

        commands.push(
            Command::new("go mod tidy", "go mod tidy")
                .with_description("Tidy module dependencies")
                .with_source(source.clone())
                .with_tags(vec!["go".to_string(), "mod".to_string()]),
        );

        commands.push(
            Command::new("go mod download", "go mod download")
                .with_description("Download module dependencies")
                .with_source(source.clone())
                .with_tags(vec!["go".to_string(), "mod".to_string()]),
        );

        commands.push(
            Command::new("go vet ./...", "go vet ./...")
                .with_description("Examine code for suspicious constructs")
                .with_source(source.clone())
                .with_tags(vec!["go".to_string(), "lint".to_string()]),
        );

        commands.push(
            Command::new("go fmt ./...", "go fmt ./...")
                .with_description("Format all Go source files")
                .with_source(source.clone())
                .with_tags(vec!["go".to_string(), "format".to_string()]),
        );

        // Look for main packages in cmd/ directory
        let cmd_dir = dir.join("cmd");
        if cmd_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&cmd_dir) {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if path.is_dir() {
                        // Check if this directory contains a main.go or any .go file
                        if has_go_files(&path) {
                            if let Some(cmd_name) = path.file_name().and_then(|n| n.to_str()) {
                                commands.push(
                                    Command::new(
                                        format!("go run ./cmd/{cmd_name}"),
                                        format!("go run ./cmd/{cmd_name}"),
                                    )
                                    .with_description(format!("Run {cmd_name}"))
                                    .with_source(source.clone())
                                    .with_tags(vec![
                                        "go".to_string(),
                                        "run".to_string(),
                                        cmd_name.to_string(),
                                    ]),
                                );
                            }
                        }
                    }
                }
            }
        }

        // Check for main.go in project root (only add "go run ." if it exists)
        // We already added "go run ." above, but let's check if main.go exists
        // to potentially add a more specific command
        let main_go_path = dir.join("main.go");
        if main_go_path.exists() {
            commands.push(
                Command::new("go run main.go", "go run main.go")
                    .with_description("Run main.go directly")
                    .with_source(source.clone())
                    .with_tags(vec!["go".to_string(), "run".to_string()]),
            );
        }

        Ok(commands)
    }
}

/// Parse go.mod file to extract the module name.
fn parse_go_mod(path: &Path) -> anyhow::Result<String> {
    let content = std::fs::read_to_string(path)?;

    // go.mod format is simple text:
    // module github.com/user/project
    // go 1.21
    // ...
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("module ") {
            let module_name = line.strip_prefix("module ").map(|s| s.trim()).unwrap_or("unknown");
            return Ok(module_name.to_string());
        }
    }

    // If no module line found, return a default
    Ok("go-project".to_string())
}

/// Check if a directory contains any .go files.
fn has_go_files(dir: &Path) -> bool {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "go" {
                        return true;
                    }
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_go_scanner_name() {
        let scanner = GoScanner;
        assert_eq!(scanner.name(), "go");
    }

    #[test]
    fn test_scan_no_go_mod() {
        let scanner = GoScanner;
        let temp_dir = TempDir::new().unwrap();
        let commands = scanner.scan(temp_dir.path()).unwrap();
        assert!(commands.is_empty());
    }

    #[test]
    fn test_scan_simple_go_project() {
        let scanner = GoScanner;
        let temp_dir = TempDir::new().unwrap();

        // Create go.mod
        let go_mod_content = r"module github.com/example/myapp

go 1.21

require (
    github.com/gin-gonic/gin v1.9.0
)
";
        fs::write(temp_dir.path().join("go.mod"), go_mod_content).unwrap();

        let commands = scanner.scan(temp_dir.path()).unwrap();

        // Should have basic commands
        assert!(!commands.is_empty());

        // Check for expected commands
        let command_names: Vec<&str> = commands.iter().map(|c| c.name.as_str()).collect();
        assert!(command_names.contains(&"go build"));
        assert!(command_names.contains(&"go build ./..."));
        assert!(command_names.contains(&"go test ./..."));
        assert!(command_names.contains(&"go run ."));
        assert!(command_names.contains(&"go mod tidy"));
        assert!(command_names.contains(&"go mod download"));
        assert!(command_names.contains(&"go vet ./..."));
        assert!(command_names.contains(&"go fmt ./..."));
    }

    #[test]
    fn test_scan_with_cmd_directory() {
        let scanner = GoScanner;
        let temp_dir = TempDir::new().unwrap();

        // Create go.mod
        fs::write(
            temp_dir.path().join("go.mod"),
            "module github.com/example/multi-cmd\n\ngo 1.21\n",
        )
        .unwrap();

        // Create cmd directory with subdirectories
        let cmd_dir = temp_dir.path().join("cmd");
        fs::create_dir(&cmd_dir).unwrap();

        // Create cmd/server with a main.go
        let server_dir = cmd_dir.join("server");
        fs::create_dir(&server_dir).unwrap();
        fs::write(server_dir.join("main.go"), "package main\nfunc main() {}").unwrap();

        // Create cmd/client with a main.go
        let client_dir = cmd_dir.join("client");
        fs::create_dir(&client_dir).unwrap();
        fs::write(client_dir.join("main.go"), "package main\nfunc main() {}").unwrap();

        let commands = scanner.scan(temp_dir.path()).unwrap();

        let command_names: Vec<&str> = commands.iter().map(|c| c.name.as_str()).collect();
        assert!(command_names.contains(&"go run ./cmd/server"));
        assert!(command_names.contains(&"go run ./cmd/client"));
    }

    #[test]
    fn test_scan_with_main_go() {
        let scanner = GoScanner;
        let temp_dir = TempDir::new().unwrap();

        // Create go.mod
        fs::write(temp_dir.path().join("go.mod"), "module github.com/example/simple\n\ngo 1.21\n")
            .unwrap();

        // Create main.go in root
        fs::write(temp_dir.path().join("main.go"), "package main\nfunc main() {}").unwrap();

        let commands = scanner.scan(temp_dir.path()).unwrap();

        let command_names: Vec<&str> = commands.iter().map(|c| c.name.as_str()).collect();
        assert!(command_names.contains(&"go run main.go"));
    }

    #[test]
    fn test_parse_go_mod_simple() {
        let temp_dir = TempDir::new().unwrap();
        let go_mod_path = temp_dir.path().join("go.mod");

        fs::write(&go_mod_path, "module github.com/user/project\n\ngo 1.21\n").unwrap();

        let module_name = parse_go_mod(&go_mod_path).unwrap();
        assert_eq!(module_name, "github.com/user/project");
    }

    #[test]
    fn test_parse_go_mod_with_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let go_mod_path = temp_dir.path().join("go.mod");

        let content = r"module example.com/my-app

go 1.22

require (
    github.com/gin-gonic/gin v1.9.0
    github.com/lib/pq v1.10.9
)

require (
    github.com/bytedance/sonic v1.9.1 // indirect
)
";
        fs::write(&go_mod_path, content).unwrap();

        let module_name = parse_go_mod(&go_mod_path).unwrap();
        assert_eq!(module_name, "example.com/my-app");
    }

    #[test]
    fn test_parse_go_mod_no_module_line() {
        let temp_dir = TempDir::new().unwrap();
        let go_mod_path = temp_dir.path().join("go.mod");

        // Invalid go.mod without module line
        fs::write(&go_mod_path, "go 1.21\n").unwrap();

        let module_name = parse_go_mod(&go_mod_path).unwrap();
        assert_eq!(module_name, "go-project");
    }

    #[test]
    fn test_has_go_files() {
        let temp_dir = TempDir::new().unwrap();

        // Empty directory
        assert!(!has_go_files(temp_dir.path()));

        // Directory with non-go files
        fs::write(temp_dir.path().join("README.md"), "# README").unwrap();
        assert!(!has_go_files(temp_dir.path()));

        // Directory with go files
        fs::write(temp_dir.path().join("main.go"), "package main").unwrap();
        assert!(has_go_files(temp_dir.path()));
    }

    #[test]
    fn test_command_source_is_go_mod() {
        let scanner = GoScanner;
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("go.mod"), "module test\n\ngo 1.21\n").unwrap();

        let commands = scanner.scan(temp_dir.path()).unwrap();

        for cmd in &commands {
            match &cmd.source {
                CommandSource::GoMod(_) => {}
                _ => panic!("Expected GoMod source, got {:?}", cmd.source),
            }
        }
    }

    #[test]
    fn test_cmd_directory_without_go_files_skipped() {
        let scanner = GoScanner;
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("go.mod"), "module test\n\ngo 1.21\n").unwrap();

        // Create cmd directory with a subdirectory that has no .go files
        let cmd_dir = temp_dir.path().join("cmd");
        fs::create_dir(&cmd_dir).unwrap();

        let empty_cmd = cmd_dir.join("empty");
        fs::create_dir(&empty_cmd).unwrap();
        fs::write(empty_cmd.join("README.md"), "# Empty").unwrap();

        let commands = scanner.scan(temp_dir.path()).unwrap();

        let command_names: Vec<&str> = commands.iter().map(|c| c.name.as_str()).collect();
        assert!(!command_names.contains(&"go run ./cmd/empty"));
    }
}
