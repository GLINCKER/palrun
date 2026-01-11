//! Gradle Scanner Plugin for Palrun
//!
//! This plugin scans Gradle build files and extracts available tasks
//! for use in the Palrun command palette.

use serde::{Deserialize, Serialize};

/// A command discovered by the scanner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    /// Command display name
    pub name: String,
    /// The actual command to execute
    pub command: String,
    /// Optional description
    pub description: Option<String>,
    /// Working directory (relative to project root)
    pub working_dir: Option<String>,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Scanner configuration from the manifest.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ScannerConfig {
    /// Maximum depth to scan for subprojects
    #[serde(default = "default_scan_depth")]
    pub scan_depth: u32,
    /// Include subproject tasks
    #[serde(default = "default_true")]
    pub include_subprojects: bool,
    /// Task patterns to exclude
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

fn default_scan_depth() -> u32 {
    3
}

fn default_true() -> bool {
    true
}

/// Common Gradle tasks that are always available.
const COMMON_TASKS: &[(&str, &str)] = &[
    ("build", "Build the project"),
    ("clean", "Clean build artifacts"),
    ("test", "Run tests"),
    ("assemble", "Assemble the outputs"),
    ("check", "Run all checks"),
    ("jar", "Create JAR file"),
    ("classes", "Compile main classes"),
    ("testClasses", "Compile test classes"),
];

/// Parse a build.gradle file and extract task definitions.
pub fn parse_build_gradle(content: &str) -> Vec<PluginCommand> {
    let mut commands = Vec::new();

    // Add common tasks
    for (name, desc) in COMMON_TASKS {
        commands.push(PluginCommand {
            name: format!("gradle {}", name),
            command: format!("./gradlew {}", name),
            description: Some(desc.to_string()),
            working_dir: None,
            tags: vec!["gradle".to_string(), "build".to_string()],
        });
    }

    // Parse custom tasks from content
    // Look for patterns like: task taskName { ... } or tasks.register("taskName") { ... }

    // Simple regex-like parsing for task definitions
    for line in content.lines() {
        let line = line.trim();

        // Pattern: task taskName
        if line.starts_with("task ") {
            if let Some(task_name) = extract_task_name(line, "task ") {
                if !is_common_task(&task_name) {
                    commands.push(PluginCommand {
                        name: format!("gradle {}", task_name),
                        command: format!("./gradlew {}", task_name),
                        description: Some(format!("Custom task: {}", task_name)),
                        working_dir: None,
                        tags: vec!["gradle".to_string(), "custom".to_string()],
                    });
                }
            }
        }

        // Pattern: tasks.register("taskName")
        if line.contains("tasks.register") {
            if let Some(task_name) = extract_quoted_task_name(line) {
                if !is_common_task(&task_name) {
                    commands.push(PluginCommand {
                        name: format!("gradle {}", task_name),
                        command: format!("./gradlew {}", task_name),
                        description: Some(format!("Registered task: {}", task_name)),
                        working_dir: None,
                        tags: vec!["gradle".to_string(), "custom".to_string()],
                    });
                }
            }
        }
    }

    commands
}

/// Extract task name from a line like "task taskName {"
fn extract_task_name(line: &str, prefix: &str) -> Option<String> {
    let after_prefix = line.strip_prefix(prefix)?;
    let name: String = after_prefix
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();

    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

/// Extract task name from a line like tasks.register("taskName")
fn extract_quoted_task_name(line: &str) -> Option<String> {
    let start = line.find('"')? + 1;
    let end = line[start..].find('"')? + start;
    Some(line[start..end].to_string())
}

/// Check if a task is in the common tasks list.
fn is_common_task(name: &str) -> bool {
    COMMON_TASKS.iter().any(|(n, _)| *n == name)
}

/// Main entry point for the scanner plugin.
///
/// This function is called by the Palrun host with the project path
/// and returns a JSON-encoded list of discovered commands.
#[no_mangle]
pub extern "C" fn scan(project_path_ptr: *const u8, project_path_len: usize) -> *mut u8 {
    // Safety: This is called by the host with valid pointers
    let project_path = unsafe {
        let slice = std::slice::from_raw_parts(project_path_ptr, project_path_len);
        std::str::from_utf8(slice).unwrap_or("")
    };

    let commands = scan_project(project_path);

    // Serialize to JSON
    let json = serde_json::to_string(&commands).unwrap_or_else(|_| "[]".to_string());

    // Return pointer to the result (host will free this)
    let bytes = json.into_bytes();
    let ptr = bytes.as_ptr() as *mut u8;
    std::mem::forget(bytes);
    ptr
}

/// Scan a project directory for Gradle commands.
pub fn scan_project(project_path: &str) -> Vec<PluginCommand> {
    let mut commands = Vec::new();

    // Check for build.gradle or build.gradle.kts
    let build_file = std::path::Path::new(project_path).join("build.gradle");
    let build_file_kts = std::path::Path::new(project_path).join("build.gradle.kts");

    // In a real plugin, we would read these files through the host API
    // For now, just return common tasks if the files exist
    if build_file.exists() || build_file_kts.exists() {
        // Add common Gradle tasks
        for (name, desc) in COMMON_TASKS {
            commands.push(PluginCommand {
                name: format!("gradle {}", name),
                command: format!("./gradlew {}", name),
                description: Some(desc.to_string()),
                working_dir: Some(project_path.to_string()),
                tags: vec!["gradle".to_string()],
            });
        }
    }

    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_common_tasks() {
        let content = "";
        let commands = parse_build_gradle(content);

        assert!(!commands.is_empty());
        assert!(commands.iter().any(|c| c.name == "gradle build"));
        assert!(commands.iter().any(|c| c.name == "gradle test"));
    }

    #[test]
    fn test_parse_custom_task() {
        let content = r#"
            task customBuild {
                doLast {
                    println "Building..."
                }
            }
        "#;

        let commands = parse_build_gradle(content);
        assert!(commands.iter().any(|c| c.name == "gradle customBuild"));
    }

    #[test]
    fn test_parse_registered_task() {
        let content = r#"
            tasks.register("deployProd") {
                doLast {
                    println "Deploying..."
                }
            }
        "#;

        let commands = parse_build_gradle(content);
        assert!(commands.iter().any(|c| c.name == "gradle deployProd"));
    }

    #[test]
    fn test_extract_task_name() {
        assert_eq!(extract_task_name("task myTask {", "task "), Some("myTask".to_string()));
        assert_eq!(extract_task_name("task build_all(", "task "), Some("build_all".to_string()));
        assert_eq!(extract_task_name("task {", "task "), None);
    }

    #[test]
    fn test_extract_quoted_task_name() {
        assert_eq!(
            extract_quoted_task_name(r#"tasks.register("myTask")"#),
            Some("myTask".to_string())
        );
        assert_eq!(
            extract_quoted_task_name(r#"tasks.register("deploy-prod") {"#),
            Some("deploy-prod".to_string())
        );
    }
}
