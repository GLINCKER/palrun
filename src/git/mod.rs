//! Git integration module.
//!
//! Provides Git repository detection, branch awareness, and Git operations
//! for enhanced command palette functionality.

pub mod hooks;

use std::path::{Path, PathBuf};

use git2::{BranchType, Repository, StatusOptions};

pub use hooks::HooksManager;

/// Information about a Git repository.
#[derive(Debug, Clone)]
pub struct GitInfo {
    /// Path to the repository root
    pub root: PathBuf,

    /// Current branch name (None if detached HEAD)
    pub branch: Option<String>,

    /// Whether the working directory is clean
    pub is_clean: bool,

    /// Number of staged changes
    pub staged_count: usize,

    /// Number of unstaged changes
    pub unstaged_count: usize,

    /// Number of untracked files
    pub untracked_count: usize,

    /// Commits ahead of remote
    pub ahead: usize,

    /// Commits behind remote
    pub behind: usize,

    /// Whether this is a Git worktree
    pub is_worktree: bool,

    /// Remote URL (origin)
    pub remote_url: Option<String>,
}

impl GitInfo {
    /// Get a display string for the branch (or "HEAD" if detached).
    #[must_use]
    pub fn branch_display(&self) -> &str {
        self.branch.as_deref().unwrap_or("HEAD")
    }

    /// Check if there are any changes (staged, unstaged, or untracked).
    #[must_use]
    pub const fn has_changes(&self) -> bool {
        self.staged_count > 0 || self.unstaged_count > 0 || self.untracked_count > 0
    }

    /// Get a compact status string for display.
    #[must_use]
    pub fn status_string(&self) -> String {
        let mut parts = Vec::new();

        if self.ahead > 0 {
            parts.push(format!("↑{}", self.ahead));
        }
        if self.behind > 0 {
            parts.push(format!("↓{}", self.behind));
        }
        if self.staged_count > 0 {
            parts.push(format!("●{}", self.staged_count));
        }
        if self.unstaged_count > 0 {
            parts.push(format!("✚{}", self.unstaged_count));
        }
        if self.untracked_count > 0 {
            parts.push(format!("?{}", self.untracked_count));
        }

        if parts.is_empty() {
            "✓".to_string()
        } else {
            parts.join(" ")
        }
    }
}

/// Git repository wrapper with high-level operations.
pub struct GitRepository {
    repo: Repository,
}

impl GitRepository {
    /// Open a Git repository from the given path.
    ///
    /// This will search up the directory tree to find a Git repository.
    #[must_use]
    pub fn discover(path: impl AsRef<Path>) -> Option<Self> {
        Repository::discover(path.as_ref()).ok().map(|repo| Self { repo })
    }

    /// Open a Git repository at the exact path.
    #[must_use]
    pub fn open(path: impl AsRef<Path>) -> Option<Self> {
        Repository::open(path.as_ref()).ok().map(|repo| Self { repo })
    }

    /// Get the repository root path.
    #[must_use]
    pub fn root(&self) -> Option<PathBuf> {
        self.repo.workdir().map(Path::to_path_buf)
    }

    /// Get the current branch name.
    #[must_use]
    pub fn current_branch(&self) -> Option<String> {
        let head = self.repo.head().ok()?;

        if head.is_branch() {
            head.shorthand().map(String::from)
        } else {
            // Detached HEAD - return None
            None
        }
    }

    /// Check if HEAD is detached.
    #[must_use]
    pub fn is_detached(&self) -> bool {
        self.repo.head_detached().unwrap_or(false)
    }

    /// Check if this is a Git worktree.
    #[must_use]
    pub fn is_worktree(&self) -> bool {
        self.repo.is_worktree()
    }

    /// Get the remote URL for the given remote name.
    #[must_use]
    pub fn remote_url(&self, name: &str) -> Option<String> {
        self.repo.find_remote(name).ok().and_then(|r| r.url().map(String::from))
    }

    /// Get all local branch names.
    #[must_use]
    pub fn branches(&self) -> Vec<String> {
        self.repo
            .branches(Some(BranchType::Local))
            .ok()
            .map(|branches| {
                branches
                    .filter_map(std::result::Result::ok)
                    .filter_map(|(branch, _)| branch.name().ok().flatten().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get the number of commits ahead/behind the upstream.
    #[must_use]
    pub fn ahead_behind(&self) -> (usize, usize) {
        let Ok(head) = self.repo.head() else {
            return (0, 0);
        };

        let Some(local_oid) = head.target() else {
            return (0, 0);
        };

        // Get upstream branch
        let Some(branch_name) = head.shorthand() else {
            return (0, 0);
        };

        let Ok(branch) = self.repo.find_branch(branch_name, BranchType::Local) else {
            return (0, 0);
        };

        let Ok(upstream) = branch.upstream() else {
            return (0, 0);
        };

        let Some(upstream_oid) = upstream.get().target() else {
            return (0, 0);
        };

        self.repo.graph_ahead_behind(local_oid, upstream_oid).unwrap_or((0, 0))
    }

    /// Get repository status counts.
    #[must_use]
    pub fn status_counts(&self) -> (usize, usize, usize) {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(false)
            .include_ignored(false)
            .include_unmodified(false);

        let Ok(statuses) = self.repo.statuses(Some(&mut opts)) else {
            return (0, 0, 0);
        };

        let mut staged = 0;
        let mut unstaged = 0;
        let mut untracked = 0;

        for entry in statuses.iter() {
            let status = entry.status();

            if status.is_index_new()
                || status.is_index_modified()
                || status.is_index_deleted()
                || status.is_index_renamed()
                || status.is_index_typechange()
            {
                staged += 1;
            }

            if status.is_wt_modified()
                || status.is_wt_deleted()
                || status.is_wt_renamed()
                || status.is_wt_typechange()
            {
                unstaged += 1;
            }

            if status.is_wt_new() {
                untracked += 1;
            }
        }

        (staged, unstaged, untracked)
    }

    /// Get complete Git information.
    #[must_use]
    pub fn info(&self) -> GitInfo {
        let root = self.root().unwrap_or_default();
        let branch = self.current_branch();
        let (staged_count, unstaged_count, untracked_count) = self.status_counts();
        let (ahead, behind) = self.ahead_behind();
        let is_clean = staged_count == 0 && unstaged_count == 0 && untracked_count == 0;
        let is_worktree = self.is_worktree();
        let remote_url = self.remote_url("origin");

        GitInfo {
            root,
            branch,
            is_clean,
            staged_count,
            unstaged_count,
            untracked_count,
            ahead,
            behind,
            is_worktree,
            remote_url,
        }
    }
}

/// Discover Git repository from the current directory.
#[must_use]
pub fn discover_repo() -> Option<GitRepository> {
    std::env::current_dir().ok().and_then(GitRepository::discover)
}

/// Get Git info for the current directory.
#[must_use]
pub fn current_git_info() -> Option<GitInfo> {
    discover_repo().map(|repo| repo.info())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_info_status_string_clean() {
        let info = GitInfo {
            root: PathBuf::from("/test"),
            branch: Some("main".to_string()),
            is_clean: true,
            staged_count: 0,
            unstaged_count: 0,
            untracked_count: 0,
            ahead: 0,
            behind: 0,
            is_worktree: false,
            remote_url: None,
        };

        assert_eq!(info.status_string(), "✓");
        assert!(!info.has_changes());
    }

    #[test]
    fn test_git_info_status_string_with_changes() {
        let info = GitInfo {
            root: PathBuf::from("/test"),
            branch: Some("feature/test".to_string()),
            is_clean: false,
            staged_count: 3,
            unstaged_count: 5,
            untracked_count: 2,
            ahead: 1,
            behind: 2,
            is_worktree: false,
            remote_url: None,
        };

        let status = info.status_string();
        assert!(status.contains("↑1"));
        assert!(status.contains("↓2"));
        assert!(status.contains("●3"));
        assert!(status.contains("✚5"));
        assert!(status.contains("?2"));
        assert!(info.has_changes());
    }

    #[test]
    fn test_git_info_branch_display() {
        let with_branch = GitInfo {
            root: PathBuf::from("/test"),
            branch: Some("main".to_string()),
            is_clean: true,
            staged_count: 0,
            unstaged_count: 0,
            untracked_count: 0,
            ahead: 0,
            behind: 0,
            is_worktree: false,
            remote_url: None,
        };

        let detached = GitInfo {
            root: PathBuf::from("/test"),
            branch: None,
            is_clean: true,
            staged_count: 0,
            unstaged_count: 0,
            untracked_count: 0,
            ahead: 0,
            behind: 0,
            is_worktree: false,
            remote_url: None,
        };

        assert_eq!(with_branch.branch_display(), "main");
        assert_eq!(detached.branch_display(), "HEAD");
    }

    #[test]
    fn test_discover_repo_from_current_dir() {
        // This test will work if run from within a git repo
        if let Some(repo) = discover_repo() {
            let info = repo.info();
            assert!(!info.root.as_os_str().is_empty());
        }
    }
}
