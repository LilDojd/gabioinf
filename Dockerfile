ARG APPNAME=gabioinf

ARG OUTDIR=target/dx/${APPNAME}/release/web

FROM node:24-alpine AS tailwind
WORKDIR /app
COPY . .
RUN npm ci && npx @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css --minify

FROM rustlang/rust:nightly-bookworm AS builder

# Pin the last nightly before rustc's LLVM 23 wasm regression.
RUN rustup toolchain install nightly-2026-08-05 \
      --profile minimal \
      --component clippy,rustfmt \
      --target wasm32-unknown-unknown \
    && rustup default nightly-2026-08-05
RUN cargo install dioxus-cli@0.7.10 --locked
RUN apt-get update && apt-get install -y binaryen
WORKDIR /app
# Copy over the source code and build the project
COPY . .
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
