# Stage 1: Build the Rust binary with musl target
FROM rust:1.82 AS builder

# Add musl target for static linking
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app
# Copy manifest files first to cache dependencies if unchanged
COPY Cargo.toml Cargo.lock ./

# Create an empty src directory to avoid build errors if not present yet
RUN mkdir src

# Build dependencies only
RUN cargo build --release --target x86_64-unknown-linux-musl || true

# Remove dummy src and copy real source code
RUN rm -rf src
COPY ./src ./src

# Install musl-tools to link statically
RUN apt-get update && apt-get install -y musl-tools

# Build the actual release binary statically
RUN cargo build --release --target x86_64-unknown-linux-musl
# Stage 2: Create a minimal runtime image
FROM alpine:latest

# Copy the statically linked binary from builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/backend /usr/local/bin/backend

# Expose port if your app listens on a port (adjust if needed)
EXPOSE 8080

# Run the binary
CMD ["/usr/local/bin/backend"]
