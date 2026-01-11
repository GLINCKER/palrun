//! My Scanner Plugin for Palrun
//!
//! This is a template for creating custom scanner plugins.
//! Replace this documentation with your own.

use palrun_plugin_sdk::prelude::*;

/// My custom scanner.
///
/// This scanner discovers commands from project files.
/// Modify this struct to add configuration fields if needed.
#[derive(Default)]
pub struct MyScanner;

impl Scanner for MyScanner {
    /// Returns the scanner name.
    ///
    /// This should match the name in plugin.toml.
    fn name(&self) -> &'static str {
        "my-scanner"
    }

    /// Returns file patterns this scanner handles.
    ///
    /// The host will only provide files matching these patterns.
    /// Supports glob patterns like "*.json" or "build.*".
    fn file_patterns(&self) -> &'static [&'static str] {
        &["Myfile", "*.myext"]
    }

    /// Scans the project and returns discovered commands.
    ///
    /// This is the main entry point of your scanner.
    fn scan(&self, context: &ScanContext) -> Vec<Command> {
        let mut commands = Vec::new();

        // Example: Check for a specific file
        if let Some(content) = context.get_file("Myfile") {
            // Parse the file content and extract commands
            commands.extend(parse_myfile(content));
        }

        // Example: Process all matched files
        for path in context.file_paths() {
            if path.ends_with(".myext") {
                if let Some(content) = context.get_file(path) {
                    commands.extend(parse_myext_file(path, content));
                }
            }
        }

        // Example: Add common commands if project matches
        if context.has_file("Myfile") {
            commands.push(
                Command::new("my-tool init", "my-tool init")
                    .with_description("Initialize my-tool")
                    .with_tag("my-tool")
                    .with_tag("init"),
            );

            commands.push(
                Command::new("my-tool run", "my-tool run")
                    .with_description("Run the default task")
                    .with_tag("my-tool")
                    .with_tag("run"),
            );
        }

        commands
    }

    /// Optional: description for the scanner.
    fn description(&self) -> Option<&'static str> {
        Some("Scans for my-tool project files")
    }

    /// Optional: priority (higher runs first).
    fn priority(&self) -> i32 {
        0
    }
}

/// Parse the main Myfile and extract commands.
fn parse_myfile(content: &str) -> Vec<Command> {
    let mut commands = Vec::new();

    // Example: Parse lines that look like task definitions
    // Customize this for your file format
    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Example: Parse "task: command" format
        if let Some((name, cmd)) = line.split_once(':') {
            let name = name.trim();
            let cmd = cmd.trim();

            if !name.is_empty() && !cmd.is_empty() {
                commands.push(
                    Command::new(format!("my-tool {}", name), cmd.to_string())
                        .with_description(format!("Run {} task", name))
                        .with_tag("my-tool")
                        .with_tag("task"),
                );
            }
        }
    }

    commands
}

/// Parse a .myext file and extract commands.
fn parse_myext_file(path: &str, content: &str) -> Vec<Command> {
    let mut commands = Vec::new();

    // Extract filename without extension for command naming
    let filename = path
        .rsplit('/')
        .next()
        .unwrap_or(path)
        .strip_suffix(".myext")
        .unwrap_or(path);

    // Example: Each non-empty line is a command
    for (i, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        commands.push(
            Command::new(
                format!("{}:{}", filename, i + 1),
                line.to_string(),
            )
            .with_description(format!("Command from {} line {}", path, i + 1))
            .with_tag("my-tool"),
        );
    }

    commands
}

// Export the scanner as a WASM plugin
export_scanner!(MyScanner);

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_name() {
        let scanner = MyScanner;
        assert_eq!(scanner.name(), "my-scanner");
    }

    #[test]
    fn test_file_patterns() {
        let scanner = MyScanner;
        let patterns = scanner.file_patterns();
        assert!(patterns.contains(&"Myfile"));
        assert!(patterns.contains(&"*.myext"));
    }

    #[test]
    fn test_scan_with_myfile() {
        let scanner = MyScanner;
        let context = ScanContext::new("/project", "test")
            .with_file("Myfile", "build: make build\ntest: make test");

        let commands = scanner.scan(&context);

        // Should find parsed commands plus common commands
        assert!(commands.len() >= 2);

        // Check for parsed task
        assert!(commands.iter().any(|c| c.command == "make build"));
        assert!(commands.iter().any(|c| c.command == "make test"));
    }

    #[test]
    fn test_scan_common_commands() {
        let scanner = MyScanner;
        let context = ScanContext::new("/project", "test")
            .with_file("Myfile", "");

        let commands = scanner.scan(&context);

        // Should include common commands
        assert!(commands.iter().any(|c| c.command == "my-tool init"));
        assert!(commands.iter().any(|c| c.command == "my-tool run"));
    }

    #[test]
    fn test_scan_no_match() {
        let scanner = MyScanner;
        let context = ScanContext::new("/project", "test");

        let commands = scanner.scan(&context);
        assert!(commands.is_empty());
    }

    #[test]
    fn test_parse_myfile() {
        let content = "build: cargo build\ntest: cargo test\n# comment\n";
        let commands = parse_myfile(content);

        assert_eq!(commands.len(), 2);
        assert!(commands[0].name.contains("build"));
        assert!(commands[1].name.contains("test"));
    }

    #[test]
    fn test_parse_myext_file() {
        let content = "echo hello\necho world";
        let commands = parse_myext_file("tasks.myext", content);

        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].command, "echo hello");
        assert_eq!(commands[1].command, "echo world");
    }
}
