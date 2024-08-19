# Set the default shell
set shell := ["bash", "-c"]

default:
  just --list

# Serve the frontend using dioxus-cli
serve:
    dx serve

# Format Rust code
format:
    dx fmt --all-code
    cargo clippy --fix

# Format Dioxus code
dioxus-format:
    dx fmt

# Install required crates
install-deps:
    cargo install dioxus-cli cargo-clippy rustfmt
