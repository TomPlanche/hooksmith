# 🪝 Hooksmith

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

<h1 align="center">Git Hook Management Made Simple</h1>

<p align="center">
  <a href="https://crates.io/crates/hooksmith"><img src="https://img.shields.io/crates/v/hooksmith.svg" alt="Crates.io Version"></a>
  <a href="https://docs.rs/hooksmith"><img src="https://img.shields.io/docsrs/hooksmith/latest" alt="Documentation"></a>
  <a href="https://sonarcloud.io/summary/new_code?id=TomPlanche_hooksmith"><img src="https://sonarcloud.io/api/project_badges/measure?project=TomPlanche_hooksmith&metric=alert_status" alt="SonarCloud Status"></a>
  <a href="https://sonarcloud.io/summary/new_code?id=TomPlanche_hooksmith"><img src="https://sonarcloud.io/api/project_badges/measure?project=TomPlanche_hooksmith&metric=sqale_rating" alt="SonarCloud SQALE Rating"></a>
  <a href="https://sonarcloud.io/summary/new_code?id=TomPlanche_hooksmith"><img src="https://sonarcloud.io/api/project_badges/measure?project=TomPlanche_hooksmith&metric=security_rating" alt="SonarCloud Security Rating"></a>
  <a href="https://github.com/TomPlanche/hooksmith/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/hooksmith" alt="License"></a>
  <a href="https://github.com/TomPlanche/hooksmith/actions/workflows/rust.yaml"><img src="https://github.com/TomPlanche/hooksmith/actions/workflows/rust.yaml/badge.svg" alt="Build Status"></a>
</p>

**Hooksmith** is a lightweight, easy-to-use tool that simplifies Git hook management. Define your hooks in a simple YAML file and let Hooksmith handle the rest.

## 📋 Table of Contents

- [✨ Features](#-features)
- [⚡ Why Hooksmith?](#-why-hooksmith)
- [🔧 Installation](#-installation)
- [🚀 Quick Start](#-quick-start)
- [📖 Usage](#-usage)
- [📚 Command Reference](#-command-reference)
- [🤝 Contributing](#-contributing)
- [📄 License](#-license)

## ✨ Features

- **⚙️ Automatic Installation** - Set up hooks through your build scripts with `build.rs`
- **🧪 Local Testing** - Run hooks manually without triggering Git events
- **🔍 Dry Run Mode** - Preview what would happen without making changes
- **✅ Hook Validation** - Ensure your hooks comply with Git standards
- **📝 Simple Configuration** - Define all your hooks in a clean YAML format
- **🎨 Beautiful CLI** - Enjoy a polished terminal interface with clear output
- **🐟 Shell Completion** - Built-in Fish shell completions for improved productivity
- **🔄 Version Control** - Easily track hook changes with your repository
- **🚦 Error Handling** - Robust error handling with clear, actionable messages

## ⚡ Why Hooksmith?

- **Minimal Dependencies** - Lightweight with only essential dependencies
- **Rust Powered** - Fast, reliable, and type-safe
- **Team Friendly** - Version control your hook configurations
- **Seamless Integration** - Works naturally with your Git workflow
- **Low Learning Curve** - Simple commands and clear documentation

## 🔧 Installation

### Using Cargo

```bash
cargo install hooksmith
```

### As a Build Dependency

Add to your `Cargo.toml`:

```toml
[build-dependencies]
hooksmith = "1.10.0"
```

Create a `build.rs` file:

```rust
use std::path::Path;

fn main() {
    let config_path = Path::new("hooksmith.yaml");
    hooksmith::init(&config_path);
}
```

> 💡 **Note**: Hooksmith includes shell completions for Fish. After installation, they become available automatically.

### Dependencies

Hooksmith is built with minimal but powerful dependencies:
- `clap`: For robust command-line argument parsing
- `console` & `dialoguer`: For beautiful terminal interfaces
- `serde` & `serde_yaml`: For YAML configuration handling
- `thiserror`: For ergonomic error handling

## 🚀 Quick Start

1. Create a `hooksmith.yaml` file in your project root:

```yaml
pre-commit:
  commands:
    - cargo fmt --all -- --check
    - cargo clippy -- --deny warnings

pre-push:
  commands:
    - cargo test
```

2. Install the hooks:

```bash
hooksmith install
```

That's it! Your Git hooks are now ready to use.

## 📖 Usage

### Configuration File

Hooksmith uses a YAML configuration file (default: `hooksmith.yaml`) to define your hooks:

```yaml
# Format and lint code before committing
pre-commit:
  commands:
    - cargo fmt --all -- --check
    - cargo clippy --workspace --all-features -- --deny warnings

# Run tests before pushing
pre-push:
  commands:
    - cargo test --all-features
    - cargo build --verbose

# Validate commit messages
commit-msg:
  commands:
    # Use custom script to validate commit messages
    - ./scripts/verify-commit-message.sh $1
```

### Common Commands

```bash
# Install all hooks defined in configuration
hooksmith install

# Run a specific hook manually
hooksmith run pre-commit

# Uninstall all hooks or a specific one
hooksmith uninstall
hooksmith uninstall pre-commit

# Compare installed hooks with configuration
hooksmith compare

# Validate hook configuration against Git standards
hooksmith validate
```

Add `--dry-run` to any command to preview changes without applying them:

```bash
hooksmith install --dry-run
```

## 📚 Command Reference

| Command | Description |
|---------|-------------|
| `install` | Install all hooks from configuration file |
| `run <hook>` | Run a specific hook manually |
| `uninstall [hook]` | Uninstall all hooks or a specific one |
| `compare` | Compare installed hooks with configuration |
| `validate` | Validate hook configuration against Git standards |

### Global Options

| Option | Description |
|--------|-------------|
| `--config-path <PATH>` | Specify a custom configuration file path |
| `--dry-run` | Preview changes without applying them |
| `--verbose` | Show detailed output during execution |
| `--help` | Display help information |

## 🤝 Contributing

Contributions are welcome! Feel free to:

- Report bugs and suggest features
- Submit pull requests
- Improve documentation
- Share your use cases and feedback

## 📄 License

This project is dual-licensed under either:

- [Apache License 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.
