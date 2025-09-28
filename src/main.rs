use std::process;
use wrappy::ContainerCommands;

fn main() {
    let exit_code = ContainerCommands::run();
    process::exit(exit_code);
}
