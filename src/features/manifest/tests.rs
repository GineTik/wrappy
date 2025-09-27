#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ContainerManifest, Version, ContainerType, Dependency};
    use crate::error::ContainerError;
    use tempfile::TempDir;

    #[test]
    fn test_manifest_creation() {
        let manifest = ContainerManifest::new(
            "test-app".to_string(),
            Version::new(1, 0, 0),
            ContainerType::Application,
        );

        assert_eq!(manifest.name, "test-app");
        assert_eq!(manifest.version, Version::new(1, 0, 0));
        assert_eq!(manifest.container_type, ContainerType::Application);
        assert!(manifest.scripts.contains_key("default"));
    }

    #[test]
    fn test_manifest_validation() {
        let mut manifest = ContainerManifest::new(
            "test-app".to_string(),
            Version::new(1, 0, 0),
            ContainerType::Application,
        );

        // Valid manifest should pass
        assert!(manifest.validate().is_ok());

        // Empty name should fail
        manifest.name = "".to_string();
        assert!(manifest.validate().is_err());

        // Invalid name with special characters should fail
        manifest.name = "test@app".to_string();
        assert!(manifest.validate().is_err());

        // Valid name should pass again
        manifest.name = "test-app_123".to_string();
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_missing_default_script() {
        let mut manifest = ContainerManifest::new(
            "test-app".to_string(),
            Version::new(1, 0, 0),
            ContainerType::Application,
        );

        // Remove default script
        manifest.scripts.remove("default");

        // Should fail validation
        assert!(manifest.validate().is_err());
        assert!(matches!(
            manifest.validate().unwrap_err(),
            ContainerError::MissingDefaultScript
        ));
    }

    #[test]
    fn test_manifest_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_path = temp_dir.path().join("manifest.json");

        let original_manifest = ContainerManifest::new(
            "test-app".to_string(),
            Version::new(1, 0, 0),
            ContainerType::Application,
        );

        // Save to file
        original_manifest.to_file(&manifest_path).unwrap();

        // Load from file
        let loaded_manifest = ContainerManifest::from_file(&manifest_path).unwrap();

        assert_eq!(original_manifest.name, loaded_manifest.name);
        assert_eq!(original_manifest.version, loaded_manifest.version);
        assert_eq!(original_manifest.container_type, loaded_manifest.container_type);
    }

    #[test]
    fn test_dependency_validation() {
        let mut manifest = ContainerManifest::new(
            "test-app".to_string(),
            Version::new(1, 0, 0),
            ContainerType::Application,
        );

        // Add valid dependency
        manifest.add_dependency(Dependency {
            name: "node-runtime".to_string(),
            version: "18.0.0".to_string(),
            optional: false,
        });

        assert!(manifest.validate().is_ok());

        // Add invalid dependency with empty name
        manifest.add_dependency(Dependency {
            name: "".to_string(),
            version: "1.0.0".to_string(),
            optional: false,
        });

        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_script_operations() {
        let mut manifest = ContainerManifest::new(
            "test-app".to_string(),
            Version::new(1, 0, 0),
            ContainerType::Application,
        );

        // Get default script
        assert!(manifest.default_script().is_ok());

        // Add custom script
        manifest.add_script("build".to_string(), "scripts/build.sh".to_string());

        // Get custom script
        assert!(manifest.get_script("build").is_ok());
        assert_eq!(manifest.get_script("build").unwrap(), "scripts/build.sh");

        // Try to get non-existent script
        assert!(manifest.get_script("nonexistent").is_err());
    }
}