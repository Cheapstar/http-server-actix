# ---------- Build Stage ----------
FROM rust:1.77 as builder

# Create app directory
WORKDIR /app

# Copy only manifests first (for caching deps)
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs so that deps can be cached
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev
RUN cargo build --release || true

# Remove dummy files and copy real source
RUN rm -r src
COPY . .

# Build the real app
RUN cargo build --release

# ---------- Runtime Stage ----------
FROM debian:buster-slim

# Install any runtime dependencies
RUN apt-get update && apt-get install -y libssl1.1 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy compiled binary from builder
COPY --from=builder /app/target/release/http-server-actix /app/

# Expose Actix port
EXPOSE 8080

# Run the binary
CMD ["./http-server-actix"]
