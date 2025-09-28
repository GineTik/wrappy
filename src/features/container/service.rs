use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::features::{ContainerManifest, Version};
use crate::shared::error::{ContainerError, ContainerResult};

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

/// Container service handles business logic for container operations
pub struct ContainerService;

impl ContainerService {
    /// Initializes container with validated structure and runtime state.
    /// Core factory method for creating deployable container instances.
    pub fn create_container(manifest: ContainerManifest, path: PathBuf) -> ContainerResult<Container> {
        Self::validate_manifest(&manifest)?;
        Self::validate_structure(&path, &manifest)?;

        let now = Utc::now();
        
        Ok(Container {
            manifest,
            path,
            runtime: ContainerRuntime::default(),
            installed_at: now,
            last_accessed: now,
        })
    }

    /// Loads container from existing installation directory.
    /// Reconstructs container instance from manifest and validates structure.
    pub fn load_from_directory<P: AsRef<Path>>(path: P) -> ContainerResult<Container> {
        let path = path.as_ref().to_path_buf();
        
        Self::validate_path_exists(&path)?;
        
        let manifest = Self::load_manifest(&path)?;
        Self::create_container(manifest, path)
    }

    /// Validates that path exists and is a directory
    fn validate_path_exists(path: &PathBuf) -> ContainerResult<()> {
        if !path.exists() {
            return Err(ContainerError::InvalidPath { 
                path: path.clone(), 
                reason: "Path does not exist".to_string() 
            });
        }

        if !path.is_dir() {
            return Err(ContainerError::InvalidStructure(
                "Container path must be a directory".to_string(),
            ));
        }

        Ok(())
    }

    /// Loads and validates manifest from directory
    fn load_manifest(path: &PathBuf) -> ContainerResult<ContainerManifest> {
        let manifest_path = path.join("manifest.json");
        ContainerManifest::from_file(&manifest_path)
    }

    /// Validates manifest data
    fn validate_manifest(manifest: &ContainerManifest) -> ContainerResult<()> {
        manifest.validate()
    }

    /// Validates container directory structure to ensure proper deployment.
    /// Prevents runtime failures by catching missing dependencies early.
    pub fn validate_structure(path: &Path, manifest: &ContainerManifest) -> ContainerResult<()> {
        Self::validate_path_exists(&path.to_path_buf())?;
        Self::validate_required_directories(path)?;
        Self::validate_manifest_file_exists(path)?;
        Self::validate_scripts_exist(path, manifest)?;
        Self::validate_config_files_exist(path)?;

        Ok(())
    }

    /// Validates required directory structure exists
    fn validate_required_directories(path: &Path) -> ContainerResult<()> {
        let required_dirs = ["scripts", "content", "config"];
        for dir in &required_dirs {
            let dir_path = path.join(dir);
            if !dir_path.exists() {
                return Err(ContainerError::InvalidStructure(
                    format!("Required directory '{}' not found", dir),
                ));
            }
        }
        Ok(())
    }

    /// Validates manifest file exists
    fn validate_manifest_file_exists(path: &Path) -> ContainerResult<()> {
        let manifest_path = path.join("manifest.json");
        if !manifest_path.exists() {
            return Err(ContainerError::InvalidStructure(
                "manifest.json not found".to_string(),
            ));
        }
        Ok(())
    }

    /// Validates all script files exist
    fn validate_scripts_exist(path: &Path, manifest: &ContainerManifest) -> ContainerResult<()> {
        Self::validate_default_script_exists(path, manifest)?;
        Self::validate_all_scripts_exist(path, manifest)?;
        Ok(())
    }

    /// Validates default script exists
    fn validate_default_script_exists(path: &Path, manifest: &ContainerManifest) -> ContainerResult<()> {
        let default_script_path = path.join(manifest.default_script()?);
        if !default_script_path.exists() {
            return Err(ContainerError::MissingDefaultScript);
        }
        Ok(())
    }

    /// Validates all referenced scripts exist
    fn validate_all_scripts_exist(path: &Path, manifest: &ContainerManifest) -> ContainerResult<()> {
        for (script_name, script_path) in &manifest.scripts {
            let full_script_path = path.join(script_path);
            if !full_script_path.exists() {
                return Err(ContainerError::ScriptNotFound {
                    container: manifest.name.clone(),
                    script: script_name.clone(),
                });
            }
        }
        Ok(())
    }

    /// Validates required configuration files exist
    fn validate_config_files_exist(path: &Path) -> ContainerResult<()> {
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

    /// Ensures all required packages are available before container execution.
    /// Prevents runtime failures from missing or incompatible dependencies.
    pub fn validate_dependencies(
        container: &Container,
        available_packages: &HashMap<String, Version>
    ) -> ContainerResult<()> {
        for dependency in &container.manifest.dependencies {
            Self::validate_single_dependency(dependency, available_packages)?;
        }
        Ok(())
    }

    /// Validates single dependency availability and compatibility
    fn validate_single_dependency(
        dependency: &crate::features::manifest::Dependency,
        available_packages: &HashMap<String, Version>
    ) -> ContainerResult<()> {
        let package_version = available_packages
            .get(&dependency.name)
            .ok_or_else(|| ContainerError::PackageNotFound {
                package: dependency.name.clone(),
            })?;

        let required_version: Version = dependency.version.parse()?;

        if !package_version.is_compatible_with(&required_version) {
            return Err(ContainerError::VersionConflict {
                conflict: format!(
                    "Package '{}' version {} is not compatible with required version {}",
                    dependency.name, package_version, required_version
                ),
            });
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
}

impl Container {
    /// Factory method using service
    pub fn new(manifest: ContainerManifest, path: PathBuf) -> ContainerResult<Self> {
        ContainerService::create_container(manifest, path)
    }

    /// Factory method using service
    pub fn from_directory<P: AsRef<Path>>(path: P) -> ContainerResult<Self> {
        ContainerService::load_from_directory(path)
    }

    pub fn name(&self) -> &str {
        &self.manifest.name
    }

    pub fn version(&self) -> &Version {
        &self.manifest.version
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

    /// Validates dependencies using service
    pub fn validate_dependencies(&self, available_packages: &HashMap<String, Version>) -> ContainerResult<()> {
        ContainerService::validate_dependencies(self, available_packages)
    }

    /// Checks circular dependencies using service
    pub fn check_circular_dependencies(
        containers: &HashMap<String, Container>,
        visited: &mut Vec<String>,
        current: &str,
    ) -> ContainerResult<()> {
        ContainerService::check_circular_dependencies(containers, visited, current)
    }

    pub fn to_json(&self) -> ContainerResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| ContainerError::JsonError { source: e })
    }

    pub fn from_json(json: &str) -> ContainerResult<Self> {
        serde_json::from_str(json).map_err(|e| ContainerError::JsonError { source: e })
    }
}
