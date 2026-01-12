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
        AppMode::TrustConfirmation => {
            handle_trust_confirmation_mode(key, app);
        }
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
        AppMode::Workflow => {
            handle_workflow_mode(key, app);
        }
        #[cfg(feature = "ai")]
        AppMode::AiChat => {
            handle_ai_chat_mode(key, app);
        }
        #[cfg(feature = "ai")]
        AppMode::AiSetup => {
            handle_ai_setup_mode(key, app);
        }
        _ => {
            handle_normal_mode(key, app);
        }
    }
}

/// Handle input in trust confirmation mode.
fn handle_trust_confirmation_mode(key: KeyEvent, app: &mut App) {
    match key.code {
        // Navigate between options
        KeyCode::Left | KeyCode::Up | KeyCode::Char('h' | 'k') => {
            app.trust_selected = 0;
        }
        KeyCode::Right | KeyCode::Down | KeyCode::Char('l' | 'j') => {
            app.trust_selected = 1;
        }
        KeyCode::Tab => {
            app.trust_selected = if app.trust_selected == 0 { 1 } else { 0 };
        }

        // Confirm selection
        KeyCode::Enter => {
            if app.trust_selected == 0 {
                // User trusts the directory
                if let Err(e) = app.trust_store.trust_directory(&app.cwd) {
                    app.status_message = Some(format!("Failed to save trust: {}", e));
                }
                app.mode = AppMode::Normal;
            } else {
                // User declined - exit
                app.quit();
            }
        }

        // Quick shortcuts
        KeyCode::Char('y' | 'Y') => {
            // Trust and proceed
            if let Err(e) = app.trust_store.trust_directory(&app.cwd) {
                app.status_message = Some(format!("Failed to save trust: {}", e));
            }
            app.mode = AppMode::Normal;
        }
        KeyCode::Char('n' | 'N') | KeyCode::Esc => {
            // Decline and exit
            app.quit();
        }

        // Ctrl+C to quit
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
        }

        _ => {}
    }
}

/// Handle input in help mode.
fn handle_help_mode(key: KeyEvent, app: &mut App) {
    match key.code {
        // Dismiss help
        KeyCode::Esc | KeyCode::Char('?' | 'q') | KeyCode::Enter => {
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
        KeyCode::Char('a')
            if key.modifiers.contains(KeyModifiers::CONTROL) && app.multi_select_mode =>
        {
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

        // Navigation - use directory navigation when browsing directories
        KeyCode::Up | KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.is_dir_browsing() {
                app.select_dir_previous();
            } else {
                app.select_previous();
            }
        }
        KeyCode::Down | KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.is_dir_browsing() {
                app.select_dir_next();
            } else {
                app.select_next();
            }
        }
        KeyCode::Up => {
            if app.is_slash_browsing() {
                app.select_slash_previous();
            } else if app.is_dir_browsing() {
                app.select_dir_previous();
            } else {
                app.select_previous();
            }
        }
        KeyCode::Down => {
            if app.is_slash_browsing() {
                app.select_slash_next();
            } else if app.is_dir_browsing() {
                app.select_dir_next();
            } else {
                app.select_next();
            }
        }

        // Selection - Execute command and show result (stay in TUI)
        KeyCode::Enter => {
            // Check for slash commands first
            if app.try_slash_command() {
                // Slash command was handled
            } else if app.is_dir_browsing() {
                // Execute selected directory entry
                app.execute_dir_selection();
            } else if app.get_selected_command().is_some() {
                if app.multi_select_mode && !app.selected_commands.is_empty() {
                    // Execute selected commands in parallel
                    app.execute_parallel_commands();
                } else {
                    app.execute_selected_command();
                }
            } else if !app.input.is_empty() {
                // No command matched - try auto-execute safe shell commands first
                if !app.try_auto_shell_command() {
                    // Not a safe command - ask for confirmation
                    app.enter_pass_through();
                }
            }
        }

        // Command palette (Ctrl+P)
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.show_palette();
        }

        // Toggle to AI chat mode (Ctrl+T)
        #[cfg(feature = "ai")]
        KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.show_ai_chat();
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

        // Tab completion for directory browsing
        KeyCode::Tab => {
            if app.is_dir_browsing() {
                app.complete_dir_selection();
            }
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
                0 => app.execute_selected_command(), // Run
                1 => app.execute_in_background(),    // Run in background
                2 => app.toggle_favorite(),          // Toggle favorite
                3 => { /* Copy command - TODO */ }   // Copy
                4 => { /* Edit - TODO */ }           // Edit
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
        KeyCode::Char('f' | 's') => {
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

/// Handle input in workflow mode.
fn handle_workflow_mode(key: KeyEvent, app: &mut App) {
    match key.code {
        // Dismiss workflow
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
            app.dismiss_workflow();
        }
        // Ctrl+C to quit completely
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
        }
        // Reload workflow context
        KeyCode::Char('r') => {
            app.load_workflow_context();
            app.set_status("Workflow context reloaded");
        }
        _ => {}
    }
}

/// Handle input in AI chat mode.
#[cfg(feature = "ai")]
fn handle_ai_chat_mode(key: KeyEvent, app: &mut App) {
    match key.code {
        // Toggle back to command palette (Ctrl+T)
        KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.dismiss_ai_chat();
        }
        // Dismiss AI chat
        KeyCode::Esc => {
            app.dismiss_ai_chat();
        }
        // Ctrl+C to quit completely
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
        }
        // Scroll up through chat history
        KeyCode::Up | KeyCode::PageUp => {
            app.ai_chat_scroll_up();
        }
        // Scroll down through chat history
        KeyCode::Down | KeyCode::PageDown => {
            app.ai_chat_scroll_down();
        }
        // Go to bottom (latest)
        KeyCode::End => {
            app.ai_chat_scroll_to_bottom();
        }
        // Send message
        KeyCode::Enter => {
            if !app.ai_chat_input.is_empty() && !app.ai_thinking {
                let input = std::mem::take(&mut app.ai_chat_input);

                // Check for slash commands first
                if input.starts_with('/') {
                    handle_ai_slash_command(&input, app);
                    return;
                }

                // Check if we have an AI provider available
                match &app.ai_status {
                    Some(status) if status.contains("Ollama") => {
                        // Build context-aware system prompt
                        let context = build_ai_context(app);
                        let system_prompt = context.build_system_prompt();

                        // Clone history before adding new message
                        let history: Vec<(String, String)> = app.ai_chat_history.clone();

                        // Show user's message immediately and auto-scroll to bottom
                        app.ai_chat_history.push((input.clone(), String::new()));
                        app.ai_chat_scroll_to_bottom();
                        app.ai_thinking = true;
                        app.set_status("Thinking...");

                        // Create runtime for async call
                        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build();

                        match rt {
                            Ok(runtime) => {
                                let result = runtime.block_on(async {
                                    call_ollama(&input, &system_prompt, &history).await
                                });

                                app.ai_thinking = false;

                                // Update the last entry with the response
                                if let Some(last) = app.ai_chat_history.last_mut() {
                                    match result {
                                        Ok(response) => {
                                            last.1 = response;
                                            app.set_status("AI response received");
                                        }
                                        Err(e) => {
                                            last.1 = format!("Error: {}", e);
                                            app.set_status("Ollama error - is it running?");
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                app.ai_thinking = false;
                                if let Some(last) = app.ai_chat_history.last_mut() {
                                    last.1 = "Failed to create async runtime".to_string();
                                }
                            }
                        }
                    }
                    Some(status) => {
                        // Other provider (Claude, OpenAI, etc.) - placeholder
                        app.ai_chat_history.push((
                            input.clone(),
                            format!("Using {} (API call not yet implemented)", status),
                        ));
                        app.set_status("API providers coming soon");
                    }
                    None => {
                        // No AI provider - show setup instructions
                        app.ai_chat_history.push((
                            input,
                            "No AI provider available.\n\nSetup options:\n\
                            • Ollama (local): Install from ollama.ai, run 'ollama run llama3.2'\n\
                            • Claude: Set ANTHROPIC_API_KEY environment variable\n\
                            • OpenAI: Set OPENAI_API_KEY environment variable\n\
                            • Grok: Set XAI_API_KEY environment variable"
                                .to_string(),
                        ));
                        app.set_status("No AI provider configured");
                    }
                }
            }
        }
        // Input editing
        KeyCode::Char(c) => {
            app.ai_chat_input.push(c);
        }
        KeyCode::Backspace => {
            app.ai_chat_input.pop();
        }
        _ => {}
    }
}

/// Handle AI chat slash commands.
#[cfg(feature = "ai")]
fn handle_ai_slash_command(cmd: &str, app: &mut App) {
    let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
    let command = parts[0].to_lowercase();
    let _args = parts.get(1).map(|s| s.trim());

    match command.as_str() {
        "/clear" => {
            app.ai_chat_history.clear();
            app.ai_chat_scroll = 0;
            app.set_status("Chat history cleared");
        }
        "/model" | "/models" => {
            app.show_ai_setup();
        }
        "/context" => {
            // Show current context in chat
            let context = build_ai_context(app);
            let git_info = if let Some(ref git) = app.git_info {
                format!("Branch: {}", git.branch.as_deref().unwrap_or("detached"))
            } else {
                "Not a git repo".to_string()
            };
            let context_info = format!(
                "**Current Context:**\n\
                 - Directory: {}\n\
                 - Project: {}\n\
                 - Commands: {} discovered\n\
                 - Git: {}",
                app.cwd.display(),
                context.project_type,
                app.registry.len(),
                git_info
            );
            app.ai_chat_history.push(("/context".to_string(), context_info));
        }
        "/help" => {
            let help_text = "**AI Chat Commands:**\n\
                 - `/clear` - Clear chat history\n\
                 - `/model` - Manage AI models\n\
                 - `/context` - Show current project context\n\
                 - `/help` - Show this help\n\
                 - `Ctrl+T` - Switch to Commands mode\n\
                 - `Esc` - Exit AI chat";
            app.ai_chat_history.push(("/help".to_string(), help_text.to_string()));
        }
        _ => {
            // Unknown command
            app.ai_chat_history.push((
                cmd.to_string(),
                format!("Unknown command: `{}`\nType `/help` for available commands.", command),
            ));
        }
    }
}

/// Handle input in AI setup mode (model management).
#[cfg(feature = "ai")]
fn handle_ai_setup_mode(key: KeyEvent, app: &mut App) {
    match key.code {
        // Dismiss AI setup or cancel pending delete
        KeyCode::Esc => {
            if app.ai_delete_pending.is_some() {
                app.cancel_delete_ai_model();
            } else {
                app.dismiss_ai_setup();
            }
        }
        // Ctrl+C to quit completely
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
        }
        // Navigate model list (also cancels pending delete)
        KeyCode::Up | KeyCode::Char('k') if app.ai_model_input.is_empty() => {
            app.ai_delete_pending = None; // Cancel pending delete on navigation
            if app.ai_model_selected > 0 {
                app.ai_model_selected -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') if app.ai_model_input.is_empty() => {
            app.ai_delete_pending = None; // Cancel pending delete on navigation
            if !app.ai_models.is_empty() && app.ai_model_selected < app.ai_models.len() - 1 {
                app.ai_model_selected += 1;
            }
        }
        // Use selected model (Enter)
        KeyCode::Enter => {
            app.ai_delete_pending = None; // Cancel pending delete
            if !app.ai_model_input.is_empty() {
                // Pull the entered model name
                let model = app.ai_model_input.clone();
                app.pull_ai_model(&model);
            } else if !app.ai_models.is_empty() {
                // Use selected model
                app.use_selected_model();
            }
        }
        // Refresh model list
        KeyCode::Char('r')
            if !key.modifiers.contains(KeyModifiers::CONTROL) && app.ai_model_input.is_empty() =>
        {
            app.ai_delete_pending = None;
            app.refresh_ai_models();
        }
        // Delete selected model (requires confirmation)
        KeyCode::Char('d') if app.ai_model_input.is_empty() => {
            if app.ai_delete_pending.is_some() {
                // Second press - confirm delete
                app.confirm_delete_ai_model();
            } else {
                // First press - request confirmation
                app.request_delete_ai_model();
            }
        }
        // Pull/download model (type model name)
        KeyCode::Char(c) => {
            app.ai_delete_pending = None; // Cancel pending delete when typing
            app.ai_model_input.push(c);
        }
        KeyCode::Backspace => {
            app.ai_model_input.pop();
        }
        _ => {}
    }
}

/// Build AI context from app state.
#[cfg(feature = "ai")]
fn build_ai_context(app: &App) -> crate::ai::ProjectContext {
    use crate::ai::ProjectContext;

    let mut context = ProjectContext::from_current_dir().unwrap_or_default();

    // Override with app's current directory
    context.current_directory = app.cwd.clone();

    // Add available commands from registry
    let commands: Vec<String> =
        app.registry.get_all().iter().take(30).map(|cmd| cmd.name.clone()).collect();
    context.available_commands = commands;

    // Add recent commands from history
    if let Some(ref manager) = app.history_manager {
        let recent: Vec<String> =
            manager.get_recent(5).iter().map(|e| e.command_name.clone()).collect();
        context.recent_commands = recent;
    }

    // Get project name from directory
    context.project_name =
        app.cwd.file_name().and_then(|n| n.to_str()).unwrap_or("project").to_string();

    context
}

/// Call Ollama API with context-aware system prompt and conversation history.
#[cfg(feature = "ai")]
async fn call_ollama(
    prompt: &str,
    system_prompt: &str,
    history: &[(String, String)],
) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let base_url =
        std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".to_string());

    #[derive(serde::Serialize)]
    struct OllamaChatMessage {
        role: String,
        content: String,
    }

    #[derive(serde::Serialize)]
    struct OllamaChatRequest {
        model: String,
        messages: Vec<OllamaChatMessage>,
        stream: bool,
    }

    #[derive(serde::Deserialize)]
    struct OllamaChatMessageResponse {
        content: String,
    }

    #[derive(serde::Deserialize)]
    struct OllamaChatResponse {
        message: OllamaChatMessageResponse,
    }

    // Build messages with system prompt, history, and current message
    let mut messages =
        vec![OllamaChatMessage { role: "system".to_string(), content: system_prompt.to_string() }];

    // Add conversation history (last 5 exchanges to avoid token overflow)
    for (user_msg, ai_msg) in history.iter().rev().take(5).rev() {
        messages.push(OllamaChatMessage { role: "user".to_string(), content: user_msg.clone() });
        if !ai_msg.is_empty() {
            messages
                .push(OllamaChatMessage { role: "assistant".to_string(), content: ai_msg.clone() });
        }
    }

    // Add current user message
    messages.push(OllamaChatMessage { role: "user".to_string(), content: prompt.to_string() });

    let request = OllamaChatRequest { model: model.clone(), messages, stream: false };

    let response = client
        .post(format!("{}/api/chat", base_url))
        .json(&request)
        .timeout(std::time::Duration::from_secs(120))
        .send()
        .await?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        anyhow::bail!(
            "Model '{}' not found.\n\nTo install it, run:\n  ollama pull {}\n\nOr set OLLAMA_MODEL to an installed model.",
            model,
            model
        );
    }

    if !response.status().is_success() {
        anyhow::bail!("Ollama error ({}). Is it running?", response.status());
    }

    let result: OllamaChatResponse = response.json().await?;
    Ok(result.message.content.trim().to_string())
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
