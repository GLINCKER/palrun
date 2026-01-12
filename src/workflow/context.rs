//! Workflow context management.
//!
//! Loads and manages project context documents for AI-assisted workflows.

use std::path::{Path, PathBuf};

use super::analysis::CodebaseAnalysis;
use super::documents::{PlanDoc, ProjectDoc, RoadmapDoc, StateDoc};

/// Workflow context for AI requests.
///
/// Contains all project context documents and provides
/// methods to convert them to AI prompt context.
#[derive(Debug, Clone)]
pub struct WorkflowContext {
    /// Root directory
    pub root: PathBuf,

    /// Planning directory (.palrun)
    pub planning_dir: PathBuf,

    /// Project document
    pub project: Option<ProjectDoc>,

    /// Roadmap document
    pub roadmap: Option<RoadmapDoc>,

    /// State document
    pub state: Option<StateDoc>,

    /// Current plan
    pub plan: Option<PlanDoc>,

    /// Codebase analysis
    pub codebase: Option<CodebaseAnalysis>,
}

impl WorkflowContext {
    /// Create a new workflow context for a directory.
    pub fn new(root: PathBuf) -> Self {
        let planning_dir = root.join(".palrun");
        Self {
            root,
            planning_dir,
            project: None,
            roadmap: None,
            state: None,
            plan: None,
            codebase: None,
        }
    }

    /// Load context from a directory.
    pub fn load(root: &Path) -> anyhow::Result<Self> {
        let mut ctx = Self::new(root.to_path_buf());
        ctx.reload()?;
        Ok(ctx)
    }

    /// Reload all context documents.
    pub fn reload(&mut self) -> anyhow::Result<()> {
        // Load PROJECT.md
        let project_path = self.planning_dir.join("PROJECT.md");
        if project_path.exists() {
            self.project = ProjectDoc::load(&project_path).ok();
        }

        // Load ROADMAP.md
        let roadmap_path = self.planning_dir.join("ROADMAP.md");
        if roadmap_path.exists() {
            self.roadmap = RoadmapDoc::load(&roadmap_path).ok();
        }

        // Load STATE.md
        let state_path = self.planning_dir.join("STATE.md");
        if state_path.exists() {
            self.state = StateDoc::load(&state_path).ok();
        }

        // Load PLAN.md
        let plan_path = self.planning_dir.join("PLAN.md");
        if plan_path.exists() {
            self.plan = PlanDoc::load(&plan_path).ok();
        }

        // Load CODEBASE.md analysis
        let codebase_path = self.planning_dir.join("CODEBASE.md");
        if codebase_path.exists() {
            self.codebase = CodebaseAnalysis::load(&codebase_path).ok();
        }

        Ok(())
    }

    /// Initialize a new workflow in the directory.
    pub fn init(&self, project_name: &str) -> anyhow::Result<()> {
        // Create .palrun directory
        std::fs::create_dir_all(&self.planning_dir)?;

        // Create PROJECT.md
        let project_path = self.planning_dir.join("PROJECT.md");
        if !project_path.exists() {
            std::fs::write(&project_path, ProjectDoc::template(project_name))?;
        }

        // Create STATE.md
        let state_path = self.planning_dir.join("STATE.md");
        if !state_path.exists() {
            std::fs::write(&state_path, StateDoc::template())?;
        }

        Ok(())
    }

    /// Check if workflow is initialized.
    pub fn is_initialized(&self) -> bool {
        self.planning_dir.exists() && self.planning_dir.join("PROJECT.md").exists()
    }

    /// Get the project name.
    pub fn project_name(&self) -> &str {
        self.project
            .as_ref()
            .map(|p| p.name.as_str())
            .or_else(|| self.root.file_name().and_then(|n| n.to_str()))
            .unwrap_or("unknown")
    }

    /// Get current phase.
    pub fn current_phase(&self) -> Option<usize> {
        self.state.as_ref().map(|s| s.current_phase).or_else(|| {
            self.roadmap.as_ref().map(|r| r.current_phase + 1) // 1-based
        })
    }

    /// Get current task.
    pub fn current_task(&self) -> Option<&super::documents::Task> {
        self.plan.as_ref().and_then(|p| p.next_task())
    }

    /// Convert to AI prompt context with token limit.
    ///
    /// Prioritizes: current plan > state > project > roadmap > codebase
    pub fn to_prompt_context(&self, max_tokens: usize) -> String {
        // Rough estimate: 4 chars per token
        let max_chars = max_tokens * 4;
        let mut ctx = String::new();
        let mut remaining = max_chars;

        // 1. Current plan (highest priority)
        if let Some(plan) = &self.plan {
            let plan_ctx = plan.to_context(remaining / 2);
            if plan_ctx.len() < remaining {
                ctx.push_str(&plan_ctx);
                ctx.push('\n');
                remaining = remaining.saturating_sub(plan_ctx.len());
            }
        }

        // 2. State
        if let Some(state) = &self.state {
            let state_ctx = state.to_context(remaining / 3);
            if state_ctx.len() < remaining {
                ctx.push_str(&state_ctx);
                ctx.push('\n');
                remaining = remaining.saturating_sub(state_ctx.len());
            }
        }

        // 3. Project
        if let Some(project) = &self.project {
            let project_ctx = project.to_context(remaining / 2);
            if project_ctx.len() < remaining {
                ctx.push_str(&project_ctx);
                ctx.push('\n');
                remaining = remaining.saturating_sub(project_ctx.len());
            }
        }

        // 4. Roadmap
        if let Some(roadmap) = &self.roadmap {
            let roadmap_ctx = roadmap.to_context(remaining / 2);
            if roadmap_ctx.len() < remaining {
                ctx.push_str(&roadmap_ctx);
                ctx.push('\n');
                remaining = remaining.saturating_sub(roadmap_ctx.len());
            }
        }

        // 5. Codebase analysis (lowest priority, often large)
        if let Some(codebase) = &self.codebase {
            let codebase_ctx = codebase.to_context(remaining);
            if codebase_ctx.len() < remaining {
                ctx.push_str(&codebase_ctx);
            }
        }

        ctx
    }

    /// Update state after completing a task.
    pub fn complete_task(&mut self, task_id: usize) -> anyhow::Result<()> {
        if let Some(ref mut state) = self.state {
            state.current_task = task_id;
            state.status = "In Progress".to_string();
            state.recent_changes.insert(
                0,
                format!("{}: Completed task {}", chrono::Utc::now().format("%Y-%m-%d"), task_id),
            );

            let state_path = self.planning_dir.join("STATE.md");
            state.save(&state_path)?;
        }
        Ok(())
    }

    /// Add a blocker.
    pub fn add_blocker(&mut self, blocker: &str) -> anyhow::Result<()> {
        if let Some(ref mut state) = self.state {
            state.blockers.push(blocker.to_string());
            state.status = "Blocked".to_string();

            let state_path = self.planning_dir.join("STATE.md");
            state.save(&state_path)?;
        }
        Ok(())
    }

    /// Get a summary of the current workflow status.
    pub fn summary(&self) -> String {
        let mut summary = String::new();

        summary.push_str(&format!("Project: {}\n", self.project_name()));

        if let Some(phase) = self.current_phase() {
            let total = self.roadmap.as_ref().map(|r| r.phases.len()).unwrap_or(1);
            summary.push_str(&format!("Phase: {} of {}\n", phase, total));
        }

        if let Some(task) = self.current_task() {
            summary.push_str(&format!("Task: {} - {}\n", task.id, task.name));
        }

        if let Some(state) = &self.state {
            summary.push_str(&format!("Status: {}\n", state.status));
            if !state.blockers.is_empty() {
                summary.push_str(&format!("Blockers: {}\n", state.blockers.len()));
            }
        }

        summary
    }
}

impl Default for WorkflowContext {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_context_new() {
        let ctx = WorkflowContext::new(PathBuf::from("/tmp/test"));
        assert_eq!(ctx.planning_dir, PathBuf::from("/tmp/test/.palrun"));
        assert!(!ctx.is_initialized());
    }

    #[test]
    fn test_workflow_context_default() {
        let ctx = WorkflowContext::default();
        assert_eq!(ctx.root, PathBuf::from("."));
    }

    #[test]
    fn test_project_name_fallback() {
        let ctx = WorkflowContext::new(PathBuf::from("/tmp/my-project"));
        assert_eq!(ctx.project_name(), "my-project");
    }

    #[test]
    fn test_to_prompt_context_empty() {
        let ctx = WorkflowContext::default();
        let prompt = ctx.to_prompt_context(1000);
        assert!(prompt.is_empty());
    }
}
