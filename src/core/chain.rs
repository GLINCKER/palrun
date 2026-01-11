//! Command chaining and conditional execution module.
//!
//! Supports chaining commands with operators:
//! - `&&` - run next if previous succeeds
//! - `||` - run next if previous fails
//! - `;`  - run next regardless of previous result

use std::process::{Command as ProcessCommand, Stdio};
use std::time::{Duration, Instant};

/// Chain operators for connecting commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainOperator {
    /// Run next command only if previous succeeds (exit code 0)
    And,
    /// Run next command only if previous fails (non-zero exit code)
    Or,
    /// Run next command regardless of previous result
    Sequence,
}

impl ChainOperator {
    /// Parse a chain operator from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "&&" => Some(Self::And),
            "||" => Some(Self::Or),
            ";" => Some(Self::Sequence),
            _ => None,
        }
    }

    /// Get the string representation of the operator.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::And => "&&",
            Self::Or => "||",
            Self::Sequence => ";",
        }
    }

    /// Check if the next command should run based on previous success.
    pub fn should_continue(&self, previous_success: bool) -> bool {
        match self {
            Self::And => previous_success,
            Self::Or => !previous_success,
            Self::Sequence => true,
        }
    }
}

/// A single step in a command chain.
#[derive(Debug, Clone)]
pub struct ChainStep {
    /// The command to execute
    pub command: String,
    /// The operator connecting to the next command (None for last step)
    pub operator: Option<ChainOperator>,
}

impl ChainStep {
    /// Create a new chain step.
    pub fn new(command: String, operator: Option<ChainOperator>) -> Self {
        Self { command, operator }
    }
}

/// Status of a chain step execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainStepStatus {
    /// Step is pending execution
    Pending,
    /// Step is currently running
    Running,
    /// Step completed successfully
    Success,
    /// Step failed with optional exit code
    Failed(Option<i32>),
    /// Step was skipped due to chain logic
    Skipped,
}

impl ChainStepStatus {
    /// Check if the step was successful.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    /// Check if the step has finished.
    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Success | Self::Failed(_) | Self::Skipped)
    }
}

/// Result of executing a single chain step.
#[derive(Debug, Clone)]
pub struct ChainStepResult {
    /// The command that was executed
    pub command: String,
    /// Status of execution
    pub status: ChainStepStatus,
    /// Combined stdout output
    pub stdout: String,
    /// Combined stderr output
    pub stderr: String,
    /// Duration of execution
    pub duration: Duration,
}

/// A parsed command chain.
#[derive(Debug, Clone)]
pub struct CommandChain {
    /// The original raw command string
    pub raw: String,
    /// Individual steps in the chain
    pub steps: Vec<ChainStep>,
}

impl CommandChain {
    /// Parse a command string into a chain.
    ///
    /// Supports `&&`, `||`, and `;` operators.
    pub fn parse(input: &str) -> Self {
        let mut steps = Vec::new();
        let mut current_cmd = String::new();
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '&' if chars.peek() == Some(&'&') => {
                    chars.next(); // consume second &
                    let cmd = current_cmd.trim().to_string();
                    if !cmd.is_empty() {
                        steps.push(ChainStep::new(cmd, Some(ChainOperator::And)));
                    }
                    current_cmd.clear();
                }
                '|' if chars.peek() == Some(&'|') => {
                    chars.next(); // consume second |
                    let cmd = current_cmd.trim().to_string();
                    if !cmd.is_empty() {
                        steps.push(ChainStep::new(cmd, Some(ChainOperator::Or)));
                    }
                    current_cmd.clear();
                }
                ';' => {
                    let cmd = current_cmd.trim().to_string();
                    if !cmd.is_empty() {
                        steps.push(ChainStep::new(cmd, Some(ChainOperator::Sequence)));
                    }
                    current_cmd.clear();
                }
                // Handle quoted strings to avoid parsing operators inside them
                '"' => {
                    current_cmd.push(c);
                    while let Some(qc) = chars.next() {
                        current_cmd.push(qc);
                        if qc == '"' {
                            break;
                        }
                    }
                }
                '\'' => {
                    current_cmd.push(c);
                    while let Some(qc) = chars.next() {
                        current_cmd.push(qc);
                        if qc == '\'' {
                            break;
                        }
                    }
                }
                _ => current_cmd.push(c),
            }
        }

        // Add the last command (no trailing operator)
        let cmd = current_cmd.trim().to_string();
        if !cmd.is_empty() {
            steps.push(ChainStep::new(cmd, None));
        }

        Self { raw: input.to_string(), steps }
    }

    /// Check if this is a simple command (no chaining).
    pub fn is_simple(&self) -> bool {
        self.steps.len() <= 1
    }

    /// Get the number of steps in the chain.
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Check if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

/// Result of executing a complete chain.
#[derive(Debug)]
pub struct ChainResult {
    /// Results for each step
    pub steps: Vec<ChainStepResult>,
    /// Total duration of chain execution
    pub total_duration: Duration,
    /// Whether the overall chain succeeded
    pub success: bool,
}

impl ChainResult {
    /// Get the number of successful steps.
    pub fn success_count(&self) -> usize {
        self.steps.iter().filter(|s| s.status.is_success()).count()
    }

    /// Get the number of failed steps.
    pub fn failed_count(&self) -> usize {
        self.steps.iter().filter(|s| matches!(s.status, ChainStepStatus::Failed(_))).count()
    }

    /// Get the number of skipped steps.
    pub fn skipped_count(&self) -> usize {
        self.steps.iter().filter(|s| matches!(s.status, ChainStepStatus::Skipped)).count()
    }

    /// Get combined output from all steps.
    pub fn combined_output(&self) -> String {
        let mut output = String::new();
        for (i, step) in self.steps.iter().enumerate() {
            if i > 0 {
                output.push('\n');
            }
            output.push_str(&format!("=== {} ===\n", step.command));
            match &step.status {
                ChainStepStatus::Skipped => {
                    output.push_str("(skipped)\n");
                }
                _ => {
                    if !step.stdout.is_empty() {
                        output.push_str(&step.stdout);
                        if !step.stdout.ends_with('\n') {
                            output.push('\n');
                        }
                    }
                    if !step.stderr.is_empty() {
                        output.push_str(&step.stderr);
                        if !step.stderr.ends_with('\n') {
                            output.push('\n');
                        }
                    }
                }
            }
        }
        output
    }
}

/// Chain executor with progress callback support.
pub struct ChainExecutor {
    /// Working directory for commands
    working_dir: Option<String>,
}

impl Default for ChainExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ChainExecutor {
    /// Create a new chain executor.
    pub fn new() -> Self {
        Self { working_dir: None }
    }

    /// Set the working directory for commands.
    #[must_use]
    pub fn working_dir(mut self, dir: String) -> Self {
        self.working_dir = Some(dir);
        self
    }

    /// Execute a command chain.
    pub fn execute(&self, chain: &CommandChain) -> anyhow::Result<ChainResult> {
        self.execute_with_progress(chain, |_, _| {})
    }

    /// Execute a command chain with progress callback.
    ///
    /// The callback is called before each step starts with (step_index, total_steps).
    pub fn execute_with_progress<F>(
        &self,
        chain: &CommandChain,
        mut on_progress: F,
    ) -> anyhow::Result<ChainResult>
    where
        F: FnMut(usize, usize),
    {
        let start = Instant::now();
        let mut results = Vec::new();
        let mut previous_success = true;
        let total_steps = chain.steps.len();

        for (i, step) in chain.steps.iter().enumerate() {
            on_progress(i, total_steps);

            // Check if we should run this step based on previous result and operator
            let should_run = if i == 0 {
                true
            } else if let Some(prev_step) = chain.steps.get(i - 1) {
                if let Some(op) = prev_step.operator {
                    op.should_continue(previous_success)
                } else {
                    true
                }
            } else {
                true
            };

            if !should_run {
                results.push(ChainStepResult {
                    command: step.command.clone(),
                    status: ChainStepStatus::Skipped,
                    stdout: String::new(),
                    stderr: String::new(),
                    duration: Duration::ZERO,
                });
                continue;
            }

            // Execute the step
            let step_result = self.execute_step(&step.command)?;
            previous_success = step_result.status.is_success();
            results.push(step_result);
        }

        let total_duration = start.elapsed();
        let success = results
            .iter()
            .all(|r| matches!(r.status, ChainStepStatus::Success | ChainStepStatus::Skipped));

        Ok(ChainResult { steps: results, total_duration, success })
    }

    /// Execute a single command step.
    fn execute_step(&self, command: &str) -> anyhow::Result<ChainStepResult> {
        let start = Instant::now();

        let (shell, shell_arg) = get_shell();

        let mut cmd = ProcessCommand::new(shell);
        cmd.arg(shell_arg);
        cmd.arg(command);

        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        match cmd.output() {
            Ok(output) => {
                let duration = start.elapsed();
                let status = if output.status.success() {
                    ChainStepStatus::Success
                } else {
                    ChainStepStatus::Failed(output.status.code())
                };

                Ok(ChainStepResult {
                    command: command.to_string(),
                    status,
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    duration,
                })
            }
            Err(e) => {
                let duration = start.elapsed();
                Ok(ChainStepResult {
                    command: command.to_string(),
                    status: ChainStepStatus::Failed(None),
                    stdout: String::new(),
                    stderr: e.to_string(),
                    duration,
                })
            }
        }
    }
}

/// Get the shell and argument for the current platform.
fn get_shell() -> (&'static str, &'static str) {
    if cfg!(target_os = "windows") {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_operator_from_str() {
        assert_eq!(ChainOperator::from_str("&&"), Some(ChainOperator::And));
        assert_eq!(ChainOperator::from_str("||"), Some(ChainOperator::Or));
        assert_eq!(ChainOperator::from_str(";"), Some(ChainOperator::Sequence));
        assert_eq!(ChainOperator::from_str("&"), None);
        assert_eq!(ChainOperator::from_str("|"), None);
    }

    #[test]
    fn test_chain_operator_should_continue() {
        assert!(ChainOperator::And.should_continue(true));
        assert!(!ChainOperator::And.should_continue(false));

        assert!(!ChainOperator::Or.should_continue(true));
        assert!(ChainOperator::Or.should_continue(false));

        assert!(ChainOperator::Sequence.should_continue(true));
        assert!(ChainOperator::Sequence.should_continue(false));
    }

    #[test]
    fn test_parse_simple_command() {
        let chain = CommandChain::parse("echo hello");
        assert!(chain.is_simple());
        assert_eq!(chain.len(), 1);
        assert_eq!(chain.steps[0].command, "echo hello");
        assert!(chain.steps[0].operator.is_none());
    }

    #[test]
    fn test_parse_and_chain() {
        let chain = CommandChain::parse("echo one && echo two && echo three");
        assert!(!chain.is_simple());
        assert_eq!(chain.len(), 3);
        assert_eq!(chain.steps[0].command, "echo one");
        assert_eq!(chain.steps[0].operator, Some(ChainOperator::And));
        assert_eq!(chain.steps[1].command, "echo two");
        assert_eq!(chain.steps[1].operator, Some(ChainOperator::And));
        assert_eq!(chain.steps[2].command, "echo three");
        assert!(chain.steps[2].operator.is_none());
    }

    #[test]
    fn test_parse_or_chain() {
        let chain = CommandChain::parse("false || echo fallback");
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.steps[0].command, "false");
        assert_eq!(chain.steps[0].operator, Some(ChainOperator::Or));
        assert_eq!(chain.steps[1].command, "echo fallback");
    }

    #[test]
    fn test_parse_sequence_chain() {
        let chain = CommandChain::parse("echo one; echo two");
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.steps[0].command, "echo one");
        assert_eq!(chain.steps[0].operator, Some(ChainOperator::Sequence));
        assert_eq!(chain.steps[1].command, "echo two");
    }

    #[test]
    fn test_parse_mixed_operators() {
        let chain = CommandChain::parse("cmd1 && cmd2 || cmd3 ; cmd4");
        assert_eq!(chain.len(), 4);
        assert_eq!(chain.steps[0].operator, Some(ChainOperator::And));
        assert_eq!(chain.steps[1].operator, Some(ChainOperator::Or));
        assert_eq!(chain.steps[2].operator, Some(ChainOperator::Sequence));
        assert!(chain.steps[3].operator.is_none());
    }

    #[test]
    fn test_parse_quoted_string() {
        let chain = CommandChain::parse("echo \"hello && world\" && echo done");
        assert_eq!(chain.len(), 2);
        assert_eq!(chain.steps[0].command, "echo \"hello && world\"");
        assert_eq!(chain.steps[1].command, "echo done");
    }

    #[test]
    fn test_execute_simple() {
        let chain = CommandChain::parse("echo hello");
        let executor = ChainExecutor::new();
        let result = executor.execute(&chain).unwrap();

        assert!(result.success);
        assert_eq!(result.steps.len(), 1);
        assert!(result.steps[0].status.is_success());
        assert!(result.steps[0].stdout.contains("hello"));
    }

    #[test]
    fn test_execute_and_chain_success() {
        let chain = CommandChain::parse("echo one && echo two");
        let executor = ChainExecutor::new();
        let result = executor.execute(&chain).unwrap();

        assert!(result.success);
        assert_eq!(result.success_count(), 2);
        assert_eq!(result.failed_count(), 0);
    }

    #[test]
    fn test_execute_and_chain_failure() {
        let chain = CommandChain::parse("false && echo never");
        let executor = ChainExecutor::new();
        let result = executor.execute(&chain).unwrap();

        assert!(!result.success);
        assert_eq!(result.failed_count(), 1);
        assert_eq!(result.skipped_count(), 1);
    }

    #[test]
    fn test_execute_or_chain() {
        let chain = CommandChain::parse("false || echo fallback");
        let executor = ChainExecutor::new();
        let result = executor.execute(&chain).unwrap();

        // The chain succeeds because fallback runs
        assert_eq!(result.success_count(), 1);
        assert_eq!(result.failed_count(), 1);
        assert!(result.steps[1].stdout.contains("fallback"));
    }

    #[test]
    fn test_execute_sequence() {
        let chain = CommandChain::parse("false ; echo always");
        let executor = ChainExecutor::new();
        let result = executor.execute(&chain).unwrap();

        // Second command always runs with sequence operator
        assert_eq!(result.steps.len(), 2);
        assert!(matches!(result.steps[0].status, ChainStepStatus::Failed(_)));
        assert!(result.steps[1].status.is_success());
    }

    #[test]
    fn test_chain_step_status() {
        assert!(!ChainStepStatus::Pending.is_finished());
        assert!(!ChainStepStatus::Running.is_finished());
        assert!(ChainStepStatus::Success.is_finished());
        assert!(ChainStepStatus::Failed(Some(1)).is_finished());
        assert!(ChainStepStatus::Skipped.is_finished());

        assert!(ChainStepStatus::Success.is_success());
        assert!(!ChainStepStatus::Failed(Some(1)).is_success());
        assert!(!ChainStepStatus::Skipped.is_success());
    }
}
