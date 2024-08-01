# Set the default shell
set shell := ["bash", "-c"]

default:
  just --list

# Run the API using cargo-shuttle
api-run: front-build
    RUST_LOG=info cargo shuttle run

# Serve the frontend using dioxus-cli
front-serve:
    cd frontend && dx serve

# Build the frontend
front-build profile="release":
    rm -rf dist frontend/dist
    mkdir dist
    cd frontend && npx tailwindcss -i ./input.css -o ./assets/tailwind.css && dx build --profile {{profile}}
    cp -r frontend/dist/* dist

# Format Rust code
format:
    dx fmt --all-code
    cargo clippy --fix --bin "backend"
    cargo clippy --fix


# Format Dioxus code
dioxusf-format:
    dx fmt

# Install required crates
install-deps:
    cargo install cargo-shuttle dioxus-cli cargo-clippy rustfmt
