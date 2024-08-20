ARG APPNAME=gabioinf

ARG OUTDIR=dist

FROM rust:bookworm AS chef

# Install build tools
RUN cargo install cargo-chef
RUN rustup target add wasm32-unknown-unknown
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall --git https://github.com/dioxuslabs/dioxus dioxus-cli --locked -y
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM node:22-alpine as tailwind
WORKDIR /app
COPY . .
RUN npm install && npx tailwindcss -i ./input.css -o ./public/tailwind.css


# Cook the dependencies using the recipe prepared earlier
FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
COPY --from=tailwind /app/public/tailwind.css public/tailwind.css
RUN cargo chef cook --release --recipe-path recipe.json --features server
RUN cargo chef cook --release --recipe-path recipe.json --features web --target wasm32-unknown-unknown
# Copy over the source code and build the project
COPY . .
RUN cargo update -p wasm-bindgen --precise 0.2.92 && dx build --release --platform fullstack
# RUN cargo build --release --features server

FROM debian:bookworm-slim AS runtime

ARG OUTDIR
ARG APPNAME

WORKDIR /usr/local/bin
RUN apt-get update && apt-get install -y openssl && apt-get clean
COPY --from=builder /app/$OUTDIR /usr/local/bin/
COPY --from=builder /app/config /usr/local/bin/config

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/server"] 
