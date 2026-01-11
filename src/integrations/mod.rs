//! External integrations module.
//!
//! Provides integration with external services like CI/CD platforms,
//! issue trackers, notification services, webhooks, and REST API.

pub mod api;
pub mod github_actions;
pub mod github_issues;
pub mod linear;
pub mod notifications;
pub mod webhooks;

pub use api::{
    ApiConfig, ApiError, ApiResult, ApiServer, ApiState, CommandInfo, ExecuteRequest,
    ExecuteResponse, HistoryEntry, RateLimiter, StatusResponse,
};
pub use github_actions::{GitHubActions, Workflow, WorkflowRun, WorkflowStatus};
pub use github_issues::{
    CreateIssueOptions, GitHubIssues, Issue, IssueComment, IssueStats, IssuesError, IssuesResult,
    Label, ListIssuesOptions, Milestone, UpdateIssueOptions, User,
};
pub use linear::{
    CreateLinearIssueOptions, LinearClient, LinearError, LinearIssue, LinearLabel, LinearResult,
    LinearState, LinearStats, LinearTeam, LinearUser, ListLinearIssuesOptions,
};
pub use notifications::{
    NotificationClient, NotificationConfig, NotificationError, NotificationEvent,
    NotificationMessage, NotificationResult, NotificationType,
};
pub use webhooks::{
    AgentEventData, CommandEventData, McpToolEventData, RunbookEventData, WebhookConfig,
    WebhookData, WebhookDelivery, WebhookError, WebhookEvent, WebhookManager, WebhookPayload,
    WebhookResult,
};
