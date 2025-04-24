# Basic command options
complete -c hooksmith -s h -l help -d "Show help"
complete -c hooksmith -s v -l verbose -d "Enable verbose output"
complete -c hooksmith -s c -l config-path -d "Path to the hooksmith.yaml file"
complete -c hooksmith -l dry-run -d "Perform a dry run without executing commands"

# Subcommands
complete -c hooksmith -n "__fish_use_subcommand" -a "compare" -d "Compare installed hooks with configuration file"
complete -c hooksmith -n "__fish_use_subcommand" -a "install" -d "Install all hooks listed in the config file"
complete -c hooksmith -n "__fish_use_subcommand" -a "run" -d "Run a specific hook"
complete -c hooksmith -n "__fish_use_subcommand" -a "uninstall" -d "Uninstall hooks (all or specific)"
complete -c hooksmith -n "__fish_use_subcommand" -a "validate" -d "Validate the configuration file"
