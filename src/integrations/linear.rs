//! Linear issue tracker integration.
//!
//! Provides functionality to interact with Linear's GraphQL API for
//! viewing, creating, and managing issues from Palrun.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Linear API client.
#[derive(Debug, Clone)]
pub struct LinearClient {
    /// Linear API token
    token: String,
    /// HTTP client
    client: reqwest::Client,
}

/// A Linear issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearIssue {
    /// Issue ID
    pub id: String,
    /// Issue identifier (e.g., "ENG-123")
    pub identifier: String,
    /// Issue title
    pub title: String,
    /// Issue description (markdown)
    pub description: Option<String>,
    /// Issue state
    pub state: LinearState,
    /// Issue priority (0-4, 0 = no priority, 1 = urgent, 4 = low)
    pub priority: i32,
    /// Issue labels
    pub labels: Vec<LinearLabel>,
    /// Assignee
    pub assignee: Option<LinearUser>,
    /// Created timestamp
    pub created_at: String,
    /// Updated timestamp
    pub updated_at: String,
    /// URL to the issue
    pub url: String,
    /// Due date if set
    pub due_date: Option<String>,
    /// Estimate in points
    pub estimate: Option<f32>,
}

/// A Linear issue state (workflow state).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearState {
    /// State ID
    pub id: String,
    /// State name
    pub name: String,
    /// State color
    pub color: String,
    /// State type (backlog, unstarted, started, completed, canceled)
    #[serde(rename = "type")]
    pub state_type: String,
}

/// A Linear label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearLabel {
    /// Label ID
    pub id: String,
    /// Label name
    pub name: String,
    /// Label color
    pub color: String,
}

/// A Linear user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearUser {
    /// User ID
    pub id: String,
    /// User name
    pub name: String,
    /// User email
    pub email: String,
    /// User display name
    pub display_name: String,
}

/// A Linear team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearTeam {
    /// Team ID
    pub id: String,
    /// Team name
    pub name: String,
    /// Team key (used in identifiers)
    pub key: String,
}

/// Options for creating a Linear issue.
#[derive(Debug, Clone, Default)]
pub struct CreateLinearIssueOptions {
    /// Issue title (required)
    pub title: String,
    /// Issue description
    pub description: Option<String>,
    /// Team ID (required)
    pub team_id: String,
    /// Priority (0-4)
    pub priority: Option<i32>,
    /// Label IDs
    pub label_ids: Vec<String>,
    /// Assignee ID
    pub assignee_id: Option<String>,
    /// Due date (YYYY-MM-DD)
    pub due_date: Option<String>,
    /// Estimate in points
    pub estimate: Option<f32>,
}

/// Options for listing Linear issues.
#[derive(Debug, Clone, Default)]
pub struct ListLinearIssuesOptions {
    /// Filter by team ID
    pub team_id: Option<String>,
    /// Filter by assignee ID (use "me" for current user)
    pub assignee_id: Option<String>,
    /// Filter by state type (backlog, unstarted, started, completed, canceled)
    pub state_type: Option<String>,
    /// Include completed/canceled issues
    pub include_archived: bool,
    /// Maximum number of results
    pub limit: Option<u32>,
}

/// Linear issue statistics.
#[derive(Debug, Clone, Default)]
pub struct LinearStats {
    /// Total issues assigned to user
    pub assigned_count: u64,
    /// Issues in progress
    pub in_progress_count: u64,
    /// Issues completed this cycle
    pub completed_count: u64,
    /// Total teams
    pub team_count: u64,
}

/// Result type for Linear operations.
pub type LinearResult<T> = Result<T, LinearError>;

/// Error types for Linear operations.
#[derive(Debug, thiserror::Error)]
pub enum LinearError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("GraphQL error: {0}")]
    GraphQL(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Authentication required")]
    Unauthorized,

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl LinearClient {
    /// Create a new Linear client.
    pub fn new(token: impl Into<String>) -> Self {
        Self { token: token.into(), client: reqwest::Client::new() }
    }

    /// Create from environment variable LINEAR_API_KEY.
    pub fn from_env() -> Option<Self> {
        let token = std::env::var("LINEAR_API_KEY").ok()?;
        Some(Self::new(token))
    }

    /// Execute a GraphQL query.
    async fn query<T: for<'de> Deserialize<'de>>(
        &self,
        query: &str,
        variables: Option<serde_json::Value>,
    ) -> LinearResult<T> {
        let body = serde_json::json!({
            "query": query,
            "variables": variables.unwrap_or(serde_json::json!({}))
        });

        let response = self
            .client
            .post("https://api.linear.app/graphql")
            .header("Authorization", &self.token)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(LinearError::Unauthorized);
        }

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(LinearError::RateLimited);
        }

        let result: serde_json::Value = response.json().await?;

        // Check for GraphQL errors
        if let Some(errors) = result.get("errors") {
            if let Some(first_error) = errors.as_array().and_then(|e| e.first()) {
                let message =
                    first_error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error");
                return Err(LinearError::GraphQL(message.to_string()));
            }
        }

        // Extract data
        let data = result
            .get("data")
            .ok_or_else(|| LinearError::GraphQL("No data in response".to_string()))?;

        serde_json::from_value(data.clone())
            .map_err(|e| LinearError::GraphQL(format!("Failed to parse response: {}", e)))
    }

    /// Get the current user.
    pub async fn get_viewer(&self) -> LinearResult<LinearUser> {
        #[derive(Deserialize)]
        struct Response {
            viewer: LinearUser,
        }

        let query = r#"
            query {
                viewer {
                    id
                    name
                    email
                    displayName
                }
            }
        "#;

        let response: Response = self.query(query, None).await?;
        Ok(response.viewer)
    }

    /// List teams the user has access to.
    pub async fn list_teams(&self) -> LinearResult<Vec<LinearTeam>> {
        #[derive(Deserialize)]
        struct TeamsNode {
            nodes: Vec<LinearTeam>,
        }

        #[derive(Deserialize)]
        struct Response {
            teams: TeamsNode,
        }

        let query = r#"
            query {
                teams {
                    nodes {
                        id
                        name
                        key
                    }
                }
            }
        "#;

        let response: Response = self.query(query, None).await?;
        Ok(response.teams.nodes)
    }

    /// List issues.
    pub async fn list_issues(
        &self,
        options: ListLinearIssuesOptions,
    ) -> LinearResult<Vec<LinearIssue>> {
        #[derive(Deserialize)]
        struct IssuesNode {
            nodes: Vec<LinearIssue>,
        }

        #[derive(Deserialize)]
        struct Response {
            issues: IssuesNode,
        }

        let limit = options.limit.unwrap_or(25);

        // Build filter
        let mut filters: HashMap<&str, serde_json::Value> = HashMap::new();

        if let Some(ref team_id) = options.team_id {
            filters.insert("team", serde_json::json!({ "id": { "eq": team_id } }));
        }

        if let Some(ref assignee_id) = options.assignee_id {
            if assignee_id == "me" {
                filters.insert("assignee", serde_json::json!({ "isMe": { "eq": true } }));
            } else {
                filters.insert("assignee", serde_json::json!({ "id": { "eq": assignee_id } }));
            }
        }

        if !options.include_archived {
            filters.insert(
                "state",
                serde_json::json!({ "type": { "nin": ["completed", "canceled"] } }),
            );
        }

        if let Some(ref state_type) = options.state_type {
            filters.insert("state", serde_json::json!({ "type": { "eq": state_type } }));
        }

        let query = r#"
            query ListIssues($first: Int!, $filter: IssueFilter) {
                issues(first: $first, filter: $filter) {
                    nodes {
                        id
                        identifier
                        title
                        description
                        priority
                        createdAt
                        updatedAt
                        url
                        dueDate
                        estimate
                        state {
                            id
                            name
                            color
                            type
                        }
                        labels {
                            nodes {
                                id
                                name
                                color
                            }
                        }
                        assignee {
                            id
                            name
                            email
                            displayName
                        }
                    }
                }
            }
        "#;

        let variables = serde_json::json!({
            "first": limit,
            "filter": if filters.is_empty() { serde_json::Value::Null } else { serde_json::json!(filters) }
        });

        let response: Response = self.query(query, Some(variables)).await?;
        Ok(response.issues.nodes)
    }

    /// Get a specific issue by identifier (e.g., "ENG-123").
    pub async fn get_issue(&self, identifier: &str) -> LinearResult<LinearIssue> {
        #[derive(Deserialize)]
        struct IssuesNode {
            nodes: Vec<LinearIssue>,
        }

        #[derive(Deserialize)]
        struct Response {
            issues: IssuesNode,
        }

        let query = r#"
            query GetIssue($filter: IssueFilter) {
                issues(filter: $filter, first: 1) {
                    nodes {
                        id
                        identifier
                        title
                        description
                        priority
                        createdAt
                        updatedAt
                        url
                        dueDate
                        estimate
                        state {
                            id
                            name
                            color
                            type
                        }
                        labels {
                            nodes {
                                id
                                name
                                color
                            }
                        }
                        assignee {
                            id
                            name
                            email
                            displayName
                        }
                    }
                }
            }
        "#;

        let (team_key, number) = identifier.split_once('-').ok_or_else(|| {
            LinearError::InvalidInput(format!(
                "Invalid issue identifier '{}'. Expected format: TEAM-123",
                identifier
            ))
        })?;

        let number: i32 = number.parse().map_err(|_| {
            LinearError::InvalidInput(format!(
                "Invalid issue number in identifier '{}'",
                identifier
            ))
        })?;

        let variables = serde_json::json!({
            "filter": {
                "team": { "key": { "eq": team_key } },
                "number": { "eq": number }
            }
        });

        let response: Response = self.query(query, Some(variables)).await?;
        response
            .issues
            .nodes
            .into_iter()
            .next()
            .ok_or_else(|| LinearError::NotFound(format!("Issue {} not found", identifier)))
    }

    /// Create a new issue.
    pub async fn create_issue(
        &self,
        options: CreateLinearIssueOptions,
    ) -> LinearResult<LinearIssue> {
        if options.title.is_empty() {
            return Err(LinearError::InvalidInput("Title is required".to_string()));
        }

        if options.team_id.is_empty() {
            return Err(LinearError::InvalidInput("Team ID is required".to_string()));
        }

        #[derive(Deserialize)]
        struct IssuePayload {
            issue: LinearIssue,
        }

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "issueCreate")]
            issue_create: IssuePayload,
        }

        let query = r#"
            mutation CreateIssue($input: IssueCreateInput!) {
                issueCreate(input: $input) {
                    issue {
                        id
                        identifier
                        title
                        description
                        priority
                        createdAt
                        updatedAt
                        url
                        dueDate
                        estimate
                        state {
                            id
                            name
                            color
                            type
                        }
                        labels {
                            nodes {
                                id
                                name
                                color
                            }
                        }
                        assignee {
                            id
                            name
                            email
                            displayName
                        }
                    }
                }
            }
        "#;

        let mut input: HashMap<&str, serde_json::Value> = HashMap::new();
        input.insert("title", serde_json::json!(options.title));
        input.insert("teamId", serde_json::json!(options.team_id));

        if let Some(ref desc) = options.description {
            input.insert("description", serde_json::json!(desc));
        }
        if let Some(priority) = options.priority {
            input.insert("priority", serde_json::json!(priority));
        }
        if !options.label_ids.is_empty() {
            input.insert("labelIds", serde_json::json!(options.label_ids));
        }
        if let Some(ref assignee) = options.assignee_id {
            input.insert("assigneeId", serde_json::json!(assignee));
        }
        if let Some(ref due_date) = options.due_date {
            input.insert("dueDate", serde_json::json!(due_date));
        }
        if let Some(estimate) = options.estimate {
            input.insert("estimate", serde_json::json!(estimate));
        }

        let variables = serde_json::json!({ "input": input });

        let response: Response = self.query(query, Some(variables)).await?;
        Ok(response.issue_create.issue)
    }

    /// Update an issue's state.
    pub async fn update_issue_state(
        &self,
        issue_id: &str,
        state_id: &str,
    ) -> LinearResult<LinearIssue> {
        #[derive(Deserialize)]
        struct IssuePayload {
            issue: LinearIssue,
        }

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "issueUpdate")]
            issue_update: IssuePayload,
        }

        let query = r#"
            mutation UpdateIssue($id: String!, $input: IssueUpdateInput!) {
                issueUpdate(id: $id, input: $input) {
                    issue {
                        id
                        identifier
                        title
                        description
                        priority
                        createdAt
                        updatedAt
                        url
                        dueDate
                        estimate
                        state {
                            id
                            name
                            color
                            type
                        }
                        labels {
                            nodes {
                                id
                                name
                                color
                            }
                        }
                        assignee {
                            id
                            name
                            email
                            displayName
                        }
                    }
                }
            }
        "#;

        let variables = serde_json::json!({
            "id": issue_id,
            "input": { "stateId": state_id }
        });

        let response: Response = self.query(query, Some(variables)).await?;
        Ok(response.issue_update.issue)
    }

    /// Get workflow states for a team.
    pub async fn get_team_states(&self, team_id: &str) -> LinearResult<Vec<LinearState>> {
        #[derive(Deserialize)]
        struct StatesNode {
            nodes: Vec<LinearState>,
        }

        #[derive(Deserialize)]
        struct Team {
            states: StatesNode,
        }

        #[derive(Deserialize)]
        struct Response {
            team: Team,
        }

        let query = r#"
            query GetTeamStates($teamId: String!) {
                team(id: $teamId) {
                    states {
                        nodes {
                            id
                            name
                            color
                            type
                        }
                    }
                }
            }
        "#;

        let variables = serde_json::json!({ "teamId": team_id });

        let response: Response = self.query(query, Some(variables)).await?;
        Ok(response.team.states.nodes)
    }

    /// Search for issues.
    pub async fn search_issues(&self, query_str: &str) -> LinearResult<Vec<LinearIssue>> {
        #[derive(Deserialize)]
        struct SearchNode {
            nodes: Vec<LinearIssue>,
        }

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "issueSearch")]
            issue_search: SearchNode,
        }

        let query = r#"
            query SearchIssues($query: String!) {
                issueSearch(query: $query, first: 25) {
                    nodes {
                        id
                        identifier
                        title
                        description
                        priority
                        createdAt
                        updatedAt
                        url
                        dueDate
                        estimate
                        state {
                            id
                            name
                            color
                            type
                        }
                        labels {
                            nodes {
                                id
                                name
                                color
                            }
                        }
                        assignee {
                            id
                            name
                            email
                            displayName
                        }
                    }
                }
            }
        "#;

        let variables = serde_json::json!({ "query": query_str });

        let response: Response = self.query(query, Some(variables)).await?;
        Ok(response.issue_search.nodes)
    }

    /// Get statistics for the current user.
    pub async fn get_stats(&self) -> LinearResult<LinearStats> {
        #[derive(Deserialize)]
        struct IssuesNode {
            #[serde(rename = "totalCount")]
            total_count: u64,
        }

        #[derive(Deserialize)]
        struct TeamsNode {
            #[serde(rename = "totalCount")]
            total_count: u64,
        }

        #[derive(Deserialize)]
        struct Response {
            #[serde(rename = "assignedIssues")]
            assigned_issues: IssuesNode,
            #[serde(rename = "inProgressIssues")]
            in_progress_issues: IssuesNode,
            teams: TeamsNode,
        }

        let query = r#"
            query Stats {
                assignedIssues: issues(filter: { assignee: { isMe: { eq: true } }, state: { type: { nin: ["completed", "canceled"] } } }) {
                    totalCount
                }
                inProgressIssues: issues(filter: { assignee: { isMe: { eq: true } }, state: { type: { eq: "started" } } }) {
                    totalCount
                }
                teams {
                    totalCount
                }
            }
        "#;

        let response: Response = self.query(query, None).await?;

        Ok(LinearStats {
            assigned_count: response.assigned_issues.total_count,
            in_progress_count: response.in_progress_issues.total_count,
            completed_count: 0, // Would require cycle context
            team_count: response.teams.total_count,
        })
    }
}

/// Format a Linear issue for display.
pub fn format_linear_issue(issue: &LinearIssue, verbose: bool) -> String {
    let state_icon = match issue.state.state_type.as_str() {
        "backlog" => "○",
        "unstarted" => "◌",
        "started" => "◐",
        "completed" => "●",
        "canceled" => "⊘",
        _ => "?",
    };

    let priority_icon = match issue.priority {
        0 => "",
        1 => "⚡",
        2 => "🔺",
        3 => "▲",
        4 => "▽",
        _ => "",
    };

    let labels = if issue.labels.is_empty() {
        String::new()
    } else {
        let label_names: Vec<&str> = issue.labels.iter().map(|l| l.name.as_str()).collect();
        format!(" [{}]", label_names.join(", "))
    };

    let assignee =
        issue.assignee.as_ref().map(|a| format!(" → {}", a.display_name)).unwrap_or_default();

    if verbose {
        let desc = issue
            .description
            .as_ref()
            .map(|d| {
                let truncated = if d.len() > 200 { format!("{}...", &d[..200]) } else { d.clone() };
                format!("\n  {}", truncated.replace('\n', "\n  "))
            })
            .unwrap_or_default();

        format!(
            "{} {} {} {}{}{}{}",
            state_icon, priority_icon, issue.identifier, issue.title, labels, assignee, desc
        )
    } else {
        format!(
            "{} {} {} {}{}{}",
            state_icon, priority_icon, issue.identifier, issue.title, labels, assignee
        )
    }
}

/// Format Linear statistics for display.
pub fn format_linear_stats(stats: &LinearStats) -> String {
    format!(
        "Assigned: {}  In Progress: {}  Teams: {}",
        stats.assigned_count, stats.in_progress_count, stats.team_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_linear_issue() {
        let issue = LinearIssue {
            id: "123".to_string(),
            identifier: "ENG-42".to_string(),
            title: "Test issue".to_string(),
            description: Some("This is a test".to_string()),
            state: LinearState {
                id: "state-1".to_string(),
                name: "In Progress".to_string(),
                color: "#f00".to_string(),
                state_type: "started".to_string(),
            },
            priority: 2,
            labels: vec![LinearLabel {
                id: "label-1".to_string(),
                name: "bug".to_string(),
                color: "#d73a4a".to_string(),
            }],
            assignee: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            url: "https://linear.app/team/issue/ENG-42".to_string(),
            due_date: None,
            estimate: None,
        };

        let output = format_linear_issue(&issue, false);
        assert!(output.contains("ENG-42"));
        assert!(output.contains("Test issue"));
        assert!(output.contains("[bug]"));
    }

    #[test]
    fn test_create_issue_options_default() {
        let options = CreateLinearIssueOptions::default();
        assert!(options.title.is_empty());
        assert!(options.description.is_none());
    }

    #[test]
    fn test_list_issues_options_default() {
        let options = ListLinearIssuesOptions::default();
        assert!(options.team_id.is_none());
        assert!(!options.include_archived);
    }
}
