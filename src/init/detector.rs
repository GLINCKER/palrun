//! Project type detection.

use std::path::Path;

use anyhow::Result;

/// Detected project type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    /// Node.js/NPM project
    NodeJs,
    /// Next.js project
    NextJs,
    /// React project
    React,
    /// Rust/Cargo project
    Rust,
    /// Go project
    Go,
    /// Python project
    Python,
    /// Nx monorepo
    NxMonorepo,
    /// Turborepo
    Turborepo,
    /// Generic project
    Generic,
}

impl ProjectType {
    /// Get display name for the project type.
    pub fn display_name(&self) -> &str {
        match self {
            Self::NodeJs => "Node.js/NPM",
            Self::NextJs => "Next.js",
            Self::React => "React",
            Self::Rust => "Rust/Cargo",
            Self::Go => "Go",
            Self::Python => "Python",
            Self::NxMonorepo => "Nx Monorepo",
            Self::Turborepo => "Turborepo",
            Self::Generic => "Generic",
        }
    }

    /// Get recommended scanners for this project type.
    pub fn recommended_scanners(&self) -> Vec<&str> {
        match self {
            Self::NodeJs | Self::React => vec!["npm", "docker", "make"],
            Self::NextJs => vec!["npm", "docker"],
            Self::Rust => vec!["cargo", "make", "docker", "taskfile"],
            Self::Go => vec!["go", "make", "docker"],
            Self::Python => vec!["python", "make", "docker"],
            Self::NxMonorepo => vec!["npm", "nx", "docker", "make"],
            Self::Turborepo => vec!["npm", "turbo", "docker", "make"],
            Self::Generic => vec!["npm", "cargo", "make", "docker", "go", "python"],
        }
    }

    /// Get recommended ignore directories for this project type.
    pub fn recommended_ignore_dirs(&self) -> Vec<&str> {
        match self {
            Self::NodeJs | Self::React | Self::NextJs => {
                vec!["node_modules", ".git", "dist", "build", ".next", "coverage"]
            }
            Self::Rust => vec!["target", ".git", "node_modules"],
            Self::Go => vec![".git", "vendor", "bin"],
            Self::Python => vec![".git", "__pycache__", ".venv", "venv", ".pytest_cache"],
            Self::NxMonorepo => {
                vec!["node_modules", ".git", "dist", "build", ".nx", "coverage"]
            }
            Self::Turborepo => {
                vec!["node_modules", ".git", "dist", "build", ".turbo", "coverage"]
            }
            Self::Generic => vec!["node_modules", ".git", "target", "dist", "build"],
        }
    }

    /// Get recommended max depth for scanning.
    pub fn recommended_max_depth(&self) -> usize {
        match self {
            Self::NxMonorepo | Self::Turborepo => 10,
            Self::Rust => 5, // For workspaces
            _ => 5,
        }
    }

    /// Whether recursive scanning is recommended.
    pub fn recommended_recursive(&self) -> bool {
        matches!(self, Self::NxMonorepo | Self::Turborepo)
    }
}

/// Project type detector.
pub struct ProjectDetector<'a> {
    path: &'a Path,
}

impl<'a> ProjectDetector<'a> {
    /// Create a new project detector.
    pub fn new(path: &'a Path) -> Self {
        Self { path }
    }

    /// Detect the project type.
    pub fn detect(&self) -> Result<ProjectType> {
        // Check for specific frameworks first
        if self.is_nextjs() {
            return Ok(ProjectType::NextJs);
        }

        if self.is_nx_monorepo() {
            return Ok(ProjectType::NxMonorepo);
        }

        if self.is_turborepo() {
            return Ok(ProjectType::Turborepo);
        }

        // Check for language-specific projects
        if self.is_rust() {
            return Ok(ProjectType::Rust);
        }

        if self.is_go() {
            return Ok(ProjectType::Go);
        }

        if self.is_python() {
            return Ok(ProjectType::Python);
        }

        if self.is_react() {
            return Ok(ProjectType::React);
        }

        if self.is_nodejs() {
            return Ok(ProjectType::NodeJs);
        }

        // Default to generic
        Ok(ProjectType::Generic)
    }

    fn is_nextjs(&self) -> bool {
        self.path.join("next.config.js").exists()
            || self.path.join("next.config.mjs").exists()
            || self.path.join("next.config.ts").exists()
    }

    fn is_nx_monorepo(&self) -> bool {
        self.path.join("nx.json").exists()
    }

    fn is_turborepo(&self) -> bool {
        self.path.join("turbo.json").exists()
    }

    fn is_rust(&self) -> bool {
        self.path.join("Cargo.toml").exists()
    }

    fn is_go(&self) -> bool {
        self.path.join("go.mod").exists()
    }

    fn is_python(&self) -> bool {
        self.path.join("pyproject.toml").exists()
            || self.path.join("setup.py").exists()
            || self.path.join("requirements.txt").exists()
    }

    fn is_react(&self) -> bool {
        if let Ok(content) = std::fs::read_to_string(self.path.join("package.json")) {
            content.contains("\"react\"")
        } else {
            false
        }
    }

    fn is_nodejs(&self) -> bool {
        self.path.join("package.json").exists()
    }
}
