use clap::{Parser, Subcommand};
use std::env;
use std::path::PathBuf;
use std::process;
use wrappy::*;

#[derive(Parser)]
#[command(
    name = "wrappy",
    about = "Container file system abstraction - manage isolated application environments",
    version = env!("CARGO_PKG_VERSION")
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { path, verbose } => {
            let result = validate_container(path, verbose);
            process::exit(result);
        }
    }
}

/// Validates container structure and reports results to the user.
/// Returns exit code: 0 for success, 1 for validation errors, 2 for system errors.
fn validate_container(path: Option<PathBuf>, verbose: bool) -> i32 {
    // Determine the path to validate
    let container_path = match path {
        Some(p) => p,
        None => match env::current_dir() {
            Ok(dir) => dir,
            Err(e) => {
                eprintln!("Error: Unable to get current directory: {}", e);
                return 2;
            }
        }
    };

    if verbose {
        println!("Validating container at: {}", container_path.display());
    }

    // Attempt to load and validate the container
    match Container::from_directory(&container_path) {
        Ok(container) => {
            println!("✅ Container validation successful!");
            
            if verbose {
                println!("Container details:");
                println!("  Name: {}", container.name());
                println!("  Version: {}", container.version());
                println!("  Type: {:?}", container.container_type());
                println!("  Path: {}", container.path.display());
                
                // Show scripts information
                if !container.manifest.scripts.is_empty() {
                    println!("  Scripts:");
                    for (name, path) in &container.manifest.scripts {
                        println!("    {}: {}", name, path);
                    }
                }
                
                // Show dependencies information
                if !container.manifest.dependencies.is_empty() {
                    println!("  Dependencies:");
                    for dep in &container.manifest.dependencies {
                        println!("    {}: {}", dep.name, dep.version);
                    }
                }
            } else {
                println!("Container '{}' (v{}) is valid", container.name(), container.version());
            }
            
            0
        }
        Err(e) => {
            eprintln!("❌ Container validation failed: {}", e);
            
            if verbose {
                eprintln!("Error details: {:?}", e);
                
                // Provide helpful suggestions based on error type
                match &e {
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
            
            1
        }
    }
}
