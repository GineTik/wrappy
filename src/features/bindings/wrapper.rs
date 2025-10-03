use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::shared::error::{ContainerError, ContainerResult};

/// Generates wrapper scripts for container executables with execution tracking.
pub struct WrapperGenerator {
    target_dir: PathBuf,
}

impl WrapperGenerator {
    /// Creates wrapper generator for specified target directory.
    pub fn new(target_dir: PathBuf) -> Self {
        Self { target_dir }
    }

    /// Creates wrapper generator for user's local bin directory.
    pub fn for_user_bin() -> ContainerResult<Self> {
        let home = dirs::home_dir().ok_or_else(|| {
            ContainerError::InvalidPath {
                path: PathBuf::from("~"),
                reason: "Could not determine home directory".to_string(),
            }
        })?;

        let target_dir = home.join(".local/bin");
        fs::create_dir_all(&target_dir).map_err(|e| ContainerError::IoError {
            path: target_dir.clone(),
            source: e,
        })?;

        Ok(Self::new(target_dir))
    }

    /// Generates wrapper script for executable with console output tracking.
    pub fn create_wrapper(
        &self,
        executable_name: &str,
        container_name: &str,
        executable_path: &Path,
        display_name: Option<&str>,
    ) -> ContainerResult<PathBuf> {
        let wrapper_path = self.target_dir.join(executable_name);
        let display = display_name.unwrap_or(executable_name);

        let script_content = self.generate_wrapper_script(
            container_name,
            executable_path,
            display,
        );

        // Write wrapper script
        fs::write(&wrapper_path, script_content).map_err(|e| ContainerError::IoError {
            path: wrapper_path.clone(),
            source: e,
        })?;

        // Make executable
        let mut perms = fs::metadata(&wrapper_path)
            .map_err(|e| ContainerError::IoError {
                path: wrapper_path.clone(),
                source: e,
            })?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&wrapper_path, perms).map_err(|e| ContainerError::IoError {
            path: wrapper_path.clone(),
            source: e,
        })?;

        Ok(wrapper_path)
    }

    /// Removes wrapper script from target directory.
    pub fn remove_wrapper(&self, executable_name: &str) -> ContainerResult<()> {
        let wrapper_path = self.target_dir.join(executable_name);
        
        if wrapper_path.exists() {
            fs::remove_file(&wrapper_path).map_err(|e| ContainerError::IoError {
                path: wrapper_path,
                source: e,
            })?;
        }

        Ok(())
    }

    /// Generates the actual wrapper script content with execution tracking.
    fn generate_wrapper_script(
        &self,
        container_name: &str,
        executable_path: &Path,
        display_name: &str,
    ) -> String {
        format!(
            r#"#!/bin/bash
# Wrappy container wrapper for {container_name}/{display_name}
# Generated automatically - do not modify

CONTAINER_NAME="{container_name}"
DISPLAY_NAME="{display_name}"
EXECUTABLE_PATH="{executable_path}"

# Function to get current timestamp
get_timestamp() {{
    date '+%Y-%m-%d %H:%M:%S'
}}

# Function to calculate duration
calculate_duration() {{
    local start_time=$1
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    if [ $duration -lt 60 ]; then
        echo "${{duration}}s"
    elif [ $duration -lt 3600 ]; then
        echo "$((duration / 60))m $((duration % 60))s"
    else
        echo "$((duration / 3600))h $((duration % 3600 / 60))m $((duration % 60))s"
    fi
}}

# Record start time
START_TIME=$(date +%s)
TIMESTAMP=$(get_timestamp)

# Console output for container start
echo "🚀 [$TIMESTAMP] Starting $CONTAINER_NAME/$DISPLAY_NAME"

# Execute the actual command with all arguments
"$EXECUTABLE_PATH" "$@"
EXIT_CODE=$?

# Record end time and calculate duration
END_TIMESTAMP=$(get_timestamp)
DURATION=$(calculate_duration $START_TIME)

# Console output for container end
if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ [$END_TIMESTAMP] Finished $CONTAINER_NAME/$DISPLAY_NAME (took $DURATION)"
else
    echo "❌ [$END_TIMESTAMP] Failed $CONTAINER_NAME/$DISPLAY_NAME (exit code: $EXIT_CODE, took $DURATION)"
fi

# Preserve original exit code
exit $EXIT_CODE
"#,
            container_name = container_name,
            display_name = display_name,
            executable_path = executable_path.display()
        )
    }

    /// Lists all wrapper scripts in the target directory.
    pub fn list_wrappers(&self) -> ContainerResult<Vec<String>> {
        if !self.target_dir.exists() {
            return Ok(Vec::new());
        }

        let mut wrappers = Vec::new();
        
        for entry in fs::read_dir(&self.target_dir).map_err(|e| ContainerError::IoError {
            path: self.target_dir.clone(),
            source: e,
        })? {
            let entry = entry.map_err(|e| ContainerError::IoError {
                path: self.target_dir.clone(),
                source: e,
            })?;

            if entry.file_type().map_err(|e| ContainerError::IoError {
                path: entry.path(),
                source: e,
            })?.is_file() {
                // Check if it's a wrappy wrapper by reading first few lines
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    if content.contains("# Wrappy container wrapper") {
                        if let Some(name) = entry.file_name().to_str() {
                            wrappers.push(name.to_string());
                        }
                    }
                }
            }
        }

        wrappers.sort();
        Ok(wrappers)
    }
}
