# Set the default shell
set shell := ["bash", "-c"]

default:
  just --list

# Serve the frontend using dioxus-cli
serve:
    dx serve

build:
    dx build --fullstack

# Format Rust code
format:
    dx fmt --all-code
    cargo clippy --fix --all-features

# Format Dioxus code
dioxus-format:
    dx fmt

# Install the CLI version matching the Dioxus crates
install-deps:
    cargo install dioxus-cli --version 0.7.10 --locked
