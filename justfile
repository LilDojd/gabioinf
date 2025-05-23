# Set the default shell
set shell := ["bash", "-c"]

default:
  just --list

# Serve the frontend using dioxus-cli
serve:
    dx serve

build:
    dx build --platform fullstack

# Format Rust code
format:
    dx fmt --all-code
    cargo clippy --fix --all-features

# Format Dioxus code
dioxus-format:
    dx fmt

# Install required crates
install-deps:
    cargo install dioxus-cli cargo-clippy rustfmt
