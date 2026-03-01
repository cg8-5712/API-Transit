# ---------------------------------------------------------------------------
# Stage 1: Build
# ---------------------------------------------------------------------------
FROM rust:1.82-slim AS builder

WORKDIR /app

# Cache dependencies separately from source for faster rebuilds.
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true

# Build the real source.
COPY src ./src
RUN touch src/main.rs && cargo build --release

# ---------------------------------------------------------------------------
# Stage 2: Runtime
# ---------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# Install CA certificates for HTTPS upstream calls.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

# Run as non-root user.
RUN useradd -r -u 1000 -g nogroup api-transit

WORKDIR /app

COPY --from=builder /app/target/release/api-transit /usr/local/bin/api-transit
RUN chmod +x /usr/local/bin/api-transit

# Data directory for SQLite.
RUN mkdir -p /app/data && chown 1000:nogroup /app/data

USER 1000:nogroup

EXPOSE 8080

CMD ["api-transit"]
