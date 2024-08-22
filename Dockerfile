ARG APPNAME=gabioinf

ARG OUTDIR=dist

FROM rust:bookworm AS chef

# Install build tools
RUN rustup target add wasm32-unknown-unknown
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall cargo-chef -y
RUN cargo install --git https://github.com/dioxuslabs/dioxus dioxus-cli --rev 851abe8 --locked
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
RUN cargo chef cook --release --recipe-path recipe.json --features server
RUN cargo chef cook --release --recipe-path recipe.json --features web --target wasm32-unknown-unknown
# Copy over the source code and build the project
# Note that we control profiles for server and client by ./cargo/cargo.toml
COPY . .
# Copy tailwind.css we generated earlier
COPY --from=tailwind /app/public/tailwind.css ./public/tailwind.css
RUN dx build --platform fullstack --release

FROM debian:bookworm-slim AS runtime

ARG OUTDIR
ARG APPNAME

WORKDIR /usr/local/bin
RUN apt-get update \
  && apt-get install -y libssl-dev pkg-config ca-certificates \
  && apt-get clean && update-ca-certificates
COPY --from=builder /app/$OUTDIR /usr/local/bin
COPY --from=builder /app/config /usr/local/bin/config

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/gabioinf"] 
