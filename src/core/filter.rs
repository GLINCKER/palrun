//! Advanced search filters for commands.
//!
//! Supports filter syntax:
//! - `#tag` - Filter by tag
//! - `source:npm` - Filter by source type
//! - `@workspace` - Filter by workspace name
//! - Text without prefixes is used for fuzzy search

use super::Command;

/// A parsed search query with filters and fuzzy pattern.
#[derive(Debug, Clone, Default)]
pub struct ParsedQuery {
    /// The fuzzy search pattern (text without filter prefixes)
    pub pattern: String,
    /// Tag filters (from #tag syntax)
    pub tags: Vec<String>,
    /// Source type filters (from source:type syntax)
    pub sources: Vec<String>,
    /// Workspace filters (from @workspace syntax)
    pub workspaces: Vec<String>,
}

impl ParsedQuery {
    /// Parse a search query into filters and pattern.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let query = ParsedQuery::parse("build #dev source:npm");
    /// assert_eq!(query.pattern, "build");
    /// assert_eq!(query.tags, vec!["dev"]);
    /// assert_eq!(query.sources, vec!["npm"]);
    /// ```
    pub fn parse(input: &str) -> Self {
        let mut result = Self::default();
        let mut pattern_parts = Vec::new();

        for token in input.split_whitespace() {
            if let Some(tag) = token.strip_prefix('#') {
                if !tag.is_empty() {
                    result.tags.push(tag.to_lowercase());
                }
            } else if let Some(source) = token.strip_prefix("source:") {
                if !source.is_empty() {
                    result.sources.push(source.to_lowercase());
                }
            } else if let Some(workspace) = token.strip_prefix('@') {
                if !workspace.is_empty() {
                    result.workspaces.push(workspace.to_lowercase());
                }
            } else {
                pattern_parts.push(token);
            }
        }

        result.pattern = pattern_parts.join(" ");
        result
    }

    /// Check if this query has any filters.
    pub fn has_filters(&self) -> bool {
        !self.tags.is_empty() || !self.sources.is_empty() || !self.workspaces.is_empty()
    }

    /// Check if a command matches all filters in this query.
    pub fn matches(&self, command: &Command) -> bool {
        // Check tag filters (command must have at least one matching tag)
        if !self.tags.is_empty() {
            let has_matching_tag = self.tags.iter().any(|filter_tag| {
                command.tags.iter().any(|cmd_tag| cmd_tag.to_lowercase() == *filter_tag)
            });
            if !has_matching_tag {
                return false;
            }
        }

        // Check source filters
        if !self.sources.is_empty() {
            let source_name = command.source.type_name().to_lowercase();
            let source_short = command.source.short_name().to_lowercase();
            let has_matching_source = self.sources.iter().any(|filter_source| {
                source_name.contains(filter_source) || source_short.contains(filter_source)
            });
            if !has_matching_source {
                return false;
            }
        }

        // Check workspace filters
        if !self.workspaces.is_empty() {
            if let Some(ref workspace) = command.workspace {
                let ws_lower = workspace.to_lowercase();
                let has_matching_ws =
                    self.workspaces.iter().any(|filter_ws| ws_lower.contains(filter_ws));
                if !has_matching_ws {
                    return false;
                }
            } else {
                // Command has no workspace but we're filtering by workspace
                return false;
            }
        }

        true
    }

    /// Get a display string showing active filters.
    pub fn filter_display(&self) -> Option<String> {
        if !self.has_filters() {
            return None;
        }

        let mut parts = Vec::new();

        for tag in &self.tags {
            parts.push(format!("#{}", tag));
        }
        for source in &self.sources {
            parts.push(format!("source:{}", source));
        }
        for ws in &self.workspaces {
            parts.push(format!("@{}", ws));
        }

        Some(parts.join(" "))
    }
}

/// Filter commands by source type.
pub fn filter_by_source<'a>(
    commands: impl Iterator<Item = &'a Command>,
    source_type: &str,
) -> Vec<&'a Command> {
    let source_lower = source_type.to_lowercase();
    commands
        .filter(|c| {
            let type_name = c.source.type_name().to_lowercase();
            let short_name = c.source.short_name().to_lowercase();
            type_name.contains(&source_lower) || short_name.contains(&source_lower)
        })
        .collect()
}

/// Filter commands by tag.
pub fn filter_by_tag<'a>(
    commands: impl Iterator<Item = &'a Command>,
    tag: &str,
) -> Vec<&'a Command> {
    let tag_lower = tag.to_lowercase();
    commands.filter(|c| c.tags.iter().any(|t| t.to_lowercase() == tag_lower)).collect()
}

/// Filter commands by workspace.
pub fn filter_by_workspace<'a>(
    commands: impl Iterator<Item = &'a Command>,
    workspace: &str,
) -> Vec<&'a Command> {
    let ws_lower = workspace.to_lowercase();
    commands
        .filter(|c| {
            c.workspace.as_ref().map(|w| w.to_lowercase().contains(&ws_lower)).unwrap_or(false)
        })
        .collect()
}

/// Get unique source types from a list of commands.
pub fn get_source_types(commands: &[Command]) -> Vec<String> {
    let mut sources: Vec<String> =
        commands.iter().map(|c| c.source.short_name().to_string()).collect();
    sources.sort();
    sources.dedup();
    sources
}

/// Get unique tags from a list of commands.
pub fn get_tags(commands: &[Command]) -> Vec<String> {
    let mut tags: Vec<String> = commands.iter().flat_map(|c| c.tags.iter().cloned()).collect();
    tags.sort();
    tags.dedup();
    tags
}

/// Get unique workspaces from a list of commands.
pub fn get_workspaces(commands: &[Command]) -> Vec<String> {
    let mut workspaces: Vec<String> = commands.iter().filter_map(|c| c.workspace.clone()).collect();
    workspaces.sort();
    workspaces.dedup();
    workspaces
}

#[cfg(test)]
mod tests {
    use super::super::CommandSource;
    use super::*;
    use std::path::PathBuf;

    fn create_test_commands() -> Vec<Command> {
        vec![
            Command::new("npm test", "npm test")
                .with_source(CommandSource::PackageJson(PathBuf::from(".")))
                .with_tag("test")
                .with_tag("dev"),
            Command::new("npm build", "npm run build")
                .with_source(CommandSource::PackageJson(PathBuf::from(".")))
                .with_tag("build"),
            Command::new("cargo test", "cargo test")
                .with_source(CommandSource::Cargo(PathBuf::from(".")))
                .with_tag("test"),
            Command::new("make build", "make build")
                .with_source(CommandSource::Makefile(PathBuf::from(".")))
                .with_tag("build"),
            Command::new("deploy", "./deploy.sh")
                .with_source(CommandSource::Manual)
                .with_workspace("frontend"),
        ]
    }

    #[test]
    fn test_parse_simple_pattern() {
        let query = ParsedQuery::parse("build");
        assert_eq!(query.pattern, "build");
        assert!(query.tags.is_empty());
        assert!(query.sources.is_empty());
        assert!(!query.has_filters());
    }

    #[test]
    fn test_parse_tag_filter() {
        let query = ParsedQuery::parse("#dev");
        assert_eq!(query.pattern, "");
        assert_eq!(query.tags, vec!["dev"]);
        assert!(query.has_filters());
    }

    #[test]
    fn test_parse_source_filter() {
        let query = ParsedQuery::parse("source:npm");
        assert_eq!(query.pattern, "");
        assert_eq!(query.sources, vec!["npm"]);
        assert!(query.has_filters());
    }

    #[test]
    fn test_parse_workspace_filter() {
        let query = ParsedQuery::parse("@frontend");
        assert_eq!(query.pattern, "");
        assert_eq!(query.workspaces, vec!["frontend"]);
        assert!(query.has_filters());
    }

    #[test]
    fn test_parse_combined() {
        let query = ParsedQuery::parse("build #dev source:npm @frontend");
        assert_eq!(query.pattern, "build");
        assert_eq!(query.tags, vec!["dev"]);
        assert_eq!(query.sources, vec!["npm"]);
        assert_eq!(query.workspaces, vec!["frontend"]);
    }

    #[test]
    fn test_parse_multiple_tags() {
        let query = ParsedQuery::parse("#dev #test");
        assert_eq!(query.tags, vec!["dev", "test"]);
    }

    #[test]
    fn test_matches_tag() {
        let commands = create_test_commands();
        let query = ParsedQuery::parse("#test");

        assert_eq!(commands.iter().filter(|c| query.matches(c)).count(), 2); // npm test and cargo test
    }

    #[test]
    fn test_matches_source() {
        let commands = create_test_commands();
        let query = ParsedQuery::parse("source:npm");

        let matching: Vec<_> = commands.iter().filter(|c| query.matches(c)).collect();
        assert_eq!(matching.len(), 2); // npm test and npm build
    }

    #[test]
    fn test_matches_workspace() {
        let commands = create_test_commands();
        let query = ParsedQuery::parse("@frontend");

        let matching: Vec<_> = commands.iter().filter(|c| query.matches(c)).collect();
        assert_eq!(matching.len(), 1); // deploy
    }

    #[test]
    fn test_matches_combined_filters() {
        let commands = create_test_commands();
        let query = ParsedQuery::parse("#test source:npm");

        // Must have #test tag AND be from npm source
        let matching: Vec<_> = commands.iter().filter(|c| query.matches(c)).collect();
        assert_eq!(matching.len(), 1); // only npm test
    }

    #[test]
    fn test_filter_display() {
        let query = ParsedQuery::parse("build #dev source:npm");
        assert_eq!(query.filter_display(), Some("#dev source:npm".to_string()));

        let query = ParsedQuery::parse("build");
        assert_eq!(query.filter_display(), None);
    }

    #[test]
    fn test_filter_by_source() {
        let commands = create_test_commands();
        let filtered = filter_by_source(commands.iter(), "npm");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_by_tag() {
        let commands = create_test_commands();
        let filtered = filter_by_tag(commands.iter(), "build");
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_get_source_types() {
        let commands = create_test_commands();
        let sources = get_source_types(&commands);
        assert!(sources.contains(&"npm".to_string()));
        assert!(sources.contains(&"cargo".to_string()));
        assert!(sources.contains(&"make".to_string()));
    }

    #[test]
    fn test_get_tags() {
        let commands = create_test_commands();
        let tags = get_tags(&commands);
        assert!(tags.contains(&"test".to_string()));
        assert!(tags.contains(&"build".to_string()));
        assert!(tags.contains(&"dev".to_string()));
    }

    #[test]
    fn test_case_insensitive_matching() {
        let commands = create_test_commands();
        let query = ParsedQuery::parse("#TEST");
        let matching: Vec<_> = commands.iter().filter(|c| query.matches(c)).collect();
        assert_eq!(matching.len(), 2); // Case insensitive
    }
}
