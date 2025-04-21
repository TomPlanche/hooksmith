<h2 align="center">
    Hooksmith, a simple Git hooks management tool.
</h2>

## Features

- Run hooks locally without triggering them via Git.

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
pub fn main() {
    hooksmith::init();
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
