use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

use crate::features::bindings::{
    ActiveBinding, BindingType, BindingsConfig, ConfigBinding, DataBinding, 
    ExecutableBinding, WrapperGenerator,
};
use crate::features::Container;
use crate::shared::error::{ContainerError, ContainerResult};

/// Manages container bindings to host system including executables, configs, and data.
pub struct BindingManager {
    user_bin_dir: PathBuf,
    user_config_dir: PathBuf,
    user_data_dir: PathBuf,
    wrapper_generator: WrapperGenerator,
}

impl BindingManager {
    /// Creates binding manager with standard user directories.
    pub fn new() -> ContainerResult<Self> {
        let home = dirs::home_dir().ok_or_else(|| {
            ContainerError::InvalidPath {
                path: PathBuf::from("~"),
                reason: "Could not determine home directory".to_string(),
            }
        })?;

        let user_bin_dir = home.join(".local/bin");
        let user_config_dir = home.join(".config");
        let user_data_dir = home.join(".local/share");

        // Ensure directories exist
        for dir in &[&user_bin_dir, &user_config_dir, &user_data_dir] {
            fs::create_dir_all(dir).map_err(|e| ContainerError::IoError {
                path: dir.to_path_buf(),
                source: e,
            })?;
        }

        let wrapper_generator = WrapperGenerator::new(user_bin_dir.clone());

        Ok(Self {
            user_bin_dir,
            user_config_dir,
            user_data_dir,
            wrapper_generator,
        })
    }

    /// Installs all bindings for a container based on its manifest configuration.
    pub fn install_bindings(&self, container: &Container) -> ContainerResult<Vec<ActiveBinding>> {
        let mut active_bindings = Vec::new();

        // Install executable bindings
        for executable in &container.manifest.bindings.executables {
            let binding = self.install_executable_binding(container, executable)?;
            active_bindings.push(binding);
        }

        // Install config bindings
        for config in &container.manifest.bindings.configs {
            let binding = self.install_config_binding(container, config)?;
            active_bindings.push(binding);
        }

        // Install data bindings
        for data in &container.manifest.bindings.data {
            let binding = self.install_data_binding(container, data)?;
            active_bindings.push(binding);
        }

        println!("✅ Installed {} bindings for container '{}'", 
                 active_bindings.len(), container.name());

        Ok(active_bindings)
    }

    /// Removes all bindings for a container.
    pub fn remove_bindings(&self, container: &Container) -> ContainerResult<()> {
        let mut removed_count = 0;

        // Remove executable bindings
        for executable in &container.manifest.bindings.executables {
            if self.remove_executable_binding(container, executable)? {
                removed_count += 1;
            }
        }

        // Remove config bindings
        for config in &container.manifest.bindings.configs {
            if self.remove_config_binding(container, config)? {
                removed_count += 1;
            }
        }

        // Remove data bindings
        for data in &container.manifest.bindings.data {
            if self.remove_data_binding(container, data)? {
                removed_count += 1;
            }
        }

        if removed_count > 0 {
            println!("✅ Removed {} bindings for container '{}'", 
                     removed_count, container.name());
        } else {
            println!("ℹ️  No bindings found to remove for container '{}'", container.name());
        }

        Ok(())
    }

    /// Lists all active wrapper scripts managed by this system.
    pub fn list_active_wrappers(&self) -> ContainerResult<Vec<String>> {
        self.wrapper_generator.list_wrappers()
    }

    /// Installs binding for a single executable.
    fn install_executable_binding(
        &self,
        container: &Container,
        executable: &ExecutableBinding,
    ) -> ContainerResult<ActiveBinding> {
        let source_path = container.path.join(&executable.source);
        let target_path = self.expand_path(&executable.target)?;

        // Validate source exists and is executable
        if !source_path.exists() {
            return Err(ContainerError::ScriptNotFound {
                container: container.name().to_string(),
                script: executable.source.clone(),
            });
        }

        if !source_path.is_file() {
            return Err(ContainerError::InvalidPath {
                path: source_path,
                reason: "Source is not a file".to_string(),
            });
        }

        match executable.binding_type {
            BindingType::Wrapper => {
                let executable_name = target_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| ContainerError::InvalidPath {
                        path: target_path.clone(),
                        reason: "Invalid executable name".to_string(),
                    })?;

                self.wrapper_generator.create_wrapper(
                    executable_name,
                    container.name(),
                    &source_path,
                    executable.display_name.as_deref(),
                )?;

                println!("🔗 Created wrapper: {} -> {}", 
                         executable_name, source_path.display());
            }
            BindingType::Symlink => {
                self.create_symlink(&source_path, &target_path)?;
                println!("🔗 Created symlink: {} -> {}", 
                         target_path.display(), source_path.display());
            }
            BindingType::Copy => {
                fs::copy(&source_path, &target_path).map_err(|e| ContainerError::IoError {
                    path: target_path.clone(),
                    source: e,
                })?;
                println!("📋 Copied executable: {} -> {}", 
                         source_path.display(), target_path.display());
            }
        }

        Ok(ActiveBinding {
            container_name: container.name().to_string(),
            source_path,
            target_path,
            binding_type: executable.binding_type.clone(),
            created_at: std::time::SystemTime::now(),
        })
    }

    /// Installs binding for a configuration directory.
    fn install_config_binding(
        &self,
        container: &Container,
        config: &ConfigBinding,
    ) -> ContainerResult<ActiveBinding> {
        let source_path = container.path.join(&config.source);
        let target_path = self.expand_path(&config.target)?;

        self.install_directory_binding(
            container,
            &source_path,
            &target_path,
            &config.binding_type,
            config.backup_existing,
            "config",
        )
    }

    /// Installs binding for a data directory.
    fn install_data_binding(
        &self,
        container: &Container,
        data: &DataBinding,
    ) -> ContainerResult<ActiveBinding> {
        let source_path = container.path.join(&data.source);
        let target_path = self.expand_path(&data.target)?;

        self.install_directory_binding(
            container,
            &source_path,
            &target_path,
            &data.binding_type,
            data.backup_existing,
            "data",
        )
    }

    /// Generic directory binding installation.
    fn install_directory_binding(
        &self,
        container: &Container,
        source_path: &Path,
        target_path: &Path,
        binding_type: &BindingType,
        backup_existing: bool,
        binding_kind: &str,
    ) -> ContainerResult<ActiveBinding> {
        // Validate source exists
        if !source_path.exists() {
            return Err(ContainerError::InvalidPath {
                path: source_path.to_path_buf(),
                reason: format!("Source {} directory does not exist", binding_kind),
            });
        }

        // Handle existing target
        if target_path.exists() {
            if backup_existing {
                let backup_path = format!("{}.wrappy-backup", target_path.display());
                fs::rename(target_path, &backup_path).map_err(|e| ContainerError::IoError {
                    path: target_path.to_path_buf(),
                    source: e,
                })?;
                println!("📦 Backed up existing {} to {}", 
                         target_path.display(), backup_path);
            } else {
                return Err(ContainerError::InvalidPath {
                    path: target_path.to_path_buf(),
                    reason: format!("Target {} already exists", binding_kind),
                });
            }
        }

        // Create parent directory if needed
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|e| ContainerError::IoError {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        match binding_type {
            BindingType::Symlink => {
                self.create_symlink(source_path, target_path)?;
                println!("🔗 Created {} symlink: {} -> {}", 
                         binding_kind, target_path.display(), source_path.display());
            }
            BindingType::Copy => {
                self.copy_directory(source_path, target_path)?;
                println!("📋 Copied {} directory: {} -> {}", 
                         binding_kind, source_path.display(), target_path.display());
            }
            BindingType::Wrapper => {
                return Err(ContainerError::InvalidPath {
                    path: target_path.to_path_buf(),
                    reason: format!("Wrapper binding not supported for {} directories", binding_kind),
                });
            }
        }

        Ok(ActiveBinding {
            container_name: container.name().to_string(),
            source_path: source_path.to_path_buf(),
            target_path: target_path.to_path_buf(),
            binding_type: binding_type.clone(),
            created_at: std::time::SystemTime::now(),
        })
    }

    /// Removes executable binding.
    fn remove_executable_binding(
        &self,
        container: &Container,
        executable: &ExecutableBinding,
    ) -> ContainerResult<bool> {
        let target_path = self.expand_path(&executable.target)?;

        match executable.binding_type {
            BindingType::Wrapper => {
                let executable_name = target_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| ContainerError::InvalidPath {
                        path: target_path.clone(),
                        reason: "Invalid executable name".to_string(),
                    })?;

                self.wrapper_generator.remove_wrapper(executable_name)?;
                println!("🗑️  Removed wrapper: {}", executable_name);
                Ok(true)
            }
            _ => {
                if target_path.exists() {
                    fs::remove_file(&target_path).map_err(|e| ContainerError::IoError {
                        path: target_path.clone(),
                        source: e,
                    })?;
                    println!("🗑️  Removed executable: {}", target_path.display());
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    /// Removes config binding.
    fn remove_config_binding(
        &self,
        container: &Container,
        config: &ConfigBinding,
    ) -> ContainerResult<bool> {
        let target_path = self.expand_path(&config.target)?;
        self.remove_directory_binding(&target_path, "config")
    }

    /// Removes data binding.
    fn remove_data_binding(
        &self,
        container: &Container,
        data: &DataBinding,
    ) -> ContainerResult<bool> {
        let target_path = self.expand_path(&data.target)?;
        self.remove_directory_binding(&target_path, "data")
    }

    /// Generic directory binding removal.
    fn remove_directory_binding(
        &self,
        target_path: &Path,
        binding_kind: &str,
    ) -> ContainerResult<bool> {
        if target_path.exists() {
            if target_path.is_dir() {
                fs::remove_dir_all(target_path).map_err(|e| ContainerError::IoError {
                    path: target_path.to_path_buf(),
                    source: e,
                })?;
            } else {
                fs::remove_file(target_path).map_err(|e| ContainerError::IoError {
                    path: target_path.to_path_buf(),
                    source: e,
                })?;
            }
            println!("🗑️  Removed {} binding: {}", binding_kind, target_path.display());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Creates a symbolic link with error handling.
    fn create_symlink(&self, source: &Path, target: &Path) -> ContainerResult<()> {
        unix_fs::symlink(source, target).map_err(|e| ContainerError::IoError {
            path: target.to_path_buf(),
            source: e,
        })?;
        Ok(())
    }

    /// Recursively copies a directory.
    fn copy_directory(&self, source: &Path, target: &Path) -> ContainerResult<()> {
        fs::create_dir_all(target).map_err(|e| ContainerError::IoError {
            path: target.to_path_buf(),
            source: e,
        })?;

        for entry in fs::read_dir(source).map_err(|e| ContainerError::IoError {
            path: source.to_path_buf(),
            source: e,
        })? {
            let entry = entry.map_err(|e| ContainerError::IoError {
                path: source.to_path_buf(),
                source: e,
            })?;

            let source_path = entry.path();
            let target_path = target.join(entry.file_name());

            if source_path.is_dir() {
                self.copy_directory(&source_path, &target_path)?;
            } else {
                fs::copy(&source_path, &target_path).map_err(|e| ContainerError::IoError {
                    path: target_path,
                    source: e,
                })?;
            }
        }

        Ok(())
    }

    /// Expands ~ in paths to actual home directory.
    fn expand_path(&self, path: &str) -> ContainerResult<PathBuf> {
        if path.starts_with("~/") {
            let home = dirs::home_dir().ok_or_else(|| {
                ContainerError::InvalidPath {
                    path: PathBuf::from(path),
                    reason: "Could not determine home directory".to_string(),
                }
            })?;
            Ok(home.join(&path[2..]))
        } else {
            Ok(PathBuf::from(path))
        }
    }
}
