# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Security Features

Palrun includes built-in security features to protect users:

### Command Validation
- **Dangerous Pattern Detection**: Detects and warns about potentially harmful commands
- **Shell Injection Prevention**: Identifies shell injection attempts
- **Path Traversal Protection**: Blocks excessive directory traversal
- **Privilege Escalation Detection**: Warns about sudo and setuid operations

### Input Sanitization
- **Environment Variable Protection**: Redacts sensitive variables (API keys, passwords)
- **Null Byte Detection**: Rejects commands containing null bytes
- **Length Validation**: Limits command length to prevent buffer issues

### Code Security
- **Zero Unsafe Code**: `#![forbid(unsafe_code)]` enforced throughout
- **Clippy Lints**: Strict linting with `-D warnings`
- **Dependency Auditing**: Regular `cargo audit` checks

## Reporting a Vulnerability

If you discover a security vulnerability in Palrun, please report it responsibly:

1. **DO NOT** open a public GitHub issue
2. Email security concerns to the maintainers
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 1 week
- **Fix Timeline**: Depends on severity
  - Critical: Within 24-48 hours
  - High: Within 1 week
  - Medium: Within 2 weeks
  - Low: Next release cycle

### Disclosure Policy

- We follow coordinated disclosure
- Vulnerabilities will be publicly disclosed after a fix is available
- Credit will be given to reporters (unless anonymity is requested)

## Security Best Practices for Users

### API Key Storage
- Use environment variables for API keys
- Never commit keys to version control
- Consider using OS keychain for sensitive credentials

### Running Commands
- Review commands before execution (enabled by default)
- Use `--dry-run` for unfamiliar commands
- Be cautious with piped commands from external sources

### Plugin Security
- Only install plugins from trusted sources
- Review plugin permissions before enabling
- Plugins run in a sandboxed WASM environment

## Security Audits

- Static analysis via `cargo clippy` on every build
- Dependency scanning via `cargo audit` in CI/CD
- Integration tests for security features (24+ tests)

## Known Limitations

1. **Local Execution**: Commands run with user's permissions
2. **AI Responses**: AI-generated commands should be reviewed before execution
3. **Network Access**: Some features require internet (AI, webhooks)

## Contact

For security concerns, please use responsible disclosure as described above.
