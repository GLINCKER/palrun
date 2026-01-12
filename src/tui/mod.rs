//! Terminal User Interface module.
//!
//! This module handles the TUI rendering and input handling using ratatui.

mod app;
mod input;
mod theme;
mod ui;

#[cfg(feature = "ai")]
pub use app::run_ai_chat_inline;
pub use app::run_tui;
pub use input::handle_events;
pub use theme::{parse_hex_color, Theme};
pub use ui::draw;
