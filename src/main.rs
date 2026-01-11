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

    /// Install a plugin
    Install {
        /// Plugin source (file path, URL, or registry name)
        source: String,
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
        Some(Commands::Exec { name, yes }) => {
            cmd_exec(&name, yes)?;
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
fn cmd_exec(name: &str, skip_confirm: bool) -> Result<()> {
    let mut app = App::new()?;
    app.initialize()?;

    // Search for the command
    let matches = app.registry.search(name);

    if matches.is_empty() {
        anyhow::bail!("No command matching '{name}' found");
    }

    let cmd = app.registry.get_by_index(matches[0]).unwrap();

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
        }

        Ok(())
    })
}

/// Handle Git hooks commands.
#[cfg(feature = "git")]
fn cmd_hooks(operation: HooksOperation) -> Result<()> {
    use palrun::core::Config;
    use palrun::git::HooksManager;

    let manager = HooksManager::discover()
        .ok_or_else(|| anyhow::anyhow!("Not in a Git repository"))?;

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
                    println!(
                        "  {} {} ({}){}",
                        file.icon(),
                        file.name,
                        vars,
                        active
                    );
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

            if !diff.has_differences() {
                println!("No differences found between {} and {}", file1, file2);
            } else {
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

            let required_str = version
                .required
                .as_ref()
                .map(|v| v.as_str())
                .unwrap_or("-");
            let current_str = version
                .current
                .as_ref()
                .map(|v| v.as_str())
                .unwrap_or("not installed");

            let source_str = version
                .source
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("");

            println!("  {} {} {}", icon, name, status);
            println!("      Required: {} (from {})", required_str, source_str);
            println!("      Current:  {}", current_str);

            if let Some(false) = version.is_compatible {
                println!("      ⚠️  Version mismatch detected!");
            }

            println!();
        }
    }

    // Summary
    let with_requirements: Vec<_> = versions.values().filter(|v| v.required.is_some()).collect();
    let incompatible: Vec<_> = with_requirements
        .iter()
        .filter(|v| v.is_compatible == Some(false))
        .collect();

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
                    let source = reference
                        .source
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");
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
                    print!(
                        "  {} {} ... ",
                        reference.provider.icon(),
                        reference.variable
                    );

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
                    println!(
                        "Injected {} secret(s), {} failed.",
                        success_count, error_count
                    );
                }
            }
        }
    }

    Ok(())
}

/// Handle plugin commands.
#[cfg(feature = "plugins")]
fn cmd_plugin(operation: PluginOperation) -> Result<()> {
    use palrun::plugin::{PluginManager, PluginState, PluginType};

    // Get plugins directory
    let plugins_dir = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?
        .join("palrun")
        .join("plugins");

    let mut manager = PluginManager::new(plugins_dir)?;

    match operation {
        PluginOperation::List { enabled, plugin_type } => {
            let plugins: Vec<_> = if enabled {
                manager.list_enabled().collect()
            } else if let Some(ref type_str) = plugin_type {
                let ptype = match type_str.to_lowercase().as_str() {
                    "scanner" => PluginType::Scanner,
                    "ai-provider" | "ai" => PluginType::AiProvider,
                    "integration" => PluginType::Integration,
                    "ui" => PluginType::Ui,
                    _ => anyhow::bail!(
                        "Unknown plugin type: {}. Use: scanner, ai-provider, integration, ui",
                        type_str
                    ),
                };
                manager.list_by_type(ptype).collect()
            } else {
                manager.list().collect()
            };

            if plugins.is_empty() {
                println!("No plugins installed.");
                println!("\nInstall plugins with:");
                println!("  pal plugin install ./path/to/plugin.wasm");
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

        PluginOperation::Install { source } => {
            let path = std::path::Path::new(&source);

            if !path.exists() {
                anyhow::bail!("Plugin file not found: {}", source);
            }

            println!("Installing plugin from {}...", source);

            match manager.install_from_file(path) {
                Ok(name) => {
                    println!("Successfully installed plugin: {}", name);
                }
                Err(e) => {
                    anyhow::bail!("Failed to install plugin: {}", e);
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

        PluginOperation::Enable { name } => {
            match manager.enable(&name) {
                Ok(()) => {
                    println!("Plugin '{}' enabled", name);
                }
                Err(e) => {
                    anyhow::bail!("Failed to enable plugin: {}", e);
                }
            }
        }

        PluginOperation::Disable { name } => {
            match manager.disable(&name) {
                Ok(()) => {
                    println!("Plugin '{}' disabled", name);
                }
                Err(e) => {
                    anyhow::bail!("Failed to disable plugin: {}", e);
                }
            }
        }

        PluginOperation::Info { name } => {
            if let Some(plugin) = manager.get(&name) {
                let manifest = &plugin.manifest;
                let icon = manifest.plugin.plugin_type.icon();

                println!("{} {} v{}", icon, manifest.plugin.name, manifest.plugin.version);
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
                anyhow::bail!("Plugin '{}' not found", name);
            }
        }
    }

    Ok(())
}
