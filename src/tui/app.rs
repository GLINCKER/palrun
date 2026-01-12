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
use ratatui::{backend::CrosstermBackend, Terminal, TerminalOptions, Viewport};

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

/// Run AI chat in inline mode (native terminal scrolling like Claude Code).
///
/// This uses `Viewport::Inline` to render within the terminal's scrollback
/// buffer, allowing native scroll behavior.
#[cfg(feature = "ai")]
pub fn run_ai_chat_inline(mut app: App) -> Result<()> {
    use crossterm::event::KeyCode;
    use ratatui::{
        prelude::Widget,
        style::{Color, Style},
        text::{Line, Span},
        widgets::{Block, Borders, Paragraph},
    };

    // Setup terminal with inline viewport (4 lines for input area)
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal =
        Terminal::with_options(backend, TerminalOptions { viewport: Viewport::Inline(4) })?;

    // Initialize the app
    if let Err(e) = app.initialize() {
        disable_raw_mode()?;
        return Err(e);
    }

    // Print initial header
    println!("\n{}", "─".repeat(60));
    println!(
        " {} AI Chat │ {} │ Type message or /help",
        "●",
        app.ai_status.as_deref().unwrap_or("No AI")
    );
    println!("{}\n", "─".repeat(60));

    loop {
        // Draw the input area
        terminal.draw(|frame| {
            let area = frame.area();
            let _theme = &app.theme; // Reserved for future theming

            // Show input prompt
            let input_style = if app.ai_thinking {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Cyan)
            };

            let input_text = if app.ai_thinking {
                format!("{} {}", app.spinner_char(), app.thinking_message())
            } else {
                app.ai_chat_input.clone()
            };

            let prompt = if app.ai_thinking { "  " } else { "> " };

            let input = Paragraph::new(Line::from(vec![
                Span::styled(prompt, Style::default().fg(Color::Green)),
                Span::styled(input_text, input_style),
                Span::styled("█", Style::default().fg(Color::Gray)), // Cursor
            ]))
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );

            frame.render_widget(input, area);
        })?;

        // Handle events
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Char('c')
                        if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        break
                    }
                    KeyCode::Enter if !app.ai_chat_input.is_empty() && !app.ai_thinking => {
                        let input = std::mem::take(&mut app.ai_chat_input);

                        // Print user message above viewport
                        terminal.insert_before(1, |buf| {
                            Paragraph::new(Line::from(vec![
                                Span::styled("> ", Style::default().fg(Color::Green)),
                                Span::styled(&input, Style::default().fg(Color::White)),
                            ]))
                            .render(buf.area, buf);
                        })?;

                        // Handle slash commands
                        if input.starts_with('/') {
                            let response = match input.as_str() {
                                "/help" => "Commands: /clear, /model, /context, /help, Esc to exit",
                                "/clear" => "Chat cleared",
                                "/context" => &format!(
                                    "Dir: {} | Commands: {}",
                                    app.cwd.display(),
                                    app.registry.len()
                                ),
                                _ => "Unknown command. Type /help",
                            };
                            terminal.insert_before(1, |buf| {
                                Paragraph::new(Line::from(Span::styled(
                                    response,
                                    Style::default().fg(Color::Cyan),
                                )))
                                .render(buf.area, buf);
                            })?;
                            continue;
                        }

                        // Call AI
                        app.ai_thinking = true;

                        // Actually call Ollama API
                        let response = call_ollama_sync(&input, &app.ai_chat_history);
                        app.ai_thinking = false;

                        // Store in history for context
                        app.ai_chat_history.push((input.clone(), response.clone()));

                        // Show AI response (handle multiline)
                        for line in response.lines() {
                            let line_owned = line.to_string();
                            terminal.insert_before(1, |buf| {
                                Paragraph::new(Line::from(Span::styled(
                                    format!("● {}", line_owned),
                                    Style::default().fg(Color::Blue),
                                )))
                                .render(buf.area, buf);
                            })?;
                        }
                        terminal.insert_before(1, |buf| {
                            Paragraph::new(Line::from(Span::styled(
                                "───",
                                Style::default().fg(Color::DarkGray),
                            )))
                            .render(buf.area, buf);
                        })?;
                    }
                    KeyCode::Char(c) if !app.ai_thinking => {
                        app.ai_chat_input.push(c);
                    }
                    KeyCode::Backspace if !app.ai_thinking => {
                        app.ai_chat_input.pop();
                    }
                    _ => {}
                }
            }
        }

        // Update spinner animation
        app.tick();
    }

    // Cleanup
    disable_raw_mode()?;
    println!("\n");

    Ok(())
}

/// Call Ollama API synchronously for inline chat.
#[cfg(feature = "ai")]
fn call_ollama_sync(prompt: &str, history: &[(String, String)]) -> String {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize)]
    struct Message {
        role: String,
        content: String,
    }

    #[derive(Serialize)]
    struct Request {
        model: String,
        messages: Vec<Message>,
        stream: bool,
    }

    #[derive(Deserialize)]
    struct ResponseMessage {
        content: String,
    }

    #[derive(Deserialize)]
    struct Response {
        message: ResponseMessage,
    }

    let base_url =
        std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".to_string());

    // Build messages with history
    let mut messages = vec![Message {
        role: "system".to_string(),
        content: "You are a helpful assistant for a developer working in a terminal. Keep responses concise and actionable.".to_string(),
    }];

    // Add conversation history (last 5 exchanges)
    for (user_msg, ai_msg) in history.iter().rev().take(5).rev() {
        messages.push(Message { role: "user".to_string(), content: user_msg.clone() });
        if !ai_msg.is_empty() {
            messages.push(Message { role: "assistant".to_string(), content: ai_msg.clone() });
        }
    }

    // Add current prompt
    messages.push(Message { role: "user".to_string(), content: prompt.to_string() });

    let request = Request { model: model.clone(), messages, stream: false };

    // Use blocking client
    let client = reqwest::blocking::Client::new();
    match client
        .post(format!("{}/api/chat", base_url))
        .json(&request)
        .timeout(std::time::Duration::from_secs(120))
        .send()
    {
        Ok(resp) => {
            if resp.status() == reqwest::StatusCode::NOT_FOUND {
                return format!("Model '{}' not found. Run: ollama pull {}", model, model);
            }
            if !resp.status().is_success() {
                return format!("Ollama error ({}). Is it running?", resp.status());
            }
            match resp.json::<Response>() {
                Ok(r) => r.message.content.trim().to_string(),
                Err(e) => format!("Failed to parse response: {}", e),
            }
        }
        Err(e) => format!("Failed to connect to Ollama: {}", e),
    }
}
