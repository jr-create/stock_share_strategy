FROM rust:1.75-slim as builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY apps ./apps
COPY benches ./benches
COPY tests ./tests

RUN cargo build --release --package stock-replay-web

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/stock-replay-web /app/stock-replay-web
COPY --from=builder /app/apps/web/templates /app/templates

ENV SERVER_ADDR=0.0.0.0:3000
ENV RUST_LOG=info

EXPOSE 3000

CMD ["./stock-replay-web"]
