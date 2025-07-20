# Start from an official Rust image with Debian Bullseye (stable & supported)
FROM rust:1.72-bullseye AS builder

# Create app directory
WORKDIR /usr/src/app

# Copy project files
COPY . .

# Build in release mode
RUN cargo build --release

# Final runtime image
FROM debian:bullseye-slim

# Install only the necessary runtime dependencies
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder
COPY --from=builder /usr/src/app/target/release/quantumcoin-backend /usr/local/bin/quantumcoin-backend

# Expose port (match your backend config, e.g. Rocket or Actix)
EXPOSE 8000

# Launch the app
CMD ["quantumcoin-backend"]