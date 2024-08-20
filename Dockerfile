FROM rust:1.80 AS chef

# Install build tools
RUN cargo install cargo-chef
RUN rustup target add wasm32-unknown-unknown
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall --git https://github.com/dioxuslabs/dioxus dioxus-cli -y
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM node:22-alpine as tailwind
WORKDIR /app
COPY . .
RUN npm install && npx tailwindcss -i ./input.css -o ./assets/tailwind.css


# Cook the dependencies using the recipe prepared earlier
FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
COPY --from=tailwind /app/assets/tailwind.css assets/tailwind.css
RUN cargo chef cook --release --recipe-path recipe.json --features server
# Copy over the source code and build the project
COPY . .
RUN cargo update -p wasm-bindgen --precise 0.2.92 && dx build --release --features web
RUN cargo build --release --features server

FROM debian:1.80-slim-bookworm AS runtime
WORKDIR /app
COPY --from=builder /app/dist /usr/local/bin/dist
COPY --from=builder /app/target/release/gabioinf /usr/local/bin

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/gabioinf"] 
