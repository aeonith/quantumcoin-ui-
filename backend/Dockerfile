# Use official Rust image as the base
FROM rust:1.72 as builder

# Create app directory inside container
WORKDIR /app

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Start a new stage with minimal dependencies
FROM debian:buster-slim

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/backend /usr/local/bin/backend

# Expose port your Rust app runs on (update as needed)
EXPOSE 8000

# Run the binary
CMD ["backend"]