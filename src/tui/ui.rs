//! UI rendering for the TUI.
//!
//! Handles layout and widget rendering using ratatui.
//! Supports customizable themes via the Theme struct.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Padding, Paragraph, Wrap},
    Frame,
};

use crate::app::AppMode;
use crate::App;

/// Draw the main UI.
pub fn draw(frame: &mut Frame, app: &App) {
    // Check if we're showing trust confirmation
    if matches!(app.mode, AppMode::TrustConfirmation) {
        draw_trust_confirmation(frame, app);
        return;
    }

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

    // Check if we're showing workflow screen
    if matches!(app.mode, AppMode::Workflow) {
        draw_workflow_screen(frame, app);
        return;
    }

    // Check if we're showing AI chat screen
    #[cfg(feature = "ai")]
    if matches!(app.mode, AppMode::AiChat) {
        draw_ai_chat_screen(frame, app);
        return;
    }

    // Check if we're showing AI setup screen
    #[cfg(feature = "ai")]
    if matches!(app.mode, AppMode::AiSetup) {
        draw_ai_setup_screen(frame, app);
        return;
    }

    let area = frame.area();

    // Main vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header with search
            Constraint::Min(8),    // Main content (list + preview)
            Constraint::Length(1), // Status bar
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
    draw_status_bar(frame, app, chunks[2]);

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

    // Build terminal-style prompt with directory
    let prompt_dir = short_dir_name(&app.cwd);
    let prompt = format!(" {}  ", prompt_dir);

    // Build input line with optional ghost text
    let mut input_spans = vec![
        Span::styled(&prompt, Style::default().fg(theme.secondary)),
        Span::styled(&app.input, Style::default().fg(theme.text)),
    ];

    // Add ghost text autocomplete suggestion
    if let Some(ref ghost) = app.ghost_text {
        // Only show ghost text if input ends with space or slash (ready for completion)
        let input_trimmed = app.input.trim();
        if (input_trimmed.starts_with("cd ") || input_trimmed.starts_with("ls "))
            && !ghost.is_empty()
        {
            // Calculate what part of ghost to show (excluding already typed prefix)
            let last_segment = app.input.split(['/', ' ']).next_back().unwrap_or("");
            let ghost_suffix = if ghost.to_lowercase().starts_with(&last_segment.to_lowercase()) {
                &ghost[last_segment.len()..]
            } else {
                ghost.as_str()
            };

            if !ghost_suffix.is_empty() {
                input_spans.push(Span::styled(
                    ghost_suffix,
                    Style::default().fg(theme.text_muted).add_modifier(Modifier::ITALIC),
                ));
            }
        }
    }

    input_spans.push(Span::styled("│", Style::default().fg(theme.border)));

    let input =
        Paragraph::new(Line::from(input_spans)).style(Style::default().fg(theme.text)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary))
                .title(left_title)
                .title_style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
                .title_bottom(Line::from(right_title).right_aligned())
                .title_style(Style::default().fg(theme.text_dim)),
        );

    frame.render_widget(input, area);

    // Position cursor after prompt (border + prompt + input position)
    let prompt_len = prompt.len() as u16;
    frame.set_cursor_position((area.x + 1 + prompt_len + app.cursor_position as u16, area.y + 1));
}

/// Get a short directory name for the prompt.
fn short_dir_name(path: &std::path::Path) -> String {
    // Try to use ~ for home directory
    if let Some(home) = dirs::home_dir() {
        if path == home {
            return "~".to_string();
        }
        if let Ok(suffix) = path.strip_prefix(&home) {
            let name = suffix.file_name().and_then(|s| s.to_str()).unwrap_or("~");
            return format!("~/{}", name);
        }
    }

    // Just use the last component
    path.file_name().and_then(|s| s.to_str()).unwrap_or("/").to_string()
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

    // Handle empty state - show slash commands or directory listing if available
    if app.filtered_commands.is_empty() {
        // Check if we have slash commands to show
        if let Some(ref slash_cmds) = app.slash_commands {
            if !slash_cmds.is_empty() {
                let items: Vec<ListItem> = slash_cmds
                    .iter()
                    .enumerate()
                    .map(|(i, cmd)| {
                        let is_selected = i == app.slash_selected;
                        let style = if is_selected {
                            Style::default()
                                .fg(theme.primary)
                                .add_modifier(Modifier::BOLD)
                                .bg(theme.selected_bg)
                        } else {
                            Style::default().fg(theme.text)
                        };
                        let desc_style = if is_selected {
                            Style::default().fg(theme.text_dim).bg(theme.selected_bg)
                        } else {
                            Style::default().fg(theme.text_muted)
                        };

                        ListItem::new(Line::from(vec![
                            Span::styled(&cmd.name, style),
                            Span::styled("  ", Style::default()),
                            Span::styled(&cmd.description, desc_style),
                        ]))
                    })
                    .collect();

                let list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme.border))
                            .title(" / Commands ")
                            .title_style(
                                Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD),
                            ),
                    )
                    .highlight_style(Style::default().bg(theme.selected_bg));

                let mut list_state = ListState::default().with_selected(Some(app.slash_selected));
                frame.render_stateful_widget(list, area, &mut list_state);
                return;
            }
        }

        // Check if we have directory listing to show
        if let Some(ref dir_entries) = app.dir_listing {
            if !dir_entries.is_empty() {
                // Show directory entries as selectable list items
                let items: Vec<ListItem> = dir_entries
                    .iter()
                    .enumerate()
                    .map(|(i, entry)| {
                        let is_selected = i == app.dir_selected;
                        let (icon, color) = if entry.is_dir {
                            ("▸ ", if is_selected { theme.primary } else { theme.accent })
                        } else {
                            ("  ", if is_selected { theme.text } else { theme.text_dim })
                        };

                        let style = if is_selected {
                            Style::default()
                                .fg(color)
                                .add_modifier(Modifier::BOLD)
                                .bg(theme.selected_bg)
                        } else {
                            Style::default().fg(color)
                        };

                        ListItem::new(Line::from(vec![
                            Span::styled(icon, style),
                            Span::styled(&entry.name, style),
                            if entry.is_dir { Span::styled("/", style) } else { Span::raw("") },
                        ]))
                    })
                    .collect();

                let input = app.input.trim();
                let title =
                    if input.starts_with("cd") { " Select directory " } else { " Directories " };

                let list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme.border))
                            .title(title)
                            .title_style(
                                Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD),
                            ),
                    )
                    .highlight_style(Style::default().bg(theme.selected_bg));

                // Use stateful widget for auto-scroll
                let mut list_state = ListState::default().with_selected(Some(app.dir_selected));
                frame.render_stateful_widget(list, area, &mut list_state);
                return;
            }
        }

        let empty_message = if app.input.is_empty() {
            vec![
                Line::from(""),
                Line::from(Span::styled("No commands found", Style::default().fg(theme.text_dim))),
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

        let empty = Paragraph::new(empty_message).alignment(Alignment::Center).block(
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
                            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                            Style::default().fg(theme.primary),
                        )
                    } else {
                        (Style::default().fg(theme.text), Style::default().fg(theme.text_dim))
                    };

                    // Build spans for the line - add checkbox for multi-select mode
                    let mut spans = Vec::new();

                    // Multi-select checkbox
                    if app.multi_select_mode {
                        let checkbox = if is_multi_selected { "[✓] " } else { "[ ] " };
                        spans.push(Span::styled(
                            checkbox,
                            Style::default().fg(if is_multi_selected {
                                theme.success
                            } else {
                                theme.text_dim
                            }),
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
                        spans.push(Span::styled(branch_text, Style::default().fg(theme.accent)));
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

    // Use stateful widget for auto-scroll
    let mut list_state = ListState::default().with_selected(Some(app.selected));
    frame.render_stateful_widget(list, area, &mut list_state);
}

/// Draw the preview panel (right side) with context-aware content.
fn draw_preview_panel(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let mut lines = Vec::new();

    // --- Section 1: User & Location Context ---
    // User@host style display (like terminal prompt)
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "user".to_string());

    lines.push(Line::from(vec![
        Span::styled(" ", Style::default().fg(theme.text_muted)),
        Span::styled(&username, Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
    ]));

    // Current directory
    let cwd_display = app.cwd.display().to_string();
    let cwd_short = if let Some(home) = dirs::home_dir() {
        if let Ok(suffix) = app.cwd.strip_prefix(&home) {
            format!("~/{}", suffix.display())
        } else {
            cwd_display.clone()
        }
    } else {
        cwd_display.clone()
    };
    let cwd_truncated = if cwd_short.len() > 35 {
        format!("...{}", &cwd_short[cwd_short.len() - 32..])
    } else {
        cwd_short
    };
    lines.push(Line::from(vec![
        Span::styled(" ", Style::default().fg(theme.text_muted)),
        Span::styled(&cwd_truncated, Style::default().fg(theme.text_dim)),
    ]));

    // Git info (if available)
    #[cfg(feature = "git")]
    if let Some(ref git) = app.git_info {
        let branch_name = git.branch.as_deref().unwrap_or("detached");
        let mut git_spans = vec![
            Span::styled(" ", Style::default().fg(theme.accent)),
            Span::styled(branch_name, Style::default().fg(theme.accent)),
        ];

        // Add ahead/behind if available
        if git.ahead > 0 || git.behind > 0 {
            git_spans.push(Span::styled(
                format!(" ↑{} ↓{}", git.ahead, git.behind),
                Style::default().fg(theme.text_muted),
            ));
        }

        // Add change counts
        let changes = git.staged_count + git.unstaged_count + git.untracked_count;
        if changes > 0 {
            git_spans
                .push(Span::styled(format!(" • {}Δ", changes), Style::default().fg(theme.warning)));
        }

        lines.push(Line::from(git_spans));
    }

    // Project type indicator (sorted for consistent display)
    if !app.registry.is_empty() {
        let sources: std::collections::HashSet<_> =
            app.registry.get_all().iter().map(|c| c.source.short_name()).collect();
        let mut project_types: Vec<_> = sources.into_iter().collect();
        project_types.sort_unstable(); // Sort for consistent ordering
        let project_types: Vec<_> = project_types.into_iter().take(3).collect();
        if !project_types.is_empty() {
            lines.push(Line::from(vec![
                Span::styled(" ", Style::default().fg(theme.secondary)),
                Span::styled(project_types.join(", "), Style::default().fg(theme.text_muted)),
            ]));
        }
    }

    // --- Network/AI Status and Degradation Info ---
    if app.degradation.has_degradations() {
        // Show degraded features with recovery hints
        for degraded in app.degradation.degraded_features() {
            lines.push(Line::from(vec![
                Span::styled("⚠ ", Style::default().fg(theme.warning)),
                Span::styled(format!("{}", degraded.feature), Style::default().fg(theme.warning)),
            ]));
            if let Some(ref fallback) = degraded.fallback {
                lines.push(Line::from(Span::styled(
                    format!("  Using: {}", fallback),
                    Style::default().fg(theme.text_muted),
                )));
            }
            if let Some(ref hint) = degraded.recovery_hint {
                lines.push(Line::from(Span::styled(
                    format!("  Fix: {}", hint),
                    Style::default().fg(theme.text_dim).add_modifier(Modifier::ITALIC),
                )));
            }
        }
    } else if app.is_offline {
        lines.push(Line::from(vec![
            Span::styled("⚡ ", Style::default().fg(theme.warning)),
            Span::styled("Offline Mode", Style::default().fg(theme.warning)),
        ]));
        lines.push(Line::from(Span::styled(
            "  AI features unavailable",
            Style::default().fg(theme.text_muted),
        )));
    } else if let Some(ref ai_status) = app.ai_status {
        lines.push(Line::from(vec![
            Span::styled("✦ ", Style::default().fg(theme.success)),
            Span::styled(ai_status.as_str(), Style::default().fg(theme.text_dim)),
        ]));
    }

    // Show offline queue status if there are pending operations
    if !app.offline_manager.queue().is_empty() {
        let summary = app.offline_manager.queue().summary();
        lines.push(Line::from(vec![
            Span::styled("⏳ ", Style::default().fg(theme.secondary)),
            Span::styled(
                format!("{} ops queued", summary.total),
                Style::default().fg(theme.text_muted),
            ),
        ]));
    }

    lines.push(Line::from("")); // Divider

    // --- Section 2: Selected Command or Status ---
    if let Some(cmd) = app.get_selected_command() {
        // Command name
        lines.push(Line::from(Span::styled(
            &cmd.name,
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        )));

        // Description if available
        if let Some(ref desc) = cmd.description {
            lines
                .push(Line::from(Span::styled(desc.as_str(), Style::default().fg(theme.text_dim))));
        }

        lines.push(Line::from("")); // Spacer

        // Command to execute
        lines.push(Line::from(vec![
            Span::styled("$ ", Style::default().fg(theme.secondary)),
            Span::styled(&cmd.command, Style::default().fg(theme.text)),
        ]));

        // Source info (compact)
        lines.push(Line::from(vec![Span::styled(
            format!("{} {}", cmd.source.icon(), cmd.source.short_name()),
            Style::default().fg(theme.text_muted),
        )]));

        // Execution stats from history
        if let Some(entry) = app.get_history_entry(&cmd.id) {
            lines.push(Line::from("")); // Spacer
            let mut stats_spans = vec![Span::styled(
                format!("{} runs", entry.execution_count),
                Style::default().fg(theme.text_dim),
            )];

            if let Some(rate) = entry.success_rate() {
                let rate_color = if rate >= 80.0 {
                    theme.success
                } else if rate >= 50.0 {
                    theme.warning
                } else {
                    theme.error
                };
                stats_spans.push(Span::styled(" • ", Style::default().fg(theme.border)));
                stats_spans.push(Span::styled(
                    format!("{:.0}% success", rate),
                    Style::default().fg(rate_color),
                ));
            }

            lines.push(Line::from(stats_spans));
        }

        // Branch patterns if command is branch-specific
        if !cmd.branch_patterns.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("⎇ ", Style::default().fg(theme.accent)),
                Span::styled(cmd.branch_patterns.join(", "), Style::default().fg(theme.accent)),
            ]));
        }
    } else if app.is_dir_browsing() {
        // Directory browsing mode - show quick suggestions
        lines.push(Line::from(Span::styled(
            "Quick Navigation",
            Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Shell shortcuts
        let shortcuts = [
            ("cd ..", "Parent directory"),
            ("cd ~", "Home directory"),
            ("cd -", "Previous directory"),
            ("ls -la", "Detailed listing"),
        ];

        for (cmd, desc) in shortcuts {
            lines.push(Line::from(vec![
                Span::styled(cmd, Style::default().fg(theme.accent)),
                Span::styled(format!("  {}", desc), Style::default().fg(theme.text_muted)),
            ]));
        }

        lines.push(Line::from("")); // Spacer

        // Navigation hints
        lines.push(Line::from(Span::styled(
            "Controls:",
            Style::default().fg(theme.text_dim).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(vec![
            Span::styled("↑↓ ", Style::default().fg(theme.accent)),
            Span::styled("Navigate", Style::default().fg(theme.text_dim)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Tab ", Style::default().fg(theme.accent)),
            Span::styled("Complete path", Style::default().fg(theme.text_dim)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Enter ", Style::default().fg(theme.accent)),
            Span::styled("Execute command", Style::default().fg(theme.text_dim)),
        ]));
    } else if let Some(ref status) = app.status_message {
        // Show status message
        lines.push(Line::from(Span::styled(status.as_str(), Style::default().fg(theme.warning))));
    } else {
        // Empty state
        lines.push(Line::from(Span::styled(
            "Select a command",
            Style::default().fg(theme.text_dim),
        )));
        lines.push(Line::from(Span::styled(
            "or type to search",
            Style::default().fg(theme.text_muted),
        )));
    }

    // --- Section 3: Tip (at bottom) ---
    // Calculate remaining space and add tip at end
    let tip = app.current_tip();
    lines.push(Line::from("")); // Spacer before tip
    lines.push(Line::from(vec![
        Span::styled("", Style::default().fg(theme.text_muted)),
        Span::styled(tip, Style::default().fg(theme.text_muted).add_modifier(Modifier::ITALIC)),
    ]));

    let preview = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" Preview ")
            .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
            .padding(Padding::horizontal(1)),
    );

    frame.render_widget(preview, area);
}

/// Tips that rotate in the status bar
const STATUS_TIPS: &[&str] = &[
    "Type to search • ? help",
    "Ctrl+P palette • . actions",
    "Ctrl+S favorite • Ctrl+B background",
    "Ctrl+Space multi-select",
    "Ctrl+H history • Ctrl+G analytics",
];

/// Draw the smart status bar at the bottom.
fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let mut left_spans = Vec::new();

    // Mode indicator (Commands mode with toggle hint)
    #[cfg(feature = "ai")]
    {
        left_spans.push(Span::styled(
            " CMD ",
            Style::default().bg(theme.accent).fg(theme.background).add_modifier(Modifier::BOLD),
        ));
        left_spans.push(Span::styled(" Ctrl+T→AI ", Style::default().fg(theme.text_dim)));
        left_spans.push(Span::styled("│ ", Style::default().fg(theme.border)));
    }

    // Git branch and status (if available)
    #[cfg(feature = "git")]
    if let Some(ref git) = app.git_info {
        let branch = git.branch.as_deref().unwrap_or("detached");
        left_spans.push(Span::styled(
            format!(" {} ", branch),
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));

        // Ahead/behind indicator
        if git.ahead > 0 || git.behind > 0 {
            let sync = if git.ahead > 0 && git.behind > 0 {
                format!("↑{} ↓{}", git.ahead, git.behind)
            } else if git.ahead > 0 {
                format!("↑{}", git.ahead)
            } else {
                format!("↓{}", git.behind)
            };
            left_spans.push(Span::styled(sync, Style::default().fg(theme.text_muted)));
        } else if git.is_clean {
            left_spans.push(Span::styled("✓", Style::default().fg(theme.success)));
        }

        left_spans.push(Span::styled(" │ ", Style::default().fg(theme.border)));
    }

    // Command count
    let cmd_count = app.registry.len();
    left_spans.push(Span::styled(
        format!("{} commands", cmd_count),
        Style::default().fg(theme.text_muted),
    ));

    // Degradation/Offline indicator
    if app.degradation.has_degradations() {
        let count = app.degradation.degraded_features().len();
        left_spans.push(Span::styled(" │ ", Style::default().fg(theme.border)));
        left_spans.push(Span::styled(
            format!("⚠ {} degraded", count),
            Style::default().fg(theme.warning),
        ));
    } else if app.is_offline {
        left_spans.push(Span::styled(" │ ", Style::default().fg(theme.border)));
        left_spans.push(Span::styled(
            "OFFLINE",
            Style::default().fg(theme.error).add_modifier(Modifier::BOLD),
        ));
    }

    left_spans.push(Span::styled(" │ ", Style::default().fg(theme.border)));

    // Current directory (truncated)
    let cwd_display = truncate_path(&app.cwd, 20);
    left_spans.push(Span::styled(cwd_display, Style::default().fg(theme.text_dim)));

    // Right side: rotating tip or mode-specific help
    let right_text = if app.multi_select_mode {
        let count = app.selected_count();
        format!("Space select • Enter run {} • Esc cancel", count)
    } else {
        STATUS_TIPS[app.tip_index % STATUS_TIPS.len()].to_string()
    };

    // Calculate available width for centering
    let left_content = Line::from(left_spans.clone());
    let left_width = left_content.width();
    let right_width = right_text.len();
    let total_width = area.width as usize;

    // Create padding between left and right
    let padding = if total_width > left_width + right_width + 2 {
        total_width - left_width - right_width - 2
    } else {
        1
    };

    left_spans.push(Span::styled(" ".repeat(padding), Style::default()));

    left_spans.push(Span::styled(right_text, Style::default().fg(theme.text_muted)));

    let status =
        Paragraph::new(Line::from(left_spans)).style(Style::default().bg(theme.background));

    frame.render_widget(status, area);
}

/// Truncate a path for display
fn truncate_path(path: &std::path::Path, max_len: usize) -> String {
    // Try to use ~ for home directory
    let display = if let Some(home) = dirs::home_dir() {
        if let Ok(suffix) = path.strip_prefix(&home) {
            format!("~/{}", suffix.display())
        } else {
            path.display().to_string()
        }
    } else {
        path.display().to_string()
    };

    if display.len() <= max_len {
        display
    } else {
        // Truncate from the left with ...
        let start = display.len() - max_len + 3;
        format!("...{}", &display[start..])
    }
}

/// Draw the execution result screen.
fn draw_execution_result(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = frame.area();

    // Layout: header, output area, help bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Header with command info
            Constraint::Min(8),    // Output area (stdout/stderr)
            Constraint::Length(2), // Help bar
        ])
        .split(area);

    // Get the output
    let output = if let Some(o) = &app.last_output {
        o
    } else {
        // Should not happen, but handle gracefully
        let msg = Paragraph::new("No output available").alignment(Alignment::Center);
        frame.render_widget(msg, area);
        return;
    };

    // Header with command info and status
    let status_icon = if output.success { "✓" } else { "✗" };
    let status_color = if output.success { theme.success } else { theme.error };
    let exit_code_text = output.exit_code.map(|c| format!(" (exit {})", c)).unwrap_or_default();

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
            Span::styled(exit_code_text, Style::default().fg(theme.text_dim)),
        ]),
        Line::from(vec![
            Span::styled("$ ", Style::default().fg(theme.secondary)),
            Span::styled(&output.command_str, Style::default().fg(theme.text_dim)),
        ]),
    ];

    let header = Paragraph::new(header_lines).block(
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

        draw_output_panel(
            frame,
            app,
            "stdout",
            &output.stdout,
            theme.text,
            output_chunks[0],
            scroll,
        );
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
    let help_items = [
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
            Style::default().fg(theme.text).bg(theme.selected_bg).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(format!(" {} ", action), Style::default().fg(theme.text_dim)));
    }

    let help = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Center)
        .style(Style::default().fg(theme.text_dim));

    frame.render_widget(help, chunks[2]);
}

/// Draw an output panel (stdout or stderr) with scroll support.
fn draw_output_panel(
    frame: &mut Frame,
    app: &App,
    title: &str,
    content: &str,
    color: Color,
    area: Rect,
    scroll: usize,
) {
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
    let visible_lines: Vec<Line> = all_lines.into_iter().skip(scroll).collect();

    let output = Paragraph::new(visible_lines).block(
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
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Content
            Constraint::Length(2), // Footer
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![Span::styled(
        " Keyboard Shortcuts ",
        Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
    )]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.primary)));
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

    let content = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .padding(Padding::horizontal(2)),
    );
    frame.render_widget(content, chunks[1]);

    // Footer with dismiss hint
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Press ", Style::default().fg(theme.text_dim)),
        Span::styled(
            "Esc",
            Style::default().fg(theme.text).bg(theme.selected_bg).add_modifier(Modifier::BOLD),
        ),
        Span::styled(", ", Style::default().fg(theme.text_dim)),
        Span::styled(
            "?",
            Style::default().fg(theme.text).bg(theme.selected_bg).add_modifier(Modifier::BOLD),
        ),
        Span::styled(", or ", Style::default().fg(theme.text_dim)),
        Span::styled(
            "Enter",
            Style::default().fg(theme.text).bg(theme.selected_bg).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to close ", Style::default().fg(theme.text_dim)),
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
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Content
            Constraint::Length(2), // Footer
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![Span::styled(
        " Command History ",
        Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
    )]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.primary)));
    frame.render_widget(title, chunks[0]);

    // Get history entries
    let history_entries = app.get_recent_history(50);

    if history_entries.is_empty() {
        // Empty state
        let empty_lines = vec![
            Line::from(""),
            Line::from(Span::styled("No command history yet", Style::default().fg(theme.text_dim))),
            Line::from(""),
            Line::from(Span::styled(
                "Execute some commands to see history here",
                Style::default().fg(theme.text_muted),
            )),
        ];

        let empty = Paragraph::new(empty_lines).alignment(Alignment::Center).block(
            Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border)),
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
        lines.push(Line::from(Span::styled("─".repeat(60), Style::default().fg(theme.border))));

        // History entries
        for entry in history_entries.iter().take(30) {
            let success_rate = entry
                .success_rate()
                .map(|r| format!("{:.0}%", r * 100.0))
                .unwrap_or_else(|| "-".to_string());

            let success_color = entry
                .success_rate()
                .map(|r| {
                    if r >= 0.8 {
                        theme.success
                    } else if r >= 0.5 {
                        theme.warning
                    } else {
                        theme.error
                    }
                })
                .unwrap_or(theme.text_dim);

            // Truncate command name if too long
            let cmd_name = if entry.command_name.len() > 28 {
                format!("{}...", &entry.command_name[..25])
            } else {
                entry.command_name.clone()
            };

            lines.push(Line::from(vec![
                Span::styled(format!("{:<30}", cmd_name), Style::default().fg(theme.text)),
                Span::styled(
                    format!("{:>8}", entry.execution_count),
                    Style::default().fg(theme.text_dim),
                ),
                Span::styled(format!("{:>10}", success_rate), Style::default().fg(success_color)),
                Span::styled(
                    format!("{:>12}", entry.last_used_display()),
                    Style::default().fg(theme.text_muted),
                ),
            ]));
        }

        let content = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .padding(Padding::horizontal(1)),
        );
        frame.render_widget(content, chunks[1]);
    }

    // Footer with hints
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Press ", Style::default().fg(theme.text_dim)),
        Span::styled(
            "Esc",
            Style::default().fg(theme.text).bg(theme.selected_bg).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" or ", Style::default().fg(theme.text_dim)),
        Span::styled(
            "Ctrl+H",
            Style::default().fg(theme.text).bg(theme.selected_bg).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to close ", Style::default().fg(theme.text_dim)),
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
            Constraint::Length(3), // Title
            Constraint::Length(5), // Summary stats
            Constraint::Min(8),    // Bar chart
            Constraint::Length(8), // Insights
            Constraint::Length(2), // Footer
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " Usage Analytics ",
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" ({})", report.period), Style::default().fg(theme.text_dim)),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.primary)));
    frame.render_widget(title, chunks[0]);

    // Summary stats row
    let total_time_str = crate::core::Analytics::format_duration(report.total_time);
    let stats_text = vec![Line::from(vec![
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
                .fg(if report.overall_success_rate >= 80.0 {
                    theme.success
                } else if report.overall_success_rate >= 50.0 {
                    theme.warning
                } else {
                    theme.error
                })
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("    Total Time: ", Style::default().fg(theme.text_dim)),
        Span::styled(total_time_str, Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
    ])];

    let stats = Paragraph::new(stats_text).block(
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
            Line::from(Span::styled("No command history yet", Style::default().fg(theme.text_dim))),
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
            let bar_len = (f64::from(stat.execution_count) / f64::from(max_count)
                * bar_max_width as f64) as usize;

            // Alternate colors for visual clarity
            let bar_color = if i % 2 == 0 { theme.primary } else { theme.accent };

            chart_lines.push(Line::from(vec![
                Span::styled(name, Style::default().fg(theme.text)),
                Span::raw(" "),
                Span::styled("█".repeat(bar_len.max(1)), Style::default().fg(bar_color)),
                Span::styled(
                    format!(" ({})", stat.execution_count),
                    Style::default().fg(theme.text_dim),
                ),
            ]));
        }

        let chart = Paragraph::new(chart_lines).block(
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
        let no_insights = Paragraph::new(vec![Line::from(Span::styled(
            "  Run more commands to generate insights",
            Style::default().fg(theme.text_dim),
        ))])
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

        let insights = Paragraph::new(insight_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(Span::styled(" Insights ", Style::default().fg(theme.accent))),
        );
        frame.render_widget(insights, chunks[3]);
    }

    // Footer with hints
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Press ", Style::default().fg(theme.text_dim)),
        Span::styled(
            "Esc",
            Style::default().fg(theme.text).bg(theme.selected_bg).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" or ", Style::default().fg(theme.text_dim)),
        Span::styled(
            "Ctrl+G",
            Style::default().fg(theme.text).bg(theme.selected_bg).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to close ", Style::default().fg(theme.text_dim)),
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
        Span::styled(description, Style::default().fg(theme.text)),
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
    let truncated_cmd = if cmd.len() > 40 { format!("{}...", &cmd[..37]) } else { cmd.to_string() };

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

    let popup = Paragraph::new(content).block(
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

    let items = [
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
                Span::styled(*shortcut, Style::default().fg(theme.text_muted)),
            ]);
            ListItem::new(line).style(if is_selected {
                Style::default().bg(theme.selected_bg)
            } else {
                Style::default()
            })
        })
        .collect();

    let list = List::new(list_items).block(
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

    let items = [
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
                Span::styled(*shortcut, Style::default().fg(theme.text_muted)),
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

    let list = List::new(list_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary))
            .title(title)
            .title_style(Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(theme.background)),
    );

    frame.render_widget(list, popup_area);
}

/// Draw the workflow dashboard screen.
fn draw_workflow_screen(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = frame.area();

    // Layout: title, content, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Content
            Constraint::Length(2), // Footer
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![Span::styled(
        " Workflow Dashboard ",
        Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
    )]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.primary)));
    frame.render_widget(title, chunks[0]);

    // Content - show workflow context or empty state
    let mut lines = Vec::new();

    if let Some(ref ctx) = app.workflow_context {
        // Project info
        if let Some(ref project) = ctx.project {
            lines.push(Line::from(Span::styled(
                " Project",
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(vec![
                Span::styled("   Name: ", Style::default().fg(theme.text_dim)),
                Span::styled(&project.name, Style::default().fg(theme.text)),
            ]));
            if !project.description.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("   ", Style::default().fg(theme.text_dim)),
                    Span::styled(&project.description, Style::default().fg(theme.text_muted)),
                ]));
            }
            lines.push(Line::from(""));
        }

        // Roadmap info
        if let Some(ref roadmap) = ctx.roadmap {
            lines.push(Line::from(Span::styled(
                " Roadmap",
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            )));
            for phase in &roadmap.phases {
                let status_icon = match phase.status {
                    crate::workflow::PhaseStatus::Pending => "○",
                    crate::workflow::PhaseStatus::InProgress => "◐",
                    crate::workflow::PhaseStatus::Completed => "●",
                    crate::workflow::PhaseStatus::Blocked => "⊗",
                };
                let status_color = match phase.status {
                    crate::workflow::PhaseStatus::Pending => theme.text_muted,
                    crate::workflow::PhaseStatus::InProgress => theme.warning,
                    crate::workflow::PhaseStatus::Completed => theme.success,
                    crate::workflow::PhaseStatus::Blocked => theme.error,
                };
                lines.push(Line::from(vec![
                    Span::styled(format!("   {} ", status_icon), Style::default().fg(status_color)),
                    Span::styled(&phase.name, Style::default().fg(theme.text)),
                ]));
            }
            lines.push(Line::from(""));
        }

        // Current state
        if let Some(ref state) = ctx.state {
            lines.push(Line::from(Span::styled(
                " Current State",
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(vec![
                Span::styled("   Phase: ", Style::default().fg(theme.text_dim)),
                Span::styled(format!("{}", state.current_phase), Style::default().fg(theme.text)),
            ]));
            if !state.blockers.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("   Blockers: ", Style::default().fg(theme.error)),
                    Span::styled(
                        format!("{}", state.blockers.len()),
                        Style::default().fg(theme.error),
                    ),
                ]));
            }
        }
    } else {
        // Empty state - no workflow documents
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "No workflow documents found",
            Style::default().fg(theme.text_dim),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Create PROJECT.md, ROADMAP.md, or PLAN.md to get started",
            Style::default().fg(theme.text_muted),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Run 'pal workflow init' to create templates",
            Style::default().fg(theme.accent),
        )));
    }

    let content = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .padding(Padding::horizontal(2)),
    );
    frame.render_widget(content, chunks[1]);

    // Footer with hints
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" Press ", Style::default().fg(theme.text_dim)),
        Span::styled(
            "Esc",
            Style::default().fg(theme.text).bg(theme.selected_bg).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to close ", Style::default().fg(theme.text_dim)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

/// Draw the AI chat screen.
#[cfg(feature = "ai")]
fn draw_ai_chat_screen(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = frame.area();

    // Layout: header, chat history, input at BOTTOM
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(5),    // Chat history (flexible)
            Constraint::Length(3), // Input bar at bottom
        ])
        .split(area);

    // Minimal header with mode indicator
    let ai_status = app.ai_status.as_deref().unwrap_or("No AI");
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " AI ",
            Style::default().bg(theme.primary).fg(theme.background).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Ctrl+T→CMD ", Style::default().fg(theme.text_dim)),
        Span::styled("│ ", Style::default().fg(theme.border)),
        Span::styled(format!("[{}]", ai_status), Style::default().fg(theme.text_dim)),
        Span::styled("  ↑↓ scroll  Esc close", Style::default().fg(theme.text_muted)),
    ]));
    frame.render_widget(header, chunks[0]);

    // Chat history with markdown rendering
    let mut lines: Vec<Line> = Vec::new();
    if app.ai_chat_history.is_empty() {
        if app.ai_status.is_none() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " Setup: ollama run llama3.2",
                Style::default().fg(theme.accent),
            )));
        } else {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                " Ask: \"how to build?\" \"run tests\" \"git status\"",
                Style::default().fg(theme.text_dim),
            )));
        }
    } else {
        for (user_msg, ai_response) in &app.ai_chat_history {
            // User message
            lines.push(Line::from(vec![
                Span::styled("> ", Style::default().fg(theme.secondary)),
                Span::styled(user_msg.as_str(), Style::default().fg(theme.text)),
            ]));

            // AI response with markdown
            if !ai_response.is_empty() {
                let rendered = render_markdown(ai_response, theme);
                lines.extend(rendered);
                lines.push(Line::from(Span::styled("───", Style::default().fg(theme.border))));
            }
        }

        // Thinking indicator with dynamic message
        if app.ai_thinking {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("{} ", app.spinner_char()),
                    Style::default().fg(theme.warning),
                ),
                Span::styled(
                    app.thinking_message(),
                    Style::default().fg(theme.warning).add_modifier(Modifier::ITALIC),
                ),
            ]));
        }
    }

    // Auto-scroll to bottom when new content
    let total_lines = lines.len();
    let visible_height = chunks[1].height.saturating_sub(2) as usize;
    let max_scroll = total_lines.saturating_sub(visible_height);
    let scroll_offset = if app.ai_chat_scroll == 0 {
        max_scroll // Auto-scroll to bottom
    } else {
        max_scroll.saturating_sub(app.ai_chat_scroll)
    };

    let history = Paragraph::new(lines)
        .block(Block::default().padding(Padding::horizontal(1)))
        .wrap(Wrap { trim: false })
        .scroll((scroll_offset as u16, 0));
    frame.render_widget(history, chunks[1]);

    // Input bar at bottom - clean and responsive
    let input_text = if app.ai_thinking {
        format!("{} {}", app.spinner_char(), app.thinking_message())
    } else {
        app.ai_chat_input.clone()
    };

    let input_style = if app.ai_thinking {
        Style::default().fg(theme.warning)
    } else {
        Style::default().fg(theme.text)
    };

    let border_color = if app.ai_thinking { theme.warning } else { theme.accent };

    // Show slash command hints when input starts with "/"
    let input_title = if app.ai_thinking {
        "".to_string()
    } else if app.ai_chat_input.starts_with('/') {
        " / /clear /model /context /help ".to_string()
    } else {
        " > ".to_string()
    };

    let input = Paragraph::new(input_text).style(input_style).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(input_title)
            .title_style(Style::default().fg(theme.accent)),
    );
    frame.render_widget(input, chunks[2]);
}

/// Render markdown text into styled Lines.
#[cfg(feature = "ai")]
fn render_markdown<'a>(text: &'a str, theme: &'a crate::tui::Theme) -> Vec<Line<'a>> {
    let mut lines: Vec<Line> = Vec::new();
    let mut in_code_block = false;
    let mut code_lang = String::new();

    for line in text.lines() {
        // Check for code block start/end
        if line.trim().starts_with("```") {
            if in_code_block {
                // End of code block
                in_code_block = false;
                code_lang.clear();
            } else {
                // Start of code block
                in_code_block = true;
                code_lang = line.trim().strip_prefix("```").unwrap_or("").to_string();
            }
            continue;
        }

        if in_code_block {
            // Code block content - highlighted
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(
                    line.to_string(),
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ),
            ]));
        } else if line.trim().starts_with("- ") || line.trim().starts_with("* ") {
            // Bullet point
            let content = line
                .trim()
                .strip_prefix("- ")
                .or_else(|| line.trim().strip_prefix("* "))
                .unwrap_or(line);
            lines.push(Line::from(vec![
                Span::styled("  • ", Style::default().fg(theme.accent)),
                Span::styled(
                    render_inline_markdown(content, theme),
                    Style::default().fg(theme.text),
                ),
            ]));
        } else if line.trim().starts_with("# ") {
            // Header
            let content = line.trim().strip_prefix("# ").unwrap_or(line);
            lines.push(Line::from(Span::styled(
                content.to_string(),
                Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
            )));
        } else if line.trim().is_empty() {
            lines.push(Line::from(""));
        } else {
            // Regular text with inline code handling
            let spans = parse_inline_code(line, theme);
            lines.push(Line::from(spans));
        }
    }

    lines
}

/// Parse inline code (backticks) and return styled spans.
#[cfg(feature = "ai")]
fn parse_inline_code<'a>(text: &'a str, theme: &'a crate::tui::Theme) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let mut current = String::new();
    let mut in_code = false;

    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c == '`' {
            if in_code {
                // End of inline code
                spans.push(Span::styled(
                    format!(" {} ", current),
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ));
                current.clear();
                in_code = false;
            } else {
                // Start of inline code
                if !current.is_empty() {
                    spans.push(Span::styled(current.clone(), Style::default().fg(theme.text)));
                    current.clear();
                }
                in_code = true;
            }
        } else {
            current.push(c);
        }
        i += 1;
    }

    // Handle remaining text
    if !current.is_empty() {
        if in_code {
            // Unclosed code block
            spans.push(Span::styled(format!("`{}", current), Style::default().fg(theme.text)));
        } else {
            spans.push(Span::styled(current, Style::default().fg(theme.text)));
        }
    }

    if spans.is_empty() {
        spans.push(Span::raw(""));
    }

    spans
}

/// Render inline markdown (for bullet point content).
#[cfg(feature = "ai")]
fn render_inline_markdown(text: &str, _theme: &crate::tui::Theme) -> String {
    // Simple passthrough for now - could add bold/italic support
    text.to_string()
}

/// Draw the AI setup screen for managing models.
#[cfg(feature = "ai")]
fn draw_ai_setup_screen(frame: &mut Frame, app: &App) {
    let theme = &app.theme;
    let area = frame.area();

    // Layout: title, model list, input, footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Model list
            Constraint::Length(3), // Pull input
            Constraint::Length(2), // Footer
        ])
        .split(area);

    // Title with Ollama status
    let ollama_status = if app.ai_models_loading {
        "Loading..."
    } else if app.ai_models.is_empty() {
        "No models"
    } else {
        "Connected"
    };
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            " AI Models ",
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("[Ollama: {}]", ollama_status), Style::default().fg(theme.text_dim)),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.primary)));
    frame.render_widget(title, chunks[0]);

    // Model list
    let mut lines = Vec::new();

    if app.ai_models_loading {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Loading models...",
            Style::default().fg(theme.text_dim),
        )));
    } else if app.ai_models.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " No models installed",
            Style::default().fg(theme.warning),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Type a model name below and press Enter to download",
            Style::default().fg(theme.text_dim),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(" Popular models:", Style::default().fg(theme.accent))));
        lines.push(Line::from(Span::styled(
            "   llama3.2      - Meta's latest LLaMA (small, fast)",
            Style::default().fg(theme.text_muted),
        )));
        lines.push(Line::from(Span::styled(
            "   qwen2.5:3b    - Alibaba Qwen (efficient)",
            Style::default().fg(theme.text_muted),
        )));
        lines.push(Line::from(Span::styled(
            "   mistral       - Mistral 7B (balanced)",
            Style::default().fg(theme.text_muted),
        )));
        lines.push(Line::from(Span::styled(
            "   codellama     - Code-focused LLaMA",
            Style::default().fg(theme.text_muted),
        )));
        lines.push(Line::from(Span::styled(
            "   phi3          - Microsoft Phi-3 (compact)",
            Style::default().fg(theme.text_muted),
        )));
    } else {
        // Header
        lines.push(Line::from(vec![
            Span::styled(
                format!("   {:<30}", "Model"),
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{:>10}", "Size"),
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(Span::styled("─".repeat(45), Style::default().fg(theme.border))));

        // Model entries
        for (i, model) in app.ai_models.iter().enumerate() {
            let is_selected = i == app.ai_model_selected;
            let current_model = std::env::var("OLLAMA_MODEL").unwrap_or_default();
            let is_active = model.name == current_model;
            let is_pending_delete = app.ai_delete_pending.as_ref() == Some(&model.name);

            let prefix = if is_pending_delete {
                " X "
            } else if is_selected {
                " > "
            } else {
                "   "
            };
            let active_marker = if is_active { " *" } else { "" };

            let size_str = format_size(model.size);

            // Style: red if pending delete, otherwise normal selection
            let style = if is_pending_delete {
                Style::default().fg(theme.error).add_modifier(Modifier::BOLD).bg(theme.selected_bg)
            } else if is_selected {
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD).bg(theme.selected_bg)
            } else {
                Style::default().fg(theme.text)
            };

            let active_style = if is_pending_delete {
                Style::default().fg(theme.error).add_modifier(Modifier::BOLD)
            } else if is_active {
                Style::default().fg(theme.success)
            } else {
                style
            };

            let prefix_style = if is_pending_delete {
                Style::default().fg(theme.error)
            } else {
                Style::default().fg(theme.accent)
            };

            lines.push(Line::from(vec![
                Span::styled(prefix, prefix_style),
                Span::styled(format!("{:<30}", model.name), active_style),
                Span::styled(format!("{:>10}", size_str), Style::default().fg(theme.text_dim)),
                Span::styled(active_marker, Style::default().fg(theme.success)),
            ]));
        }

        // Show pull progress if downloading
        if let Some(ref progress) = app.ai_pull_progress {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("  {}", progress),
                Style::default().fg(theme.warning),
            )));
        }
    }

    let model_list = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(" Installed Models ")
            .title_style(Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD))
            .padding(Padding::horizontal(1)),
    );
    frame.render_widget(model_list, chunks[1]);

    // Pull input
    let input = Paragraph::new(Line::from(vec![
        Span::styled(" Pull: ", Style::default().fg(theme.secondary)),
        Span::styled(&app.ai_model_input, Style::default().fg(theme.text)),
        Span::styled("│", Style::default().fg(theme.border)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent))
            .title(" Download New Model ")
            .title_style(Style::default().fg(theme.accent)),
    );
    frame.render_widget(input, chunks[2]);

    // Footer with hints - changes based on state
    let footer_content = if app.ai_delete_pending.is_some() {
        // Delete confirmation mode
        vec![
            Span::styled("d", Style::default().fg(theme.error).add_modifier(Modifier::BOLD)),
            Span::styled(" CONFIRM DELETE  ", Style::default().fg(theme.error)),
            Span::styled("Esc", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(" cancel", Style::default().fg(theme.text_dim)),
        ]
    } else if app.ai_pull_progress.is_some() {
        // Downloading mode
        vec![
            Span::styled("Downloading... ", Style::default().fg(theme.warning)),
            Span::styled("Please wait", Style::default().fg(theme.text_dim)),
        ]
    } else {
        // Normal mode
        vec![
            Span::styled("Enter", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(" use/pull  ", Style::default().fg(theme.text_dim)),
            Span::styled("d", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(" delete  ", Style::default().fg(theme.text_dim)),
            Span::styled("r", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(" refresh  ", Style::default().fg(theme.text_dim)),
            Span::styled("Esc", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(" close", Style::default().fg(theme.text_dim)),
        ]
    };

    let footer = Paragraph::new(Line::from(footer_content)).alignment(Alignment::Center);
    frame.render_widget(footer, chunks[3]);
}

/// Format file size in human readable form.
#[cfg(feature = "ai")]
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}

/// Draw the trust confirmation dialog.
fn draw_trust_confirmation(frame: &mut Frame, app: &App) {
    use crate::core::trust_warning_message;

    let theme = &app.theme;
    let area = frame.area();

    // Center the dialog
    let dialog_width = 60.min(area.width.saturating_sub(4));
    let dialog_height = 12;
    let dialog_area = Rect::new(
        (area.width.saturating_sub(dialog_width)) / 2,
        (area.height.saturating_sub(dialog_height)) / 2,
        dialog_width,
        dialog_height,
    );

    // Clear background
    frame.render_widget(Clear, area);

    // Build warning message
    let warning_lines = trust_warning_message(&app.cwd);
    let mut content: Vec<Line> = vec![Line::from("")];

    for line in warning_lines {
        content
            .push(Line::from(Span::styled(format!(" {}", line), Style::default().fg(theme.text))));
    }

    content.push(Line::from(""));

    // Options
    let yes_style = if app.trust_selected == 0 {
        Style::default().fg(theme.background).bg(theme.success).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_dim)
    };

    let no_style = if app.trust_selected == 1 {
        Style::default().fg(theme.background).bg(theme.error).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text_dim)
    };

    content.push(Line::from(vec![
        Span::raw("   "),
        Span::styled(if app.trust_selected == 0 { "> " } else { "  " }, yes_style),
        Span::styled("Yes, proceed", yes_style),
        Span::raw("     "),
        Span::styled(if app.trust_selected == 1 { "> " } else { "  " }, no_style),
        Span::styled("No, exit", no_style),
    ]));

    content.push(Line::from(""));
    content.push(Line::from(Span::styled(
        " Use arrow keys to select, Enter to confirm",
        Style::default().fg(theme.text_muted),
    )));

    let dialog = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.warning))
            .title(" Trust This Folder? ")
            .title_style(Style::default().fg(theme.warning).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(theme.background)),
    );

    frame.render_widget(dialog, dialog_area);
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
