//! Input handling for the TUI.
//!
//! Processes keyboard events and updates application state.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::AppMode;
use crate::App;

/// Handle keyboard events.
pub fn handle_events(key: KeyEvent, app: &mut App) {
    // Handle different modes
    match &app.mode {
        AppMode::ExecutionResult => {
            handle_result_mode(key, app);
        }
        AppMode::Executing(_) => {
            // While executing, only allow Ctrl+C to cancel
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                app.dismiss_result();
            }
        }
        AppMode::Help => {
            handle_help_mode(key, app);
        }
        AppMode::History => {
            handle_history_mode(key, app);
        }
        AppMode::Analytics => {
            handle_analytics_mode(key, app);
        }
        AppMode::PassThrough => {
            handle_pass_through_mode(key, app);
        }
        AppMode::Palette => {
            handle_palette_mode(key, app);
        }
        AppMode::ContextMenu => {
            handle_context_menu_mode(key, app);
        }
        _ => {
            handle_normal_mode(key, app);
        }
    }
}

/// Handle input in help mode.
fn handle_help_mode(key: KeyEvent, app: &mut App) {
    match key.code {
        // Dismiss help
        KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Enter => {
            app.dismiss_help();
        }
        // Ctrl+C to quit completely
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
        }
        _ => {}
    }
}

/// Handle input in history mode.
fn handle_history_mode(key: KeyEvent, app: &mut App) {
    match key.code {
        // Dismiss history
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
            app.dismiss_history();
        }
        // Ctrl+H toggles history
        KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.dismiss_history();
        }
        // Ctrl+C to quit completely
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
        }
        _ => {}
    }
}

/// Handle input in analytics mode.
fn handle_analytics_mode(key: KeyEvent, app: &mut App) {
    match key.code {
        // Dismiss analytics
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
            app.dismiss_analytics();
        }
        // Ctrl+G toggles analytics
        KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.dismiss_analytics();
        }
        // Ctrl+C to quit completely
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
        }
        _ => {}
    }
}

/// Handle input in execution result mode.
fn handle_result_mode(key: KeyEvent, app: &mut App) {
    match key.code {
        // Scrolling output
        KeyCode::Up | KeyCode::Char('k') => {
            app.scroll_output_up();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.scroll_output_down();
        }
        KeyCode::PageUp => {
            app.scroll_output_page_up();
        }
        KeyCode::PageDown => {
            app.scroll_output_page_down();
        }
        KeyCode::Home => {
            app.scroll_output_top();
        }
        KeyCode::End => {
            app.scroll_output_bottom();
        }
        // Dismiss result and return to normal mode
        KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') => {
            app.dismiss_result();
        }
        // Ctrl+C to quit completely
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
        }
        // 'q' to quit from result screen
        KeyCode::Char('q') => {
            app.quit();
        }
        // 'r' to re-run the same command
        KeyCode::Char('r') => {
            app.dismiss_result();
            app.execute_selected_command();
        }
        _ => {}
    }
}

/// Handle input in normal command palette mode.
fn handle_normal_mode(key: KeyEvent, app: &mut App) {
    match key.code {
        // Quit
        KeyCode::Esc => {
            if app.multi_select_mode {
                // Exit multi-select mode first
                app.toggle_multi_select();
            } else {
                app.quit();
            }
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
        KeyCode::Char('q') if app.input.is_empty() && !app.multi_select_mode => app.quit(),

        // Multi-select mode toggle (Ctrl+Space)
        KeyCode::Char(' ') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.toggle_multi_select();
        }

        // Select/deselect in multi-select mode (Space)
        KeyCode::Char(' ') if app.multi_select_mode => {
            app.toggle_current_selection();
            app.select_next(); // Move to next for quick selection
        }

        // Select all (Ctrl+A in multi-select mode)
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::CONTROL) && app.multi_select_mode => {
            app.select_all();
        }

        // Run in background (Ctrl+B)
        KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.get_selected_command().is_some() {
                app.execute_in_background();
            }
        }

        // Toggle favorite (Ctrl+S)
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.toggle_favorite();
        }

        // Show history (Ctrl+H)
        KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.show_history();
        }

        // Show analytics (Ctrl+G for "graph")
        KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.show_analytics();
        }

        // Navigation
        KeyCode::Up | KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.select_previous();
        }
        KeyCode::Down | KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.select_next();
        }
        KeyCode::Up => app.select_previous(),
        KeyCode::Down => app.select_next(),

        // Selection - Execute command and show result (stay in TUI)
        KeyCode::Enter => {
            if app.get_selected_command().is_some() {
                if app.multi_select_mode && !app.selected_commands.is_empty() {
                    // Execute selected commands in parallel
                    app.execute_parallel_commands();
                } else {
                    app.execute_selected_command();
                }
            } else if !app.input.is_empty() {
                // No command matched - offer to run as shell command (pass-through)
                app.enter_pass_through();
            }
        }

        // Command palette (Ctrl+P)
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.show_palette();
        }

        // Context menu for selected command (. when input is empty)
        KeyCode::Char('.') if app.input.is_empty() => {
            app.show_context_menu();
        }

        // Page navigation
        KeyCode::PageUp => {
            for _ in 0..10 {
                app.select_previous();
            }
        }
        KeyCode::PageDown => {
            for _ in 0..10 {
                app.select_next();
            }
        }
        KeyCode::Home if key.modifiers.contains(KeyModifiers::CONTROL) => app.select_first(),
        KeyCode::End if key.modifiers.contains(KeyModifiers::CONTROL) => app.select_last(),

        // Clear input (must be before general Char handler)
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => app.clear_input(),

        // Show help screen (must be before general Char handler)
        KeyCode::Char('?') => {
            app.show_help();
        }

        // Input editing
        KeyCode::Char(c) => app.enter_char(c),
        KeyCode::Backspace => app.delete_char(),
        KeyCode::Delete => app.delete_char_forward(),

        // Cursor movement
        KeyCode::Left => app.move_cursor_left(),
        KeyCode::Right => app.move_cursor_right(),
        KeyCode::Home => app.move_cursor_start(),
        KeyCode::End => app.move_cursor_end(),

        // Tab completion (future)
        KeyCode::Tab => {
            // TODO: Implement tab completion
        }

        _ => {}
    }
}

/// Handle input in pass-through mode (confirming shell command execution).
fn handle_pass_through_mode(key: KeyEvent, app: &mut App) {
    match key.code {
        // Confirm and execute the shell command
        KeyCode::Enter | KeyCode::Char('y') => {
            app.execute_pass_through();
        }
        // Cancel and return to normal mode
        KeyCode::Esc | KeyCode::Char('n') => {
            app.cancel_pass_through();
        }
        // Ctrl+C to quit
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
        }
        _ => {}
    }
}

/// Handle input in command palette mode.
fn handle_palette_mode(key: KeyEvent, app: &mut App) {
    match key.code {
        // Dismiss palette
        KeyCode::Esc => {
            app.dismiss_palette();
        }
        // Navigate
        KeyCode::Up | KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.palette_selected = app.palette_selected.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.palette_selected = (app.palette_selected + 1).min(5); // 6 palette items
        }
        KeyCode::Up => {
            app.palette_selected = app.palette_selected.saturating_sub(1);
        }
        KeyCode::Down => {
            app.palette_selected = (app.palette_selected + 1).min(5);
        }
        // Select palette item
        KeyCode::Enter => {
            match app.palette_selected {
                0 => app.show_history(),
                1 => app.show_analytics(),
                2 => { /* Settings - TODO */ }
                3 => { /* Plugins - TODO */ }
                4 => { /* Environment - TODO */ }
                5 => app.show_help(),
                _ => {}
            }
            app.dismiss_palette();
        }
        // Direct shortcuts
        KeyCode::Char('h') => {
            app.show_history();
            app.dismiss_palette();
        }
        KeyCode::Char('a') => {
            app.show_analytics();
            app.dismiss_palette();
        }
        KeyCode::Char('?') => {
            app.show_help();
            app.dismiss_palette();
        }
        // Ctrl+C to quit
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
        }
        // Ctrl+P to dismiss (toggle)
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.dismiss_palette();
        }
        _ => {}
    }
}

/// Handle input in context menu mode.
fn handle_context_menu_mode(key: KeyEvent, app: &mut App) {
    match key.code {
        // Dismiss menu
        KeyCode::Esc | KeyCode::Char('.') => {
            app.dismiss_context_menu();
        }
        // Navigate
        KeyCode::Up | KeyCode::Char('k') => {
            app.context_menu_selected = app.context_menu_selected.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.context_menu_selected = (app.context_menu_selected + 1).min(4); // 5 menu items
        }
        // Select menu item
        KeyCode::Enter => {
            match app.context_menu_selected {
                0 => app.execute_selected_command(),     // Run
                1 => app.execute_in_background(),        // Run in background
                2 => app.toggle_favorite(),              // Toggle favorite
                3 => { /* Copy command - TODO */ }       // Copy
                4 => { /* Edit - TODO */ }               // Edit
                _ => {}
            }
            app.dismiss_context_menu();
        }
        // Direct shortcuts
        KeyCode::Char('r') => {
            app.execute_selected_command();
            app.dismiss_context_menu();
        }
        KeyCode::Char('b') => {
            app.execute_in_background();
            app.dismiss_context_menu();
        }
        KeyCode::Char('f') | KeyCode::Char('s') => {
            app.toggle_favorite();
            app.dismiss_context_menu();
        }
        // Ctrl+C to quit
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn test_quit_on_escape() {
        let mut app = App::new_test();
        handle_events(create_key_event(KeyCode::Esc, KeyModifiers::NONE), &mut app);
        assert!(app.should_quit);
    }

    #[test]
    fn test_quit_on_ctrl_c() {
        let mut app = App::new_test();
        handle_events(create_key_event(KeyCode::Char('c'), KeyModifiers::CONTROL), &mut app);
        assert!(app.should_quit);
    }

    #[test]
    fn test_char_input() {
        let mut app = App::new_test();
        handle_events(create_key_event(KeyCode::Char('a'), KeyModifiers::NONE), &mut app);
        assert_eq!(app.input, "a");

        handle_events(create_key_event(KeyCode::Char('b'), KeyModifiers::NONE), &mut app);
        assert_eq!(app.input, "ab");
    }

    #[test]
    fn test_backspace() {
        let mut app = App::new_test();
        app.input = "test".to_string();
        app.cursor_position = 4;

        handle_events(create_key_event(KeyCode::Backspace, KeyModifiers::NONE), &mut app);
        assert_eq!(app.input, "tes");
    }

    #[test]
    fn test_navigation() {
        let mut app = App::new_test();
        app.filtered_commands = vec![0, 1, 2, 3, 4];

        handle_events(create_key_event(KeyCode::Down, KeyModifiers::NONE), &mut app);
        assert_eq!(app.selected, 1);

        handle_events(create_key_event(KeyCode::Up, KeyModifiers::NONE), &mut app);
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_clear_input() {
        let mut app = App::new_test();
        app.input = "test".to_string();
        app.cursor_position = 4;

        handle_events(create_key_event(KeyCode::Char('u'), KeyModifiers::CONTROL), &mut app);
        assert!(app.input.is_empty());
        assert_eq!(app.cursor_position, 0);
    }

    #[test]
    fn test_show_help() {
        use crate::app::AppMode;
        let mut app = App::new_test();
        assert!(!matches!(app.mode, AppMode::Help));

        handle_events(create_key_event(KeyCode::Char('?'), KeyModifiers::NONE), &mut app);
        assert!(matches!(app.mode, AppMode::Help));
    }

    #[test]
    fn test_dismiss_help() {
        use crate::app::AppMode;
        let mut app = App::new_test();
        app.show_help();
        assert!(matches!(app.mode, AppMode::Help));

        // Esc should dismiss help
        handle_events(create_key_event(KeyCode::Esc, KeyModifiers::NONE), &mut app);
        assert!(matches!(app.mode, AppMode::Normal));
    }

    #[test]
    fn test_dismiss_help_with_question_mark() {
        use crate::app::AppMode;
        let mut app = App::new_test();
        app.show_help();
        assert!(matches!(app.mode, AppMode::Help));

        // ? should dismiss help
        handle_events(create_key_event(KeyCode::Char('?'), KeyModifiers::NONE), &mut app);
        assert!(matches!(app.mode, AppMode::Normal));
    }
}
