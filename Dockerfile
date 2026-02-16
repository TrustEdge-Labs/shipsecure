# Build stage
FROM rust:1.88-slim-bookworm AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src target/release/.fingerprint/shipsecure-* target/release/deps/shipsecure-* target/release/deps/libshipsecure-* target/release/shipsecure

# Build application
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
LABEL org.opencontainers.image.source=https://github.com/trustedge-labs/shipsecure

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    wget \
    unzip \
    git \
    bash \
    openssl \
    procps \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Nuclei binary
RUN NUCLEI_VERSION=$(curl -s https://api.github.com/repos/projectdiscovery/nuclei/releases/latest | grep '"tag_name"' | sed 's/.*"v\(.*\)".*/\1/') \
    && wget -q "https://github.com/projectdiscovery/nuclei/releases/download/v${NUCLEI_VERSION}/nuclei_${NUCLEI_VERSION}_linux_amd64.zip" -O /tmp/nuclei.zip \
    && unzip /tmp/nuclei.zip -d /usr/local/bin/ nuclei \
    && chmod +x /usr/local/bin/nuclei \
    && rm /tmp/nuclei.zip

# Install testssl.sh
RUN git clone --depth 1 https://github.com/drwetter/testssl.sh.git /opt/testssl.sh \
    && ln -s /opt/testssl.sh/testssl.sh /usr/local/bin/testssl.sh \
    && chmod +x /opt/testssl.sh/testssl.sh

# Create non-root user
RUN useradd -m -u 1000 -U -s /bin/bash shipsecure

# Copy artifacts from builder
COPY --from=builder /app/target/release/shipsecure /usr/local/bin/
COPY --from=builder /app/migrations /app/migrations

# Copy templates directory (for vibe-code scanning)
COPY templates /app/templates

# Copy fonts directory (for PDF generation)
COPY fonts /app/fonts

# Set ownership and switch to non-root user
RUN chown -R shipsecure:shipsecure /app
USER shipsecure

# Set scanner environment defaults
ENV NUCLEI_BINARY_PATH=/usr/local/bin/nuclei
ENV TESTSSL_BINARY_PATH=/usr/local/bin/testssl.sh
ENV SHIPSECURE_TEMPLATES_DIR=/app/templates/nuclei

WORKDIR /app
EXPOSE 3000
CMD ["shipsecure"]
