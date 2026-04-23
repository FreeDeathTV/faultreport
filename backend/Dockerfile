FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y redis-tools ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/faultreport /usr/local/bin/faultreport

EXPOSE 8000
CMD ["faultreport"]