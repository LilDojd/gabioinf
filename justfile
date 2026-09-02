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

# Create a validated draft
[positional-arguments]
new-post slug:
    #!/usr/bin/env bash
    set -euo pipefail
    slug="$1"
    [[ "$slug" =~ ^[a-z0-9]+(-[a-z0-9]+)*$ ]] || { echo "slug must use lowercase words separated by hyphens" >&2; exit 1; }
    post="content/blog/$slug.md"
    [[ ! -e "$post" ]] || { echo "$post already exists" >&2; exit 1; }
    cat > "$post" <<EOF
    ---
    title: "Replace with the article title"
    description: "Replace with a concise description of at least twenty characters."
    published: $(date +%F)
    draft: true
    tags: []
    ---

    Start writing here. Use heading level 2 and below inside posts.
    EOF
    echo "Created $post"

check-posts:
    SQLX_OFFLINE=true cargo test --all-features blog

# Format Rust code
format:
    cargo fmt --all

# Install the CLI version matching the Dioxus crates
install-deps:
    cargo install dioxus-cli --version 0.7.10 --locked
