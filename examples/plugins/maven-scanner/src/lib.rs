//! Maven Scanner Plugin for Palrun
//!
//! Scans Maven pom.xml files and extracts available goals and lifecycle phases.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
    pub working_dir: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Maven lifecycle phases with descriptions.
const LIFECYCLE_PHASES: &[(&str, &str)] = &[
    ("validate", "Validate the project is correct"),
    ("compile", "Compile the source code"),
    ("test", "Run unit tests"),
    ("package", "Package compiled code (e.g., JAR)"),
    ("verify", "Run integration tests"),
    ("install", "Install package to local repository"),
    ("deploy", "Deploy to remote repository"),
    ("clean", "Clean build artifacts"),
    ("site", "Generate project documentation"),
];

/// Common Maven goals.
const COMMON_GOALS: &[(&str, &str)] = &[
    ("dependency:tree", "Display dependency tree"),
    ("dependency:resolve", "Resolve all dependencies"),
    ("versions:display-dependency-updates", "Check for dependency updates"),
    ("versions:display-plugin-updates", "Check for plugin updates"),
    ("help:effective-pom", "Display effective POM"),
    ("help:effective-settings", "Display effective settings"),
];

/// Parse a pom.xml file and extract available commands.
pub fn parse_pom_xml(content: &str) -> Vec<PluginCommand> {
    let mut commands = Vec::new();
    let use_wrapper = content.contains("maven-wrapper") || std::path::Path::new("mvnw").exists();
    let mvn_cmd = if use_wrapper { "./mvnw" } else { "mvn" };

    // Add lifecycle phases
    for (phase, desc) in LIFECYCLE_PHASES {
        commands.push(PluginCommand {
            name: format!("mvn {}", phase),
            command: format!("{} {}", mvn_cmd, phase),
            description: Some(desc.to_string()),
            working_dir: None,
            tags: vec!["maven".to_string(), "lifecycle".to_string()],
        });
    }

    // Add common compound commands
    commands.push(PluginCommand {
        name: "mvn clean install".to_string(),
        command: format!("{} clean install", mvn_cmd),
        description: Some("Clean and install to local repository".to_string()),
        working_dir: None,
        tags: vec!["maven".to_string(), "build".to_string()],
    });

    commands.push(PluginCommand {
        name: "mvn clean package".to_string(),
        command: format!("{} clean package", mvn_cmd),
        description: Some("Clean and create package".to_string()),
        working_dir: None,
        tags: vec!["maven".to_string(), "build".to_string()],
    });

    commands.push(PluginCommand {
        name: "mvn clean test".to_string(),
        command: format!("{} clean test", mvn_cmd),
        description: Some("Clean and run tests".to_string()),
        working_dir: None,
        tags: vec!["maven".to_string(), "test".to_string()],
    });

    // Add common goals
    for (goal, desc) in COMMON_GOALS {
        commands.push(PluginCommand {
            name: format!("mvn {}", goal),
            command: format!("{} {}", mvn_cmd, goal),
            description: Some(desc.to_string()),
            working_dir: None,
            tags: vec!["maven".to_string(), "goal".to_string()],
        });
    }

    // Parse custom profiles from pom.xml
    for profile_id in extract_profiles(content) {
        commands.push(PluginCommand {
            name: format!("mvn install -P{}", profile_id),
            command: format!("{} install -P{}", mvn_cmd, profile_id),
            description: Some(format!("Install with '{}' profile", profile_id)),
            working_dir: None,
            tags: vec!["maven".to_string(), "profile".to_string()],
        });
    }

    commands
}

/// Extract profile IDs from pom.xml content.
fn extract_profiles(content: &str) -> Vec<String> {
    let mut profiles = Vec::new();

    // Simple XML parsing for <profile><id>...</id></profile>
    let mut in_profile = false;
    let mut in_id = false;
    let mut current_id = String::new();

    for line in content.lines() {
        let line = line.trim();

        if line.contains("<profile>") {
            in_profile = true;
        }

        // Check for <id> before checking </profile> to handle inline XML
        if in_profile && line.contains("<id>") {
            in_id = true;
            if let Some(start) = line.find("<id>") {
                let after_tag = &line[start + 4..];
                if let Some(end) = after_tag.find("</id>") {
                    current_id = after_tag[..end].to_string();
                    profiles.push(current_id.clone());
                    current_id.clear();
                    in_id = false;
                } else {
                    current_id = after_tag.to_string();
                }
            }
        } else if in_id {
            if let Some(end) = line.find("</id>") {
                current_id.push_str(&line[..end]);
                profiles.push(current_id.clone());
                current_id.clear();
                in_id = false;
            } else {
                current_id.push_str(line);
            }
        }

        if line.contains("</profile>") {
            in_profile = false;
        }
    }

    profiles
}

/// Main entry point for the scanner plugin.
#[no_mangle]
pub extern "C" fn scan(project_path_ptr: *const u8, project_path_len: usize) -> *mut u8 {
    let project_path = unsafe {
        let slice = std::slice::from_raw_parts(project_path_ptr, project_path_len);
        std::str::from_utf8(slice).unwrap_or("")
    };

    let commands = scan_project(project_path);
    let json = serde_json::to_string(&commands).unwrap_or_else(|_| "[]".to_string());
    let bytes = json.into_bytes();
    let ptr = bytes.as_ptr() as *mut u8;
    std::mem::forget(bytes);
    ptr
}

/// Scan a project directory for Maven commands.
pub fn scan_project(project_path: &str) -> Vec<PluginCommand> {
    let pom_path = std::path::Path::new(project_path).join("pom.xml");

    if pom_path.exists() {
        // In a real plugin, read through host API
        parse_pom_xml("")
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_lifecycle_phases() {
        let commands = parse_pom_xml("");
        assert!(commands.iter().any(|c| c.name == "mvn compile"));
        assert!(commands.iter().any(|c| c.name == "mvn test"));
        assert!(commands.iter().any(|c| c.name == "mvn package"));
        assert!(commands.iter().any(|c| c.name == "mvn install"));
    }

    #[test]
    fn test_parse_common_goals() {
        let commands = parse_pom_xml("");
        assert!(commands.iter().any(|c| c.name == "mvn dependency:tree"));
    }

    #[test]
    fn test_parse_compound_commands() {
        let commands = parse_pom_xml("");
        assert!(commands.iter().any(|c| c.name == "mvn clean install"));
        assert!(commands.iter().any(|c| c.name == "mvn clean package"));
    }

    #[test]
    fn test_extract_profiles() {
        let pom = r#"
            <profiles>
                <profile>
                    <id>dev</id>
                </profile>
                <profile>
                    <id>prod</id>
                </profile>
            </profiles>
        "#;

        let profiles = extract_profiles(pom);
        assert!(profiles.contains(&"dev".to_string()));
        assert!(profiles.contains(&"prod".to_string()));
    }

    #[test]
    fn test_profile_commands() {
        let pom = r#"
            <profile><id>staging</id></profile>
        "#;

        let commands = parse_pom_xml(pom);
        assert!(commands.iter().any(|c| c.name.contains("-Pstaging")));
    }
}
