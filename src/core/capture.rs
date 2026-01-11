//! Output capture and replay module.
//!
//! Handles capturing, storing, and replaying command output with:
//! - ANSI color code preservation
//! - File-based storage with timestamps
//! - Fuzzy search across captured output
//! - Configurable retention policy

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use chrono::{DateTime, Local, Utc};

/// Unique identifier for captured output.
pub type CaptureId = u64;

/// Metadata for captured output.
#[derive(Debug, Clone)]
pub struct CaptureMetadata {
    /// Unique ID
    pub id: CaptureId,
    /// Command name
    pub name: String,
    /// Full command string
    pub command: String,
    /// When the command was executed
    pub executed_at: DateTime<Utc>,
    /// Duration of execution
    pub duration: Duration,
    /// Whether the command succeeded
    pub success: bool,
    /// Exit code (if available)
    pub exit_code: Option<i32>,
    /// Size of captured output in bytes
    pub output_size: usize,
}

impl CaptureMetadata {
    /// Get a formatted timestamp string.
    pub fn timestamp_string(&self) -> String {
        let local: DateTime<Local> = self.executed_at.into();
        local.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    /// Get a relative time string (e.g., "2 minutes ago").
    pub fn relative_time(&self) -> String {
        let now = Utc::now();
        let diff = now.signed_duration_since(self.executed_at);

        if diff.num_seconds() < 60 {
            format!("{}s ago", diff.num_seconds())
        } else if diff.num_minutes() < 60 {
            format!("{}m ago", diff.num_minutes())
        } else if diff.num_hours() < 24 {
            format!("{}h ago", diff.num_hours())
        } else {
            format!("{}d ago", diff.num_days())
        }
    }
}

/// Captured command output.
#[derive(Debug, Clone)]
pub struct CapturedOutput {
    /// Metadata about the capture
    pub metadata: CaptureMetadata,
    /// Standard output (with ANSI codes preserved)
    pub stdout: String,
    /// Standard error (with ANSI codes preserved)
    pub stderr: String,
}

impl CapturedOutput {
    /// Get combined output (stdout + stderr).
    pub fn combined(&self) -> String {
        let mut combined = self.stdout.clone();
        if !self.stderr.is_empty() {
            if !combined.is_empty() && !combined.ends_with('\n') {
                combined.push('\n');
            }
            combined.push_str(&self.stderr);
        }
        combined
    }

    /// Strip ANSI codes from output.
    pub fn plain_stdout(&self) -> String {
        strip_ansi_codes(&self.stdout)
    }

    /// Strip ANSI codes from stderr.
    pub fn plain_stderr(&self) -> String {
        strip_ansi_codes(&self.stderr)
    }

    /// Get line count.
    pub fn line_count(&self) -> usize {
        self.stdout.lines().count() + self.stderr.lines().count()
    }
}

/// Search result for output search.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The capture this result belongs to
    pub capture_id: CaptureId,
    /// Command name
    pub command_name: String,
    /// Line number (1-indexed)
    pub line_number: usize,
    /// The matching line
    pub line: String,
    /// Whether this is from stderr
    pub is_stderr: bool,
}

/// Output capture manager.
pub struct CaptureManager {
    /// Directory for storing captures
    capture_dir: PathBuf,
    /// Counter for generating unique IDs
    next_id: CaptureId,
    /// Retention period (default: 7 days)
    retention: Duration,
}

impl std::fmt::Debug for CaptureManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CaptureManager")
            .field("capture_dir", &self.capture_dir)
            .field("next_id", &self.next_id)
            .finish()
    }
}

impl CaptureManager {
    /// Create a new capture manager with default directory.
    pub fn new() -> anyhow::Result<Self> {
        let capture_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("palrun")
            .join("captures");

        Self::with_dir(capture_dir)
    }

    /// Create a new capture manager with a custom directory.
    pub fn with_dir(capture_dir: PathBuf) -> anyhow::Result<Self> {
        fs::create_dir_all(&capture_dir)?;

        // Find the highest existing ID
        let next_id = Self::find_max_id(&capture_dir) + 1;

        Ok(Self {
            capture_dir,
            next_id,
            retention: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
        })
    }

    /// Set the retention period.
    #[must_use]
    pub fn with_retention(mut self, retention: Duration) -> Self {
        self.retention = retention;
        self
    }

    /// Find the maximum capture ID in the directory.
    fn find_max_id(dir: &PathBuf) -> CaptureId {
        fs::read_dir(dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .filter_map(|e| {
                        e.file_name()
                            .to_str()
                            .and_then(|name| name.split('-').next())
                            .and_then(|id| id.parse::<CaptureId>().ok())
                    })
                    .max()
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }

    /// Capture command output.
    pub fn capture(
        &mut self,
        name: &str,
        command: &str,
        stdout: &str,
        stderr: &str,
        duration: Duration,
        exit_code: Option<i32>,
    ) -> anyhow::Result<CaptureId> {
        let id = self.next_id;
        self.next_id += 1;

        let now = Utc::now();
        let timestamp = now.timestamp();

        let metadata = CaptureMetadata {
            id,
            name: name.to_string(),
            command: command.to_string(),
            executed_at: now,
            duration,
            success: exit_code == Some(0),
            exit_code,
            output_size: stdout.len() + stderr.len(),
        };

        // Save metadata
        let meta_path = self.capture_dir.join(format!("{}-{}.meta", id, timestamp));
        let meta_json = serde_json::json!({
            "id": metadata.id,
            "name": metadata.name,
            "command": metadata.command,
            "executed_at": metadata.executed_at.to_rfc3339(),
            "duration_ms": metadata.duration.as_millis(),
            "success": metadata.success,
            "exit_code": metadata.exit_code,
            "output_size": metadata.output_size,
        });
        fs::write(&meta_path, meta_json.to_string())?;

        // Save stdout
        if !stdout.is_empty() {
            let stdout_path = self.capture_dir.join(format!("{}-{}.stdout", id, timestamp));
            fs::write(&stdout_path, stdout)?;
        }

        // Save stderr
        if !stderr.is_empty() {
            let stderr_path = self.capture_dir.join(format!("{}-{}.stderr", id, timestamp));
            fs::write(&stderr_path, stderr)?;
        }

        Ok(id)
    }

    /// List all captured outputs (most recent first).
    pub fn list(&self) -> Vec<CaptureMetadata> {
        let mut captures: Vec<CaptureMetadata> = fs::read_dir(&self.capture_dir)
            .ok()
            .map(|entries| {
                entries
                    .filter_map(Result::ok)
                    .filter(|e| e.path().extension().map(|ext| ext == "meta").unwrap_or(false))
                    .filter_map(|e| self.read_metadata(&e.path()))
                    .collect()
            })
            .unwrap_or_default();

        // Sort by execution time (most recent first)
        captures.sort_by(|a, b| b.executed_at.cmp(&a.executed_at));
        captures
    }

    /// Get captured output by ID.
    pub fn get(&self, id: CaptureId) -> Option<CapturedOutput> {
        // Find the metadata file for this ID
        let meta_file = fs::read_dir(&self.capture_dir).ok()?.filter_map(Result::ok).find(|e| {
            e.file_name()
                .to_str()
                .map(|name| name.starts_with(&format!("{}-", id)) && name.ends_with(".meta"))
                .unwrap_or(false)
        })?;

        let metadata = self.read_metadata(&meta_file.path())?;

        // Get the timestamp from the filename
        let filename = meta_file.file_name();
        let filename_str = filename.to_str()?;
        let timestamp = filename_str.strip_prefix(&format!("{}-", id))?.strip_suffix(".meta")?;

        // Read stdout
        let stdout_path = self.capture_dir.join(format!("{}-{}.stdout", id, timestamp));
        let stdout = fs::read_to_string(&stdout_path).unwrap_or_default();

        // Read stderr
        let stderr_path = self.capture_dir.join(format!("{}-{}.stderr", id, timestamp));
        let stderr = fs::read_to_string(&stderr_path).unwrap_or_default();

        Some(CapturedOutput { metadata, stdout, stderr })
    }

    /// Get the most recent capture.
    pub fn get_latest(&self) -> Option<CapturedOutput> {
        self.list().first().and_then(|m| self.get(m.id))
    }

    /// Search across all captured outputs.
    pub fn search(&self, pattern: &str) -> Vec<SearchResult> {
        let pattern_lower = pattern.to_lowercase();
        let mut results = Vec::new();

        for meta in self.list() {
            if let Some(capture) = self.get(meta.id) {
                // Search stdout
                for (i, line) in capture.stdout.lines().enumerate() {
                    if strip_ansi_codes(line).to_lowercase().contains(&pattern_lower) {
                        results.push(SearchResult {
                            capture_id: meta.id,
                            command_name: meta.name.clone(),
                            line_number: i + 1,
                            line: line.to_string(),
                            is_stderr: false,
                        });
                    }
                }

                // Search stderr
                for (i, line) in capture.stderr.lines().enumerate() {
                    if strip_ansi_codes(line).to_lowercase().contains(&pattern_lower) {
                        results.push(SearchResult {
                            capture_id: meta.id,
                            command_name: meta.name.clone(),
                            line_number: i + 1,
                            line: line.to_string(),
                            is_stderr: true,
                        });
                    }
                }
            }
        }

        results
    }

    /// Delete a specific capture.
    pub fn delete(&self, id: CaptureId) -> anyhow::Result<()> {
        let files: Vec<_> = fs::read_dir(&self.capture_dir)?
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|name| name.starts_with(&format!("{}-", id)))
                    .unwrap_or(false)
            })
            .collect();

        for file in files {
            fs::remove_file(file.path())?;
        }

        Ok(())
    }

    /// Clean up captures older than the retention period.
    pub fn cleanup(&self) -> anyhow::Result<usize> {
        let now = Utc::now();
        let mut removed = 0;

        for meta in self.list() {
            let age = now.signed_duration_since(meta.executed_at);
            if age.to_std().unwrap_or_default() > self.retention {
                if self.delete(meta.id).is_ok() {
                    removed += 1;
                }
            }
        }

        Ok(removed)
    }

    /// Read metadata from a file.
    fn read_metadata(&self, path: &PathBuf) -> Option<CaptureMetadata> {
        let content = fs::read_to_string(path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;

        Some(CaptureMetadata {
            id: json["id"].as_u64()?,
            name: json["name"].as_str()?.to_string(),
            command: json["command"].as_str()?.to_string(),
            executed_at: DateTime::parse_from_rfc3339(json["executed_at"].as_str()?)
                .ok()?
                .with_timezone(&Utc),
            duration: Duration::from_millis(json["duration_ms"].as_u64()?),
            success: json["success"].as_bool()?,
            exit_code: json["exit_code"].as_i64().map(|c| c as i32),
            output_size: json["output_size"].as_u64()? as usize,
        })
    }
}

impl Default for CaptureManager {
    fn default() -> Self {
        Self::new().expect("Failed to create capture manager")
    }
}

/// Strip ANSI escape codes from a string.
pub fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip escape sequence
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                              // Skip until we hit a letter (which terminates the sequence)
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi_codes() {
        // Test basic ANSI stripping
        assert_eq!(strip_ansi_codes("\x1b[31mred\x1b[0m"), "red");
        assert_eq!(strip_ansi_codes("\x1b[1;32mbold green\x1b[0m"), "bold green");
        assert_eq!(strip_ansi_codes("no codes here"), "no codes here");
        assert_eq!(strip_ansi_codes("\x1b[38;5;196mextended\x1b[0m"), "extended");
    }

    #[test]
    fn test_capture_metadata_relative_time() {
        let meta = CaptureMetadata {
            id: 1,
            name: "test".to_string(),
            command: "echo test".to_string(),
            executed_at: Utc::now(),
            duration: Duration::from_secs(1),
            success: true,
            exit_code: Some(0),
            output_size: 100,
        };

        let time_str = meta.relative_time();
        assert!(time_str.contains("s ago") || time_str.contains("0s ago"));
    }

    #[test]
    fn test_captured_output_combined() {
        let output = CapturedOutput {
            metadata: CaptureMetadata {
                id: 1,
                name: "test".to_string(),
                command: "echo test".to_string(),
                executed_at: Utc::now(),
                duration: Duration::from_secs(1),
                success: true,
                exit_code: Some(0),
                output_size: 100,
            },
            stdout: "stdout content".to_string(),
            stderr: "stderr content".to_string(),
        };

        let combined = output.combined();
        assert!(combined.contains("stdout content"));
        assert!(combined.contains("stderr content"));
    }

    #[test]
    fn test_captured_output_line_count() {
        let output = CapturedOutput {
            metadata: CaptureMetadata {
                id: 1,
                name: "test".to_string(),
                command: "echo test".to_string(),
                executed_at: Utc::now(),
                duration: Duration::from_secs(1),
                success: true,
                exit_code: Some(0),
                output_size: 100,
            },
            stdout: "line1\nline2\nline3".to_string(),
            stderr: "error1\nerror2".to_string(),
        };

        assert_eq!(output.line_count(), 5);
    }

    #[test]
    fn test_capture_manager_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let manager = CaptureManager::with_dir(temp_dir.path().to_path_buf()).unwrap();
        assert!(manager.capture_dir.exists());
    }

    #[test]
    fn test_capture_and_retrieve() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut manager = CaptureManager::with_dir(temp_dir.path().to_path_buf()).unwrap();

        let id = manager
            .capture("test_cmd", "echo hello", "hello\n", "", Duration::from_millis(100), Some(0))
            .unwrap();

        let captured = manager.get(id).unwrap();
        assert_eq!(captured.metadata.name, "test_cmd");
        assert_eq!(captured.stdout, "hello\n");
        assert!(captured.metadata.success);
    }

    #[test]
    fn test_search() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut manager = CaptureManager::with_dir(temp_dir.path().to_path_buf()).unwrap();

        let id = manager
            .capture(
                "search_test",
                "echo searchable",
                "this is searchable content\nmore lines\n",
                "",
                Duration::from_millis(100),
                Some(0),
            )
            .unwrap();

        let results = manager.search("searchable");
        assert!(!results.is_empty());
        assert!(results[0].line.contains("searchable"));
    }
}
