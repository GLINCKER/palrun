//! Universal slash command system for AI IDEs.
//!
//! This module provides the infrastructure to generate and install
//! Palrun commands as slash commands in any AI IDE (Claude Code,
//! Cursor, Windsurf, Continue.dev, Aider, etc.).
//!
//! ## Architecture
//!
//! - `CommandTarget` trait: Abstracts over IDE-specific command formats
//! - `SlashCommandRegistry`: Manages available commands and targets
//! - `PalrunCommand`: Represents a command that can be exposed to IDEs
//!
//! ## Usage
//!
//! ```bash
//! # Install commands to all detected IDEs
//! palrun commands install --all
//!
//! # Install to specific IDE
//! palrun commands install --target claude
//!
//! # List available targets
//! palrun commands list
//! ```

mod registry;
mod target;
pub mod targets;

pub use registry::{SlashCommandRegistry, PALRUN_COMMANDS};
pub use target::{CommandArg, CommandCategory, CommandTarget, PalrunCommand};
pub use targets::default_registry;
