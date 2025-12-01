# Build Stage
FROM rust:1-bookworm as builder

WORKDIR /app
COPY . .

# Build the TUI example
RUN cargo build --release --example 07_terminal_ui

# Runtime Stage
FROM debian:bookworm-slim

# Install OpenSSL (required for HTTPS/WSS)
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/examples/07_terminal_ui /app/kraken_tui

# Run the TUI
CMD ["./kraken_tui"]
