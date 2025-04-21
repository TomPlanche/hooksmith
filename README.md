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

<h2 align="center">
    Hooksmith, a trivial Git hooks management tool.
</h2>

<p align="center">
    <a href="https://crates.io/crates/hooksmith">
        <img alt="Crates.io" src="https://img.shields.io/crates/v/hooksmith.svg">
    </a>
</p>

## Features

- ⚙️ Automate the hooks installation process with `build.rs` files.
- 💻 Run hooks locally without triggering them via Git.

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
