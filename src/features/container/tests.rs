#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Container, ContainerManifest, Version, ContainerType, ContainerStatus, Dependency};
    use crate::error::ContainerError;
    use tempfile::TempDir;
    use std::fs;
    use std::collections::HashMap;
    use std::path::Path;

    fn create_test_container_structure(base_path: &Path, manifest: &ContainerManifest) {
        // Create directories
        fs::create_dir_all(base_path.join("scripts")).unwrap();
        fs::create_dir_all(base_path.join("content")).unwrap();
        fs::create_dir_all(base_path.join("config")).unwrap();

        // Create manifest file
        manifest.to_file(base_path.join("manifest.json")).unwrap();

        // Create default script
        fs::write(base_path.join("scripts/default.sh"), "#!/bin/bash\necho 'Hello World'").unwrap();

        // Create config files
        fs::write(base_path.join("config/permissions.json"), "{}").unwrap();
        fs::write(base_path.join("config/environment.json"), "{}").unwrap();
    }

    #[test]
    fn test_container_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manifest = ContainerManifest::new(
            "test-app".to_string(),
            Version::new(1, 0, 0),
            ContainerType::Application,
        );

        create_test_container_structure(temp_dir.path(), &manifest);

        let container = Container::new(manifest.clone(), temp_dir.path().to_path_buf()).unwrap();

        assert_eq!(container.name(), "test-app");
        assert_eq!(container.version(), &Version::new(1, 0, 0));
        assert_eq!(container.container_type(), &ContainerType::Application);
        assert_eq!(container.runtime.status, ContainerStatus::Ready);
    }

    #[test]
    fn test_container_from_directory() {
        let temp_dir = TempDir::new().unwrap();
        let manifest = ContainerManifest::new(
            "test-app".to_string(),
            Version::new(1, 0, 0),
            ContainerType::Application,
        );

        create_test_container_structure(temp_dir.path(), &manifest);

        let container = Container::from_directory(temp_dir.path()).unwrap();

        assert_eq!(container.name(), "test-app");
        assert_eq!(container.version(), &Version::new(1, 0, 0));
    }

    #[test]
    fn test_invalid_container_structure() {
        let temp_dir = TempDir::new().unwrap();
        let manifest = ContainerManifest::new(
            "test-app".to_string(),
            Version::new(1, 0, 0),
            ContainerType::Application,
        );

        // Only create manifest, missing required directories
        manifest.to_file(temp_dir.path().join("manifest.json")).unwrap();

        let result = Container::new(manifest, temp_dir.path().to_path_buf());
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_default_script() {
        let temp_dir = TempDir::new().unwrap();
        let manifest = ContainerManifest::new(
            "test-app".to_string(),
            Version::new(1, 0, 0),
            ContainerType::Application,
        );

        // Create structure but no default script
        fs::create_dir_all(temp_dir.path().join("scripts")).unwrap();
        fs::create_dir_all(temp_dir.path().join("content")).unwrap();
        fs::create_dir_all(temp_dir.path().join("config")).unwrap();
        manifest.to_file(temp_dir.path().join("manifest.json")).unwrap();
        fs::write(temp_dir.path().join("config/permissions.json"), "{}").unwrap();
        fs::write(temp_dir.path().join("config/environment.json"), "{}").unwrap();

        let result = Container::new(manifest, temp_dir.path().to_path_buf());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ContainerError::MissingDefaultScript));
    }

    #[test]
    fn test_container_runtime_operations() {
        let temp_dir = TempDir::new().unwrap();
        let manifest = ContainerManifest::new(
            "test-app".to_string(),
            Version::new(1, 0, 0),
            ContainerType::Application,
        );

        create_test_container_structure(temp_dir.path(), &manifest);

        let mut container = Container::new(manifest, temp_dir.path().to_path_buf()).unwrap();

        assert!(!container.is_running());

        // Mark as running
        container.mark_running(1234);
        assert!(container.is_running());
        assert_eq!(container.runtime.pid, Some(1234));
        assert!(container.runtime.started_at.is_some());

        // Mark as stopped
        container.mark_stopped(0);
        assert!(!container.is_running());
        assert_eq!(container.runtime.exit_code, Some(0));
        assert!(container.runtime.stopped_at.is_some());
    }

    #[test]
    fn test_script_path_operations() {
        let temp_dir = TempDir::new().unwrap();
        let mut manifest = ContainerManifest::new(
            "test-app".to_string(),
            Version::new(1, 0, 0),
            ContainerType::Application,
        );

        manifest.add_script("build".to_string(), "scripts/build.sh".to_string());
        create_test_container_structure(temp_dir.path(), &manifest);

        // Create build script
        fs::write(temp_dir.path().join("scripts/build.sh"), "#!/bin/bash\necho 'Building...'").unwrap();

        let container = Container::new(manifest, temp_dir.path().to_path_buf()).unwrap();

        // Test default script path
        let default_path = container.get_default_script_path().unwrap();
        assert_eq!(default_path, temp_dir.path().join("scripts/default.sh"));

        // Test custom script path
        let build_path = container.get_script_path("build").unwrap();
        assert_eq!(build_path, temp_dir.path().join("scripts/build.sh"));

        // Test non-existent script
        assert!(container.get_script_path("nonexistent").is_err());
    }

    #[test]
    fn test_dependency_validation() {
        let temp_dir = TempDir::new().unwrap();
        let mut manifest = ContainerManifest::new(
            "test-app".to_string(),
            Version::new(1, 0, 0),
            ContainerType::Application,
        );

        manifest.add_dependency(crate::core::Dependency {
            name: "node-runtime".to_string(),
            version: "18.0.0".to_string(),
            optional: false,
        });

        create_test_container_structure(temp_dir.path(), &manifest);
        let container = Container::new(manifest, temp_dir.path().to_path_buf()).unwrap();

        let mut available_packages = HashMap::new();
        available_packages.insert("node-runtime".to_string(), Version::new(18, 0, 0));

        // Should pass with exact version
        assert!(container.validate_dependencies(&available_packages).is_ok());

        // Should pass with compatible version
        available_packages.insert("node-runtime".to_string(), Version::new(18, 1, 0));
        assert!(container.validate_dependencies(&available_packages).is_ok());

        // Should fail with incompatible version
        available_packages.insert("node-runtime".to_string(), Version::new(17, 0, 0));
        assert!(container.validate_dependencies(&available_packages).is_err());

        // Should fail with missing package
        available_packages.remove("node-runtime");
        assert!(container.validate_dependencies(&available_packages).is_err());
    }

    #[test]
    fn test_json_serialization() {
        let temp_dir = TempDir::new().unwrap();
        let manifest = ContainerManifest::new(
            "test-app".to_string(),
            Version::new(1, 0, 0),
            ContainerType::Application,
        );

        create_test_container_structure(temp_dir.path(), &manifest);
        let container = Container::new(manifest, temp_dir.path().to_path_buf()).unwrap();

        // Test serialization
        let json = container.to_json().unwrap();
        assert!(json.contains("test-app"));

        // Test deserialization
        let deserialized = Container::from_json(&json).unwrap();
        assert_eq!(deserialized.name(), container.name());
        assert_eq!(deserialized.version(), container.version());
    }
}