# Use the official Rust image to build
FROM rust:1.77 AS builder

# Create app dir
WORKDIR /app

# Copy manifest files and build dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -r src

# Copy actual source and build
COPY . .
RUN cargo build --release

# ---

# Use a minimal image for runtime
FROM debian:bullseye-slim

# Install necessary libs
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && apt-get clean

# Copy from builder
COPY --from=builder /app/target/release/photoencoder /usr/local/bin/photoencoder

# Expose web port
EXPOSE 8080

# Start app
CMD ["photoencoder"]
