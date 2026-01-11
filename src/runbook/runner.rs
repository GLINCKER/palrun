//! Runbook execution engine.
//!
//! Executes runbook steps with variable interpolation and condition evaluation.

use std::collections::HashMap;

use regex::Regex;

use super::{Runbook, Step};
use crate::core::Executor;

/// Runbook runner state.
#[derive(Debug)]
pub struct RunbookRunner {
    /// The runbook being executed
    runbook: Runbook,

    /// Variable values
    variables: HashMap<String, String>,

    /// Current step index
    current_step: usize,

    /// Runner state
    state: RunnerState,

    /// Execution results
    results: Vec<StepResult>,
}

/// Runner state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerState {
    Ready,
    Running,
    AwaitingConfirmation,
    Completed,
    Failed(String),
}

/// Result of executing a step.
#[derive(Debug)]
pub struct StepResult {
    /// Step name
    pub name: String,

    /// Whether the step succeeded
    pub success: bool,

    /// Exit code
    pub exit_code: Option<i32>,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl RunbookRunner {
    /// Create a new runner for a runbook.
    pub fn new(runbook: Runbook) -> Self {
        // Initialize variables from defaults
        let mut variables = HashMap::new();
        if let Some(ref vars) = runbook.variables {
            for (name, var) in vars {
                if let Some(ref default) = var.default {
                    variables.insert(name.clone(), default.clone());
                }
            }
        }

        Self { runbook, variables, current_step: 0, state: RunnerState::Ready, results: Vec::new() }
    }

    /// Set a variable value.
    pub fn set_variable(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(name.into(), value.into());
    }

    /// Set multiple variables.
    pub fn set_variables(&mut self, vars: HashMap<String, String>) {
        self.variables.extend(vars);
    }

    /// Get the current state.
    pub fn state(&self) -> &RunnerState {
        &self.state
    }

    /// Get the current step (if any).
    pub fn current_step(&self) -> Option<&Step> {
        self.runbook.steps.get(self.current_step)
    }

    /// Get execution results.
    pub fn results(&self) -> &[StepResult] {
        &self.results
    }

    /// Run the entire runbook.
    pub fn run(&mut self) -> anyhow::Result<()> {
        self.state = RunnerState::Running;

        // Set default variable values
        if let Some(ref vars) = self.runbook.variables {
            for (name, var) in vars {
                if !self.variables.contains_key(name) {
                    if let Some(ref default) = var.default {
                        self.variables.insert(name.clone(), default.clone());
                    }
                }
            }
        }

        // Execute each step
        while self.current_step < self.runbook.steps.len() {
            let step = &self.runbook.steps[self.current_step];

            // Check condition
            if let Some(ref condition) = step.condition {
                if !self.evaluate_condition(condition) {
                    tracing::debug!(step = step.name, "Skipping step (condition not met)");
                    self.current_step += 1;
                    continue;
                }
            }

            // Check confirmation
            if step.confirm.unwrap_or(false) {
                self.state = RunnerState::AwaitingConfirmation;
                // In a real implementation, we'd pause here for user input
                // For now, we'll just continue
            }

            // Execute the step
            match self.execute_step(step) {
                Ok(result) => {
                    let success = result.success;
                    self.results.push(result);

                    if !success && !step.continue_on_error.unwrap_or(false) {
                        if !step.optional.unwrap_or(false) {
                            self.state =
                                RunnerState::Failed(format!("Step '{}' failed", step.name));
                            return Err(anyhow::anyhow!("Step '{}' failed", step.name));
                        }
                    }
                }
                Err(e) => {
                    if !step.optional.unwrap_or(false) {
                        self.state = RunnerState::Failed(e.to_string());
                        return Err(e);
                    }
                }
            }

            self.current_step += 1;
        }

        self.state = RunnerState::Completed;
        Ok(())
    }

    /// Execute a single step.
    fn execute_step(&self, step: &Step) -> anyhow::Result<StepResult> {
        let command = self.interpolate(&step.command);

        tracing::info!(step = step.name, command = command, "Executing step");

        let mut cmd = crate::core::Command::new(&step.name, &command);

        if let Some(ref dir) = step.working_dir {
            cmd = cmd.with_working_dir(self.interpolate(dir));
        }

        if let Some(ref env) = step.env {
            for (key, value) in env {
                cmd = cmd.with_env(key, self.interpolate(value));
            }
        }

        let executor = Executor::new().capture(true);
        let start = std::time::Instant::now();

        match executor.execute(&cmd) {
            Ok(result) => {
                let duration_ms = start.elapsed().as_millis() as u64;

                Ok(StepResult {
                    name: step.name.clone(),
                    success: result.success(),
                    exit_code: result.code(),
                    error: if result.success() { None } else { result.stderr },
                    duration_ms,
                })
            }
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;

                Ok(StepResult {
                    name: step.name.clone(),
                    success: false,
                    exit_code: None,
                    error: Some(e.to_string()),
                    duration_ms,
                })
            }
        }
    }

    /// Interpolate variables in a string.
    fn interpolate(&self, template: &str) -> String {
        let re = Regex::new(r"\{\{\s*(\w+)\s*\}\}").unwrap();

        re.replace_all(template, |caps: &regex::Captures| {
            let var_name = &caps[1];
            self.variables.get(var_name).cloned().unwrap_or_else(|| format!("{{{{{var_name}}}}}"))
        })
        .to_string()
    }

    /// Evaluate a condition expression.
    fn evaluate_condition(&self, condition: &str) -> bool {
        let condition = condition.trim();

        // Handle negation
        if let Some(rest) = condition.strip_prefix('!') {
            let var_name = rest.trim();
            let value = self.variables.get(var_name).map(String::as_str).unwrap_or("");
            return value.is_empty() || value == "false" || value == "0";
        }

        // Handle equality
        if condition.contains("==") {
            let parts: Vec<&str> = condition.split("==").collect();
            if parts.len() == 2 {
                let var_name = parts[0].trim();
                let expected = parts[1].trim().trim_matches(|c| c == '\'' || c == '"');
                let actual = self.variables.get(var_name).map(String::as_str).unwrap_or("");
                return actual == expected;
            }
        }

        // Handle inequality
        if condition.contains("!=") {
            let parts: Vec<&str> = condition.split("!=").collect();
            if parts.len() == 2 {
                let var_name = parts[0].trim();
                let expected = parts[1].trim().trim_matches(|c| c == '\'' || c == '"');
                let actual = self.variables.get(var_name).map(String::as_str).unwrap_or("");
                return actual != expected;
            }
        }

        // Simple truthiness check
        let value = self.variables.get(condition).map(String::as_str).unwrap_or("");
        !value.is_empty() && value != "false" && value != "0"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runbook::parse_runbook_str;

    #[test]
    fn test_interpolation() {
        let yaml = r#"
name: test
variables:
  name:
    type: string
    default: world
steps:
  - name: greet
    command: echo "Hello {{ name }}"
"#;

        let runbook = parse_runbook_str(yaml).unwrap();
        let runner = RunbookRunner::new(runbook);

        let result = runner.interpolate("Hello {{ name }}!");
        assert_eq!(result, "Hello world!");
    }

    #[test]
    fn test_condition_negation() {
        let yaml = r#"
name: test
steps:
  - name: step1
    command: echo "test"
"#;

        let runbook = parse_runbook_str(yaml).unwrap();
        let mut runner = RunbookRunner::new(runbook);

        runner.set_variable("skip", "true");
        assert!(!runner.evaluate_condition("!skip"));

        runner.set_variable("skip", "false");
        assert!(runner.evaluate_condition("!skip"));
    }

    #[test]
    fn test_condition_equality() {
        let yaml = r#"
name: test
steps:
  - name: step1
    command: echo "test"
"#;

        let runbook = parse_runbook_str(yaml).unwrap();
        let mut runner = RunbookRunner::new(runbook);

        runner.set_variable("env", "prod");

        assert!(runner.evaluate_condition("env == 'prod'"));
        assert!(!runner.evaluate_condition("env == 'staging'"));
        assert!(runner.evaluate_condition("env != 'staging'"));
    }
}
