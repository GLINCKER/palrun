//! Composer Scanner Plugin for Palrun
//!
//! Scans PHP Composer projects (composer.json) and extracts available
//! scripts and commands.

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

/// Common Composer commands.
const COMPOSER_COMMANDS: &[(&str, &str)] = &[
    ("install", "Install dependencies from composer.lock"),
    ("update", "Update dependencies to latest versions"),
    ("require", "Add a new dependency"),
    ("remove", "Remove a dependency"),
    ("dump-autoload", "Regenerate autoload files"),
    ("validate", "Validate composer.json"),
    ("outdated", "Show outdated packages"),
    ("show", "Show installed packages"),
    ("why", "Show why a package is installed"),
    ("why-not", "Show why a package cannot be installed"),
    ("fund", "Show funding information"),
    ("licenses", "Show package licenses"),
];

/// Common PHP development commands.
const PHP_DEV_COMMANDS: &[(&str, &str)] = &[
    ("phpunit", "Run PHPUnit tests"),
    ("phpunit --coverage-html coverage", "Run tests with HTML coverage"),
    ("phpstan analyse", "Static analysis with PHPStan"),
    ("psalm", "Static analysis with Psalm"),
    ("php-cs-fixer fix", "Fix code style with PHP CS Fixer"),
    ("phpcs", "Check code style with PHP_CodeSniffer"),
    ("phpcbf", "Fix code style with PHP_CodeSniffer"),
];

/// Parsed composer.json structure.
#[derive(Debug, Deserialize)]
struct ComposerJson {
    #[serde(default)]
    scripts: std::collections::HashMap<String, serde_json::Value>,
    #[serde(rename = "scripts-descriptions", default)]
    scripts_descriptions: std::collections::HashMap<String, String>,
    #[serde(default)]
    require: std::collections::HashMap<String, String>,
    #[serde(rename = "require-dev", default)]
    require_dev: std::collections::HashMap<String, String>,
}

/// Parse a composer.json file and extract available commands.
pub fn parse_composer_json(content: &str) -> Vec<PluginCommand> {
    let mut commands = Vec::new();

    // Try to parse the JSON
    let composer: ComposerJson = match serde_json::from_str(content) {
        Ok(c) => c,
        Err(_) => return commands,
    };

    // Add basic Composer commands
    for (cmd, desc) in COMPOSER_COMMANDS {
        commands.push(PluginCommand {
            name: format!("composer {}", cmd),
            command: format!("composer {}", cmd),
            description: Some(desc.to_string()),
            working_dir: None,
            tags: vec!["composer".to_string(), "php".to_string()],
        });
    }

    // Add custom scripts from composer.json
    for (name, value) in &composer.scripts {
        // Skip lifecycle hooks (they start with pre- or post-)
        if name.starts_with("pre-") || name.starts_with("post-") {
            continue;
        }

        let description = composer
            .scripts_descriptions
            .get(name)
            .cloned()
            .or_else(|| {
                // Try to create description from the script value
                match value {
                    serde_json::Value::String(s) => Some(format!("Run: {}", s)),
                    serde_json::Value::Array(arr) => {
                        if arr.len() == 1 {
                            arr[0].as_str().map(|s| format!("Run: {}", s))
                        } else {
                            Some(format!("Run {} commands", arr.len()))
                        }
                    }
                    _ => None,
                }
            });

        commands.push(PluginCommand {
            name: format!("composer {}", name),
            command: format!("composer run-script {}", name),
            description,
            working_dir: None,
            tags: vec!["composer".to_string(), "script".to_string()],
        });
    }

    // Add PHP dev commands if relevant dev dependencies exist
    let has_phpunit = composer.require_dev.contains_key("phpunit/phpunit");
    let has_phpstan = composer.require_dev.contains_key("phpstan/phpstan");
    let has_psalm = composer.require_dev.contains_key("vimeo/psalm");
    let has_cs_fixer = composer.require_dev.contains_key("friendsofphp/php-cs-fixer");
    let has_phpcs = composer.require_dev.contains_key("squizlabs/php_codesniffer");

    for (cmd, desc) in PHP_DEV_COMMANDS {
        let should_add = match *cmd {
            c if c.starts_with("phpunit") => has_phpunit,
            c if c.starts_with("phpstan") => has_phpstan,
            "psalm" => has_psalm,
            c if c.starts_with("php-cs-fixer") => has_cs_fixer,
            c if c.starts_with("phpcs") || c.starts_with("phpcbf") => has_phpcs,
            _ => false,
        };

        if should_add {
            commands.push(PluginCommand {
                name: format!("vendor/bin/{}", cmd.split_whitespace().next().unwrap_or(cmd)),
                command: format!("./vendor/bin/{}", cmd),
                description: Some(desc.to_string()),
                working_dir: None,
                tags: vec!["php".to_string(), "dev".to_string()],
            });
        }
    }

    commands
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

/// Scan a project directory for Composer commands.
pub fn scan_project(project_path: &str) -> Vec<PluginCommand> {
    let composer_path = std::path::Path::new(project_path).join("composer.json");

    if composer_path.exists() {
        // In a real plugin, read through host API
        parse_composer_json("{}")
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_commands() {
        let content = r#"{"name": "test/project"}"#;
        let commands = parse_composer_json(content);

        assert!(commands.iter().any(|c| c.name == "composer install"));
        assert!(commands.iter().any(|c| c.name == "composer update"));
        assert!(commands.iter().any(|c| c.name == "composer require"));
    }

    #[test]
    fn test_parse_custom_scripts() {
        let content = r#"{
            "scripts": {
                "test": "phpunit",
                "lint": "phpcs src/",
                "build": ["npm run build", "composer dump-autoload"]
            },
            "scripts-descriptions": {
                "test": "Run the test suite"
            }
        }"#;

        let commands = parse_composer_json(content);
        assert!(commands.iter().any(|c| c.name == "composer test"));
        assert!(commands.iter().any(|c| c.name == "composer lint"));
        assert!(commands.iter().any(|c| c.name == "composer build"));

        // Check description from scripts-descriptions
        let test_cmd = commands.iter().find(|c| c.name == "composer test").unwrap();
        assert_eq!(test_cmd.description, Some("Run the test suite".to_string()));
    }

    #[test]
    fn test_skip_lifecycle_hooks() {
        let content = r#"{
            "scripts": {
                "pre-install-cmd": "echo before install",
                "post-install-cmd": "echo after install",
                "test": "phpunit"
            }
        }"#;

        let commands = parse_composer_json(content);
        assert!(!commands.iter().any(|c| c.name.contains("pre-install")));
        assert!(!commands.iter().any(|c| c.name.contains("post-install")));
        assert!(commands.iter().any(|c| c.name == "composer test"));
    }

    #[test]
    fn test_dev_dependency_detection() {
        let content = r#"{
            "require-dev": {
                "phpunit/phpunit": "^10.0",
                "phpstan/phpstan": "^1.0"
            }
        }"#;

        let commands = parse_composer_json(content);
        assert!(commands.iter().any(|c| c.name.contains("phpunit")));
        assert!(commands.iter().any(|c| c.name.contains("phpstan")));
        // Should not have psalm since it's not in require-dev
        assert!(!commands.iter().any(|c| c.command.contains("psalm")));
    }

    #[test]
    fn test_invalid_json() {
        let content = "not valid json {";
        let commands = parse_composer_json(content);
        assert!(commands.is_empty());
    }
}
