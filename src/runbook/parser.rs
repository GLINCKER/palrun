//! Runbook parser.
//!
//! Parses YAML runbook files into Runbook structs.

use std::path::Path;

use super::Runbook;

/// Parse a runbook from a file.
pub fn parse_runbook(path: &Path) -> anyhow::Result<Runbook> {
    let content = std::fs::read_to_string(path)?;
    parse_runbook_str(&content)
}

/// Parse a runbook from a string.
pub fn parse_runbook_str(content: &str) -> anyhow::Result<Runbook> {
    let runbook: Runbook = serde_yaml::from_str(content)?;
    validate_runbook(&runbook)?;
    Ok(runbook)
}

/// Validate a runbook for common errors.
fn validate_runbook(runbook: &Runbook) -> anyhow::Result<()> {
    // Check for empty name
    if runbook.name.is_empty() {
        anyhow::bail!("Runbook name cannot be empty");
    }

    // Check for empty steps
    if runbook.steps.is_empty() {
        anyhow::bail!("Runbook must have at least one step");
    }

    // Validate each step
    for (i, step) in runbook.steps.iter().enumerate() {
        if step.name.is_empty() {
            anyhow::bail!("Step {} has no name", i + 1);
        }
        if step.command.is_empty() {
            anyhow::bail!("Step '{}' has no command", step.name);
        }
    }

    // Check for undefined variables in commands
    if let Some(ref variables) = runbook.variables {
        let var_pattern = regex::Regex::new(r"\{\{\s*(\w+)\s*\}\}").unwrap();

        for step in &runbook.steps {
            for cap in var_pattern.captures_iter(&step.command) {
                let var_name = &cap[1];
                if !variables.contains_key(var_name) {
                    // Check if it's an environment variable reference
                    if !var_name.starts_with("env.") && !var_name.starts_with("ENV_") {
                        tracing::warn!(
                            step = step.name,
                            variable = var_name,
                            "Undefined variable in command"
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

/// Discover runbooks in a directory.
pub fn discover_runbooks(dir: &Path) -> anyhow::Result<Vec<(String, Runbook)>> {
    let mut runbooks = Vec::new();

    // Check .palrun/runbooks/
    let runbooks_dir = dir.join(".palrun").join("runbooks");
    if runbooks_dir.exists() {
        runbooks.extend(scan_runbook_dir(&runbooks_dir)?);
    }

    // Check runbooks/ directory
    let alt_runbooks_dir = dir.join("runbooks");
    if alt_runbooks_dir.exists() {
        runbooks.extend(scan_runbook_dir(&alt_runbooks_dir)?);
    }

    Ok(runbooks)
}

/// Scan a directory for runbook files.
fn scan_runbook_dir(dir: &Path) -> anyhow::Result<Vec<(String, Runbook)>> {
    let mut runbooks = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "yaml" || e == "yml") {
                match parse_runbook(&path) {
                    Ok(runbook) => {
                        let name = path
                            .file_stem()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        runbooks.push((name, runbook));
                    }
                    Err(e) => {
                        tracing::warn!(path = ?path, error = %e, "Failed to parse runbook");
                    }
                }
            }
        }
    }

    Ok(runbooks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_runbook() {
        let yaml = r#"
name: test
steps:
  - name: step1
    command: echo "hello"
"#;

        let runbook = parse_runbook_str(yaml).unwrap();
        assert_eq!(runbook.name, "test");
        assert_eq!(runbook.steps.len(), 1);
    }

    #[test]
    fn test_parse_empty_name_fails() {
        let yaml = r#"
name: ""
steps:
  - name: step1
    command: echo "hello"
"#;

        let result = parse_runbook_str(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_no_steps_fails() {
        let yaml = r"
name: test
steps: []
";

        let result = parse_runbook_str(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_step_without_command_fails() {
        let yaml = r#"
name: test
steps:
  - name: step1
    command: ""
"#;

        let result = parse_runbook_str(yaml);
        assert!(result.is_err());
    }
}
