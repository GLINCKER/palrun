//! Runtime version detection module.
//!
//! Detects runtime version requirements from various configuration files
//! and compares them with currently installed versions.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Result;

/// Supported runtime types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeType {
    /// Node.js runtime
    Node,
    /// Python runtime
    Python,
    /// Rust toolchain
    Rust,
    /// Go runtime
    Go,
    /// Ruby runtime
    Ruby,
    /// Java runtime
    Java,
}

impl RuntimeType {
    /// Get the display name for the runtime.
    pub fn name(&self) -> &'static str {
        match self {
            RuntimeType::Node => "Node.js",
            RuntimeType::Python => "Python",
            RuntimeType::Rust => "Rust",
            RuntimeType::Go => "Go",
            RuntimeType::Ruby => "Ruby",
            RuntimeType::Java => "Java",
        }
    }

    /// Get an icon for the runtime.
    pub fn icon(&self) -> &'static str {
        match self {
            RuntimeType::Node => "⬢",
            RuntimeType::Python => "🐍",
            RuntimeType::Rust => "🦀",
            RuntimeType::Go => "🐹",
            RuntimeType::Ruby => "💎",
            RuntimeType::Java => "☕",
        }
    }

    /// Get the command to check the runtime version.
    pub fn version_command(&self) -> (&'static str, &'static [&'static str]) {
        match self {
            RuntimeType::Node => ("node", &["--version"]),
            RuntimeType::Python => ("python3", &["--version"]),
            RuntimeType::Rust => ("rustc", &["--version"]),
            RuntimeType::Go => ("go", &["version"]),
            RuntimeType::Ruby => ("ruby", &["--version"]),
            RuntimeType::Java => ("java", &["-version"]),
        }
    }
}

/// A detected runtime version requirement.
#[derive(Debug, Clone)]
pub struct RuntimeVersion {
    /// Type of runtime
    pub runtime: RuntimeType,

    /// Required version (from config file)
    pub required: Option<String>,

    /// Source file for the requirement
    pub source: Option<PathBuf>,

    /// Currently installed version
    pub current: Option<String>,

    /// Whether the current version satisfies the requirement
    pub is_compatible: Option<bool>,
}

impl RuntimeVersion {
    /// Create a new runtime version with detected current version.
    pub fn new(runtime: RuntimeType) -> Self {
        let current = detect_current_version(runtime);
        Self { runtime, required: None, source: None, current, is_compatible: None }
    }

    /// Set the required version.
    pub fn with_required(mut self, version: String, source: PathBuf) -> Self {
        self.required = Some(version);
        self.source = Some(source);
        self.update_compatibility();
        self
    }

    /// Update compatibility check.
    fn update_compatibility(&mut self) {
        if let (Some(ref required), Some(ref current)) = (&self.required, &self.current) {
            self.is_compatible = Some(check_version_compatibility(required, current));
        }
    }

    /// Get a status icon based on compatibility.
    pub fn status_icon(&self) -> &'static str {
        match self.is_compatible {
            Some(true) => "✓",
            Some(false) => "⚠",
            None => "?",
        }
    }
}

/// Version manager for detecting and managing runtime versions.
pub struct VersionManager {
    /// Project root directory
    root: PathBuf,

    /// Detected runtime versions
    versions: HashMap<RuntimeType, RuntimeVersion>,
}

impl VersionManager {
    /// Create a new version manager for the given project root.
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self { root: root.as_ref().to_path_buf(), versions: HashMap::new() }
    }

    /// Scan for version requirements and detect current versions.
    pub fn scan(&mut self) -> Result<&HashMap<RuntimeType, RuntimeVersion>> {
        self.versions.clear();

        // Detect Node.js
        if let Some(version) = self.detect_node_version() {
            self.versions.insert(RuntimeType::Node, version);
        }

        // Detect Python
        if let Some(version) = self.detect_python_version() {
            self.versions.insert(RuntimeType::Python, version);
        }

        // Detect Rust
        if let Some(version) = self.detect_rust_version() {
            self.versions.insert(RuntimeType::Rust, version);
        }

        // Detect Go
        if let Some(version) = self.detect_go_version() {
            self.versions.insert(RuntimeType::Go, version);
        }

        // Detect Ruby
        if let Some(version) = self.detect_ruby_version() {
            self.versions.insert(RuntimeType::Ruby, version);
        }

        Ok(&self.versions)
    }

    /// Get all detected versions.
    pub fn get_versions(&self) -> &HashMap<RuntimeType, RuntimeVersion> {
        &self.versions
    }

    /// Get a specific runtime version.
    pub fn get_version(&self, runtime: RuntimeType) -> Option<&RuntimeVersion> {
        self.versions.get(&runtime)
    }

    /// Detect Node.js version from .nvmrc, .node-version, or package.json.
    fn detect_node_version(&self) -> Option<RuntimeVersion> {
        let version = RuntimeVersion::new(RuntimeType::Node);

        // Check .nvmrc
        let nvmrc = self.root.join(".nvmrc");
        if nvmrc.exists() {
            if let Ok(content) = fs::read_to_string(&nvmrc) {
                let required = content.trim().to_string();
                if !required.is_empty() {
                    return Some(version.with_required(required, nvmrc));
                }
            }
        }

        // Check .node-version
        let node_version = self.root.join(".node-version");
        if node_version.exists() {
            if let Ok(content) = fs::read_to_string(&node_version) {
                let required = content.trim().to_string();
                if !required.is_empty() {
                    return Some(version.with_required(required, node_version));
                }
            }
        }

        // Check package.json engines.node
        let package_json = self.root.join("package.json");
        if package_json.exists() {
            if let Ok(content) = fs::read_to_string(&package_json) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(engines) = json.get("engines") {
                        if let Some(node) = engines.get("node").and_then(|v| v.as_str()) {
                            return Some(version.with_required(node.to_string(), package_json));
                        }
                    }
                }
            }
        }

        // Check .tool-versions (asdf/mise)
        let tool_versions = self.root.join(".tool-versions");
        if tool_versions.exists() {
            if let Ok(content) = fs::read_to_string(&tool_versions) {
                for line in content.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 && (parts[0] == "nodejs" || parts[0] == "node") {
                        return Some(version.with_required(parts[1].to_string(), tool_versions));
                    }
                }
            }
        }

        // Return with just current version if no requirement found
        if version.current.is_some() {
            Some(version)
        } else {
            None
        }
    }

    /// Detect Python version from .python-version or pyproject.toml.
    fn detect_python_version(&self) -> Option<RuntimeVersion> {
        let version = RuntimeVersion::new(RuntimeType::Python);

        // Check .python-version
        let python_version = self.root.join(".python-version");
        if python_version.exists() {
            if let Ok(content) = fs::read_to_string(&python_version) {
                let required = content.trim().to_string();
                if !required.is_empty() {
                    return Some(version.with_required(required, python_version));
                }
            }
        }

        // Check pyproject.toml
        let pyproject = self.root.join("pyproject.toml");
        if pyproject.exists() {
            if let Ok(content) = fs::read_to_string(&pyproject) {
                if let Ok(toml) = content.parse::<toml::Value>() {
                    // Check [project] requires-python
                    if let Some(project) = toml.get("project") {
                        if let Some(requires_python) =
                            project.get("requires-python").and_then(|v| v.as_str())
                        {
                            return Some(
                                version.with_required(requires_python.to_string(), pyproject),
                            );
                        }
                    }

                    // Check [tool.poetry.dependencies] python
                    if let Some(tool) = toml.get("tool") {
                        if let Some(poetry) = tool.get("poetry") {
                            if let Some(deps) = poetry.get("dependencies") {
                                if let Some(python) = deps.get("python").and_then(|v| v.as_str()) {
                                    return Some(
                                        version.with_required(python.to_string(), pyproject),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check .tool-versions (asdf/mise)
        let tool_versions = self.root.join(".tool-versions");
        if tool_versions.exists() {
            if let Ok(content) = fs::read_to_string(&tool_versions) {
                for line in content.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 && parts[0] == "python" {
                        return Some(version.with_required(parts[1].to_string(), tool_versions));
                    }
                }
            }
        }

        // Return with just current version if no requirement found
        if version.current.is_some() {
            Some(version)
        } else {
            None
        }
    }

    /// Detect Rust version from rust-toolchain.toml or rust-toolchain.
    fn detect_rust_version(&self) -> Option<RuntimeVersion> {
        let version = RuntimeVersion::new(RuntimeType::Rust);

        // Check rust-toolchain.toml
        let toolchain_toml = self.root.join("rust-toolchain.toml");
        if toolchain_toml.exists() {
            if let Ok(content) = fs::read_to_string(&toolchain_toml) {
                if let Ok(toml) = content.parse::<toml::Value>() {
                    if let Some(toolchain) = toml.get("toolchain") {
                        if let Some(channel) = toolchain.get("channel").and_then(|v| v.as_str()) {
                            return Some(
                                version.with_required(channel.to_string(), toolchain_toml),
                            );
                        }
                    }
                }
            }
        }

        // Check rust-toolchain (plain text)
        let toolchain = self.root.join("rust-toolchain");
        if toolchain.exists() {
            if let Ok(content) = fs::read_to_string(&toolchain) {
                let required = content.trim().to_string();
                if !required.is_empty() {
                    return Some(version.with_required(required, toolchain));
                }
            }
        }

        // Check Cargo.toml rust-version
        let cargo_toml = self.root.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = fs::read_to_string(&cargo_toml) {
                if let Ok(toml) = content.parse::<toml::Value>() {
                    if let Some(package) = toml.get("package") {
                        if let Some(rust_version) =
                            package.get("rust-version").and_then(|v| v.as_str())
                        {
                            return Some(
                                version.with_required(rust_version.to_string(), cargo_toml),
                            );
                        }
                    }
                }
            }
        }

        // Check .tool-versions (asdf/mise)
        let tool_versions = self.root.join(".tool-versions");
        if tool_versions.exists() {
            if let Ok(content) = fs::read_to_string(&tool_versions) {
                for line in content.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 && parts[0] == "rust" {
                        return Some(version.with_required(parts[1].to_string(), tool_versions));
                    }
                }
            }
        }

        // Return with just current version if no requirement found
        if version.current.is_some() {
            Some(version)
        } else {
            None
        }
    }

    /// Detect Go version from go.mod.
    fn detect_go_version(&self) -> Option<RuntimeVersion> {
        let version = RuntimeVersion::new(RuntimeType::Go);

        // Check go.mod
        let go_mod = self.root.join("go.mod");
        if go_mod.exists() {
            if let Ok(content) = fs::read_to_string(&go_mod) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("go ") {
                        let go_version = trimmed.strip_prefix("go ").unwrap().trim();
                        return Some(version.with_required(go_version.to_string(), go_mod));
                    }
                }
            }
        }

        // Check .tool-versions (asdf/mise)
        let tool_versions = self.root.join(".tool-versions");
        if tool_versions.exists() {
            if let Ok(content) = fs::read_to_string(&tool_versions) {
                for line in content.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 && (parts[0] == "golang" || parts[0] == "go") {
                        return Some(version.with_required(parts[1].to_string(), tool_versions));
                    }
                }
            }
        }

        // Return with just current version if no requirement found
        if version.current.is_some() {
            Some(version)
        } else {
            None
        }
    }

    /// Detect Ruby version from .ruby-version or Gemfile.
    fn detect_ruby_version(&self) -> Option<RuntimeVersion> {
        let version = RuntimeVersion::new(RuntimeType::Ruby);

        // Check .ruby-version
        let ruby_version = self.root.join(".ruby-version");
        if ruby_version.exists() {
            if let Ok(content) = fs::read_to_string(&ruby_version) {
                let required = content.trim().to_string();
                if !required.is_empty() {
                    return Some(version.with_required(required, ruby_version));
                }
            }
        }

        // Check Gemfile for ruby directive
        let gemfile = self.root.join("Gemfile");
        if gemfile.exists() {
            if let Ok(content) = fs::read_to_string(&gemfile) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    // Match ruby "version" or ruby 'version'
                    if trimmed.starts_with("ruby ") {
                        if let Some(version_str) = extract_quoted_string(trimmed) {
                            return Some(version.with_required(version_str, gemfile));
                        }
                    }
                }
            }
        }

        // Check .tool-versions (asdf/mise)
        let tool_versions = self.root.join(".tool-versions");
        if tool_versions.exists() {
            if let Ok(content) = fs::read_to_string(&tool_versions) {
                for line in content.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 && parts[0] == "ruby" {
                        return Some(version.with_required(parts[1].to_string(), tool_versions));
                    }
                }
            }
        }

        // Return with just current version if no requirement found
        if version.current.is_some() {
            Some(version)
        } else {
            None
        }
    }
}

/// Detect the current installed version of a runtime.
fn detect_current_version(runtime: RuntimeType) -> Option<String> {
    let (cmd, args) = runtime.version_command();

    let output = Command::new(cmd).args(args).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Java outputs to stderr
    let version_output = if runtime == RuntimeType::Java && !stderr.is_empty() {
        stderr.to_string()
    } else {
        stdout.to_string()
    };

    // Parse version from output
    parse_version_output(runtime, &version_output)
}

/// Parse version from command output.
fn parse_version_output(runtime: RuntimeType, output: &str) -> Option<String> {
    let first_line = output.lines().next()?;

    match runtime {
        RuntimeType::Node => {
            // "v20.10.0" -> "20.10.0"
            first_line.strip_prefix('v').map(|s| s.to_string())
        }
        RuntimeType::Python => {
            // "Python 3.12.0" -> "3.12.0"
            first_line.strip_prefix("Python ").map(|s| s.trim().to_string())
        }
        RuntimeType::Rust => {
            // "rustc 1.82.0 (f6e511eec 2024-10-15)" -> "1.82.0"
            first_line
                .strip_prefix("rustc ")
                .and_then(|s| s.split_whitespace().next())
                .map(|s| s.to_string())
        }
        RuntimeType::Go => {
            // "go version go1.22.0 darwin/arm64" -> "1.22.0"
            first_line
                .split_whitespace()
                .find(|s| s.starts_with("go1."))
                .and_then(|s| s.strip_prefix("go"))
                .map(|s| s.to_string())
        }
        RuntimeType::Ruby => {
            // "ruby 3.3.0 (2023-12-25 revision 5124f9ac75) [arm64-darwin23]" -> "3.3.0"
            first_line
                .strip_prefix("ruby ")
                .and_then(|s| s.split_whitespace().next())
                .map(|s| s.to_string())
        }
        RuntimeType::Java => {
            // 'java version "21.0.1"' or 'openjdk version "21.0.1"' -> "21.0.1"
            extract_quoted_string(first_line)
        }
    }
}

/// Extract a quoted string from text.
fn extract_quoted_string(text: &str) -> Option<String> {
    // Try double quotes first
    if let Some(start) = text.find('"') {
        if let Some(end) = text[start + 1..].find('"') {
            return Some(text[start + 1..start + 1 + end].to_string());
        }
    }

    // Try single quotes
    if let Some(start) = text.find('\'') {
        if let Some(end) = text[start + 1..].find('\'') {
            return Some(text[start + 1..start + 1 + end].to_string());
        }
    }

    None
}

/// Check if the current version satisfies the required version.
fn check_version_compatibility(required: &str, current: &str) -> bool {
    // Clean up version strings
    let required = required.trim().trim_start_matches('v');
    let current = current.trim().trim_start_matches('v');

    // Handle channel names (stable, nightly, beta) - always compatible if we have a version
    let channel_keywords = ["stable", "nightly", "beta", "latest"];
    if channel_keywords.iter().any(|k| required.eq_ignore_ascii_case(k)) {
        return true;
    }

    // Handle semver ranges
    if required.starts_with('^') || required.starts_with('~') {
        return check_semver_range(required, current);
    }

    // Handle >= constraints
    if required.starts_with(">=") {
        let min_version = required.strip_prefix(">=").unwrap().trim();
        return compare_versions(current, min_version) >= 0;
    }

    // Handle > constraints
    if required.starts_with('>') && !required.starts_with(">=") {
        let min_version = required.strip_prefix('>').unwrap().trim();
        return compare_versions(current, min_version) > 0;
    }

    // Handle <= constraints
    if required.starts_with("<=") {
        let max_version = required.strip_prefix("<=").unwrap().trim();
        return compare_versions(current, max_version) <= 0;
    }

    // Handle < constraints
    if required.starts_with('<') && !required.starts_with("<=") {
        let max_version = required.strip_prefix('<').unwrap().trim();
        return compare_versions(current, max_version) < 0;
    }

    // Handle exact match or "lts/*" style
    if required.contains("lts") || required == "*" {
        return true; // Assume compatible with LTS or wildcard
    }

    // Direct comparison (major.minor match)
    let req_parts: Vec<&str> = required.split('.').collect();
    let cur_parts: Vec<&str> = current.split('.').collect();

    // Match major version at minimum
    if req_parts.is_empty() || cur_parts.is_empty() {
        return true; // Can't compare, assume compatible
    }

    // If required has only major version, check major version
    if req_parts.len() == 1 {
        return req_parts[0] == cur_parts[0];
    }

    // If required has major.minor, check both
    if req_parts.len() == 2 {
        return req_parts[0] == cur_parts[0]
            && cur_parts.len() > 1
            && compare_versions(current, required) >= 0;
    }

    // Full version comparison
    compare_versions(current, required) >= 0
}

/// Check semver range compatibility.
fn check_semver_range(required: &str, current: &str) -> bool {
    let is_caret = required.starts_with('^');
    let version = required.trim_start_matches('^').trim_start_matches('~');

    let req_parts: Vec<u32> = version.split('.').filter_map(|s| s.parse().ok()).collect();
    let cur_parts: Vec<u32> = current.split('.').filter_map(|s| s.parse().ok()).collect();

    if req_parts.is_empty() || cur_parts.is_empty() {
        return true;
    }

    // For caret (^), major must match and current >= required
    if is_caret {
        if cur_parts[0] != req_parts[0] {
            return false;
        }
        return compare_versions(current, version) >= 0;
    }

    // For tilde (~), major and minor must match and current >= required
    if cur_parts[0] != req_parts[0] {
        return false;
    }
    if req_parts.len() > 1 && cur_parts.len() > 1 && cur_parts[1] != req_parts[1] {
        return false;
    }
    compare_versions(current, version) >= 0
}

/// Compare two version strings.
/// Returns: -1 if a < b, 0 if a == b, 1 if a > b
fn compare_versions(a: &str, b: &str) -> i32 {
    let a_parts: Vec<u32> = a
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect();
    let b_parts: Vec<u32> = b
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect();

    let max_len = a_parts.len().max(b_parts.len());

    for i in 0..max_len {
        let a_val = a_parts.get(i).copied().unwrap_or(0);
        let b_val = b_parts.get(i).copied().unwrap_or(0);

        if a_val < b_val {
            return -1;
        }
        if a_val > b_val {
            return 1;
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_node_version_from_nvmrc() {
        let temp = TempDir::new().unwrap();
        create_test_file(temp.path(), ".nvmrc", "20.10.0");

        let mut manager = VersionManager::new(temp.path());
        manager.scan().unwrap();

        let node = manager.get_version(RuntimeType::Node).unwrap();
        assert_eq!(node.required, Some("20.10.0".to_string()));
    }

    #[test]
    fn test_node_version_from_package_json() {
        let temp = TempDir::new().unwrap();
        create_test_file(temp.path(), "package.json", r#"{"engines": {"node": ">=18.0.0"}}"#);

        let mut manager = VersionManager::new(temp.path());
        manager.scan().unwrap();

        let node = manager.get_version(RuntimeType::Node).unwrap();
        assert_eq!(node.required, Some(">=18.0.0".to_string()));
    }

    #[test]
    fn test_rust_version_from_toolchain_toml() {
        let temp = TempDir::new().unwrap();
        create_test_file(
            temp.path(),
            "rust-toolchain.toml",
            r#"[toolchain]
channel = "1.82"
"#,
        );

        let mut manager = VersionManager::new(temp.path());
        manager.scan().unwrap();

        let rust = manager.get_version(RuntimeType::Rust).unwrap();
        assert_eq!(rust.required, Some("1.82".to_string()));
    }

    #[test]
    fn test_rust_version_from_cargo_toml() {
        let temp = TempDir::new().unwrap();
        create_test_file(
            temp.path(),
            "Cargo.toml",
            r#"[package]
name = "test"
version = "0.1.0"
rust-version = "1.82"
"#,
        );

        let mut manager = VersionManager::new(temp.path());
        manager.scan().unwrap();

        let rust = manager.get_version(RuntimeType::Rust).unwrap();
        assert_eq!(rust.required, Some("1.82".to_string()));
    }

    #[test]
    fn test_python_version_from_file() {
        let temp = TempDir::new().unwrap();
        create_test_file(temp.path(), ".python-version", "3.12.0");

        let mut manager = VersionManager::new(temp.path());
        manager.scan().unwrap();

        let python = manager.get_version(RuntimeType::Python).unwrap();
        assert_eq!(python.required, Some("3.12.0".to_string()));
    }

    #[test]
    fn test_python_version_from_pyproject() {
        let temp = TempDir::new().unwrap();
        create_test_file(
            temp.path(),
            "pyproject.toml",
            r#"[project]
requires-python = ">=3.10"
"#,
        );

        let mut manager = VersionManager::new(temp.path());
        manager.scan().unwrap();

        let python = manager.get_version(RuntimeType::Python).unwrap();
        assert_eq!(python.required, Some(">=3.10".to_string()));
    }

    #[test]
    fn test_go_version_from_go_mod() {
        let temp = TempDir::new().unwrap();
        create_test_file(
            temp.path(),
            "go.mod",
            r"module example.com/test

go 1.22
",
        );

        let mut manager = VersionManager::new(temp.path());
        manager.scan().unwrap();

        let go = manager.get_version(RuntimeType::Go).unwrap();
        assert_eq!(go.required, Some("1.22".to_string()));
    }

    #[test]
    fn test_tool_versions_file() {
        let temp = TempDir::new().unwrap();
        create_test_file(
            temp.path(),
            ".tool-versions",
            r"nodejs 20.10.0
python 3.12.0
rust 1.82.0
golang 1.22.0
ruby 3.3.0
",
        );

        let mut manager = VersionManager::new(temp.path());
        manager.scan().unwrap();

        // All runtimes should be detected
        assert_eq!(
            manager.get_version(RuntimeType::Node).unwrap().required,
            Some("20.10.0".to_string())
        );
        assert_eq!(
            manager.get_version(RuntimeType::Python).unwrap().required,
            Some("3.12.0".to_string())
        );
        assert_eq!(
            manager.get_version(RuntimeType::Rust).unwrap().required,
            Some("1.82.0".to_string())
        );
        assert_eq!(
            manager.get_version(RuntimeType::Go).unwrap().required,
            Some("1.22.0".to_string())
        );
        assert_eq!(
            manager.get_version(RuntimeType::Ruby).unwrap().required,
            Some("3.3.0".to_string())
        );
    }

    #[test]
    fn test_version_comparison() {
        assert_eq!(compare_versions("1.0.0", "1.0.0"), 0);
        assert_eq!(compare_versions("2.0.0", "1.0.0"), 1);
        assert_eq!(compare_versions("1.0.0", "2.0.0"), -1);
        assert_eq!(compare_versions("1.10.0", "1.9.0"), 1);
        assert_eq!(compare_versions("1.0.10", "1.0.9"), 1);
    }

    #[test]
    fn test_version_compatibility() {
        // Exact match
        assert!(check_version_compatibility("20.10.0", "20.10.0"));

        // >= constraint
        assert!(check_version_compatibility(">=18.0.0", "20.10.0"));
        assert!(!check_version_compatibility(">=21.0.0", "20.10.0"));

        // Caret range
        assert!(check_version_compatibility("^20.0.0", "20.10.0"));
        assert!(!check_version_compatibility("^21.0.0", "20.10.0"));

        // Major version only
        assert!(check_version_compatibility("20", "20.10.0"));
        assert!(!check_version_compatibility("21", "20.10.0"));
    }

    #[test]
    fn test_parse_node_version() {
        assert_eq!(
            parse_version_output(RuntimeType::Node, "v20.10.0"),
            Some("20.10.0".to_string())
        );
    }

    #[test]
    fn test_parse_python_version() {
        assert_eq!(
            parse_version_output(RuntimeType::Python, "Python 3.12.0"),
            Some("3.12.0".to_string())
        );
    }

    #[test]
    fn test_parse_rust_version() {
        assert_eq!(
            parse_version_output(RuntimeType::Rust, "rustc 1.82.0 (f6e511eec 2024-10-15)"),
            Some("1.82.0".to_string())
        );
    }

    #[test]
    fn test_parse_go_version() {
        assert_eq!(
            parse_version_output(RuntimeType::Go, "go version go1.22.0 darwin/arm64"),
            Some("1.22.0".to_string())
        );
    }

    #[test]
    fn test_parse_ruby_version() {
        assert_eq!(
            parse_version_output(
                RuntimeType::Ruby,
                "ruby 3.3.0 (2023-12-25 revision 5124f9ac75) [arm64-darwin23]"
            ),
            Some("3.3.0".to_string())
        );
    }

    #[test]
    fn test_extract_quoted_string() {
        assert_eq!(extract_quoted_string(r#"java version "21.0.1""#), Some("21.0.1".to_string()));
        assert_eq!(extract_quoted_string("ruby '3.3.0'"), Some("3.3.0".to_string()));
    }
}
