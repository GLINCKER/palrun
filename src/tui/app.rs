//! TUI application runner.
//!
//! Handles the main event loop and terminal setup/teardown.

use std::io::{self, stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use super::{draw, handle_events};
use crate::App;

/// Run the TUI application.
///
/// This is the main entry point for the interactive command palette.
pub fn run_tui(mut app: App) -> Result<()> {
    // Setup terminal
    setup_terminal()?;

    // Create terminal backend
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    // Initialize the app (scan for commands, etc.)
    if let Err(e) = app.initialize() {
        restore_terminal()?;
        return Err(e);
    }

    // Run the main loop
    let result = run_main_loop(&mut terminal, &mut app);

    // Restore terminal
    restore_terminal()?;

    // Execute the selected command if user chose one (Enter, not Esc)
    if app.command_selected {
        if let Some(cmd) = app.get_selected_command() {
            println!("Executing: {}", cmd.command);
            let executor = crate::core::Executor::new();
            let _ = executor.execute(cmd);
        }
    }

    result
}

/// Setup the terminal for TUI mode.
fn setup_terminal() -> Result<()> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    // Setup panic hook to restore terminal on panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal();
        original_hook(panic_info);
    }));

    Ok(())
}

/// Restore the terminal to normal mode.
fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}

/// Main event loop.
fn run_main_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    let tick_rate = Duration::from_millis(100);

    loop {
        // Draw the UI
        terminal.draw(|frame| draw(frame, app))?;

        // Handle events
        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                handle_events(key, app);
            }
        }

        // Check if we should quit
        if app.should_quit {
            break;
        }

        // Periodic tick
        app.tick();
    }

    Ok(())
}

/// Run the TUI and return the selected command (if any).
#[allow(dead_code)]
pub fn run_tui_and_get_command(mut app: App) -> Result<Option<String>> {
    // Setup terminal
    setup_terminal()?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    if let Err(e) = app.initialize() {
        restore_terminal()?;
        return Err(e);
    }

    let result = run_main_loop(&mut terminal, &mut app);

    restore_terminal()?;

    result?;

    // Return the selected command
    Ok(app.get_selected_command().map(|c| c.command.clone()))
}
