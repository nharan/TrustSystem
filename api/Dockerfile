FROM rust:1.79 as builder
WORKDIR /app
# Expect build context at repo root
COPY Cargo.toml ./
COPY api/Cargo.toml api/Cargo.toml
COPY workers/Cargo.toml workers/Cargo.toml
RUN mkdir -p api/src workers/src && \
    echo 'fn main(){}' > api/src/main.rs && \
    echo 'fn main(){}' > workers/src/main.rs && \
    cargo build -p trustsystem-api --release || true
COPY . .
RUN cargo build -p trustsystem-api --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /srv
COPY --from=builder /app/target/release/trustsystem-api /usr/local/bin/api
ENV RUST_LOG=info
EXPOSE 8080
CMD ["/usr/local/bin/api"]


