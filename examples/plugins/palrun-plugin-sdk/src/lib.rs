//! Palrun Plugin SDK
//!
//! This SDK provides the types and utilities needed to build Palrun plugins
//! that compile to WebAssembly (WASM).
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use palrun_plugin_sdk::prelude::*;
//!
//! struct MyScanner;
//!
//! impl Scanner for MyScanner {
//!     fn name(&self) -> &'static str {
//!         "my-scanner"
//!     }
//!
//!     fn file_patterns(&self) -> &'static [&'static str] {
//!         &["Myfile", "*.myext"]
//!     }
//!
//!     fn scan(&self, context: &ScanContext) -> Vec<Command> {
//!         vec![
//!             Command::new("my-command", "echo hello")
//!                 .with_description("Say hello")
//!                 .with_tag("example"),
//!         ]
//!     }
//! }
//!
//! // Export the scanner
//! export_scanner!(MyScanner);
//! ```
//!
//! # Plugin Types
//!
//! - **Scanner** - Discovers commands from project files
//! - **AI Provider** - Custom LLM integration (coming soon)
//! - **Integration** - External service connection (coming soon)
//! - **UI** - Custom TUI components (coming soon)

#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

mod command;
mod context;
mod error;
mod scanner;

/// FFI utilities for WASM plugins.
///
/// This module is public for macro use but should not be used directly.
#[doc(hidden)]
pub mod ffi;

pub mod prelude;

pub use command::{Command, CommandBuilder};
pub use context::ScanContext;
pub use error::{PluginError, PluginResult};
pub use scanner::Scanner;

/// Plugin API version supported by this SDK.
pub const API_VERSION: &str = "0.1.0";

/// Re-export serde for plugins that need custom serialization.
pub mod serde {
    pub use ::serde::*;
}

/// Re-export serde_json for plugins that need JSON handling.
pub mod json {
    pub use ::serde_json::*;
}
