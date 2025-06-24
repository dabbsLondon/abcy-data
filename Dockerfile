# Build stage
FROM rust:1 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim
COPY --from=builder /app/target/release/abcy-data /usr/local/bin/abcy-data
CMD ["abcy-data"]
