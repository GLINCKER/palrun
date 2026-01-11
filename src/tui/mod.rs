//! Terminal User Interface module.
//!
//! This module handles the TUI rendering and input handling using ratatui.

mod app;
mod input;
mod theme;
mod ui;

pub use app::run_tui;
pub use input::handle_events;
pub use theme::{parse_hex_color, Theme};
pub use ui::draw;
