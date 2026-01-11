#![allow(dead_code)]
#![allow(clippy::vec_init_then_push)]
#![allow(clippy::needless_collect)]
#![allow(clippy::format_push_string)]
#![allow(clippy::unused_self)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::unnecessary_filter_map)]
#![allow(clippy::unnecessary_lazy_evaluations)]
#![allow(clippy::match_wildcard_for_single_variants)]
#![allow(clippy::manual_strip)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::single_char_pattern)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::or_fun_call)]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
#![allow(clippy::should_implement_trait)]

//! # Palrun
//!
//! AI command palette for your terminal - discover and run project commands instantly.
//!
//! Palrun automatically detects your project's available commands (npm scripts, Nx targets,
//! Makefile targets, etc.) and presents them in a fuzzy-searchable command palette.
//!
//! ## Features
//!
//! - **Fuzzy Search**: Instantly find commands with fuzzy matching (powered by nucleo)
//! - **Project Awareness**: Auto-detects npm, Nx, Turborepo, Makefile, and more
//! - **AI Integration**: Natural language to shell commands (optional)
//! - **Runbooks**: Executable team workflows in YAML
//! - **Cross-Platform**: Works on Linux, macOS, and Windows
//!
//! ## Quick Start
//!
//! ```bash
//! # Install
//! cargo install palrun
//!
//! # Open command palette
//! pal
//!
//! # Or use the full name
//! palrun
//! ```

#![forbid(unsafe_code)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
// Allow common patterns that are intentional in this codebase
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::redundant_else)]
#![allow(clippy::if_not_else)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::significant_drop_tightening)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::unnecessary_literal_bound)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::unnecessary_map_or)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::unnecessary_debug_formatting)]
#![allow(clippy::for_kv_map)]
#![allow(clippy::use_self)]
#![allow(clippy::ptr_arg)]

pub mod app;
pub mod core;
pub mod init;
pub mod scanner;
pub mod tui;

#[cfg(feature = "ai")]
pub mod ai;

#[cfg(feature = "ai")]
pub use ai::{
    AIManager, AIProvider, Agent, AgentMessage, AgentProvider, AgentResponse, AgentState,
    AgentStopReason, AgentTool, AgentToolCall, AgentToolResult, CompositeExecutor, MCPToolExecutor,
    OllamaProvider, ProjectContext, ShellExecutor, ToolExecutor,
};

pub mod runbook;

#[cfg(feature = "git")]
pub mod git;

#[cfg(feature = "git")]
pub use git::{GitInfo, GitRepository};

pub mod env;
pub use env::{
    EnvManager, ProviderStatus, ResolvedSecret, RuntimeType, RuntimeVersion, SecretProvider,
    SecretReference, SecretsManager, VersionManager,
};

#[cfg(feature = "plugins")]
pub mod plugin;

#[cfg(feature = "plugins")]
pub use plugin::{
    PluginCommand, PluginError, PluginHost, PluginInfo, PluginManager, PluginManifest,
    PluginPermissions, PluginResult, PluginRuntime, PluginType,
};

pub mod integrations;
pub use integrations::{
    CreateIssueOptions, CreateLinearIssueOptions, GitHubActions, GitHubIssues, Issue, IssueComment,
    IssueStats, IssuesError, Label, LinearClient, LinearError, LinearIssue, LinearLabel,
    LinearState, LinearStats, LinearTeam, LinearUser, ListIssuesOptions, ListLinearIssuesOptions,
    Milestone, NotificationClient, NotificationConfig, NotificationEvent, NotificationMessage,
    NotificationType, UpdateIssueOptions, User, Workflow, WorkflowRun, WorkflowStatus,
};

pub mod mcp;
pub use mcp::{
    MCPClient, MCPManager, MCPServer, MCPServerConfig, MCPTool, ToolCall, ToolRegistry, ToolResult,
};

pub mod security;
pub use security::{
    CommandValidator, SecurityConfig, SecurityManager, ValidationError, ValidationResult,
    ValidationSeverity,
};

// Re-export commonly used types
pub use app::App;
pub use core::{Command, CommandRegistry, CommandSource, Config};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = "palrun";

/// Short alias
pub const APP_ALIAS: &str = "pal";
