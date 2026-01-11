//! Core types and functionality for Palrun.
//!
//! This module contains the fundamental data structures used throughout
//! the application: commands, the registry, configuration, and execution.

mod analytics;
mod background;
mod capture;
mod chain;
mod command;
mod config;
mod context;
mod degradation;
mod executor;
mod filter;
mod history;
mod network;
mod parallel;
mod registry;
mod retry;

pub use analytics::{
    Analytics, AnalyticsReport, CommandStats, Insight, InsightCategory, TimePeriod,
};
pub use background::{
    send_notification, BackgroundEvent, BackgroundId, BackgroundManager, BackgroundProcess,
    BackgroundStatus,
};
pub use capture::{
    strip_ansi_codes, CaptureId, CaptureManager, CaptureMetadata, CapturedOutput, SearchResult,
};
pub use chain::{
    ChainExecutor, ChainOperator, ChainResult, ChainStep, ChainStepResult, ChainStepStatus,
    CommandChain,
};
pub use command::{Command, CommandSource};
pub use config::Config;
#[cfg(feature = "git")]
pub use config::HooksConfig;
pub use context::{CommandContext, ContextFilter, LocationIndicator};
pub use executor::{ExecutionResult, Executor};
pub use filter::{
    filter_by_source, filter_by_tag, filter_by_workspace, get_source_types, get_tags,
    get_workspaces, ParsedQuery,
};
pub use history::{CommandHistory, HistoryEntry, HistoryManager};
pub use network::{NetworkChecker, NetworkStatus, ServiceChecker};
pub use parallel::{
    ParallelExecutor, ParallelProcess, ParallelResult, ProcessEvent, ProcessId, ProcessOutput,
    ProcessStatus,
};
pub use registry::CommandRegistry;
pub use retry::{retry, CircuitBreaker, CircuitState, RetryConfig, RetryResult};
pub use degradation::{
    DegradationManager, DegradationReason, DegradedFeature, FallbackResult, Feature,
    with_fallback,
};
