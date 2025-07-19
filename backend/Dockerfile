# Stage 1: build
FROM rust:1.72 AS builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# Stage 2: runtime
FROM debian:buster-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/app/target/release/quantumcoin-backend /usr/local/bin/quantumcoin-backend
CMD ["quantumcoin-backend"]