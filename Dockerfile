# QuantumCoin Docker Configuration
FROM rust:1.70 as builder

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml ./
COPY src/ ./src/

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the built application
COPY --from=builder /app/target/release/quantumcoin-integrated /usr/local/bin/

# Copy web assets
COPY *.html /app/web/
COPY *.js /app/web/
COPY *.css /app/web/

# Set working directory
WORKDIR /app

# Expose ports
EXPOSE 8332 8333

# Default command
CMD ["quantumcoin-integrated", "node", "--genesis"]
