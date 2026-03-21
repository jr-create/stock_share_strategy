# ================================
# Stage 1: Build dependencies only
# ================================
FROM rust:1.75-slim AS planner

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 只复制 Cargo 配置文件
COPY Cargo.toml Cargo.lock ./

# 创建空的源文件结构（用于缓存依赖）
RUN mkdir -p crates/domain/src \
    && mkdir -p crates/db/src \
    && mkdir -p crates/cache/src \
    && mkdir -p crates/common/src \
    && mkdir -p crates/analysis/src \
    && mkdir -p apps/web/src \
    && mkdir -p tests/src \
    && mkdir -p benches/benches \
    && touch crates/domain/src/lib.rs \
    && touch crates/db/src/lib.rs \
    && touch crates/cache/src/lib.rs \
    && touch crates/common/src/lib.rs \
    && touch crates/analysis/src/lib.rs \
    && echo "fn main() {}" > apps/web/src/main.rs \
    && touch tests/src/lib.rs \
    && touch benches/benches/cache_operations.rs \
    && touch benches/benches/domain_operations.rs

# 构建依赖（缓存层）
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

# 复制 Cargo 配置
COPY Cargo.toml Cargo.lock ./

# 复制真正的源代码
COPY crates ./crates
COPY apps ./apps
COPY benches ./benches
COPY tests ./tests

# 从 planner 复制已编译的依赖
COPY --from=planner /app/target /app/target

# 构建应用（只编译变更的代码）
RUN cargo build --release --package stock-replay-web

# ================================
# Stage 3: Runtime image
# ================================
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
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
