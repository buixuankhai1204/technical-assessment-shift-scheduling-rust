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
COPY data-service ./data-service

# Build the application
RUN cargo build --release --bin data-service

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/data-service /app/data-service

# Copy config files
COPY data-service/config /app/config

# Copy migrations
COPY data-service/migrations /app/migrations

# Expose port
EXPOSE 8080

# Run the application
CMD ["/app/data-service"]
