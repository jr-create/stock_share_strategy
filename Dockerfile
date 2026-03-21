FROM rust:1.88
LABEL "language"="rust"

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY . .

RUN rm -f Cargo.lock && cargo build --release -p stock-replay-web

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=0 /app/target/release/stock-replay-web ./app

EXPOSE 8080

CMD ["./app"]
