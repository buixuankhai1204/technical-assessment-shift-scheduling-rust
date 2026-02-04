# Build stage
FROM rust:1.83-slim AS builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY shared ./shared
COPY scheduling-service ./scheduling-service

# Build the application
RUN cargo build --release --bin scheduling-service

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/scheduling-service /app/scheduling-service

# Copy config files
COPY scheduling-service/config /app/config

# Copy migrations
COPY scheduling-service/migrations /app/migrations

# Expose port
EXPOSE 8081

# Run the application
CMD ["/app/scheduling-service"]
