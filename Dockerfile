FROM rust:nightly-slim as builder
LABEL "language"="rust"

WORKDIR /app

# 安装依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 复制整个项目（包括 workspace 结构）
COPY . .

# 构建应用
RUN cargo build --release -p stock-replay-web

# 运行时镜像
FROM debian:bookworm-slim

WORKDIR /app

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 从构建镜像复制二进制文件
COPY --from=builder /app/target/release/stock-replay-web ./app

EXPOSE 8080

CMD ["./app"]
