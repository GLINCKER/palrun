//! Prelude module - import commonly used types.
//!
//! ```rust,ignore
//! use palrun_plugin_sdk::prelude::*;
//! ```

pub use crate::command::{Command, CommandBuilder};
pub use crate::context::ScanContext;
pub use crate::error::{PluginError, PluginResult};
pub use crate::scanner::Scanner;
pub use crate::API_VERSION;

// Re-export the macro for exporting scanners
pub use crate::export_scanner;
