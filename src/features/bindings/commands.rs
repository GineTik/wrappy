use clap::Subcommand;
use std::path::PathBuf;

use crate::features::bindings::BindingManager;
use crate::features::container::{Container, ContainerService};
use crate::shared::error::ContainerError;

#[derive(Subcommand)]
pub enum BindingsCommands {
    /// List all active bindings
    List,
    /// Enable bindings for a container
    Enable {
        /// Container name or path to enable bindings for
        container: String,
        /// Only enable executable bindings
        #[arg(long)]
        executables_only: bool,
        /// Only enable config bindings
        #[arg(long)]
        configs_only: bool,
        /// Only enable data bindings
        #[arg(long)]
        data_only: bool,
    },
    /// Disable bindings for a container
    Disable {
        /// Container name or path to disable bindings for
        container: String,
    },
    /// Show bindings configuration for a container
    Show {
        /// Container name or path to show bindings for
        container: String,
    },
}

pub struct BindingsHandler;

impl BindingsHandler {
    /// Routes and executes the appropriate bindings command
    pub fn execute_command(command: BindingsCommands) -> i32 {
        match command {
            BindingsCommands::List => Self::handle_list_command(),
            BindingsCommands::Enable { 
                container, 
                executables_only, 
                configs_only, 
                data_only 
            } => Self::handle_enable_command(
                container, 
                executables_only, 
                configs_only, 
                data_only
            ),
            BindingsCommands::Disable { container } => {
                Self::handle_disable_command(container)
            }
            BindingsCommands::Show { container } => {
                Self::handle_show_command(container)
            }
        }
    }

    /// Handles the list command execution
    fn handle_list_command() -> i32 {
        match Self::list_active_bindings() {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("❌ Failed to list bindings: {}", error);
                1
            }
        }
    }

    /// Handles the enable command execution
    fn handle_enable_command(
        container_input: String,
        executables_only: bool,
        configs_only: bool,
        data_only: bool,
    ) -> i32 {
        match Self::enable_bindings(container_input, executables_only, configs_only, data_only) {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("❌ Failed to enable bindings: {}", error);
                1
            }
        }
    }

    /// Handles the disable command execution
    fn handle_disable_command(container_input: String) -> i32 {
        match Self::disable_bindings(container_input) {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("❌ Failed to disable bindings: {}", error);
                1
            }
        }
    }

    /// Handles the show command execution
    fn handle_show_command(container_input: String) -> i32 {
        match Self::show_bindings(container_input) {
            Ok(()) => 0,
            Err(error) => {
                eprintln!("❌ Failed to show bindings: {}", error);
                1
            }
        }
    }

    /// Lists all active bindings in the system
    fn list_active_bindings() -> Result<(), ContainerError> {
        let binding_manager = BindingManager::new()?;
        let wrappers = binding_manager.list_active_wrappers()?;

        println!("🔗 Active Wrappy Bindings");
        println!();

        if wrappers.is_empty() {
            println!("  No active bindings found.");
            println!("  Use 'wrappy bindings enable <container>' to create bindings.");
        } else {
            println!("  Wrapper Scripts in ~/.local/bin/:");
            for wrapper in wrappers {
                println!("    📋 {}", wrapper);
            }
        }

        Ok(())
    }

    /// Enables bindings for a container
    fn enable_bindings(
        container_input: String,
        executables_only: bool,
        configs_only: bool,
        data_only: bool,
    ) -> Result<(), ContainerError> {
        let container = Self::resolve_container(container_input)?;
        let binding_manager = BindingManager::new()?;

        // Check if container has any bindings configured
        if container.manifest.bindings.is_empty() {
            println!("ℹ️  Container '{}' has no bindings configured.", container.name());
            println!("   Add bindings to the manifest.json file to enable integration.");
            return Ok(());
        }

        // Filter bindings based on flags
        let mut filtered_container = container.clone();
        if executables_only {
            filtered_container.manifest.bindings.configs.clear();
            filtered_container.manifest.bindings.data.clear();
        } else if configs_only {
            filtered_container.manifest.bindings.executables.clear();
            filtered_container.manifest.bindings.data.clear();
        } else if data_only {
            filtered_container.manifest.bindings.executables.clear();
            filtered_container.manifest.bindings.configs.clear();
        }

        println!("🔗 Enabling bindings for container '{}'...", container.name());
        let active_bindings = binding_manager.install_bindings(&filtered_container)?;

        if active_bindings.is_empty() {
            println!("ℹ️  No bindings were created (they may already exist).");
        }

        Ok(())
    }

    /// Disables bindings for a container
    fn disable_bindings(container_input: String) -> Result<(), ContainerError> {
        let container = Self::resolve_container(container_input)?;
        let binding_manager = BindingManager::new()?;

        println!("🗑️  Disabling bindings for container '{}'...", container.name());
        binding_manager.remove_bindings(&container)?;

        Ok(())
    }

    /// Shows bindings configuration for a container
    fn show_bindings(container_input: String) -> Result<(), ContainerError> {
        let container = Self::resolve_container(container_input)?;

        println!("🔗 Bindings configuration for container '{}'", container.name());
        println!();

        let bindings = &container.manifest.bindings;

        if bindings.is_empty() {
            println!("  No bindings configured for this container.");
            println!();
            println!("  To add bindings, edit the manifest.json file:");
            println!("  {{");
            println!("    \"bindings\": {{");
            println!("      \"executables\": [");
            println!("        {{\"source\": \"bin/myapp\", \"target\": \"~/.local/bin/myapp\"}}");
            println!("      ],");
            println!("      \"configs\": [");
            println!("        {{\"source\": \"config/myapp\", \"target\": \"~/.config/myapp\", \"type\": \"symlink\"}}");
            println!("      ]");
            println!("    }}");
            println!("  }}");
            return Ok(());
        }

        // Show executable bindings
        if !bindings.executables.is_empty() {
            println!("  📋 Executable Bindings:");
            for executable in &bindings.executables {
                let display_name = executable.display_name
                    .as_ref()
                    .unwrap_or(&executable.source);
                println!("    {} -> {} ({})", 
                         executable.source, executable.target, 
                         format!("{:?}", executable.binding_type).to_lowercase());
                if let Some(display) = &executable.display_name {
                    println!("      Display name: {}", display);
                }
            }
            println!();
        }

        // Show config bindings
        if !bindings.configs.is_empty() {
            println!("  ⚙️  Config Bindings:");
            for config in &bindings.configs {
                println!("    {} -> {} ({})", 
                         config.source, config.target,
                         format!("{:?}", config.binding_type).to_lowercase());
                if config.backup_existing {
                    println!("      Backup existing: yes");
                }
            }
            println!();
        }

        // Show data bindings
        if !bindings.data.is_empty() {
            println!("  💾 Data Bindings:");
            for data in &bindings.data {
                println!("    {} -> {} ({})", 
                         data.source, data.target,
                         format!("{:?}", data.binding_type).to_lowercase());
                if data.backup_existing {
                    println!("      Backup existing: yes");
                }
            }
            println!();
        }

        Ok(())
    }

    /// Resolves container input to Container instance
    fn resolve_container(container_input: String) -> Result<Container, ContainerError> {
        // Try as path first
        let path = PathBuf::from(&container_input);
        if path.exists() && path.is_dir() {
            return ContainerService::load_from_directory(&path);
        }

        // For now, just try as path - in the future we could search by name
        Err(ContainerError::InvalidPath {
            path,
            reason: format!("Container '{}' not found. Please provide a valid container directory path.", container_input),
        })
    }
}