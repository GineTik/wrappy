use std::process;
use wrappy::ContainerCommands;

use clap::{Parser, Subcommand};
use std::env;

#[derive(Parser)]
#[command(
    name = "wrappy",
    about = "Container file system abstraction - manage isolated application environments",
    version = env!("CARGO_PKG_VERSION")
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: ContainerCommands,
}

fn main() {
    let cli = Cli::parse();
    let exit_code = ContainerHandler::execute_command(cli.command)
    process::exit(exit_code);
}
