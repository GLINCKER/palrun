//! Offline mode support and operation queuing.
//!
//! Provides mechanisms for Palrun to queue operations when offline
//! and sync them when connectivity is restored.

use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

/// Types of operations that can be queued for offline execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueuedOperation {
    /// AI request to be sent when online
    AiRequest { prompt: String, context: Option<String> },
    /// Sync history to cloud
    SyncHistory { entries_count: usize },
    /// Send analytics/telemetry
    SendAnalytics { event_type: String, data: String },
    /// Webhook notification
    Webhook { url: String, payload: String },
    /// Custom operation
    Custom { operation_type: String, data: String },
}

impl std::fmt::Display for QueuedOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueuedOperation::AiRequest { .. } => write!(f, "AI Request"),
            QueuedOperation::SyncHistory { entries_count } => {
                write!(f, "Sync {} history entries", entries_count)
            }
            QueuedOperation::SendAnalytics { event_type, .. } => {
                write!(f, "Send analytics: {}", event_type)
            }
            QueuedOperation::Webhook { url, .. } => write!(f, "Webhook to {}", url),
            QueuedOperation::Custom { operation_type, .. } => {
                write!(f, "Custom: {}", operation_type)
            }
        }
    }
}

/// A queued operation with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueEntry {
    /// Unique ID for this entry
    pub id: u64,
    /// The operation to perform
    pub operation: QueuedOperation,
    /// When the operation was queued
    pub queued_at: SystemTime,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Maximum retries before giving up
    pub max_retries: u32,
    /// Priority (higher = more important)
    pub priority: u8,
}

impl QueueEntry {
    /// Create a new queue entry.
    pub fn new(operation: QueuedOperation) -> Self {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Self {
            id: COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            operation,
            queued_at: SystemTime::now(),
            retry_count: 0,
            max_retries: 3,
            priority: 5,
        }
    }

    /// Set priority (0-10, higher = more important).
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(10);
        self
    }

    /// Set max retries.
    pub fn with_max_retries(mut self, max: u32) -> Self {
        self.max_retries = max;
        self
    }

    /// Check if this entry has exceeded max retries.
    pub fn is_exhausted(&self) -> bool {
        self.retry_count >= self.max_retries
    }

    /// Get age of this entry.
    pub fn age(&self) -> Duration {
        self.queued_at.elapsed().unwrap_or_default()
    }
}

/// Offline operation queue with persistence.
#[derive(Debug)]
pub struct OfflineQueue {
    /// Queued operations
    queue: VecDeque<QueueEntry>,
    /// Path for persistence
    persistence_path: Option<PathBuf>,
    /// Maximum queue size
    max_size: usize,
    /// Whether queue is paused
    paused: bool,
}

impl Default for OfflineQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl OfflineQueue {
    /// Create a new offline queue.
    pub fn new() -> Self {
        Self { queue: VecDeque::new(), persistence_path: None, max_size: 1000, paused: false }
    }

    /// Create with persistence to a file.
    pub fn with_persistence(path: PathBuf) -> Self {
        let mut queue = Self::new();
        queue.persistence_path = Some(path.clone());

        // Try to load existing queue
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(entries) = serde_json::from_str::<Vec<QueueEntry>>(&data) {
                queue.queue = entries.into();
            }
        }

        queue
    }

    /// Set maximum queue size.
    pub fn with_max_size(mut self, max: usize) -> Self {
        self.max_size = max;
        self
    }

    /// Enqueue an operation.
    pub fn enqueue(&mut self, operation: QueuedOperation) -> Option<u64> {
        if self.queue.len() >= self.max_size {
            // Remove oldest low-priority item
            if let Some(idx) = self.find_lowest_priority() {
                self.queue.remove(idx);
            } else {
                return None; // Queue full, can't add
            }
        }

        let entry = QueueEntry::new(operation);
        let id = entry.id;
        self.queue.push_back(entry);
        self.persist();
        Some(id)
    }

    /// Enqueue with custom entry settings.
    pub fn enqueue_entry(&mut self, entry: QueueEntry) -> Option<u64> {
        if self.queue.len() >= self.max_size {
            if let Some(idx) = self.find_lowest_priority() {
                self.queue.remove(idx);
            } else {
                return None;
            }
        }

        let id = entry.id;
        self.queue.push_back(entry);
        self.persist();
        Some(id)
    }

    /// Dequeue the next operation (highest priority first).
    pub fn dequeue(&mut self) -> Option<QueueEntry> {
        if self.paused {
            return None;
        }

        // Find highest priority entry
        let idx = self.find_highest_priority()?;
        let entry = self.queue.remove(idx)?;
        self.persist();
        Some(entry)
    }

    /// Peek at the next operation without removing.
    pub fn peek(&self) -> Option<&QueueEntry> {
        if self.paused {
            return None;
        }
        self.find_highest_priority().and_then(|idx| self.queue.get(idx))
    }

    /// Re-queue a failed operation (increment retry count).
    pub fn requeue(&mut self, mut entry: QueueEntry) -> bool {
        entry.retry_count += 1;
        if entry.is_exhausted() {
            return false;
        }
        self.queue.push_back(entry);
        self.persist();
        true
    }

    /// Remove an entry by ID.
    pub fn remove(&mut self, id: u64) -> Option<QueueEntry> {
        let idx = self.queue.iter().position(|e| e.id == id)?;
        let entry = self.queue.remove(idx)?;
        self.persist();
        Some(entry)
    }

    /// Get queue length.
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Check if queue is empty.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// Pause queue processing.
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume queue processing.
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Check if queue is paused.
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.queue.clear();
        self.persist();
    }

    /// Get all entries (for display).
    pub fn entries(&self) -> impl Iterator<Item = &QueueEntry> {
        self.queue.iter()
    }

    /// Get summary of queued operations.
    pub fn summary(&self) -> QueueSummary {
        let mut summary = QueueSummary::default();
        for entry in &self.queue {
            summary.total += 1;
            match &entry.operation {
                QueuedOperation::AiRequest { .. } => summary.ai_requests += 1,
                QueuedOperation::SyncHistory { .. } => summary.sync_ops += 1,
                QueuedOperation::SendAnalytics { .. } => summary.analytics += 1,
                QueuedOperation::Webhook { .. } => summary.webhooks += 1,
                QueuedOperation::Custom { .. } => summary.custom += 1,
            }
        }
        summary
    }

    /// Find index of highest priority entry.
    fn find_highest_priority(&self) -> Option<usize> {
        self.queue.iter().enumerate().max_by_key(|(_, e)| e.priority).map(|(i, _)| i)
    }

    /// Find index of lowest priority entry.
    fn find_lowest_priority(&self) -> Option<usize> {
        self.queue.iter().enumerate().min_by_key(|(_, e)| e.priority).map(|(i, _)| i)
    }

    /// Persist queue to disk.
    fn persist(&self) {
        if let Some(ref path) = self.persistence_path {
            let entries: Vec<_> = self.queue.iter().cloned().collect();
            if let Ok(data) = serde_json::to_string_pretty(&entries) {
                let _ = fs::write(path, data);
            }
        }
    }
}

/// Summary of queue contents.
#[derive(Debug, Default)]
pub struct QueueSummary {
    pub total: usize,
    pub ai_requests: usize,
    pub sync_ops: usize,
    pub analytics: usize,
    pub webhooks: usize,
    pub custom: usize,
}

impl std::fmt::Display for QueueSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.total == 0 {
            write!(f, "Queue empty")
        } else {
            write!(f, "{} queued", self.total)
        }
    }
}

/// Offline mode manager combining network detection and queue.
#[derive(Debug)]
pub struct OfflineManager {
    /// Whether currently offline
    is_offline: bool,
    /// Operation queue
    queue: OfflineQueue,
    /// Last connectivity check
    last_check: Option<SystemTime>,
    /// Check interval
    check_interval: Duration,
}

impl Default for OfflineManager {
    fn default() -> Self {
        Self::new()
    }
}

impl OfflineManager {
    /// Create a new offline manager.
    pub fn new() -> Self {
        Self {
            is_offline: false,
            queue: OfflineQueue::new(),
            last_check: None,
            check_interval: Duration::from_secs(30),
        }
    }

    /// Create with a persistent queue.
    pub fn with_queue_persistence(mut self, path: PathBuf) -> Self {
        self.queue = OfflineQueue::with_persistence(path);
        self
    }

    /// Check if currently offline.
    pub fn is_offline(&self) -> bool {
        self.is_offline
    }

    /// Set offline status.
    pub fn set_offline(&mut self, offline: bool) {
        let was_offline = self.is_offline;
        self.is_offline = offline;

        // If coming back online, resume queue
        if was_offline && !offline {
            self.queue.resume();
        } else if offline {
            self.queue.pause();
        }
    }

    /// Queue an operation for when online.
    pub fn queue_operation(&mut self, operation: QueuedOperation) -> Option<u64> {
        self.queue.enqueue(operation)
    }

    /// Get the operation queue.
    pub fn queue(&self) -> &OfflineQueue {
        &self.queue
    }

    /// Get mutable operation queue.
    pub fn queue_mut(&mut self) -> &mut OfflineQueue {
        &mut self.queue
    }

    /// Check if should check connectivity.
    pub fn should_check_connectivity(&self) -> bool {
        match self.last_check {
            Some(last) => last.elapsed().unwrap_or_default() >= self.check_interval,
            None => true,
        }
    }

    /// Mark connectivity as checked.
    pub fn mark_checked(&mut self) {
        self.last_check = Some(SystemTime::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_entry_creation() {
        let entry = QueueEntry::new(QueuedOperation::AiRequest {
            prompt: "test".to_string(),
            context: None,
        });
        assert_eq!(entry.retry_count, 0);
        assert_eq!(entry.max_retries, 3);
    }

    #[test]
    fn test_queue_operations() {
        let mut queue = OfflineQueue::new();

        let id = queue.enqueue(QueuedOperation::SyncHistory { entries_count: 5 });
        assert!(id.is_some());
        assert_eq!(queue.len(), 1);

        let entry = queue.dequeue();
        assert!(entry.is_some());
        assert!(queue.is_empty());
    }

    #[test]
    fn test_queue_priority() {
        let mut queue = OfflineQueue::new();

        queue.enqueue_entry(
            QueueEntry::new(QueuedOperation::SendAnalytics {
                event_type: "low".to_string(),
                data: "{}".to_string(),
            })
            .with_priority(1),
        );

        queue.enqueue_entry(
            QueueEntry::new(QueuedOperation::AiRequest {
                prompt: "high".to_string(),
                context: None,
            })
            .with_priority(9),
        );

        // Should dequeue high priority first
        let entry = queue.dequeue().unwrap();
        assert_eq!(entry.priority, 9);
    }

    #[test]
    fn test_queue_requeue() {
        let mut queue = OfflineQueue::new();

        let entry = QueueEntry::new(QueuedOperation::Webhook {
            url: "http://example.com".to_string(),
            payload: "{}".to_string(),
        })
        .with_max_retries(2);

        queue.enqueue_entry(entry);

        let mut entry = queue.dequeue().unwrap();
        assert_eq!(entry.retry_count, 0);

        // Requeue after failure
        assert!(queue.requeue(entry.clone()));
        entry.retry_count += 1;

        let entry = queue.dequeue().unwrap();
        assert_eq!(entry.retry_count, 1);
    }

    #[test]
    fn test_queue_pause_resume() {
        let mut queue = OfflineQueue::new();
        queue.enqueue(QueuedOperation::SyncHistory { entries_count: 1 });

        queue.pause();
        assert!(queue.dequeue().is_none());

        queue.resume();
        assert!(queue.dequeue().is_some());
    }

    #[test]
    fn test_offline_manager() {
        let mut manager = OfflineManager::new();
        assert!(!manager.is_offline());

        manager.set_offline(true);
        assert!(manager.is_offline());

        let id = manager.queue_operation(QueuedOperation::AiRequest {
            prompt: "test".to_string(),
            context: None,
        });
        assert!(id.is_some());
    }

    #[test]
    fn test_queue_summary() {
        let mut queue = OfflineQueue::new();
        queue.enqueue(QueuedOperation::AiRequest { prompt: "a".to_string(), context: None });
        queue.enqueue(QueuedOperation::AiRequest { prompt: "b".to_string(), context: None });
        queue.enqueue(QueuedOperation::SyncHistory { entries_count: 5 });

        let summary = queue.summary();
        assert_eq!(summary.total, 3);
        assert_eq!(summary.ai_requests, 2);
        assert_eq!(summary.sync_ops, 1);
    }
}
