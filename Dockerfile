FROM rust:1.75-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./

COPY crates/domain/Cargo.toml ./crates/domain/
COPY crates/db/Cargo.toml ./crates/db/
COPY crates/cache/Cargo.toml ./crates/cache/
COPY crates/common/Cargo.toml ./crates/common/
COPY crates/analysis/Cargo.toml ./crates/analysis/
COPY apps/web/Cargo.toml ./apps/web/
COPY benches/Cargo.toml ./benches/
COPY tests/Cargo.toml ./tests/

RUN mkdir -p crates/domain/src crates/db/src crates/cache/src crates/common/src crates/analysis/src \
    && mkdir -p apps/web/src benches/benches tests/src \
    && echo "pub fn dummy() {}" > crates/domain/src/lib.rs \
    && echo "pub fn dummy() {}" > crates/db/src/lib.rs \
    && echo "pub fn dummy() {}" > crates/cache/src/lib.rs \
    && echo "pub fn dummy() {}" > crates/common/src/lib.rs \
    && echo "pub fn dummy() {}" > crates/analysis/src/lib.rs \
    && echo "fn main() {}" > apps/web/src/main.rs \
    && echo "fn main() {}" > benches/benches/main.rs \
    && echo "pub fn dummy() {}" > tests/src/lib.rs

RUN cargo build --release --package stock-replay-web

COPY crates ./crates
COPY apps ./apps
COPY benches ./benches
COPY tests ./tests

RUN touch crates/domain/src/lib.rs \
    && touch crates/db/src/lib.rs \
    && touch crates/cache/src/lib.rs \
    && touch crates/common/src/lib.rs \
    && touch crates/analysis/src/lib.rs \
    && touch apps/web/src/main.rs

RUN cargo build --release --package stock-replay-web

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -r -s /bin/false appuser

COPY --from=builder /app/target/release/stock-replay-web /app/stock-replay-web
COPY --from=builder /app/apps/web/templates /app/templates

RUN chown -R appuser:appuser /app

USER appuser

ENV SERVER_ADDR=0.0.0.0:3000
ENV RUST_LOG=info,stock_replay_web=debug

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/ || exit 1

CMD ["./stock-replay-web"]
