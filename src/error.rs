use thiserror::Error;

/// The main error type for Hooksmith operations.
#[derive(Error, Debug)]
pub enum HooksmithError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Git error: {0}")]
    Git(#[from] GitError),

    #[error("Hook execution error: {0}")]
    HookExecution(#[from] HookExecutionError),

    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Errors related to configuration file operations.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse config file: {0}")]
    Parse(#[from] serde_yaml::Error),

    #[error("Config file not found at: {0}")]
    NotFound(String),
}

/// Errors related to Git operations.
#[derive(Error, Debug)]
pub enum GitError {
    #[error("Failed to execute git command: {0}")]
    Command(#[from] std::io::Error),

    #[error("Git hooks directory not found")]
    HooksDirNotFound,

    #[error("Not a git repository")]
    NotGitRepo,
}

/// Errors related to hook execution.
#[derive(Error, Debug)]
pub enum HookExecutionError {
    #[error("Failed to execute command: {0}")]
    Command(#[from] std::io::Error),

    #[error("Command failed with status code: {0}")]
    CommandFailed(i32),

    #[error("Hook not found: {0}")]
    HookNotFound(String),

    #[error("Invalid regex pattern: {0}")]
    InvalidRegex(String),
}

/// Errors related to validation operations.
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid hook name: {0}")]
    InvalidHookName(String),

    #[error("Invalid command: {0}")]
    InvalidCommand(String),
}

/// Type alias for Result using `HooksmithError`
pub type Result<T> = std::result::Result<T, HooksmithError>;
