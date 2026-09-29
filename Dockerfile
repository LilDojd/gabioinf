ARG APPNAME=gabioinf

ARG OUTDIR=target/dx/${APPNAME}/release/web

FROM node:24-alpine AS tailwind
WORKDIR /app
COPY . .
RUN npm ci && npx @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css --minify

FROM rustlang/rust:nightly-bookworm AS builder

WORKDIR /app
COPY rust-toolchain.toml .
RUN rustup toolchain install
RUN cargo install dioxus-cli@0.7.10 --locked
RUN apt-get update && apt-get install -y binaryen
# Copy over the source code and build the project
COPY . .
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN cargo fetch --locked
# Copy tailwind.css we generated earlier
COPY --from=tailwind /app/assets/tailwind.css ./assets/tailwind.css
RUN dx build --release --fullstack

FROM debian:bookworm-slim AS runtime

ARG OUTDIR
ARG APPNAME

WORKDIR /usr/local/bin
RUN apt-get update \
  && apt-get install -y ca-certificates \
  && apt-get clean && update-ca-certificates
COPY --from=builder /app/$OUTDIR /usr/local/bin
COPY --from=builder /app/config /usr/local/bin/config

ENV PORT=8080
ENV IP=0.0.0.0

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/server"] 
