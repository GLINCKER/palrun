//! GitHub Issues integration.
//!
//! Provides functionality to interact with GitHub Issues API for
//! viewing, creating, and managing issues from Palrun.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// GitHub Issues API client.
#[derive(Debug, Clone)]
pub struct GitHubIssues {
    /// GitHub API token
    token: String,
    /// Repository owner
    owner: String,
    /// Repository name
    repo: String,
    /// HTTP client
    client: reqwest::Client,
}

/// A GitHub issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Issue number
    pub number: u64,
    /// Issue title
    pub title: String,
    /// Issue body/description
    pub body: Option<String>,
    /// Issue state (open, closed)
    pub state: String,
    /// Issue labels
    pub labels: Vec<Label>,
    /// Issue assignees
    pub assignees: Vec<User>,
    /// Issue author
    pub user: User,
    /// HTML URL to the issue
    pub html_url: String,
    /// Created timestamp
    pub created_at: String,
    /// Updated timestamp
    pub updated_at: String,
    /// Milestone if assigned
    pub milestone: Option<Milestone>,
}

/// A GitHub label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    /// Label name
    pub name: String,
    /// Label color (hex without #)
    pub color: String,
    /// Label description
    pub description: Option<String>,
}

/// A GitHub user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Username/login
    pub login: String,
    /// Avatar URL
    pub avatar_url: String,
    /// User type (User, Bot, etc.)
    #[serde(rename = "type")]
    pub user_type: String,
}

/// A GitHub milestone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    /// Milestone number
    pub number: u64,
    /// Milestone title
    pub title: String,
    /// Milestone state
    pub state: String,
    /// Due date if set
    pub due_on: Option<String>,
}

/// Options for creating a new issue.
#[derive(Debug, Clone, Default)]
pub struct CreateIssueOptions {
    /// Issue title (required)
    pub title: String,
    /// Issue body/description
    pub body: Option<String>,
    /// Labels to add
    pub labels: Vec<String>,
    /// Assignees to add
    pub assignees: Vec<String>,
    /// Milestone number to assign
    pub milestone: Option<u64>,
}

/// Options for listing issues.
#[derive(Debug, Clone, Default)]
pub struct ListIssuesOptions {
    /// Filter by state: open, closed, all
    pub state: Option<String>,
    /// Filter by labels (comma-separated)
    pub labels: Option<String>,
    /// Filter by assignee
    pub assignee: Option<String>,
    /// Filter by creator
    pub creator: Option<String>,
    /// Filter by milestone number or "none"/"*"
    pub milestone: Option<String>,
    /// Sort by: created, updated, comments
    pub sort: Option<String>,
    /// Sort direction: asc, desc
    pub direction: Option<String>,
    /// Maximum number of results
    pub per_page: Option<u32>,
}

/// Options for updating an issue.
#[derive(Debug, Clone, Default)]
pub struct UpdateIssueOptions {
    /// New title
    pub title: Option<String>,
    /// New body
    pub body: Option<String>,
    /// New state (open, closed)
    pub state: Option<String>,
    /// Labels to set (replaces existing)
    pub labels: Option<Vec<String>>,
    /// Assignees to set (replaces existing)
    pub assignees: Option<Vec<String>>,
    /// Milestone number to assign
    pub milestone: Option<u64>,
}

/// A comment on an issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueComment {
    /// Comment ID
    pub id: u64,
    /// Comment body
    pub body: String,
    /// Comment author
    pub user: User,
    /// Created timestamp
    pub created_at: String,
    /// Updated timestamp
    pub updated_at: String,
}

/// Issue statistics.
#[derive(Debug, Clone, Default)]
pub struct IssueStats {
    /// Total open issues
    pub open_count: u64,
    /// Total closed issues
    pub closed_count: u64,
    /// Issues assigned to current user
    pub assigned_to_me: u64,
    /// Issues created by current user
    pub created_by_me: u64,
}

/// Result type for GitHub Issues operations.
pub type IssuesResult<T> = Result<T, IssuesError>;

/// Error types for GitHub Issues operations.
#[derive(Debug, thiserror::Error)]
pub enum IssuesError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("GitHub API error: {message} (status: {status})")]
    Api { status: u16, message: String },

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Authentication required")]
    Unauthorized,

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl GitHubIssues {
    /// Create a new GitHub Issues client.
    pub fn new(
        token: impl Into<String>,
        owner: impl Into<String>,
        repo: impl Into<String>,
    ) -> Self {
        Self {
            token: token.into(),
            owner: owner.into(),
            repo: repo.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Create from environment variables and git remote.
    ///
    /// Looks for GITHUB_TOKEN or GH_TOKEN environment variable.
    pub fn from_env(owner: impl Into<String>, repo: impl Into<String>) -> Option<Self> {
        let token = std::env::var("GITHUB_TOKEN").or_else(|_| std::env::var("GH_TOKEN")).ok()?;
        Some(Self::new(token, owner, repo))
    }

    /// Get the API base URL for this repository.
    fn api_url(&self, path: &str) -> String {
        format!("https://api.github.com/repos/{}/{}/{}", self.owner, self.repo, path)
    }

    /// Make an authenticated request.
    fn request(&self, method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
        self.client
            .request(method, url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "palrun")
            .header("X-GitHub-Api-Version", "2022-11-28")
    }

    /// Parse error response from GitHub API.
    async fn parse_error(&self, response: reqwest::Response) -> IssuesError {
        let status = response.status().as_u16();

        match status {
            401 => IssuesError::Unauthorized,
            403 => {
                // Check if rate limited
                if response
                    .headers()
                    .get("x-ratelimit-remaining")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s == "0")
                    .unwrap_or(false)
                {
                    return IssuesError::RateLimited;
                }
                IssuesError::Api { status, message: "Forbidden".to_string() }
            }
            404 => IssuesError::NotFound("Resource not found".to_string()),
            _ => {
                let message = response
                    .json::<serde_json::Value>()
                    .await
                    .ok()
                    .and_then(|v| v.get("message").and_then(|m| m.as_str()).map(String::from))
                    .unwrap_or_else(|| format!("HTTP {}", status));
                IssuesError::Api { status, message }
            }
        }
    }

    /// List issues in the repository.
    pub async fn list_issues(&self, options: ListIssuesOptions) -> IssuesResult<Vec<Issue>> {
        let mut url = self.api_url("issues");
        let mut params = Vec::new();

        if let Some(state) = &options.state {
            params.push(format!("state={}", state));
        }
        if let Some(labels) = &options.labels {
            params.push(format!("labels={}", labels));
        }
        if let Some(assignee) = &options.assignee {
            params.push(format!("assignee={}", assignee));
        }
        if let Some(creator) = &options.creator {
            params.push(format!("creator={}", creator));
        }
        if let Some(milestone) = &options.milestone {
            params.push(format!("milestone={}", milestone));
        }
        if let Some(sort) = &options.sort {
            params.push(format!("sort={}", sort));
        }
        if let Some(direction) = &options.direction {
            params.push(format!("direction={}", direction));
        }
        if let Some(per_page) = options.per_page {
            params.push(format!("per_page={}", per_page));
        }

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        let response = self.request(reqwest::Method::GET, &url).send().await?;

        if !response.status().is_success() {
            return Err(self.parse_error(response).await);
        }

        let issues: Vec<Issue> = response.json().await?;
        Ok(issues)
    }

    /// Get a specific issue by number.
    pub async fn get_issue(&self, issue_number: u64) -> IssuesResult<Issue> {
        let url = self.api_url(&format!("issues/{}", issue_number));

        let response = self.request(reqwest::Method::GET, &url).send().await?;

        if !response.status().is_success() {
            return Err(self.parse_error(response).await);
        }

        let issue: Issue = response.json().await?;
        Ok(issue)
    }

    /// Create a new issue.
    pub async fn create_issue(&self, options: CreateIssueOptions) -> IssuesResult<Issue> {
        if options.title.is_empty() {
            return Err(IssuesError::InvalidInput("Title is required".to_string()));
        }

        let url = self.api_url("issues");

        let mut body: HashMap<&str, serde_json::Value> = HashMap::new();
        body.insert("title", serde_json::json!(options.title));

        if let Some(desc) = &options.body {
            body.insert("body", serde_json::json!(desc));
        }
        if !options.labels.is_empty() {
            body.insert("labels", serde_json::json!(options.labels));
        }
        if !options.assignees.is_empty() {
            body.insert("assignees", serde_json::json!(options.assignees));
        }
        if let Some(milestone) = options.milestone {
            body.insert("milestone", serde_json::json!(milestone));
        }

        let response = self.request(reqwest::Method::POST, &url).json(&body).send().await?;

        if !response.status().is_success() {
            return Err(self.parse_error(response).await);
        }

        let issue: Issue = response.json().await?;
        Ok(issue)
    }

    /// Update an existing issue.
    pub async fn update_issue(
        &self,
        issue_number: u64,
        options: UpdateIssueOptions,
    ) -> IssuesResult<Issue> {
        let url = self.api_url(&format!("issues/{}", issue_number));

        let mut body: HashMap<&str, serde_json::Value> = HashMap::new();

        if let Some(title) = &options.title {
            body.insert("title", serde_json::json!(title));
        }
        if let Some(desc) = &options.body {
            body.insert("body", serde_json::json!(desc));
        }
        if let Some(state) = &options.state {
            body.insert("state", serde_json::json!(state));
        }
        if let Some(labels) = &options.labels {
            body.insert("labels", serde_json::json!(labels));
        }
        if let Some(assignees) = &options.assignees {
            body.insert("assignees", serde_json::json!(assignees));
        }
        if let Some(milestone) = options.milestone {
            body.insert("milestone", serde_json::json!(milestone));
        }

        let response = self.request(reqwest::Method::PATCH, &url).json(&body).send().await?;

        if !response.status().is_success() {
            return Err(self.parse_error(response).await);
        }

        let issue: Issue = response.json().await?;
        Ok(issue)
    }

    /// Close an issue.
    pub async fn close_issue(&self, issue_number: u64) -> IssuesResult<Issue> {
        self.update_issue(
            issue_number,
            UpdateIssueOptions { state: Some("closed".to_string()), ..Default::default() },
        )
        .await
    }

    /// Reopen an issue.
    pub async fn reopen_issue(&self, issue_number: u64) -> IssuesResult<Issue> {
        self.update_issue(
            issue_number,
            UpdateIssueOptions { state: Some("open".to_string()), ..Default::default() },
        )
        .await
    }

    /// Add labels to an issue.
    pub async fn add_labels(
        &self,
        issue_number: u64,
        labels: Vec<String>,
    ) -> IssuesResult<Vec<Label>> {
        let url = self.api_url(&format!("issues/{}/labels", issue_number));

        let body = serde_json::json!({ "labels": labels });

        let response = self.request(reqwest::Method::POST, &url).json(&body).send().await?;

        if !response.status().is_success() {
            return Err(self.parse_error(response).await);
        }

        let labels: Vec<Label> = response.json().await?;
        Ok(labels)
    }

    /// Remove a label from an issue.
    pub async fn remove_label(&self, issue_number: u64, label: &str) -> IssuesResult<()> {
        let url =
            self.api_url(&format!("issues/{}/labels/{}", issue_number, urlencoding::encode(label)));

        let response = self.request(reqwest::Method::DELETE, &url).send().await?;

        if !response.status().is_success() && response.status().as_u16() != 404 {
            return Err(self.parse_error(response).await);
        }

        Ok(())
    }

    /// List comments on an issue.
    pub async fn list_comments(&self, issue_number: u64) -> IssuesResult<Vec<IssueComment>> {
        let url = self.api_url(&format!("issues/{}/comments", issue_number));

        let response = self.request(reqwest::Method::GET, &url).send().await?;

        if !response.status().is_success() {
            return Err(self.parse_error(response).await);
        }

        let comments: Vec<IssueComment> = response.json().await?;
        Ok(comments)
    }

    /// Add a comment to an issue.
    pub async fn add_comment(&self, issue_number: u64, body: &str) -> IssuesResult<IssueComment> {
        let url = self.api_url(&format!("issues/{}/comments", issue_number));

        let payload = serde_json::json!({ "body": body });

        let response = self.request(reqwest::Method::POST, &url).json(&payload).send().await?;

        if !response.status().is_success() {
            return Err(self.parse_error(response).await);
        }

        let comment: IssueComment = response.json().await?;
        Ok(comment)
    }

    /// Get issue statistics for the repository.
    pub async fn get_stats(&self, username: Option<&str>) -> IssuesResult<IssueStats> {
        let mut stats = IssueStats::default();

        // Get open issues count
        let open_issues = self
            .list_issues(ListIssuesOptions {
                state: Some("open".to_string()),
                per_page: Some(1),
                ..Default::default()
            })
            .await?;
        stats.open_count = open_issues.len() as u64;

        // Get closed issues count
        let closed_issues = self
            .list_issues(ListIssuesOptions {
                state: Some("closed".to_string()),
                per_page: Some(1),
                ..Default::default()
            })
            .await?;
        stats.closed_count = closed_issues.len() as u64;

        // If username provided, get user-specific stats
        if let Some(user) = username {
            let assigned = self
                .list_issues(ListIssuesOptions {
                    state: Some("open".to_string()),
                    assignee: Some(user.to_string()),
                    per_page: Some(100),
                    ..Default::default()
                })
                .await?;
            stats.assigned_to_me = assigned.len() as u64;

            let created = self
                .list_issues(ListIssuesOptions {
                    state: Some("open".to_string()),
                    creator: Some(user.to_string()),
                    per_page: Some(100),
                    ..Default::default()
                })
                .await?;
            stats.created_by_me = created.len() as u64;
        }

        Ok(stats)
    }

    /// Search for issues.
    pub async fn search_issues(&self, query: &str) -> IssuesResult<Vec<Issue>> {
        let search_query = format!("repo:{}/{} {}", self.owner, self.repo, query);
        let url = format!(
            "https://api.github.com/search/issues?q={}",
            urlencoding::encode(&search_query)
        );

        let response = self.request(reqwest::Method::GET, &url).send().await?;

        if !response.status().is_success() {
            return Err(self.parse_error(response).await);
        }

        #[derive(Deserialize)]
        struct SearchResult {
            items: Vec<Issue>,
        }

        let result: SearchResult = response.json().await?;
        Ok(result.items)
    }

    /// List all labels in the repository.
    pub async fn list_labels(&self) -> IssuesResult<Vec<Label>> {
        let url = self.api_url("labels");

        let response = self.request(reqwest::Method::GET, &url).send().await?;

        if !response.status().is_success() {
            return Err(self.parse_error(response).await);
        }

        let labels: Vec<Label> = response.json().await?;
        Ok(labels)
    }

    /// List all milestones in the repository.
    pub async fn list_milestones(&self) -> IssuesResult<Vec<Milestone>> {
        let url = self.api_url("milestones");

        let response = self.request(reqwest::Method::GET, &url).send().await?;

        if !response.status().is_success() {
            return Err(self.parse_error(response).await);
        }

        let milestones: Vec<Milestone> = response.json().await?;
        Ok(milestones)
    }
}

/// Format an issue for display.
pub fn format_issue(issue: &Issue, verbose: bool) -> String {
    let state_icon = if issue.state == "open" { "○" } else { "●" };
    let state_color = if issue.state == "open" { "\x1b[32m" } else { "\x1b[35m" };
    let reset = "\x1b[0m";

    let labels = if issue.labels.is_empty() {
        String::new()
    } else {
        let label_names: Vec<&str> = issue.labels.iter().map(|l| l.name.as_str()).collect();
        format!(" [{}]", label_names.join(", "))
    };

    let assignees = if issue.assignees.is_empty() {
        String::new()
    } else {
        let names: Vec<&str> = issue.assignees.iter().map(|a| a.login.as_str()).collect();
        format!(" → {}", names.join(", "))
    };

    if verbose {
        let body = issue
            .body
            .as_ref()
            .map(|b| {
                let truncated = if b.len() > 200 { format!("{}...", &b[..200]) } else { b.clone() };
                format!("\n  {}", truncated.replace('\n', "\n  "))
            })
            .unwrap_or_default();

        format!(
            "{}{} #{}{} {}{}{}{}",
            state_color, state_icon, issue.number, reset, issue.title, labels, assignees, body
        )
    } else {
        format!(
            "{}{} #{}{} {}{}{}",
            state_color, state_icon, issue.number, reset, issue.title, labels, assignees
        )
    }
}

/// Format issue statistics for display.
pub fn format_stats(stats: &IssueStats) -> String {
    let mut lines = Vec::new();

    lines.push(format!("Open: {}  Closed: {}", stats.open_count, stats.closed_count));

    if stats.assigned_to_me > 0 || stats.created_by_me > 0 {
        lines.push(format!(
            "Assigned to me: {}  Created by me: {}",
            stats.assigned_to_me, stats.created_by_me
        ));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_issue() {
        let issue = Issue {
            number: 42,
            title: "Test issue".to_string(),
            body: Some("This is a test".to_string()),
            state: "open".to_string(),
            labels: vec![Label {
                name: "bug".to_string(),
                color: "d73a4a".to_string(),
                description: None,
            }],
            assignees: vec![],
            user: User {
                login: "testuser".to_string(),
                avatar_url: "https://example.com/avatar".to_string(),
                user_type: "User".to_string(),
            },
            html_url: "https://github.com/test/repo/issues/42".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            milestone: None,
        };

        let output = format_issue(&issue, false);
        assert!(output.contains("#42"));
        assert!(output.contains("Test issue"));
        assert!(output.contains("[bug]"));
    }

    #[test]
    fn test_create_issue_options_default() {
        let options = CreateIssueOptions::default();
        assert!(options.title.is_empty());
        assert!(options.body.is_none());
        assert!(options.labels.is_empty());
    }

    #[test]
    fn test_list_issues_options_default() {
        let options = ListIssuesOptions::default();
        assert!(options.state.is_none());
        assert!(options.labels.is_none());
        assert!(options.per_page.is_none());
    }
}
