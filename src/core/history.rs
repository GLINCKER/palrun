//! Command history and favorites management.
//!
//! Tracks command execution history with frecency scoring and manages
//! favorite commands for quick access.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// History and favorites manager.
#[derive(Debug)]
pub struct HistoryManager {
    /// Path to the history file
    history_path: PathBuf,
    /// Command history data
    history: CommandHistory,
    /// Maximum number of history entries to keep
    max_entries: usize,
}

/// Stored command history data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandHistory {
    /// History entries keyed by command ID
    pub entries: HashMap<String, HistoryEntry>,
    /// Favorite command IDs
    pub favorites: Vec<String>,
    /// Version for future migrations
    #[serde(default)]
    pub version: u32,
}

/// A single history entry for a command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Command ID
    pub command_id: String,
    /// Command name (for display when command no longer exists)
    pub command_name: String,
    /// Number of times executed
    pub execution_count: u32,
    /// Last execution timestamp (Unix epoch seconds)
    pub last_executed: u64,
    /// First execution timestamp
    pub first_executed: u64,
    /// Total execution time in milliseconds (for averages)
    pub total_duration_ms: u64,
    /// Number of successful executions
    pub success_count: u32,
    /// Number of failed executions
    pub failure_count: u32,
}

impl HistoryEntry {
    /// Create a new history entry.
    pub fn new(command_id: String, command_name: String) -> Self {
        let now = current_timestamp();
        Self {
            command_id,
            command_name,
            execution_count: 0,
            last_executed: now,
            first_executed: now,
            total_duration_ms: 0,
            success_count: 0,
            failure_count: 0,
        }
    }

    /// Record a command execution.
    pub fn record_execution(&mut self, duration_ms: u64, success: bool) {
        self.execution_count += 1;
        self.last_executed = current_timestamp();
        self.total_duration_ms += duration_ms;
        if success {
            self.success_count += 1;
        } else {
            self.failure_count += 1;
        }
    }

    /// Calculate frecency score.
    ///
    /// Frecency = frequency * recency_weight
    /// - Higher frequency = higher score
    /// - More recent = higher weight
    pub fn frecency_score(&self) -> f64 {
        let now = current_timestamp();
        let age_seconds = now.saturating_sub(self.last_executed);

        // Recency weight decays over time
        // - Used within last hour: weight = 4.0
        // - Used within last day: weight = 2.0
        // - Used within last week: weight = 1.0
        // - Older: weight decays further
        let recency_weight = if age_seconds < 3600 {
            4.0 // Last hour
        } else if age_seconds < 86400 {
            2.0 // Last day
        } else if age_seconds < 604800 {
            1.0 // Last week
        } else if age_seconds < 2592000 {
            0.5 // Last month
        } else {
            0.25 // Older
        };

        // Frequency factor (log scale to prevent very frequent commands from dominating)
        let frequency_factor = (self.execution_count as f64 + 1.0).ln();

        frequency_factor * recency_weight
    }

    /// Get average execution duration.
    pub fn average_duration(&self) -> Option<Duration> {
        if self.execution_count > 0 {
            let avg_ms = self.total_duration_ms / self.execution_count as u64;
            Some(Duration::from_millis(avg_ms))
        } else {
            None
        }
    }

    /// Get success rate as a percentage.
    pub fn success_rate(&self) -> Option<f64> {
        if self.execution_count > 0 {
            Some((self.success_count as f64 / self.execution_count as f64) * 100.0)
        } else {
            None
        }
    }

    /// Get a human-readable "last used" string.
    pub fn last_used_display(&self) -> String {
        let now = current_timestamp();
        let age_seconds = now.saturating_sub(self.last_executed);

        if age_seconds < 60 {
            "just now".to_string()
        } else if age_seconds < 3600 {
            let mins = age_seconds / 60;
            format!("{}m ago", mins)
        } else if age_seconds < 86400 {
            let hours = age_seconds / 3600;
            format!("{}h ago", hours)
        } else if age_seconds < 604800 {
            let days = age_seconds / 86400;
            format!("{}d ago", days)
        } else if age_seconds < 2592000 {
            let weeks = age_seconds / 604800;
            format!("{}w ago", weeks)
        } else {
            let months = age_seconds / 2592000;
            format!("{}mo ago", months)
        }
    }
}

impl HistoryManager {
    /// Create a new history manager.
    pub fn new() -> anyhow::Result<Self> {
        let history_path = Self::default_history_path()?;
        let history = Self::load_history(&history_path).unwrap_or_default();

        Ok(Self {
            history_path,
            history,
            max_entries: 1000,
        })
    }

    /// Create a history manager with a custom path (for testing).
    pub fn with_path(path: PathBuf) -> anyhow::Result<Self> {
        let history = Self::load_history(&path).unwrap_or_default();

        Ok(Self {
            history_path: path,
            history,
            max_entries: 1000,
        })
    }

    /// Get the default history file path.
    fn default_history_path() -> anyhow::Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let palrun_dir = home.join(".palrun");

        // Ensure directory exists
        if !palrun_dir.exists() {
            fs::create_dir_all(&palrun_dir)?;
        }

        Ok(palrun_dir.join("history.json"))
    }

    /// Load history from file.
    fn load_history(path: &PathBuf) -> anyhow::Result<CommandHistory> {
        if !path.exists() {
            return Ok(CommandHistory::default());
        }

        let content = fs::read_to_string(path)?;
        let history: CommandHistory = serde_json::from_str(&content)?;
        Ok(history)
    }

    /// Save history to file.
    pub fn save(&self) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(&self.history)?;

        // Ensure parent directory exists
        if let Some(parent) = self.history_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.history_path, content)?;
        Ok(())
    }

    /// Record a command execution.
    pub fn record_execution(
        &mut self,
        command_id: &str,
        command_name: &str,
        duration_ms: u64,
        success: bool,
    ) {
        let entry = self.history.entries
            .entry(command_id.to_string())
            .or_insert_with(|| HistoryEntry::new(command_id.to_string(), command_name.to_string()));

        entry.record_execution(duration_ms, success);

        // Update name in case it changed
        entry.command_name = command_name.to_string();

        // Prune old entries if needed
        self.prune_old_entries();
    }

    /// Get history entry for a command.
    pub fn get_entry(&self, command_id: &str) -> Option<&HistoryEntry> {
        self.history.entries.get(command_id)
    }

    /// Get frecency score for a command.
    pub fn get_frecency(&self, command_id: &str) -> f64 {
        self.history.entries
            .get(command_id)
            .map(|e| e.frecency_score())
            .unwrap_or(0.0)
    }

    /// Get sorted command IDs by frecency (highest first).
    pub fn get_by_frecency(&self) -> Vec<&str> {
        let mut entries: Vec<_> = self.history.entries.iter().collect();
        entries.sort_by(|a, b| {
            b.1.frecency_score()
                .partial_cmp(&a.1.frecency_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        entries.into_iter().map(|(id, _)| id.as_str()).collect()
    }

    /// Get recent commands (sorted by last_executed, most recent first).
    pub fn get_recent(&self, limit: usize) -> Vec<&HistoryEntry> {
        let mut entries: Vec<_> = self.history.entries.values().collect();
        // Sort by last_executed descending, with command_id as tie-breaker for determinism
        entries.sort_by(|a, b| {
            b.last_executed.cmp(&a.last_executed)
                .then_with(|| b.command_id.cmp(&a.command_id))
        });
        entries.into_iter().take(limit).collect()
    }

    /// Get most frequently used commands.
    pub fn get_frequent(&self, limit: usize) -> Vec<&HistoryEntry> {
        let mut entries: Vec<_> = self.history.entries.values().collect();
        entries.sort_by(|a, b| b.execution_count.cmp(&a.execution_count));
        entries.into_iter().take(limit).collect()
    }

    // --- Favorites management ---

    /// Add a command to favorites.
    pub fn add_favorite(&mut self, command_id: &str) {
        if !self.history.favorites.contains(&command_id.to_string()) {
            self.history.favorites.push(command_id.to_string());
        }
    }

    /// Remove a command from favorites.
    pub fn remove_favorite(&mut self, command_id: &str) {
        self.history.favorites.retain(|id| id != command_id);
    }

    /// Toggle favorite status for a command.
    pub fn toggle_favorite(&mut self, command_id: &str) -> bool {
        if self.is_favorite(command_id) {
            self.remove_favorite(command_id);
            false
        } else {
            self.add_favorite(command_id);
            true
        }
    }

    /// Check if a command is a favorite.
    pub fn is_favorite(&self, command_id: &str) -> bool {
        self.history.favorites.contains(&command_id.to_string())
    }

    /// Get all favorite command IDs.
    pub fn get_favorites(&self) -> &[String] {
        &self.history.favorites
    }

    /// Get count of favorites.
    pub fn favorites_count(&self) -> usize {
        self.history.favorites.len()
    }

    // --- Utility methods ---

    /// Prune old entries to stay within max_entries limit.
    fn prune_old_entries(&mut self) {
        if self.history.entries.len() <= self.max_entries {
            return;
        }

        // Sort by frecency and keep only top entries
        let mut entries: Vec<_> = self.history.entries.drain().collect();
        entries.sort_by(|a, b| {
            b.1.frecency_score()
                .partial_cmp(&a.1.frecency_score())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Keep favorites regardless of frecency
        let favorites_set: std::collections::HashSet<_> =
            self.history.favorites.iter().cloned().collect();

        // Partition into favorites and non-favorites
        let (favorites, non_favorites): (Vec<_>, Vec<_>) = entries
            .into_iter()
            .partition(|(id, _)| favorites_set.contains(id));

        // Keep all favorites plus top non-favorites up to max_entries
        let remaining_slots = self.max_entries.saturating_sub(favorites.len());
        self.history.entries = favorites
            .into_iter()
            .chain(non_favorites.into_iter().take(remaining_slots))
            .collect();
    }

    /// Clear all history (but keep favorites).
    pub fn clear_history(&mut self) {
        let favorites = self.history.favorites.clone();
        self.history.entries.retain(|id, _| favorites.contains(id));
    }

    /// Get total command count in history.
    pub fn history_count(&self) -> usize {
        self.history.entries.len()
    }

    /// Check if history has any entries.
    pub fn has_history(&self) -> bool {
        !self.history.entries.is_empty()
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            history_path: PathBuf::from(".palrun/history.json"),
            history: CommandHistory::default(),
            max_entries: 1000,
        })
    }
}

/// Get current Unix timestamp in seconds.
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_history_entry_creation() {
        let entry = HistoryEntry::new("cmd1".to_string(), "test command".to_string());
        assert_eq!(entry.command_id, "cmd1");
        assert_eq!(entry.execution_count, 0);
    }

    #[test]
    fn test_record_execution() {
        let mut entry = HistoryEntry::new("cmd1".to_string(), "test".to_string());
        entry.record_execution(100, true);

        assert_eq!(entry.execution_count, 1);
        assert_eq!(entry.success_count, 1);
        assert_eq!(entry.failure_count, 0);
        assert_eq!(entry.total_duration_ms, 100);

        entry.record_execution(200, false);
        assert_eq!(entry.execution_count, 2);
        assert_eq!(entry.success_count, 1);
        assert_eq!(entry.failure_count, 1);
    }

    #[test]
    fn test_frecency_score() {
        let mut entry = HistoryEntry::new("cmd1".to_string(), "test".to_string());

        // Fresh entry with no executions
        let initial_score = entry.frecency_score();

        // After execution, score should be higher
        entry.record_execution(100, true);
        let after_score = entry.frecency_score();
        assert!(after_score > initial_score);

        // Multiple executions increase score
        entry.record_execution(100, true);
        entry.record_execution(100, true);
        let frequent_score = entry.frecency_score();
        assert!(frequent_score > after_score);
    }

    #[test]
    fn test_success_rate() {
        let mut entry = HistoryEntry::new("cmd1".to_string(), "test".to_string());

        assert!(entry.success_rate().is_none());

        entry.record_execution(100, true);
        entry.record_execution(100, true);
        entry.record_execution(100, false);

        let rate = entry.success_rate().unwrap();
        assert!((rate - 66.67).abs() < 1.0); // ~66.67%
    }

    #[test]
    fn test_last_used_display() {
        let entry = HistoryEntry::new("cmd1".to_string(), "test".to_string());
        let display = entry.last_used_display();
        assert_eq!(display, "just now");
    }

    #[test]
    fn test_history_manager_favorites() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("history.json");
        let mut manager = HistoryManager::with_path(path).unwrap();

        assert!(!manager.is_favorite("cmd1"));

        manager.add_favorite("cmd1");
        assert!(manager.is_favorite("cmd1"));
        assert_eq!(manager.favorites_count(), 1);

        manager.add_favorite("cmd1"); // Duplicate should be ignored
        assert_eq!(manager.favorites_count(), 1);

        manager.remove_favorite("cmd1");
        assert!(!manager.is_favorite("cmd1"));
        assert_eq!(manager.favorites_count(), 0);
    }

    #[test]
    fn test_toggle_favorite() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("history.json");
        let mut manager = HistoryManager::with_path(path).unwrap();

        let is_fav = manager.toggle_favorite("cmd1");
        assert!(is_fav);
        assert!(manager.is_favorite("cmd1"));

        let is_fav = manager.toggle_favorite("cmd1");
        assert!(!is_fav);
        assert!(!manager.is_favorite("cmd1"));
    }

    #[test]
    fn test_record_and_retrieve() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("history.json");
        let mut manager = HistoryManager::with_path(path).unwrap();

        manager.record_execution("cmd1", "test command", 100, true);
        manager.record_execution("cmd2", "another command", 200, true);
        manager.record_execution("cmd1", "test command", 150, true);

        let entry = manager.get_entry("cmd1").unwrap();
        assert_eq!(entry.execution_count, 2);
        assert_eq!(entry.total_duration_ms, 250);
    }

    #[test]
    fn test_get_recent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("history.json");
        let mut manager = HistoryManager::with_path(path).unwrap();

        manager.record_execution("cmd1", "first", 100, true);
        manager.record_execution("cmd2", "second", 100, true);
        manager.record_execution("cmd3", "third", 100, true);

        let recent = manager.get_recent(2);
        assert_eq!(recent.len(), 2);
        // Most recent should be first
        assert_eq!(recent[0].command_id, "cmd3");
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("history.json");

        // Create and save
        {
            let mut manager = HistoryManager::with_path(path.clone()).unwrap();
            manager.record_execution("cmd1", "test", 100, true);
            manager.add_favorite("cmd1");
            manager.save().unwrap();
        }

        // Load and verify
        {
            let manager = HistoryManager::with_path(path).unwrap();
            assert!(manager.is_favorite("cmd1"));
            let entry = manager.get_entry("cmd1").unwrap();
            assert_eq!(entry.execution_count, 1);
        }
    }

    #[test]
    fn test_clear_history() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("history.json");
        let mut manager = HistoryManager::with_path(path).unwrap();

        manager.record_execution("cmd1", "test", 100, true);
        manager.record_execution("cmd2", "test2", 100, true);
        manager.add_favorite("cmd1");

        manager.clear_history();

        // Favorites should be preserved
        assert!(manager.is_favorite("cmd1"));
        assert!(manager.get_entry("cmd1").is_some());
        // Non-favorites should be cleared
        assert!(manager.get_entry("cmd2").is_none());
    }
}
