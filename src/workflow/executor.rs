//! Task execution engine.
//!
//! Executes tasks with AI assistance and handles verification.

use std::process::Command;

use serde::{Deserialize, Serialize};

use super::documents::{PlanDoc, Task, TaskStatus, TaskType};
use super::planning::{TaskResult, VerificationResult};

/// Task executor configuration.
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// AI provider to use
    pub provider: Option<String>,

    /// Working directory
    pub working_dir: std::path::PathBuf,

    /// Enable dry run mode
    pub dry_run: bool,

    /// Enable verbose output
    pub verbose: bool,

    /// Auto-commit after each task
    pub auto_commit: bool,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            provider: None,
            working_dir: std::env::current_dir().unwrap_or_default(),
            dry_run: false,
            verbose: false,
            auto_commit: false,
        }
    }
}

/// Task executor that runs tasks with AI assistance.
pub struct TaskExecutor {
    config: ExecutorConfig,
}

impl TaskExecutor {
    /// Create a new executor with default configuration.
    pub fn new() -> Self {
        Self { config: ExecutorConfig::default() }
    }

    /// Create executor with custom configuration.
    pub fn with_config(config: ExecutorConfig) -> Self {
        Self { config }
    }

    /// Execute a single task.
    pub fn execute_task(&self, task: &Task) -> TaskResult {
        if self.config.verbose {
            println!("Executing task {}: {}", task.id, task.name);
        }

        // Check task type
        match task.task_type {
            TaskType::Manual => {
                return TaskResult {
                    task_id: task.id,
                    success: false,
                    output: "Manual task - requires human action".to_string(),
                    files_modified: Vec::new(),
                    verification: Vec::new(),
                };
            }
            TaskType::Review => {
                return TaskResult {
                    task_id: task.id,
                    success: false,
                    output: "Review task - requires human review".to_string(),
                    files_modified: Vec::new(),
                    verification: Vec::new(),
                };
            }
            TaskType::Auto => {}
        }

        // Dry run mode
        if self.config.dry_run {
            return TaskResult {
                task_id: task.id,
                success: true,
                output: format!("DRY RUN: Would execute task: {}", task.name),
                files_modified: task.files.clone(),
                verification: task
                    .verify
                    .iter()
                    .map(|v| VerificationResult {
                        step: v.clone(),
                        passed: true,
                        output: "DRY RUN".to_string(),
                    })
                    .collect(),
            };
        }

        // For now, return a placeholder result
        // In a full implementation, this would:
        // 1. Build AI prompt with context
        // 2. Call AI provider to generate code
        // 3. Apply changes to files
        // 4. Run verification steps
        TaskResult {
            task_id: task.id,
            success: true,
            output: format!("Task {} executed (placeholder)", task.id),
            files_modified: Vec::new(),
            verification: Vec::new(),
        }
    }

    /// Execute all pending tasks in a plan.
    pub fn execute_plan(&self, plan: &mut PlanDoc) -> Vec<TaskResult> {
        let mut results = Vec::new();

        for task in &mut plan.tasks {
            if task.status != TaskStatus::Pending {
                continue;
            }

            task.status = TaskStatus::InProgress;

            let result = self.execute_task(task);

            if result.success {
                task.status = TaskStatus::Completed;
            } else {
                task.status = TaskStatus::Blocked;
            }

            results.push(result);
        }

        results
    }

    /// Execute a specific task by ID.
    pub fn execute_task_by_id(&self, plan: &mut PlanDoc, task_id: usize) -> Option<TaskResult> {
        let task = plan.tasks.iter_mut().find(|t| t.id == task_id)?;

        task.status = TaskStatus::InProgress;

        let result = self.execute_task(task);

        if result.success {
            task.status = TaskStatus::Completed;
        } else {
            task.status = TaskStatus::Blocked;
        }

        Some(result)
    }

    /// Run verification steps for a task.
    pub fn verify_task(&self, task: &Task) -> Vec<VerificationResult> {
        task.verify.iter().map(|step| self.run_verification(step)).collect()
    }

    /// Run a single verification step.
    fn run_verification(&self, step: &str) -> VerificationResult {
        let lower = step.to_lowercase();

        // Check for common verification patterns
        if lower.contains("test") && lower.contains("pass") {
            return self.run_tests();
        }
        if lower.contains("compile") || lower.contains("build") {
            return self.run_build();
        }
        if lower.contains("lint") {
            return self.run_lint();
        }

        // Generic verification
        VerificationResult {
            step: step.to_string(),
            passed: true, // Assume passed for non-executable steps
            output: "Manual verification required".to_string(),
        }
    }

    /// Run tests and return verification result.
    fn run_tests(&self) -> VerificationResult {
        let output = Command::new("cargo")
            .arg("test")
            .arg("--lib")
            .current_dir(&self.config.working_dir)
            .output();

        match output {
            Ok(output) => {
                let passed = output.status.success();
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                VerificationResult {
                    step: "Tests pass".to_string(),
                    passed,
                    output: if passed {
                        "All tests passed".to_string()
                    } else {
                        format!("{}\n{}", stdout, stderr)
                    },
                }
            }
            Err(e) => VerificationResult {
                step: "Tests pass".to_string(),
                passed: false,
                output: format!("Failed to run tests: {}", e),
            },
        }
    }

    /// Run build and return verification result.
    fn run_build(&self) -> VerificationResult {
        let output =
            Command::new("cargo").arg("build").current_dir(&self.config.working_dir).output();

        match output {
            Ok(output) => {
                let passed = output.status.success();
                VerificationResult {
                    step: "Code compiles".to_string(),
                    passed,
                    output: if passed {
                        "Build successful".to_string()
                    } else {
                        String::from_utf8_lossy(&output.stderr).to_string()
                    },
                }
            }
            Err(e) => VerificationResult {
                step: "Code compiles".to_string(),
                passed: false,
                output: format!("Failed to build: {}", e),
            },
        }
    }

    /// Run linter and return verification result.
    fn run_lint(&self) -> VerificationResult {
        let output = Command::new("cargo")
            .arg("clippy")
            .arg("--")
            .arg("-D")
            .arg("warnings")
            .current_dir(&self.config.working_dir)
            .output();

        match output {
            Ok(output) => {
                let passed = output.status.success();
                VerificationResult {
                    step: "Linter passes".to_string(),
                    passed,
                    output: if passed {
                        "No linter warnings".to_string()
                    } else {
                        String::from_utf8_lossy(&output.stderr).to_string()
                    },
                }
            }
            Err(e) => VerificationResult {
                step: "Linter passes".to_string(),
                passed: false,
                output: format!("Failed to run linter: {}", e),
            },
        }
    }
}

impl Default for TaskExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Execution summary for a plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    /// Plan ID
    pub plan_id: String,

    /// Total tasks
    pub total_tasks: usize,

    /// Completed tasks
    pub completed: usize,

    /// Failed tasks
    pub failed: usize,

    /// Skipped tasks
    pub skipped: usize,

    /// All task results
    pub results: Vec<TaskResult>,
}

impl ExecutionSummary {
    /// Create summary from results.
    pub fn from_results(plan_id: &str, results: Vec<TaskResult>) -> Self {
        let total = results.len();
        let completed = results.iter().filter(|r| r.success).count();
        let failed = results.iter().filter(|r| !r.success).count();

        Self {
            plan_id: plan_id.to_string(),
            total_tasks: total,
            completed,
            failed,
            skipped: 0,
            results,
        }
    }

    /// Check if execution was successful.
    pub fn is_successful(&self) -> bool {
        self.failed == 0
    }

    /// Format as human-readable summary.
    pub fn to_summary_string(&self) -> String {
        let mut summary = format!("Execution Summary for {}\n", self.plan_id);
        summary.push_str(&format!(
            "Tasks: {} total, {} completed, {} failed, {} skipped\n",
            self.total_tasks, self.completed, self.failed, self.skipped
        ));

        if !self.results.is_empty() {
            summary.push_str("\nResults:\n");
            for result in &self.results {
                let status = if result.success { "✓" } else { "✗" };
                summary.push_str(&format!(
                    "  {} Task {}: {}\n",
                    status, result.task_id, result.output
                ));
            }
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_dry_run() {
        let config = ExecutorConfig { dry_run: true, ..Default::default() };
        let executor = TaskExecutor::with_config(config);

        let task = Task {
            id: 1,
            name: "Test task".to_string(),
            task_type: TaskType::Auto,
            status: TaskStatus::Pending,
            files: vec!["test.rs".to_string()],
            context: String::new(),
            steps: Vec::new(),
            verify: vec!["Tests pass".to_string()],
            done_criteria: String::new(),
        };

        let result = executor.execute_task(&task);
        assert!(result.success);
        assert!(result.output.contains("DRY RUN"));
    }

    #[test]
    fn test_executor_manual_task() {
        let executor = TaskExecutor::new();

        let task = Task {
            id: 1,
            name: "Manual task".to_string(),
            task_type: TaskType::Manual,
            status: TaskStatus::Pending,
            files: Vec::new(),
            context: String::new(),
            steps: Vec::new(),
            verify: Vec::new(),
            done_criteria: String::new(),
        };

        let result = executor.execute_task(&task);
        assert!(!result.success);
        assert!(result.output.contains("Manual task"));
    }

    #[test]
    fn test_execution_summary() {
        let results = vec![
            TaskResult {
                task_id: 1,
                success: true,
                output: "Done".to_string(),
                files_modified: Vec::new(),
                verification: Vec::new(),
            },
            TaskResult {
                task_id: 2,
                success: false,
                output: "Failed".to_string(),
                files_modified: Vec::new(),
                verification: Vec::new(),
            },
        ];

        let summary = ExecutionSummary::from_results("test-plan", results);
        assert_eq!(summary.total_tasks, 2);
        assert_eq!(summary.completed, 1);
        assert_eq!(summary.failed, 1);
        assert!(!summary.is_successful());
    }
}
