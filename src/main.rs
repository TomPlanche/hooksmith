use std::path::Path;

use clap::Parser;
use hooksmith::{
    Cli, GIT_ROOT,
    commands::{Command, install_hooks, run_hook, uninstall_given_hook, uninstall_hooks, compare_hooks},
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

    if cli.dry_run {
        println!("🔄 DRY RUN MODE - No commands will be executed\n");
    }

    match cli.command {
        Command::Install => install_hooks(&config, cli.dry_run, cli.verbose),
        Command::Uninstall { hook_name } => {
            if hook_name.is_none() {
                uninstall_hooks(&config, cli.dry_run, cli.verbose);
            } else {
                let hook_name = hook_name.unwrap();

                uninstall_given_hook(&config, &hook_name, cli.dry_run, cli.verbose);
            }
        }
        Command::Run { hook_name } => {
            run_hook(&config, &hook_name, cli.dry_run, cli.verbose);
        }
        Command::Compare => {
            compare_hooks(&config, cli.verbose);
        }
    }
}
