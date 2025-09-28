use std::path::PathBuf;
use thiserror::Error;

/// Comprehensive error handling for container lifecycle operations.
/// Provides detailed context for debugging and user feedback.
#[derive(Error, Debug)]
pub enum ContainerError {
    #[error("Invalid container structure: {0}")]
    InvalidStructure(String),

    #[error("Default startup script not found")]
    MissingDefaultScript,

    #[error("Script '{script}' not found in container '{container}'")]
    ScriptNotFound { container: String, script: String },

    #[error("Invalid manifest format: {0}")]
    InvalidManifest(String),

    #[error("Manifest validation failed: {0}")]
    ManifestValidation(String),

    #[error("Invalid package dependency '{package}': {reason}")]
    InvalidDependency { package: String, reason: String },

    #[error("Package '{package}' not found")]
    PackageNotFound { package: String },

    #[error("Circular dependency detected: {chain}")]
    CircularDependency { chain: String },

    #[error("Invalid container version format: {version}")]
    InvalidVersion { version: String },

    #[error("Version conflict: {conflict}")]
    VersionConflict { conflict: String },

    #[error("Permission denied: {operation}")]
    PermissionDenied { operation: String },

    #[error("Container '{name}' already exists")]
    ContainerExists { name: String },

    #[error("Container '{name}' not found")]
    ContainerNotFound { name: String },

    #[error("IO error at path '{path}': {source}")]
    IoError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("JSON parsing error: {source}")]
    JsonError {
        #[source]
        source: serde_json::Error,
    },

    #[error("Invalid file path: {path} - {reason}")]
    InvalidPath { path: PathBuf, reason: String },

    #[error("Unsupported container type: {container_type}")]
    UnsupportedType { container_type: String },

    #[error("Runtime error: {message}")]
    Runtime { message: String },
}

pub type ContainerResult<T> = Result<T, ContainerError>;

impl From<std::io::Error> for ContainerError {
    fn from(error: std::io::Error) -> Self {
        ContainerError::IoError {
            path: PathBuf::new(),
            source: error,
        }
    }
}

impl From<serde_json::Error> for ContainerError {
    fn from(error: serde_json::Error) -> Self {
        ContainerError::JsonError { source: error }
    }
}
