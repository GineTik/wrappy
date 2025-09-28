use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::features::Version;
use crate::features::bindings::BindingsConfig;
use crate::shared::error::{ContainerError, ContainerResult};

/// Defines container category for isolation and deployment strategies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerType {
    Application,
    Package,
    System,
}

/// Controls container security boundaries and resource access.
/// Balances security isolation with functional requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolationConfig {
    pub enabled: bool,
    pub network: String,
    pub filesystem: String,
}

impl Default for IsolationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            network: "restricted".to_string(),
            filesystem: "sandboxed".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub optional: bool,
}

/// Core container configuration defining deployment behavior and requirements.
/// Central metadata store for container lifecycle management and validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerManifest {
    pub name: String,
    pub version: Version,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub scripts: HashMap<String, String>,
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
    #[serde(default)]
    pub environment: HashMap<String, String>,
    #[serde(default)]
    pub bindings: BindingsConfig,
}

impl ContainerManifest {
    /// Initializes manifest with default configuration and required default script.
    pub fn new(name: String, version: Version) -> Self {
        let mut scripts = HashMap::new();
        scripts.insert("default".to_string(), "scripts/default.sh".to_string());

        Self {
            name,
            version,
            description: String::new(),
            author: String::new(),
            scripts,
            dependencies: Vec::new(),
            environment: HashMap::new(),
            bindings: BindingsConfig::new(),
        }
    }

    /// Deserializes manifest from filesystem with validation.
    pub fn from_file<P: AsRef<Path>>(path: P) -> ContainerResult<Self> {
        let content = std::fs::read_to_string(&path).map_err(|e| ContainerError::IoError {
            path: path.as_ref().to_path_buf(),
            source: e,
        })?;

        let manifest: ContainerManifest = serde_json::from_str(&content)
            .map_err(|e| ContainerError::InvalidManifest(e.to_string()))?;

        manifest.validate()?;
        Ok(manifest)
    }

    /// Serializes validated manifest to filesystem for deployment.
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> ContainerResult<()> {
        self.validate()?;

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ContainerError::JsonError { source: e })?;

        std::fs::write(&path, content).map_err(|e| ContainerError::IoError {
            path: path.as_ref().to_path_buf(),
            source: e,
        })?;

        Ok(())
    }

    /// Ensures manifest integrity before container deployment.
    /// Prevents runtime failures from malformed configuration.
    pub fn validate(&self) -> ContainerResult<()> {
        // Validate container name format and presence
        if self.name.is_empty() {
            return Err(ContainerError::ManifestValidation(
                "Container name cannot be empty".to_string(),
            ));
        }

        if !self
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ContainerError::ManifestValidation(
                "Container name can only contain alphanumeric characters, hyphens, and underscores"
                    .to_string(),
            ));
        }

        // Validate version format
        self.version.validate()?;

        // Ensure required default script is defined
        if !self.scripts.contains_key("default") {
            return Err(ContainerError::MissingDefaultScript);
        }

        // Validate all script paths are non-empty
        for (script_name, script_path) in &self.scripts {
            if script_path.is_empty() {
                return Err(ContainerError::ManifestValidation(format!(
                    "Script '{}' has empty path",
                    script_name
                )));
            }
        }

        // Validate dependencies
        for dependency in &self.dependencies {
            if dependency.name.is_empty() {
                return Err(ContainerError::InvalidDependency {
                    package: "".to_string(),
                    reason: "Dependency name cannot be empty".to_string(),
                });
            }

            if dependency.version.is_empty() {
                return Err(ContainerError::InvalidDependency {
                    package: dependency.name.clone(),
                    reason: "Dependency version cannot be empty".to_string(),
                });
            }

            // Basic version format validation
            if dependency.version.parse::<Version>().is_err() {
                return Err(ContainerError::InvalidDependency {
                    package: dependency.name.clone(),
                    reason: format!("Invalid version format: {}", dependency.version),
                });
            }
        }

        Ok(())
    }

    pub fn default_script(&self) -> ContainerResult<&String> {
        self.scripts
            .get("default")
            .ok_or(ContainerError::MissingDefaultScript)
    }

    pub fn get_script(&self, name: &str) -> ContainerResult<&String> {
        self.scripts
            .get(name)
            .ok_or(ContainerError::ScriptNotFound {
                container: self.name.clone(),
                script: name.to_string(),
            })
    }

    pub fn add_script(&mut self, name: String, path: String) {
        self.scripts.insert(name, path);
    }

    pub fn add_dependency(&mut self, dependency: Dependency) {
        self.dependencies.push(dependency);
    }
}

