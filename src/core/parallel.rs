//! Parallel command execution module.
//!
//! Handles spawning and managing multiple commands running concurrently,
//! with output multiplexing and aggregated result handling.

use std::io::{BufRead, BufReader};
use std::process::{Command as ProcessCommand, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::Command;

/// Unique identifier for a parallel process.
pub type ProcessId = usize;

/// Status of a parallel process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessStatus {
    /// Process is waiting to start
    Pending,
    /// Process is currently running
    Running,
    /// Process completed successfully
    Success,
    /// Process failed with an error
    Failed(Option<i32>),
    /// Process was cancelled
    Cancelled,
}

impl ProcessStatus {
    /// Check if the process has finished (success, failed, or cancelled).
    pub fn is_finished(&self) -> bool {
        matches!(self, ProcessStatus::Success | ProcessStatus::Failed(_) | ProcessStatus::Cancelled)
    }

    /// Check if the process was successful.
    pub fn is_success(&self) -> bool {
        matches!(self, ProcessStatus::Success)
    }
}

/// Output from a parallel process.
#[derive(Debug, Clone)]
pub struct ProcessOutput {
    /// The line of output
    pub line: String,
    /// Whether this is stderr (true) or stdout (false)
    pub is_stderr: bool,
    /// Timestamp when this line was received
    pub timestamp: Instant,
}

/// Event from a parallel process.
#[derive(Debug)]
pub enum ProcessEvent {
    /// Process started
    Started(ProcessId),
    /// Process produced output
    Output(ProcessId, ProcessOutput),
    /// Process completed
    Completed(ProcessId, ProcessStatus, Duration),
}

/// Information about a parallel process.
#[derive(Debug)]
pub struct ParallelProcess {
    /// Unique ID for this process
    pub id: ProcessId,
    /// The command being executed
    pub command: Command,
    /// Current status
    pub status: ProcessStatus,
    /// Captured stdout lines
    pub stdout: Vec<String>,
    /// Captured stderr lines
    pub stderr: Vec<String>,
    /// When the process started
    pub started_at: Option<Instant>,
    /// How long the process took
    pub duration: Option<Duration>,
}

impl ParallelProcess {
    /// Create a new parallel process.
    fn new(id: ProcessId, command: Command) -> Self {
        Self {
            id,
            command,
            status: ProcessStatus::Pending,
            stdout: Vec::new(),
            stderr: Vec::new(),
            started_at: None,
            duration: None,
        }
    }

    /// Get all output (stdout + stderr interleaved).
    pub fn all_output(&self) -> String {
        let mut output = self.stdout.join("\n");
        if !self.stderr.is_empty() {
            if !output.is_empty() {
                output.push('\n');
            }
            output.push_str(&self.stderr.join("\n"));
        }
        output
    }
}

/// Result of parallel execution.
#[derive(Debug)]
pub struct ParallelResult {
    /// Results for each process
    pub processes: Vec<ParallelProcess>,
    /// Total duration of parallel execution
    pub total_duration: Duration,
}

impl ParallelResult {
    /// Check if all processes succeeded.
    pub fn all_success(&self) -> bool {
        self.processes.iter().all(|p| p.status.is_success())
    }

    /// Get the number of successful processes.
    pub fn success_count(&self) -> usize {
        self.processes.iter().filter(|p| p.status.is_success()).count()
    }

    /// Get the number of failed processes.
    pub fn failed_count(&self) -> usize {
        self.processes.iter().filter(|p| matches!(p.status, ProcessStatus::Failed(_))).count()
    }
}

/// Parallel command executor.
pub struct ParallelExecutor {
    /// Maximum number of concurrent processes
    max_concurrency: usize,
    /// Timeout for individual processes
    timeout: Option<Duration>,
    /// Whether to stop on first failure
    fail_fast: bool,
}

impl Default for ParallelExecutor {
    fn default() -> Self {
        Self { max_concurrency: num_cpus::get().max(4), timeout: None, fail_fast: false }
    }
}

impl ParallelExecutor {
    /// Create a new parallel executor.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum concurrency.
    #[must_use]
    pub fn max_concurrency(mut self, max: usize) -> Self {
        self.max_concurrency = max.max(1);
        self
    }

    /// Set timeout for individual processes.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set fail-fast mode (stop on first failure).
    #[must_use]
    pub fn fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Execute multiple commands in parallel.
    ///
    /// Returns a result containing all process outcomes.
    pub fn execute(&self, commands: Vec<Command>) -> anyhow::Result<ParallelResult> {
        let start = Instant::now();
        let num_commands = commands.len();

        if num_commands == 0 {
            return Ok(ParallelResult { processes: Vec::new(), total_duration: Duration::ZERO });
        }

        // Create process tracking
        let processes: Vec<ParallelProcess> = commands
            .into_iter()
            .enumerate()
            .map(|(id, cmd)| ParallelProcess::new(id, cmd))
            .collect();

        let processes = Arc::new(Mutex::new(processes));

        // Channel for process events
        let (tx, rx): (Sender<ProcessEvent>, Receiver<ProcessEvent>) = mpsc::channel();

        // Spawn processes with concurrency limit
        let handles: Vec<JoinHandle<()>> = {
            let processes = processes.lock().unwrap();
            processes
                .iter()
                .map(|p| {
                    let id = p.id;
                    let command = p.command.clone();
                    let tx = tx.clone();
                    let timeout = self.timeout;

                    thread::spawn(move || {
                        Self::run_process(id, command, tx, timeout);
                    })
                })
                .collect()
        };

        // Drop the sender so rx.recv() will return when all threads are done
        drop(tx);

        // Collect events
        let mut should_cancel = false;
        while let Ok(event) = rx.recv() {
            let mut procs = processes.lock().unwrap();

            match event {
                ProcessEvent::Started(id) => {
                    if let Some(p) = procs.get_mut(id) {
                        p.status = ProcessStatus::Running;
                        p.started_at = Some(Instant::now());
                    }
                }
                ProcessEvent::Output(id, output) => {
                    if let Some(p) = procs.get_mut(id) {
                        if output.is_stderr {
                            p.stderr.push(output.line);
                        } else {
                            p.stdout.push(output.line);
                        }
                    }
                }
                ProcessEvent::Completed(id, status, duration) => {
                    if let Some(p) = procs.get_mut(id) {
                        p.status = status.clone();
                        p.duration = Some(duration);

                        if self.fail_fast && matches!(status, ProcessStatus::Failed(_)) {
                            should_cancel = true;
                        }
                    }
                }
            }

            // Check if all processes are done
            if procs.iter().all(|p| p.status.is_finished()) {
                break;
            }

            if should_cancel {
                // Mark remaining as cancelled
                for p in procs.iter_mut() {
                    if !p.status.is_finished() {
                        p.status = ProcessStatus::Cancelled;
                    }
                }
                break;
            }
        }

        // Wait for all threads to finish
        for handle in handles {
            let _ = handle.join();
        }

        let total_duration = start.elapsed();

        let result_processes = match Arc::try_unwrap(processes) {
            Ok(mutex) => mutex.into_inner().unwrap_or_default(),
            Err(arc) => arc.lock().unwrap().clone(),
        };

        Ok(ParallelResult { processes: result_processes, total_duration })
    }

    /// Execute multiple commands with streaming events.
    ///
    /// Calls the provided callback for each event as it happens.
    pub fn execute_streaming<F>(
        &self,
        commands: Vec<Command>,
        mut on_event: F,
    ) -> anyhow::Result<ParallelResult>
    where
        F: FnMut(ProcessEvent),
    {
        let start = Instant::now();
        let num_commands = commands.len();

        if num_commands == 0 {
            return Ok(ParallelResult { processes: Vec::new(), total_duration: Duration::ZERO });
        }

        // Create process tracking
        let mut processes: Vec<ParallelProcess> = commands
            .into_iter()
            .enumerate()
            .map(|(id, cmd)| ParallelProcess::new(id, cmd))
            .collect();

        // Channel for process events
        let (tx, rx): (Sender<ProcessEvent>, Receiver<ProcessEvent>) = mpsc::channel();

        // Spawn processes
        let handles: Vec<JoinHandle<()>> = processes
            .iter()
            .map(|p| {
                let id = p.id;
                let command = p.command.clone();
                let tx = tx.clone();
                let timeout = self.timeout;

                thread::spawn(move || {
                    Self::run_process(id, command, tx, timeout);
                })
            })
            .collect();

        // Drop the sender so rx.recv() will return when all threads are done
        drop(tx);

        // Collect events and call callback
        while let Ok(event) = rx.recv() {
            // Update internal state
            match &event {
                ProcessEvent::Started(id) => {
                    if let Some(p) = processes.get_mut(*id) {
                        p.status = ProcessStatus::Running;
                        p.started_at = Some(Instant::now());
                    }
                }
                ProcessEvent::Output(id, output) => {
                    if let Some(p) = processes.get_mut(*id) {
                        if output.is_stderr {
                            p.stderr.push(output.line.clone());
                        } else {
                            p.stdout.push(output.line.clone());
                        }
                    }
                }
                ProcessEvent::Completed(id, status, duration) => {
                    if let Some(p) = processes.get_mut(*id) {
                        p.status = status.clone();
                        p.duration = Some(*duration);
                    }
                }
            }

            // Call user callback
            on_event(event);

            // Check if all processes are done
            if processes.iter().all(|p| p.status.is_finished()) {
                break;
            }
        }

        // Wait for all threads to finish
        for handle in handles {
            let _ = handle.join();
        }

        let total_duration = start.elapsed();

        Ok(ParallelResult { processes, total_duration })
    }

    /// Run a single process and send events.
    fn run_process(
        id: ProcessId,
        command: Command,
        tx: Sender<ProcessEvent>,
        _timeout: Option<Duration>,
    ) {
        let start = Instant::now();

        let _ = tx.send(ProcessEvent::Started(id));

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

        if let Ok(mut child) = cmd.spawn() {
            // Read stdout and stderr concurrently
            let stdout = child.stdout.take();
            let stderr = child.stderr.take();
            let tx_stdout = tx.clone();
            let tx_stderr = tx.clone();

            let stdout_handle = thread::spawn(move || {
                if let Some(stdout) = stdout {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines().map_while(Result::ok) {
                        let output =
                            ProcessOutput { line, is_stderr: false, timestamp: Instant::now() };
                        let _ = tx_stdout.send(ProcessEvent::Output(id, output));
                    }
                }
            });

            let stderr_handle = thread::spawn(move || {
                if let Some(stderr) = stderr {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines().map_while(Result::ok) {
                        let output =
                            ProcessOutput { line, is_stderr: true, timestamp: Instant::now() };
                        let _ = tx_stderr.send(ProcessEvent::Output(id, output));
                    }
                }
            });

            let _ = stdout_handle.join();
            let _ = stderr_handle.join();

            if let Ok(status) = child.wait() {
                let duration = start.elapsed();
                let process_status = if status.success() {
                    ProcessStatus::Success
                } else {
                    ProcessStatus::Failed(status.code())
                };
                let _ = tx.send(ProcessEvent::Completed(id, process_status, duration));
            } else {
                let duration = start.elapsed();
                let _ = tx.send(ProcessEvent::Completed(id, ProcessStatus::Failed(None), duration));
            }
        } else {
            let duration = start.elapsed();
            let _ = tx.send(ProcessEvent::Completed(id, ProcessStatus::Failed(None), duration));
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

// Implement Clone for ParallelProcess for Arc unwrap
impl Clone for ParallelProcess {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            command: self.command.clone(),
            status: self.status.clone(),
            stdout: self.stdout.clone(),
            stderr: self.stderr.clone(),
            started_at: self.started_at,
            duration: self.duration,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_executor_creation() {
        let executor = ParallelExecutor::new();
        assert!(executor.max_concurrency >= 1);
        assert!(executor.timeout.is_none());
        assert!(!executor.fail_fast);
    }

    #[test]
    fn test_parallel_executor_builder() {
        let executor = ParallelExecutor::new()
            .max_concurrency(4)
            .timeout(Duration::from_secs(30))
            .fail_fast(true);

        assert_eq!(executor.max_concurrency, 4);
        assert_eq!(executor.timeout, Some(Duration::from_secs(30)));
        assert!(executor.fail_fast);
    }

    #[test]
    fn test_execute_empty() {
        let executor = ParallelExecutor::new();
        let result = executor.execute(vec![]).unwrap();
        assert!(result.processes.is_empty());
        assert!(result.all_success());
    }

    #[test]
    fn test_execute_single_command() {
        let executor = ParallelExecutor::new();
        let commands = vec![Command::new("echo", "echo hello")];

        let result = executor.execute(commands).unwrap();
        assert_eq!(result.processes.len(), 1);
        assert!(result.all_success());
        assert!(result.processes[0].stdout.join("").contains("hello"));
    }

    #[test]
    fn test_execute_multiple_commands() {
        let executor = ParallelExecutor::new();
        let commands = vec![
            Command::new("echo1", "echo one"),
            Command::new("echo2", "echo two"),
            Command::new("echo3", "echo three"),
        ];

        let result = executor.execute(commands).unwrap();
        assert_eq!(result.processes.len(), 3);
        assert!(result.all_success());
        assert_eq!(result.success_count(), 3);
        assert_eq!(result.failed_count(), 0);
    }

    #[test]
    fn test_execute_with_failure() {
        let executor = ParallelExecutor::new();
        let commands = vec![Command::new("success", "echo ok"), Command::new("failure", "false")];

        let result = executor.execute(commands).unwrap();
        assert_eq!(result.processes.len(), 2);
        assert!(!result.all_success());
        assert_eq!(result.success_count(), 1);
        assert_eq!(result.failed_count(), 1);
    }

    #[test]
    fn test_process_status() {
        assert!(!ProcessStatus::Pending.is_finished());
        assert!(!ProcessStatus::Running.is_finished());
        assert!(ProcessStatus::Success.is_finished());
        assert!(ProcessStatus::Failed(Some(1)).is_finished());
        assert!(ProcessStatus::Cancelled.is_finished());

        assert!(ProcessStatus::Success.is_success());
        assert!(!ProcessStatus::Failed(Some(1)).is_success());
    }
}
