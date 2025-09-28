use crate::cli::MainCommands;
use crate::features::container::ContainerHandler;
use crate::features::bindings::BindingsHandler;

pub struct CommandRouter;

impl CommandRouter {
    pub fn execute(command: MainCommands) -> i32 {
        match command {
            MainCommands::Container { action } => {
                ContainerHandler::execute_command(action)
            }
            MainCommands::Flathub { action } => {
                Self::handle_flathub_placeholder(action)
            }
            MainCommands::Bindings { action } => {
                BindingsHandler::execute_command(action)
            }
        }
    }

    fn handle_flathub_placeholder(action: crate::cli::FlathubCommands) -> i32 {
        match action {
            crate::cli::FlathubCommands::Install { app_id } => {
                println!("🚧 Flathub integration coming soon!");
                println!("Would install: {}", app_id);
                0
            }
            crate::cli::FlathubCommands::Search { query } => {
                println!("🚧 Flathub integration coming soon!");
                println!("Would search for: {}", query);
                0
            }
        }
    }

}