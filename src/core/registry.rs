//! Command registry for storing and searching commands.
//!
//! The registry maintains all discovered commands and provides
//! fuzzy search functionality using the nucleo library.

use std::sync::Arc;

use nucleo::{
    pattern::{CaseMatching, Normalization},
    Config, Nucleo,
};
use parking_lot::Mutex;

use super::Command;

/// Registry for storing and searching commands.
///
/// Uses nucleo for high-performance fuzzy matching.
pub struct CommandRegistry {
    /// All registered commands
    commands: Vec<Command>,

    /// Nucleo fuzzy matcher
    matcher: Arc<Mutex<Nucleo<String>>>,
}

impl std::fmt::Debug for CommandRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandRegistry").field("commands", &self.commands.len()).finish()
    }
}

impl CommandRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        let config = Config::DEFAULT.match_paths();
        let matcher = Nucleo::new(config, Arc::new(|| {}), None, 1);

        Self { commands: Vec::new(), matcher: Arc::new(Mutex::new(matcher)) }
    }

    /// Add a command to the registry.
    pub fn add(&mut self, command: Command) {
        let index = self.commands.len(); // Get index before adding
        let match_text = command.match_text();

        // Add to nucleo matcher with index as data
        {
            let matcher = self.matcher.lock();
            let injector = matcher.injector();
            // Store the index as a string for later retrieval
            injector.push(index.to_string(), {
                move |_, cols| {
                    cols[0] = match_text.as_str().into();
                }
            });
        }

        self.commands.push(command);
    }

    /// Add multiple commands at once.
    pub fn add_all(&mut self, commands: impl IntoIterator<Item = Command>) {
        for cmd in commands {
            self.add(cmd);
        }
    }

    /// Get total number of commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if registry is empty.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Get a command by its index.
    pub fn get_by_index(&self, index: usize) -> Option<&Command> {
        self.commands.get(index)
    }

    /// Get a command by its ID.
    pub fn get_by_id(&self, id: &str) -> Option<&Command> {
        self.commands.iter().find(|c| c.id == id)
    }

    /// Get all commands.
    pub fn get_all(&self) -> &[Command] {
        &self.commands
    }

    /// Search commands with fuzzy matching.
    ///
    /// Returns indices of matching commands, sorted by match score.
    pub fn search(&self, pattern: &str) -> Vec<usize> {
        if pattern.is_empty() {
            // Return all commands in order
            return (0..self.commands.len()).collect();
        }

        let mut matcher = self.matcher.lock();

        // Update the search pattern
        matcher.pattern.reparse(
            0,
            pattern,
            CaseMatching::Smart,
            Normalization::Smart,
            false, // append
        );

        // Tick to process matches
        let status = matcher.tick(10);

        // If still running, do another tick
        if status.running {
            matcher.tick(100);
        }

        // Get snapshot and collect results
        let snapshot = matcher.snapshot();
        let matched_count = snapshot.matched_item_count();

        let mut results: Vec<(usize, u32)> = (0..matched_count)
            .filter_map(|i| {
                snapshot
                    .get_matched_item(i)
                    .map(|item| (item.data.parse::<usize>().unwrap_or(0), i))
            })
            .collect();

        // Sort by score (highest first)
        results.sort_by(|a, b| b.1.cmp(&a.1));

        // Return just the indices
        results.into_iter().map(|(idx, _)| idx).collect()
    }

    /// Clear all commands from the registry.
    pub fn clear(&mut self) {
        self.commands.clear();
        // Recreate matcher
        let config = Config::DEFAULT.match_paths();
        self.matcher = Arc::new(Mutex::new(Nucleo::new(config, Arc::new(|| {}), None, 1)));
    }

    /// Remove a command by ID.
    pub fn remove(&mut self, id: &str) -> Option<Command> {
        if let Some(pos) = self.commands.iter().position(|c| c.id == id) {
            // Note: This doesn't remove from nucleo - would need to rebuild
            Some(self.commands.remove(pos))
        } else {
            None
        }
    }

    /// Get commands filtered by source type.
    pub fn get_by_source_type(&self, source_type: &str) -> Vec<&Command> {
        self.commands.iter().filter(|c| c.source.type_name() == source_type).collect()
    }

    /// Get commands filtered by tag.
    pub fn get_by_tag(&self, tag: &str) -> Vec<&Command> {
        self.commands.iter().filter(|c| c.tags.iter().any(|t| t == tag)).collect()
    }

    /// Get commands available on the given branch.
    pub fn get_by_branch(&self, branch: Option<&str>) -> Vec<&Command> {
        self.commands.iter().filter(|c| c.matches_branch(branch)).collect()
    }

    /// Search commands with branch filtering.
    ///
    /// Returns indices of matching commands that are available on the given branch.
    pub fn search_on_branch(&self, pattern: &str, branch: Option<&str>) -> Vec<usize> {
        let results = self.search(pattern);

        results
            .into_iter()
            .filter(|&idx| {
                self.commands.get(idx).map(|c| c.matches_branch(branch)).unwrap_or(false)
            })
            .collect()
    }

    /// Search commands with context-aware scoring.
    ///
    /// Returns indices sorted by combined fuzzy match score and proximity score.
    pub fn search_with_context(
        &self,
        pattern: &str,
        context: &super::CommandContext,
    ) -> Vec<usize> {
        if pattern.is_empty() {
            // Return all commands sorted by proximity
            let mut indices: Vec<usize> = (0..self.commands.len()).collect();
            indices.sort_by(|&a, &b| {
                let score_a = context.proximity_score(&self.commands[a]);
                let score_b = context.proximity_score(&self.commands[b]);
                score_b.cmp(&score_a)
            });
            return indices;
        }

        let mut matcher = self.matcher.lock();

        // Update the search pattern
        matcher.pattern.reparse(0, pattern, CaseMatching::Smart, Normalization::Smart, false);

        // Tick to process matches
        let status = matcher.tick(10);
        if status.running {
            matcher.tick(100);
        }

        // Get snapshot and collect results with scores
        let snapshot = matcher.snapshot();
        let matched_count = snapshot.matched_item_count();

        let mut results: Vec<(usize, u32, u32)> = (0..matched_count)
            .filter_map(|i| {
                snapshot.get_matched_item(i).map(|item| {
                    let idx = item.data.parse::<usize>().unwrap_or(0);
                    let fuzzy_score = i; // Lower index = better fuzzy match
                    let proximity_score =
                        self.commands.get(idx).map(|c| context.proximity_score(c)).unwrap_or(0);
                    (idx, fuzzy_score, proximity_score)
                })
            })
            .collect();

        // Sort by combined score: fuzzy match (inverted) + proximity bonus
        // Proximity adds up to 20% bonus to maintain fuzzy relevance
        results.sort_by(|a, b| {
            let combined_a = (100 - a.1.min(100)) + (a.2 / 5);
            let combined_b = (100 - b.1.min(100)) + (b.2 / 5);
            combined_b.cmp(&combined_a)
        });

        results.into_iter().map(|(idx, _, _)| idx).collect()
    }

    /// Filter search results by context filter.
    pub fn search_filtered(&self, pattern: &str, context: &super::CommandContext) -> Vec<usize> {
        let results = self.search_with_context(pattern, context);

        results
            .into_iter()
            .filter(|&idx| {
                self.commands.get(idx).map(|c| context.matches_filter(c)).unwrap_or(false)
            })
            .collect()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Manual implementation since Nucleo doesn't implement Clone
impl Clone for CommandRegistry {
    fn clone(&self) -> Self {
        let mut new_registry = Self::new();
        for cmd in &self.commands {
            new_registry.add(cmd.clone());
        }
        new_registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_commands() -> Vec<Command> {
        vec![
            Command::new("npm run build", "npm run build"),
            Command::new("npm run test", "npm run test"),
            Command::new("npm run dev", "npm run dev"),
            Command::new("make build", "make build"),
            Command::new("cargo test", "cargo test"),
        ]
    }

    #[test]
    fn test_registry_creation() {
        let registry = CommandRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_add_commands() {
        let mut registry = CommandRegistry::new();

        registry.add(Command::new("test", "npm test"));
        assert_eq!(registry.len(), 1);

        registry.add(Command::new("build", "npm run build"));
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_get_by_index() {
        let mut registry = CommandRegistry::new();
        registry.add(Command::new("test", "npm test"));

        let cmd = registry.get_by_index(0);
        assert!(cmd.is_some());
        assert_eq!(cmd.unwrap().name, "test");

        assert!(registry.get_by_index(1).is_none());
    }

    #[test]
    fn test_search_empty_pattern() {
        let mut registry = CommandRegistry::new();
        registry.add_all(create_test_commands());

        let results = registry.search("");
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn test_search_with_pattern() {
        let mut registry = CommandRegistry::new();
        registry.add_all(create_test_commands());

        let results = registry.search("build");
        assert!(!results.is_empty());

        // First result should be one of the build commands
        let first_cmd = registry.get_by_index(results[0]).unwrap();
        assert!(first_cmd.name.contains("build"));
    }

    #[test]
    fn test_fuzzy_search() {
        let mut registry = CommandRegistry::new();
        registry.add_all(create_test_commands());

        // "nrt" should fuzzy match "npm run test"
        let results = registry.search("nrt");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_clear() {
        let mut registry = CommandRegistry::new();
        registry.add_all(create_test_commands());
        assert_eq!(registry.len(), 5);

        registry.clear();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_get_by_source_type() {
        let mut registry = CommandRegistry::new();
        registry.add(
            Command::new("npm test", "npm test").with_source(
                super::super::CommandSource::PackageJson(std::path::PathBuf::from(".")),
            ),
        );
        registry.add(
            Command::new("make build", "make build")
                .with_source(super::super::CommandSource::Makefile(std::path::PathBuf::from("."))),
        );

        let npm_commands = registry.get_by_source_type("npm");
        assert_eq!(npm_commands.len(), 1);

        let make_commands = registry.get_by_source_type("make");
        assert_eq!(make_commands.len(), 1);
    }

    #[test]
    fn test_get_by_branch() {
        let mut registry = CommandRegistry::new();

        // Command available on all branches
        registry.add(Command::new("test", "npm test"));

        // Command only on main
        registry.add(Command::new("deploy", "npm run deploy").with_branch_pattern("main"));

        // Command only on feature branches
        registry.add(Command::new("dev", "npm run dev").with_branch_pattern("feature/*"));

        // On main: should get test and deploy
        let main_cmds = registry.get_by_branch(Some("main"));
        assert_eq!(main_cmds.len(), 2);

        // On feature/foo: should get test and dev
        let feature_cmds = registry.get_by_branch(Some("feature/foo"));
        assert_eq!(feature_cmds.len(), 2);

        // On develop: should only get test
        let develop_cmds = registry.get_by_branch(Some("develop"));
        assert_eq!(develop_cmds.len(), 1);
    }

    #[test]
    fn test_search_on_branch() {
        let mut registry = CommandRegistry::new();

        registry.add(Command::new("npm test", "npm test"));
        registry.add(Command::new("npm deploy", "npm run deploy").with_branch_pattern("main"));

        // Search on main branch
        let results = registry.search_on_branch("npm", Some("main"));
        assert_eq!(results.len(), 2);

        // Search on feature branch
        let results = registry.search_on_branch("npm", Some("feature/foo"));
        assert_eq!(results.len(), 1);
    }
}
