# Build Stage
FROM rust:1.75-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/ftrack
COPY . .

# Build the application
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim

# Install runtime dependencies and Chromium for SPA support
RUN apt-get update && apt-get install -y \
    libssl3 \
    libsqlite3-0 \
    ca-certificates \
    chromium \
    chromium-driver \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from:builder /usr/src/ftrack/target/release/ftrack /usr/local/bin/ftrack

# Set environment variables for Chromium
ENV CHROME_BIN=/usr/bin/chromium
ENV PUPPETEER_SKIP_CHROMIUM_DOWNLOAD=true

ENTRYPOINT ["ftrack"]
