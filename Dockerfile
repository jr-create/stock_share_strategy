# ================================
# Stage 1: Build stage
# ================================
FROM rust:1.92-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY . .

RUN cargo build --release --package stock-replay-web

# ================================
# Stage 2: Runtime image
# ================================
FROM debian:bookworm-slim

LABEL "language"="rust"

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

RUN useradd -r -s /bin/false appuser

COPY --from=builder /app/target/release/stock-replay-web /app/stock-replay-web

RUN chown -R appuser:appuser /app

USER appuser

ENV SERVER_ADDR=0.0.0.0:3000
ENV RUST_LOG=info

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/ || exit 1

CMD ["./stock-replay-web"]
