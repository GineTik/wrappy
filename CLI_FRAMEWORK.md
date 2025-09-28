# CLI Framework для Wrappy

## Архітектура subsubcommands

### Структура команд
```
wrappy <feature> <action> [options]

Examples:
wrappy container validate --path ./
wrappy container create --name my-app
wrappy flathub install firefox
wrappy flathub search browser
wrappy bindings list
wrappy bindings enable nvim --config
wrappy isolation set --level full
```

## Реалізація фреймворку

### 1. Основна CLI структура

```rust
// src/cli/mod.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "wrappy",
    about = "Container file system abstraction",
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
    /// Flathub integration commands  
    Flathub {
        #[command(subcommand)]
        action: FlathubCommands,
    },
    /// Bindings management commands
    Bindings {
        #[command(subcommand)]
        action: BindingsCommands,
    },
    /// Isolation configuration commands
    Isolation {
        #[command(subcommand)]
        action: IsolationCommands,
    },
}
```

### 2. Feature-specific commands

```rust
// src/features/container/commands.rs
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub enum ContainerCommands {
    /// Validate container structure
    Validate {
        #[arg(short, long)]
        path: Option<PathBuf>,
        #[arg(short, long)]
        verbose: bool,
    },
    /// Create new container
    Create {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        template: Option<String>,
    },
    /// List all containers
    List {
        #[arg(short, long)]
        format: Option<String>,
    },
    /// Remove container
    Remove {
        name: String,
        #[arg(long)]
        force: bool,
    },
    /// Run container
    Run {
        name: String,
        #[arg(short, long)]
        script: Option<String>,
        #[arg(last = true)]
        args: Vec<String>,
    },
}

// src/features/flathub/commands.rs
#[derive(Subcommand)]
pub enum FlathubCommands {
    /// Install app from Flathub
    Install {
        app_id: String,
        #[arg(long)]
        no_bindings: bool,
    },
    /// Search apps on Flathub
    Search {
        query: String,
        #[arg(short, long)]
        limit: Option<usize>,
    },
    /// Update Flathub apps
    Update {
        #[arg(short, long)]
        all: bool,
        app_names: Vec<String>,
    },
}

// src/features/bindings/commands.rs
#[derive(Subcommand)]
pub enum BindingsCommands {
    /// List all bindings
    List {
        #[arg(short, long)]
        container: Option<String>,
    },
    /// Enable bindings for container
    Enable {
        container: String,
        #[arg(long)]
        bin: bool,
        #[arg(long)]
        config: bool,
        #[arg(long)]
        data: bool,
    },
    /// Disable bindings
    Disable {
        container: String,
        #[arg(long)]
        all: bool,
    },
}
```

### 3. Command handlers

```rust
// src/cli/router.rs
use crate::cli::MainCommands;
use crate::features::{
    container::ContainerHandler,
    flathub::FlathubHandler,
    bindings::BindingsHandler,
    isolation::IsolationHandler,
};

pub struct CommandRouter;

impl CommandRouter {
    pub fn execute(command: MainCommands) -> i32 {
        match command {
            MainCommands::Container { action } => {
                ContainerHandler::execute(action)
            }
            MainCommands::Flathub { action } => {
                FlathubHandler::execute(action)
            }
            MainCommands::Bindings { action } => {
                BindingsHandler::execute(action)
            }
            MainCommands::Isolation { action } => {
                IsolationHandler::execute(action)
            }
        }
    }
}
```

### 4. Feature handlers

```rust
// src/features/container/handler.rs
use super::commands::ContainerCommands;
use super::service::ContainerService;

pub struct ContainerHandler;

impl ContainerHandler {
    pub fn execute(command: ContainerCommands) -> i32 {
        match command {
            ContainerCommands::Validate { path, verbose } => {
                Self::handle_validate(path, verbose)
            }
            ContainerCommands::Create { name, template } => {
                Self::handle_create(name, template)
            }
            ContainerCommands::List { format } => {
                Self::handle_list(format)
            }
            ContainerCommands::Remove { name, force } => {
                Self::handle_remove(name, force)
            }
            ContainerCommands::Run { name, script, args } => {
                Self::handle_run(name, script, args)
            }
        }
    }
    
    fn handle_validate(path: Option<PathBuf>, verbose: bool) -> i32 {
        // Існуюча логіка валідації
        match ContainerService::validate_at_path(path) {
            Ok(container) => {
                println!("✅ Container validation successful!");
                if verbose {
                    Self::print_container_details(&container);
                }
                0
            }
            Err(error) => {
                eprintln!("❌ Container validation failed: {}", error);
                1
            }
        }
    }
    
    fn handle_create(name: String, template: Option<String>) -> i32 {
        match ContainerService::create_new(&name, template.as_deref()) {
            Ok(_) => {
                println!("✅ Container '{}' created successfully!", name);
                0
            }
            Err(error) => {
                eprintln!("❌ Failed to create container: {}", error);
                1
            }
        }
    }
    
    fn handle_list(format: Option<String>) -> i32 {
        match ContainerService::list_all() {
            Ok(containers) => {
                match format.as_deref() {
                    Some("json") => Self::print_json(&containers),
                    Some("table") | None => Self::print_table(&containers),
                    _ => {
                        eprintln!("❌ Unknown format. Use 'json' or 'table'");
                        return 1;
                    }
                }
                0
            }
            Err(error) => {
                eprintln!("❌ Failed to list containers: {}", error);
                1
            }
        }
    }
}
```

### 5. Trait для уніфікації handlers

```rust
// src/cli/traits.rs
use clap::Subcommand;

pub trait CommandHandler<T: Subcommand> {
    fn execute(command: T) -> i32;
    fn get_feature_name() -> &'static str;
}

// Реалізація для кожної фічі
impl CommandHandler<ContainerCommands> for ContainerHandler {
    fn execute(command: ContainerCommands) -> i32 {
        // логіка виконання
    }
    
    fn get_feature_name() -> &'static str {
        "container"
    }
}
```

### 6. Auto-completion та help

```rust
// src/cli/completion.rs
use clap::Command;

pub fn generate_completion() {
    let mut cmd = Cli::command();
    
    // Генерація completion для різних shells
    clap_complete::generate(
        clap_complete::Shell::Bash,
        &mut cmd,
        "wrappy",
        &mut std::io::stdout()
    );
}

// Використання:
// wrappy completion bash > /etc/bash_completion.d/wrappy
```

## Структура проекту

```
src/
├── cli/
│   ├── mod.rs              # Основна CLI структура
│   ├── router.rs           # Роутинг команд
│   ├── traits.rs           # Traits для handlers
│   └── completion.rs       # Auto-completion
├── features/
│   ├── container/
│   │   ├── commands.rs     # Container subcommands
│   │   ├── handler.rs      # Container command handler
│   │   └── service.rs      # Business logic
│   ├── flathub/
│   │   ├── commands.rs     # Flathub subcommands
│   │   ├── handler.rs      # Flathub command handler
│   │   └── service.rs      # Flathub logic
│   └── bindings/
│       ├── commands.rs     # Bindings subcommands
│       ├── handler.rs      # Bindings handler
│       └── service.rs      # Bindings logic
└── main.rs                 # Entry point
```

## Переваги фреймворку

1. **Модульність** - кожна фіча має свої команди
2. **Розширюваність** - легко додати нові фічі
3. **Consistency** - однаковий підхід до всіх команд
4. **Auto-completion** - підтримка bash/zsh completion
5. **Help generation** - автоматична генерація help

## Приклади використання

```bash
# Container управління
wrappy container validate
wrappy container create --name my-app
wrappy container list --format json
wrappy container run my-app --script build

# Flathub інтеграція
wrappy flathub search browser
wrappy flathub install org.mozilla.firefox
wrappy flathub update --all

# Bindings управління
wrappy bindings list
wrappy bindings enable nvim --bin --config
wrappy bindings disable nvim --all

# Help системи
wrappy --help
wrappy container --help
wrappy container validate --help
```

## Macro для спрощення

```rust
// src/cli/macros.rs
macro_rules! define_feature_commands {
    ($feature:ident, $commands:ident, $handler:ident) => {
        pub mod $feature {
            use super::*;
            
            pub use commands::$commands;
            pub use handler::$handler;
            
            impl CommandHandler<$commands> for $handler {
                fn execute(command: $commands) -> i32 {
                    $handler::execute(command)
                }
                
                fn get_feature_name() -> &'static str {
                    stringify!($feature)
                }
            }
        }
    };
}

// Використання:
define_feature_commands!(container, ContainerCommands, ContainerHandler);
define_feature_commands!(flathub, FlathubCommands, FlathubHandler);
```

## Advanced features

### Global options

```rust
#[derive(Parser)]
pub struct GlobalOpts {
    /// Enable debug output
    #[arg(short, long, global = true)]
    debug: bool,
    
    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
    
    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}
```

### Plugin система

```rust
// src/cli/plugins.rs
pub trait PluginCommand {
    fn name() -> &'static str;
    fn execute(&self, args: Vec<String>) -> i32;
}

pub struct PluginManager {
    plugins: HashMap<String, Box<dyn PluginCommand>>,
}

impl PluginManager {
    pub fn register_plugin<P: PluginCommand + 'static>(&mut self, plugin: P) {
        self.plugins.insert(P::name().to_string(), Box::new(plugin));
    }
}
```