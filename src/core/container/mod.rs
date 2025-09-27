use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::core::{ContainerManifest, ContainerType, Version};
use crate::error::{ContainerError, ContainerResult};

/// Tracks container lifecycle for execution monitoring and user feedback.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerStatus {
    Ready,
    Running,
    Stopped,
    Error,
    Installing,
    Removing,
}

/// Tracks container runtime state for lifecycle management and user reporting.
/// Enables monitoring execution status, process information, and error history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerRuntime {
    pub id: Uuid,
    pub status: ContainerStatus,
    pub pid: Option<u32>,
    pub started_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
    pub errors: Vec<String>,
}

impl Default for ContainerRuntime {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            status: ContainerStatus::Ready,
            pid: None,
            started_at: None,
            stopped_at: None,
            exit_code: None,
            errors: Vec::new(),
        }
    }
}

/// Core abstraction for isolated application environments in the container file system.
/// Encapsulates application lifecycle, deployment validation, and runtime management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub manifest: ContainerManifest,
    pub path: PathBuf,
    pub runtime: ContainerRuntime,
    pub installed_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

impl Container {
    /// Initializes container with validated structure and runtime state.
    /// Core factory method for creating deployable container instances.
    pub fn new(manifest: ContainerManifest, path: PathBuf) -> ContainerResult<Self> {
        // Validate manifest before proceeding
        manifest.validate()?;

        // Validate the container directory structure
        Self::validate_structure(&path, &manifest)?;

        let now = Utc::now();
        
        Ok(Self {
            manifest,
            path,
            runtime: ContainerRuntime::default(),
            installed_at: now,
            last_accessed: now,
        })
    }

    /// Loads container from existing installation directory.
    /// Reconstructs container instance from manifest and validates structure.
    pub fn from_directory<P: AsRef<Path>>(path: P) -> ContainerResult<Self> {
        let path = path.as_ref().to_path_buf();
        
        // Ensure the path exists and is a directory
        if !path.exists() {
            return Err(ContainerError::InvalidPath { path });
        }

        if !path.is_dir() {
            return Err(ContainerError::InvalidStructure(
                "Container path must be a directory".to_string(),
            ));
        }

        // Load manifest from the directory
        let manifest_path = path.join("manifest.json");
        let manifest = ContainerManifest::from_file(&manifest_path)?;

        // Create container instance
        Self::new(manifest, path)
    }

    /// Validates container directory structure to ensure proper deployment.
    /// Prevents runtime failures by catching missing dependencies early.
    fn validate_structure(path: &Path, manifest: &ContainerManifest) -> ContainerResult<()> {
        // Ensure path exists and is a directory
        if !path.exists() {
            return Err(ContainerError::InvalidPath {
                path: path.to_path_buf(),
            });
        }

        if !path.is_dir() {
            return Err(ContainerError::InvalidStructure(
                "Container path must be a directory".to_string(),
            ));
        }

        // Validate required directory structure
        let required_dirs = ["scripts", "content", "config"];
        for dir in &required_dirs {
            let dir_path = path.join(dir);
            if !dir_path.exists() {
                return Err(ContainerError::InvalidStructure(
                    format!("Required directory '{}' not found", dir),
                ));
            }
        }

        // Ensure manifest file exists
        let manifest_path = path.join("manifest.json");
        if !manifest_path.exists() {
            return Err(ContainerError::InvalidStructure(
                "manifest.json not found".to_string(),
            ));
        }

        // Validate default script exists
        let default_script_path = path.join(manifest.default_script()?);
        if !default_script_path.exists() {
            return Err(ContainerError::MissingDefaultScript);
        }

        // Validate all referenced scripts exist
        for (script_name, script_path) in &manifest.scripts {
            let full_script_path = path.join(script_path);
            if !full_script_path.exists() {
                return Err(ContainerError::ScriptNotFound {
                    container: manifest.name.clone(),
                    script: script_name.clone(),
                });
            }
        }

        // Validate required configuration files exist
        let permissions_path = path.join("config/permissions.json");
        let environment_path = path.join("config/environment.json");
        
        if !permissions_path.exists() {
            return Err(ContainerError::InvalidStructure(
                "config/permissions.json not found".to_string(),
            ));
        }

        if !environment_path.exists() {
            return Err(ContainerError::InvalidStructure(
                "config/environment.json not found".to_string(),
            ));
        }

        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.manifest.name
    }

    pub fn version(&self) -> &Version {
        &self.manifest.version
    }

    pub fn container_type(&self) -> &ContainerType {
        &self.manifest.container_type
    }

    pub fn is_running(&self) -> bool {
        self.runtime.status == ContainerStatus::Running
    }

    /// Resolves script name to absolute filesystem path for execution.
    pub fn get_script_path(&self, script_name: &str) -> ContainerResult<PathBuf> {
        let script_relative_path = self.manifest.get_script(script_name)?;
        Ok(self.path.join(script_relative_path))
    }

    pub fn get_default_script_path(&self) -> ContainerResult<PathBuf> {
        self.get_script_path("default")
    }

    /// Updates access timestamp for usage tracking and cleanup decisions.
    pub fn update_last_accessed(&mut self) {
        self.last_accessed = Utc::now();
    }

    /// Updates runtime state when container execution begins.
    /// Enables process monitoring and lifecycle tracking.
    pub fn mark_running(&mut self, pid: u32) {
        self.runtime.status = ContainerStatus::Running;
        self.runtime.pid = Some(pid);
        self.runtime.started_at = Some(Utc::now());
        self.update_last_accessed();
    }

    /// Updates runtime state when container execution ends.
    /// Records exit status for debugging and user feedback.
    pub fn mark_stopped(&mut self, exit_code: i32) {
        self.runtime.status = ContainerStatus::Stopped;
        self.runtime.pid = None;
        self.runtime.stopped_at = Some(Utc::now());
        self.runtime.exit_code = Some(exit_code);
    }

    /// Records container failure for debugging and user notification.
    /// Maintains error history for troubleshooting repeated issues.
    pub fn mark_error(&mut self, error: String) {
        self.runtime.status = ContainerStatus::Error;
        self.runtime.errors.push(error);
        self.runtime.stopped_at = Some(Utc::now());
    }

    pub fn content_path(&self) -> PathBuf {
        self.path.join("content")
    }

    pub fn config_path(&self) -> PathBuf {
        self.path.join("config")
    }

    pub fn scripts_path(&self) -> PathBuf {
        self.path.join("scripts")
    }

    /// Ensures all required packages are available before container execution.
    /// Prevents runtime failures from missing or incompatible dependencies.
    pub fn validate_dependencies(&self, available_packages: &HashMap<String, Version>) -> ContainerResult<()> {
        for dependency in &self.manifest.dependencies {
            // Ensure the required package is available
            let package_version = available_packages
                .get(&dependency.name)
                .ok_or_else(|| ContainerError::PackageNotFound {
                    package: dependency.name.clone(),
                })?;

            // Parse the required version specification
            let required_version: Version = dependency.version.parse()?;

            // Verify version compatibility
            if !package_version.is_compatible_with(&required_version) {
                return Err(ContainerError::VersionConflict {
                    conflict: format!(
                        "Package '{}' version {} is not compatible with required version {}",
                        dependency.name, package_version, required_version
                    ),
                });
            }
        }

        Ok(())
    }

    /// Detects circular dependencies to prevent infinite dependency loops.
    /// Critical for safe container installation and dependency resolution.
    pub fn check_circular_dependencies(
        containers: &HashMap<String, Container>,
        visited: &mut Vec<String>,
        current: &str,
    ) -> ContainerResult<()> {
        if visited.contains(&current.to_string()) {
            return Err(ContainerError::CircularDependency {
                chain: visited.join(" -> "),
            });
        }

        if let Some(container) = containers.get(current) {
            visited.push(current.to_string());

            for dependency in &container.manifest.dependencies {
                Self::check_circular_dependencies(containers, visited, &dependency.name)?;
            }

            visited.pop();
        }

        Ok(())
    }

    pub fn to_json(&self) -> ContainerResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| ContainerError::JsonError { source: e })
    }

    pub fn from_json(json: &str) -> ContainerResult<Self> {
        serde_json::from_str(json).map_err(|e| ContainerError::JsonError { source: e })
    }
}

#[cfg(test)]
mod tests;

