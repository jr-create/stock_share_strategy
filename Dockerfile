FROM rust:latest AS builder
LABEL "language"="rust"

WORKDIR /src

COPY . .

RUN cargo build --release -p stock-replay-web

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /src/target/release/stock-replay-web /app/stock-replay-web

EXPOSE 8080

CMD ["/app/stock-replay-web"]
