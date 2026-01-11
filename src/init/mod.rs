//! Project initialization and setup.
//!
//! This module handles intelligent project detection and configuration generation.

mod detector;
mod runbooks;
mod templates;

pub use detector::{ProjectDetector, ProjectType};

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use anyhow::{Context, Result};

use crate::core::Config;

/// Options for project setup.
#[derive(Debug, Clone)]
pub struct SetupOptions {
    /// Force overwrite existing files
    pub force: bool,
    /// Dry run - show what would be done without doing it
    pub dry_run: bool,
    /// Non-interactive mode - use defaults
    pub non_interactive: bool,
}

impl Default for SetupOptions {
    fn default() -> Self {
        Self { force: false, dry_run: false, non_interactive: false }
    }
}

/// Initialize a Palrun project with intelligent detection and configuration.
pub fn setup_project(path: &Path, options: SetupOptions) -> Result<()> {
    println!("🔍 Detecting project type...\n");

    // Detect project type
    let detector = ProjectDetector::new(path);
    let project_type = detector.detect()?;

    println!("✓ Detected: {}\n", project_type.display_name());

    // Check if .palrun.toml already exists
    let config_path = path.join(".palrun.toml");
    if config_path.exists() && !options.force && !options.dry_run {
        print!(".palrun.toml already exists. Overwrite? [y/N] ");
        io::stdout().flush()?;

        if !options.non_interactive {
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Cancelled. Use --force to overwrite without prompting.");
                return Ok(());
            }
        } else {
            println!("Cancelled. Use --force to overwrite.");
            return Ok(());
        }
    }

    // Generate configuration
    println!("📝 Generating configuration...\n");
    let config_content = templates::generate_config(project_type)?;

    // Validate the generated config
    let _config: Config = toml::from_str(&config_content)
        .context("Generated config is invalid (this is a bug, please report it)")?;

    if options.dry_run {
        println!("[DRY RUN] Would create .palrun.toml:");
        println!("{}", config_content);
    } else {
        // Write config atomically
        write_file_atomic(&config_path, &config_content)?;
        println!("✓ Created .palrun.toml");
    }

    // Create .palrun/runbooks directory
    let runbooks_dir = path.join(".palrun").join("runbooks");
    if options.dry_run {
        println!("[DRY RUN] Would create .palrun/runbooks/");
    } else {
        fs::create_dir_all(&runbooks_dir).context("Failed to create .palrun/runbooks directory")?;
        println!("✓ Created .palrun/runbooks/");
    }

    // Generate sample runbooks
    let sample_runbooks = runbooks::generate_samples(project_type)?;
    for (name, content) in &sample_runbooks {
        let runbook_path = runbooks_dir.join(name);
        if options.dry_run {
            println!("[DRY RUN] Would create .palrun/runbooks/{}", name);
        } else {
            write_file_atomic(&runbook_path, content)?;
            println!("  ✓ Created {}", name);
        }
    }

    if options.dry_run {
        println!("\n[DRY RUN] No files were created.");
        return Ok(());
    }

    // Show next steps
    println!("\n✨ Project initialized successfully!\n");
    println!("Next steps:");
    println!("  1. Review .palrun.toml and customize as needed");
    println!("  2. Check .palrun/runbooks/ for sample workflows");
    println!("  3. Run 'palrun' to see available commands");
    println!("  4. Run 'palrun runbook <name>' to execute a runbook");

    // Show suggestions
    let suggestions = get_suggestions(project_type);
    if !suggestions.is_empty() {
        println!("\n💡 Suggested next steps:");
        for suggestion in suggestions {
            println!("  - {}", suggestion);
        }
    }

    Ok(())
}

/// Write a file atomically (write to temp, then rename).
fn write_file_atomic(path: &Path, content: &str) -> Result<()> {
    let temp_path = path.with_extension("tmp");

    // Write to temp file
    fs::write(&temp_path, content)
        .with_context(|| format!("Failed to write to {}", temp_path.display()))?;

    // Rename to final location (atomic on most systems)
    fs::rename(&temp_path, path).with_context(|| {
        format!("Failed to rename {} to {}", temp_path.display(), path.display())
    })?;

    Ok(())
}

/// Get suggestions for next steps based on project type.
fn get_suggestions(project_type: ProjectType) -> Vec<String> {
    let mut suggestions = vec![
        "Set up shell integration: eval \"$(palrun init bash)\"".to_string(),
        "Try running: palrun".to_string(),
    ];

    match project_type {
        ProjectType::NodeJs | ProjectType::NextJs | ProjectType::React => {
            suggestions.push("Run: palrun runbook deploy".to_string());
        }
        ProjectType::Rust => {
            suggestions.push("Run: palrun runbook build".to_string());
        }
        ProjectType::Go => {
            suggestions.push("Run: palrun runbook build".to_string());
        }
        ProjectType::Python => {
            suggestions.push("Run: palrun runbook test".to_string());
        }
        ProjectType::NxMonorepo | ProjectType::Turborepo => {
            suggestions.push("Run: palrun runbook build-all".to_string());
        }
        ProjectType::Generic => {}
    }

    suggestions
}
