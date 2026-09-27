FROM rust:1-bookworm AS builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends binaryen \
    && rm -rf /var/lib/apt/lists/*
RUN rustup target add wasm32-unknown-unknown
RUN cargo install dioxus-cli --version 0.7.10 --locked

WORKDIR /app
COPY . .
RUN dx build --web --release --bin tetramaster-web

FROM caddy:2-alpine

COPY Caddyfile /etc/caddy/Caddyfile
COPY --from=builder /app/target/dx/tetramaster-web/release/web/public /srv

EXPOSE 3000
