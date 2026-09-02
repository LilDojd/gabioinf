ARG APPNAME=gabioinf

ARG OUTDIR=target/dx/${APPNAME}/release/web

FROM rustlang/rust:nightly-bookworm AS chef

# Pin the last nightly before rustc's LLVM 23 wasm regression.
RUN rustup toolchain install nightly-2026-08-05 \
      --profile minimal \
      --component clippy,rustfmt \
      --target wasm32-unknown-unknown \
    && rustup default nightly-2026-08-05
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall cargo-chef@0.1.78 --locked -y
RUN cargo binstall dioxus-cli@0.7.10 --locked -y
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM node:24-alpine AS tailwind
WORKDIR /app
COPY . .
RUN npm ci && npx @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css --minify


# Cook the dependencies using the recipe prepared earlier
FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --features server
RUN cargo chef cook --release --recipe-path recipe.json --features web --target wasm32-unknown-unknown
RUN apt-get update && apt-get install -y binaryen
# Copy over the source code and build the project
# Note that we control profiles for server and client by ./cargo/cargo.toml
COPY . .
# Copy tailwind.css we generated earlier
COPY --from=tailwind /app/assets/tailwind.css ./assets/tailwind.css
RUN dx build --release --fullstack

FROM debian:bookworm-slim AS runtime

ARG OUTDIR
ARG APPNAME

WORKDIR /usr/local/bin
RUN apt-get update \
  && apt-get install -y libssl-dev pkg-config ca-certificates \
  && apt-get clean && update-ca-certificates
COPY --from=builder /app/$OUTDIR /usr/local/bin
COPY --from=builder /app/config /usr/local/bin/config

ENV PORT=8080
ENV IP=0.0.0.0

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/server"] 
