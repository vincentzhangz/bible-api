FROM rust:1.96-slim AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY migrations/ ./migrations/

RUN apt-get update && apt-get install -y pkg-config libpq-dev && rm -rf /var/lib/apt/lists/*
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --shell /bin/bash appuser

COPY --from=builder /app/target/release/bible-api /usr/local/bin/
COPY data/ /app/data/
COPY migrations/ /app/migrations/

USER appuser

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health/live || exit 1

ENTRYPOINT ["bible-api"]
