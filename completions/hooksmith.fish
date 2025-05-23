# Helper function to get available hooks
function __hooksmith_available_hooks
    set -l fish_tokens (commandline -op)

    hooksmith list "$fish_tokens"
end


# Disable file completions for all hooksmith commands
complete -c hooksmith -f

# Basic command options (global flags)
complete -c hooksmith -s h -l help -d "Show help"
complete -c hooksmith -s v -l verbose -d "Enable verbose output"
complete -c hooksmith -s c -l config-path -r -d "Path to the hooksmith.yaml file"
complete -c hooksmith -l dry-run -d "Perform a dry run without executing commands"


# Subcommands
complete -c hooksmith -n "__fish_use_subcommand" -a "install" -d "Install all hooks listed in the config file"
complete -c hooksmith -n "__fish_use_subcommand" -a "uninstall" -d "Uninstall hooks (all or specific)"
complete -c hooksmith -n "__fish_use_subcommand" -a "run" -d "Run specific hooks"
complete -c hooksmith -n "__fish_use_subcommand" -a "compare" -d "Compare installed hooks with configuration file"
complete -c hooksmith -n "__fish_use_subcommand" -a "validate" -d "Validate hooks in configuration file"
complete -c hooksmith -n "__fish_use_subcommand" -a "list" -d "List available hooks from configuration"

# Run command options and hook name completions
complete -c hooksmith -n "__fish_seen_subcommand_from run" -s i -l interactive -d "Use interactive hook selection"
complete -c hooksmith -n "__fish_seen_subcommand_from run; and not __fish_contains_opt -s i interactive" -a "(__hooksmith_available_hooks)" -d "Git hook to run"

# Uninstall command hook name completions
complete -c hooksmith -n "__fish_seen_subcommand_from uninstall" -a "(__hooksmith_available_hooks)" -d "Git hook to uninstall"
