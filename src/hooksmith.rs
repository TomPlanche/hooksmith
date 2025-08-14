use crate::{
    error::{ConfigError, HookExecutionError, Result, ValidationError},
    git_related::{check_for_git_hooks, get_git_hooks_path},
    my_clap_theme,
    utils::{format_list, print_error, print_success, print_warning},
    HooksmithError,
};

use dialoguer::{Confirm, MultiSelect};
use serde::Deserialize;
use std::{
    fs::{self},
    path::Path,
    process::{Command, ExitStatus},
};

const GIT_HOOKS: [&str; 28] = [
    "applypatch-msg",
    "pre-applypatch",
    "post-applypatch",
    "pre-commit",
    "pre-merge-commit",
    "prepare-commit-msg",
    "commit-msg",
    "post-commit",
    "pre-rebase",
    "post-checkout",
    "post-merge",
    "pre-push",
    "pre-receive",
    "update",
    "proc-receive",
    "post-receive",
    "post-update",
    "reference-transaction",
    "push-to-checkout",
    "pre-auto-gc",
    "post-rewrite",
    "sendemail-validate",
    "fsmonitor-watchman",
    "p4-changelist",
    "p4-prepare-changelist",
    "p4-post-changelist",
    "p4-pre-submit",
    "post-index-change",
];

/// Configuration structure for hooksmith.
#[derive(Deserialize)]
struct Config {
    #[serde(flatten)]
    hooks: std::collections::HashMap<String, Hook>,
}

/// Path-scoped configuration for a hook.
#[derive(Deserialize)]
struct PathScopedConfig {
    commands: Vec<String>,
    #[serde(default)]
    working_directory: Option<String>,
}

/// Hook structure for hooksmith.
#[derive(Deserialize)]
struct Hook {
    #[serde(default)]
    commands: Option<Vec<String>>,
    #[serde(default)]
    paths: Option<std::collections::HashMap<String, PathScopedConfig>>, // path prefix -> config
}

/// Hooksmith structure for managing git hooks.
pub struct Hooksmith {
    config: Config,
    dry_run: bool,
    verbose: bool,
}

impl Hooksmith {
    /// Create a new instance of `Hooksmith` from a configuration file.
    ///
    /// # Arguments
    /// * `config` - Path to the configuration file
    /// * `dry_run` - Whether to run in dry run mode
    /// * `verbose` - Whether to print verbose output
    ///
    /// # Errors
    /// * If the configuration file cannot be read or parsed
    pub fn new_from_config(config: &Path, dry_run: bool, verbose: bool) -> Result<Self> {
        let config = Self::read_config(config)?;

        if dry_run {
            println!("🔄 DRY RUN MODE - No commands will be executed\n");
        }

        Ok(Self {
            config,
            dry_run,
            verbose,
        })
    }

    /// Check for hooks that are in config but not installed.
    /// Iterates through hooks in the config and checks if they are installed.
    /// Updates the `differences_found` flag and prints messages for missing hooks.
    ///
    /// # Arguments
    /// * `git_hooks_path` - Path to the git hooks directory
    /// * `differences_found` - Mutable reference to track if differences were found
    fn check_missing_hooks(&self, git_hooks_path: &Path, differences_found: &mut bool) {
        for hook_name in self.config.hooks.keys() {
            let hook_path = git_hooks_path.join(hook_name);
            if !hook_path.exists() {
                if !*differences_found {
                    println!("\n❌ Differences found:");

                    *differences_found = true;
                }

                println!("  - Hook '{hook_name}' is in config but not installed");
            }
        }
    }

    /// Check for hooks that are installed but not in config.
    /// Scans the git hooks directory and checks if each hook is in the config.
    /// Updates the `differences_found` flag and prints messages for extra hooks.
    ///
    /// # Arguments
    /// * `git_hooks_path` - Path to the git hooks directory
    /// * `differences_found` - Mutable reference to track if differences were found
    ///
    /// # Errors
    /// * If there is an error reading the git hooks directory
    fn check_extra_hooks(&self, git_hooks_path: &Path, differences_found: &mut bool) {
        if let Ok(entries) = fs::read_dir(git_hooks_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if !file_type.is_file() {
                        continue;
                    }

                    let hook_name = entry.file_name().to_string_lossy().to_string();

                    if hook_name.ends_with(".sample") {
                        continue;
                    }

                    if !self.config.hooks.contains_key(&hook_name) {
                        if !*differences_found {
                            println!("\n❌ Differences found:");

                            *differences_found = true;
                        }

                        println!("  - Hook '{hook_name}' is installed but not in config");
                    }
                }
            }
        }
    }

    /// Compare installed hooks with the configuration file.
    ///
    /// # Errors
    /// * If there is an error reading the git hooks directory.
    pub fn compare_hooks(&self) -> Result<()> {
        let git_hooks_path = get_git_hooks_path()?;
        let mut differences_found = false;

        if self.verbose {
            println!("🔍 Comparing installed hooks with configuration file...");
        }

        // Check for hooks in config but not installed
        self.check_missing_hooks(&git_hooks_path, &mut differences_found);

        // Check for installed hooks not in config
        self.check_extra_hooks(&git_hooks_path, &mut differences_found);

        if !differences_found {
            println!("✅ All hooks match the configuration file");
        }

        Ok(())
    }

    /// Creates the git hooks directory if it doesn't exist.
    /// Handles both normal and dry run modes.
    ///
    /// # Arguments
    /// * `git_hooks_path` - Path to the git hooks directory
    ///
    /// # Errors
    /// * If the directory cannot be created
    fn ensure_hooks_directory(&self, git_hooks_path: &Path) -> Result<()> {
        if !git_hooks_path.exists() {
            if self.dry_run {
                println!("🪝 Skipping creation of .git/hooks directory in dry run mode");
            } else {
                if self.verbose {
                    println!("  - Creating .git/hooks directory...");
                }
                fs::create_dir_all(git_hooks_path)?;
            }
        }
        Ok(())
    }

    /// Generates configuration content for a specific hook type
    ///
    /// # Arguments
    /// * `hook` - The name of the hook to generate configuration for
    ///
    /// # Returns
    /// * `String` - The generated configuration content for the hook
    fn generate_hook_config(hook: &str) -> String {
        let mut config = String::new();
        config.push_str(hook);
        config.push_str(":\n");
        config.push_str("  commands:\n");

        // Add hook-specific default commands and comments
        let (echo_msg, examples) = match hook {
            "pre-commit" => (
                "Running pre-commit checks...",
                vec![
                    "# Add your pre-commit commands here",
                    "# Examples:",
                    "# - cargo fmt --all -- --check",
                    "# - cargo clippy -- --deny warnings",
                ],
            ),
            "pre-push" => (
                "Running pre-push checks...",
                vec![
                    "# Add your pre-push commands here",
                    "# Examples:",
                    "# - cargo test",
                    "# - cargo build --release",
                ],
            ),
            "commit-msg" => (
                "Validating commit message...",
                vec![
                    "# Add your commit message validation here",
                    "# Example:",
                    "# - ./scripts/validate-commit-msg.sh $1",
                ],
            ),
            "post-commit" => (
                "Post-commit actions...",
                vec!["# Add your post-commit commands here"],
            ),
            _ => (
                &format!("Running {hook} hook...")[..],
                vec!["# Add your commands here"],
            ),
        };

        config.push_str(&format!("    - echo \"{echo_msg}\"\n")[..]);

        for example in examples {
            config.push_str(&format!("    {example}\n")[..]);
        }

        config.push('\n');

        config
    }

    /// Initialize hooksmith configuration interactively.
    ///
    /// # Arguments
    /// * `config_path` - Path where the configuration file will be created
    /// * `dry_run` - Whether to run in dry run mode
    /// * `verbose` - Whether to print verbose output
    ///
    /// # Errors
    /// * If the user cancels the selection
    /// * If there's an error writing the configuration file
    pub fn init_interactive(config_path: &Path, dry_run: bool, verbose: bool) -> Result<()> {
        if dry_run {
            println!("🔄 DRY RUN MODE - No files will be created\n");
        }

        if verbose {
            println!("🚀 Initializing hooksmith configuration...");
        }

        // Check if config file already exists
        if config_path.exists() && !dry_run {
            let overwrite = Confirm::with_theme(&my_clap_theme::ColorfulTheme::default())
                .with_prompt(format!(
                    "Configuration file '{}' already exists. Overwrite?",
                    config_path.display()
                ))
                .default(false)
                .interact()
                .map_err(|e| HookExecutionError::HookNotFound(e.to_string()))?;

            if !overwrite {
                println!("❌ Initialization cancelled");
                return Ok(());
            }
        }

        // Get all available Git hooks
        let hook_options: Vec<String> = GIT_HOOKS.iter().map(|&s| s.to_string()).collect();

        // Interactive hook selection
        let selections = MultiSelect::with_theme(&my_clap_theme::ColorfulTheme::default())
            .with_prompt("Select hooks to configure (Space to select, Enter to confirm)")
            .items(&hook_options)
            .interact()
            .map_err(|e| HookExecutionError::HookNotFound(e.to_string()))?;

        if selections.is_empty() {
            println!("❌ No hooks selected. Configuration file not created.");
            return Ok(());
        }

        let selected_hooks: Vec<String> = selections
            .into_iter()
            .map(|i| hook_options[i].clone())
            .collect();

        if verbose {
            println!("📝 Selected hooks: {}", selected_hooks.join(", "));
        }

        // Create configuration content
        let config_content: String = selected_hooks
            .iter()
            .map(|hook| Self::generate_hook_config(hook))
            .collect();

        // Write configuration file
        if dry_run {
            println!(
                "🔍 Would create configuration file '{}' with content:",
                config_path.display()
            );
            println!("{config_content}");
        } else {
            fs::write(config_path, config_content)?;
            println!(
                "✅ Configuration file '{}' created successfully!",
                config_path.display()
            );
            println!("📝 You can now edit the file to customize your hook commands.");
            println!("🚀 Run 'hooksmith install' to install the configured hooks.");
        }

        Ok(())
    }

    /// Generates the hook script content.
    /// Creates a shell script that checks for hooksmith and runs the specified hook.
    ///
    /// # Arguments
    /// * `hook_name` - Name of the hook to create content for
    fn generate_hook_content(hook_name: &str) -> String {
        format!(
            "#!/bin/sh\n
    if hooksmith -h >/dev/null 2>&1
    then
      exec hooksmith run {hook_name}
    else
      cargo install hooksmith
      exec hooksmith run {hook_name}
    fi"
        )
    }

    /// Writes the hook file and sets appropriate permissions.
    /// Handles both normal and dry run modes.
    ///
    /// # Arguments
    /// * `hook_path` - Path where the hook file should be written
    /// * `hook_name` - Name of the hook being installed
    /// * `content` - Content to write to the hook file
    ///
    /// # Errors
    /// * If the file cannot be written
    /// * If permissions cannot be set
    fn write_hook_file(&self, hook_path: &Path, hook_name: &str, content: &str) -> Result<()> {
        if self.dry_run {
            println!("🪝 Skipping installation of {hook_name} hook in dry run mode");
            return Ok(());
        }

        fs::write(hook_path, content)?;

        if self.verbose {
            println!("  - Installing {hook_name} file...");
        }

        // Linux only
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut permissions = fs::metadata(hook_path)?.permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(hook_path, permissions)?;

            if self.verbose {
                println!("  - Setting file permissions...");
            }
        }

        Ok(())
    }

    /// Install a single, given hook.
    ///
    /// # Arguments
    /// * `hook_name` - Name of the hook to install
    ///
    /// # Errors
    /// * If the `.git/hooks` directory cannot be created
    /// * If the hook cannot be installed/given permission
    pub fn install_hook(&self, hook_name: &str) -> Result<()> {
        if self.verbose && !self.dry_run {
            println!("🪝 Installing {hook_name} hook...");
        }

        let git_hooks_path = get_git_hooks_path()?;
        self.ensure_hooks_directory(&git_hooks_path)?;

        let hook_path = git_hooks_path.join(hook_name);
        let hook_content = Self::generate_hook_content(hook_name);
        self.write_hook_file(&hook_path, hook_name, &hook_content)?;

        if self.verbose {
            println!("  ✅ Installed {hook_name} file");
        }

        Ok(())
    }

    /// Install all hooks.
    ///
    /// # Errors
    /// * If the `.git/hooks` directory cannot be created
    ///
    /// # Arguments
    /// * `config` - Parsed configuration file
    pub fn install_hooks(&self) -> Result<()> {
        self.validate_hooks()?;

        let git_hooks_path = get_git_hooks_path()?;

        if !check_for_git_hooks() {
            fs::create_dir_all(&git_hooks_path)?;
        }

        if self.verbose {
            println!("🪝 Installing hooks...");
        }

        for hook_name in self.config.hooks.keys() {
            self.install_hook(hook_name)?;
        }

        Ok(())
    }

    /// Executes a single command and handles its output
    ///
    /// # Arguments
    /// * `command_str` - The command to execute
    /// * `hook_name` - The name of the hook being executed
    fn execute_single_command(
        &self,
        command_str: &str,
        hook_name: &str,
        working_directory: Option<&Path>,
    ) {
        if self.verbose && !self.dry_run {
            println!("  - Running command: {command_str}");
        }

        match self.execute_command(command_str, working_directory) {
            Ok(status) if status.success() => {
                if self.verbose && !self.dry_run {
                    println!("\n  ✅ Command completed successfully");
                }
            }
            Ok(status) => {
                let code = status.code().unwrap_or(1);
                print_error(
                    "Command failed",
                    &format!("Hook '{hook_name}' command failed with status code {code}"),
                    "Please check your command and try again.",
                );

                std::process::exit(code);
            }
            Err(e) => {
                print_error(
                    "Failed to execute command",
                    &format!("Error: {e}"),
                    "Please ensure the command exists and is executable.",
                );

                std::process::exit(1);
            }
        }
    }

    /// Get a list of available hooks from the configuration.
    #[must_use]
    pub fn get_available_hooks(&self) -> Vec<String> {
        self.config.hooks.keys().cloned().collect()
    }

    /// Handle the "hook not found error"
    ///
    /// # Arguments
    /// * `hook_name` - The name of the hook being executed
    ///
    /// # Errors
    /// * If the hook is not found in the configuration.
    fn handle_hook_not_found(&self, hook_name: &str) -> Result<()> {
        let formatted_hooks = format_list(&self.config.hooks.keys().collect::<Vec<_>>());

        print_error(
            "Hook not found",
            &format!("No commands defined for hook '{hook_name}'"),
            &format!(
                "Available hooks:\n{formatted_hooks}\n\nPlease check your configuration file."
            ),
        );

        Err(HookExecutionError::HookNotFound(hook_name.to_string()).into())
    }

    /// Runs multiple hooks by executing their commands.
    ///
    /// # Arguments
    /// * `hook_names` - Vector of hook names to run
    ///
    /// # Errors
    /// * If a command cannot be executed
    /// * If any hook is not found in the configuration
    pub fn run_hooks(&self, hook_names: &[String]) -> Result<()> {
        for hook_name in hook_names {
            self.run_hook_internal(hook_name)?;
        }
        Ok(())
    }

    /// Internal method to run a single hook
    ///
    /// # Arguments
    /// * `hook_name` - Name of the hook to run
    ///
    /// # Errors
    /// * If a command cannot be executed
    /// * If the hook is not found in the configuration
    fn run_hook_internal(&self, hook_name: &str) -> Result<()> {
        let Some(hook) = self.config.hooks.get(hook_name) else {
            return self.handle_hook_not_found(hook_name);
        };

        if self.verbose && !self.dry_run {
            println!("📋 Running Hook: {hook_name}");
        }

        // New execution model supports global commands and optional path-scoped commands
        let mut executed_commands_count: usize = 0;

        // If there are path-based blocks, selectively execute those that match changed files
        if let Some(paths_map) = &hook.paths {
            if let Some(changed_files) = Self::detect_changed_files(hook_name) {
                let changed_files: Vec<String> = changed_files;

                // For each configured path prefix, run commands if any changed file matches the prefix
                for (path_prefix, path_cfg) in paths_map {
                    let has_match = changed_files.iter().any(|f| f.starts_with(path_prefix));
                    if !has_match {
                        continue;
                    }

                    for (idx, command_str) in path_cfg.commands.iter().enumerate() {
                        if self.dry_run {
                            handle_dry_run_with_dir(
                                command_str,
                                idx,
                                path_cfg.commands.len(),
                                path_cfg.working_directory.as_deref(),
                            );
                            executed_commands_count += 1;
                            continue;
                        }

                        let working_directory =
                            path_cfg.working_directory.as_deref().map(Path::new);

                        self.execute_single_command(command_str, hook_name, working_directory);
                        executed_commands_count += 1;
                    }
                }
            }
        }

        // Always run global commands if present
        if let Some(global_cmds) = &hook.commands {
            for (idx, command_str) in global_cmds.iter().enumerate() {
                if self.dry_run {
                    handle_dry_run(command_str, idx, global_cmds.len());
                    executed_commands_count += 1;
                    continue;
                }

                self.execute_single_command(command_str, hook_name, None);
                executed_commands_count += 1;
            }
        }

        if self.dry_run {
            println!(
                "🏁 Dry run completed. {executed_commands_count} command(s) would be executed",
            );
        }

        Ok(())
    }

    /// Runs hooks either interactively or from provided names.
    ///
    /// # Arguments
    /// * `hook_names` - Optional vector of hook names to run. If None, and interactive is true, will prompt for selection.
    /// * `interactive` - Whether to use interactive selection when `hook_names` is None.
    ///
    /// # Errors
    /// * If a command cannot be executed
    /// * If hook selection fails
    /// * If any hook is not found in the configuration
    pub fn run_hook(&self, hook_names: Option<&[String]>, interactive: bool) -> Result<()> {
        if interactive {
            let selected_hooks = self.select_hooks_interactively()?;
            self.run_hooks(&selected_hooks)
        } else if let Some(names) = hook_names {
            if names.is_empty() {
                return Err(
                    HookExecutionError::HookNotFound("No hooks specified".to_string()).into(),
                );
            }

            // remove duplicate hooks
            let unique_hooks = names
                .iter()
                .cloned()
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();

            self.run_hooks(&unique_hooks)
        } else {
            Err(HookExecutionError::HookNotFound(
                "No hook specified and interactive mode is disabled".to_string(),
            )
            .into())
        }
    }

    /// Uninstalls a single, given hook by removing its file.
    ///
    /// # Arguments
    /// * `hook_name` - The name of the hook to run.
    ///
    /// # Errors
    /// * Errors if the command fails to remove the file.
    pub fn uninstall_given_hook(&self, hook_name: &str) -> Result<()> {
        if self.config.hooks.contains_key(hook_name) {
            if self.verbose && !self.dry_run {
                println!("🗑️ Uninstalling hook: {hook_name}");
            }

            let git_hooks_path = get_git_hooks_path()?;
            let hook_path = git_hooks_path.join(hook_name);

            if hook_path.exists() {
                if self.dry_run {
                    println!(
                        "  🚧 Dry run: Would remove hook file: {}",
                        hook_path.display()
                    );
                } else {
                    fs::remove_file(&hook_path)?;
                }
            } else {
                println!("  ⚠️ No hook file found for {hook_name}");
            }
        } else {
            let possible_hooks = self.config.hooks.keys().collect::<Vec<_>>();
            eprintln!("No file found for hook '{hook_name}'");
            eprintln!("Possible hooks: {possible_hooks:?}");

            return Err(ValidationError::InvalidHookName(hook_name.to_string()).into());
        }

        Ok(())
    }

    /// Uninstalls all hooks by removing their files.
    ///
    /// # Errors
    /// * If there is an error uninstalling a hook.
    pub fn uninstall_hooks(&self) -> Result<()> {
        if self.verbose && !self.dry_run {
            println!("🗑️ Uninstalling all hooks");
        }

        for hook_name in self.config.hooks.keys() {
            self.uninstall_given_hook(hook_name)?;
        }

        if self.verbose && !self.dry_run {
            println!(
                "🏁 Uninstallation completed: {} hooks removed",
                self.config.hooks.len()
            );
        }

        Ok(())
    }

    /// Validate that hooks in the configuration file are standard Git hooks.
    ///
    /// # Errors
    /// None, I just return Ok(()) to aggregate all calls in a `match` statement in the main function.
    pub fn validate_hooks(&self) -> Result<()> {
        if self.verbose {
            println!("🔍 Validating hooks in configuration file...");
        }

        let mut invalid_hooks = Vec::new();
        let mut valid_hooks = 0;

        for hook_name in self.config.hooks.keys() {
            if GIT_HOOKS.contains(&hook_name.as_str()) {
                valid_hooks += 1;
                if self.verbose {
                    println!("  ✅ Hook '{hook_name}' is valid");
                }
            } else {
                invalid_hooks.push(hook_name.clone());
            }
        }

        if invalid_hooks.is_empty() {
            if self.verbose {
                print_success(
                    "All hooks are valid",
                    &format!("Found {valid_hooks} valid Git hooks in your configuration."),
                );
            }
        } else {
            print_warning(
                "Invalid hooks detected",
                &format!(
                    "The following hooks are not recognized by Git:\n{}\n\nPlease use only valid Git hook names in your configuration.",
                    format_list(&invalid_hooks)
                ),
            );
        }

        Ok(())
    }

    /// Validate hooks configuration before installation.
    ///
    /// # Errors
    /// * If any invalid hook names are found.
    pub fn validate_hooks_for_install(&self) -> Result<()> {
        if self.verbose {
            println!("🔍 Validating hooks before installation...");
        }

        let mut invalid_hooks = Vec::new();
        for hook_name in self.config.hooks.keys() {
            if !GIT_HOOKS.contains(&hook_name.as_str()) {
                invalid_hooks.push(hook_name.clone());
            }
        }

        if !invalid_hooks.is_empty() {
            let error_message = format!(
                "Invalid hook names detected\n\nThe following hooks are not recognized by Git:\n{}\n\nPlease check your configuration file and use only valid Git hook names.",
                format_list(&invalid_hooks)
            );

            return Err(ValidationError::InvalidHookName(error_message).into());
        }

        Ok(())
    }

    /// Executes a command.
    ///
    /// # Arguments
    /// * `command` - The command to execute.
    ///
    /// # Errors
    /// * If a command cannot be executed
    fn execute_command(
        &self,
        command: &str,
        working_directory: Option<&Path>,
    ) -> Result<ExitStatus> {
        if self.dry_run {
            println!("🔍 Would execute: {command}");

            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;

                Ok(ExitStatusExt::from_raw(0))
            }
            #[cfg(windows)]
            {
                use std::os::windows::process::ExitStatusExt;

                Ok(ExitStatusExt::from_raw(0))
            }
        } else {
            let mut cmd = Command::new("sh");
            cmd.arg("-c").arg(command);
            if let Some(dir) = working_directory {
                cmd.current_dir(dir);
            }
            Ok(cmd.status()?)
        }
    }

    /// Read the configuration file and parse it into a Config struct.
    ///
    /// # Arguments
    /// * `config_path` - Path to the configuration file
    ///
    /// # Errors
    /// * If the configuration file cannot be read or parsed
    ///
    /// # Returns
    /// * `Config` - Parsed configuration file
    fn read_config(config_path: &Path) -> Result<Config> {
        let config_string = fs::read_to_string(config_path)?;

        match serde_yaml::from_str(&config_string) {
            Ok(config) => Ok(config),
            Err(err) => Err(HooksmithError::Config(ConfigError::Parse(err))),
        }
    }

    /// Select hooks interactively using `dialoguer`.
    ///
    /// # Errors
    /// * If the user cancels the selection, or an error occurs during selection
    /// * If the selection is empty
    ///
    /// # Returns
    /// * `Vec<String>` - Selected hooks
    fn select_hooks_interactively(&self) -> Result<Vec<String>> {
        let hooks = self.get_available_hooks();

        if hooks.is_empty() {
            return Err(HookExecutionError::HookNotFound(
                "No hooks available in configuration".to_string(),
            )
            .into());
        }

        let selections = MultiSelect::with_theme(&my_clap_theme::ColorfulTheme::default())
            .with_prompt("Select hooks to run (Space to select, Enter to confirm)")
            .items(&hooks)
            .interact()
            .map_err(|e| HookExecutionError::HookNotFound(e.to_string()))?;

        if selections.is_empty() {
            return Err(HookExecutionError::HookNotFound("No hooks selected".to_string()).into());
        }

        Ok(selections.into_iter().map(|i| hooks[i].clone()).collect())
    }
}

/// Handles the dry run output for a command
fn handle_dry_run(command_str: &str, idx: usize, total_commands: usize) {
    let current_dir = std::env::current_dir();

    println!("Step {} of {}:", idx + 1, total_commands);
    println!("  Command: {command_str}");

    if let Ok(dir) = current_dir {
        println!("  Working directory: {}", dir.display());
    }

    println!();
}

/// Handles dry run output for a command with an explicit working directory
fn handle_dry_run_with_dir(
    command_str: &str,
    idx: usize,
    total_commands: usize,
    working_directory: Option<&str>,
) {
    println!("Step {} of {}:", idx + 1, total_commands);
    println!("  Command: {command_str}");

    if let Some(dir) = working_directory {
        println!("  Working directory (override): {dir}");
    } else if let Ok(dir) = std::env::current_dir() {
        println!("  Working directory: {}", dir.display());
    }

    println!();
}

impl Hooksmith {
    /// Detect changed files for a given hook when possible.
    ///
    /// Behavior by hook name:
    /// - `pre-commit`: Returns the list of staged files using `git diff --name-only --cached`.
    /// - `pre-push`: Attempts to diff against the configured upstream with `@{u}..HEAD`.
    ///   If no upstream is configured or that diff fails, falls back to `HEAD~1..HEAD`.
    ///
    /// # Arguments
    /// * `hook_name` - The hook to compute changed files for.
    ///
    /// # Returns
    /// * `Some(Vec<String>)` when detection succeeds (the vector may be empty if no files changed).
    /// * `None` if the hook is not supported or if detection fails (e.g., not a Git repo, no upstream, or the git command fails).
    ///
    /// # Notes
    /// This helper is best-effort and never returns an error. Callers should treat `None`
    /// as "path-scoped execution not applicable" and continue with global commands.
    fn detect_changed_files(hook_name: &str) -> Option<Vec<String>> {
        match hook_name {
            "pre-commit" => Self::git_diff_name_only(&["--cached"]).ok(),
            "pre-push" => {
                // Try to diff against upstream; fall back to last commit range
                if let Ok(files) = Self::git_diff_upstream_range() {
                    Some(files)
                } else {
                    Self::git_diff_name_only(&["HEAD~1..HEAD"]).ok()
                }
            }
            _ => None,
        }
    }

    /// Compute the list of files changed relative to the configured upstream branch.
    ///
    /// Attempts to diff `@{u}..HEAD` if an upstream is configured. If no upstream is
    /// configured, returns an error so callers can fall back to an alternative range.
    ///
    /// # Errors
    /// * If no upstream is configured or if running the underlying `git` command fails.
    fn git_diff_upstream_range() -> Result<Vec<String>> {
        // Check if upstream exists; if so, diff against it
        let upstream_check = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"])
            .output();

        if let Ok(output) = upstream_check {
            if output.status.success() {
                return Self::git_diff_name_only(&["@{u}..HEAD"]);
            }
        }

        Err(HookExecutionError::HookNotFound("No upstream configured".to_string()).into())
    }

    /// Run `git diff --name-only` with the provided arguments and return changed file paths.
    ///
    /// # Arguments
    /// * `args` - Additional arguments or revision ranges to pass to `git diff`.
    ///
    /// # Returns
    /// A vector of path strings for files reported by `git diff --name-only`.
    ///
    /// # Errors
    /// * If the underlying `git diff` command fails.
    fn git_diff_name_only(args: &[&str]) -> Result<Vec<String>> {
        let mut cmd = Command::new("git");
        cmd.arg("diff").arg("--name-only");

        for a in args {
            cmd.arg(a);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            return Err(HookExecutionError::HookNotFound(
                "Failed to compute changed files".to_string(),
            )
            .into());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let files = stdout
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>();

        Ok(files)
    }
}
