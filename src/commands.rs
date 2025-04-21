use std::{fs, path::Path};

use clap::Subcommand;

use crate::{Config, GIT_HOOKS, GIT_ROOT, check_for_git_hooks};

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
}

/// # `install_hooks`
/// Install all hooks listed in the config file.
///
/// ## Arguments
/// * `config` - Parsed configuration file
pub fn install_hooks(config: &Config) {
    let git_root = Path::new(GIT_ROOT);
    let git_hooks = git_root.join(GIT_HOOKS);

    if !check_for_git_hooks() {
        fs::create_dir_all(git_hooks).expect("Failed to create .git/hooks directory");
    }

    for hook_name in config.hooks.keys() {
        install_hook(hook_name);
    }
}

/// # `install_hook`
/// Install a single hook.
///
/// ## Arguments
/// * `hook_name` - Name of the hook to install
pub fn install_hook(hook_name: &str) {
    let git_root = Path::new(GIT_ROOT);
    let git_hooks = git_root.join(GIT_HOOKS);

    if !git_hooks.exists() {
        fs::create_dir_all(&git_hooks).expect("Failed to create .git/hooks directory");
    }

    let hook_path = format!("{}/{}", git_hooks.to_str().unwrap(), hook_name);

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
        eprintln!("Possible hooks: {:?}", possible_hooks);

        std::process::exit(1);
    }
}
