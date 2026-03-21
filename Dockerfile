# ================================
# Stage 1: Build dependencies
# ================================
FROM rust:1.75-slim AS planner

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY apps ./apps

RUN mkdir -p apps/web/src \
    && echo "fn main() {}" > apps/web/src/main.rs \
    && mkdir -p crates/domain/src && echo "" > crates/domain/src/lib.rs \
    && mkdir -p crates/db/src && echo "" > crates/db/src/lib.rs \
    && mkdir -p crates/cache/src && echo "" > crates/cache/src/lib.rs \
    && mkdir -p crates/common/src && echo "" > crates/common/src/lib.rs \
    && mkdir -p crates/analysis/src && echo "" > crates/analysis/src/lib.rs

RUN cargo build --release --package stock-replay-web

# ================================
# Stage 2: Build application
# ================================
FROM rust:1.75-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY apps ./apps

COPY --from=planner /app/target /app/target

RUN cargo build --release --package stock-replay-web

# ================================
# Stage 3: Runtime image
# ================================
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

RUN useradd -r -s /bin/false appuser

COPY --from=builder /app/target/release/stock-replay-web /app/stock-replay-web
COPY --from=builder /app/apps/web/templates /app/templates

RUN chown -R appuser:appuser /app

USER appuser

ENV SERVER_ADDR=0.0.0.0:3000
ENV RUST_LOG=info

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/ || exit 1

CMD ["./stock-replay-web"]
