# Set the default shell
set shell := ["bash", "-c"]

default:
    just --list

# Serve the app with process-scoped secrets
serve:
    secretspec run -- dx serve

# Publish one declared SecretSpec value to Fly without exposing it in argv
[positional-arguments]
publish-fly-secret secret:
    #!/usr/bin/env bash
    set -euo pipefail
    value="$(secretspec get "$1")"
    printf '%s' "$value" | flyctl secrets set "$1=-" --app gabioinf

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
