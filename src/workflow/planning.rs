//! Plan generation and management.
//!
//! Generates executable plans from roadmap phases.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::documents::{Phase, PlanDoc, Task, TaskStatus, TaskType};

/// Plan generator that creates plans from roadmap phases.
pub struct PlanGenerator {
    /// AI provider name to use for generation (optional)
    pub ai_provider: Option<String>,
}

impl PlanGenerator {
    /// Create a new plan generator.
    pub fn new() -> Self {
        Self { ai_provider: None }
    }

    /// Set the AI provider to use for generation.
    pub fn with_ai(mut self, provider: impl Into<String>) -> Self {
        self.ai_provider = Some(provider.into());
        self
    }

    /// Generate a basic plan from a roadmap phase.
    ///
    /// This creates a skeleton plan that can be refined with AI.
    pub fn generate_basic(&self, phase: &Phase, phase_number: usize) -> PlanDoc {
        let plan_name = format!("Phase {}: {}", phase_number, phase.name);

        // Convert deliverables to tasks
        let tasks: Vec<Task> = phase
            .deliverables
            .iter()
            .enumerate()
            .map(|(i, deliverable)| Task {
                id: i + 1,
                name: deliverable.clone(),
                task_type: TaskType::Auto,
                status: TaskStatus::Pending,
                files: Vec::new(),
                context: String::new(),
                steps: Vec::new(),
                verify: Vec::new(),
                done_criteria: format!("{} is complete and working", deliverable),
            })
            .collect();

        PlanDoc { id: slugify(&plan_name), name: plan_name, phase: phase_number, tasks }
    }

    /// Generate a detailed plan from a phase with context.
    ///
    /// This creates a more detailed plan based on the phase description
    /// and any existing codebase context.
    pub fn generate_detailed(
        &self,
        phase: &Phase,
        phase_number: usize,
        context: Option<&str>,
    ) -> PlanDoc {
        let plan_name = format!("Phase {}: {}", phase_number, phase.name);

        // Start with deliverables as tasks
        let mut tasks: Vec<Task> = phase
            .deliverables
            .iter()
            .enumerate()
            .map(|(i, deliverable)| {
                // Generate basic steps based on deliverable name
                let steps = generate_steps_for_deliverable(deliverable);
                let verify = generate_verify_for_deliverable(deliverable);

                Task {
                    id: i + 1,
                    name: deliverable.clone(),
                    task_type: infer_task_type(deliverable),
                    status: TaskStatus::Pending,
                    files: Vec::new(),
                    context: context.unwrap_or("").to_string(),
                    steps,
                    verify,
                    done_criteria: format!("{} is complete and verified", deliverable),
                }
            })
            .collect();

        // If no deliverables, create a single task from description
        if tasks.is_empty() {
            tasks.push(Task {
                id: 1,
                name: phase.name.clone(),
                task_type: TaskType::Auto,
                status: TaskStatus::Pending,
                files: Vec::new(),
                context: phase.description.clone(),
                steps: vec!["Implement the phase requirements".to_string()],
                verify: vec!["All tests pass".to_string()],
                done_criteria: format!("{} is complete", phase.name),
            });
        }

        PlanDoc { id: slugify(&plan_name), name: plan_name, phase: phase_number, tasks }
    }

    /// Save a plan to a PLAN.md file.
    pub fn save_plan(&self, plan: &PlanDoc, dir: &Path) -> anyhow::Result<()> {
        let path = dir.join("PLAN.md");
        let content = plan.to_markdown();
        std::fs::write(&path, content)?;
        Ok(())
    }
}

impl Default for PlanGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl PlanDoc {
    /// Convert to markdown format.
    pub fn to_markdown(&self) -> String {
        let mut md = format!("# {}\n\n", self.name);
        md.push_str(&format!("**Phase:** {}\n\n", self.phase));

        for task in &self.tasks {
            md.push_str(&format!("## Task {}: {}\n\n", task.id, task.name));
            md.push_str(&format!(
                "**Type:** {}\n",
                match task.task_type {
                    TaskType::Auto => "auto",
                    TaskType::Manual => "manual",
                    TaskType::Review => "review",
                }
            ));
            md.push_str(&format!(
                "**Status:** {}\n\n",
                match task.status {
                    TaskStatus::Pending => "pending",
                    TaskStatus::InProgress => "in-progress",
                    TaskStatus::Completed => "completed",
                    TaskStatus::Blocked => "blocked",
                    TaskStatus::Skipped => "skipped",
                }
            ));

            if !task.files.is_empty() {
                md.push_str("### Files\n\n");
                for file in &task.files {
                    md.push_str(&format!("- `{file}`\n"));
                }
                md.push('\n');
            }

            if !task.context.is_empty() {
                md.push_str("### Context\n\n");
                md.push_str(&task.context);
                md.push_str("\n\n");
            }

            if !task.steps.is_empty() {
                md.push_str("### Steps\n\n");
                for (i, step) in task.steps.iter().enumerate() {
                    md.push_str(&format!("{}. {step}\n", i + 1));
                }
                md.push('\n');
            }

            if !task.verify.is_empty() {
                md.push_str("### Verify\n\n");
                for v in &task.verify {
                    md.push_str(&format!("- [ ] {v}\n"));
                }
                md.push('\n');
            }

            if !task.done_criteria.is_empty() {
                md.push_str("### Done\n\n");
                md.push_str(&task.done_criteria);
                md.push_str("\n\n");
            }
        }

        md
    }

    /// Mark a task as in progress.
    pub fn start_task(&mut self, task_id: usize) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == task_id).map(|t| {
            t.status = TaskStatus::InProgress;
            t
        })
    }

    /// Mark a task as completed.
    pub fn complete_task(&mut self, task_id: usize) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == task_id).map(|t| {
            t.status = TaskStatus::Completed;
            t
        })
    }

    /// Mark a task as blocked.
    pub fn block_task(&mut self, task_id: usize, reason: &str) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == task_id).map(|t| {
            t.status = TaskStatus::Blocked;
            t.context = format!("BLOCKED: {}", reason);
            t
        })
    }

    /// Get progress as (completed, total).
    pub fn progress(&self) -> (usize, usize) {
        let completed = self.tasks.iter().filter(|t| t.status == TaskStatus::Completed).count();
        (completed, self.tasks.len())
    }

    /// Check if plan is complete.
    pub fn is_complete(&self) -> bool {
        self.tasks
            .iter()
            .all(|t| t.status == TaskStatus::Completed || t.status == TaskStatus::Skipped)
    }
}

/// Execution result for a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Task ID
    pub task_id: usize,

    /// Whether task succeeded
    pub success: bool,

    /// Output/summary
    pub output: String,

    /// Files modified
    pub files_modified: Vec<String>,

    /// Verification results
    pub verification: Vec<VerificationResult>,
}

/// Result of a verification step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// The verification step
    pub step: String,

    /// Whether it passed
    pub passed: bool,

    /// Output/details
    pub output: String,
}

// Helper functions

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

fn infer_task_type(deliverable: &str) -> TaskType {
    let lower = deliverable.to_lowercase();
    if lower.contains("review") || lower.contains("approve") {
        TaskType::Review
    } else if lower.contains("deploy")
        || lower.contains("configure")
        || lower.contains("setup")
        || lower.contains("manual")
    {
        TaskType::Manual
    } else {
        TaskType::Auto
    }
}

fn generate_steps_for_deliverable(deliverable: &str) -> Vec<String> {
    let lower = deliverable.to_lowercase();

    if lower.contains("test") {
        vec![
            "Create test file structure".to_string(),
            "Write unit tests".to_string(),
            "Add integration tests if needed".to_string(),
            "Ensure all tests pass".to_string(),
        ]
    } else if lower.contains("document") || lower.contains("readme") {
        vec![
            "Outline document structure".to_string(),
            "Write main content".to_string(),
            "Add examples and code snippets".to_string(),
            "Review for clarity".to_string(),
        ]
    } else if lower.contains("api") || lower.contains("endpoint") {
        vec![
            "Define API interface".to_string(),
            "Implement handler logic".to_string(),
            "Add input validation".to_string(),
            "Write API tests".to_string(),
            "Update API documentation".to_string(),
        ]
    } else if lower.contains("database") || lower.contains("schema") {
        vec![
            "Design schema structure".to_string(),
            "Create migration files".to_string(),
            "Implement models".to_string(),
            "Add database tests".to_string(),
        ]
    } else if lower.contains("ui") || lower.contains("component") {
        vec![
            "Create component structure".to_string(),
            "Implement layout".to_string(),
            "Add styling".to_string(),
            "Connect to data/state".to_string(),
            "Add component tests".to_string(),
        ]
    } else {
        vec![
            format!("Implement {}", deliverable),
            "Add tests".to_string(),
            "Verify functionality".to_string(),
        ]
    }
}

fn generate_verify_for_deliverable(deliverable: &str) -> Vec<String> {
    let lower = deliverable.to_lowercase();

    let mut verifications = vec!["Code compiles without errors".to_string()];

    if lower.contains("test") {
        verifications.push("All tests pass".to_string());
        verifications.push("Code coverage is adequate".to_string());
    } else if lower.contains("api") || lower.contains("endpoint") {
        verifications.push("API responds correctly".to_string());
        verifications.push("Error cases handled".to_string());
        verifications.push("API tests pass".to_string());
    } else if lower.contains("document") {
        verifications.push("Documentation is complete".to_string());
        verifications.push("Examples work correctly".to_string());
    } else {
        verifications.push("Tests pass".to_string());
        verifications.push("Functionality works as expected".to_string());
    }

    verifications
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::documents::PhaseStatus;

    #[test]
    fn test_plan_generator_basic() {
        let phase = Phase {
            number: 1,
            name: "Foundation".to_string(),
            description: "Set up project".to_string(),
            deliverables: vec!["Project structure".to_string(), "Basic tests".to_string()],
            status: PhaseStatus::Pending,
        };

        let generator = PlanGenerator::new();
        let plan = generator.generate_basic(&phase, 1);

        assert_eq!(plan.phase, 1);
        assert_eq!(plan.tasks.len(), 2);
        assert_eq!(plan.tasks[0].name, "Project structure");
        assert_eq!(plan.tasks[1].name, "Basic tests");
    }

    #[test]
    fn test_plan_generator_detailed() {
        let phase = Phase {
            number: 2,
            name: "Features".to_string(),
            description: "Implement features".to_string(),
            deliverables: vec!["API endpoint".to_string(), "Unit tests".to_string()],
            status: PhaseStatus::InProgress,
        };

        let generator = PlanGenerator::new();
        let plan = generator.generate_detailed(&phase, 2, None);

        assert_eq!(plan.phase, 2);
        assert_eq!(plan.tasks.len(), 2);
        // API tasks should have steps
        assert!(!plan.tasks[0].steps.is_empty());
        // Tests should have verify steps
        assert!(!plan.tasks[1].verify.is_empty());
    }

    #[test]
    fn test_plan_doc_to_markdown() {
        let plan = PlanDoc {
            id: "test-plan".to_string(),
            name: "Test Plan".to_string(),
            phase: 1,
            tasks: vec![Task {
                id: 1,
                name: "Test Task".to_string(),
                task_type: TaskType::Auto,
                status: TaskStatus::Pending,
                files: vec!["src/main.rs".to_string()],
                context: String::new(),
                steps: vec!["Do thing".to_string()],
                verify: vec!["It works".to_string()],
                done_criteria: "Task complete".to_string(),
            }],
        };

        let md = plan.to_markdown();
        assert!(md.contains("# Test Plan"));
        assert!(md.contains("## Task 1: Test Task"));
        assert!(md.contains("**Type:** auto"));
        assert!(md.contains("`src/main.rs`"));
        assert!(md.contains("1. Do thing"));
        assert!(md.contains("- [ ] It works"));
    }

    #[test]
    fn test_plan_task_lifecycle() {
        let mut plan = PlanDoc {
            id: "test".to_string(),
            name: "Test".to_string(),
            phase: 1,
            tasks: vec![
                Task {
                    id: 1,
                    name: "Task 1".to_string(),
                    task_type: TaskType::Auto,
                    status: TaskStatus::Pending,
                    files: Vec::new(),
                    context: String::new(),
                    steps: Vec::new(),
                    verify: Vec::new(),
                    done_criteria: String::new(),
                },
                Task {
                    id: 2,
                    name: "Task 2".to_string(),
                    task_type: TaskType::Auto,
                    status: TaskStatus::Pending,
                    files: Vec::new(),
                    context: String::new(),
                    steps: Vec::new(),
                    verify: Vec::new(),
                    done_criteria: String::new(),
                },
            ],
        };

        assert_eq!(plan.progress(), (0, 2));
        assert!(!plan.is_complete());

        plan.start_task(1);
        assert_eq!(plan.tasks[0].status, TaskStatus::InProgress);

        plan.complete_task(1);
        assert_eq!(plan.tasks[0].status, TaskStatus::Completed);
        assert_eq!(plan.progress(), (1, 2));

        plan.complete_task(2);
        assert!(plan.is_complete());
    }

    #[test]
    fn test_infer_task_type() {
        assert_eq!(infer_task_type("Review PR"), TaskType::Review);
        assert_eq!(infer_task_type("Deploy to production"), TaskType::Manual);
        assert_eq!(infer_task_type("Implement feature"), TaskType::Auto);
    }
}
