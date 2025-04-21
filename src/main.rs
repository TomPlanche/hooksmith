use std::path::Path;

use clap::Parser;
use hooksmith::{
    Cli, GIT_ROOT,
    commands::{Command, install_hooks, run_hook},
    read_config,
};

fn main() {
    if !Path::new(GIT_ROOT).exists() {
        eprintln!(
            "Error: .git directory not found. Ensure you're at the root of a Git repository."
        );
        std::process::exit(1);
    }

    let cli = Cli::parse();

    let config_path = Path::new(&cli.config_path);

    if !config_path.exists() {
        eprintln!("Error: Config file not found at {}", config_path.display());
        std::process::exit(1);
    }

    let config = read_config(config_path);

    match cli.command {
        Command::Install => install_hooks(&config),
        Command::Run { hook_name } => {
            if cli.verbose {
                println!("Running hook: {}", hook_name);
            }

            run_hook(&config, &hook_name);
        }
    }
}
