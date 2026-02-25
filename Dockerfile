# Stage 1: Build
FROM rust:slim AS builder

WORKDIR /app
COPY . .
RUN cargo build --release --bin finplan

# Stage 2: Runtime
FROM ubuntu:24.04

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/finplan /usr/local/bin/finplan

ENTRYPOINT ["finplan"]
