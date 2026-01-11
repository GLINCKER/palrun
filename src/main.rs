//! Palrun - AI command palette for your terminal.
//!
//! Palrun automatically detects your project's available commands and
//! presents them in a fuzzy-searchable command palette.

#![allow(clippy::single_match_else)]

use std::io::{self, Write};

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use palrun::{tui, App};

/// AI command palette for your terminal
#[derive(Parser)]
#[command(name = "palrun")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Subcommand to run
    #[command(subcommand)]
    command: Option<Commands>,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Run in non-interactive mode (list commands and exit)
    #[arg(short, long)]
    non_interactive: bool,

    /// Dry run mode - show what would be executed without running
    #[arg(long, global = true)]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Open the command palette (default)
    Run,

    /// List all available commands
    List {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Filter by source type (npm, make, etc.)
        #[arg(short, long)]
        source: Option<String>,
    },

    /// Execute a command directly by name
    Exec {
        /// Command name or pattern to execute
        name: String,

        /// Don't confirm before executing
        #[arg(short = 'y', long)]
        yes: bool,

        /// Dry run - show command without executing
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Scan the project and show what would be discovered
    Scan {
        /// Directory to scan
        #[arg(default_value = ".")]
        path: String,

        /// Enable recursive scanning
        #[arg(short, long)]
        recursive: bool,
    },

    /// Run a runbook
    Runbook {
        /// Runbook name
        name: String,

        /// Dry run (show what would be executed)
        #[arg(short, long)]
        dry_run: bool,

        /// Variable assignments (key=value)
        #[arg(short, long)]
        var: Vec<String>,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },

    /// Initialize shell integration
    Init {
        /// Shell to initialize
        shell: String,
    },

    /// Set up a new Palrun project with intelligent detection
    Setup {
        /// Directory to set up (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,

        /// Force overwrite existing files
        #[arg(short, long)]
        force: bool,

        /// Dry run - show what would be done without doing it
        #[arg(short, long)]
        dry_run: bool,

        /// Non-interactive mode - use defaults
        #[arg(short, long)]
        non_interactive: bool,
    },

    /// Show configuration
    Config {
        /// Show config file path
        #[arg(long)]
        path: bool,
    },

    /// AI-powered command generation
    #[cfg(feature = "ai")]
    Ai {
        /// AI operation mode
        #[command(subcommand)]
        operation: AiOperation,
    },

    /// Manage Git hooks
    #[cfg(feature = "git")]
    Hooks {
        /// Hooks operation
        #[command(subcommand)]
        operation: HooksOperation,
    },

    /// Manage environment variables
    Env {
        /// Env operation
        #[command(subcommand)]
        operation: EnvOperation,
    },

    /// Show runtime version requirements
    Versions {
        /// Show all detected runtimes (including those without requirements)
        #[arg(short, long)]
        all: bool,
    },

    /// Manage secrets from external providers
    Secrets {
        /// Secrets operation
        #[command(subcommand)]
        operation: SecretsOperation,
    },

    /// Manage plugins
    #[cfg(feature = "plugins")]
    Plugin {
        /// Plugin operation
        #[command(subcommand)]
        operation: PluginOperation,
    },

    /// GitHub Actions CI/CD integration
    Ci {
        /// CI operation
        #[command(subcommand)]
        operation: CiOperation,
    },

    /// Send notifications to Slack, Discord, or webhooks
    Notify {
        /// Notification operation
        #[command(subcommand)]
        operation: NotifyOperation,
    },

    /// GitHub Issues integration
    Issues {
        /// Issues operation
        #[command(subcommand)]
        operation: IssuesOperation,
    },

    /// Linear issue tracker integration
    Linear {
        /// Linear operation
        #[command(subcommand)]
        operation: LinearOperation,
    },

    /// MCP (Model Context Protocol) server management
    Mcp {
        /// MCP operation
        #[command(subcommand)]
        operation: McpOperation,
    },

    /// Debug and inspect Palrun internals
    Debug {
        /// Debug operation
        #[command(subcommand)]
        operation: DebugOperation,
    },
}

/// Debug operations.
#[derive(Subcommand)]
enum DebugOperation {
    /// Show loaded configuration
    Config,

    /// Show all discovered commands with metadata
    Commands {
        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Show project detection results
    Project,

    /// Show fuzzy search internals
    Search {
        /// Query to test
        query: String,
    },

    /// Test AI provider connection
    #[cfg(feature = "ai")]
    Ai,

    /// Show environment information
    Env,

    /// Show scanner detection results
    Scanners,
}

/// MCP operations.
#[derive(Subcommand)]
enum McpOperation {
    /// List configured MCP servers
    Servers,

    /// List available tools from MCP servers
    Tools {
        /// Only show tools from a specific server
        #[arg(short, long)]
        server: Option<String>,
    },

    /// Call an MCP tool
    Call {
        /// Server name
        server: String,

        /// Tool name
        tool: String,

        /// Tool arguments as JSON
        #[arg(short, long)]
        args: Option<String>,
    },

    /// Start an MCP server
    Start {
        /// Server name
        name: String,
    },

    /// Stop an MCP server
    Stop {
        /// Server name
        name: String,
    },

    /// Show MCP configuration
    Config,
}

/// Environment operations.
#[derive(Subcommand)]
enum EnvOperation {
    /// List detected .env files
    List,

    /// Show environment variables
    Show {
        /// Show all variables (including system)
        #[arg(short, long)]
        all: bool,

        /// Show sensitive values unmasked
        #[arg(long)]
        unmask: bool,

        /// Filter by variable name pattern
        #[arg(short, long)]
        filter: Option<String>,
    },

    /// Load a specific .env file
    Load {
        /// .env file name or path
        file: String,
    },

    /// Compare two .env files
    Diff {
        /// First .env file
        file1: String,

        /// Second .env file
        file2: String,
    },

    /// Show which .env file is currently active
    Active,
}

/// Secrets operations.
#[derive(Subcommand)]
enum SecretsOperation {
    /// Check status of secret providers (1Password, Vault, etc.)
    Status,

    /// Scan .env files for secret references
    Scan,

    /// Inject secrets from providers into environment
    Inject {
        /// Don't actually inject, just show what would be injected
        #[arg(short, long)]
        dry_run: bool,

        /// Only inject secrets from a specific provider (1password, vault)
        #[arg(short, long)]
        provider: Option<String>,
    },

    /// List detected secret references
    List {
        /// Filter by provider (1password, vault)
        #[arg(short, long)]
        provider: Option<String>,
    },
}

/// Git hooks operations.
#[cfg(feature = "git")]
#[derive(Subcommand)]
enum HooksOperation {
    /// List installed Git hooks
    List,

    /// Install a Git hook
    Install {
        /// Hook name (pre-commit, pre-push, etc.)
        hook: String,

        /// Command to run
        command: String,

        /// Force overwrite existing hook
        #[arg(short, long)]
        force: bool,
    },

    /// Uninstall a Git hook
    Uninstall {
        /// Hook name to uninstall
        hook: String,

        /// Force remove even if not managed by Palrun
        #[arg(short, long)]
        force: bool,
    },

    /// Uninstall all Palrun-managed hooks
    UninstallAll,

    /// Sync hooks from palrun.toml configuration
    Sync {
        /// Force overwrite existing hooks
        #[arg(short, long)]
        force: bool,
    },
}

/// Plugin operations.
#[cfg(feature = "plugins")]
#[derive(Subcommand)]
enum PluginOperation {
    /// List installed plugins
    List {
        /// Show only enabled plugins
        #[arg(short, long)]
        enabled: bool,

        /// Filter by plugin type (scanner, ai-provider, integration, ui)
        #[arg(short = 't', long)]
        plugin_type: Option<String>,
    },

    /// Search for plugins in the registry
    Search {
        /// Search query
        query: String,

        /// Filter by plugin type (scanner, ai-provider, integration, ui)
        #[arg(short = 't', long)]
        plugin_type: Option<String>,

        /// Show all results (including incompatible plugins)
        #[arg(short, long)]
        all: bool,
    },

    /// Browse available plugins in the registry
    Browse {
        /// Filter by plugin type (scanner, ai-provider, integration, ui)
        #[arg(short = 't', long)]
        plugin_type: Option<String>,

        /// Sort by (popularity, name, updated)
        #[arg(short, long, default_value = "popularity")]
        sort: String,

        /// Force refresh the registry cache
        #[arg(short, long)]
        refresh: bool,
    },

    /// Install a plugin
    Install {
        /// Plugin source (file path or registry plugin name)
        source: String,

        /// Force install (overwrite if exists)
        #[arg(short, long)]
        force: bool,
    },

    /// Uninstall a plugin
    Uninstall {
        /// Plugin name
        name: String,

        /// Force uninstall without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Enable a disabled plugin
    Enable {
        /// Plugin name
        name: String,
    },

    /// Disable a plugin
    Disable {
        /// Plugin name
        name: String,
    },

    /// Show plugin information
    Info {
        /// Plugin name
        name: String,
    },

    /// Update installed plugins
    Update {
        /// Plugin name (update all if not specified)
        name: Option<String>,

        /// Check for updates only (don't install)
        #[arg(short, long)]
        check: bool,
    },

    /// Clear the registry cache
    ClearCache,
}

/// AI operation modes.
#[cfg(feature = "ai")]
#[derive(Subcommand)]
enum AiOperation {
    /// Generate a command from natural language
    Gen {
        /// Natural language prompt
        prompt: String,

        /// Execute the generated command immediately
        #[arg(short = 'x', long)]
        execute: bool,
    },

    /// Explain what a command does
    Explain {
        /// The command to explain
        command: String,
    },

    /// Diagnose why a command failed
    Diagnose {
        /// The command that failed
        command: String,

        /// The error message
        error: String,
    },

    /// Show which AI provider is active
    Status,

    /// Run an AI agent that can use MCP tools to complete tasks
    Agent {
        /// The task to accomplish
        task: String,

        /// Maximum number of iterations (default: 10)
        #[arg(short, long, default_value = "10")]
        max_iterations: usize,

        /// Use only local LLM (Ollama)
        #[arg(long)]
        local: bool,
    },
}

/// CI/CD operations.
#[derive(Subcommand)]
enum CiOperation {
    /// Show CI status for current branch
    Status {
        /// Branch to check (defaults to current)
        #[arg(short, long)]
        branch: Option<String>,
    },

    /// List available workflows
    Workflows,

    /// List recent workflow runs
    Runs {
        /// Filter by workflow name or ID
        #[arg(short, long)]
        workflow: Option<String>,

        /// Filter by branch
        #[arg(short, long)]
        branch: Option<String>,

        /// Number of runs to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Trigger a workflow
    Trigger {
        /// Workflow name or ID
        workflow: String,

        /// Branch to run on (defaults to current)
        #[arg(short, long)]
        branch: Option<String>,

        /// Input parameters as JSON
        #[arg(short, long)]
        inputs: Option<String>,
    },

    /// Re-run a failed workflow
    Rerun {
        /// Run ID to re-run
        run_id: u64,
    },

    /// Cancel a running workflow
    Cancel {
        /// Run ID to cancel
        run_id: u64,
    },

    /// Open CI page in browser
    Open {
        /// Open a specific run ID
        run_id: Option<u64>,
    },
}

/// Notification operations.
#[derive(Subcommand)]
enum NotifyOperation {
    /// Send a message to Slack
    Slack {
        /// Slack webhook URL
        #[arg(short, long, env = "SLACK_WEBHOOK_URL")]
        url: String,

        /// Message to send
        message: String,

        /// Optional title
        #[arg(short, long)]
        title: Option<String>,

        /// Color (hex format: #RRGGBB)
        #[arg(short, long)]
        color: Option<String>,
    },

    /// Send a message to Discord
    Discord {
        /// Discord webhook URL
        #[arg(short, long, env = "DISCORD_WEBHOOK_URL")]
        url: String,

        /// Message to send
        message: String,

        /// Optional title
        #[arg(short, long)]
        title: Option<String>,

        /// Color (hex format: #RRGGBB)
        #[arg(short, long)]
        color: Option<String>,
    },

    /// Send a message to a generic webhook
    Webhook {
        /// Webhook URL
        #[arg(short, long)]
        url: String,

        /// Message to send
        message: String,

        /// Optional title
        #[arg(short, long)]
        title: Option<String>,
    },

    /// Test a notification endpoint
    Test {
        /// Notification type (slack, discord, webhook)
        #[arg(short = 't', long)]
        notification_type: String,

        /// Webhook URL
        #[arg(short, long)]
        url: String,
    },
}

/// GitHub Issues operations.
#[derive(Subcommand)]
enum IssuesOperation {
    /// List issues in the repository
    List {
        /// Filter by state (open, closed, all)
        #[arg(short, long, default_value = "open")]
        state: String,

        /// Filter by labels (comma-separated)
        #[arg(short, long)]
        labels: Option<String>,

        /// Filter by assignee
        #[arg(short, long)]
        assignee: Option<String>,

        /// Maximum number of issues to show
        #[arg(short = 'n', long, default_value = "20")]
        limit: u32,
    },

    /// Get details of a specific issue
    View {
        /// Issue number
        number: u64,

        /// Show comments
        #[arg(short, long)]
        comments: bool,
    },

    /// Create a new issue
    Create {
        /// Issue title
        #[arg(short, long)]
        title: String,

        /// Issue body/description
        #[arg(short, long)]
        body: Option<String>,

        /// Labels to add (comma-separated)
        #[arg(short, long)]
        labels: Option<String>,

        /// Assignees (comma-separated)
        #[arg(short, long)]
        assignees: Option<String>,
    },

    /// Close an issue
    Close {
        /// Issue number
        number: u64,

        /// Add a comment when closing
        #[arg(short, long)]
        comment: Option<String>,
    },

    /// Reopen an issue
    Reopen {
        /// Issue number
        number: u64,
    },

    /// Add a comment to an issue
    Comment {
        /// Issue number
        number: u64,

        /// Comment body
        body: String,
    },

    /// Add labels to an issue
    Label {
        /// Issue number
        number: u64,

        /// Labels to add (comma-separated)
        labels: String,
    },

    /// Search for issues
    Search {
        /// Search query (GitHub search syntax)
        query: String,
    },

    /// Show issue statistics
    Stats,

    /// Open issue in browser
    Open {
        /// Issue number (opens issues list if not specified)
        number: Option<u64>,
    },
}

/// Linear operations.
#[derive(Subcommand)]
enum LinearOperation {
    /// List your assigned issues
    List {
        /// Filter by team key (e.g., ENG)
        #[arg(short, long)]
        team: Option<String>,

        /// Include completed/canceled issues
        #[arg(short, long)]
        all: bool,

        /// Maximum number of issues
        #[arg(short = 'n', long, default_value = "20")]
        limit: u32,
    },

    /// View a specific issue
    View {
        /// Issue identifier (e.g., ENG-123)
        identifier: String,
    },

    /// Create a new issue
    Create {
        /// Issue title
        #[arg(short, long)]
        title: String,

        /// Issue description
        #[arg(short, long)]
        description: Option<String>,

        /// Team key (e.g., ENG)
        #[arg(short = 'T', long)]
        team: String,

        /// Priority (1=urgent, 2=high, 3=medium, 4=low)
        #[arg(short, long)]
        priority: Option<i32>,
    },

    /// List available teams
    Teams,

    /// Search for issues
    Search {
        /// Search query
        query: String,
    },

    /// Show your issue statistics
    Stats,

    /// Show current user info
    Me,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let filter = if cli.verbose { EnvFilter::new("debug") } else { EnvFilter::new("warn") };

    tracing_subscriber::registry().with(fmt::layer().with_target(false)).with(filter).init();

    // Handle commands
    match cli.command {
        None | Some(Commands::Run) => {
            if cli.non_interactive {
                cmd_list("text", None)?;
            } else {
                cmd_run()?;
            }
        }
        Some(Commands::List { format, source }) => {
            cmd_list(&format, source.as_deref())?;
        }
        Some(Commands::Exec { name, yes, dry_run }) => {
            cmd_exec(&name, yes, dry_run || cli.dry_run)?;
        }
        Some(Commands::Scan { path, recursive }) => {
            cmd_scan(&path, recursive)?;
        }
        Some(Commands::Runbook { name, dry_run, var }) => {
            cmd_runbook(&name, dry_run, &var)?;
        }
        Some(Commands::Completions { shell }) => {
            cmd_completions(shell);
        }
        Some(Commands::Init { shell }) => {
            cmd_init(&shell)?;
        }
        Some(Commands::Setup { path, force, dry_run, non_interactive }) => {
            cmd_setup(&path, force, dry_run, non_interactive)?;
        }
        Some(Commands::Config { path }) => {
            cmd_config(path)?;
        }
        #[cfg(feature = "ai")]
        Some(Commands::Ai { operation }) => {
            cmd_ai(operation)?;
        }
        #[cfg(feature = "git")]
        Some(Commands::Hooks { operation }) => {
            cmd_hooks(operation)?;
        }
        Some(Commands::Env { operation }) => {
            cmd_env(operation)?;
        }
        Some(Commands::Versions { all }) => {
            cmd_versions(all)?;
        }
        Some(Commands::Secrets { operation }) => {
            cmd_secrets(operation)?;
        }
        #[cfg(feature = "plugins")]
        Some(Commands::Plugin { operation }) => {
            cmd_plugin(operation)?;
        }
        Some(Commands::Ci { operation }) => {
            cmd_ci(operation)?;
        }
        Some(Commands::Notify { operation }) => {
            cmd_notify(operation)?;
        }
        Some(Commands::Issues { operation }) => {
            cmd_issues(operation)?;
        }
        Some(Commands::Linear { operation }) => {
            cmd_linear(operation)?;
        }
        Some(Commands::Mcp { operation }) => {
            cmd_mcp(operation)?;
        }
        Some(Commands::Debug { operation }) => {
            cmd_debug(operation)?;
        }
    }

    Ok(())
}

/// Run the interactive TUI.
fn cmd_run() -> Result<()> {
    let app = App::new()?;
    tui::run_tui(app)
}

/// List available commands.
fn cmd_list(format: &str, source_filter: Option<&str>) -> Result<()> {
    let mut app = App::new()?;
    app.initialize()?;

    let commands: Vec<_> = if let Some(source) = source_filter {
        app.registry.get_by_source_type(source).into_iter().cloned().collect()
    } else {
        app.registry.get_all().to_vec()
    };

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&commands)?;
            println!("{json}");
        }
        _ => {
            for cmd in &commands {
                println!(
                    "{} {} - {}",
                    cmd.source.icon(),
                    cmd.name,
                    cmd.description.as_deref().unwrap_or("")
                );
            }
            println!("\nTotal: {} commands", commands.len());
        }
    }

    Ok(())
}

/// Execute a command directly.
fn cmd_exec(name: &str, skip_confirm: bool, dry_run: bool) -> Result<()> {
    let mut app = App::new()?;
    app.initialize()?;

    // Search for the command
    let matches = app.registry.search(name);

    if matches.is_empty() {
        anyhow::bail!("No command matching '{name}' found");
    }

    let cmd = app.registry.get_by_index(matches[0]).unwrap();

    // Dry run - just show what would be executed
    if dry_run {
        println!("[DRY RUN] Would execute:");
        println!("  Name: {}", cmd.name);
        println!("  Command: {}", cmd.command);
        if let Some(dir) = &cmd.working_dir {
            println!("  Working dir: {}", dir.display());
        }
        if let Some(desc) = &cmd.description {
            println!("  Description: {}", desc);
        }
        println!("  Source: {:?}", cmd.source);
        if !cmd.tags.is_empty() {
            println!("  Tags: {}", cmd.tags.join(", "));
        }
        return Ok(());
    }

    // Confirm if needed
    if cmd.confirm && !skip_confirm {
        print!("Execute '{}'? [y/N] ", cmd.command);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled");
            return Ok(());
        }
    }

    // Execute
    println!("Executing: {}", cmd.command);
    let executor = palrun::core::Executor::new();
    let result = executor.execute(cmd)?;

    std::process::exit(result.code().unwrap_or(0));
}

/// Scan a project and show discovered commands.
fn cmd_scan(path: &str, recursive: bool) -> Result<()> {
    use palrun::scanner::ProjectScanner;

    let path = std::path::Path::new(path);
    let scanner = ProjectScanner::new(path);

    let commands = if recursive { scanner.scan_recursive(5)? } else { scanner.scan()? };

    println!("Discovered {} commands in {:?}\n", commands.len(), path);

    // Group by source
    let mut by_source: std::collections::HashMap<&str, Vec<_>> = std::collections::HashMap::new();
    for cmd in &commands {
        by_source.entry(cmd.source.type_name()).or_default().push(cmd);
    }

    for (source, cmds) in &by_source {
        println!("{}:", source.to_uppercase());
        for cmd in cmds {
            println!("  - {}", cmd.name);
        }
        println!();
    }

    Ok(())
}

/// Run a runbook.
fn cmd_runbook(name: &str, dry_run: bool, vars: &[String]) -> Result<()> {
    use palrun::runbook::{discover_runbooks, RunbookRunner};

    let cwd = std::env::current_dir()?;
    let runbooks = discover_runbooks(&cwd)?;

    let runbook = runbooks
        .into_iter()
        .find(|(n, _)| n == name)
        .map(|(_, r)| r)
        .ok_or_else(|| anyhow::anyhow!("Runbook '{}' not found", name))?;

    println!("Runbook: {}", runbook.name);
    if let Some(ref desc) = runbook.description {
        println!("Description: {desc}");
    }
    println!("Steps: {}\n", runbook.steps.len());

    if dry_run {
        println!("DRY RUN - Steps that would be executed:");
        for (i, step) in runbook.steps.iter().enumerate() {
            println!("  {}. {} - {}", i + 1, step.name, step.command);
        }
        return Ok(());
    }

    let mut runner = RunbookRunner::new(runbook);

    // Set variables from command line
    for var_str in vars {
        if let Some((key, value)) = var_str.split_once('=') {
            runner.set_variable(key, value);
        }
    }

    runner.run()?;

    println!("\nRunbook completed successfully!");
    Ok(())
}

/// Generate shell completions.
fn cmd_completions(shell: Shell) {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "palrun", &mut io::stdout());
}

/// Output shell initialization script.
fn cmd_init(shell: &str) -> Result<()> {
    let script = match shell.to_lowercase().as_str() {
        "bash" => include_str!("../shell/palrun.bash"),
        "zsh" => include_str!("../shell/palrun.zsh"),
        "fish" => include_str!("../shell/palrun.fish"),
        "powershell" | "pwsh" => include_str!("../shell/palrun.ps1"),
        _ => anyhow::bail!("Unsupported shell: {shell}. Supported: bash, zsh, fish, powershell"),
    };

    println!("{script}");
    Ok(())
}

/// Set up a new Palrun project.
fn cmd_setup(path: &str, force: bool, dry_run: bool, non_interactive: bool) -> Result<()> {
    use palrun::init::{setup_project, SetupOptions};
    use std::path::PathBuf;

    let path = PathBuf::from(path);
    let options = SetupOptions { force, dry_run, non_interactive };

    setup_project(&path, options)?;

    Ok(())
}

/// Show configuration.
fn cmd_config(show_path: bool) -> Result<()> {
    use palrun::core::Config;

    if show_path {
        if let Some(path) = Config::config_dir() {
            println!("{}", path.display());
        }
        return Ok(());
    }

    let config = Config::load()?;
    let toml = toml::to_string_pretty(&config)?;
    println!("{toml}");

    Ok(())
}

/// Handle AI commands.
#[cfg(feature = "ai")]
fn cmd_ai(operation: AiOperation) -> Result<()> {
    use palrun::ai::{AIManager, ProjectContext};

    // Create tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        // Build project context
        let mut context = ProjectContext::from_current_dir()?;

        // Get available commands for context
        let mut app = App::new()?;
        app.initialize()?;
        let commands: Vec<String> = app.registry.get_all().iter().map(|c| c.name.clone()).collect();
        context = context.with_commands(commands);

        // Initialize AI manager
        let ai = AIManager::new().await;

        if !ai.is_available() {
            anyhow::bail!(
                "No AI provider available.\n\
                 Set ANTHROPIC_API_KEY for Claude, or run Ollama locally."
            );
        }

        match operation {
            AiOperation::Gen { prompt, execute } => {
                println!("Generating command...\n");

                let command = ai.generate_command(&prompt, &context).await?;
                println!("Generated: {command}");

                if execute {
                    print!("\nExecute? [y/N] ");
                    io::stdout().flush()?;

                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;

                    if input.trim().eq_ignore_ascii_case("y") {
                        let cmd = palrun::Command::new("ai-generated", &command);
                        let executor = palrun::core::Executor::new();
                        let result = executor.execute(&cmd)?;
                        std::process::exit(result.code().unwrap_or(0));
                    }
                }
            }

            AiOperation::Explain { command } => {
                println!("Explaining command...\n");

                let explanation = ai.explain_command(&command, &context).await?;
                println!("{explanation}");
            }

            AiOperation::Diagnose { command, error } => {
                println!("Diagnosing error...\n");

                let diagnosis = ai.diagnose_error(&command, &error, &context).await?;
                println!("{diagnosis}");
            }

            AiOperation::Status => {
                if let Some(provider) = ai.active_provider() {
                    println!("Active AI provider: {provider}");
                } else {
                    println!("No AI provider available");
                }
            }

            AiOperation::Agent { task, max_iterations, local } => {
                use palrun::ai::{
                    AIProvider, Agent, AgentProvider, AgentState, MCPToolExecutor, OllamaProvider,
                };
                use palrun::mcp::MCPServerConfig;
                use palrun::Config;

                println!("Starting AI Agent...\n");

                // Load MCP configuration
                let config = Config::load().unwrap_or_default();

                // Create tool executor with MCP servers
                let mut executor = MCPToolExecutor::new();

                // Add MCP servers from config
                for server_entry in &config.mcp.servers {
                    let mcp_config = MCPServerConfig {
                        name: server_entry.name.clone(),
                        command: server_entry.command.clone(),
                        args: server_entry.args.clone(),
                        env: server_entry.env.clone(),
                        cwd: server_entry.cwd.clone(),
                    };

                    if let Err(e) = executor.add_server(mcp_config) {
                        eprintln!("Warning: Failed to add server '{}': {}", server_entry.name, e);
                    }
                }

                // Start all servers
                if !config.mcp.servers.is_empty() {
                    if let Err(e) = executor.start() {
                        eprintln!("Warning: Some MCP servers failed to start: {}", e);
                    }
                }

                // Get available tools
                let tools = executor.available_tools();

                if tools.is_empty() {
                    println!("No MCP tools available. Running without tools.");
                    println!("Configure MCP servers in palrun.toml to enable tool use.\n");
                } else {
                    println!(
                        "Available tools: {}",
                        tools.iter().map(|t| t.name.as_str()).collect::<Vec<_>>().join(", ")
                    );
                    println!();
                }

                // Create agent state
                let state = AgentState::new(context.clone())
                    .with_tools(tools)
                    .with_max_iterations(max_iterations);

                // Run agent
                if local {
                    // Use Ollama only
                    let provider = OllamaProvider::new();
                    if !provider.is_available().await {
                        anyhow::bail!("Ollama is not available. Make sure it's running.");
                    }

                    println!("Using Ollama (local LLM)");
                    println!("Task: {}\n", task);

                    let mut agent = Agent::new(provider, executor);
                    let final_state = agent.run(&task, state).await?;

                    // Print final response
                    if let Some(response) =
                        Agent::<OllamaProvider, MCPToolExecutor>::get_final_response(&final_state)
                    {
                        println!("\n--- Agent Response ---\n");
                        println!("{}", response);
                    }

                    println!(
                        "\nAgent completed in {} iteration(s).",
                        final_state.current_iteration
                    );
                } else {
                    // Use provider from AIManager (Claude or Ollama)
                    let provider = OllamaProvider::new();
                    if !provider.is_available().await {
                        anyhow::bail!(
                            "No AI provider available. Run Ollama or set ANTHROPIC_API_KEY."
                        );
                    }

                    println!("Using: {}", AgentProvider::name(&provider));
                    println!("Task: {}\n", task);

                    let mut agent = Agent::new(provider, executor);
                    let final_state = agent.run(&task, state).await?;

                    // Print final response
                    if let Some(response) =
                        Agent::<OllamaProvider, MCPToolExecutor>::get_final_response(&final_state)
                    {
                        println!("\n--- Agent Response ---\n");
                        println!("{}", response);
                    }

                    println!(
                        "\nAgent completed in {} iteration(s).",
                        final_state.current_iteration
                    );
                }
            }
        }

        Ok(())
    })
}

/// Handle Git hooks commands.
#[cfg(feature = "git")]
fn cmd_hooks(operation: HooksOperation) -> Result<()> {
    use palrun::core::Config;
    use palrun::git::HooksManager;

    let manager =
        HooksManager::discover().ok_or_else(|| anyhow::anyhow!("Not in a Git repository"))?;

    match operation {
        HooksOperation::List => {
            let hooks = manager.list_hooks();

            if hooks.is_empty() {
                println!("No Git hooks installed.");
                println!("\nAvailable hooks:");
                for name in palrun::git::hooks::HOOK_NAMES {
                    println!("  - {name}");
                }
            } else {
                println!("Installed Git hooks:\n");
                for hook in hooks {
                    let managed = if hook.is_palrun { " (palrun)" } else { "" };
                    let exec = if hook.is_executable { "" } else { " [not executable]" };
                    println!("  {} {}{}", hook.name, managed, exec);
                }
            }

            // Show configured hooks from config
            let config = Config::load()?;
            let configured = config.hooks.get_configured_hooks();
            if !configured.is_empty() {
                println!("\nConfigured in palrun.toml:");
                for (name, cmd) in &configured {
                    let installed = if manager.hook_exists(name) { " [installed]" } else { "" };
                    println!("  {name}: {cmd}{installed}");
                }
                println!("\nRun 'pal hooks sync' to install configured hooks.");
            }
        }

        HooksOperation::Install { hook, command, force } => {
            manager.install_hook(&hook, &command, force)?;
            println!("Installed hook: {hook}");
            println!("Command: {command}");
        }

        HooksOperation::Uninstall { hook, force } => {
            manager.uninstall_hook(&hook, force)?;
            println!("Uninstalled hook: {hook}");
        }

        HooksOperation::UninstallAll => {
            let count = manager.uninstall_all()?;
            println!("Uninstalled {} Palrun-managed hooks", count);
        }

        HooksOperation::Sync { force } => {
            let config = Config::load()?;
            let configured = config.hooks.get_configured_hooks();

            if configured.is_empty() {
                println!("No hooks configured in palrun.toml.");
                println!("\nAdd hooks to your configuration:");
                println!("  [hooks]");
                println!("  pre-commit = \"cargo test && cargo fmt --check\"");
                println!("  pre-push = \"cargo build --release\"");
                return Ok(());
            }

            println!("Syncing hooks from configuration...\n");

            let mut installed = 0;
            let mut skipped = 0;

            for (name, command) in &configured {
                if manager.hook_exists(name) && !manager.is_palrun_hook(name) && !force {
                    println!("  {name}: skipped (external hook exists, use --force)");
                    skipped += 1;
                } else {
                    manager.install_hook(name, command, force)?;
                    println!("  {name}: installed");
                    installed += 1;
                }
            }

            println!("\nSynced {installed} hooks ({skipped} skipped)");
        }
    }

    Ok(())
}

/// Handle environment commands.
fn cmd_env(operation: EnvOperation) -> Result<()> {
    use palrun::env::EnvManager;

    let cwd = std::env::current_dir()?;
    let mut manager = EnvManager::new(&cwd);

    match operation {
        EnvOperation::List => {
            manager.scan()?;
            let files = manager.get_env_files();

            if files.is_empty() {
                println!("No .env files found in the current directory.");
                println!("\nCommon .env file patterns:");
                for pattern in palrun::env::ENV_FILE_PATTERNS.iter().take(6) {
                    println!("  - {pattern}");
                }
            } else {
                println!("Detected .env files:\n");
                for file in files {
                    let active = if file.is_active { " [active]" } else { "" };
                    let vars = format!("{} vars", file.variable_count);
                    println!("  {} {} ({}){}", file.icon(), file.name, vars, active);
                }
            }
        }

        EnvOperation::Show { all, unmask, filter } => {
            // Try to load the default .env if it exists
            let default_env = cwd.join(".env");
            if default_env.exists() {
                let _ = manager.load_env_file(&default_env);
            }

            let variables = manager.get_all_variables();

            // Filter variables
            let filtered: Vec<_> = if all {
                variables
            } else {
                // Only show .env variables by default
                variables
                    .into_iter()
                    .filter(|v| matches!(v.source, palrun::env::EnvSource::DotEnv(_)))
                    .collect()
            };

            let filtered: Vec<_> = if let Some(ref pattern) = filter {
                let pattern_upper = pattern.to_uppercase();
                filtered
                    .into_iter()
                    .filter(|v| v.name.to_uppercase().contains(&pattern_upper))
                    .collect()
            } else {
                filtered
            };

            if filtered.is_empty() {
                if all {
                    println!("No environment variables found.");
                } else {
                    println!("No .env variables loaded. Use --all to show system variables.");
                }
            } else {
                println!("Environment variables:\n");
                for var in &filtered {
                    let value = if unmask || !var.is_sensitive {
                        var.value.clone()
                    } else {
                        var.masked_value()
                    };
                    let source = var.source.display();
                    let sensitive = if var.is_sensitive { " [sensitive]" } else { "" };
                    println!("  {}={} ({}){}", var.name, value, source, sensitive);
                }
                println!("\nTotal: {} variables", filtered.len());
            }
        }

        EnvOperation::Load { file } => {
            let path = if file.starts_with('.') || file.starts_with('/') {
                std::path::PathBuf::from(&file)
            } else {
                cwd.join(&file)
            };

            if !path.exists() {
                anyhow::bail!("File not found: {}", path.display());
            }

            let count = manager.load_env_file(&path)?;
            println!("Loaded {} variables from {}", count, path.display());

            // Apply to current process
            manager.apply_to_process();
            println!("Environment variables applied to current session.");
        }

        EnvOperation::Diff { file1, file2 } => {
            let path1 = cwd.join(&file1);
            let path2 = cwd.join(&file2);

            if !path1.exists() {
                anyhow::bail!("File not found: {}", file1);
            }
            if !path2.exists() {
                anyhow::bail!("File not found: {}", file2);
            }

            let diff = manager.compare_env_files(&path1, &path2)?;

            if diff.has_differences() {
                println!("Differences between {} and {}:\n", file1, file2);

                if !diff.only_in_first.is_empty() {
                    println!("Only in {}:", file1);
                    for name in &diff.only_in_first {
                        println!("  - {name}");
                    }
                    println!();
                }

                if !diff.only_in_second.is_empty() {
                    println!("Only in {}:", file2);
                    for name in &diff.only_in_second {
                        println!("  + {name}");
                    }
                    println!();
                }

                if !diff.different.is_empty() {
                    println!("Different values:");
                    for (name, val1, val2) in &diff.different {
                        println!("  {name}:");
                        println!("    {}: {}", file1, val1);
                        println!("    {}: {}", file2, val2);
                    }
                }
            } else {
                println!("No differences found between {} and {}", file1, file2);
            }
        }

        EnvOperation::Active => {
            manager.scan()?;

            if let Some(name) = manager.active_env_name() {
                println!("Active environment: {name}");
            } else {
                // Check if any .env file exists and suggest
                let files = manager.get_env_files();
                if files.is_empty() {
                    println!("No .env files found.");
                } else {
                    println!("No environment currently active.");
                    println!("\nAvailable:");
                    for file in files {
                        println!("  - {} ({})", file.name, file.env_type());
                    }
                    println!("\nUse 'pal env load <file>' to activate an environment.");
                }
            }
        }
    }

    Ok(())
}

/// Handle runtime version detection.
fn cmd_versions(show_all: bool) -> Result<()> {
    use palrun::env::{RuntimeType, VersionManager};

    let cwd = std::env::current_dir()?;
    let mut manager = VersionManager::new(&cwd);
    manager.scan()?;

    let versions = manager.get_versions();

    if versions.is_empty() {
        println!("No runtime versions detected in this project.");
        println!("\nSupported version files:");
        println!("  - .nvmrc, .node-version, package.json (Node.js)");
        println!("  - .python-version, pyproject.toml (Python)");
        println!("  - rust-toolchain.toml, Cargo.toml (Rust)");
        println!("  - go.mod (Go)");
        println!("  - .ruby-version, Gemfile (Ruby)");
        println!("  - .tool-versions (asdf/mise)");
        return Ok(());
    }

    println!("Runtime versions:\n");

    // Define the order of runtimes to display
    let runtime_order = [
        RuntimeType::Node,
        RuntimeType::Python,
        RuntimeType::Rust,
        RuntimeType::Go,
        RuntimeType::Ruby,
        RuntimeType::Java,
    ];

    for runtime in runtime_order {
        if let Some(version) = versions.get(&runtime) {
            // Skip runtimes without requirements unless --all is specified
            if !show_all && version.required.is_none() {
                continue;
            }

            let icon = version.runtime.icon();
            let name = version.runtime.name();
            let status = version.status_icon();

            let required_str = version.required.as_deref().unwrap_or("-");
            let current_str = version.current.as_deref().unwrap_or("not installed");

            let source_str = version
                .source
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("");

            println!("  {} {} {}", icon, name, status);
            println!("      Required: {} (from {})", required_str, source_str);
            println!("      Current:  {}", current_str);

            if version.is_compatible == Some(false) {
                println!("      ⚠️  Version mismatch detected!");
            }

            println!();
        }
    }

    // Summary
    let with_requirements: Vec<_> = versions.values().filter(|v| v.required.is_some()).collect();
    let incompatible: Vec<_> =
        with_requirements.iter().filter(|v| v.is_compatible == Some(false)).collect();

    if !incompatible.is_empty() {
        println!("Warning: {} runtime(s) have version mismatches", incompatible.len());
    }

    Ok(())
}

/// Handle secrets management commands.
fn cmd_secrets(operation: SecretsOperation) -> Result<()> {
    use palrun::env::{SecretProvider, SecretsManager};

    let cwd = std::env::current_dir()?;
    let mut manager = SecretsManager::new(&cwd);

    match operation {
        SecretsOperation::Status => {
            println!("Secret Providers Status:\n");

            manager.check_providers();

            // 1Password
            if let Some(status) = manager.get_provider_status("1password") {
                let icon = status.provider.icon();
                let name = status.provider.name();

                if status.installed {
                    let version = status.version.as_deref().unwrap_or("unknown");
                    let auth_status = if status.authenticated {
                        "✓ authenticated"
                    } else {
                        "⚠ not signed in"
                    };
                    println!("  {} {} (v{})", icon, name, version);
                    println!("      Status: {}", auth_status);
                } else {
                    println!("  {} {} - not installed", icon, name);
                    if let Some(ref err) = status.error {
                        println!("      {}", err);
                    }
                }
                println!();
            }

            // Vault
            if let Some(status) = manager.get_provider_status("vault") {
                let icon = status.provider.icon();
                let name = status.provider.name();

                if status.installed {
                    let version = status.version.as_deref().unwrap_or("unknown");
                    let auth_status = if status.authenticated {
                        "✓ authenticated"
                    } else {
                        "⚠ not authenticated"
                    };
                    println!("  {} {} ({})", icon, name, version);
                    println!("      Status: {}", auth_status);
                } else {
                    println!("  {} {} - not installed", icon, name);
                    if let Some(ref err) = status.error {
                        println!("      {}", err);
                    }
                }
                println!();
            }

            println!("Supported secret reference formats:");
            println!("  1Password: op://vault/item/field");
            println!("  Vault:     vault://path/to/secret#field");
        }

        SecretsOperation::Scan => {
            manager.scan_references()?;
            let refs = manager.get_references();

            if refs.is_empty() {
                println!("No secret references found in .env files.");
                println!("\nTo use secrets, add references like:");
                println!("  API_KEY=op://vault/item/field");
                println!("  DB_PASSWORD=vault://secret/database#password");
            } else {
                println!("Found {} secret reference(s):\n", refs.len());

                for reference in refs {
                    let icon = reference.provider.icon();
                    let source =
                        reference.source.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
                    println!("  {} {} (from {})", icon, reference.variable, source);
                    println!("      {}", reference.reference);
                    println!();
                }
            }
        }

        SecretsOperation::List { provider } => {
            manager.scan_references()?;

            let refs: Vec<_> = if let Some(ref p) = provider {
                let provider_type = match p.to_lowercase().as_str() {
                    "1password" | "op" => Some(SecretProvider::OnePassword),
                    "vault" => Some(SecretProvider::Vault),
                    _ => {
                        anyhow::bail!("Unknown provider: {}. Use '1password' or 'vault'.", p);
                    }
                };

                if let Some(pt) = provider_type {
                    manager.get_references_for_provider(&pt)
                } else {
                    vec![]
                }
            } else {
                manager.get_references().iter().collect()
            };

            if refs.is_empty() {
                println!("No secret references found.");
            } else {
                println!("Secret references:\n");
                for reference in &refs {
                    println!(
                        "  {}={} ({})",
                        reference.variable,
                        reference.reference,
                        reference.provider.name()
                    );
                }
                println!("\nTotal: {} reference(s)", refs.len());
            }
        }

        SecretsOperation::Inject { dry_run, provider } => {
            // Check providers first
            manager.check_providers();
            manager.scan_references()?;

            let refs = manager.get_references();

            if refs.is_empty() {
                println!("No secret references found to inject.");
                return Ok(());
            }

            // Filter by provider if specified
            let refs_to_inject: Vec<_> = if let Some(ref p) = provider {
                let provider_type = match p.to_lowercase().as_str() {
                    "1password" | "op" => Some(SecretProvider::OnePassword),
                    "vault" => Some(SecretProvider::Vault),
                    _ => {
                        anyhow::bail!("Unknown provider: {}. Use '1password' or 'vault'.", p);
                    }
                };

                if let Some(pt) = provider_type {
                    manager.get_references_for_provider(&pt)
                } else {
                    vec![]
                }
            } else {
                refs.iter().collect()
            };

            if refs_to_inject.is_empty() {
                println!("No matching secret references found.");
                return Ok(());
            }

            if dry_run {
                println!("DRY RUN - Would inject {} secret(s):\n", refs_to_inject.len());
                for reference in &refs_to_inject {
                    println!(
                        "  {} {} <- {}",
                        reference.provider.icon(),
                        reference.variable,
                        reference.reference
                    );
                }
                println!("\nRun without --dry-run to actually inject secrets.");
            } else {
                println!("Injecting {} secret(s)...\n", refs_to_inject.len());

                let mut success_count = 0;
                let mut error_count = 0;

                for reference in &refs_to_inject {
                    print!("  {} {} ... ", reference.provider.icon(), reference.variable);

                    match manager.resolve_reference(reference) {
                        Ok(resolved) => {
                            std::env::set_var(&resolved.variable, &resolved.value);
                            println!("✓");
                            success_count += 1;
                        }
                        Err(e) => {
                            println!("✗ {}", e);
                            error_count += 1;
                        }
                    }
                }

                println!();
                if error_count == 0 {
                    println!("Successfully injected {} secret(s).", success_count);
                } else {
                    println!("Injected {} secret(s), {} failed.", success_count, error_count);
                }
            }
        }
    }

    Ok(())
}

/// Parse a plugin type string.
#[cfg(feature = "plugins")]
fn parse_plugin_type(type_str: &str) -> Result<palrun::plugin::PluginType> {
    use palrun::plugin::PluginType;
    match type_str.to_lowercase().as_str() {
        "scanner" => Ok(PluginType::Scanner),
        "ai-provider" | "ai" => Ok(PluginType::AiProvider),
        "integration" => Ok(PluginType::Integration),
        "ui" => Ok(PluginType::Ui),
        _ => anyhow::bail!(
            "Unknown plugin type: {}. Use: scanner, ai-provider, integration, ui",
            type_str
        ),
    }
}

/// Handle plugin commands.
#[cfg(feature = "plugins")]
fn cmd_plugin(operation: PluginOperation) -> Result<()> {
    use palrun::plugin::{PluginManager, PluginState, RegistryClient};

    // Get plugins directory
    let plugins_dir = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?
        .join("palrun")
        .join("plugins");

    // Get cache directory for registry
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))?
        .join("palrun");

    let mut manager = PluginManager::new(plugins_dir.clone())?;

    match operation {
        PluginOperation::List { enabled, plugin_type } => {
            let plugins: Vec<_> = if enabled {
                manager.list_enabled().collect()
            } else if let Some(ref type_str) = plugin_type {
                let ptype = parse_plugin_type(type_str)?;
                manager.list_by_type(ptype).collect()
            } else {
                manager.list().collect()
            };

            if plugins.is_empty() {
                println!("No plugins installed.");
                println!("\nInstall plugins with:");
                println!("  pal plugin install <name>       # From registry");
                println!("  pal plugin install ./plugin.wasm  # From file");
                println!("\nBrowse available plugins with:");
                println!("  pal plugin browse");
            } else {
                println!("Installed plugins:\n");
                for plugin in &plugins {
                    let icon = plugin.manifest.plugin.plugin_type.icon();
                    let name = &plugin.manifest.plugin.name;
                    let version = &plugin.manifest.plugin.version;
                    let state = match plugin.state {
                        PluginState::Enabled => "enabled",
                        PluginState::Disabled => "disabled",
                        PluginState::Error => "error",
                    };
                    let state_icon = match plugin.state {
                        PluginState::Enabled => "✓",
                        PluginState::Disabled => "○",
                        PluginState::Error => "✗",
                    };

                    println!("  {} {} v{} [{} {}]", icon, name, version, state_icon, state);

                    if let Some(ref desc) = plugin.manifest.plugin.description {
                        println!("      {}", desc);
                    }

                    if plugin.state == PluginState::Error {
                        if let Some(ref err) = plugin.last_error {
                            println!("      Error: {}", err);
                        }
                    }
                }
                println!("\nTotal: {} plugin(s)", plugins.len());
            }
        }

        PluginOperation::Search { query, plugin_type, all } => {
            println!("Searching for '{}'...\n", query);

            let mut registry_client = RegistryClient::new(cache_dir)?;
            let registry = registry_client.fetch(false)?;

            let mut results = registry.search(&query);

            // Filter by type if specified
            if let Some(ref type_str) = plugin_type {
                let ptype = parse_plugin_type(type_str)?;
                results.retain(|p| p.plugin_type == ptype);
            }

            // Filter incompatible unless --all
            if !all {
                results.retain(|p| p.is_compatible());
            }

            if results.is_empty() {
                println!("No plugins found matching '{}'.", query);
                println!("\nTry browsing all plugins with:");
                println!("  pal plugin browse");
            } else {
                println!("Found {} plugin(s):\n", results.len());
                for plugin in &results {
                    let icon = plugin.plugin_type.icon();
                    let compat = if plugin.is_compatible() { "" } else { " [incompatible]" };
                    let installed =
                        if manager.get(&plugin.name).is_some() { " [installed]" } else { "" };

                    println!(
                        "  {} {} v{}{}{} ",
                        icon, plugin.name, plugin.version, installed, compat
                    );
                    println!("      {}", plugin.description);
                    println!("      ⬇ {} downloads  ⭐ {} stars", plugin.downloads, plugin.stars);
                    println!();
                }

                println!("Install with: pal plugin install <name>");
            }
        }

        PluginOperation::Browse { plugin_type, sort, refresh } => {
            println!("Fetching plugin registry...\n");

            let mut registry_client = RegistryClient::new(cache_dir)?;
            let registry = registry_client.fetch(refresh)?;

            let mut plugins: Vec<_> = if let Some(ref type_str) = plugin_type {
                let ptype = parse_plugin_type(type_str)?;
                registry.by_type(ptype)
            } else {
                registry.plugins.iter().collect()
            };

            // Sort
            match sort.to_lowercase().as_str() {
                "name" => plugins.sort_by(|a, b| a.name.cmp(&b.name)),
                "updated" => plugins.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)),
                _ => {
                    // popularity (default)
                    plugins.sort_by(|a, b| {
                        let score_a = a.downloads + a.stars * 10;
                        let score_b = b.downloads + b.stars * 10;
                        score_b.cmp(&score_a)
                    });
                }
            }

            if plugins.is_empty() {
                println!("No plugins available in the registry.");
            } else {
                println!("Available plugins ({} total):\n", plugins.len());

                for plugin in &plugins {
                    let icon = plugin.plugin_type.icon();
                    let compat = if plugin.is_compatible() { "" } else { " [incompatible]" };
                    let installed =
                        if manager.get(&plugin.name).is_some() { " [installed]" } else { "" };

                    println!(
                        "  {} {} v{}{}{}",
                        icon, plugin.name, plugin.version, installed, compat
                    );
                    println!("      {}", plugin.description);
                    if plugin.downloads > 0 || plugin.stars > 0 {
                        println!(
                            "      ⬇ {} downloads  ⭐ {} stars",
                            plugin.downloads, plugin.stars
                        );
                    }
                    println!();
                }

                println!("Install with: pal plugin install <name>");
                println!("Search with: pal plugin search <query>");
            }
        }

        PluginOperation::Install { source, force } => {
            let path = std::path::Path::new(&source);

            if path.exists() {
                // Install from local file
                println!("Installing plugin from {}...", source);

                if force {
                    // Try to uninstall first if exists
                    let _ = manager.uninstall(&source);
                }

                match manager.install_from_file(path) {
                    Ok(name) => {
                        println!("Successfully installed plugin: {}", name);
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to install plugin: {}", e);
                    }
                }
            } else {
                // Try to install from registry
                println!("Looking up '{}' in registry...", source);

                let mut registry_client = RegistryClient::new(cache_dir)?;
                let registry = registry_client.fetch(false)?;

                // Clone the plugin data to avoid borrow issues
                let plugin = registry.find(&source).cloned().ok_or_else(|| {
                    anyhow::anyhow!(
                        "Plugin '{}' not found in registry and file does not exist.\n\
                             Search with: pal plugin search <query>",
                        source
                    )
                })?;

                if !plugin.is_compatible() {
                    anyhow::bail!(
                        "Plugin '{}' requires API version {} (current: {})",
                        plugin.name,
                        plugin.api_version,
                        palrun::plugin::PLUGIN_API_VERSION
                    );
                }

                if manager.get(&plugin.name).is_some() && !force {
                    anyhow::bail!(
                        "Plugin '{}' is already installed. Use --force to reinstall.",
                        plugin.name
                    );
                }

                println!("Downloading {} v{}...", plugin.name, plugin.version);

                // Download to temp directory
                let temp_dir = tempfile::tempdir()?;
                let wasm_path = registry_client.download(&plugin, temp_dir.path())?;

                // Create a manifest file for installation
                let manifest_content = format!(
                    r#"[plugin]
name = "{}"
version = "{}"
type = "{:?}"
api_version = "{}"
description = "{}"
"#,
                    plugin.name,
                    plugin.version,
                    plugin.plugin_type,
                    plugin.api_version,
                    plugin.description
                );
                std::fs::write(temp_dir.path().join("plugin.toml"), manifest_content)?;

                // Install from downloaded file
                if force {
                    let _ = manager.uninstall(&source);
                }

                match manager.install_from_file(&wasm_path) {
                    Ok(name) => {
                        println!("Successfully installed plugin: {}", name);
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to install plugin: {}", e);
                    }
                }
            }
        }

        PluginOperation::Uninstall { name, force } => {
            if !force {
                print!("Uninstall plugin '{}'? [y/N] ", name);
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if !input.trim().eq_ignore_ascii_case("y") {
                    println!("Cancelled");
                    return Ok(());
                }
            }

            match manager.uninstall(&name) {
                Ok(()) => {
                    println!("Successfully uninstalled plugin: {}", name);
                }
                Err(e) => {
                    anyhow::bail!("Failed to uninstall plugin: {}", e);
                }
            }
        }

        PluginOperation::Enable { name } => match manager.enable(&name) {
            Ok(()) => {
                println!("Plugin '{}' enabled", name);
            }
            Err(e) => {
                anyhow::bail!("Failed to enable plugin: {}", e);
            }
        },

        PluginOperation::Disable { name } => match manager.disable(&name) {
            Ok(()) => {
                println!("Plugin '{}' disabled", name);
            }
            Err(e) => {
                anyhow::bail!("Failed to disable plugin: {}", e);
            }
        },

        PluginOperation::Info { name } => {
            // First check locally installed
            if let Some(plugin) = manager.get(&name) {
                let manifest = &plugin.manifest;
                let icon = manifest.plugin.plugin_type.icon();

                println!(
                    "{} {} v{} [installed]",
                    icon, manifest.plugin.name, manifest.plugin.version
                );
                println!();

                if let Some(ref desc) = manifest.plugin.description {
                    println!("Description: {}", desc);
                }

                if let Some(ref author) = manifest.plugin.author {
                    println!("Author: {}", author);
                }

                println!("Type: {}", manifest.plugin.plugin_type);
                println!("API Version: {}", manifest.plugin.api_version);

                let state_str = match plugin.state {
                    PluginState::Enabled => "Enabled",
                    PluginState::Disabled => "Disabled",
                    PluginState::Error => "Error",
                };
                println!("Status: {}", state_str);

                if plugin.state == PluginState::Error {
                    if let Some(ref err) = plugin.last_error {
                        println!("Last Error: {}", err);
                    }
                }

                println!();
                println!("Permissions:");
                let perms = &manifest.permissions;
                println!("  Filesystem read: {}", perms.filesystem.read);
                println!("  Filesystem write: {}", perms.filesystem.write);
                println!("  Network: {}", perms.network);
                println!("  Execute: {}", perms.execute);
                println!("  Environment: {}", perms.environment);

                if let Some(ref homepage) = manifest.plugin.homepage {
                    println!();
                    println!("Homepage: {}", homepage);
                }

                if let Some(ref repo) = manifest.plugin.repository {
                    println!("Repository: {}", repo);
                }

                if let Some(ref license) = manifest.plugin.license {
                    println!("License: {}", license);
                }
            } else {
                // Check registry
                let mut registry_client = RegistryClient::new(cache_dir)?;
                let registry = registry_client.fetch(false)?;

                if let Some(plugin) = registry.find(&name) {
                    let icon = plugin.plugin_type.icon();
                    let compat = if plugin.is_compatible() { "compatible" } else { "incompatible" };

                    println!("{} {} v{} [{}]", icon, plugin.name, plugin.version, compat);
                    println!();
                    println!("Description: {}", plugin.description);

                    if let Some(ref author) = plugin.author {
                        println!("Author: {}", author);
                    }

                    println!("Type: {:?}", plugin.plugin_type);
                    println!("API Version: {}", plugin.api_version);
                    println!();
                    println!("Downloads: {}", plugin.downloads);
                    println!("Stars: {}", plugin.stars);

                    if let Some(ref homepage) = plugin.homepage {
                        println!();
                        println!("Homepage: {}", homepage);
                    }

                    if let Some(ref repo) = plugin.repository {
                        println!("Repository: {}", repo);
                    }

                    if let Some(ref license) = plugin.license {
                        println!("License: {}", license);
                    }

                    println!();
                    println!("Install with: pal plugin install {}", name);
                } else {
                    anyhow::bail!("Plugin '{}' not found locally or in registry", name);
                }
            }
        }

        PluginOperation::Update { name, check } => {
            let mut registry_client = RegistryClient::new(cache_dir)?;
            let registry = registry_client.fetch(true)?;

            let plugins_to_check: Vec<_> = if let Some(ref plugin_name) = name {
                manager.get(plugin_name).into_iter().collect()
            } else {
                manager.list().collect()
            };

            if plugins_to_check.is_empty() {
                if let Some(ref plugin_name) = name {
                    anyhow::bail!("Plugin '{}' is not installed", plugin_name);
                }
                println!("No plugins installed.");
                return Ok(());
            }

            println!("Checking for updates...\n");

            let mut updates_available = Vec::new();

            for installed in &plugins_to_check {
                let current_version = &installed.manifest.plugin.version;

                if let Some(registry_plugin) = registry.find(&installed.manifest.plugin.name) {
                    // Simple version comparison (assumes semver)
                    if registry_plugin.version != *current_version {
                        updates_available.push((
                            installed.manifest.plugin.name.clone(),
                            current_version.clone(),
                            registry_plugin.version.clone(),
                        ));
                    }
                }
            }

            if updates_available.is_empty() {
                println!("All plugins are up to date.");
            } else {
                println!("Updates available:\n");
                for (plugin_name, current, latest) in &updates_available {
                    println!("  {} {} -> {}", plugin_name, current, latest);
                }

                if check {
                    println!("\nRun 'pal plugin update' to install updates.");
                } else {
                    // Clone plugin data from registry before we start modifying things
                    let plugins_to_download: Vec<_> = updates_available
                        .iter()
                        .filter_map(|(name, _, _)| registry.find(name).cloned())
                        .collect();

                    println!();
                    for registry_plugin in &plugins_to_download {
                        print!("Updating {}... ", registry_plugin.name);
                        io::stdout().flush()?;

                        // Uninstall and reinstall
                        if let Err(e) = manager.uninstall(&registry_plugin.name) {
                            println!("failed (uninstall): {}", e);
                            continue;
                        }

                        let temp_dir = tempfile::tempdir()?;
                        match registry_client.download(registry_plugin, temp_dir.path()) {
                            Ok(wasm_path) => {
                                let manifest_content = format!(
                                    r#"[plugin]
name = "{}"
version = "{}"
type = "{:?}"
api_version = "{}"
description = "{}"
"#,
                                    registry_plugin.name,
                                    registry_plugin.version,
                                    registry_plugin.plugin_type,
                                    registry_plugin.api_version,
                                    registry_plugin.description
                                );
                                let _ = std::fs::write(
                                    temp_dir.path().join("plugin.toml"),
                                    manifest_content,
                                );

                                match manager.install_from_file(&wasm_path) {
                                    Ok(_) => println!("done"),
                                    Err(e) => println!("failed (install): {}", e),
                                }
                            }
                            Err(e) => println!("failed (download): {}", e),
                        }
                    }
                    println!("\nUpdate complete.");
                }
            }
        }

        PluginOperation::ClearCache => {
            let registry_client = RegistryClient::new(cache_dir)?;
            registry_client.clear_cache()?;
            println!("Registry cache cleared.");
        }
    }

    Ok(())
}

/// Handle CI/CD commands.
fn cmd_ci(operation: CiOperation) -> Result<()> {
    use palrun::integrations::GitHubActions;

    // Try to create a GitHub Actions client
    let github = match GitHubActions::from_env() {
        Ok(Some(client)) => client,
        Ok(None) => {
            println!("GitHub Actions integration not configured.");
            println!();
            println!("To enable CI/CD integration, set the following:");
            println!("  GITHUB_TOKEN - Personal access token with 'repo' and 'workflow' scopes");
            println!();
            println!("Optionally set GITHUB_REPOSITORY=owner/repo, or run from a git repository");
            println!("with a GitHub remote.");
            return Ok(());
        }
        Err(e) => {
            anyhow::bail!("Failed to initialize GitHub Actions client: {}", e);
        }
    };

    match operation {
        CiOperation::Status { branch } => {
            let branch = branch
                .unwrap_or_else(|| get_current_branch().unwrap_or_else(|| "main".to_string()));

            println!(
                "CI Status for {}/{} on branch '{}':\n",
                github.owner(),
                github.repo(),
                branch
            );

            match github.get_branch_status(&branch) {
                Ok(Some(status)) => {
                    let icon = status.icon();
                    let status_text = status.to_string();
                    println!("  {} Overall: {}", icon, status_text);

                    // Show recent runs for the branch
                    if let Ok(runs) = github.list_runs(None, Some(&branch), 5) {
                        if !runs.is_empty() {
                            println!("\nRecent runs:");
                            for run in runs {
                                let conclusion = run.conclusion.unwrap_or(run.status);
                                println!(
                                    "  {} {} #{} - {}",
                                    conclusion.icon(),
                                    run.name.as_deref().unwrap_or("Workflow"),
                                    run.run_number,
                                    conclusion
                                );
                            }
                        }
                    }
                }
                Ok(None) => {
                    println!("  No workflow runs found for branch '{}'", branch);
                }
                Err(e) => {
                    println!("  Failed to get status: {}", e);
                }
            }
        }

        CiOperation::Workflows => {
            println!("Workflows for {}/{}:\n", github.owner(), github.repo());

            match github.list_workflows() {
                Ok(workflows) => {
                    if workflows.is_empty() {
                        println!("  No workflows found.");
                        println!("\nTo add workflows, create .github/workflows/*.yml files.");
                    } else {
                        for workflow in &workflows {
                            let state_icon = if workflow.state == "active" { "✓" } else { "○" };
                            println!("  {} {} ({})", state_icon, workflow.name, workflow.path);
                            println!("      ID: {}  State: {}", workflow.id, workflow.state);
                            println!();
                        }
                        println!("Total: {} workflow(s)", workflows.len());
                    }
                }
                Err(e) => {
                    anyhow::bail!("Failed to list workflows: {}", e);
                }
            }
        }

        CiOperation::Runs { workflow, branch, limit } => {
            // Find workflow ID if name provided
            let workflow_id = if let Some(ref wf) = workflow {
                // Try to parse as ID first
                if let Ok(id) = wf.parse::<u64>() {
                    Some(id)
                } else {
                    // Look up by name
                    match github.list_workflows() {
                        Ok(workflows) => workflows
                            .iter()
                            .find(|w| w.name.to_lowercase().contains(&wf.to_lowercase()))
                            .map(|w| w.id),
                        Err(_) => None,
                    }
                }
            } else {
                None
            };

            println!("Recent workflow runs for {}/{}:\n", github.owner(), github.repo());

            match github.list_runs(workflow_id, branch.as_deref(), limit) {
                Ok(runs) => {
                    if runs.is_empty() {
                        println!("  No runs found.");
                    } else {
                        for run in runs {
                            let conclusion = run.conclusion.unwrap_or(run.status);
                            let actor = run
                                .triggering_actor
                                .map(|a| a.login)
                                .unwrap_or_else(|| "unknown".to_string());

                            println!(
                                "  {} #{} {} [{}]",
                                conclusion.icon(),
                                run.run_number,
                                run.name.as_deref().unwrap_or("Workflow"),
                                run.head_branch
                            );
                            println!("      ID: {}  Status: {}  By: {}", run.id, conclusion, actor);
                            println!("      SHA: {}  {}", &run.head_sha[..7], run.updated_at);
                            println!();
                        }
                    }
                }
                Err(e) => {
                    anyhow::bail!("Failed to list runs: {}", e);
                }
            }
        }

        CiOperation::Trigger { workflow, branch, inputs } => {
            let branch = branch
                .unwrap_or_else(|| get_current_branch().unwrap_or_else(|| "main".to_string()));

            // Parse inputs if provided
            let inputs_json = if let Some(ref input_str) = inputs {
                Some(serde_json::from_str(input_str)?)
            } else {
                None
            };

            // Resolve workflow name to ID if needed
            let workflow_id = if workflow.chars().all(|c| c.is_ascii_digit()) {
                workflow.clone()
            } else {
                // Look up by name
                match github.list_workflows() {
                    Ok(workflows) => workflows
                        .iter()
                        .find(|w| {
                            w.name.to_lowercase() == workflow.to_lowercase()
                                || w.path.ends_with(&format!("{}.yml", workflow))
                                || w.path.ends_with(&format!("{}.yaml", workflow))
                        })
                        .map(|w| w.id.to_string())
                        .ok_or_else(|| anyhow::anyhow!("Workflow '{}' not found", workflow))?,
                    Err(e) => anyhow::bail!("Failed to look up workflow: {}", e),
                }
            };

            println!("Triggering workflow '{}' on branch '{}'...", workflow, branch);

            match github.trigger_workflow(&workflow_id, &branch, inputs_json) {
                Ok(()) => {
                    println!("Workflow triggered successfully!");
                    println!(
                        "\nView at: https://github.com/{}/{}/actions",
                        github.owner(),
                        github.repo()
                    );
                }
                Err(e) => {
                    anyhow::bail!("Failed to trigger workflow: {}", e);
                }
            }
        }

        CiOperation::Rerun { run_id } => {
            println!("Re-running workflow run {}...", run_id);

            match github.rerun_workflow(run_id) {
                Ok(()) => {
                    println!("Workflow re-run triggered successfully!");
                }
                Err(e) => {
                    anyhow::bail!("Failed to re-run workflow: {}", e);
                }
            }
        }

        CiOperation::Cancel { run_id } => {
            println!("Cancelling workflow run {}...", run_id);

            match github.cancel_run(run_id) {
                Ok(()) => {
                    println!("Workflow run cancelled.");
                }
                Err(e) => {
                    anyhow::bail!("Failed to cancel workflow: {}", e);
                }
            }
        }

        CiOperation::Open { run_id } => {
            let url = if let Some(id) = run_id {
                format!(
                    "https://github.com/{}/{}/actions/runs/{}",
                    github.owner(),
                    github.repo(),
                    id
                )
            } else {
                format!("https://github.com/{}/{}/actions", github.owner(), github.repo())
            };

            println!("Opening: {}", url);

            // Try to open in browser
            #[cfg(target_os = "macos")]
            {
                let _ = std::process::Command::new("open").arg(&url).spawn();
            }
            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
            }
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("cmd").args(["/c", "start", &url]).spawn();
            }
        }
    }

    Ok(())
}

/// Get the current git branch.
fn get_current_branch() -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Handle notification commands.
fn cmd_notify(operation: NotifyOperation) -> Result<()> {
    use palrun::integrations::{NotificationClient, NotificationConfig, NotificationMessage};

    let client = NotificationClient::new()
        .map_err(|e| anyhow::anyhow!("Failed to create notification client: {}", e))?;

    match operation {
        NotifyOperation::Slack { url, message, title, color } => {
            let config = NotificationConfig::slack("cli", &url);

            let mut msg = if let Some(ref t) = title {
                NotificationMessage::with_title(t, &message)
            } else {
                NotificationMessage::text(&message)
            };

            if let Some(c) = color {
                msg = msg.color(c);
            }

            println!("Sending Slack notification...");
            match client.send(&config, &msg) {
                Ok(()) => {
                    println!("Notification sent successfully!");
                }
                Err(e) => {
                    anyhow::bail!("Failed to send notification: {}", e);
                }
            }
        }

        NotifyOperation::Discord { url, message, title, color } => {
            let config = NotificationConfig::discord("cli", &url);

            let mut msg = if let Some(ref t) = title {
                NotificationMessage::with_title(t, &message)
            } else {
                NotificationMessage::text(&message)
            };

            if let Some(c) = color {
                msg = msg.color(c);
            }

            println!("Sending Discord notification...");
            match client.send(&config, &msg) {
                Ok(()) => {
                    println!("Notification sent successfully!");
                }
                Err(e) => {
                    anyhow::bail!("Failed to send notification: {}", e);
                }
            }
        }

        NotifyOperation::Webhook { url, message, title } => {
            let config = NotificationConfig::webhook("cli", &url);

            let msg = if let Some(ref t) = title {
                NotificationMessage::with_title(t, &message)
            } else {
                NotificationMessage::text(&message)
            };

            println!("Sending webhook notification...");
            match client.send(&config, &msg) {
                Ok(()) => {
                    println!("Notification sent successfully!");
                }
                Err(e) => {
                    anyhow::bail!("Failed to send notification: {}", e);
                }
            }
        }

        NotifyOperation::Test { notification_type, url } => {
            let config = match notification_type.to_lowercase().as_str() {
                "slack" => NotificationConfig::slack("test", &url),
                "discord" => NotificationConfig::discord("test", &url),
                "webhook" => NotificationConfig::webhook("test", &url),
                _ => anyhow::bail!(
                    "Unknown notification type: {}. Use: slack, discord, webhook",
                    notification_type
                ),
            };

            let msg = NotificationMessage::with_title(
                "Palrun Test Notification",
                "This is a test notification from Palrun. If you see this, your webhook is configured correctly!",
            ).color("#28a745"); // Green

            println!("Sending test notification to {}...", notification_type);
            match client.send(&config, &msg) {
                Ok(()) => {
                    println!("Test notification sent successfully!");
                    println!("Check your {} channel for the message.", notification_type);
                }
                Err(e) => {
                    anyhow::bail!("Failed to send test notification: {}", e);
                }
            }
        }
    }

    Ok(())
}

/// Handle GitHub Issues commands.
fn cmd_issues(operation: IssuesOperation) -> Result<()> {
    use palrun::integrations::{
        github_issues::{format_issue, format_stats, CreateIssueOptions, ListIssuesOptions},
        GitHubIssues,
    };

    // Create tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;

    // Try to get GitHub credentials and repo info
    let (owner, repo) = get_github_repo()?;

    let github = GitHubIssues::from_env(&owner, &repo).ok_or_else(|| {
        anyhow::anyhow!(
            "GitHub Issues integration not configured.\n\n\
             To enable, set GITHUB_TOKEN with 'repo' scope.\n\
             Optionally set GITHUB_REPOSITORY=owner/repo."
        )
    })?;

    rt.block_on(async {
        match operation {
            IssuesOperation::List { state, labels, assignee, limit } => {
                println!("Issues for {}/{}:\n", owner, repo);

                let options = ListIssuesOptions {
                    state: Some(state),
                    labels,
                    assignee,
                    per_page: Some(limit),
                    ..Default::default()
                };

                match github.list_issues(options).await {
                    Ok(issues) => {
                        if issues.is_empty() {
                            println!("  No issues found.");
                        } else {
                            for issue in &issues {
                                println!("{}", format_issue(issue, false));
                            }
                            println!("\nShowing {} issue(s)", issues.len());
                        }
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to list issues: {}", e);
                    }
                }
            }

            IssuesOperation::View { number, comments } => match github.get_issue(number).await {
                Ok(issue) => {
                    println!("{}", format_issue(&issue, true));
                    println!("\nURL: {}", issue.html_url);
                    println!("Created: {}  Updated: {}", issue.created_at, issue.updated_at);

                    if let Some(ref milestone) = issue.milestone {
                        println!("Milestone: {} ({})", milestone.title, milestone.state);
                    }

                    if comments {
                        println!("\n--- Comments ---\n");
                        match github.list_comments(number).await {
                            Ok(issue_comments) => {
                                if issue_comments.is_empty() {
                                    println!("  No comments.");
                                } else {
                                    for comment in &issue_comments {
                                        println!(
                                            "@{} ({})",
                                            comment.user.login, comment.created_at
                                        );
                                        println!("{}\n", comment.body);
                                    }
                                }
                            }
                            Err(e) => {
                                println!("  Failed to load comments: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    anyhow::bail!("Failed to get issue #{}: {}", number, e);
                }
            },

            IssuesOperation::Create { title, body, labels, assignees } => {
                let label_list = labels
                    .map(|l| l.split(',').map(|s| s.trim().to_string()).collect())
                    .unwrap_or_default();
                let assignee_list = assignees
                    .map(|a| a.split(',').map(|s| s.trim().to_string()).collect())
                    .unwrap_or_default();

                let options = CreateIssueOptions {
                    title,
                    body,
                    labels: label_list,
                    assignees: assignee_list,
                    milestone: None,
                };

                println!("Creating issue...");

                match github.create_issue(options).await {
                    Ok(issue) => {
                        println!("\nCreated issue #{}: {}", issue.number, issue.title);
                        println!("URL: {}", issue.html_url);
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to create issue: {}", e);
                    }
                }
            }

            IssuesOperation::Close { number, comment } => {
                // Add comment if provided
                if let Some(ref comment_body) = comment {
                    if let Err(e) = github.add_comment(number, comment_body).await {
                        println!("Warning: Failed to add comment: {}", e);
                    }
                }

                match github.close_issue(number).await {
                    Ok(issue) => {
                        println!("Closed issue #{}: {}", issue.number, issue.title);
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to close issue #{}: {}", number, e);
                    }
                }
            }

            IssuesOperation::Reopen { number } => match github.reopen_issue(number).await {
                Ok(issue) => {
                    println!("Reopened issue #{}: {}", issue.number, issue.title);
                }
                Err(e) => {
                    anyhow::bail!("Failed to reopen issue #{}: {}", number, e);
                }
            },

            IssuesOperation::Comment { number, body } => {
                match github.add_comment(number, &body).await {
                    Ok(comment) => {
                        println!("Added comment to issue #{}:", number);
                        println!("  {}", comment.body);
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to add comment: {}", e);
                    }
                }
            }

            IssuesOperation::Label { number, labels } => {
                let label_list: Vec<String> =
                    labels.split(',').map(|s| s.trim().to_string()).collect();

                match github.add_labels(number, label_list).await {
                    Ok(new_labels) => {
                        println!("Updated labels on issue #{}:", number);
                        for label in &new_labels {
                            println!("  - {}", label.name);
                        }
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to add labels: {}", e);
                    }
                }
            }

            IssuesOperation::Search { query } => {
                println!("Searching issues for: {}\n", query);

                match github.search_issues(&query).await {
                    Ok(issues) => {
                        if issues.is_empty() {
                            println!("  No issues found matching '{}'.", query);
                        } else {
                            for issue in &issues {
                                println!("{}", format_issue(issue, false));
                            }
                            println!("\nFound {} issue(s)", issues.len());
                        }
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to search issues: {}", e);
                    }
                }
            }

            IssuesOperation::Stats => {
                println!("Issue statistics for {}/{}:\n", owner, repo);

                // Try to get current GitHub username for user-specific stats
                let username = std::env::var("GITHUB_ACTOR").ok();

                match github.get_stats(username.as_deref()).await {
                    Ok(stats) => {
                        println!("{}", format_stats(&stats));
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to get stats: {}", e);
                    }
                }
            }

            IssuesOperation::Open { number } => {
                let url = if let Some(n) = number {
                    format!("https://github.com/{}/{}/issues/{}", owner, repo, n)
                } else {
                    format!("https://github.com/{}/{}/issues", owner, repo)
                };

                println!("Opening: {}", url);

                // Try to open in browser
                #[cfg(target_os = "macos")]
                {
                    let _ = std::process::Command::new("open").arg(&url).spawn();
                }
                #[cfg(target_os = "linux")]
                {
                    let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
                }
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("cmd").args(["/c", "start", &url]).spawn();
                }
            }
        }

        Ok(())
    })
}

/// Get the GitHub repository owner and name.
fn get_github_repo() -> Result<(String, String)> {
    // First try GITHUB_REPOSITORY env var
    if let Ok(repo) = std::env::var("GITHUB_REPOSITORY") {
        if let Some((owner, name)) = repo.split_once('/') {
            return Ok((owner.to_string(), name.to_string()));
        }
    }

    // Try to get from git remote
    let output =
        std::process::Command::new("git").args(["remote", "get-url", "origin"]).output()?;

    if output.status.success() {
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Parse GitHub URL formats:
        // git@github.com:owner/repo.git
        // https://github.com/owner/repo.git
        // https://github.com/owner/repo

        if let Some(path) = url.strip_prefix("git@github.com:") {
            let path = path.strip_suffix(".git").unwrap_or(path);
            if let Some((owner, repo)) = path.split_once('/') {
                return Ok((owner.to_string(), repo.to_string()));
            }
        } else if url.contains("github.com") {
            // Handle HTTPS URLs
            let parts: Vec<&str> = url.split('/').collect();
            if parts.len() >= 2 {
                let repo =
                    parts.last().unwrap().strip_suffix(".git").unwrap_or(parts.last().unwrap());
                let owner = parts.get(parts.len() - 2).unwrap_or(&"");
                if !owner.is_empty() && !repo.is_empty() {
                    return Ok((owner.to_string(), repo.to_string()));
                }
            }
        }
    }

    anyhow::bail!(
        "Could not determine GitHub repository.\n\
         Set GITHUB_REPOSITORY=owner/repo or run from a git repository with a GitHub remote."
    )
}

/// Handle Linear commands.
fn cmd_linear(operation: LinearOperation) -> Result<()> {
    use palrun::integrations::{
        linear::{
            format_linear_issue, format_linear_stats, CreateLinearIssueOptions,
            ListLinearIssuesOptions,
        },
        LinearClient,
    };

    // Create tokio runtime for async operations
    let rt = tokio::runtime::Runtime::new()?;

    let linear = LinearClient::from_env().ok_or_else(|| {
        anyhow::anyhow!(
            "Linear integration not configured.\n\n\
             To enable, set LINEAR_API_KEY with your Linear API key.\n\
             Generate one at: https://linear.app/settings/api"
        )
    })?;

    rt.block_on(async {
        match operation {
            LinearOperation::List { team, all, limit } => {
                // Find team ID if team key provided
                let team_id = if let Some(ref key) = team {
                    let teams = linear.list_teams().await?;
                    teams.iter().find(|t| t.key.eq_ignore_ascii_case(key)).map(|t| t.id.clone())
                } else {
                    None
                };

                let options = ListLinearIssuesOptions {
                    team_id,
                    assignee_id: Some("me".to_string()),
                    include_archived: all,
                    limit: Some(limit),
                    ..Default::default()
                };

                println!("Your Linear issues:\n");

                match linear.list_issues(options).await {
                    Ok(issues) => {
                        if issues.is_empty() {
                            println!("  No issues found.");
                        } else {
                            for issue in &issues {
                                println!("{}", format_linear_issue(issue, false));
                            }
                            println!("\nShowing {} issue(s)", issues.len());
                        }
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to list issues: {}", e);
                    }
                }
            }

            LinearOperation::View { identifier } => match linear.get_issue(&identifier).await {
                Ok(issue) => {
                    println!("{}", format_linear_issue(&issue, true));
                    println!("\nURL: {}", issue.url);
                    println!("Created: {}  Updated: {}", issue.created_at, issue.updated_at);

                    if let Some(ref due) = issue.due_date {
                        println!("Due: {}", due);
                    }
                    if let Some(estimate) = issue.estimate {
                        println!("Estimate: {} points", estimate);
                    }
                }
                Err(e) => {
                    anyhow::bail!("Failed to get issue {}: {}", identifier, e);
                }
            },

            LinearOperation::Create { title, description, team, priority } => {
                // Find team ID by key
                let teams = linear.list_teams().await?;
                let team_obj =
                    teams.iter().find(|t| t.key.eq_ignore_ascii_case(&team)).ok_or_else(|| {
                        let available: Vec<&str> = teams.iter().map(|t| t.key.as_str()).collect();
                        anyhow::anyhow!(
                            "Team '{}' not found. Available: {}",
                            team,
                            available.join(", ")
                        )
                    })?;

                let options = CreateLinearIssueOptions {
                    title,
                    description,
                    team_id: team_obj.id.clone(),
                    priority,
                    ..Default::default()
                };

                println!("Creating issue in {}...", team_obj.name);

                match linear.create_issue(options).await {
                    Ok(issue) => {
                        println!("\nCreated: {}", issue.identifier);
                        println!("Title: {}", issue.title);
                        println!("URL: {}", issue.url);
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to create issue: {}", e);
                    }
                }
            }

            LinearOperation::Teams => {
                println!("Your Linear teams:\n");

                match linear.list_teams().await {
                    Ok(teams) => {
                        if teams.is_empty() {
                            println!("  No teams found.");
                        } else {
                            for team in &teams {
                                println!("  {} - {} ({})", team.key, team.name, team.id);
                            }
                            println!("\nTotal: {} team(s)", teams.len());
                        }
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to list teams: {}", e);
                    }
                }
            }

            LinearOperation::Search { query } => {
                println!("Searching Linear for: {}\n", query);

                match linear.search_issues(&query).await {
                    Ok(issues) => {
                        if issues.is_empty() {
                            println!("  No issues found matching '{}'.", query);
                        } else {
                            for issue in &issues {
                                println!("{}", format_linear_issue(issue, false));
                            }
                            println!("\nFound {} issue(s)", issues.len());
                        }
                    }
                    Err(e) => {
                        anyhow::bail!("Failed to search issues: {}", e);
                    }
                }
            }

            LinearOperation::Stats => match linear.get_stats().await {
                Ok(stats) => {
                    println!("Your Linear statistics:\n");
                    println!("{}", format_linear_stats(&stats));
                }
                Err(e) => {
                    anyhow::bail!("Failed to get stats: {}", e);
                }
            },

            LinearOperation::Me => match linear.get_viewer().await {
                Ok(user) => {
                    println!("Logged in as:\n");
                    println!("  Name: {}", user.name);
                    println!("  Display Name: {}", user.display_name);
                    println!("  Email: {}", user.email);
                }
                Err(e) => {
                    anyhow::bail!("Failed to get user info: {}", e);
                }
            },
        }

        Ok(())
    })
}

/// Handle MCP (Model Context Protocol) commands.
fn cmd_mcp(operation: McpOperation) -> Result<()> {
    use palrun::mcp::{MCPManager, MCPServerConfig};
    use palrun::Config;

    // Load configuration (use default if not found)
    let config = Config::load().unwrap_or_default();

    match operation {
        McpOperation::Servers => {
            println!("Configured MCP servers:\n");

            if config.mcp.servers.is_empty() {
                println!("  No MCP servers configured.");
                println!("\n  Add servers to palrun.toml:");
                println!("    [[mcp.servers]]");
                println!("    name = \"my-server\"");
                println!("    command = \"/path/to/mcp-server\"");
                println!("    args = []");
            } else {
                for server in &config.mcp.servers {
                    println!("  {} - {}", server.name, server.command);
                    if !server.args.is_empty() {
                        println!("    Args: {}", server.args.join(" "));
                    }
                    if !server.env.is_empty() {
                        println!(
                            "    Env: {}",
                            server.env.keys().cloned().collect::<Vec<_>>().join(", ")
                        );
                    }
                }
                println!("\nTotal: {} server(s)", config.mcp.servers.len());
            }
        }

        McpOperation::Tools { server } => {
            if config.mcp.servers.is_empty() {
                anyhow::bail!("No MCP servers configured.");
            }

            let mut manager = MCPManager::new();

            // Filter servers
            let servers: Vec<_> = if let Some(ref name) = server {
                config.mcp.servers.iter().filter(|s| s.name == *name).collect()
            } else {
                config.mcp.servers.iter().collect()
            };

            if servers.is_empty() {
                if let Some(name) = server {
                    anyhow::bail!("Server '{}' not found.", name);
                }
                anyhow::bail!("No MCP servers configured.");
            }

            // Start servers and list tools
            for server_entry in servers {
                let mcp_config = MCPServerConfig {
                    name: server_entry.name.clone(),
                    command: server_entry.command.clone(),
                    args: server_entry.args.clone(),
                    env: server_entry.env.clone(),
                    cwd: server_entry.cwd.clone(),
                };

                let _ = manager.add_server(mcp_config);
            }

            // Start all servers
            if let Err(e) = manager.start_all() {
                eprintln!("Warning: Some servers failed to start: {}", e);
            }

            println!("Available MCP tools:\n");

            let tools = manager.list_tools();
            if tools.is_empty() {
                println!("  No tools available.");
            } else {
                for reg_tool in &tools {
                    print!("  [{}] {}", reg_tool.server, reg_tool.tool.name);
                    if let Some(ref desc) = reg_tool.tool.description {
                        print!(" - {}", desc);
                    }
                    println!();

                    // Show required parameters
                    if let Some(ref required) = reg_tool.tool.input_schema.required {
                        if !required.is_empty() {
                            println!("    Required: {}", required.join(", "));
                        }
                    }
                }
                println!("\nTotal: {} tool(s)", tools.len());
            }

            // Stop all servers
            let _ = manager.stop_all();
        }

        McpOperation::Call { server, tool, args } => {
            let server_entry = config
                .mcp
                .servers
                .iter()
                .find(|s| s.name == server)
                .ok_or_else(|| anyhow::anyhow!("Server '{}' not found.", server))?;

            let mcp_config = MCPServerConfig {
                name: server_entry.name.clone(),
                command: server_entry.command.clone(),
                args: server_entry.args.clone(),
                env: server_entry.env.clone(),
                cwd: server_entry.cwd.clone(),
            };

            let mut manager = MCPManager::new();
            let _ = manager.add_server(mcp_config);
            manager.start_all()?;

            // Parse arguments
            let arguments: Option<std::collections::HashMap<String, serde_json::Value>> =
                if let Some(ref args_json) = args {
                    Some(
                        serde_json::from_str(args_json)
                            .map_err(|e| anyhow::anyhow!("Invalid JSON arguments: {}", e))?,
                    )
                } else {
                    None
                };

            println!("Calling tool '{}' on server '{}'...\n", tool, server);

            match manager.call_tool(&tool, arguments) {
                Ok(result) => {
                    if result.is_error.unwrap_or(false) {
                        println!("Tool returned an error:");
                    }

                    for content in &result.content {
                        if let Some(text) = content.as_text() {
                            println!("{}", text);
                        }
                    }
                }
                Err(e) => {
                    anyhow::bail!("Failed to call tool: {}", e);
                }
            }

            let _ = manager.stop_all();
        }

        McpOperation::Start { name } => {
            let server_entry = config
                .mcp
                .servers
                .iter()
                .find(|s| s.name == name)
                .ok_or_else(|| anyhow::anyhow!("Server '{}' not found.", name))?;

            let mcp_config = MCPServerConfig {
                name: server_entry.name.clone(),
                command: server_entry.command.clone(),
                args: server_entry.args.clone(),
                env: server_entry.env.clone(),
                cwd: server_entry.cwd.clone(),
            };

            let mut manager = MCPManager::new();
            let _ = manager.add_server(mcp_config);

            manager.start_all()?;
            println!("Server '{}' started.", name);

            // Get tools
            let tools = manager.list_tools();
            if !tools.is_empty() {
                println!("\nAvailable tools:");
                for reg_tool in &tools {
                    print!("  {}", reg_tool.tool.name);
                    if let Some(ref desc) = reg_tool.tool.description {
                        print!(" - {}", desc);
                    }
                    println!();
                }
            }

            // Keep server running until Ctrl+C
            println!("\nServer is running. Press Ctrl+C to stop.");

            // Wait for interrupt
            let (tx, rx) = std::sync::mpsc::channel();
            ctrlc::set_handler(move || {
                let _ = tx.send(());
            })?;

            let _ = rx.recv();
            println!("\nStopping server...");
            let _ = manager.stop_all();
        }

        McpOperation::Stop { name } => {
            // In this CLI context, servers are ephemeral
            // This command is mainly useful when we add daemon mode
            println!("Server '{}' is not running (servers are ephemeral in CLI mode).", name);
            println!("\nTip: Use 'pal mcp start {}' to start a persistent server.", name);
        }

        McpOperation::Config => {
            println!("MCP Configuration:\n");

            println!("  Enabled: {}", config.mcp.enabled);
            println!("  Servers: {}", config.mcp.servers.len());

            if !config.mcp.servers.is_empty() {
                println!("\n  Configured servers:");
                for server in &config.mcp.servers {
                    println!("    - {}: {}", server.name, server.command);
                }
            }

            println!("\n  Configuration file: palrun.toml");
            println!("\n  Example configuration:");
            println!("    [mcp]");
            println!("    enabled = true");
            println!();
            println!("    [[mcp.servers]]");
            println!("    name = \"filesystem\"");
            println!("    command = \"npx\"");
            println!("    args = [\"-y\", \"@modelcontextprotocol/server-filesystem\", \".\"]");
        }
    }

    Ok(())
}

/// Handle debug commands.
fn cmd_debug(operation: DebugOperation) -> Result<()> {
    use palrun::Config;

    match operation {
        DebugOperation::Config => {
            println!("Palrun Configuration Debug\n");
            println!("{}", "=".repeat(50));

            // Show config file path
            let config_path = dirs::config_dir()
                .map(|p| p.join("palrun").join("config.toml"))
                .unwrap_or_default();
            println!("\nConfig file: {}", config_path.display());
            println!("Exists: {}", config_path.exists());

            // Load and display config
            let config = Config::load().unwrap_or_default();
            println!("\nLoaded configuration:");
            #[cfg(feature = "ai")]
            {
                println!("  AI enabled: {}", config.ai.enabled);
                println!("  AI provider: {}", config.ai.provider);
            }
            println!("  History max size: {}", config.general.max_history);
            println!("  MCP enabled: {}", config.mcp.enabled);
            println!("  MCP servers: {}", config.mcp.servers.len());
            println!("  Theme: {}", config.ui.theme);
            println!("  Show preview: {}", config.ui.show_preview);

            // Show environment variables
            println!("\nRelevant environment variables:");
            let env_vars = [
                "ANTHROPIC_API_KEY",
                "OPENAI_API_KEY",
                "OLLAMA_HOST",
                "OLLAMA_MODEL",
                "GITHUB_TOKEN",
                "RUST_LOG",
            ];
            for var in env_vars {
                let value = std::env::var(var).ok();
                let display = match &value {
                    Some(v) if var.contains("KEY") || var.contains("TOKEN") => {
                        format!("{}...", &v[..v.len().min(8)])
                    }
                    Some(v) => v.clone(),
                    None => "(not set)".to_string(),
                };
                println!("  {}: {}", var, display);
            }
        }

        DebugOperation::Commands { detailed } => {
            let mut app = App::new()?;
            app.initialize()?;

            println!("Discovered Commands Debug\n");
            println!("{}", "=".repeat(50));
            println!("\nTotal commands: {}", app.registry.len());

            // Group by source
            let mut by_source: std::collections::HashMap<&str, Vec<_>> =
                std::collections::HashMap::new();
            for cmd in app.registry.get_all() {
                by_source.entry(cmd.source.type_name()).or_default().push(cmd);
            }

            for (source, commands) in &by_source {
                println!("\n[{}] {} commands", source, commands.len());
                for cmd in commands {
                    if detailed {
                        println!("  {} - {}", cmd.name, cmd.description.as_deref().unwrap_or(""));
                        println!("    Command: {}", cmd.command);
                        if let Some(ref dir) = cmd.working_dir {
                            println!("    Dir: {}", dir.display());
                        }
                        if !cmd.tags.is_empty() {
                            println!("    Tags: {}", cmd.tags.join(", "));
                        }
                    } else {
                        println!("  {}", cmd.name);
                    }
                }
            }
        }

        DebugOperation::Project => {
            let cwd = std::env::current_dir()?;
            println!("Project Detection Debug\n");
            println!("{}", "=".repeat(50));
            println!("\nCurrent directory: {}", cwd.display());

            // Check for project files
            let project_files = [
                ("package.json", "Node.js"),
                ("Cargo.toml", "Rust"),
                ("go.mod", "Go"),
                ("pyproject.toml", "Python"),
                ("Makefile", "Make"),
                ("Taskfile.yml", "Task"),
                ("docker-compose.yml", "Docker"),
                ("nx.json", "Nx Monorepo"),
                ("turbo.json", "Turborepo"),
                (".palrun.toml", "Palrun Config"),
            ];

            println!("\nDetected project files:");
            let mut found_any = false;
            for (file, project_type) in project_files {
                if cwd.join(file).exists() {
                    println!("  [x] {} ({})", file, project_type);
                    found_any = true;
                }
            }
            if !found_any {
                println!("  (none detected)");
            }

            // Check for .palrun directory
            let palrun_dir = cwd.join(".palrun");
            println!("\nPalrun directory:");
            if palrun_dir.exists() {
                println!("  [x] .palrun/");
                if palrun_dir.join("runbooks").exists() {
                    let runbooks: Vec<_> = std::fs::read_dir(palrun_dir.join("runbooks"))
                        .map(|r| r.filter_map(|e| e.ok()).collect())
                        .unwrap_or_default();
                    println!("      runbooks/ ({} files)", runbooks.len());
                }
            } else {
                println!("  [ ] .palrun/ (not found)");
            }
        }

        DebugOperation::Search { query } => {
            use nucleo::{Config as NucleoConfig, Matcher, Utf32Str};

            let mut app = App::new()?;
            app.initialize()?;

            println!("Fuzzy Search Debug\n");
            println!("{}", "=".repeat(50));
            println!("\nQuery: \"{}\"", query);
            println!("Commands searched: {}", app.registry.len());

            let config = NucleoConfig::DEFAULT.match_paths();
            let mut matcher = Matcher::new(config);
            let pattern = nucleo::pattern::Pattern::parse(
                &query,
                nucleo::pattern::CaseMatching::Smart,
                nucleo::pattern::Normalization::Smart,
            );

            println!("\nTop matches (by score):");
            let mut scored: Vec<_> = app
                .registry
                .get_all()
                .iter()
                .filter_map(|cmd| {
                    let mut buf = vec![];
                    let haystack = Utf32Str::new(&cmd.name, &mut buf);
                    let score = pattern.score(haystack, &mut matcher)?;
                    Some((cmd, score))
                })
                .collect();

            scored.sort_by(|a, b| b.1.cmp(&a.1));

            for (cmd, score) in scored.iter().take(10) {
                println!("  {:>4} | {}", score, cmd.name);
            }

            if scored.is_empty() {
                println!("  (no matches)");
            }
        }

        #[cfg(feature = "ai")]
        DebugOperation::Ai => {
            println!("AI Provider Debug\n");
            println!("{}", "=".repeat(50));

            // Check Claude
            println!("\nClaude (Anthropic):");
            if std::env::var("ANTHROPIC_API_KEY").is_ok() {
                println!("  [x] API key set");
            } else {
                println!("  [ ] API key not set (ANTHROPIC_API_KEY)");
            }

            // Check Ollama
            println!("\nOllama:");
            let ollama_host = std::env::var("OLLAMA_HOST")
                .unwrap_or_else(|_| "http://localhost:11434".to_string());
            println!("  Host: {}", ollama_host);

            // Try to connect
            if let Ok(client) = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(2))
                .build()
            {
                let url = format!("{}/api/tags", ollama_host);
                match client.get(&url).send() {
                    Ok(resp) if resp.status().is_success() => {
                        println!("  [x] Connected to Ollama");
                        if let Ok(json) = resp.json::<serde_json::Value>() {
                            if let Some(models) = json.get("models").and_then(|m| m.as_array()) {
                                println!("  Models: {}", models.len());
                                for model in models.iter().take(5) {
                                    if let Some(name) = model.get("name").and_then(|n| n.as_str()) {
                                        println!("    - {}", name);
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        println!("  [ ] Not connected (is Ollama running?)");
                    }
                }
            }
        }

        DebugOperation::Env => {
            println!("Environment Debug\n");
            println!("{}", "=".repeat(50));

            println!("\nSystem:");
            println!("  OS: {}", std::env::consts::OS);
            println!("  Arch: {}", std::env::consts::ARCH);
            println!("  Current dir: {}", std::env::current_dir()?.display());
            println!(
                "  Home dir: {}",
                dirs::home_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "(unknown)".to_string())
            );
            println!(
                "  Config dir: {}",
                dirs::config_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "(unknown)".to_string())
            );

            println!("\nShell:");
            println!(
                "  SHELL: {}",
                std::env::var("SHELL").unwrap_or_else(|_| "(not set)".to_string())
            );
            println!(
                "  TERM: {}",
                std::env::var("TERM").unwrap_or_else(|_| "(not set)".to_string())
            );

            println!("\nPalrun version: {}", env!("CARGO_PKG_VERSION"));
        }

        DebugOperation::Scanners => {
            let mut app = App::new()?;
            app.initialize()?;

            println!("Scanner Detection Debug\n");
            println!("{}", "=".repeat(50));

            // Get unique sources
            let mut sources: std::collections::HashSet<&str> = std::collections::HashSet::new();
            for cmd in app.registry.get_all() {
                sources.insert(cmd.source.type_name());
            }

            println!("\nActive scanners (found commands):");
            for source in &sources {
                let count = app.registry.get_by_source_type(source).len();
                println!(
                    "  {} {} - {} commands",
                    app.registry
                        .get_by_source_type(source)
                        .first()
                        .map(|c| c.source.icon())
                        .unwrap_or(""),
                    source,
                    count
                );
            }

            println!("\nSupported scanners:");
            let supported = [
                "npm", "cargo", "make", "go", "python", "task", "docker", "nx", "turbo", "gradle",
                "maven", "runbook",
            ];
            for scanner in supported {
                let active = sources.contains(scanner);
                println!("  [{}] {}", if active { "x" } else { " " }, scanner);
            }
        }
    }

    Ok(())
}
