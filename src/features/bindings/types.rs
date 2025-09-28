use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Defines how container resources are bound to the host system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BindingType {
    /// Direct symbolic link to container resource
    Symlink,
    /// Wrapper script that intercepts execution
    Wrapper,
    /// Copy resource to host location
    Copy,
}

impl Default for BindingType {
    fn default() -> Self {
        Self::Wrapper
    }
}

/// Configuration for binding executable files from container to host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutableBinding {
    /// Path to executable within container (relative to container root)
    pub source: String,
    /// Target path on host system (supports ~ expansion)
    pub target: String,
    /// How the binding should be created
    #[serde(default)]
    pub binding_type: BindingType,
    /// Optional display name for console output
    pub display_name: Option<String>,
}

/// Configuration for binding configuration directories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigBinding {
    /// Path to config directory within container
    pub source: String,
    /// Target config path on host system
    pub target: String,
    /// How the binding should be created
    #[serde(default)]
    pub binding_type: BindingType,
    /// Whether to backup existing target before binding
    #[serde(default)]
    pub backup_existing: bool,
}

/// Configuration for binding data directories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataBinding {
    /// Path to data directory within container
    pub source: String,
    /// Target data path on host system  
    pub target: String,
    /// How the binding should be created
    #[serde(default)]
    pub binding_type: BindingType,
    /// Whether to backup existing target before binding
    #[serde(default)]
    pub backup_existing: bool,
}

/// Complete bindings configuration for a container.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BindingsConfig {
    /// Executable file bindings
    #[serde(default)]
    pub executables: Vec<ExecutableBinding>,
    /// Configuration directory bindings
    #[serde(default)]
    pub configs: Vec<ConfigBinding>,
    /// Data directory bindings
    #[serde(default)]
    pub data: Vec<DataBinding>,
}

impl BindingsConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_executable(&mut self, binding: ExecutableBinding) {
        self.executables.push(binding);
    }

    pub fn add_config(&mut self, binding: ConfigBinding) {
        self.configs.push(binding);
    }

    pub fn add_data(&mut self, binding: DataBinding) {
        self.data.push(binding);
    }

    pub fn is_empty(&self) -> bool {
        self.executables.is_empty() && self.configs.is_empty() && self.data.is_empty()
    }
}

/// Represents an active binding on the host system.
#[derive(Debug, Clone)]
pub struct ActiveBinding {
    pub container_name: String,
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub binding_type: BindingType,
    pub created_at: std::time::SystemTime,
}