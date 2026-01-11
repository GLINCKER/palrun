//! Runbook schema definitions.
//!
//! Defines the YAML structure for runbook files.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A runbook definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Runbook {
    /// Name of the runbook
    pub name: String,

    /// Description of what this runbook does
    pub description: Option<String>,

    /// Version of the runbook
    pub version: Option<String>,

    /// Author of the runbook
    pub author: Option<String>,

    /// Variables that can be set by the user
    pub variables: Option<HashMap<String, Variable>>,

    /// Steps to execute
    pub steps: Vec<Step>,
}

/// A variable definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    /// Type of the variable
    #[serde(rename = "type")]
    pub var_type: VarType,

    /// Default value
    pub default: Option<String>,

    /// Prompt to show the user
    pub prompt: Option<String>,

    /// Whether this variable is required
    pub required: Option<bool>,

    /// Options for select type
    pub options: Option<Vec<String>>,
}

/// Variable types.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VarType {
    #[default]
    String,
    Boolean,
    Number,
    Select,
}

/// A step in the runbook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// Name of the step
    pub name: String,

    /// Command to execute
    pub command: String,

    /// Description of this step
    pub description: Option<String>,

    /// Condition for running this step (e.g., "!skip_tests")
    pub condition: Option<String>,

    /// Whether to confirm before running
    pub confirm: Option<bool>,

    /// Whether this step is optional
    pub optional: Option<bool>,

    /// Whether to continue on error
    pub continue_on_error: Option<bool>,

    /// Timeout in seconds
    pub timeout: Option<u64>,

    /// Working directory for this step
    pub working_dir: Option<String>,

    /// Environment variables for this step
    pub env: Option<HashMap<String, String>>,
}

impl Runbook {
    /// Get the number of steps.
    #[must_use]
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Get variable names.
    #[must_use]
    pub fn variable_names(&self) -> Vec<&str> {
        self.variables.as_ref().map(|v| v.keys().map(String::as_str).collect()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_runbook_yaml() {
        let yaml = r#"
name: deploy
description: Deploy to staging
version: "1.0.0"
author: DevOps Team

variables:
  environment:
    type: select
    options: [staging, production]
    default: staging
    prompt: "Select environment"
  skip_tests:
    type: boolean
    default: "false"

steps:
  - name: Run tests
    command: npm test
    condition: "!skip_tests"

  - name: Build
    command: npm run build

  - name: Deploy
    command: npm run deploy
    confirm: true
"#;

        let runbook: Runbook = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(runbook.name, "deploy");
        assert_eq!(runbook.description, Some("Deploy to staging".to_string()));
        assert_eq!(runbook.steps.len(), 3);

        let vars = runbook.variables.unwrap();
        assert!(vars.contains_key("environment"));
        assert!(vars.contains_key("skip_tests"));

        assert_eq!(vars["environment"].var_type, VarType::Select);
        assert_eq!(vars["skip_tests"].var_type, VarType::Boolean);
    }

    #[test]
    fn test_step_with_all_fields() {
        let yaml = r#"
name: full-step
command: echo "hello"
description: A full step
condition: "env == 'prod'"
confirm: true
optional: false
continue_on_error: false
timeout: 30
working_dir: /tmp
env:
  FOO: bar
"#;

        let step: Step = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(step.name, "full-step");
        assert_eq!(step.confirm, Some(true));
        assert_eq!(step.timeout, Some(30));
        assert_eq!(step.env.unwrap().get("FOO"), Some(&"bar".to_string()));
    }
}
