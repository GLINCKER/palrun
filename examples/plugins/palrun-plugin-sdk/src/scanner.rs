//! Scanner plugin trait and utilities.

use crate::command::Command;
use crate::context::ScanContext;

/// Trait for scanner plugins.
///
/// Scanner plugins discover commands from project files.
/// Implement this trait to create a custom scanner.
///
/// # Example
///
/// ```rust
/// use palrun_plugin_sdk::prelude::*;
///
/// struct MakefileScanner;
///
/// impl Scanner for MakefileScanner {
///     fn name(&self) -> &'static str {
///         "makefile-scanner"
///     }
///
///     fn file_patterns(&self) -> &'static [&'static str] {
///         &["Makefile", "makefile", "GNUmakefile"]
///     }
///
///     fn scan(&self, context: &ScanContext) -> Vec<Command> {
///         let mut commands = Vec::new();
///
///         if let Some(content) = context.get_file("Makefile") {
///             // Parse Makefile targets
///             for line in content.lines() {
///                 if let Some(target) = line.strip_suffix(':') {
///                     if !target.starts_with('.') && !target.contains(' ') {
///                         commands.push(
///                             Command::new(format!("make {}", target), format!("make {}", target))
///                                 .with_tag("make")
///                         );
///                     }
///                 }
///             }
///         }
///
///         commands
///     }
/// }
/// ```
pub trait Scanner {
    /// Get the scanner name.
    ///
    /// This should be a unique identifier for the scanner,
    /// typically matching the plugin name.
    fn name(&self) -> &'static str;

    /// Get file patterns this scanner handles.
    ///
    /// These patterns are used to determine which files
    /// the scanner is interested in. Supports glob patterns.
    ///
    /// # Examples
    ///
    /// - `"Makefile"` - exact filename match
    /// - `"*.gradle"` - files ending in .gradle
    /// - `"build.gradle*"` - build.gradle and build.gradle.kts
    /// - `"package.json"` - exact filename
    fn file_patterns(&self) -> &'static [&'static str];

    /// Scan the project for commands.
    ///
    /// This method receives a context with the project path
    /// and contents of matched files. It should return a list
    /// of discovered commands.
    ///
    /// # Arguments
    ///
    /// * `context` - Scan context with project info and file contents
    ///
    /// # Returns
    ///
    /// List of discovered commands.
    fn scan(&self, context: &ScanContext) -> Vec<Command>;

    /// Get the scanner description.
    ///
    /// Optional description explaining what this scanner does.
    fn description(&self) -> Option<&'static str> {
        None
    }

    /// Get the scanner priority.
    ///
    /// Higher priority scanners run first. Default is 0.
    /// Use negative values for low priority scanners.
    fn priority(&self) -> i32 {
        0
    }
}

/// Macro to export a scanner as a WASM plugin.
///
/// This macro generates the necessary FFI exports for the scanner
/// to be loaded by Palrun.
///
/// # Example
///
/// ```rust,ignore
/// use palrun_plugin_sdk::prelude::*;
///
/// struct MyScanner;
///
/// impl Scanner for MyScanner {
///     fn name(&self) -> &'static str { "my-scanner" }
///     fn file_patterns(&self) -> &'static [&'static str] { &["*.my"] }
///     fn scan(&self, ctx: &ScanContext) -> Vec<Command> { vec![] }
/// }
///
/// export_scanner!(MyScanner);
/// ```
#[macro_export]
macro_rules! export_scanner {
    ($scanner_type:ty) => {
        // Static scanner instance
        static SCANNER: std::sync::OnceLock<$scanner_type> = std::sync::OnceLock::new();

        fn get_scanner() -> &'static $scanner_type {
            SCANNER.get_or_init(|| <$scanner_type>::default())
        }

        /// Returns the scanner name as a JSON string.
        #[no_mangle]
        pub extern "C" fn name() -> *mut u8 {
            use $crate::Scanner;
            let name = get_scanner().name();
            let json = serde_json::to_string(&name).unwrap_or_else(|_| "\"unknown\"".to_string());
            $crate::ffi::string_to_ptr(json)
        }

        /// Returns file patterns as a JSON array.
        #[no_mangle]
        pub extern "C" fn file_patterns() -> *mut u8 {
            use $crate::Scanner;
            let patterns = get_scanner().file_patterns();
            let json = serde_json::to_string(&patterns).unwrap_or_else(|_| "[]".to_string());
            $crate::ffi::string_to_ptr(json)
        }

        /// Scans the project and returns commands as JSON.
        #[no_mangle]
        pub extern "C" fn scan(context_ptr: *const u8, context_len: usize) -> *mut u8 {
            use $crate::Scanner;

            // Parse the context from JSON
            let context = unsafe {
                let slice = std::slice::from_raw_parts(context_ptr, context_len);
                let json_str = std::str::from_utf8(slice).unwrap_or("{}");
                serde_json::from_str::<$crate::ScanContext>(json_str)
                    .unwrap_or_default()
            };

            // Run the scanner
            let commands = get_scanner().scan(&context);

            // Return commands as JSON
            let json = serde_json::to_string(&commands).unwrap_or_else(|_| "[]".to_string());
            $crate::ffi::string_to_ptr(json)
        }

        /// Allocate memory for the host.
        #[no_mangle]
        pub extern "C" fn alloc(size: usize) -> *mut u8 {
            $crate::ffi::alloc(size)
        }

        /// Free memory allocated by the plugin.
        #[no_mangle]
        pub extern "C" fn dealloc(ptr: *mut u8, size: usize) {
            $crate::ffi::dealloc(ptr, size)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestScanner;

    impl Scanner for TestScanner {
        fn name(&self) -> &'static str {
            "test-scanner"
        }

        fn file_patterns(&self) -> &'static [&'static str] {
            &["*.test", "Testfile"]
        }

        fn scan(&self, context: &ScanContext) -> Vec<Command> {
            let mut commands = Vec::new();

            if context.has_file("Testfile") {
                commands.push(Command::new("test", "run-tests").with_tag("test"));
            }

            commands
        }

        fn description(&self) -> Option<&'static str> {
            Some("Test scanner for unit tests")
        }

        fn priority(&self) -> i32 {
            10
        }
    }

    #[test]
    fn test_scanner_trait() {
        let scanner = TestScanner;

        assert_eq!(scanner.name(), "test-scanner");
        assert_eq!(scanner.file_patterns(), &["*.test", "Testfile"]);
        assert_eq!(scanner.description(), Some("Test scanner for unit tests"));
        assert_eq!(scanner.priority(), 10);
    }

    #[test]
    fn test_scanner_scan() {
        let scanner = TestScanner;
        let context = ScanContext::new("/project", "test").with_file("Testfile", "content");

        let commands = scanner.scan(&context);
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].name, "test");
    }

    #[test]
    fn test_scanner_no_match() {
        let scanner = TestScanner;
        let context = ScanContext::new("/project", "test");

        let commands = scanner.scan(&context);
        assert!(commands.is_empty());
    }
}
