# Build stage
FROM rust:1.91-slim as builder

WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM gcr.io/distroless/cc-debian12:nonroot

# Copy CA certificates from builder stage for HTTPS requests
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# Copy the binary from builder stage
COPY --from=builder /app/target/release/solis-delay /usr/local/bin/solis-delay

ENTRYPOINT ["/usr/local/bin/solis-delay"]

