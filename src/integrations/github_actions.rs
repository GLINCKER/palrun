//! GitHub Actions integration.
//!
//! Provides functionality to interact with GitHub Actions workflows,
//! including listing workflows, triggering runs, and viewing status.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// GitHub Actions API client.
pub struct GitHubActions {
    /// GitHub API base URL
    base_url: String,

    /// Repository owner
    owner: String,

    /// Repository name
    repo: String,

    /// Personal access token
    token: String,

    /// HTTP client
    client: reqwest::blocking::Client,
}

/// A GitHub Actions workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Workflow ID
    pub id: u64,

    /// Workflow name
    pub name: String,

    /// Workflow path (e.g., ".github/workflows/ci.yml")
    pub path: String,

    /// Workflow state
    pub state: String,

    /// URL to the workflow
    pub html_url: String,

    /// When the workflow was created
    pub created_at: String,

    /// When the workflow was last updated
    pub updated_at: String,
}

/// Status of a workflow run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowStatus {
    /// Workflow is queued
    Queued,
    /// Workflow is in progress
    InProgress,
    /// Workflow completed successfully
    Completed,
    /// Workflow was cancelled
    Cancelled,
    /// Workflow failed
    Failure,
    /// Workflow succeeded
    Success,
    /// Workflow was skipped
    Skipped,
    /// Workflow timed out
    TimedOut,
    /// Waiting for approval
    Waiting,
    /// Action required
    ActionRequired,
    /// Neutral (neither success nor failure)
    Neutral,
    /// Stale
    Stale,
}

impl WorkflowStatus {
    /// Get a display icon for the status.
    #[must_use]
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::Queued | Self::Waiting => "⏳",
            Self::InProgress => "🔄",
            Self::Completed | Self::Success => "✓",
            Self::Failure | Self::TimedOut => "✗",
            Self::Cancelled | Self::Skipped => "⊘",
            Self::ActionRequired => "⚠",
            Self::Neutral | Self::Stale => "○",
        }
    }

    /// Check if the status represents a running state.
    #[must_use]
    pub const fn is_running(&self) -> bool {
        matches!(self, Self::Queued | Self::InProgress | Self::Waiting)
    }

    /// Check if the status represents a success state.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Completed | Self::Success)
    }

    /// Check if the status represents a failure state.
    #[must_use]
    pub const fn is_failure(&self) -> bool {
        matches!(self, Self::Failure | Self::TimedOut)
    }
}

impl std::fmt::Display for WorkflowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Queued => "queued",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
            Self::Failure => "failure",
            Self::Success => "success",
            Self::Skipped => "skipped",
            Self::TimedOut => "timed_out",
            Self::Waiting => "waiting",
            Self::ActionRequired => "action_required",
            Self::Neutral => "neutral",
            Self::Stale => "stale",
        };
        write!(f, "{s}")
    }
}

/// A GitHub Actions workflow run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRun {
    /// Run ID
    pub id: u64,

    /// Workflow ID
    pub workflow_id: u64,

    /// Run name
    pub name: Option<String>,

    /// Run number
    pub run_number: u64,

    /// Run attempt
    pub run_attempt: u64,

    /// Status of the run
    pub status: WorkflowStatus,

    /// Conclusion of the run (if completed)
    pub conclusion: Option<WorkflowStatus>,

    /// Branch that triggered the run
    pub head_branch: String,

    /// Commit SHA
    pub head_sha: String,

    /// URL to the run
    pub html_url: String,

    /// When the run was created
    pub created_at: String,

    /// When the run was last updated
    pub updated_at: String,

    /// Who triggered the run
    pub triggering_actor: Option<Actor>,
}

/// A GitHub user/actor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    /// Login name
    pub login: String,

    /// Avatar URL
    pub avatar_url: Option<String>,
}

/// Response from listing workflows.
#[derive(Debug, Deserialize)]
struct WorkflowsResponse {
    #[allow(dead_code)]
    total_count: u64,
    workflows: Vec<Workflow>,
}

/// Response from listing workflow runs.
#[derive(Debug, Deserialize)]
struct WorkflowRunsResponse {
    #[allow(dead_code)]
    total_count: u64,
    workflow_runs: Vec<WorkflowRun>,
}

/// Error type for GitHub Actions operations.
#[derive(Debug, thiserror::Error)]
pub enum GitHubError {
    /// HTTP request failed
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// API returned an error
    #[error("GitHub API error: {message} (status: {status})")]
    Api { status: u16, message: String },

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Rate limited
    #[error("Rate limited. Reset at: {reset_at}")]
    RateLimited { reset_at: String },

    /// Invalid response
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

/// Result type for GitHub operations.
pub type GitHubResult<T> = Result<T, GitHubError>;

impl GitHubActions {
    /// Create a new GitHub Actions client.
    ///
    /// # Arguments
    /// * `owner` - Repository owner (user or organization)
    /// * `repo` - Repository name
    /// * `token` - GitHub personal access token
    pub fn new(
        owner: impl Into<String>,
        repo: impl Into<String>,
        token: impl Into<String>,
    ) -> GitHubResult<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(format!("palrun/{}", env!("CARGO_PKG_VERSION")))
            .build()?;

        Ok(Self {
            base_url: "https://api.github.com".to_string(),
            owner: owner.into(),
            repo: repo.into(),
            token: token.into(),
            client,
        })
    }

    /// Create a client from environment variables.
    ///
    /// Uses `GITHUB_TOKEN` for authentication and `GITHUB_REPOSITORY` for owner/repo.
    pub fn from_env() -> GitHubResult<Option<Self>> {
        let token = match std::env::var("GITHUB_TOKEN") {
            Ok(t) => t,
            Err(_) => return Ok(None),
        };

        // Try to get repo from GITHUB_REPOSITORY (format: owner/repo)
        let (owner, repo) = if let Ok(repo_str) = std::env::var("GITHUB_REPOSITORY") {
            if let Some((o, r)) = repo_str.split_once('/') {
                (o.to_string(), r.to_string())
            } else {
                return Ok(None);
            }
        } else {
            // Try to detect from git remote
            if let Some((o, r)) = detect_github_repo()? {
                (o, r)
            } else {
                return Ok(None);
            }
        };

        Ok(Some(Self::new(owner, repo, token)?))
    }

    /// Get the repository URL.
    fn repo_url(&self) -> String {
        format!("{}/repos/{}/{}", self.base_url, self.owner, self.repo)
    }

    /// Make an authenticated GET request.
    fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> GitHubResult<T> {
        let url = format!("{}{}", self.repo_url(), path);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()?;

        self.handle_response(response)
    }

    /// Make an authenticated POST request.
    #[allow(dead_code)]
    fn post<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> GitHubResult<T> {
        let url = format!("{}{}", self.repo_url(), path);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(body)
            .send()?;

        self.handle_response(response)
    }

    /// Handle API response.
    fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::blocking::Response,
    ) -> GitHubResult<T> {
        let status = response.status();

        if status.is_success() {
            response.json().map_err(|e| GitHubError::InvalidResponse(e.to_string()))
        } else {
            // Try to parse error message
            let message = response.text().unwrap_or_else(|_| "Unknown error".to_string());

            match status.as_u16() {
                401 => Err(GitHubError::Auth(message)),
                403 => {
                    // Check if rate limited
                    Err(GitHubError::RateLimited { reset_at: "unknown".to_string() })
                }
                404 => Err(GitHubError::NotFound(message)),
                _ => Err(GitHubError::Api { status: status.as_u16(), message }),
            }
        }
    }

    /// List all workflows in the repository.
    pub fn list_workflows(&self) -> GitHubResult<Vec<Workflow>> {
        let response: WorkflowsResponse = self.get("/actions/workflows")?;
        Ok(response.workflows)
    }

    /// Get a specific workflow by ID or filename.
    pub fn get_workflow(&self, workflow_id: &str) -> GitHubResult<Workflow> {
        self.get(&format!("/actions/workflows/{workflow_id}"))
    }

    /// List workflow runs, optionally filtered.
    pub fn list_runs(
        &self,
        workflow_id: Option<u64>,
        branch: Option<&str>,
        limit: usize,
    ) -> GitHubResult<Vec<WorkflowRun>> {
        let mut path = if let Some(wf_id) = workflow_id {
            format!("/actions/workflows/{wf_id}/runs")
        } else {
            "/actions/runs".to_string()
        };

        // Add query parameters
        let mut params = vec![format!("per_page={limit}")];
        if let Some(b) = branch {
            params.push(format!("branch={b}"));
        }

        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        let response: WorkflowRunsResponse = self.get(&path)?;
        Ok(response.workflow_runs)
    }

    /// Get the latest run for a workflow.
    pub fn get_latest_run(&self, workflow_id: u64) -> GitHubResult<Option<WorkflowRun>> {
        let runs = self.list_runs(Some(workflow_id), None, 1)?;
        Ok(runs.into_iter().next())
    }

    /// Get a specific workflow run.
    pub fn get_run(&self, run_id: u64) -> GitHubResult<WorkflowRun> {
        self.get(&format!("/actions/runs/{run_id}"))
    }

    /// Trigger a workflow dispatch event.
    pub fn trigger_workflow(
        &self,
        workflow_id: &str,
        branch: &str,
        inputs: Option<serde_json::Value>,
    ) -> GitHubResult<()> {
        #[derive(Serialize)]
        struct DispatchRequest {
            #[serde(rename = "ref")]
            branch: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            inputs: Option<serde_json::Value>,
        }

        let body = DispatchRequest { branch: branch.to_string(), inputs };

        let url = format!("{}/actions/workflows/{workflow_id}/dispatches", self.repo_url());

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&body)
            .send()?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let message = response.text().unwrap_or_default();
            Err(GitHubError::Api { status, message })
        }
    }

    /// Re-run a failed workflow.
    pub fn rerun_workflow(&self, run_id: u64) -> GitHubResult<()> {
        let url = format!("{}/actions/runs/{run_id}/rerun", self.repo_url());

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let message = response.text().unwrap_or_default();
            Err(GitHubError::Api { status, message })
        }
    }

    /// Cancel a workflow run.
    pub fn cancel_run(&self, run_id: u64) -> GitHubResult<()> {
        let url = format!("{}/actions/runs/{run_id}/cancel", self.repo_url());

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let message = response.text().unwrap_or_default();
            Err(GitHubError::Api { status, message })
        }
    }

    /// Get the overall CI status for a branch.
    pub fn get_branch_status(&self, branch: &str) -> GitHubResult<Option<WorkflowStatus>> {
        let runs = self.list_runs(None, Some(branch), 10)?;

        if runs.is_empty() {
            return Ok(None);
        }

        // Get the status of the most recent run for each workflow
        let mut any_running = false;
        let mut any_failed = false;
        let mut all_success = true;

        for run in &runs {
            let status = run.conclusion.unwrap_or(run.status);

            if status.is_running() {
                any_running = true;
                all_success = false;
            } else if status.is_failure() {
                any_failed = true;
                all_success = false;
            } else if !status.is_success() {
                all_success = false;
            }
        }

        let overall = if any_running {
            WorkflowStatus::InProgress
        } else if any_failed {
            WorkflowStatus::Failure
        } else if all_success {
            WorkflowStatus::Success
        } else {
            WorkflowStatus::Neutral
        };

        Ok(Some(overall))
    }

    /// Get repository owner.
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Get repository name.
    pub fn repo(&self) -> &str {
        &self.repo
    }
}

/// Detect GitHub repository from git remote.
fn detect_github_repo() -> GitHubResult<Option<(String, String)>> {
    // Try to read .git/config or run git remote
    let output = std::process::Command::new("git").args(["remote", "get-url", "origin"]).output();

    let Ok(output) = output else {
        return Ok(None);
    };

    if !output.status.success() {
        return Ok(None);
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Parse GitHub URL formats:
    // - https://github.com/owner/repo.git
    // - git@github.com:owner/repo.git
    // - https://github.com/owner/repo
    // - git@github.com:owner/repo

    if url.contains("github.com") {
        let repo_part = if url.starts_with("git@") {
            // SSH format: git@github.com:owner/repo.git
            url.strip_prefix("git@github.com:").map(|s| s.strip_suffix(".git").unwrap_or(s))
        } else {
            // HTTPS format: https://github.com/owner/repo.git
            url.strip_prefix("https://github.com/")
                .or_else(|| url.strip_prefix("http://github.com/"))
                .map(|s| s.strip_suffix(".git").unwrap_or(s))
        };

        if let Some(repo_part) = repo_part {
            if let Some((owner, repo)) = repo_part.split_once('/') {
                return Ok(Some((owner.to_string(), repo.to_string())));
            }
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_status_icon() {
        assert_eq!(WorkflowStatus::Success.icon(), "✓");
        assert_eq!(WorkflowStatus::Failure.icon(), "✗");
        assert_eq!(WorkflowStatus::InProgress.icon(), "🔄");
        assert_eq!(WorkflowStatus::Queued.icon(), "⏳");
    }

    #[test]
    fn test_workflow_status_states() {
        assert!(WorkflowStatus::InProgress.is_running());
        assert!(WorkflowStatus::Queued.is_running());
        assert!(!WorkflowStatus::Success.is_running());

        assert!(WorkflowStatus::Success.is_success());
        assert!(WorkflowStatus::Completed.is_success());
        assert!(!WorkflowStatus::Failure.is_success());

        assert!(WorkflowStatus::Failure.is_failure());
        assert!(WorkflowStatus::TimedOut.is_failure());
        assert!(!WorkflowStatus::Success.is_failure());
    }

    #[test]
    fn test_parse_github_url() {
        // This would test detect_github_repo but it requires git to be present
    }
}
