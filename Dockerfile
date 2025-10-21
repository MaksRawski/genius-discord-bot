FROM rust:1.90 AS builder

WORKDIR /build
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim AS runner
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /build/target/release/genius genius

ENTRYPOINT ["/app/genius"]
