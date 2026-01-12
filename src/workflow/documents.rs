//! Workflow document structures.
//!
//! Defines the document types used for AI-assisted project management.

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Project document - vision, requirements, constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDoc {
    /// Project name
    pub name: String,

    /// Brief description
    pub description: String,

    /// Core requirements
    pub requirements: Vec<String>,

    /// Technical constraints
    pub constraints: Vec<String>,

    /// Target audience
    pub audience: Option<String>,

    /// Success criteria
    pub success_criteria: Vec<String>,
}

impl ProjectDoc {
    /// Load from a PROJECT.md file.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse from markdown content.
    pub fn parse(content: &str) -> anyhow::Result<Self> {
        let mut doc = Self {
            name: String::new(),
            description: String::new(),
            requirements: Vec::new(),
            constraints: Vec::new(),
            audience: None,
            success_criteria: Vec::new(),
        };

        let mut current_section = "";

        for line in content.lines() {
            let line = line.trim();

            // Extract title from first H1
            if line.starts_with("# ") && doc.name.is_empty() {
                doc.name = line.trim_start_matches("# ").to_string();
                continue;
            }

            // Track sections
            if line.starts_with("## ") {
                current_section = line.trim_start_matches("## ").trim();
                continue;
            }

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Parse content based on section
            match current_section.to_lowercase().as_str() {
                "description" | "overview" | "about" => {
                    if !doc.description.is_empty() {
                        doc.description.push(' ');
                    }
                    doc.description.push_str(line);
                }
                "requirements" | "features" | "goals" => {
                    if let Some(item) = parse_list_item(line) {
                        doc.requirements.push(item);
                    }
                }
                "constraints" | "limitations" | "technical constraints" => {
                    if let Some(item) = parse_list_item(line) {
                        doc.constraints.push(item);
                    }
                }
                "audience" | "target audience" | "users" => {
                    doc.audience = Some(line.to_string());
                }
                "success criteria" | "success" | "metrics" => {
                    if let Some(item) = parse_list_item(line) {
                        doc.success_criteria.push(item);
                    }
                }
                _ => {}
            }
        }

        if doc.name.is_empty() {
            anyhow::bail!("PROJECT.md must have a title (# Project Name)");
        }

        Ok(doc)
    }

    /// Generate markdown template.
    pub fn template(name: &str) -> String {
        format!(
            r#"# {name}

## Description

[Brief description of your project]

## Requirements

- [ ] Core feature 1
- [ ] Core feature 2
- [ ] Core feature 3

## Constraints

- Must use [technology/framework]
- Must integrate with [existing system]
- Performance requirements: [specify]

## Target Audience

[Who is this project for?]

## Success Criteria

- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3
"#
        )
    }

    /// Convert to prompt context string.
    pub fn to_context(&self, max_chars: usize) -> String {
        let mut ctx = format!("Project: {}\n", self.name);

        if !self.description.is_empty() {
            ctx.push_str(&format!("Description: {}\n", self.description));
        }

        if !self.requirements.is_empty() {
            ctx.push_str("Requirements:\n");
            for req in &self.requirements {
                if ctx.len() + req.len() > max_chars {
                    break;
                }
                ctx.push_str(&format!("- {req}\n"));
            }
        }

        ctx
    }
}

/// Roadmap document - phases and milestones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapDoc {
    /// Project name (from title)
    pub project: String,

    /// Phases in the roadmap
    pub phases: Vec<Phase>,

    /// Current phase index (0-based)
    pub current_phase: usize,
}

/// A phase in the roadmap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase {
    /// Phase number
    pub number: usize,

    /// Phase name
    pub name: String,

    /// Phase description
    pub description: String,

    /// Deliverables
    pub deliverables: Vec<String>,

    /// Status
    pub status: PhaseStatus,
}

/// Phase status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhaseStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
}

impl Default for PhaseStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl RoadmapDoc {
    /// Load from a ROADMAP.md file.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse from markdown content.
    pub fn parse(content: &str) -> anyhow::Result<Self> {
        let mut doc = Self { project: String::new(), phases: Vec::new(), current_phase: 0 };

        let mut current_phase: Option<Phase> = None;
        let mut in_deliverables = false;

        for line in content.lines() {
            let line = line.trim();

            // Extract title
            if line.starts_with("# ") && doc.project.is_empty() {
                doc.project =
                    line.trim_start_matches("# ").trim_end_matches("Roadmap").trim().to_string();
                continue;
            }

            // New phase (## Phase N: Name)
            if line.starts_with("## Phase ") || line.starts_with("## ") {
                // Save previous phase
                if let Some(phase) = current_phase.take() {
                    doc.phases.push(phase);
                }

                // Parse phase header
                let header = line.trim_start_matches("## ").trim_start_matches("Phase ");
                let (num_str, name) = header.split_once(':').unwrap_or(("1", header));
                let number = num_str.trim().parse().unwrap_or(doc.phases.len() + 1);

                current_phase = Some(Phase {
                    number,
                    name: name.trim().to_string(),
                    description: String::new(),
                    deliverables: Vec::new(),
                    status: PhaseStatus::Pending,
                });
                in_deliverables = false;
                continue;
            }

            // Check for deliverables section
            if line.to_lowercase().contains("deliverable") {
                in_deliverables = true;
                continue;
            }

            // Parse content within phase
            if let Some(ref mut phase) = current_phase {
                if line.contains("Status:") {
                    let status_str = extract_value(line).to_lowercase();
                    phase.status = match status_str.as_str() {
                        "completed" | "done" | "complete" => PhaseStatus::Completed,
                        "in progress" | "in-progress" | "active" => PhaseStatus::InProgress,
                        "blocked" => PhaseStatus::Blocked,
                        _ => PhaseStatus::Pending,
                    };
                } else if in_deliverables {
                    if let Some(item) = parse_list_item(line) {
                        phase.deliverables.push(item);
                    }
                } else if !line.is_empty() && phase.description.is_empty() {
                    phase.description = line.to_string();
                }
            }
        }

        // Don't forget the last phase
        if let Some(phase) = current_phase {
            doc.phases.push(phase);
        }

        // Determine current phase
        for (i, phase) in doc.phases.iter().enumerate() {
            if phase.status == PhaseStatus::InProgress {
                doc.current_phase = i;
                break;
            }
            if phase.status == PhaseStatus::Pending && doc.current_phase == 0 {
                doc.current_phase = i;
            }
        }

        Ok(doc)
    }

    /// Generate markdown template.
    pub fn template(project: &str) -> String {
        format!(
            r#"# {project} Roadmap

## Phase 1: Foundation

Set up the basic project structure and core functionality.

**Status:** Pending

### Deliverables

- [ ] Project scaffolding
- [ ] Core module structure
- [ ] Basic tests

## Phase 2: Core Features

Implement the main features.

**Status:** Pending

### Deliverables

- [ ] Feature 1
- [ ] Feature 2
- [ ] Integration tests

## Phase 3: Polish & Launch

Final polish and release.

**Status:** Pending

### Deliverables

- [ ] Documentation
- [ ] Performance optimization
- [ ] Release preparation
"#
        )
    }

    /// Get the current phase.
    pub fn current(&self) -> Option<&Phase> {
        self.phases.get(self.current_phase)
    }

    /// Convert to prompt context string.
    pub fn to_context(&self, max_chars: usize) -> String {
        let mut ctx = format!("Roadmap: {} phases\n", self.phases.len());
        ctx.push_str(&format!("Current Phase: {}\n", self.current_phase + 1));

        if let Some(phase) = self.current() {
            ctx.push_str(&format!("Phase {}: {}\n", phase.number, phase.name));
            if !phase.description.is_empty() && ctx.len() + phase.description.len() < max_chars {
                ctx.push_str(&format!("{}\n", phase.description));
            }
        }

        ctx
    }
}

/// State document - current position, decisions, blockers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDoc {
    /// Current phase number
    pub current_phase: usize,

    /// Current plan ID
    pub current_plan: Option<String>,

    /// Current task number within plan
    pub current_task: usize,

    /// Overall status
    pub status: String,

    /// Active decisions
    pub decisions: Vec<Decision>,

    /// Current blockers
    pub blockers: Vec<String>,

    /// Recent changes
    pub recent_changes: Vec<String>,

    /// Deferred items
    pub deferred: Vec<String>,
}

/// An active decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    /// What was decided
    pub decision: String,

    /// Why it was decided
    pub rationale: String,

    /// Decision status
    pub status: String,
}

impl Default for StateDoc {
    fn default() -> Self {
        Self {
            current_phase: 1,
            current_plan: None,
            current_task: 0,
            status: "Not Started".to_string(),
            decisions: Vec::new(),
            blockers: Vec::new(),
            recent_changes: Vec::new(),
            deferred: Vec::new(),
        }
    }
}

impl StateDoc {
    /// Load from a STATE.md file.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse from markdown content.
    pub fn parse(content: &str) -> anyhow::Result<Self> {
        let mut doc = Self::default();
        let mut current_section = "";

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("## ") {
                current_section = line.trim_start_matches("## ").trim();
                continue;
            }

            if line.is_empty() {
                continue;
            }

            match current_section.to_lowercase().as_str() {
                "current position" | "position" | "status" => {
                    if line.contains("Phase:") {
                        if let Some(num) = extract_number(line) {
                            doc.current_phase = num;
                        }
                    } else if line.contains("Plan:") {
                        let plan = extract_value(line);
                        if !plan.is_empty() && plan != "none" {
                            doc.current_plan = Some(plan);
                        }
                    } else if line.contains("Task:") {
                        if let Some(num) = extract_number(line) {
                            doc.current_task = num;
                        }
                    } else if line.contains("Status:") {
                        doc.status = extract_value(line);
                    }
                }
                "blockers" | "blocked" => {
                    if let Some(item) = parse_list_item(line) {
                        doc.blockers.push(item);
                    }
                }
                "recent changes" | "changes" | "history" => {
                    if let Some(item) = parse_list_item(line) {
                        doc.recent_changes.push(item);
                    }
                }
                "deferred" | "backlog" | "later" => {
                    if let Some(item) = parse_list_item(line) {
                        doc.deferred.push(item);
                    }
                }
                _ => {}
            }
        }

        Ok(doc)
    }

    /// Generate markdown template.
    pub fn template() -> String {
        r#"# Project State

## Current Position

- **Phase:** 1 of 1
- **Plan:** none
- **Task:** 0 of 0
- **Status:** Not Started

## Active Decisions

| Decision | Rationale | Status |
|----------|-----------|--------|
| - | - | - |

## Blockers

(none)

## Recent Changes

- Initial state created

## Deferred

(none)
"#
        .to_string()
    }

    /// Save to file.
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let content = self.to_markdown();
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Convert to markdown.
    pub fn to_markdown(&self) -> String {
        let mut md = String::from("# Project State\n\n");

        md.push_str("## Current Position\n\n");
        md.push_str(&format!("- **Phase:** {}\n", self.current_phase));
        md.push_str(&format!("- **Plan:** {}\n", self.current_plan.as_deref().unwrap_or("none")));
        md.push_str(&format!("- **Task:** {}\n", self.current_task));
        md.push_str(&format!("- **Status:** {}\n\n", self.status));

        if !self.blockers.is_empty() {
            md.push_str("## Blockers\n\n");
            for blocker in &self.blockers {
                md.push_str(&format!("- [ ] {blocker}\n"));
            }
            md.push('\n');
        }

        if !self.recent_changes.is_empty() {
            md.push_str("## Recent Changes\n\n");
            for change in self.recent_changes.iter().take(10) {
                md.push_str(&format!("- {change}\n"));
            }
            md.push('\n');
        }

        md
    }

    /// Convert to prompt context string.
    pub fn to_context(&self, _max_chars: usize) -> String {
        let blockers_str =
            if self.blockers.is_empty() { "none".to_string() } else { self.blockers.join(", ") };
        format!(
            "State: Phase {}, Task {}, Status: {}\nBlockers: {}\n",
            self.current_phase, self.current_task, self.status, blockers_str
        )
    }
}

/// Plan document - current task plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDoc {
    /// Plan ID
    pub id: String,

    /// Plan name
    pub name: String,

    /// Phase this plan belongs to
    pub phase: usize,

    /// Tasks in this plan
    pub tasks: Vec<Task>,
}

impl PlanDoc {
    /// Load from a PLAN.md file.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse from markdown content.
    pub fn parse(content: &str) -> anyhow::Result<Self> {
        let mut doc = Self { id: String::new(), name: String::new(), phase: 1, tasks: Vec::new() };

        let mut current_task: Option<Task> = None;
        let mut in_steps = false;
        let mut in_verify = false;

        for line in content.lines() {
            let line = line.trim();

            // Extract title
            if line.starts_with("# ") && doc.name.is_empty() {
                doc.name = line.trim_start_matches("# ").to_string();
                doc.id = slugify(&doc.name);
                continue;
            }

            // Task header (## Task N: Name)
            if line.starts_with("## Task ") {
                if let Some(task) = current_task.take() {
                    doc.tasks.push(task);
                }

                let header = line.trim_start_matches("## Task ");
                let (num_str, name) = header.split_once(':').unwrap_or(("1", header));
                let id = num_str.trim().parse().unwrap_or(doc.tasks.len() + 1);

                current_task = Some(Task {
                    id,
                    name: name.trim().to_string(),
                    task_type: TaskType::Auto,
                    status: TaskStatus::Pending,
                    files: Vec::new(),
                    context: String::new(),
                    steps: Vec::new(),
                    verify: Vec::new(),
                    done_criteria: String::new(),
                });
                in_steps = false;
                in_verify = false;
                continue;
            }

            // Section markers
            if line.to_lowercase().starts_with("### steps")
                || line.to_lowercase().starts_with("**steps")
            {
                in_steps = true;
                in_verify = false;
                continue;
            }
            if line.to_lowercase().starts_with("### verify")
                || line.to_lowercase().starts_with("**verify")
            {
                in_steps = false;
                in_verify = true;
                continue;
            }
            if line.to_lowercase().starts_with("### files")
                || line.to_lowercase().starts_with("**files")
            {
                in_steps = false;
                in_verify = false;
                continue;
            }

            // Parse task content
            if let Some(ref mut task) = current_task {
                if line.starts_with("**Type:**") || line.starts_with("Type:") {
                    let type_str = line.split(':').nth(1).unwrap_or("").trim().to_lowercase();
                    task.task_type = match type_str.as_str() {
                        "manual" => TaskType::Manual,
                        "review" => TaskType::Review,
                        _ => TaskType::Auto,
                    };
                } else if line.starts_with("**Status:**") || line.starts_with("Status:") {
                    let status_str = line.split(':').nth(1).unwrap_or("").trim().to_lowercase();
                    task.status = match status_str.as_str() {
                        "in progress" | "in-progress" | "active" => TaskStatus::InProgress,
                        "completed" | "done" | "complete" => TaskStatus::Completed,
                        "blocked" => TaskStatus::Blocked,
                        "skipped" => TaskStatus::Skipped,
                        _ => TaskStatus::Pending,
                    };
                } else if in_steps {
                    if let Some(item) = parse_list_item(line) {
                        task.steps.push(item);
                    }
                } else if in_verify {
                    if let Some(item) = parse_list_item(line) {
                        task.verify.push(item);
                    }
                } else if line.starts_with("- `") || line.starts_with("* `") {
                    // File paths
                    if let Some(file) = line.split('`').nth(1) {
                        task.files.push(file.to_string());
                    }
                }
            }
        }

        // Don't forget the last task
        if let Some(task) = current_task {
            doc.tasks.push(task);
        }

        Ok(doc)
    }

    /// Generate markdown template.
    pub fn template(name: &str, phase: usize) -> String {
        format!(
            r#"# {name}

**Phase:** {phase}

## Task 1: Initial Setup

**Type:** auto
**Status:** pending

### Files

- `src/main.rs`

### Steps

1. Create initial structure
2. Add dependencies
3. Implement basic functionality

### Verify

- [ ] Code compiles
- [ ] Tests pass

### Done

Initial setup complete with working code.
"#
        )
    }

    /// Get pending tasks.
    pub fn pending_tasks(&self) -> Vec<&Task> {
        self.tasks.iter().filter(|t| t.status == TaskStatus::Pending).collect()
    }

    /// Get the next task to work on.
    pub fn next_task(&self) -> Option<&Task> {
        self.tasks
            .iter()
            .find(|t| t.status == TaskStatus::InProgress)
            .or_else(|| self.tasks.iter().find(|t| t.status == TaskStatus::Pending))
    }

    /// Convert to prompt context string.
    pub fn to_context(&self, max_chars: usize) -> String {
        let mut ctx = format!("Plan: {} ({} tasks)\n", self.name, self.tasks.len());

        if let Some(task) = self.next_task() {
            ctx.push_str(&format!("Current Task {}: {}\n", task.id, task.name));
            if !task.steps.is_empty() {
                ctx.push_str("Steps:\n");
                for (i, step) in task.steps.iter().enumerate() {
                    if ctx.len() + step.len() > max_chars {
                        break;
                    }
                    ctx.push_str(&format!("{}. {step}\n", i + 1));
                }
            }
        }

        ctx
    }
}

/// A task in a plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task ID (number)
    pub id: usize,

    /// Task name
    pub name: String,

    /// Task type
    pub task_type: TaskType,

    /// Task status
    pub status: TaskStatus,

    /// Files to modify
    pub files: Vec<String>,

    /// Context for AI
    pub context: String,

    /// Steps to complete
    pub steps: Vec<String>,

    /// Verification steps
    pub verify: Vec<String>,

    /// Done criteria
    pub done_criteria: String,
}

/// Task type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    /// AI can complete automatically
    Auto,
    /// Requires human action
    Manual,
    /// Requires human review
    Review,
}

impl Default for TaskType {
    fn default() -> Self {
        Self::Auto
    }
}

/// Task status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
    Skipped,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Pending
    }
}

// Helper functions

fn parse_list_item(line: &str) -> Option<String> {
    let line = line.trim();
    if line.starts_with("- ") {
        Some(line.trim_start_matches("- ").trim_start_matches("[ ] ").to_string())
    } else if line.starts_with("* ") {
        Some(line.trim_start_matches("* ").trim_start_matches("[ ] ").to_string())
    } else if line.chars().next().is_some_and(|c| c.is_ascii_digit()) && line.contains(". ") {
        line.split_once(". ").map(|(_, rest)| rest.to_string())
    } else {
        None
    }
}

fn extract_number(line: &str) -> Option<usize> {
    line.split_whitespace()
        .find_map(|word| word.trim_end_matches(|c: char| !c.is_ascii_digit()).parse().ok())
}

/// Extract value after a colon, stripping markdown bold markers.
fn extract_value(line: &str) -> String {
    line.split(':')
        .nth(1)
        .unwrap_or("")
        .trim()
        .trim_start_matches("**")
        .trim_end_matches("**")
        .trim()
        .to_string()
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_doc_parse() {
        let content = r#"# My Project

## Description

A test project for demonstration.

## Requirements

- Feature A
- Feature B

## Constraints

- Must use Rust
"#;
        let doc = ProjectDoc::parse(content).unwrap();
        assert_eq!(doc.name, "My Project");
        assert_eq!(doc.requirements.len(), 2);
        assert_eq!(doc.constraints.len(), 1);
    }

    #[test]
    fn test_roadmap_doc_parse() {
        let content = r#"# Test Roadmap

## Phase 1: Setup

Initial setup phase.

**Status:** Completed

### Deliverables

- Project structure
- Basic tests

## Phase 2: Features

Main features.

**Status:** In Progress
"#;
        let doc = RoadmapDoc::parse(content).unwrap();
        assert_eq!(doc.phases.len(), 2);
        assert_eq!(doc.phases[0].status, PhaseStatus::Completed);
        assert_eq!(doc.phases[1].status, PhaseStatus::InProgress);
        assert_eq!(doc.current_phase, 1); // Phase 2 is in progress
    }

    #[test]
    fn test_state_doc_parse() {
        let content = r#"# Project State

## Current Position

- **Phase:** 2 of 5
- **Plan:** auth-implementation
- **Task:** 3 of 4
- **Status:** In Progress

## Blockers

- Waiting for API docs
"#;
        let doc = StateDoc::parse(content).unwrap();
        assert_eq!(doc.current_phase, 2);
        assert_eq!(doc.current_plan, Some("auth-implementation".to_string()));
        assert_eq!(doc.current_task, 3);
        assert_eq!(doc.blockers.len(), 1);
    }

    #[test]
    fn test_plan_doc_parse() {
        let content = r#"# Authentication Plan

**Phase:** 2

## Task 1: Setup Auth Module

**Type:** auto
**Status:** pending

### Files

- `src/auth/mod.rs`

### Steps

1. Create module
2. Add structs
3. Implement logic

### Verify

- Tests pass
"#;
        let doc = PlanDoc::parse(content).unwrap();
        assert_eq!(doc.name, "Authentication Plan");
        assert_eq!(doc.tasks.len(), 1);
        assert_eq!(doc.tasks[0].steps.len(), 3);
        assert_eq!(doc.tasks[0].files.len(), 1);
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Phase 1: Setup"), "phase-1-setup");
    }
}
