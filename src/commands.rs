use std::{fs, path::Path};

use clap::Subcommand;

use crate::{Config, check_for_git_hooks, get_git_hooks_path};

/// Commands enum for hooksmith CLI.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Install all hooks listed in the config file
    #[command(about = "Install all hooks listed in the config file")]
    Install,

    /// Run a specific hook
    #[command(about = "Run a specific hook")]
    Run {
        /// Name of the hook to run
        hook_name: String,
    },

    /// Uninstall hooks
    #[command(about = "Uninstall hooks")]
    Uninstall {
        /// Optional name of the hook to uninstall. If not provided, all hooks will be uninstalled.
        #[arg(default_value = None)]
        hook_name: Option<String>,
    },
}

/// # `install_hooks`
/// Install all hooks listed in the config file.
///
/// ## Panics
/// * If the `.git/hooks` directory cannot be created
///
/// ## Arguments
/// * `config` - Parsed configuration file
pub fn install_hooks(config: &Config) {
    let git_hooks_path = get_git_hooks_path();

    if !check_for_git_hooks() {
        fs::create_dir_all(git_hooks_path).expect("Failed to create .git/hooks directory");
    }

    for hook_name in config.hooks.keys() {
        install_hook(hook_name);
    }
}

/// # `install_hook`
/// Install a single hook.
///
/// ## Panics
/// * If the `.git/hooks` directory cannot be created
/// * If the hook cannot be installed/given permission
///
/// ## Arguments
/// * `hook_name` - Name of the hook to install
pub fn install_hook(hook_name: &str) {
    let git_hooks_path = get_git_hooks_path();

    if !git_hooks_path.exists() {
        fs::create_dir_all(&git_hooks_path).expect("Failed to create .git/hooks directory");
    }

    let hook_path = format!("{}/{}", git_hooks_path.to_str().unwrap(), hook_name);

    /*
    - >/dev/null 2>&1
        This suppresses both stdout and stderr from the hooksmith -h check
    */
    let hook_content = format!(
        "#!/bin/sh\n
if hooksmith -h >/dev/null 2>&1
then
  exec hooksmith run {hook_name}
else
  cargo install hooksmith
  exec hooksmith run {hook_name}
fi"
    );
    fs::write(&hook_path, hook_content)
        .unwrap_or_else(|_| panic!("Failed to write hook script to {hook_path}"));

    // Linux only
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(&hook_path)
            .expect("Failed to get file permissions")
            .permissions();

        permissions.set_mode(0o755);

        fs::set_permissions(&hook_path, permissions).expect("Failed to set file permissions");
    }
}

/// # `run_hook`
/// Runs a hook by executing its commands.
///
/// ## Panics
/// * If a command cannot be executed
///
/// # Arguments
/// * `config` - A reference to the configuration.
/// * `hook_name` - The name of the hook to run.
pub fn run_hook(config: &Config, hook_name: &str) {
    if let Some(hook) = config.hooks.get(hook_name) {
        for command_str in &hook.commands {
            println!("Running command: {command_str}");

            let status = std::process::Command::new("sh")
                .arg("-c")
                .arg(command_str)
                .status()
                .expect("Failed to execute command");

            if !status.success() {
                let status_code = status.code().unwrap_or(1);

                eprintln!("Command `{command_str}` failed with status code {status_code}",);
                std::process::exit(status_code);
            }
        }
    } else {
        let possible_hooks = config.hooks.keys().collect::<Vec<_>>();

        eprintln!("No commands defined for hook '{hook_name}'");
        eprintln!("Possible hooks: {possible_hooks:?}");

        std::process::exit(1);
    }
}

/// # `uninstall_given_hook`
/// Uninstalls a single hook by removing its file.
///
/// ## Panics
/// * Panics if the command fails to remove the file.
///
/// # Arguments
/// * `config` - A reference to the configuration.
/// * `hook_name` - The name of the hook to run.
pub fn uninstall_given_hook(config: &Config, hook_name: &str) {
    if config.hooks.contains_key(hook_name) {
        println!("Uninstalling hook {hook_name}");

        let hook_path = get_git_hooks_path().join(hook_name);

        if Path::new(&hook_path).exists() {
            fs::remove_file(&hook_path)
                .unwrap_or_else(|_| panic!("Failed to remove hook: {hook_name}"));
        } else {
            println!("No hook found for {hook_name}");
        }
    } else {
        let possible_hooks = config.hooks.keys().collect::<Vec<_>>();

        eprintln!("No file found for hook '{hook_name}'");
        eprintln!("Possible hooks: {possible_hooks:?}");

        std::process::exit(1);
    }
}

/// # `uninstall_hooks`
/// Uninstalls all hooks by removing their files.
///
/// # Arguments
/// * `config` - A reference to the configuration.
pub fn uninstall_hooks(config: &Config) {
    for hook_name in config.hooks.keys() {
        uninstall_given_hook(config, hook_name);

        println!("Removed hook {hook_name}");
    }
}
