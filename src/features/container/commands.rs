use clap::Subcommand;
use std::env;
use std::path::PathBuf;

use crate::features::container::{Container, ContainerService};
use crate::shared::error::ContainerError;

#[derive(Subcommand)]
pub enum ContainerCommands {
    /// Validate container structure in the current or specified directory
    Validate {
        /// Directory path to validate (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
        
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
}

pub struct ContainerHandler;

impl ContainerHandler {

    /// Routes and executes the appropriate command
    pub fn execute_command(command: ContainerCommands) -> i32 {
        match command {
            ContainerCommands::Validate { path, verbose } => {
                Self::handle_validate_command(path, verbose)
            }
        }
    }

    /// Handles the validate command execution
    pub fn handle_validate_command(path: Option<PathBuf>, verbose: bool) -> i32 {
        let container_path = match Self::resolve_container_path(path) {
            Ok(path) => path,
            Err(exit_code) => return exit_code,
        };

        Self::print_validation_start(&container_path, verbose);

        match Self::validate_container_at_path(&container_path) {
            Ok(container) => {
                Self::print_validation_success(&container, verbose);
                0
            }
            Err(error) => {
                Self::print_validation_error(&error, verbose);
                1
            }
        }
    }

    /// Resolves the container path from optional input or current directory
    fn resolve_container_path(path: Option<PathBuf>) -> Result<PathBuf, i32> {
        match path {
            Some(p) => Ok(p),
            None => env::current_dir().map_err(|e| {
                eprintln!("Error: Unable to get current directory: {}", e);
                2
            })
        }
    }

    /// Prints validation start message if verbose mode is enabled
    fn print_validation_start(path: &PathBuf, verbose: bool) {
        if verbose {
            println!("Validating container at: {}", path.display());
        }
    }

    /// Validates container at the specified path using service
    fn validate_container_at_path(path: &PathBuf) -> Result<Container, ContainerError> {
        ContainerService::load_from_directory(path)
    }

    /// Prints success message and container details
    fn print_validation_success(container: &Container, verbose: bool) {
        println!(" Container validation successful!");
        
        if verbose {
            Self::print_container_details(container);
        } else {
            println!("Container '{}' (v{}) is valid", container.name(), container.version());
        }
    }

    /// Prints detailed container information
    fn print_container_details(container: &Container) {
        println!("Container details:");
        println!("  Name: {}", container.name());
        println!("  Version: {}", container.version());
        println!("  Path: {}", container.path.display());
        
        Self::print_scripts_info(container);
        Self::print_dependencies_info(container);
    }

    /// Prints container scripts information
    fn print_scripts_info(container: &Container) {
        if !container.manifest.scripts.is_empty() {
            println!("  Scripts:");
            for (name, path) in &container.manifest.scripts {
                println!("    {}: {}", name, path);
            }
        }
    }

    /// Prints container dependencies information
    fn print_dependencies_info(container: &Container) {
        if !container.manifest.dependencies.is_empty() {
            println!("  Dependencies:");
            for dep in &container.manifest.dependencies {
                println!("    {}: {}", dep.name, dep.version);
            }
        }
    }

    /// Prints validation error message and suggestions
    fn print_validation_error(error: &ContainerError, verbose: bool) {
        eprintln!("L Container validation failed: {}", error);
        
        if verbose {
            eprintln!("Error details: {:?}", error);
            Self::print_error_suggestions(error);
        }
    }

    /// Prints helpful suggestions based on error type
    fn print_error_suggestions(error: &ContainerError) {
        match error {
            ContainerError::InvalidPath { .. } => {
                eprintln!("\nSuggestion: Ensure the path exists and is accessible");
            }
            ContainerError::InvalidStructure(msg) if msg.contains("manifest.json") => {
                eprintln!("\nSuggestion: Create a manifest.json file in the container directory");
            }
            ContainerError::InvalidStructure(msg) if msg.contains("directory") => {
                eprintln!("\nSuggestion: Ensure required directories (scripts, content, config) exist");
            }
            ContainerError::MissingDefaultScript => {
                eprintln!("\nSuggestion: Ensure the default script exists in the scripts directory");
            }
            ContainerError::ScriptNotFound { script, .. } => {
                eprintln!("\nSuggestion: Ensure script '{}' exists in the scripts directory", script);
            }
            _ => {}
        }
    }
}
