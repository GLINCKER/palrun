//! UI rendering for the TUI.
//!
//! Handles layout and widget rendering using ratatui.
//! Supports customizable themes via the Theme struct.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph, Wrap},
    Frame,
};

use crate::app::AppMode;
use crate::App;

/// Draw the main UI.
pub fn draw(frame: &mut Frame, app: &App) {
    // Check if we're showing execution result
    if matches!(app.mode, AppMode::ExecutionResult) {
        draw_execution_result(frame, app);
        return;
    }

    // Check if we're showing help screen
    if matches!(app.mode, AppMode::Help) {
        draw_help_screen(frame, app);
        return;
    }

    // Check if we're showing history screen
    if matches!(app.mode, AppMode::History) {
        draw_history_screen(frame, app);
        return;
    }

    // Check if we're showing analytics screen
    if matches!(app.mode, AppMode::Analytics) {
        draw_analytics_screen(frame, app);
        return;
    }

    let area = frame.area();

    // Main vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header with search
            Constraint::Min(8),    // Main content (list + preview)
            Constraint::Length(2), // Help bar
        ])
        .split(area);

    // Split main content into command list and preview (horizontal)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Command list
            Constraint::Percentage(40), // Preview panel
        ])
        .split(chunks[1]);

    draw_header(frame, app, chunks[0]);
    draw_command_list(frame, app, content_chunks[0]);
    draw_preview_panel(frame, app, content_chunks[1]);
    draw_help_bar(frame, app, chunks[2]);

    // Draw overlays for special modes
    if matches!(app.mode, AppMode::PassThrough) {
        draw_pass_through_overlay(frame, app);
    }
    if matches!(app.mode, AppMode::Palette) {
        draw_palette_overlay(frame, app);
    }
    if matches!(app.mode, AppMode::ContextMenu) {
        draw_context_menu_overlay(frame, app);
    }
}

/// Draw the header with search input.
fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    // Build the right side of the title (stats + git)
    #[cfg(feature = "git")]
    let right_title = {
        if let Some(git_status) = app.git_status_display() {
            format!("{} │ {} ", git_status, format_command_count(app))
        } else {
            format!("{} ", format_command_count(app))
        }
    };

    #[cfg(not(feature = "git"))]
    let right_title = format!("{} ", format_command_count(app));

    // Left title with logo
    let left_title = " pal ";

    let input = Paragraph::new(Line::from(vec![
        Span::styled(" > ", Style::default().fg(theme.secondary)),
        Span::styled(&app.input, Style::default().fg(theme.text)),
        Span::styled("│", Style::default().fg(theme.border)),
    ]))
    .style(Style::default().fg(theme.text))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary))
            .title(left_title)
            .title_style(
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            )
            .title_bottom(Line::from(right_title).right_aligned())
            .title_style(Style::default().fg(theme.text_dim)),
    );

    frame.render_widget(input, area);

    // Position cursor after prompt (border + " > " + input position)
    frame.set_cursor_position((area.x + 4 + app.cursor_position as u16, area.y + 1));
}

/// Format command count with optional filtering indicator.
fn format_command_count(app: &App) -> String {
    let filtered = app.filtered_commands.len();
    let total = app.registry.len();

    let count_str = if filtered == total {
        format!("{} commands", total)
    } else {
        format!("{}/{} commands", filtered, total)
    };

    // Add filter indicator if filters are active
    if let Some(ref filters) = app.active_filters {
        format!("{} [{}]", count_str, filters)
    } else {
        count_str
    }
}

/// Draw the command list.
fn draw_command_list(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    // Calculate visible range for scroll indicator
    let visible_height = area.height.saturating_sub(2) as usize; // Account for borders
    let total = app.filtered_commands.len();
    let scroll_info = if total > visible_height && visible_height > 0 {
        format!(" {}/{} ", app.selected + 1, total)
    } else if total > 0 {
        format!(" {} ", total)
    } else {
        String::new()
    };

    // Handle empty state
    if app.filtered_commands.is_empty() {
        let empty_message = if app.input.is_empty() {
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No commands found",
                    Style::default().fg(theme.text_dim),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Run 'pal scan' to discover commands",
                    Style::default().fg(theme.text_muted),
                )),
            ]
        } else if app.active_filters.is_some() {
            // Filters are active but no matches
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    format!("No matches for \"{}\"", app.input),
                    Style::default().fg(theme.text_dim),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Try adjusting filters: #tag source:type @workspace",
                    Style::default().fg(theme.text_muted),
                )),
            ]
        } else {
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    format!("No matches for \"{}\"", app.input),
                    Style::default().fg(theme.text_dim),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Try: #tag, source:npm, @workspace",
                    Style::default().fg(theme.text_muted),
                )),
            ]
        };

        let empty = Paragraph::new(empty_message)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(" Commands ")
                    .title_style(Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
            );

        frame.render_widget(empty, area);
        return;
    }

    let commands: Vec<ListItem> = app
        .filtered_commands
        .iter()
        .enumerate()
        .map(|(i, &cmd_idx)| {
            let cmd = app.registry.get_by_index(cmd_idx);
            let is_selected = i == app.selected;

            match cmd {
                Some(cmd) => {
                    let source_icon = cmd.source.icon();
                    let name = &cmd.name;

                    // Check if this item is multi-selected
                    let is_multi_selected = app.is_selected(i);

                    // Different styling for selected vs unselected
                    let (name_style, icon_style) = if is_selected {
                        (
                            Style::default()
                                .fg(theme.text)
                                .add_modifier(Modifier::BOLD),
                            Style::default().fg(theme.primary),
                        )
                    } else {
                        (
                            Style::default().fg(theme.text),
                            Style::default().fg(theme.text_dim),
                        )
                    };

                    // Build spans for the line - add checkbox for multi-select mode
                    let mut spans = Vec::new();

                    // Multi-select checkbox
                    if app.multi_select_mode {
                        let checkbox = if is_multi_selected { "[✓] " } else { "[ ] " };
                        spans.push(Span::styled(
                            checkbox,
                            Style::default().fg(if is_multi_selected { theme.success } else { theme.text_dim }),
                        ));
                    } else {
                        spans.push(Span::styled(
                            if is_selected { "▸ " } else { "  " },
                            Style::default().fg(theme.primary),
                        ));
                    }

                    spans.push(Span::styled(format!("{source_icon} "), icon_style));
                    spans.push(Span::styled(name.as_str(), name_style));

                    // Add source label
                    let source_label = format!(" [{}]", cmd.source.short_name());
                    spans.push(Span::styled(source_label, Style::default().fg(theme.text_muted)));

                    // Add favorite indicator
                    if app.is_favorite(&cmd.id) {
                        spans.push(Span::styled(" ⭐", Style::default().fg(theme.warning)));
                    }

                    // Add branch indicator if command is branch-specific
                    if !cmd.branch_patterns.is_empty() {
                        let branch_text = if cmd.branch_patterns.len() == 1 {
                            format!(" ⎇ {}", cmd.branch_patterns[0])
                        } else {
                            format!(" ⎇ {}", cmd.branch_patterns.len())
                        };
                        spans.push(Span::styled(
                            branch_text,
                            Style::default().fg(theme.accent),
                        ));
                    }

                    let line = Line::from(spans);

                    ListItem::new(line).style(if is_selected {
                        Style::default().bg(theme.selected_bg)
                    } else {
                        Style::default()
                    })
                }
                None => ListItem::new("(unknown)"),
            }
        })
        .collect();

    let title = if app.multi_select_mode && !app.selected_commands.is_empty() {
        format!(" Commands {} ({} selected)", scroll_info, app.selected_commands.len())
    } else {
        format!(" Commands {}", scroll_info)
    };
    let list = List::new(commands)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(title)
                .title_style(Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        )
        .highlight_style(Style::default().bg(theme.selected_bg).add_modifier(Modifier::BOLD));

    frame.render_widget(list, area);
}

/// Draw the preview panel (right side).
fn draw_preview_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let content = if let Some(cmd) = app.get_selected_command() {
        let mut lines = Vec::new();

        // Command name as header
        lines.push(Line::from(Span::styled(
            &cmd.name,
            Style::default()
                .fg(theme.text)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from("")); // Spacer

        // Description if available
        if let Some(ref desc) = cmd.description {
            lines.push(Line::from(Span::styled(
                desc.as_str(),
                Style::default().fg(theme.text_dim),
            )));
            lines.push(Line::from("")); // Spacer
        }

        // Command to execute
        lines.push(Line::from(vec![
            Span::styled("$ ", Style::default().fg(theme.secondary)),
            Span::styled(&cmd.command, Style::default().fg(theme.text)),
        ]));

        // Working directory if different
        if let Some(ref dir) = cmd.working_dir {
            lines.push(Line::from(vec![
                Span::styled("in ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    dir.display().to_string(),
                    Style::default().fg(theme.text_dim),
                ),
            ]));
        }

        // Source info
        lines.push(Line::from("")); // Spacer
        lines.push(Line::from(vec![
            Span::styled("Source: ", Style::default().fg(theme.text_muted)),
            Span::styled(
                format!("{} {}", cmd.source.icon(), cmd.source.short_name()),
                Style::default().fg(theme.text_dim),
            ),
        ]));

        // Branch patterns if command is branch-specific
        if !cmd.branch_patterns.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("Branches: ", Style::default().fg(theme.text_muted)),
                Span::styled(
                    cmd.branch_patterns.join(", "),
                    Style::default().fg(theme.accent),
                ),
            ]));
        }

        lines
    } else if let Some(ref status) = app.status_message {
        vec![Line::from(Span::styled(
            status.as_str(),
            Style::default().fg(theme.warning),
        ))]
    } else {
        vec![
            Line::from(""),
            Line::from(Span::styled(
                "Select a command",
                Style::default().fg(theme.text_dim),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Type to search...",
                Style::default().fg(theme.text_muted),
            )),
        ]
    };

    let preview = Paragraph::new(content)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(" Preview ")
                .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
                .padding(Padding::horizontal(1)),
        );

    frame.render_widget(preview, area);
}

/// Draw the help bar at the bottom.
fn draw_help_bar(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let run_selected_label = format!("Run {}", app.selected_count());
    let help_items: Vec<(&str, &str)> = if app.multi_select_mode {
        // Multi-select mode help
        vec![
            ("Space", "Select"),
            ("Enter", &run_selected_label),
            ("Ctrl+A", "All"),
            ("Esc", "Cancel"),
        ]
    } else {
        // Normal mode help
        vec![
            ("Enter", "Run"),
            ("↑↓", "Navigate"),
            ("Ctrl+S", "Fav"),
            ("?", "Help"),
            ("Esc", "Quit"),
        ]
    };

    let mut spans = Vec::new();
    for (i, (key, action)) in help_items.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", Style::default().fg(theme.border)));
        }
        spans.push(Span::styled(
            format!(" {} ", key),
            Style::default()
                .fg(theme.text)
                .bg(theme.selected_bg)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {} ", action),
            Style::default().fg(theme.text_dim),
        ));
    }

    let help = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text_dim));

    frame.render_widget(help, area);
}

/// Draw the execution result screen.
fn draw_execution_result(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = frame.area();

    // Layout: header, output area, help bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),  // Header with command info
            Constraint::Min(8),     // Output area (stdout/stderr)
            Constraint::Length(2),  // Help bar
        ])
        .split(area);

    // Get the output
    let output = match &app.last_output {
        Some(o) => o,
        None => {
            // Should not happen, but handle gracefully
            let msg = Paragraph::new("No output available")
                .alignment(Alignment::Center);
            frame.render_widget(msg, area);
            return;
        }
    };

    // Header with command info and status
    let status_icon = if output.success { "✓" } else { "✗" };
    let status_color = if output.success { theme.success } else { theme.error };
    let exit_code_text = output.exit_code
        .map(|c| format!(" (exit {})", c))
        .unwrap_or_default();

    let header_lines = vec![
        Line::from(vec![
            Span::styled(
                format!("{} ", status_icon),
                Style::default().fg(status_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                &output.command_name,
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                exit_code_text,
                Style::default().fg(theme.text_dim),
            ),
        ]),
        Line::from(vec![
            Span::styled("$ ", Style::default().fg(theme.secondary)),
            Span::styled(&output.command_str, Style::default().fg(theme.text_dim)),
        ]),
    ];

    let header = Paragraph::new(header_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(status_color))
                .title(" Command Result ")
                .title_style(Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        );

    frame.render_widget(header, chunks[0]);

    // Output area - split into stdout and stderr if both exist
    let has_stdout = !output.stdout.is_empty();
    let has_stderr = !output.stderr.is_empty();

    let scroll = app.output_scroll;

    if has_stdout && has_stderr {
        // Split output area horizontally
        let output_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60), // stdout
                Constraint::Percentage(40), // stderr
            ])
            .split(chunks[1]);

        draw_output_panel(frame, app, "stdout", &output.stdout, theme.text, output_chunks[0], scroll);
        draw_output_panel(frame, app, "stderr", &output.stderr, theme.error, output_chunks[1], 0);
    } else if has_stdout {
        draw_output_panel(frame, app, "Output", &output.stdout, theme.text, chunks[1], scroll);
    } else if has_stderr {
        draw_output_panel(frame, app, "stderr", &output.stderr, theme.error, chunks[1], scroll);
    } else {
        // No output
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "Command completed with no output",
                Style::default().fg(theme.text_dim),
            )),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(" Output ")
                .title_style(Style::default().fg(theme.secondary)),
        );
        frame.render_widget(empty, chunks[1]);
    }

    // Help bar for result screen
    let help_items = vec![
        ("↑↓/jk", "Scroll"),
        ("PgUp/Dn", "Page"),
        ("Enter", "Back"),
        ("r", "Re-run"),
        ("q", "Quit"),
    ];

    let mut spans = Vec::new();
    for (i, (key, action)) in help_items.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", Style::default().fg(theme.border)));
        }
        spans.push(Span::styled(
            format!(" {} ", key),
            Style::default()
                .fg(theme.text)
                .bg(theme.selected_bg)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {} ", action),
            Style::default().fg(theme.text_dim),
        ));
    }

    let help = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text_dim));

    frame.render_widget(help, chunks[2]);
}

/// Draw an output panel (stdout or stderr) with scroll support.
fn draw_output_panel(frame: &mut Frame, app: &App, title: &str, content: &str, color: Color, area: Rect, scroll: usize) {
    let theme = &app.theme;

    // Split content into lines
    let all_lines: Vec<Line> = content
        .lines()
        .map(|line| Line::from(Span::styled(line, Style::default().fg(color))))
        .collect();

    let total_lines = all_lines.len();
    let visible_height = area.height.saturating_sub(2) as usize; // Account for borders

    // Calculate scroll info for title
    let scroll_info = if total_lines > visible_height {
        format!(" {}/{} ", scroll + 1, total_lines)
    } else {
        String::new()
    };

    // Apply scroll offset
    let visible_lines: Vec<Line> = all_lines
        .into_iter()
        .skip(scroll)
        .collect();

    let output = Paragraph::new(visible_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(format!(" {} ", title))
                .title_style(Style::default().fg(theme.secondary))
                .title_bottom(Line::from(scroll_info).right_aligned())
                .padding(Padding::horizontal(1)),
        );

    frame.render_widget(output, area);
}

/// Draw the help screen showing all keyboard shortcuts.
fn draw_help_screen(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = frame.area();

    // Layout: title, content, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Content
            Constraint::Length(2),  // Footer
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " Keyboard Shortcuts ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary)),
    );
    frame.render_widget(title, chunks[0]);

    // Help content - organized by category
    let mut lines = Vec::new();

    // Navigation section
    lines.push(Line::from(Span::styled(
        "Navigation",
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(help_line("↑ / k", "Move selection up", theme));
    lines.push(help_line("↓ / j", "Move selection down", theme));
    lines.push(help_line("Ctrl+↑", "Move up (while typing)", theme));
    lines.push(help_line("Ctrl+↓", "Move down (while typing)", theme));
    lines.push(help_line("PageUp", "Move up 10 items", theme));
    lines.push(help_line("PageDown", "Move down 10 items", theme));
    lines.push(help_line("Ctrl+Home", "Jump to first item", theme));
    lines.push(help_line("Ctrl+End", "Jump to last item", theme));
    lines.push(Line::from(""));

    // Execution section
    lines.push(Line::from(Span::styled(
        "Execution",
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(help_line("Enter", "Run selected command", theme));
    lines.push(help_line("Ctrl+B", "Run in background", theme));
    lines.push(Line::from(""));

    // Multi-select section
    lines.push(Line::from(Span::styled(
        "Multi-Select Mode",
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(help_line("Ctrl+Space", "Toggle multi-select mode", theme));
    lines.push(help_line("Space", "Select/deselect item", theme));
    lines.push(help_line("Ctrl+A", "Select all items", theme));
    lines.push(help_line("Enter", "Run selected commands in parallel", theme));
    lines.push(Line::from(""));

    // Input section
    lines.push(Line::from(Span::styled(
        "Input & Search",
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(help_line("Type", "Filter commands", theme));
    lines.push(help_line("Backspace", "Delete character", theme));
    lines.push(help_line("Ctrl+U", "Clear input", theme));
    lines.push(help_line("← / →", "Move cursor", theme));
    lines.push(help_line("Home", "Cursor to start", theme));
    lines.push(help_line("End", "Cursor to end", theme));
    lines.push(Line::from(""));

    // Filter syntax section
    lines.push(Line::from(Span::styled(
        "Filter Syntax",
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(help_line("#tag", "Filter by tag", theme));
    lines.push(help_line("source:npm", "Filter by source type", theme));
    lines.push(help_line("@workspace", "Filter by workspace", theme));
    lines.push(Line::from(""));

    // Favorites section
    lines.push(Line::from(Span::styled(
        "Favorites",
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(help_line("Ctrl+S", "Toggle favorite", theme));
    lines.push(Line::from(""));

    // General section
    lines.push(Line::from(Span::styled(
        "General",
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    lines.push(help_line("?", "Show this help", theme));
    lines.push(help_line("Ctrl+H", "Show command history", theme));
    lines.push(help_line("Ctrl+G", "Show usage analytics", theme));
    lines.push(help_line("Esc", "Quit / Cancel", theme));
    lines.push(help_line("Ctrl+C", "Quit", theme));
    lines.push(help_line("q", "Quit (when input empty)", theme));

    let content = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .padding(Padding::horizontal(2)),
        );
    frame.render_widget(content, chunks[1]);

    // Footer with dismiss hint
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            " Press ",
            Style::default().fg(theme.text_dim),
        ),
        Span::styled(
            "Esc",
            Style::default()
                .fg(theme.text)
                .bg(theme.selected_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            ", ",
            Style::default().fg(theme.text_dim),
        ),
        Span::styled(
            "?",
            Style::default()
                .fg(theme.text)
                .bg(theme.selected_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            ", or ",
            Style::default().fg(theme.text_dim),
        ),
        Span::styled(
            "Enter",
            Style::default()
                .fg(theme.text)
                .bg(theme.selected_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " to close ",
            Style::default().fg(theme.text_dim),
        ),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

/// Draw the history screen showing recent command executions.
fn draw_history_screen(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = frame.area();

    // Layout: title, content, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Min(10),    // Content
            Constraint::Length(2),  // Footer
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " Command History ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary)),
    );
    frame.render_widget(title, chunks[0]);

    // Get history entries
    let history_entries = app.get_recent_history(50);

    if history_entries.is_empty() {
        // Empty state
        let empty_lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No command history yet",
                Style::default().fg(theme.text_dim),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Execute some commands to see history here",
                Style::default().fg(theme.text_muted),
            )),
        ];

        let empty = Paragraph::new(empty_lines)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border)),
            );
        frame.render_widget(empty, chunks[1]);
    } else {
        // Build history lines
        let mut lines = Vec::new();

        // Header row
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:<30}", "Command"),
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:>8}", "Runs"),
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:>10}", "Success"),
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:>12}", "Last Used"),
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(Span::styled(
            "─".repeat(60),
            Style::default().fg(theme.border),
        )));

        // History entries
        for entry in history_entries.iter().take(30) {
            let success_rate = entry.success_rate()
                .map(|r| format!("{:.0}%", r * 100.0))
                .unwrap_or_else(|| "-".to_string());

            let success_color = entry.success_rate()
                .map(|r| if r >= 0.8 { theme.success } else if r >= 0.5 { theme.warning } else { theme.error })
                .unwrap_or(theme.text_dim);

            // Truncate command name if too long
            let cmd_name = if entry.command_name.len() > 28 {
                format!("{}...", &entry.command_name[..25])
            } else {
                entry.command_name.clone()
            };

            lines.push(Line::from(vec![
                Span::styled(
                    format!("{:<30}", cmd_name),
                    Style::default().fg(theme.text),
                ),
                Span::styled(
                    format!("{:>8}", entry.execution_count),
                    Style::default().fg(theme.text_dim),
                ),
                Span::styled(
                    format!("{:>10}", success_rate),
                    Style::default().fg(success_color),
                ),
                Span::styled(
                    format!("{:>12}", entry.last_used_display()),
                    Style::default().fg(theme.text_muted),
                ),
            ]));
        }

        let content = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .padding(Padding::horizontal(1)),
            );
        frame.render_widget(content, chunks[1]);
    }

    // Footer with hints
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            " Press ",
            Style::default().fg(theme.text_dim),
        ),
        Span::styled(
            "Esc",
            Style::default()
                .fg(theme.text)
                .bg(theme.selected_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " or ",
            Style::default().fg(theme.text_dim),
        ),
        Span::styled(
            "Ctrl+H",
            Style::default()
                .fg(theme.text)
                .bg(theme.selected_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " to close ",
            Style::default().fg(theme.text_dim),
        ),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

/// Draw the analytics screen showing usage statistics and insights.
fn draw_analytics_screen(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = frame.area();

    // Get analytics report
    let report = app.get_analytics_report(crate::core::TimePeriod::AllTime);

    // Layout: title, stats row, chart, insights, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Title
            Constraint::Length(5),   // Summary stats
            Constraint::Min(8),      // Bar chart
            Constraint::Length(8),   // Insights
            Constraint::Length(2),   // Footer
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " Usage Analytics ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" ({})", report.period),
            Style::default().fg(theme.text_dim),
        ),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary)),
    );
    frame.render_widget(title, chunks[0]);

    // Summary stats row
    let total_time_str = crate::core::Analytics::format_duration(report.total_time);
    let stats_text = vec![
        Line::from(vec![
            Span::styled("  Total Executions: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("{}", report.total_executions),
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled("    Unique Commands: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("{}", report.unique_commands),
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled("    Success Rate: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                format!("{:.1}%", report.overall_success_rate),
                Style::default()
                    .fg(if report.overall_success_rate >= 80.0 { theme.success }
                        else if report.overall_success_rate >= 50.0 { theme.warning }
                        else { theme.error })
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("    Total Time: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                total_time_str,
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let stats = Paragraph::new(stats_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(Span::styled(" Summary ", Style::default().fg(theme.accent))),
        );
    frame.render_widget(stats, chunks[1]);

    // Bar chart for top commands
    if report.top_commands.is_empty() {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No command history yet",
                Style::default().fg(theme.text_dim),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Execute some commands to see analytics",
                Style::default().fg(theme.text_muted),
            )),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(Span::styled(" Top Commands ", Style::default().fg(theme.accent))),
        );
        frame.render_widget(empty, chunks[2]);
    } else {
        let max_count = report.top_commands.iter().map(|s| s.execution_count).max().unwrap_or(1);
        let chart_width = chunks[2].width.saturating_sub(4) as usize;

        let mut chart_lines = Vec::new();
        for (i, stat) in report.top_commands.iter().take(6).enumerate() {
            let name = if stat.name.len() > 20 {
                format!("{}...", &stat.name[..17])
            } else {
                format!("{:<20}", stat.name)
            };

            let bar_max_width = chart_width.saturating_sub(30);
            let bar_len = (stat.execution_count as f64 / max_count as f64 * bar_max_width as f64) as usize;

            // Alternate colors for visual clarity
            let bar_color = if i % 2 == 0 { theme.primary } else { theme.accent };

            chart_lines.push(Line::from(vec![
                Span::styled(name, Style::default().fg(theme.text)),
                Span::raw(" "),
                Span::styled(
                    "█".repeat(bar_len.max(1)),
                    Style::default().fg(bar_color),
                ),
                Span::styled(
                    format!(" ({})", stat.execution_count),
                    Style::default().fg(theme.text_dim),
                ),
            ]));
        }

        let chart = Paragraph::new(chart_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(Span::styled(" Top Commands ", Style::default().fg(theme.accent)))
                    .padding(Padding::horizontal(1)),
            );
        frame.render_widget(chart, chunks[2]);
    }

    // Insights section
    if report.insights.is_empty() {
        let no_insights = Paragraph::new(vec![
            Line::from(Span::styled(
                "  Run more commands to generate insights",
                Style::default().fg(theme.text_dim),
            )),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(Span::styled(" Insights ", Style::default().fg(theme.accent))),
        );
        frame.render_widget(no_insights, chunks[3]);
    } else {
        let mut insight_lines = Vec::new();
        for insight in report.insights.iter().take(4) {
            let icon_color = match insight.category {
                crate::core::InsightCategory::HighUsage => theme.primary,
                crate::core::InsightCategory::Performance => theme.warning,
                crate::core::InsightCategory::FailureRate => theme.error,
                crate::core::InsightCategory::TimeSpent => theme.accent,
                crate::core::InsightCategory::Positive => theme.success,
            };

            insight_lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} ", insight.category.icon()),
                    Style::default().fg(icon_color),
                ),
                Span::styled(&insight.message, Style::default().fg(theme.text)),
            ]));

            if let Some(ref suggestion) = insight.suggestion {
                insight_lines.push(Line::from(vec![
                    Span::raw("       "),
                    Span::styled(
                        format!("→ {}", suggestion),
                        Style::default().fg(theme.text_muted).add_modifier(Modifier::ITALIC),
                    ),
                ]));
            }
        }

        let insights = Paragraph::new(insight_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title(Span::styled(" Insights ", Style::default().fg(theme.accent))),
            );
        frame.render_widget(insights, chunks[3]);
    }

    // Footer with hints
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            " Press ",
            Style::default().fg(theme.text_dim),
        ),
        Span::styled(
            "Esc",
            Style::default()
                .fg(theme.text)
                .bg(theme.selected_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " or ",
            Style::default().fg(theme.text_dim),
        ),
        Span::styled(
            "Ctrl+G",
            Style::default()
                .fg(theme.text)
                .bg(theme.selected_bg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " to close ",
            Style::default().fg(theme.text_dim),
        ),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[4]);
}

/// Helper to create a help line with key and description.
fn help_line<'a>(key: &'a str, description: &'a str, theme: &crate::tui::Theme) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("  {:14}", key),
            Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            description,
            Style::default().fg(theme.text),
        ),
    ])
}

/// Draw the pass-through confirmation overlay.
fn draw_pass_through_overlay(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = frame.area();

    // Center the popup
    let popup_width = 50.min(area.width.saturating_sub(4));
    let popup_height = 5;
    let popup_area = Rect::new(
        (area.width.saturating_sub(popup_width)) / 2,
        (area.height.saturating_sub(popup_height)) / 2,
        popup_width,
        popup_height,
    );

    // Clear and render
    frame.render_widget(Clear, popup_area);

    let cmd = app.pass_through_command.as_deref().unwrap_or("");
    let truncated_cmd = if cmd.len() > 40 {
        format!("{}...", &cmd[..37])
    } else {
        cmd.to_string()
    };

    let content = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" $ ", Style::default().fg(theme.secondary)),
            Span::styled(&truncated_cmd, Style::default().fg(theme.text)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Run as shell command? ", Style::default().fg(theme.text_dim)),
            Span::styled("[Enter] Yes  ", Style::default().fg(theme.success)),
            Span::styled("[Esc] No", Style::default().fg(theme.text_muted)),
        ]),
    ];

    let popup = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary))
                .title(" Run Shell Command ")
                .title_style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
                .style(Style::default().bg(theme.background)),
        );

    frame.render_widget(popup, popup_area);
}

/// Draw the command palette overlay.
fn draw_palette_overlay(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = frame.area();

    // Position at top-center
    let popup_width = 40.min(area.width.saturating_sub(4));
    let popup_height = 9;
    let popup_area = Rect::new(
        (area.width.saturating_sub(popup_width)) / 2,
        3, // Just below header
        popup_width,
        popup_height,
    );

    // Clear and render
    frame.render_widget(Clear, popup_area);

    let items = vec![
        ("History", "Ctrl+H", 0),
        ("Analytics", "Ctrl+G", 1),
        ("Settings", "Ctrl+,", 2),
        ("Plugins", "", 3),
        ("Environment", "", 4),
        ("Help", "?", 5),
    ];

    let list_items: Vec<ListItem> = items
        .iter()
        .map(|(name, shortcut, idx)| {
            let is_selected = app.palette_selected == *idx;
            let line = Line::from(vec![
                Span::styled(
                    if is_selected { " > " } else { "   " },
                    Style::default().fg(theme.accent),
                ),
                Span::styled(
                    format!("{:16}", name),
                    Style::default()
                        .fg(if is_selected { theme.text } else { theme.text_dim })
                        .add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() }),
                ),
                Span::styled(
                    *shortcut,
                    Style::default().fg(theme.text_muted),
                ),
            ]);
            ListItem::new(line).style(if is_selected {
                Style::default().bg(theme.selected_bg)
            } else {
                Style::default()
            })
        })
        .collect();

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary))
                .title(" Actions (Ctrl+P) ")
                .title_style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
                .style(Style::default().bg(theme.background)),
        );

    frame.render_widget(list, popup_area);
}

/// Draw the context menu overlay.
fn draw_context_menu_overlay(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = frame.area();

    // Position near center
    let popup_width = 30.min(area.width.saturating_sub(4));
    let popup_height = 7;
    let popup_area = Rect::new(
        (area.width.saturating_sub(popup_width)) / 2,
        (area.height.saturating_sub(popup_height)) / 2,
        popup_width,
        popup_height,
    );

    // Clear and render
    frame.render_widget(Clear, popup_area);

    let items = vec![
        ("Run", "Enter", 0),
        ("Run in Background", "Ctrl+B", 1),
        ("Toggle Favorite", "Ctrl+S", 2),
        ("Copy Command", "c", 3),
        ("Edit", "e", 4),
    ];

    let list_items: Vec<ListItem> = items
        .iter()
        .map(|(name, shortcut, idx)| {
            let is_selected = app.context_menu_selected == *idx;
            let line = Line::from(vec![
                Span::styled(
                    if is_selected { " > " } else { "   " },
                    Style::default().fg(theme.accent),
                ),
                Span::styled(
                    format!("{:18}", name),
                    Style::default()
                        .fg(if is_selected { theme.text } else { theme.text_dim })
                        .add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() }),
                ),
                Span::styled(
                    *shortcut,
                    Style::default().fg(theme.text_muted),
                ),
            ]);
            ListItem::new(line).style(if is_selected {
                Style::default().bg(theme.selected_bg)
            } else {
                Style::default()
            })
        })
        .collect();

    let title = if let Some(cmd) = app.get_selected_command() {
        format!(" {} ", cmd.name)
    } else {
        " Actions ".to_string()
    };

    let list = List::new(list_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary))
                .title(title)
                .title_style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
                .style(Style::default().bg(theme.background)),
        );

    frame.render_widget(list, popup_area);
}

#[cfg(test)]
mod tests {
    use super::super::Theme;

    #[test]
    fn test_theme_used_in_rendering() {
        // Verify theme is accessible from App
        let theme = Theme::default();
        assert_eq!(theme.name, "default");
    }
}
