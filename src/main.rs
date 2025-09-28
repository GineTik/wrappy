use std::process;
use wrappy::cli::{Cli, CommandRouter};
use clap::Parser;

fn main() {
    let cli = Cli::parse();
    let exit_code = CommandRouter::execute(cli.command);
    process::exit(exit_code);
}
