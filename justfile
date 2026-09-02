# Set the default shell
set shell := ["bash", "-c"]

default:
    just --list

# Start local PostgreSQL, then serve with process-scoped secrets
serve:
    devenv processes up app

# Regenerate SQLx's checked-in query cache against local PostgreSQL
prepare-sqlx:
    devenv processes up postgres --detach
    devenv shell prepare-sqlx

# Publish one local SecretSpec value through the Fly provider without exposing it in argv
[positional-arguments]
publish-fly-secret secret:
    #!/usr/bin/env bash
    set -euo pipefail
    secretspec get "$1" --provider local | secretspec set "$1" --provider fly_prod

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
