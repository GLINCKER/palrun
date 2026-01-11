//! Application state and lifecycle management.
//!
//! This module contains the core `App` struct that holds all application state
//! and coordinates between the TUI, command registry, and scanners.

use std::collections::HashSet;
use std::path::PathBuf;

use crate::core::{
    send_notification, BackgroundEvent, BackgroundManager, CaptureManager, ChainExecutor,
    ChainStepStatus, Command, CommandChain, CommandContext, CommandRegistry, Config, ContextFilter,
    HistoryManager, ParsedQuery,
};
use crate::tui::Theme;

#[cfg(feature = "git")]
use crate::git::GitInfo;

/// Main application state.
///
/// The `App` struct is the central state container for Palrun. It manages:
/// - Current search input
/// - Command registry and filtered results
/// - Selection state
/// - Application lifecycle (running/quit)
#[derive(Debug)]
pub struct App {
    /// Current search input from the user
    pub input: String,

    /// Cursor position in the input field
    pub cursor_position: usize,

    /// The command registry containing all discovered commands
    pub registry: CommandRegistry,

    /// Currently filtered/matched commands (indices into registry)
    pub filtered_commands: Vec<usize>,

    /// Currently selected command index (in filtered list)
    pub selected: usize,

    /// Whether the application should quit
    pub should_quit: bool,

    /// Whether a command was selected (vs cancelled)
    pub command_selected: bool,

    /// Current working directory
    pub cwd: PathBuf,

    /// Application configuration
    pub config: Config,

    /// Current mode of the application
    pub mode: AppMode,

    /// Status message to display (if any)
    pub status_message: Option<String>,

    /// Context for filtering and proximity scoring
    pub context: CommandContext,

    /// Whether context-aware filtering is enabled
    pub context_aware: bool,

    /// Git repository information (if in a git repo)
    #[cfg(feature = "git")]
    pub git_info: Option<GitInfo>,

    /// Last executed command output
    pub last_output: Option<CommandOutput>,

    /// Scroll offset for output view
    pub output_scroll: usize,

    /// Multi-selected command indices (in filtered list)
    pub selected_commands: HashSet<usize>,

    /// Whether multi-select mode is active
    pub multi_select_mode: bool,

    /// Background process manager
    pub background_manager: Option<BackgroundManager>,

    /// Output capture manager
    pub capture_manager: Option<CaptureManager>,

    /// Current UI theme
    pub theme: Theme,

    /// Active filters display string (from parsed query)
    pub active_filters: Option<String>,

    /// Command history and favorites manager
    pub history_manager: Option<HistoryManager>,

    /// Shell command for pass-through mode
    pub pass_through_command: Option<String>,

    /// Command palette search input
    pub palette_input: String,

    /// Selected item in command palette
    pub palette_selected: usize,

    /// Selected item in context menu
    pub context_menu_selected: usize,

    /// Rotating tip index for status bar
    pub tip_index: usize,

    /// Directory listing for shell command preview
    pub dir_listing: Option<Vec<DirEntry>>,

    /// Selected index in directory listing
    pub dir_selected: usize,

    /// Ghost text autocomplete suggestion
    pub ghost_text: Option<String>,

    /// Slash commands shown when input starts with /
    pub slash_commands: Option<Vec<SlashCommand>>,

    /// Selected slash command index
    pub slash_selected: usize,

    /// Whether network is offline (for AI status)
    pub is_offline: bool,

    /// AI provider status message
    pub ai_status: Option<String>,

    /// Degradation manager for graceful fallbacks
    pub degradation: crate::core::DegradationManager,

    /// Offline operation queue manager
    pub offline_manager: crate::core::OfflineManager,

    /// Resilience manager for retry and circuit breaker logic
    pub resilience: crate::core::ResilienceManager,
}

/// A slash command entry
#[derive(Debug, Clone)]
pub struct SlashCommand {
    pub name: String,
    pub description: String,
}

impl SlashCommand {
    fn new(name: &str, description: &str) -> Self {
        Self { name: name.to_string(), description: description.to_string() }
    }
}

/// Get available slash commands
fn get_slash_commands() -> Vec<SlashCommand> {
    vec![
        SlashCommand::new("/help", "Show help screen"),
        SlashCommand::new("/history", "View command history"),
        SlashCommand::new("/analytics", "View usage analytics"),
        SlashCommand::new("/favorites", "Show favorite commands"),
        SlashCommand::new("/settings", "Open settings"),
        SlashCommand::new("/theme", "Change color theme"),
        SlashCommand::new("/quit", "Exit palrun"),
    ]
}

/// A directory entry for preview
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub is_dir: bool,
}

/// Output from a command execution.
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Name of the command that was executed
    pub command_name: String,
    /// The actual command string that was run
    pub command_str: String,
    /// Standard output from the command
    pub stdout: String,
    /// Standard error from the command
    pub stderr: String,
    /// Exit code from the command
    pub exit_code: Option<i32>,
    /// Whether the command succeeded
    pub success: bool,
}

/// Application modes
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AppMode {
    /// Normal command palette mode
    #[default]
    Normal,

    /// AI input mode (natural language)
    #[cfg(feature = "ai")]
    AiInput,

    /// Viewing command details/preview
    Preview,

    /// Confirmation dialog
    Confirm,

    /// Command is currently executing
    Executing(String),

    /// Showing execution result
    ExecutionResult,

    /// Showing help screen with keyboard shortcuts
    Help,

    /// Showing command history
    History,

    /// Showing usage analytics
    Analytics,

    /// Pass-through mode: asking user to confirm running shell command
    PassThrough,

    /// Command palette (quick actions)
    Palette,

    /// Context menu for selected command
    ContextMenu,
}

impl App {
    /// Create a new application instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the current working directory cannot be determined.
    pub fn new() -> anyhow::Result<Self> {
        let cwd = std::env::current_dir()?;
        let config = Config::load()?;
        let context = CommandContext::new(&cwd, &cwd);

        #[cfg(feature = "git")]
        let git_info = crate::git::current_git_info();

        let background_manager = BackgroundManager::new().ok();
        let capture_manager = CaptureManager::new().ok();
        let history_manager = HistoryManager::new().ok();

        // Resolve theme from config
        let theme = Self::resolve_theme(&config);

        Ok(Self {
            input: String::new(),
            cursor_position: 0,
            registry: CommandRegistry::new(),
            filtered_commands: Vec::new(),
            selected: 0,
            should_quit: false,
            command_selected: false,
            cwd,
            config,
            mode: AppMode::default(),
            status_message: None,
            context,
            context_aware: true,
            #[cfg(feature = "git")]
            git_info,
            last_output: None,
            output_scroll: 0,
            selected_commands: HashSet::new(),
            multi_select_mode: false,
            background_manager,
            capture_manager,
            theme,
            active_filters: None,
            history_manager,
            pass_through_command: None,
            palette_input: String::new(),
            palette_selected: 0,
            context_menu_selected: 0,
            tip_index: 0,
            dir_listing: None,
            dir_selected: 0,
            ghost_text: None,
            slash_commands: None,
            slash_selected: 0,
            is_offline: false,
            ai_status: None,
            degradation: crate::core::DegradationManager::new(),
            offline_manager: crate::core::OfflineManager::new(),
            resilience: crate::core::ResilienceManager::new(),
        })
    }

    /// Resolve theme from configuration.
    fn resolve_theme(config: &Config) -> Theme {
        use crate::tui::parse_hex_color;

        // Get base theme by name
        let mut theme = Theme::by_name(&config.ui.theme).unwrap_or_default();

        // Apply custom color overrides if present
        if let Some(ref custom) = config.ui.custom_colors {
            if let Some(ref hex) = custom.primary {
                if let Some(color) = parse_hex_color(hex) {
                    theme.primary = color;
                }
            }
            if let Some(ref hex) = custom.secondary {
                if let Some(color) = parse_hex_color(hex) {
                    theme.secondary = color;
                }
            }
            if let Some(ref hex) = custom.accent {
                if let Some(color) = parse_hex_color(hex) {
                    theme.accent = color;
                }
            }
            if let Some(ref hex) = custom.highlight {
                if let Some(color) = parse_hex_color(hex) {
                    theme.highlight = color;
                }
            }
            if let Some(ref hex) = custom.text {
                if let Some(color) = parse_hex_color(hex) {
                    theme.text = color;
                }
            }
            if let Some(ref hex) = custom.text_dim {
                if let Some(color) = parse_hex_color(hex) {
                    theme.text_dim = color;
                }
            }
            if let Some(ref hex) = custom.text_muted {
                if let Some(color) = parse_hex_color(hex) {
                    theme.text_muted = color;
                }
            }
            if let Some(ref hex) = custom.background {
                if let Some(color) = parse_hex_color(hex) {
                    theme.background = color;
                }
            }
            if let Some(ref hex) = custom.selected_bg {
                if let Some(color) = parse_hex_color(hex) {
                    theme.selected_bg = color;
                }
            }
            if let Some(ref hex) = custom.border {
                if let Some(color) = parse_hex_color(hex) {
                    theme.border = color;
                }
            }
            if let Some(ref hex) = custom.success {
                if let Some(color) = parse_hex_color(hex) {
                    theme.success = color;
                }
            }
            if let Some(ref hex) = custom.warning {
                if let Some(color) = parse_hex_color(hex) {
                    theme.warning = color;
                }
            }
            if let Some(ref hex) = custom.error {
                if let Some(color) = parse_hex_color(hex) {
                    theme.error = color;
                }
            }
        }

        theme
    }

    /// Create a new application instance for testing (with minimal setup).
    #[cfg(test)]
    pub fn new_test() -> Self {
        let cwd = PathBuf::from("/tmp");
        Self {
            input: String::new(),
            cursor_position: 0,
            registry: CommandRegistry::new(),
            filtered_commands: Vec::new(),
            selected: 0,
            should_quit: false,
            command_selected: false,
            cwd: cwd.clone(),
            config: Config::default(),
            mode: AppMode::default(),
            status_message: None,
            context: CommandContext::new(&cwd, &cwd),
            context_aware: true,
            #[cfg(feature = "git")]
            git_info: None,
            last_output: None,
            output_scroll: 0,
            selected_commands: HashSet::new(),
            multi_select_mode: false,
            background_manager: None,
            capture_manager: None,
            theme: Theme::default(),
            active_filters: None,
            history_manager: None,
            pass_through_command: None,
            palette_input: String::new(),
            palette_selected: 0,
            context_menu_selected: 0,
            tip_index: 0,
            dir_listing: None,
            dir_selected: 0,
            ghost_text: None,
            slash_commands: None,
            slash_selected: 0,
            is_offline: false,
            ai_status: None,
            degradation: crate::core::DegradationManager::new(),
            offline_manager: crate::core::OfflineManager::new(),
            resilience: crate::core::ResilienceManager::new(),
        }
    }

    /// Handle a character input (typing in search field).
    pub fn enter_char(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
        self.update_filtered_commands();
    }

    /// Delete the character before the cursor (backspace).
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input.remove(self.cursor_position);
            self.update_filtered_commands();
        }
    }

    /// Delete the character at the cursor (delete key).
    pub fn delete_char_forward(&mut self) {
        if self.cursor_position < self.input.len() {
            self.input.remove(self.cursor_position);
            self.update_filtered_commands();
        }
    }

    /// Move cursor left.
    pub fn move_cursor_left(&mut self) {
        self.cursor_position = self.cursor_position.saturating_sub(1);
    }

    /// Move cursor right.
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }

    /// Move cursor to start of input.
    pub fn move_cursor_start(&mut self) {
        self.cursor_position = 0;
    }

    /// Move cursor to end of input.
    pub fn move_cursor_end(&mut self) {
        self.cursor_position = self.input.len();
    }

    /// Clear the current input.
    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
        self.update_filtered_commands();
    }

    /// Move selection up in the command list.
    pub fn select_previous(&mut self) {
        if !self.filtered_commands.is_empty() {
            self.selected = self.selected.saturating_sub(1);
        }
    }

    /// Move selection down in the command list.
    pub fn select_next(&mut self) {
        if !self.filtered_commands.is_empty() {
            self.selected = (self.selected + 1).min(self.filtered_commands.len() - 1);
        }
    }

    /// Move selection to the first command.
    pub fn select_first(&mut self) {
        self.selected = 0;
    }

    /// Move selection to the last command.
    pub fn select_last(&mut self) {
        if !self.filtered_commands.is_empty() {
            self.selected = self.filtered_commands.len() - 1;
        }
    }

    /// Get the currently selected command (if any).
    pub fn get_selected_command(&self) -> Option<&Command> {
        self.filtered_commands.get(self.selected).and_then(|&idx| self.registry.get_by_index(idx))
    }

    /// Update filtered commands based on current search input.
    ///
    /// This performs fuzzy matching against all commands in the registry,
    /// with optional context-aware proximity scoring. Supports filter syntax:
    /// - `#tag` - Filter by tag
    /// - `source:npm` - Filter by source type
    /// - `@workspace` - Filter by workspace name
    pub fn update_filtered_commands(&mut self) {
        // Parse the input for filters
        let query = ParsedQuery::parse(&self.input);

        // Update active filters display
        self.active_filters = query.filter_display();

        // Get base filtered results using fuzzy search on the pattern
        let mut candidates = if self.context_aware {
            self.registry.search_filtered(&query.pattern, &self.context)
        } else {
            self.registry.search(&query.pattern)
        };

        // Apply additional filters if present
        if query.has_filters() {
            candidates.retain(|&idx| {
                if let Some(cmd) = self.registry.get_by_index(idx) {
                    query.matches(cmd)
                } else {
                    false
                }
            });
        }

        self.filtered_commands = candidates;

        // Reset selection if it's now out of bounds
        if self.selected >= self.filtered_commands.len() {
            self.selected = self.filtered_commands.len().saturating_sub(1);
        }

        // Update directory listing for shell command preview
        self.update_dir_listing();
    }

    /// Update directory listing and slash commands based on input.
    fn update_dir_listing(&mut self) {
        let input = self.input.trim();

        // Check for slash commands first
        if input.starts_with('/') {
            self.dir_listing = None;
            let all_commands = get_slash_commands();
            let filtered: Vec<SlashCommand> = if input == "/" {
                all_commands
            } else {
                all_commands
                    .into_iter()
                    .filter(|c| c.name.to_lowercase().starts_with(&input.to_lowercase()))
                    .collect()
            };
            self.slash_commands = if filtered.is_empty() { None } else { Some(filtered) };
            self.slash_selected = 0;
            return;
        }

        // Clear slash commands if not typing /
        self.slash_commands = None;

        // Check if input looks like a shell directory command
        if input.starts_with("cd ") || input == "cd" || input.starts_with("ls") {
            let path = if input.starts_with("cd ") {
                let path_str = input.strip_prefix("cd ").unwrap_or("").trim();
                self.resolve_path(path_str)
            } else if input.starts_with("ls ") {
                let path_str = input.strip_prefix("ls ").unwrap_or("").trim();
                if path_str.starts_with('-') {
                    // ls with flags, use current dir
                    self.cwd.clone()
                } else {
                    self.resolve_path(path_str)
                }
            } else {
                self.cwd.clone()
            };

            // Get directory listing
            self.dir_listing = self.list_directory(&path);
        } else {
            self.dir_listing = None;
        }
    }

    /// Resolve a path string to an absolute path.
    fn resolve_path(&self, path_str: &str) -> PathBuf {
        if path_str.is_empty() || path_str == "~" {
            dirs::home_dir().unwrap_or_else(|| self.cwd.clone())
        } else if path_str.starts_with("~/") {
            dirs::home_dir()
                .map(|h| h.join(&path_str[2..]))
                .unwrap_or_else(|| PathBuf::from(path_str))
        } else if path_str.starts_with('/') {
            PathBuf::from(path_str)
        } else {
            self.cwd.join(path_str)
        }
    }

    /// List directory entries.
    fn list_directory(&mut self, path: &PathBuf) -> Option<Vec<DirEntry>> {
        let dir = if path.is_dir() {
            path.clone()
        } else {
            // If path is partial, use parent
            path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| self.cwd.clone())
        };

        match std::fs::read_dir(&dir) {
            Ok(entries) => {
                let mut items: Vec<DirEntry> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
                    .map(|e| DirEntry {
                        name: e.file_name().to_string_lossy().to_string(),
                        is_dir: e.file_type().map(|t| t.is_dir()).unwrap_or(false),
                    })
                    .collect();

                // Sort: directories first, then alphabetically
                items.sort_by(|a, b| match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                });

                // Filter by partial input if there's a path fragment
                let last_segment = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if !last_segment.is_empty() && !path.is_dir() {
                    items.retain(|e| {
                        e.name.to_lowercase().starts_with(&last_segment.to_lowercase())
                    });
                }

                // Update ghost text with first match
                self.ghost_text = items.first().map(|e| e.name.clone());

                Some(items)
            }
            Err(_) => None,
        }
    }

    /// Check if we're in directory browsing mode.
    pub fn is_dir_browsing(&self) -> bool {
        self.dir_listing.as_ref().map(|d| !d.is_empty()).unwrap_or(false)
            && self.filtered_commands.is_empty()
    }

    /// Check if we're in slash command browsing mode.
    pub fn is_slash_browsing(&self) -> bool {
        self.slash_commands.as_ref().map(|c| !c.is_empty()).unwrap_or(false)
    }

    /// Navigate to next slash command.
    pub fn select_slash_next(&mut self) {
        if let Some(ref commands) = self.slash_commands {
            if !commands.is_empty() {
                self.slash_selected = (self.slash_selected + 1).min(commands.len() - 1);
            }
        }
    }

    /// Navigate to previous slash command.
    pub fn select_slash_previous(&mut self) {
        if self.slash_commands.is_some() {
            self.slash_selected = self.slash_selected.saturating_sub(1);
        }
    }

    /// Try to execute a slash command. Returns true if handled.
    pub fn try_slash_command(&mut self) -> bool {
        let input = self.input.trim().to_string();

        // Check if we have a slash command selected
        if let Some(ref commands) = self.slash_commands {
            if let Some(cmd) = commands.get(self.slash_selected) {
                let cmd_name = cmd.name.clone();
                self.execute_slash_command(&cmd_name);
                return true;
            }
        }

        // Check for exact slash command match
        if input.starts_with('/') {
            let matched = match input.as_str() {
                "/help" | "/h" | "/?" => {
                    self.show_help();
                    true
                }
                "/history" => {
                    self.show_history();
                    true
                }
                "/analytics" | "/stats" => {
                    self.show_analytics();
                    true
                }
                "/quit" | "/q" | "/exit" => {
                    self.quit();
                    true
                }
                "/favorites" | "/fav" => {
                    // TODO: Show favorites filter
                    self.set_status("Favorites: coming soon!");
                    true
                }
                "/settings" => {
                    // TODO: Show settings
                    self.set_status("Settings: coming soon!");
                    true
                }
                "/theme" => {
                    // TODO: Theme picker
                    self.set_status("Theme picker: coming soon!");
                    true
                }
                _ => false,
            };

            if matched {
                self.input.clear();
                self.cursor_position = 0;
                self.slash_commands = None;
                return true;
            }
        }

        false
    }

    /// Execute a specific slash command.
    fn execute_slash_command(&mut self, cmd: &str) {
        match cmd {
            "/help" => self.show_help(),
            "/history" => self.show_history(),
            "/analytics" => self.show_analytics(),
            "/quit" => self.quit(),
            "/favorites" => self.set_status("Favorites: coming soon!"),
            "/settings" => self.set_status("Settings: coming soon!"),
            "/theme" => self.set_status("Theme picker: coming soon!"),
            _ => {}
        }
        self.input.clear();
        self.cursor_position = 0;
        self.slash_commands = None;
    }

    /// Navigate to next directory entry.
    pub fn select_dir_next(&mut self) {
        if let Some(ref entries) = self.dir_listing {
            if !entries.is_empty() {
                self.dir_selected = (self.dir_selected + 1).min(entries.len() - 1);
                self.ghost_text = entries.get(self.dir_selected).map(|e| e.name.clone());
            }
        }
    }

    /// Navigate to previous directory entry.
    pub fn select_dir_previous(&mut self) {
        if self.dir_listing.is_some() {
            self.dir_selected = self.dir_selected.saturating_sub(1);
            if let Some(ref entries) = self.dir_listing {
                self.ghost_text = entries.get(self.dir_selected).map(|e| e.name.clone());
            }
        }
    }

    /// Complete input with selected directory (Tab completion).
    pub fn complete_dir_selection(&mut self) {
        if let Some(ref entries) = self.dir_listing {
            if let Some(entry) = entries.get(self.dir_selected) {
                let input = self.input.trim();

                // Extract the base command (cd or ls) and existing path
                let (cmd, existing_path) = if input.starts_with("cd ") {
                    ("cd ", input.strip_prefix("cd ").unwrap_or("").trim())
                } else if input.starts_with("ls ") {
                    ("ls ", input.strip_prefix("ls ").unwrap_or("").trim())
                } else if input == "cd" {
                    ("cd ", "")
                } else if input == "ls" {
                    ("ls ", "")
                } else {
                    return;
                };

                // Build new path
                let base_path = if existing_path.is_empty() {
                    String::new()
                } else {
                    // Get directory part of existing path
                    let path = self.resolve_path(existing_path);
                    if path.is_dir() {
                        format!("{}/", existing_path.trim_end_matches('/'))
                    } else if let Some(parent) = std::path::Path::new(existing_path).parent() {
                        let parent_str = parent.to_string_lossy();
                        if parent_str.is_empty() {
                            String::new()
                        } else {
                            format!("{}/", parent_str)
                        }
                    } else {
                        String::new()
                    }
                };

                let new_input = format!("{}{}{}", cmd, base_path, entry.name);
                self.input = if entry.is_dir { format!("{}/", new_input) } else { new_input };
                self.cursor_position = self.input.len();
                self.dir_selected = 0;
                self.update_filtered_commands();
            }
        }
    }

    /// Execute with selected directory.
    pub fn execute_dir_selection(&mut self) {
        // Clone entry info to avoid borrow issues
        let entry_info = self
            .dir_listing
            .as_ref()
            .and_then(|entries| entries.get(self.dir_selected))
            .map(|e| (e.name.clone(), e.is_dir));

        if let Some((_, is_dir)) = entry_info {
            // Complete first, then execute
            self.complete_dir_selection();

            // If it's a directory and we're doing cd, execute it
            if is_dir && self.input.trim().starts_with("cd ") {
                let input = self.input.trim().to_string();
                self.handle_cd_command(&input);
                self.input.clear();
                self.cursor_position = 0;
                self.dir_listing = None;
                self.dir_selected = 0;
                self.ghost_text = None;
            } else if self.input.trim().starts_with("ls") {
                // For ls, execute the command
                let input = self.input.trim().to_string();
                self.execute_shell_command(&input);
                self.input.clear();
                self.cursor_position = 0;
                self.dir_listing = None;
                self.dir_selected = 0;
                self.ghost_text = None;
            }
        }
    }

    /// Toggle context-aware filtering.
    pub fn toggle_context_aware(&mut self) {
        self.context_aware = !self.context_aware;
        self.update_filtered_commands();
        self.set_status(if self.context_aware {
            "Context-aware filtering enabled"
        } else {
            "Context-aware filtering disabled"
        });
    }

    /// Set the context filter.
    pub fn set_context_filter(&mut self, filter: ContextFilter) {
        self.context = self.context.clone().with_filter(filter);
        self.update_filtered_commands();
    }

    /// Get location indicator for a command.
    pub fn get_location_indicator(&self, command: &Command) -> crate::core::LocationIndicator {
        crate::core::LocationIndicator::for_command(&self.context, command)
    }

    /// Set a status message to display temporarily.
    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    /// Clear the status message.
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Request the application to quit.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Perform periodic updates (called on tick).
    pub fn tick(&mut self) {
        // Future: Update file watchers, refresh commands, etc.
    }

    /// Refresh Git information.
    #[cfg(feature = "git")]
    pub fn refresh_git_info(&mut self) {
        self.git_info = crate::git::current_git_info();
    }

    /// Get the current branch name (if in a git repo).
    #[cfg(feature = "git")]
    pub fn current_branch(&self) -> Option<&str> {
        self.git_info.as_ref().and_then(|info| info.branch.as_deref())
    }

    /// Get git status string for display.
    #[cfg(feature = "git")]
    pub fn git_status_display(&self) -> Option<String> {
        self.git_info
            .as_ref()
            .map(|info| format!("{} {}", info.branch_display(), info.status_string()))
    }

    /// Initialize the application (scan for commands, etc.).
    pub fn initialize(&mut self) -> anyhow::Result<()> {
        // Scan for commands in the current project
        self.scan_project()?;

        // Load aliases from config
        self.load_aliases();

        // Update filtered list with all commands initially
        self.update_filtered_commands();

        // Check AI availability
        self.update_ai_status();

        Ok(())
    }

    /// Update AI status based on available providers.
    #[cfg(feature = "ai")]
    fn update_ai_status(&mut self) {
        // Check for Claude API key
        if std::env::var("ANTHROPIC_API_KEY").is_ok() {
            self.ai_status = Some("Claude AI".to_string());
            return;
        }

        // Check for OpenAI API key
        if std::env::var("OPENAI_API_KEY").is_ok() {
            self.ai_status = Some("OpenAI".to_string());
            return;
        }

        // Check for Ollama (local) - this is a simple check, actual availability
        // requires an async call which we'll do in the TUI loop
        let ollama_host =
            std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string());
        if ollama_host.contains("localhost") || ollama_host.contains("127.0.0.1") {
            self.ai_status = Some("Ollama (local)".to_string());
            return;
        }

        // No AI provider configured
        self.ai_status = None;
    }

    /// Fallback when AI feature is disabled.
    #[cfg(not(feature = "ai"))]
    fn update_ai_status(&mut self) {
        self.ai_status = None;
    }

    /// Load aliases from config into the registry.
    fn load_aliases(&mut self) {
        for alias in &self.config.aliases {
            let cmd = Command::from_alias(alias);
            self.registry.add(cmd);
        }
    }

    /// Scan the current project for commands.
    fn scan_project(&mut self) -> anyhow::Result<()> {
        use crate::scanner::ProjectScanner;

        let scanner = ProjectScanner::new(&self.cwd);
        let commands = scanner.scan()?;

        for cmd in commands {
            self.registry.add(cmd);
        }

        Ok(())
    }

    /// Execute the currently selected command and capture output.
    ///
    /// Supports command chaining with `&&`, `||`, and `;` operators.
    pub fn execute_selected_command(&mut self) {
        if let Some(cmd) = self.get_selected_command().cloned() {
            self.mode = AppMode::Executing(cmd.name.clone());

            // Check if this is a chained command
            let chain = CommandChain::parse(&cmd.command);

            if chain.is_simple() {
                // Simple command - execute directly
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&cmd.command)
                    .current_dir(cmd.working_dir.as_ref().unwrap_or(&self.cwd))
                    .output();

                match output {
                    Ok(result) => {
                        self.last_output = Some(CommandOutput {
                            command_name: cmd.name.clone(),
                            command_str: cmd.command.clone(),
                            stdout: String::from_utf8_lossy(&result.stdout).to_string(),
                            stderr: String::from_utf8_lossy(&result.stderr).to_string(),
                            exit_code: result.status.code(),
                            success: result.status.success(),
                        });
                    }
                    Err(e) => {
                        self.last_output = Some(CommandOutput {
                            command_name: cmd.name.clone(),
                            command_str: cmd.command.clone(),
                            stdout: String::new(),
                            stderr: format!("Failed to execute: {}", e),
                            exit_code: None,
                            success: false,
                        });
                    }
                }
            } else {
                // Chained command - use ChainExecutor
                let working_dir = cmd
                    .working_dir
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| self.cwd.display().to_string());

                let executor = ChainExecutor::new().working_dir(working_dir);

                match executor.execute(&chain) {
                    Ok(result) => {
                        // Build output showing chain progress
                        let mut stdout = String::new();
                        let mut stderr = String::new();

                        for step in &result.steps {
                            // Show step status
                            let status_icon = match &step.status {
                                ChainStepStatus::Success => "✓",
                                ChainStepStatus::Failed(_) => "✗",
                                ChainStepStatus::Skipped => "○",
                                _ => "→",
                            };

                            stdout.push_str(&format!(
                                "[{}] {} ({:.2?})\n",
                                status_icon, step.command, step.duration
                            ));

                            if !step.stdout.is_empty() {
                                stdout.push_str(&step.stdout);
                                if !step.stdout.ends_with('\n') {
                                    stdout.push('\n');
                                }
                            }

                            if !step.stderr.is_empty() {
                                stderr.push_str(&format!("--- {} ---\n", step.command));
                                stderr.push_str(&step.stderr);
                                if !step.stderr.ends_with('\n') {
                                    stderr.push('\n');
                                }
                            }
                        }

                        // Summary line
                        stdout.push_str(&format!(
                            "\n━━━ Chain complete: {}/{} succeeded ({:.2?}) ━━━\n",
                            result.success_count(),
                            result.steps.len(),
                            result.total_duration
                        ));

                        let exit_code = if result.success { Some(0) } else { Some(1) };

                        self.last_output = Some(CommandOutput {
                            command_name: cmd.name.clone(),
                            command_str: cmd.command.clone(),
                            stdout,
                            stderr,
                            exit_code,
                            success: result.success,
                        });
                    }
                    Err(e) => {
                        self.last_output = Some(CommandOutput {
                            command_name: cmd.name.clone(),
                            command_str: cmd.command.clone(),
                            stdout: String::new(),
                            stderr: format!("Failed to execute chain: {}", e),
                            exit_code: None,
                            success: false,
                        });
                    }
                }
            }

            self.output_scroll = 0; // Reset scroll for new output
            self.mode = AppMode::ExecutionResult;

            // Capture output for replay
            self.capture_last_output();
        }
    }

    /// Capture the last command output to the capture manager.
    fn capture_last_output(&mut self) {
        if let (Some(output), Some(ref mut manager)) =
            (&self.last_output, self.capture_manager.as_mut())
        {
            let _ = manager.capture(
                &output.command_name,
                &output.command_str,
                &output.stdout,
                &output.stderr,
                std::time::Duration::from_secs(0), // TODO: Track actual duration
                output.exit_code,
            );
        }
    }

    /// Get the last captured output.
    pub fn get_last_capture(&self) -> Option<crate::core::CapturedOutput> {
        self.capture_manager.as_ref()?.get_latest()
    }

    /// Search captured outputs.
    pub fn search_captures(&self, pattern: &str) -> Vec<crate::core::SearchResult> {
        self.capture_manager.as_ref().map(|m| m.search(pattern)).unwrap_or_default()
    }

    /// Execute multiple selected commands in parallel.
    pub fn execute_parallel_commands(&mut self) {
        use crate::core::ParallelExecutor;

        let commands: Vec<_> = self.get_selected_commands().into_iter().cloned().collect();

        if commands.is_empty() {
            return;
        }

        let names: Vec<_> = commands.iter().map(|c| c.name.clone()).collect();
        self.mode = AppMode::Executing(format!("{} commands", commands.len()));

        let executor = ParallelExecutor::new();
        let result = executor.execute(commands);

        match result {
            Ok(parallel_result) => {
                // Combine all outputs
                let mut stdout = String::new();
                let mut stderr = String::new();
                let mut all_success = true;

                for (i, proc) in parallel_result.processes.iter().enumerate() {
                    // Add header for each command
                    stdout.push_str(&format!(
                        "━━━ {} ━━━\n",
                        names.get(i).unwrap_or(&"Command".to_string())
                    ));

                    if !proc.stdout.is_empty() {
                        stdout.push_str(&proc.stdout.join("\n"));
                        stdout.push('\n');
                    }

                    if !proc.stderr.is_empty() {
                        stderr.push_str(&format!(
                            "[{}] ",
                            names.get(i).unwrap_or(&"Command".to_string())
                        ));
                        stderr.push_str(&proc.stderr.join("\n"));
                        stderr.push('\n');
                    }

                    if !proc.status.is_success() {
                        all_success = false;
                    }
                }

                let exit_code = if all_success { Some(0) } else { Some(1) };

                self.last_output = Some(CommandOutput {
                    command_name: format!("{} commands", parallel_result.processes.len()),
                    command_str: names.join(" & "),
                    stdout,
                    stderr,
                    exit_code,
                    success: all_success,
                });
            }
            Err(e) => {
                self.last_output = Some(CommandOutput {
                    command_name: "Parallel execution".to_string(),
                    command_str: names.join(" & "),
                    stdout: String::new(),
                    stderr: format!("Failed to execute: {}", e),
                    exit_code: None,
                    success: false,
                });
            }
        }

        self.output_scroll = 0;
        self.mode = AppMode::ExecutionResult;
        self.multi_select_mode = false;
        self.selected_commands.clear();
    }

    /// Return to normal mode from execution result.
    pub fn dismiss_result(&mut self) {
        self.mode = AppMode::Normal;
        self.output_scroll = 0;
    }

    /// Check if we're showing execution result.
    pub fn is_showing_result(&self) -> bool {
        matches!(self.mode, AppMode::ExecutionResult)
    }

    // --- Multi-select methods ---

    /// Toggle multi-select mode.
    pub fn toggle_multi_select(&mut self) {
        self.multi_select_mode = !self.multi_select_mode;
        if !self.multi_select_mode {
            self.selected_commands.clear();
        }
        self.set_status(if self.multi_select_mode {
            "Multi-select enabled (Space to select, Enter to run)"
        } else {
            "Multi-select disabled"
        });
    }

    /// Toggle selection of the current command.
    pub fn toggle_current_selection(&mut self) {
        if self.filtered_commands.is_empty() {
            return;
        }

        let idx = self.selected;
        if self.selected_commands.contains(&idx) {
            self.selected_commands.remove(&idx);
        } else {
            self.selected_commands.insert(idx);
        }
    }

    /// Check if a command at index is multi-selected.
    pub fn is_selected(&self, idx: usize) -> bool {
        self.selected_commands.contains(&idx)
    }

    /// Clear all multi-selections.
    pub fn clear_selections(&mut self) {
        self.selected_commands.clear();
    }

    /// Get the selected commands for parallel execution.
    pub fn get_selected_commands(&self) -> Vec<&Command> {
        if self.selected_commands.is_empty() {
            // If nothing selected, use the current command
            self.get_selected_command().into_iter().collect()
        } else {
            self.selected_commands
                .iter()
                .filter_map(|&idx| {
                    self.filtered_commands
                        .get(idx)
                        .and_then(|&cmd_idx| self.registry.get_by_index(cmd_idx))
                })
                .collect()
        }
    }

    /// Get count of selected commands.
    pub fn selected_count(&self) -> usize {
        if self.selected_commands.is_empty() {
            usize::from(self.get_selected_command().is_some())
        } else {
            self.selected_commands.len()
        }
    }

    /// Select all visible commands.
    pub fn select_all(&mut self) {
        self.selected_commands = (0..self.filtered_commands.len()).collect();
    }

    // --- Background execution methods ---

    /// Execute the selected command in the background.
    pub fn execute_in_background(&mut self) {
        if let Some(cmd) = self.get_selected_command().cloned() {
            if let Some(ref manager) = self.background_manager {
                match manager.spawn(cmd.clone()) {
                    Ok(id) => {
                        self.set_status(format!(
                            "Started '{}' in background (ID: {})",
                            cmd.name, id
                        ));
                    }
                    Err(e) => {
                        self.set_status(format!("Failed to start background process: {}", e));
                    }
                }
            } else {
                self.set_status("Background execution not available");
            }
        }
    }

    /// Get the count of running background processes.
    pub fn background_count(&self) -> usize {
        self.background_manager.as_ref().map(|m| m.running_count()).unwrap_or(0)
    }

    /// Poll for background process events and handle notifications.
    pub fn poll_background_events(&mut self) {
        if let Some(ref manager) = self.background_manager {
            for event in manager.poll_events() {
                match event {
                    BackgroundEvent::Completed(id, status) => {
                        if let Some(process) = manager.get(id) {
                            send_notification(
                                &process.name,
                                &status,
                                process.duration.unwrap_or_default(),
                            );
                        }
                    }
                    BackgroundEvent::Started(_) => {}
                }
            }
        }
    }

    // --- Scroll methods ---

    /// Scroll output up by one line.
    pub fn scroll_output_up(&mut self) {
        self.output_scroll = self.output_scroll.saturating_sub(1);
    }

    /// Scroll output down by one line.
    pub fn scroll_output_down(&mut self) {
        if let Some(output) = &self.last_output {
            let total_lines = output.stdout.lines().count() + output.stderr.lines().count();
            // Allow scrolling but cap at reasonable amount
            if self.output_scroll < total_lines.saturating_sub(1) {
                self.output_scroll += 1;
            }
        }
    }

    /// Scroll output up by a page.
    pub fn scroll_output_page_up(&mut self) {
        self.output_scroll = self.output_scroll.saturating_sub(10);
    }

    /// Scroll output down by a page.
    pub fn scroll_output_page_down(&mut self) {
        if let Some(output) = &self.last_output {
            let total_lines = output.stdout.lines().count() + output.stderr.lines().count();
            self.output_scroll = (self.output_scroll + 10).min(total_lines.saturating_sub(1));
        }
    }

    /// Scroll to top of output.
    pub fn scroll_output_top(&mut self) {
        self.output_scroll = 0;
    }

    /// Scroll to bottom of output.
    pub fn scroll_output_bottom(&mut self) {
        if let Some(output) = &self.last_output {
            let total_lines = output.stdout.lines().count() + output.stderr.lines().count();
            self.output_scroll = total_lines.saturating_sub(1);
        }
    }

    // --- Help methods ---

    /// Show the help screen.
    pub fn show_help(&mut self) {
        self.mode = AppMode::Help;
    }

    /// Dismiss the help screen and return to normal mode.
    pub fn dismiss_help(&mut self) {
        self.mode = AppMode::Normal;
    }

    /// Check if help is currently shown.
    pub fn is_help_shown(&self) -> bool {
        matches!(self.mode, AppMode::Help)
    }

    // --- History view methods ---

    /// Show the history screen.
    pub fn show_history(&mut self) {
        self.mode = AppMode::History;
    }

    /// Dismiss the history screen and return to normal mode.
    pub fn dismiss_history(&mut self) {
        self.mode = AppMode::Normal;
    }

    /// Check if history is currently shown.
    pub fn is_history_shown(&self) -> bool {
        matches!(self.mode, AppMode::History)
    }

    /// Get recent history entries for display.
    pub fn get_recent_history(&self, limit: usize) -> Vec<&crate::core::HistoryEntry> {
        self.history_manager.as_ref().map(|m| m.get_recent(limit)).unwrap_or_default()
    }

    /// Get history entries sorted by frecency.
    pub fn get_frecency_history(&self, limit: usize) -> Vec<&crate::core::HistoryEntry> {
        self.history_manager.as_ref().map(|m| m.get_frequent(limit)).unwrap_or_default()
    }

    // --- Analytics view methods ---

    /// Show the analytics screen.
    pub fn show_analytics(&mut self) {
        self.mode = AppMode::Analytics;
    }

    /// Dismiss the analytics screen and return to normal mode.
    pub fn dismiss_analytics(&mut self) {
        self.mode = AppMode::Normal;
    }

    /// Check if analytics is currently shown.
    pub fn is_analytics_shown(&self) -> bool {
        matches!(self.mode, AppMode::Analytics)
    }

    /// Get analytics report for display.
    pub fn get_analytics_report(
        &self,
        period: crate::core::TimePeriod,
    ) -> crate::core::AnalyticsReport {
        let entries = self.history_manager.as_ref().map(|m| m.get_recent(1000)).unwrap_or_default();

        crate::core::Analytics::calculate(&entries, period)
    }

    // --- Favorites & History methods ---

    /// Toggle favorite status for the currently selected command.
    pub fn toggle_favorite(&mut self) {
        if let Some(cmd) = self.get_selected_command() {
            let command_id = cmd.id.clone();
            if let Some(ref mut manager) = self.history_manager {
                let is_favorite = manager.toggle_favorite(&command_id);
                let _ = manager.save();

                self.set_status(if is_favorite {
                    "Command added to favorites ⭐"
                } else {
                    "Command removed from favorites"
                });
            }
        }
    }

    /// Check if a command is a favorite.
    pub fn is_favorite(&self, command_id: &str) -> bool {
        self.history_manager.as_ref().map(|m| m.is_favorite(command_id)).unwrap_or(false)
    }

    /// Get frecency score for a command.
    pub fn get_frecency(&self, command_id: &str) -> f64 {
        self.history_manager.as_ref().map(|m| m.get_frecency(command_id)).unwrap_or(0.0)
    }

    /// Record a command execution in history.
    pub fn record_execution(
        &mut self,
        command_id: &str,
        command_name: &str,
        duration_ms: u64,
        success: bool,
    ) {
        if let Some(ref mut manager) = self.history_manager {
            manager.record_execution(command_id, command_name, duration_ms, success);
            let _ = manager.save();
        }
    }

    /// Get history entry for a command.
    pub fn get_history_entry(&self, command_id: &str) -> Option<&crate::core::HistoryEntry> {
        self.history_manager.as_ref()?.get_entry(command_id)
    }

    /// Get favorites count.
    pub fn favorites_count(&self) -> usize {
        self.history_manager.as_ref().map(|m| m.favorites_count()).unwrap_or(0)
    }

    /// Save history to disk.
    pub fn save_history(&self) {
        if let Some(ref manager) = self.history_manager {
            let _ = manager.save();
        }
    }

    // --- Resilience & Recovery methods ---

    /// Try to recover a degraded feature.
    pub fn try_recover_feature(&mut self, feature: crate::core::Feature) {
        use crate::core::Feature;

        // Reset the circuit breaker for this feature
        match feature {
            Feature::Ai => self.resilience.ai.reset(),
            Feature::Network => self.resilience.network.reset(),
            Feature::Sync => self.resilience.sync.reset(),
            Feature::Integrations => self.resilience.integrations.reset(),
            Feature::Mcp => self.resilience.mcp.reset(),
            _ => {}
        }

        // Mark as recovered
        self.degradation.recover(feature);
        self.set_status(format!("{} recovered - retry available", feature));
    }

    /// Try to recover all degraded features.
    pub fn try_recover_all(&mut self) {
        self.resilience.reset_all();
        self.degradation.clear();
        self.set_status("All features recovered");
    }

    /// Process the offline queue (sync pending operations).
    pub fn process_offline_queue(&mut self) -> usize {
        if self.is_offline {
            return 0;
        }

        let mut processed = 0;
        let mut to_requeue: Vec<crate::core::QueueEntry> = Vec::new();

        // Collect entries to process
        while let Some(entry) = self.offline_manager.queue_mut().dequeue() {
            // Process each queued operation
            let success = Self::process_queued_operation_static(&entry.operation);

            if success {
                processed += 1;
            } else {
                // Mark for re-queue if not exhausted
                to_requeue.push(entry);
                break; // Stop processing on first failure
            }
        }

        // Re-queue failed entries
        for entry in to_requeue {
            if !self.offline_manager.queue_mut().requeue(entry) {
                tracing::warn!("Queued operation exhausted retries");
            }
        }

        if processed > 0 {
            self.set_status(format!("Synced {} queued operations", processed));
        }

        processed
    }

    /// Process a single queued operation (static method to avoid borrow issues).
    fn process_queued_operation_static(operation: &crate::core::QueuedOperation) -> bool {
        use crate::core::QueuedOperation;

        match operation {
            QueuedOperation::AiRequest { .. } => {
                // AI requests would need async handling - skip for now
                tracing::debug!("Skipping queued AI request (async not supported in sync context)");
                true // Mark as processed to avoid infinite queue
            }
            QueuedOperation::SyncHistory { .. } => {
                // Sync history to cloud - would need implementation
                tracing::debug!("Syncing history (stub)");
                true
            }
            QueuedOperation::SendAnalytics { .. } => {
                // Analytics - would need implementation
                tracing::debug!("Sending analytics (stub)");
                true
            }
            QueuedOperation::Webhook { url, payload: _ } => {
                // Webhook execution would need async HTTP client
                // For now, log and mark as needing async processing
                tracing::debug!(url, "Webhook queued (async processing needed)");
                true
            }
            QueuedOperation::Custom { operation_type, .. } => {
                tracing::debug!(operation_type, "Processing custom operation (stub)");
                true
            }
        }
    }

    /// Get count of pending queued operations.
    pub fn pending_queue_count(&self) -> usize {
        self.offline_manager.queue().len()
    }

    /// Check and update offline status.
    pub fn check_connectivity(&mut self) {
        if self.offline_manager.should_check_connectivity() {
            // Simple connectivity check - try to resolve a known host
            let was_offline = self.is_offline;
            self.is_offline = !Self::quick_connectivity_check();
            self.offline_manager.set_offline(self.is_offline);
            self.offline_manager.mark_checked();

            // If coming back online, try to process queue
            if was_offline && !self.is_offline {
                self.set_status("Connection restored");
                self.process_offline_queue();
            } else if !was_offline && self.is_offline {
                self.set_status("Connection lost - offline mode");
            }
        }
    }

    /// Quick connectivity check.
    fn quick_connectivity_check() -> bool {
        // Try to connect to a reliable endpoint
        std::net::TcpStream::connect_timeout(
            &std::net::SocketAddr::from(([1, 1, 1, 1], 53)),
            std::time::Duration::from_millis(500),
        )
        .is_ok()
    }

    /// Get resilience status for display.
    pub fn resilience_status(&self) -> Vec<(crate::core::Feature, &'static str)> {
        use crate::core::CircuitState;

        self.resilience
            .status_summary()
            .into_iter()
            .map(|(feature, state)| {
                let status = match state {
                    CircuitState::Closed => "OK",
                    CircuitState::Open => "UNAVAILABLE",
                    CircuitState::HalfOpen => "RECOVERING",
                };
                (feature, status)
            })
            .collect()
    }

    // --- Pass-through mode methods ---

    /// Enter pass-through mode to run a shell command.
    pub fn enter_pass_through(&mut self) {
        if !self.input.is_empty() {
            self.pass_through_command = Some(self.input.clone());
            self.mode = AppMode::PassThrough;
        }
    }

    /// Try to auto-execute safe shell commands without confirmation.
    /// Returns true if command was auto-executed, false if confirmation needed.
    pub fn try_auto_shell_command(&mut self) -> bool {
        let input = self.input.trim().to_string();

        // Safe commands that can auto-execute
        if input.starts_with("cd ") || input == "cd" || input == "cd ~" {
            self.handle_cd_command(&input);
            self.input.clear();
            self.cursor_position = 0;
            self.dir_listing = None;
            return true;
        }

        if input == "ls" || input.starts_with("ls ") {
            self.execute_shell_command(&input);
            self.input.clear();
            self.cursor_position = 0;
            self.dir_listing = None;
            return true;
        }

        if input == "pwd" {
            let pwd = self.cwd.display().to_string();
            self.last_output = Some(CommandOutput {
                command_name: "pwd".to_string(),
                command_str: "pwd".to_string(),
                stdout: format!("{}\n", pwd),
                stderr: String::new(),
                exit_code: Some(0),
                success: true,
            });
            self.mode = AppMode::ExecutionResult;
            self.output_scroll = 0;
            self.input.clear();
            self.cursor_position = 0;
            return true;
        }

        if input == "clear" {
            // Just clear input, no output needed
            self.input.clear();
            self.cursor_position = 0;
            self.set_status("Screen cleared");
            return true;
        }

        // Not a safe auto-execute command
        false
    }

    /// Cancel pass-through mode and return to normal.
    pub fn cancel_pass_through(&mut self) {
        self.pass_through_command = None;
        self.mode = AppMode::Normal;
    }

    /// Execute the pass-through shell command.
    pub fn execute_pass_through(&mut self) {
        if let Some(cmd) = self.pass_through_command.take() {
            // Check for cd command to handle directory change
            if cmd.trim().starts_with("cd ") {
                self.handle_cd_command(&cmd);
            } else {
                // Execute as regular shell command
                self.execute_shell_command(&cmd);
            }
            self.input.clear();
            self.cursor_position = 0;
        }
        self.mode = AppMode::Normal;
    }

    /// Handle cd command to change directory.
    fn handle_cd_command(&mut self, cmd: &str) {
        let path_str = cmd.trim().strip_prefix("cd ").unwrap_or("").trim();
        let path = if path_str.is_empty() || path_str == "~" {
            // cd or cd ~ goes to home directory
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
        } else if path_str.starts_with("~/") {
            // Expand ~ to home directory
            dirs::home_dir()
                .map(|h| h.join(&path_str[2..]))
                .unwrap_or_else(|| PathBuf::from(path_str))
        } else {
            self.cwd.join(path_str)
        };

        match std::fs::canonicalize(&path) {
            Ok(canonical) => {
                if canonical.is_dir() {
                    self.cwd = canonical;
                    // Update context
                    self.context = CommandContext::new(&self.cwd, &self.cwd);
                    // Update git info
                    #[cfg(feature = "git")]
                    {
                        self.git_info = crate::git::current_git_info();
                    }
                    // Rescan commands in new directory
                    self.rescan_commands();
                    self.set_status(format!("Changed to: {}", self.cwd.display()));
                } else {
                    self.set_status(format!("Not a directory: {}", path.display()));
                }
            }
            Err(e) => {
                self.set_status(format!("cd error: {}", e));
            }
        }
    }

    /// Execute a shell command and capture output.
    fn execute_shell_command(&mut self, cmd: &str) {
        use std::process::Command;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

        match Command::new(&shell).arg("-c").arg(cmd).current_dir(&self.cwd).output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                self.last_output = Some(CommandOutput {
                    command_name: cmd.to_string(),
                    command_str: cmd.to_string(),
                    stdout,
                    stderr,
                    exit_code: output.status.code(),
                    success: output.status.success(),
                });

                self.mode = AppMode::ExecutionResult;
                self.output_scroll = 0;
            }
            Err(e) => {
                self.set_status(format!("Error: {}", e));
            }
        }
    }

    /// Rescan commands in the current directory.
    pub fn rescan_commands(&mut self) {
        self.registry = CommandRegistry::new();
        self.filtered_commands.clear();
        self.selected = 0;
        // Note: actual scanning happens in main.rs scan_and_run
    }

    // --- Command palette methods ---

    /// Show the command palette.
    pub fn show_palette(&mut self) {
        self.palette_input.clear();
        self.palette_selected = 0;
        self.mode = AppMode::Palette;
    }

    /// Dismiss the command palette.
    pub fn dismiss_palette(&mut self) {
        self.mode = AppMode::Normal;
    }

    // --- Context menu methods ---

    /// Show the context menu for selected command.
    pub fn show_context_menu(&mut self) {
        if self.get_selected_command().is_some() {
            self.context_menu_selected = 0;
            self.mode = AppMode::ContextMenu;
        }
    }

    /// Dismiss the context menu.
    pub fn dismiss_context_menu(&mut self) {
        self.mode = AppMode::Normal;
    }

    // --- Tips ---

    /// Get current tip text.
    pub fn current_tip(&self) -> &'static str {
        const TIPS: &[&str] = &[
            "Type to search • ? for help",
            "Ctrl+B runs in background",
            "# for tags • @workspace filter",
            "Ctrl+S to toggle favorites",
            "Ctrl+P for command palette",
            ". for quick actions menu",
        ];
        TIPS.get(self.tip_index % TIPS.len()).unwrap_or(&TIPS[0])
    }

    /// Rotate to next tip.
    pub fn next_tip(&mut self) {
        self.tip_index = (self.tip_index + 1) % 6;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            let cwd = PathBuf::from(".");
            Self {
                input: String::new(),
                cursor_position: 0,
                registry: CommandRegistry::new(),
                filtered_commands: Vec::new(),
                selected: 0,
                should_quit: false,
                command_selected: false,
                cwd: cwd.clone(),
                config: Config::default(),
                mode: AppMode::default(),
                status_message: None,
                context: CommandContext::new(&cwd, &cwd),
                context_aware: true,
                #[cfg(feature = "git")]
                git_info: None,
                last_output: None,
                output_scroll: 0,
                selected_commands: HashSet::new(),
                multi_select_mode: false,
                background_manager: None,
                capture_manager: None,
                theme: Theme::default(),
                active_filters: None,
                history_manager: None,
                pass_through_command: None,
                palette_input: String::new(),
                palette_selected: 0,
                context_menu_selected: 0,
                tip_index: 0,
                dir_listing: None,
                dir_selected: 0,
                ghost_text: None,
                slash_commands: None,
                slash_selected: 0,
                is_offline: false,
                ai_status: None,
                degradation: crate::core::DegradationManager::new(),
                offline_manager: crate::core::OfflineManager::new(),
                resilience: crate::core::ResilienceManager::new(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new_test();
        assert!(app.input.is_empty());
        assert!(!app.should_quit);
    }

    #[test]
    fn test_char_input() {
        let mut app = App::new_test();
        app.enter_char('h');
        app.enter_char('e');
        app.enter_char('l');
        app.enter_char('l');
        app.enter_char('o');
        assert_eq!(app.input, "hello");
        assert_eq!(app.cursor_position, 5);
    }

    #[test]
    fn test_backspace() {
        let mut app = App::new_test();
        app.input = "hello".to_string();
        app.cursor_position = 5;
        app.delete_char();
        assert_eq!(app.input, "hell");
        assert_eq!(app.cursor_position, 4);
    }

    #[test]
    fn test_cursor_movement() {
        let mut app = App::new_test();
        app.input = "hello".to_string();
        app.cursor_position = 2;

        app.move_cursor_left();
        assert_eq!(app.cursor_position, 1);

        app.move_cursor_right();
        assert_eq!(app.cursor_position, 2);

        app.move_cursor_start();
        assert_eq!(app.cursor_position, 0);

        app.move_cursor_end();
        assert_eq!(app.cursor_position, 5);
    }

    #[test]
    fn test_selection() {
        let mut app = App::new_test();
        app.filtered_commands = vec![0, 1, 2, 3, 4];

        app.select_next();
        assert_eq!(app.selected, 1);

        app.select_next();
        assert_eq!(app.selected, 2);

        app.select_previous();
        assert_eq!(app.selected, 1);

        app.select_last();
        assert_eq!(app.selected, 4);

        app.select_first();
        assert_eq!(app.selected, 0);
    }

    #[test]
    fn test_filter_parsing_in_search() {
        use crate::core::{Command, CommandSource};
        use std::path::PathBuf;

        let mut app = App::new_test();

        // Add some commands with tags and different sources
        app.registry.add(
            Command::new("npm test", "npm test")
                .with_source(CommandSource::PackageJson(PathBuf::from(".")))
                .with_tag("test")
                .with_tag("dev"),
        );
        app.registry.add(
            Command::new("npm build", "npm run build")
                .with_source(CommandSource::PackageJson(PathBuf::from(".")))
                .with_tag("build"),
        );
        app.registry.add(
            Command::new("cargo test", "cargo test")
                .with_source(CommandSource::Cargo(PathBuf::from(".")))
                .with_tag("test"),
        );

        // Test no filters
        app.input = "test".to_string();
        app.update_filtered_commands();
        assert!(app.active_filters.is_none());
        assert_eq!(app.filtered_commands.len(), 2); // npm test and cargo test

        // Test with tag filter
        app.input = "#test".to_string();
        app.update_filtered_commands();
        assert_eq!(app.active_filters, Some("#test".to_string()));
        assert_eq!(app.filtered_commands.len(), 2); // npm test and cargo test

        // Test with source filter
        app.input = "source:npm".to_string();
        app.update_filtered_commands();
        assert_eq!(app.active_filters, Some("source:npm".to_string()));
        assert_eq!(app.filtered_commands.len(), 2); // npm test and npm build

        // Test combined filters
        app.input = "#test source:npm".to_string();
        app.update_filtered_commands();
        assert_eq!(app.active_filters, Some("#test source:npm".to_string()));
        assert_eq!(app.filtered_commands.len(), 1); // only npm test
    }

    #[test]
    fn test_workspace_filter() {
        use crate::core::Command;

        let mut app = App::new_test();

        // Add commands with workspaces
        app.registry
            .add(Command::new("frontend build", "npm run build").with_workspace("frontend"));
        app.registry.add(Command::new("backend build", "npm run build").with_workspace("backend"));
        app.registry.add(Command::new("root build", "npm run build")); // No workspace

        // Test workspace filter
        app.input = "@frontend".to_string();
        app.update_filtered_commands();
        assert_eq!(app.active_filters, Some("@frontend".to_string()));
        assert_eq!(app.filtered_commands.len(), 1);

        // Test pattern with workspace filter
        app.input = "build @backend".to_string();
        app.update_filtered_commands();
        assert_eq!(app.active_filters, Some("@backend".to_string()));
        assert_eq!(app.filtered_commands.len(), 1);
    }

    #[test]
    fn test_filter_clears_when_input_changes() {
        let mut app = App::new_test();

        // Add a command
        app.registry.add(Command::new("test", "npm test").with_tag("test"));

        // Set filter
        app.input = "#test".to_string();
        app.update_filtered_commands();
        assert!(app.active_filters.is_some());

        // Clear input
        app.input = String::new();
        app.update_filtered_commands();
        assert!(app.active_filters.is_none());
    }
}
