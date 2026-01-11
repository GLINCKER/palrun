//! Plugin system for Palrun.
//!
//! This module provides a WASM-based plugin system that allows extending Palrun
//! with custom scanners, AI providers, and integrations.
//!
//! # Architecture
//!
//! The plugin system uses WebAssembly (via wasmtime) for secure, sandboxed
//! plugin execution. Plugins are cross-platform and can be written in any
//! language that compiles to WASM.
//!
//! # Plugin Types
//!
//! - **Scanner plugins**: Detect commands from project files
//! - **AI provider plugins**: Custom LLM integrations
//! - **Integration plugins**: External service connections
//!
//! # Example Configuration
//!
//! ```toml
//! [[plugins]]
//! name = "gradle-scanner"
//! enabled = true
//!
//! [plugins.gradle-scanner.config]
//! scan_depth = 3
//! ```

mod error;
mod host;
mod manager;
mod manifest;
mod registry;
mod runtime;
mod types;

pub use error::{PluginError, PluginResult};
pub use host::{HostCapabilities, PluginHost};
pub use manager::{InstalledPlugin, PluginManager, PluginState};
pub use manifest::{FilesystemPermissions, PluginManifest, PluginPermissions};
pub use registry::{
    RegistryClient, RegistryPlugin, RemoteRegistry, SearchResult, DEFAULT_REGISTRY_URL,
};
pub use runtime::PluginRuntime;
pub use types::{
    PluginCommand, PluginInfo, PluginType, MANIFEST_FILE, PLUGIN_API_VERSION, PLUGIN_EXTENSION,
};
