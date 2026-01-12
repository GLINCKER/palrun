//! Workflow system for AI-assisted project management.
//!
//! Provides GSD-style project context management that persists across sessions.
//!
//! ## Documents
//!
//! - `PROJECT.md` - Vision, requirements, constraints
//! - `ROADMAP.md` - Phases, milestones
//! - `STATE.md` - Current position, decisions, blockers
//! - `PLAN.md` - Current task plan
//! - `CODEBASE.md` - Auto-generated codebase analysis
//!
//! ## Task Execution
//!
//! - `PlanGenerator` - Creates plans from roadmap phases
//! - `TaskExecutor` - Executes tasks with AI assistance

mod analysis;
mod context;
mod documents;
mod executor;
mod planning;

pub use analysis::{analyze_codebase, CodebaseAnalysis};
pub use context::WorkflowContext;
pub use documents::{
    Decision, Phase, PhaseStatus, PlanDoc, ProjectDoc, RoadmapDoc, StateDoc, Task, TaskStatus,
    TaskType,
};
pub use executor::{ExecutionSummary, ExecutorConfig, TaskExecutor};
pub use planning::{PlanGenerator, TaskResult, VerificationResult};
