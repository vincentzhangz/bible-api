FROM rust:latest AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY migrations/ ./migrations/

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/bible-api /usr/local/bin/
COPY data/ /app/data/
COPY migrations/ /app/migrations/

ENTRYPOINT ["bible-api"]
CMD ["--auto-migrate-and-ingest"]
