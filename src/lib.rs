pub mod error;
pub(crate) mod git_related;
mod hooksmith;
pub(crate) mod my_clap_theme;
pub(crate) mod utils;

pub use error::{HooksmithError, Result};
pub use hooksmith::Hooksmith;

/// Initialize Hooksmith by reading the configuration file and installing hooks.
/// This is meant to be called from a `build.rs` script.
/// To see the CLI usage, run `hooksmith --help` or go to the project's GitHub [repository](https://github.com/TomPlanche/hooksmith).
///
/// # Arguments
/// * `config_path` - Path to the configuration file
///
/// # Errors
/// * If the configuration file cannot be read or parsed
pub fn init(config_path: &std::path::Path) -> Result<()> {
    let hs = Hooksmith::new_from_config(config_path, false, false)?;

    hs.install_hooks()?;

    Ok(())
}
