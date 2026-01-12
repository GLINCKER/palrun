//! Codebase analysis for AI context.
//!
//! Analyzes project structure, stack, and conventions.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

/// Codebase analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseAnalysis {
    /// Detected languages/stack
    pub stack: Vec<StackItem>,

    /// Project structure summary
    pub structure: Vec<DirectoryInfo>,

    /// Architecture patterns detected
    pub patterns: Vec<String>,

    /// Conventions detected
    pub conventions: Vec<String>,

    /// Testing information
    pub testing: TestingInfo,

    /// File statistics
    pub stats: FileStats,
}

/// Stack item (language/framework).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackItem {
    /// Name (e.g., "Rust", "TypeScript")
    pub name: String,

    /// Category (language, framework, tool)
    pub category: String,

    /// Confidence (0.0 - 1.0)
    pub confidence: f32,
}

/// Directory information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryInfo {
    /// Path relative to root
    pub path: String,

    /// Purpose/description
    pub purpose: String,

    /// File count
    pub file_count: usize,
}

/// Testing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestingInfo {
    /// Test framework detected
    pub framework: Option<String>,

    /// Test directories
    pub directories: Vec<String>,

    /// Approximate test count
    pub test_count: usize,
}

/// File statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStats {
    /// Total files
    pub total_files: usize,

    /// Lines of code (approximate)
    pub total_lines: usize,

    /// Files by extension
    pub by_extension: HashMap<String, usize>,
}

impl CodebaseAnalysis {
    /// Load from a CODEBASE.md file.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse from markdown content.
    pub fn parse(content: &str) -> anyhow::Result<Self> {
        let mut analysis = Self {
            stack: Vec::new(),
            structure: Vec::new(),
            patterns: Vec::new(),
            conventions: Vec::new(),
            testing: TestingInfo { framework: None, directories: Vec::new(), test_count: 0 },
            stats: FileStats { total_files: 0, total_lines: 0, by_extension: HashMap::new() },
        };

        let mut current_section = "";

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("## ") {
                current_section = line.trim_start_matches("## ").trim();
                continue;
            }

            if line.is_empty() {
                continue;
            }

            match current_section.to_lowercase().as_str() {
                "stack" | "technologies" | "languages" => {
                    if line.starts_with("- ") {
                        let name = line.trim_start_matches("- ").trim();
                        analysis.stack.push(StackItem {
                            name: name.to_string(),
                            category: "unknown".to_string(),
                            confidence: 1.0,
                        });
                    }
                }
                "patterns" | "architecture" => {
                    if line.starts_with("- ") {
                        analysis.patterns.push(line.trim_start_matches("- ").to_string());
                    }
                }
                "conventions" | "style" => {
                    if line.starts_with("- ") {
                        analysis.conventions.push(line.trim_start_matches("- ").to_string());
                    }
                }
                _ => {}
            }
        }

        Ok(analysis)
    }

    /// Convert to AI prompt context.
    pub fn to_context(&self, max_chars: usize) -> String {
        let mut ctx = String::from("Codebase:\n");

        // Stack
        if !self.stack.is_empty() {
            ctx.push_str("Stack: ");
            ctx.push_str(
                &self.stack.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(", "),
            );
            ctx.push('\n');
        }

        // Patterns
        if !self.patterns.is_empty() && ctx.len() < max_chars / 2 {
            ctx.push_str("Patterns: ");
            ctx.push_str(&self.patterns.join(", "));
            ctx.push('\n');
        }

        // Stats
        if self.stats.total_files > 0 && ctx.len() < max_chars {
            ctx.push_str(&format!(
                "Files: {}, Lines: ~{}\n",
                self.stats.total_files, self.stats.total_lines
            ));
        }

        ctx
    }

    /// Save to markdown file.
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let content = self.to_markdown();
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Convert to markdown.
    pub fn to_markdown(&self) -> String {
        let mut md = String::from("# Codebase Analysis\n\n");

        md.push_str("## Stack\n\n");
        for item in &self.stack {
            md.push_str(&format!("- {} ({})\n", item.name, item.category));
        }
        md.push('\n');

        md.push_str("## Structure\n\n");
        for dir in &self.structure {
            md.push_str(&format!(
                "- `{}` - {} ({} files)\n",
                dir.path, dir.purpose, dir.file_count
            ));
        }
        md.push('\n');

        if !self.patterns.is_empty() {
            md.push_str("## Patterns\n\n");
            for pattern in &self.patterns {
                md.push_str(&format!("- {pattern}\n"));
            }
            md.push('\n');
        }

        if !self.conventions.is_empty() {
            md.push_str("## Conventions\n\n");
            for convention in &self.conventions {
                md.push_str(&format!("- {convention}\n"));
            }
            md.push('\n');
        }

        md.push_str("## Testing\n\n");
        if let Some(ref framework) = self.testing.framework {
            md.push_str(&format!("- Framework: {framework}\n"));
        }
        if !self.testing.directories.is_empty() {
            md.push_str(&format!("- Directories: {}\n", self.testing.directories.join(", ")));
        }
        md.push_str(&format!("- Test count: ~{}\n\n", self.testing.test_count));

        md.push_str("## Statistics\n\n");
        md.push_str(&format!("- Total files: {}\n", self.stats.total_files));
        md.push_str(&format!("- Total lines: ~{}\n", self.stats.total_lines));
        md.push_str("- By extension:\n");
        for (ext, count) in &self.stats.by_extension {
            md.push_str(&format!("  - .{ext}: {count}\n"));
        }

        md
    }
}

/// Analyze a codebase directory.
pub fn analyze_codebase(root: &Path) -> anyhow::Result<CodebaseAnalysis> {
    let mut analysis = CodebaseAnalysis {
        stack: Vec::new(),
        structure: Vec::new(),
        patterns: Vec::new(),
        conventions: Vec::new(),
        testing: TestingInfo { framework: None, directories: Vec::new(), test_count: 0 },
        stats: FileStats { total_files: 0, total_lines: 0, by_extension: HashMap::new() },
    };

    // Detect stack from config files
    detect_stack(root, &mut analysis);

    // Analyze directory structure
    analyze_structure(root, &mut analysis);

    // Detect patterns
    detect_patterns(root, &mut analysis);

    // Detect testing setup
    detect_testing(root, &mut analysis);

    // Collect file statistics
    collect_stats(root, &mut analysis);

    Ok(analysis)
}

fn detect_stack(root: &Path, analysis: &mut CodebaseAnalysis) {
    // Rust
    if root.join("Cargo.toml").exists() {
        analysis.stack.push(StackItem {
            name: "Rust".to_string(),
            category: "language".to_string(),
            confidence: 1.0,
        });
    }

    // Node.js / TypeScript
    if root.join("package.json").exists() {
        analysis.stack.push(StackItem {
            name: "Node.js".to_string(),
            category: "runtime".to_string(),
            confidence: 1.0,
        });

        if root.join("tsconfig.json").exists() {
            analysis.stack.push(StackItem {
                name: "TypeScript".to_string(),
                category: "language".to_string(),
                confidence: 1.0,
            });
        }

        // Frameworks
        if root.join("next.config.js").exists() || root.join("next.config.ts").exists() {
            analysis.stack.push(StackItem {
                name: "Next.js".to_string(),
                category: "framework".to_string(),
                confidence: 1.0,
            });
        }
    }

    // Python
    if root.join("pyproject.toml").exists()
        || root.join("setup.py").exists()
        || root.join("requirements.txt").exists()
    {
        analysis.stack.push(StackItem {
            name: "Python".to_string(),
            category: "language".to_string(),
            confidence: 1.0,
        });
    }

    // Go
    if root.join("go.mod").exists() {
        analysis.stack.push(StackItem {
            name: "Go".to_string(),
            category: "language".to_string(),
            confidence: 1.0,
        });
    }

    // Docker
    if root.join("Dockerfile").exists() || root.join("docker-compose.yml").exists() {
        analysis.stack.push(StackItem {
            name: "Docker".to_string(),
            category: "tool".to_string(),
            confidence: 1.0,
        });
    }
}

fn analyze_structure(root: &Path, analysis: &mut CodebaseAnalysis) {
    let common_dirs = [
        ("src", "Source code"),
        ("lib", "Library code"),
        ("tests", "Test files"),
        ("test", "Test files"),
        ("docs", "Documentation"),
        ("scripts", "Build/utility scripts"),
        ("config", "Configuration files"),
        ("public", "Static assets"),
        ("assets", "Static assets"),
        ("components", "UI components"),
        ("pages", "Page components"),
        ("api", "API endpoints"),
        ("models", "Data models"),
        ("utils", "Utility functions"),
        ("helpers", "Helper functions"),
    ];

    for (dir_name, purpose) in common_dirs {
        let dir_path = root.join(dir_name);
        if dir_path.is_dir() {
            let file_count = WalkDir::new(&dir_path)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
                .count();

            if file_count > 0 {
                analysis.structure.push(DirectoryInfo {
                    path: dir_name.to_string(),
                    purpose: purpose.to_string(),
                    file_count,
                });
            }
        }
    }
}

fn detect_patterns(root: &Path, analysis: &mut CodebaseAnalysis) {
    // Check for common architectural patterns
    let src = root.join("src");

    if src.join("lib.rs").exists() && src.join("main.rs").exists() {
        analysis.patterns.push("Library + Binary crate".to_string());
    }

    if src.join("api").is_dir() || src.join("routes").is_dir() {
        analysis.patterns.push("REST API".to_string());
    }

    if src.join("components").is_dir() {
        analysis.patterns.push("Component-based UI".to_string());
    }

    if src.join("models").is_dir() || src.join("domain").is_dir() {
        analysis.patterns.push("Domain-driven design".to_string());
    }

    if root.join("Makefile").exists() || root.join("justfile").exists() {
        analysis.patterns.push("Task runner".to_string());
    }
}

fn detect_testing(root: &Path, analysis: &mut CodebaseAnalysis) {
    // Rust tests
    if root.join("Cargo.toml").exists() {
        analysis.testing.framework = Some("cargo test".to_string());
        if root.join("tests").is_dir() {
            analysis.testing.directories.push("tests".to_string());
        }
    }

    // JavaScript/TypeScript tests
    if root.join("jest.config.js").exists() || root.join("jest.config.ts").exists() {
        analysis.testing.framework = Some("Jest".to_string());
    } else if root.join("vitest.config.ts").exists() {
        analysis.testing.framework = Some("Vitest".to_string());
    }

    // Python tests
    if root.join("pytest.ini").exists() || root.join("pyproject.toml").exists() {
        analysis.testing.framework = Some("pytest".to_string());
    }

    // Count test files
    let test_patterns = ["test_", "_test", ".test.", ".spec."];
    let test_count = WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            let name = e.file_name().to_string_lossy();
            test_patterns.iter().any(|p| name.contains(p))
        })
        .count();

    analysis.testing.test_count = test_count;
}

fn collect_stats(root: &Path, analysis: &mut CodebaseAnalysis) {
    let ignore_dirs = [".git", "node_modules", "target", "dist", "build", ".next", "__pycache__"];

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !ignore_dirs.iter().any(|d| name == *d)
        })
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        analysis.stats.total_files += 1;

        // Count by extension
        if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
            *analysis.stats.by_extension.entry(ext.to_string()).or_insert(0) += 1;
        }

        // Rough line count
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            analysis.stats.total_lines += content.lines().count();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codebase_analysis_default() {
        let analysis = CodebaseAnalysis {
            stack: vec![StackItem {
                name: "Rust".to_string(),
                category: "language".to_string(),
                confidence: 1.0,
            }],
            structure: Vec::new(),
            patterns: Vec::new(),
            conventions: Vec::new(),
            testing: TestingInfo { framework: None, directories: Vec::new(), test_count: 0 },
            stats: FileStats { total_files: 10, total_lines: 500, by_extension: HashMap::new() },
        };

        let ctx = analysis.to_context(1000);
        assert!(ctx.contains("Rust"));
        assert!(ctx.contains("10"));
    }

    #[test]
    fn test_stack_item_creation() {
        let item = StackItem {
            name: "TypeScript".to_string(),
            category: "language".to_string(),
            confidence: 0.9,
        };
        assert_eq!(item.name, "TypeScript");
    }
}
