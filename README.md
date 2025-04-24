<pre align="center">
                ,"(                             .
               ////\                           /
              (//////--,,,,,_____            ,"
            _;"""----/////_______;,,        //
__________;"o,-------------......"""""`'-._/(
      ""'==._.__,;;;;"""           ____,.-.==
             "-.:______,...;---""/"   "    \(
                 '-._      `-._("           \\
                     '-._                     '._
</pre>

<h1 align="center">
    Hooksmith, a trivial Git hooks management tool.
</h1>

[![Crates.io](https://img.shields.io/crates/v/hooksmith.svg)](https://crates.io/crates/hooksmith)
[![Docs.rs](https://img.shields.io/docsrs/hooksmith/latest)](https://docs.rs/hooksmith)

## Features

- ⚙️ Automate the hooks installation process with `build.rs` files.
- 💻 Run hooks locally without triggering them via Git.
- ⚙️ Dry-run mode to preview changes without applying them.

## Installation

### With Cargo
You can install it using `cargo`:

```sh
cargo install hooksmith
```

### Build Dependency

You can add it as a build dependency:

```sh
cargo add --build hooksmith
```

Then create a `build.rs` file:

```rust
use std::path::Path;

pub fn main() {
    let config_path = Path::new("hooksmith.yaml");

    hooksmith::init(&config_path);
}
```

## Usage

Create a configuration file named `monk.yaml` in your project root:

```yaml
pre-commit:
  commands:
    - cargo fmt --all -- --check
    - cargo clippy --workspace --release --all-targets --all-features -- --deny warnings

pre-push:
  commands:
    - cargo test

```

### Commands

- `hooksmith compare`: Compare installed hooks with the configuration file.
- `hooksmith install`: Install the hooks from the configuration file.
- `hooksmith run <hook_name>`: Run a hook.
- `hooksmith uninstall [hook_name]`: Uninstall a hook (all if no name is provided).
- `hooksmith validate`: Validate hooks in the configuration file against standard Git hooks.

All commands can be preceded by:

- `--dry-run` flag to preview changes without applying them.
- `--verbose` flag to print more information.
