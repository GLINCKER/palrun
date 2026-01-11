//! File permission checking module.
//!
//! Provides verification of file permissions to ensure:
//! - Configuration files are not world-writable
//! - Secrets files have restricted access
//! - Executable files have appropriate permissions

use std::path::Path;

/// Error type for permission checks.
#[derive(Debug, Clone)]
pub enum PermissionError {
    /// File not found
    NotFound(String),
    /// Unable to read metadata
    MetadataError(String),
    /// Permission check failed
    PermissionDenied(String),
    /// Invalid file type
    InvalidFileType(String),
}

impl std::fmt::Display for PermissionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(p) => write!(f, "File not found: {}", p),
            Self::MetadataError(e) => write!(f, "Metadata error: {}", e),
            Self::PermissionDenied(e) => write!(f, "Permission denied: {}", e),
            Self::InvalidFileType(e) => write!(f, "Invalid file type: {}", e),
        }
    }
}

impl std::error::Error for PermissionError {}

/// Permission level requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionLevel {
    /// No restrictions
    None,
    /// Owner-only read/write
    OwnerOnly,
    /// Owner read/write, group read
    GroupReadable,
    /// Owner read/write, world read
    WorldReadable,
    /// Executable (owner only)
    ExecutableOwnerOnly,
    /// Executable (group)
    ExecutableGroup,
    /// Custom mode
    Custom(u32),
}

impl PermissionLevel {
    /// Get the required mode for this level (Unix).
    #[cfg(unix)]
    pub fn required_mode(&self) -> Option<u32> {
        match self {
            Self::None => None,
            Self::OwnerOnly => Some(0o600),
            Self::GroupReadable => Some(0o640),
            Self::WorldReadable => Some(0o644),
            Self::ExecutableOwnerOnly => Some(0o700),
            Self::ExecutableGroup => Some(0o750),
            Self::Custom(mode) => Some(*mode),
        }
    }

    #[cfg(not(unix))]
    pub fn required_mode(&self) -> Option<u32> {
        None
    }
}

/// File permissions information.
#[derive(Debug, Clone)]
pub struct FilePermissions {
    /// The file path
    pub path: String,
    /// Unix mode (if available)
    pub mode: Option<u32>,
    /// Whether the file is readable
    pub readable: bool,
    /// Whether the file is writable
    pub writable: bool,
    /// Whether the file is executable
    pub executable: bool,
    /// Owner user ID (Unix)
    pub owner_uid: Option<u32>,
    /// Owner group ID (Unix)
    pub owner_gid: Option<u32>,
}

impl FilePermissions {
    /// Check if the file is world-readable.
    #[cfg(unix)]
    pub fn is_world_readable(&self) -> bool {
        self.mode.map_or(false, |m| m & 0o004 != 0)
    }

    #[cfg(not(unix))]
    pub fn is_world_readable(&self) -> bool {
        false
    }

    /// Check if the file is world-writable.
    #[cfg(unix)]
    pub fn is_world_writable(&self) -> bool {
        self.mode.map_or(false, |m| m & 0o002 != 0)
    }

    #[cfg(not(unix))]
    pub fn is_world_writable(&self) -> bool {
        false
    }

    /// Check if the file is group-writable.
    #[cfg(unix)]
    pub fn is_group_writable(&self) -> bool {
        self.mode.map_or(false, |m| m & 0o020 != 0)
    }

    #[cfg(not(unix))]
    pub fn is_group_writable(&self) -> bool {
        false
    }
}

/// Result of a permission check.
#[derive(Debug, Clone)]
pub struct PermissionCheck {
    /// Whether the check passed
    pub passed: bool,
    /// The file permissions
    pub permissions: Option<FilePermissions>,
    /// Issues found
    pub issues: Vec<String>,
    /// Whether the check was skipped
    pub skipped: bool,
}

impl PermissionCheck {
    /// Create a skipped check result.
    pub fn skipped() -> Self {
        Self { passed: true, permissions: None, issues: Vec::new(), skipped: true }
    }

    /// Create a passed check result.
    pub fn passed(permissions: FilePermissions) -> Self {
        Self { passed: true, permissions: Some(permissions), issues: Vec::new(), skipped: false }
    }

    /// Create a failed check result.
    pub fn failed(permissions: FilePermissions, issues: Vec<String>) -> Self {
        Self { passed: false, permissions: Some(permissions), issues, skipped: false }
    }
}

/// Secure file permission checker.
#[derive(Debug, Default)]
pub struct SecureFileChecker {
    /// Required permission level for different file types
    strict_mode: bool,
}

impl SecureFileChecker {
    /// Create a new checker.
    pub fn new() -> Self {
        Self { strict_mode: false }
    }

    /// Create a checker in strict mode.
    pub fn strict() -> Self {
        Self { strict_mode: true }
    }

    /// Check file permissions.
    pub fn check(&self, path: &Path) -> Result<PermissionCheck, PermissionError> {
        if !path.exists() {
            return Err(PermissionError::NotFound(path.display().to_string()));
        }

        let metadata =
            std::fs::metadata(path).map_err(|e| PermissionError::MetadataError(e.to_string()))?;

        let permissions = self.get_permissions(path, &metadata);
        let issues = self.check_issues(&permissions);

        if issues.is_empty() {
            Ok(PermissionCheck::passed(permissions))
        } else {
            Ok(PermissionCheck::failed(permissions, issues))
        }
    }

    /// Get permissions from metadata.
    #[cfg(unix)]
    fn get_permissions(&self, path: &Path, metadata: &std::fs::Metadata) -> FilePermissions {
        use std::os::unix::fs::MetadataExt;
        use std::os::unix::fs::PermissionsExt;

        FilePermissions {
            path: path.display().to_string(),
            mode: Some(metadata.permissions().mode()),
            readable: metadata.permissions().mode() & 0o444 != 0,
            writable: metadata.permissions().mode() & 0o222 != 0,
            executable: metadata.permissions().mode() & 0o111 != 0,
            owner_uid: Some(metadata.uid()),
            owner_gid: Some(metadata.gid()),
        }
    }

    #[cfg(not(unix))]
    fn get_permissions(&self, path: &Path, metadata: &std::fs::Metadata) -> FilePermissions {
        FilePermissions {
            path: path.display().to_string(),
            mode: None,
            readable: !metadata.permissions().readonly(),
            writable: !metadata.permissions().readonly(),
            executable: false,
            owner_uid: None,
            owner_gid: None,
        }
    }

    /// Check for permission issues.
    fn check_issues(&self, permissions: &FilePermissions) -> Vec<String> {
        let mut issues = Vec::new();

        // Check for world-writable files
        if permissions.is_world_writable() {
            issues.push(format!("File '{}' is world-writable (insecure)", permissions.path));
        }

        // In strict mode, also check group-writable
        if self.strict_mode && permissions.is_group_writable() {
            issues.push(format!("File '{}' is group-writable (strict mode)", permissions.path));
        }

        issues
    }

    /// Check if a file has the required permission level.
    #[cfg(unix)]
    pub fn check_level(
        &self,
        path: &Path,
        required: PermissionLevel,
    ) -> Result<PermissionCheck, PermissionError> {
        let check = self.check(path)?;

        if let (Some(ref perms), Some(required_mode)) =
            (&check.permissions, required.required_mode())
        {
            if let Some(mode) = perms.mode {
                let actual_mode = mode & 0o777;
                if actual_mode != required_mode && actual_mode > required_mode {
                    let mut issues = check.issues.clone();
                    issues.push(format!(
                        "File '{}' has mode {:o}, expected {:o}",
                        perms.path, actual_mode, required_mode
                    ));
                    return Ok(PermissionCheck::failed(check.permissions.unwrap(), issues));
                }
            }
        }

        Ok(check)
    }

    #[cfg(not(unix))]
    pub fn check_level(
        &self,
        path: &Path,
        _required: PermissionLevel,
    ) -> Result<PermissionCheck, PermissionError> {
        self.check(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_permission_check_skipped() {
        let check = PermissionCheck::skipped();
        assert!(check.passed);
        assert!(check.skipped);
    }

    #[test]
    fn test_file_not_found() {
        let checker = SecureFileChecker::new();
        let result = checker.check(Path::new("/nonexistent/file"));
        assert!(matches!(result, Err(PermissionError::NotFound(_))));
    }

    #[test]
    fn test_check_existing_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();

        let checker = SecureFileChecker::new();
        let result = checker.check(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg(unix)]
    fn test_permission_level_modes() {
        assert_eq!(PermissionLevel::OwnerOnly.required_mode(), Some(0o600));
        assert_eq!(PermissionLevel::GroupReadable.required_mode(), Some(0o640));
        assert_eq!(PermissionLevel::WorldReadable.required_mode(), Some(0o644));
        assert_eq!(PermissionLevel::ExecutableOwnerOnly.required_mode(), Some(0o700));
        assert_eq!(PermissionLevel::None.required_mode(), None);
    }

    #[test]
    fn test_permission_error_display() {
        let errors = [
            PermissionError::NotFound("/test".to_string()),
            PermissionError::MetadataError("error".to_string()),
            PermissionError::PermissionDenied("denied".to_string()),
            PermissionError::InvalidFileType("invalid".to_string()),
        ];

        for error in errors {
            let display = format!("{}", error);
            assert!(!display.is_empty());
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_world_writable_detection() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("world_writable.txt");
        File::create(&file_path).unwrap();

        // Make file world-writable
        std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o666)).unwrap();

        let checker = SecureFileChecker::new();
        let result = checker.check(&file_path).unwrap();

        assert!(!result.passed);
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_strict_mode() {
        let checker = SecureFileChecker::strict();
        assert!(checker.strict_mode);
    }
}
