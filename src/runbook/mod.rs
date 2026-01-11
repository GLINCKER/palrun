//! Runbook system for executable team workflows.
//!
//! Runbooks are YAML files that define step-by-step workflows with
//! variables, conditions, and confirmations.

mod parser;
mod runner;
mod schema;

pub use parser::{discover_runbooks, parse_runbook, parse_runbook_str};
pub use runner::RunbookRunner;
pub use schema::{Runbook, Step, VarType, Variable};
