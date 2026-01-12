//! IDE-specific command target implementations.
//!
//! Each supported IDE has its own implementation of the `CommandTarget` trait.

mod aider;
mod claude;
mod continue_dev;
mod cursor;
mod windsurf;

pub use aider::AiderTarget;
pub use claude::ClaudeCodeTarget;
pub use continue_dev::ContinueDevTarget;
pub use cursor::CursorTarget;
pub use windsurf::WindsurfTarget;

use super::SlashCommandRegistry;

/// Create a registry with all built-in targets.
pub fn default_registry() -> SlashCommandRegistry {
    let mut registry = SlashCommandRegistry::new();
    registry.register(Box::new(ClaudeCodeTarget));
    registry.register(Box::new(CursorTarget));
    registry.register(Box::new(WindsurfTarget));
    registry.register(Box::new(ContinueDevTarget));
    registry.register(Box::new(AiderTarget));
    registry
}
