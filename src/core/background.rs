//! Background command execution module.
//!
//! Handles spawning commands in the background with:
//! - Process tracking and management
//! - Output capture to files
//! - Desktop notifications on completion
//! - Process listing and termination

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command as ProcessCommand, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime};

use super::Command;

/// Send a desktop notification for a completed background process.
#[cfg(feature = "notifications")]
pub fn send_notification(name: &str, status: &BackgroundStatus, duration: Duration) {
    use notify_rust::Notification;

    let (icon, body) = match status {
        BackgroundStatus::Completed => {
            ("dialog-information", format!("Completed in {:.2?}", duration))
        }
        BackgroundStatus::Failed(code) => (
            "dialog-error",
            match code {
                Some(c) => format!("Failed with exit code {} after {:.2?}", c, duration),
                None => format!("Failed after {:.2?}", duration),
            },
        ),
        BackgroundStatus::Terminated => ("dialog-warning", "Terminated by user".to_string()),
        BackgroundStatus::Running => return, // Don't notify for running
    };

    let _ = Notification::new()
        .summary(&format!("Palrun: {}", name))
        .body(&body)
        .icon(icon)
        .appname("palrun")
        .timeout(5000)
        .show();
}

/// No-op notification when feature is disabled.
#[cfg(not(feature = "notifications"))]
pub fn send_notification(_name: &str, _status: &BackgroundStatus, _duration: Duration) {}

/// Unique identifier for a background process.
pub type BackgroundId = u64;

/// Status of a background process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackgroundStatus {
    /// Process is currently running
    Running,
    /// Process completed successfully
    Completed,
    /// Process failed with exit code
    Failed(Option<i32>),
    /// Process was terminated by user
    Terminated,
}

impl BackgroundStatus {
    /// Check if the process has finished.
    pub fn is_finished(&self) -> bool {
        !matches!(self, BackgroundStatus::Running)
    }

    /// Check if the process was successful.
    pub fn is_success(&self) -> bool {
        matches!(self, BackgroundStatus::Completed)
    }
}

/// Information about a background process.
#[derive(Debug, Clone)]
pub struct BackgroundProcess {
    /// Unique ID for this process
    pub id: BackgroundId,
    /// Name of the command
    pub name: String,
    /// The actual command string
    pub command: String,
    /// Process ID (if available)
    pub pid: Option<u32>,
    /// Current status
    pub status: BackgroundStatus,
    /// When the process started
    pub started_at: SystemTime,
    /// How long the process took (set when completed)
    pub duration: Option<Duration>,
    /// Path to output file
    pub output_file: PathBuf,
}

impl BackgroundProcess {
    /// Get a formatted status string.
    pub fn status_string(&self) -> String {
        match &self.status {
            BackgroundStatus::Running => {
                let elapsed = self.started_at.elapsed().unwrap_or_default();
                format!("Running ({:.1?})", elapsed)
            }
            BackgroundStatus::Completed => {
                if let Some(duration) = self.duration {
                    format!("Completed ({:.2?})", duration)
                } else {
                    "Completed".to_string()
                }
            }
            BackgroundStatus::Failed(code) => {
                if let Some(c) = code {
                    format!("Failed (exit {})", c)
                } else {
                    "Failed".to_string()
                }
            }
            BackgroundStatus::Terminated => "Terminated".to_string(),
        }
    }

    /// Get the runtime duration.
    pub fn runtime(&self) -> Duration {
        self.duration.unwrap_or_else(|| self.started_at.elapsed().unwrap_or_default())
    }
}

/// Event from background process manager.
#[derive(Debug, Clone)]
pub enum BackgroundEvent {
    /// A process has started
    Started(BackgroundId),
    /// A process has completed
    Completed(BackgroundId, BackgroundStatus),
}

/// Manager for background processes.
pub struct BackgroundManager {
    /// Currently tracked processes
    processes: Arc<Mutex<HashMap<BackgroundId, BackgroundProcess>>>,
    /// Counter for generating unique IDs
    next_id: Arc<Mutex<BackgroundId>>,
    /// Directory for storing output files
    output_dir: PathBuf,
    /// Channel sender for events
    event_tx: Sender<BackgroundEvent>,
    /// Channel receiver for events
    event_rx: Receiver<BackgroundEvent>,
    /// Active thread handles
    handles: Arc<Mutex<HashMap<BackgroundId, JoinHandle<()>>>>,
}

impl std::fmt::Debug for BackgroundManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let count = self.processes.lock().map(|p| p.len()).unwrap_or(0);
        f.debug_struct("BackgroundManager")
            .field("process_count", &count)
            .field("output_dir", &self.output_dir)
            .finish()
    }
}

impl BackgroundManager {
    /// Create a new background manager.
    pub fn new() -> anyhow::Result<Self> {
        let output_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("palrun")
            .join("background");

        fs::create_dir_all(&output_dir)?;

        let (event_tx, event_rx) = mpsc::channel();

        Ok(Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
            output_dir,
            event_tx,
            event_rx,
            handles: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Spawn a command in the background.
    pub fn spawn(&self, command: Command) -> anyhow::Result<BackgroundId> {
        let id = {
            let mut next_id = self.next_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let timestamp =
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs();

        let output_file = self.output_dir.join(format!("{}-{}.log", id, timestamp));

        let process = BackgroundProcess {
            id,
            name: command.name.clone(),
            command: command.command.clone(),
            pid: None,
            status: BackgroundStatus::Running,
            started_at: SystemTime::now(),
            duration: None,
            output_file: output_file.clone(),
        };

        {
            let mut processes = self.processes.lock().unwrap();
            processes.insert(id, process);
        }

        let event_tx = self.event_tx.clone();
        let processes = Arc::clone(&self.processes);
        let cmd = command.clone();

        let handle = thread::spawn(move || {
            let _ = event_tx.send(BackgroundEvent::Started(id));

            let (shell, shell_arg) = get_shell();

            let mut child = match ProcessCommand::new(shell)
                .arg(shell_arg)
                .arg(&cmd.command)
                .current_dir(
                    cmd.working_dir.as_deref().unwrap_or_else(|| std::path::Path::new(".")),
                )
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(child) => child,
                Err(e) => {
                    // Write error to output file
                    if let Ok(mut file) = File::create(&output_file) {
                        let _ = writeln!(file, "Failed to spawn: {}", e);
                    }

                    let mut procs = processes.lock().unwrap();
                    if let Some(p) = procs.get_mut(&id) {
                        p.status = BackgroundStatus::Failed(None);
                        p.duration = Some(Duration::ZERO);
                    }

                    let _ = event_tx
                        .send(BackgroundEvent::Completed(id, BackgroundStatus::Failed(None)));
                    return;
                }
            };

            // Update PID
            {
                let mut procs = processes.lock().unwrap();
                if let Some(p) = procs.get_mut(&id) {
                    p.pid = Some(child.id());
                }
            }

            let start = Instant::now();

            // Capture output to file
            let output_file_clone = output_file.clone();
            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            let stdout_handle = thread::spawn(move || {
                if let Some(stdout) = stdout {
                    let reader = BufReader::new(stdout);
                    if let Ok(mut file) =
                        fs::OpenOptions::new().create(true).append(true).open(&output_file_clone)
                    {
                        for line in reader.lines().map_while(Result::ok) {
                            let _ = writeln!(file, "{}", line);
                        }
                    }
                }
            });

            let output_file_clone2 = output_file.clone();
            let stderr_handle = thread::spawn(move || {
                if let Some(stderr) = stderr {
                    let reader = BufReader::new(stderr);
                    if let Ok(mut file) =
                        fs::OpenOptions::new().create(true).append(true).open(&output_file_clone2)
                    {
                        for line in reader.lines().map_while(Result::ok) {
                            let _ = writeln!(file, "[stderr] {}", line);
                        }
                    }
                }
            });

            let _ = stdout_handle.join();
            let _ = stderr_handle.join();

            // Wait for process to complete
            let status = match child.wait() {
                Ok(exit_status) => {
                    if exit_status.success() {
                        BackgroundStatus::Completed
                    } else {
                        BackgroundStatus::Failed(exit_status.code())
                    }
                }
                Err(_) => BackgroundStatus::Failed(None),
            };

            let duration = start.elapsed();

            // Update process status
            {
                let mut procs = processes.lock().unwrap();
                if let Some(p) = procs.get_mut(&id) {
                    p.status = status.clone();
                    p.duration = Some(duration);
                }
            }

            let _ = event_tx.send(BackgroundEvent::Completed(id, status));
        });

        {
            let mut handles = self.handles.lock().unwrap();
            handles.insert(id, handle);
        }

        Ok(id)
    }

    /// Get a list of all background processes.
    pub fn list(&self) -> Vec<BackgroundProcess> {
        let processes = self.processes.lock().unwrap();
        processes.values().cloned().collect()
    }

    /// Get a list of running background processes.
    pub fn list_running(&self) -> Vec<BackgroundProcess> {
        let processes = self.processes.lock().unwrap();
        processes
            .values()
            .filter(|p| matches!(p.status, BackgroundStatus::Running))
            .cloned()
            .collect()
    }

    /// Get a specific background process by ID.
    pub fn get(&self, id: BackgroundId) -> Option<BackgroundProcess> {
        let processes = self.processes.lock().unwrap();
        processes.get(&id).cloned()
    }

    /// Get the output of a background process.
    pub fn get_output(&self, id: BackgroundId) -> anyhow::Result<String> {
        let processes = self.processes.lock().unwrap();
        if let Some(process) = processes.get(&id) {
            Ok(fs::read_to_string(&process.output_file).unwrap_or_default())
        } else {
            anyhow::bail!("Process {} not found", id)
        }
    }

    /// Terminate a running background process.
    pub fn terminate(&self, id: BackgroundId) -> anyhow::Result<()> {
        let pid = {
            let processes = self.processes.lock().unwrap();
            processes.get(&id).and_then(|p| p.pid)
        };

        if let Some(pid) = pid {
            #[cfg(unix)]
            {
                // Use kill command to terminate process safely
                let _ = ProcessCommand::new("kill").args(["-TERM", &pid.to_string()]).output();
            }

            #[cfg(windows)]
            {
                let _ =
                    ProcessCommand::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output();
            }

            // Update status
            let mut processes = self.processes.lock().unwrap();
            if let Some(p) = processes.get_mut(&id) {
                p.status = BackgroundStatus::Terminated;
                p.duration = Some(p.started_at.elapsed().unwrap_or_default());
            }

            Ok(())
        } else {
            anyhow::bail!("Process {} not found or already finished", id)
        }
    }

    /// Get the count of running processes.
    pub fn running_count(&self) -> usize {
        let processes = self.processes.lock().unwrap();
        processes.values().filter(|p| matches!(p.status, BackgroundStatus::Running)).count()
    }

    /// Poll for events (non-blocking).
    pub fn poll_events(&self) -> Vec<BackgroundEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.event_rx.try_recv() {
            events.push(event);
        }
        events
    }

    /// Clean up completed processes older than the retention period.
    pub fn cleanup(&self, retention: Duration) -> usize {
        let now = SystemTime::now();
        let mut to_remove = Vec::new();

        {
            let processes = self.processes.lock().unwrap();
            for (id, process) in processes.iter() {
                if process.status.is_finished() {
                    if let Ok(elapsed) = now.duration_since(process.started_at) {
                        if elapsed > retention {
                            to_remove.push(*id);
                        }
                    }
                }
            }
        }

        let mut processes = self.processes.lock().unwrap();
        let mut removed = 0;
        for id in to_remove {
            if let Some(process) = processes.remove(&id) {
                // Delete output file
                let _ = fs::remove_file(&process.output_file);
                removed += 1;
            }
        }

        removed
    }
}

impl Default for BackgroundManager {
    fn default() -> Self {
        Self::new().expect("Failed to create background manager")
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
    fn test_background_manager_creation() {
        let manager = BackgroundManager::new().unwrap();
        assert_eq!(manager.running_count(), 0);
        assert!(manager.list().is_empty());
    }

    #[test]
    fn test_spawn_simple_command() {
        let manager = BackgroundManager::new().unwrap();
        let cmd = Command::new("test", "echo hello");

        let id = manager.spawn(cmd).unwrap();
        assert!(id > 0);

        // Wait a bit for the process to complete
        thread::sleep(Duration::from_millis(500));

        let process = manager.get(id).unwrap();
        assert!(process.status.is_finished());
    }

    #[test]
    fn test_spawn_multiple_commands() {
        let manager = BackgroundManager::new().unwrap();

        let id1 = manager.spawn(Command::new("cmd1", "echo one")).unwrap();
        let id2 = manager.spawn(Command::new("cmd2", "echo two")).unwrap();

        assert_ne!(id1, id2);

        // Wait for completion
        thread::sleep(Duration::from_millis(500));

        let processes = manager.list();
        assert_eq!(processes.len(), 2);
    }

    #[test]
    fn test_output_capture() {
        let manager = BackgroundManager::new().unwrap();
        let cmd = Command::new("test", "echo test_output");

        let id = manager.spawn(cmd).unwrap();

        // Wait for completion
        thread::sleep(Duration::from_millis(500));

        let output = manager.get_output(id).unwrap();
        assert!(output.contains("test_output"));
    }

    #[test]
    fn test_background_status() {
        assert!(!BackgroundStatus::Running.is_finished());
        assert!(BackgroundStatus::Completed.is_finished());
        assert!(BackgroundStatus::Failed(Some(1)).is_finished());
        assert!(BackgroundStatus::Terminated.is_finished());

        assert!(BackgroundStatus::Completed.is_success());
        assert!(!BackgroundStatus::Failed(None).is_success());
    }

    #[test]
    fn test_process_status_string() {
        let mut process = BackgroundProcess {
            id: 1,
            name: "test".to_string(),
            command: "echo".to_string(),
            pid: Some(12345),
            status: BackgroundStatus::Completed,
            started_at: SystemTime::now(),
            duration: Some(Duration::from_secs(5)),
            output_file: PathBuf::from("/tmp/test.log"),
        };

        assert!(process.status_string().contains("Completed"));

        process.status = BackgroundStatus::Failed(Some(1));
        assert!(process.status_string().contains("Failed"));
        assert!(process.status_string().contains("exit 1"));

        process.status = BackgroundStatus::Terminated;
        assert_eq!(process.status_string(), "Terminated");
    }
}
