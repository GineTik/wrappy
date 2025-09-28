mod router;

use clap::{Parser, Subcommand};
use std::env;

use crate::features::container::ContainerCommands;
use crate::features::bindings::BindingsCommands;
pub use router::CommandRouter;

#[derive(Parser)]
#[command(
    name = "wrappy",
    about = "Container file system abstraction - manage isolated application environments",
    version = env!("CARGO_PKG_VERSION")
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: MainCommands,
}

#[derive(Subcommand)]
pub enum MainCommands {
    /// Container management commands
    Container {
        #[command(subcommand)]
        action: ContainerCommands,
    },
    /// Flathub integration commands (coming soon)
    Flathub {
        #[command(subcommand)]
        action: FlathubCommands,
    },
    /// Bindings management commands
    Bindings {
        #[command(subcommand)]
        action: BindingsCommands,
    },
}

// Placeholder для майбутніх команд
#[derive(Subcommand)]
pub enum FlathubCommands {
    /// Install app from Flathub
    Install {
        app_id: String,
    },
    /// Search apps on Flathub
    Search {
        query: String,
    },
}